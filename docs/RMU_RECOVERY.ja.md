# Read Model Updater の復旧運用

この文書は、Read Model Updater（RMU）の復旧を本番で動かすための資産と手順を定めます。コード側の復旧仕様は PR #603 で実装済みです。この文書は、SQS の OnFailure ワーカー、定期実行、監視、障害時の手順を扱います。実デプロイは行いません。

## 構成

| リソース | 役割 |
|:--|:--|
| RMU Lambda (`<prefix>-lambda-read-model-updater`) | head Streams を購読し、リードモデルと `read_model_positions` を更新します。イベントを適用できないときは、復旧タスクを `read_model_recovery_tasks` に書きます。 |
| OnFailure キュー (`<prefix>-read-model-updater-dlq`) | Streams の再試行を使い切ったバッチを受けます。復旧ワーカーの入力にもなります。 |
| 復旧ワーカー Lambda (`<prefix>-lambda-read-model-updater-recovery`) | 同じイメージを `APP__RECOVERY_WORKER=true` で動かします。SQS または EventBridge で起動し、1 回の復旧パス（head の全件照合と未処理タスクの再適用）を実行します。 |
| 復旧 DLQ (`<prefix>-read-model-updater-recovery-dlq`) | 復旧ワーカーが上限まで再試行しても処理できなかったメッセージを保持します。 |
| EventBridge ルール (`<prefix>-rmu-recovery-pass`) | 15 分ごとに復旧ワーカーを起動します。通知が届かなくても滞留タスクを処理します。 |
| SNS トピック (`<prefix>-rmu-ops-alerts`) | CloudWatch アラームの通知先です。`rmu_ops_alert_email` を設定するとメールでも受け取れます。 |

復旧タスクの正本は MySQL の `read_model_recovery_tasks` です。SQS は起動のきっかけにすぎず、ワーカーはメッセージ本文を使いません。イベント本文は journal から読み直します。

## 設定値

`tools/deploy/rmu/ops.json` が運用値の正本です。Terraform の `local.rmu_ops` がこのファイルを読み、`applications/read-model-updater/tests/ops_config.rs` が値の整合性を検査します。

| 設定 | 値 | 理由 |
|:--|:--|:--|
| 復旧ワーカーのタイムアウト | 300 秒 | 1 回の全件照合と未処理タスクの再適用に十分な時間です。 |
| 復旧キューの可視性タイムアウト | 1800 秒 | ワーカーのタイムアウトの 6 倍です。処理中のメッセージが別のワーカーに渡りません。 |
| 復旧ワーカーの同時実行 | 1 | 同じタスクを複数のワーカーが処理しません。DB の行ロックでも競合は防げますが、無駄な処理を減らします。 |
| SQS のバッチサイズ | 1 | 1 メッセージで 1 回の復旧パスを実行します。 |
| SQS の最大受信回数 | 5 | 一時障害を数回再試行し、越えたら復旧 DLQ に退避します。 |
| 復旧パスの定期実行 | 15 分ごと | SQS 通知がなくても未処理タスクを処理します。 |
| RMU Lambda のタイムアウト | 60 秒 | ストリーム 1 バッチの部分バッチ応答を返すのに十分です。 |
| RMU の再試行 | 5 回 / 30 秒 | 一時障害の再試行上限です（コードの既定値）。 |

復旧ワーカーの同時実行を 1 にすると、SQS の起動と定期実行が重なったときの片方はスロットルされます。スロットルが続くとメッセージは復旧 DLQ に移りますが、復旧タスクは MySQL に残るため、定期実行が処理を続けます。復旧ワーカーは CloudWatch にメトリクスを送るため、Lambda のセキュリティグループから NAT 経由の TCP 443 を許可しています。

## 運用手順

### SQS OnFailure キューにメッセージが入ったとき

OnFailure キューには、Streams の再試行を使い切ったバッチのメタデータが入ります。メッセージ本文は元のイベントではありません。復旧タスクの有無は MySQL で確認します。

1. キューの深さとメッセージの受信回数を確認します。

```sh
aws sqs get-queue-attributes \
  --queue-url "$(terraform -chdir=tools/deploy/terraform output -raw read_model_updater_on_failure_queue_url)" \
  --attribute-names ApproximateNumberOfMessages ApproximateAgeOfOldestMessage
```

2. `read_model_recovery_tasks` の未処理タスクを確認します。

```sql
SELECT store_id, projection_name, projection_version, aggregate_id, seq_nr,
       failure_type, error_message, status
FROM read_model_recovery_tasks
WHERE status = 'pending'
ORDER BY created_at;
```

`failure_type` は `event_payload`、`unsupported_event`、`journal_gap`、`read_model_constraint`、`unknown` のいずれかです。

3. 原因を取り除きます。journal にイベントがあるか、リードモデルの制約違反が解消したかを確認します。

4. 復旧ワーカーを手動で 1 回起動します。メッセージ本文を使わないため、空のイベントで未処理タスクを再生できます。

```sh
aws lambda invoke \
  --function-name "$(terraform -chdir=tools/deploy/terraform output -raw read_model_updater_recovery_function_name)" \
  --payload '{"Records":[]}' \
  --cli-binary-format raw-in-base64-out \
  /tmp/recovery-worker-response.json
jq . /tmp/recovery-worker-response.json
```

5. 対象の `status` が `completed` になり、`read_model_positions` が対象の `seq_nr` 以上に進んだことを確認します。

```sql
SELECT status FROM read_model_recovery_tasks
WHERE aggregate_id = ? AND projection_name = 'group-chat-read-model' AND seq_nr = ?;

SELECT seq_nr FROM read_model_positions WHERE aggregate_id = ?;
```

### 復旧 DLQ にメッセージが移ったとき

復旧 DLQ のメッセージは、復旧ワーカーが上限まで再試行しても処理できなかったものです。メッセージ本文は起動のきっかけであり、そのタスクがまだ `pending` とは限りません。復旧が完了したタスクの通知が残っていることもあります。

1. `read_model_recovery_tasks` の `status = 'pending'` を確認し、`failure_type` と `error_message` で原因を見ます。
2. 原因を取り除きます。
3. 復旧ワーカーをもう一度実行します。未処理タスクが残ると、ワーカーのログに `recovery task remains unresolved` が出ます。
4. 対象の `status` が `completed` になり、位置が進んだことを確認します。

メッセージを消すだけでタスクを直さないと、次の定期実行でも同じタスクが残ります。メッセージ本文だけを根拠にしないでください。

### 再同期が失敗したとき

`<prefix>-rmu-resync-failures` アラームか、復旧ワーカーの `resync did not complete` ログで気づきます。head の全件 Scan が失敗したか、journal に欠番があります。

- 一時的な失敗（DynamoDB のスロットリング、ネットワーク断）は、次の定期実行で再試行されます。
- `journal_gap` が続く場合は、journal の `aid` と `seq_nr` を確認し、欠けているイベントを復元します。復旧ワーカーは位置を進めずに失敗します。
- 24 時間を超えて購読が止まった場合は、head の全件 Scan と journal の補完で追いつきます。head テーブルが大きい場合は、定期実行の間隔を調整してください。

### ローカルで復旧を実行する

Lambda を使わずに同じ処理を実行する場合は、`recover` バイナリを使います。設定はローカル RMU と同じ `APP__DATABASE__URL`、`APP__AWS__*`、`APP__STREAM__*` を渡します。

```sh
cargo run -p read-model-updater --bin recover
```

復旧パスは冪等です。同じタスクを 2 回実行しても、リードモデルは二重になりません。

## 監視

次の CloudWatch アラームを Terraform で定義します。通知先は SNS トピック `<prefix>-rmu-ops-alerts` です。

| アラーム | メトリクス | しきい値 | 意味 |
|:--|:--|:--|:--|
| `<prefix>-rmu-stream-errors` | `AWS/Lambda Errors`（RMU） | 5 分間に 1 回以上 | ストリームの更新が失敗しています。 |
| `<prefix>-rmu-stream-throttles` | `AWS/Lambda Throttles`（RMU） | 5 分間に 1 回以上 | 同時実行の上限に当たっています。 |
| `<prefix>-rmu-recovery-worker-errors` | `AWS/Lambda Errors`（復旧ワーカー） | 5 分間に 1 回以上 | 復旧パスが失敗しています。 |
| `<prefix>-rmu-on-failure-queue-depth` | `AWS/SQS ApproximateNumberOfMessagesVisible`（OnFailure） | 1 以上が 2 期間 | 失敗したバッチがたまっています。 |
| `<prefix>-rmu-on-failure-queue-age` | `AWS/SQS ApproximateAgeOfOldestMessage`（OnFailure） | 3600 秒超が 2 期間 | 失敗したバッチが長く残っています。 |
| `<prefix>-rmu-recovery-dlq-depth` | `AWS/SQS ApproximateNumberOfMessagesVisible`（復旧 DLQ） | 1 以上 | 再試行の上限を超えました。復旧 DLQ の手順に進みます。 |
| `<prefix>-rmu-pending-recovery-tasks` | `CqrsEsExample/Rmu RecoveryTasksPending` | 0 超（復旧パス 1 回） | 復旧タスクが滞留しています。 |
| `<prefix>-rmu-failed-recovery-tasks` | `CqrsEsExample/Rmu RecoveryTasksFailed` | 0 超（復旧パス 1 回） | 復旧パスがタスクを適用できていません。 |
| `<prefix>-rmu-resync-failures` | `CqrsEsExample/Rmu ResyncFailed` | 0 超（復旧パス 1 回） | 全件照合で適用できない集約があります。 |
| `<prefix>-rmu-recovery-schedule-failures` | `AWS/Events FailedInvocations`（定期実行） | 1 以上 | 定期実行が復旧ワーカーを起動できていません。 |

復旧ワーカーは `CqrsEsExample/Rmu` 名前空間に `RecoveryTasksPending`、`RecoveryTasksFailed`、`RecoveryTasksCompleted`、`ResyncFailed` を、`Service=read-model-updater` のディメンションで出します。メトリクスは復旧パスを実行したときだけ出るため、カスタムメトリクスのアラームは欠測時に状態を変えません（`ignore`）。

`rmu_ops_alert_email` を設定すると、SNS のメール購読が作られます。実際の通知先をメール以外にする場合は、SNS トピックに購読を追加してください。

## 残課題

- 実際の AWS 環境へのデプロイと、SNS の通知先の確定。この PR では Terraform の定義と手順までです。
- MySQL のパスワードは Terraform の出力から環境変数で渡しています。本番では Secrets Manager か SSM Parameter Store の参照に置き換えてください。
- 復旧パスの定期実行は 15 分ごとです。head テーブルが大きい場合は、全件 Scan の負荷を見て間隔を延ばしてください。
- 復旧ワーカーの同時実行を 1 に固定しています。1 回のパスで処理しきれないほどタスクが多い場合は、1 回の処理件数の上限を設けてから同時実行を増やしてください。
- `terraform validate` はプロバイダとモジュールの取得が必要なため、このリポジトリの CI では実行していません。`ops.json` の値はテストで検査しています。
