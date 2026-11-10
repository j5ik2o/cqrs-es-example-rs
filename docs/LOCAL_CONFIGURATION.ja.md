# ローカル実行の設定

[English](LOCAL_CONFIGURATION.md)

LocalStack の DynamoDB と MySQL を使う Write API、ローカル Read Model Updater（RMU）、Read API の設定リファレンスです。

## ポートとエンドポイント

| アプリ | Compose のホストポート | コンテナ内部ポート | ホストで直接実行する場合の TOML 値 | GraphQL |
| --- | --- | --- | --- | --- |
| Write API | 18080 | 8080 | 18080 | `http://localhost:18080/query` |
| ローカル RMU | 18081 の割当のみ | 8080 の割当のみ | HTTP 待受なし | なし |
| Read API | 18082 | 8080 | 18082 | `http://localhost:18082/query` |

両 API は `/query` への POST で GraphQL リクエストを受け付け、同じパスへの GET で GraphiQL を表示します。RMU は `head` テーブルの DynamoDB Streams を購読し、journal のイベントを読み、MySQL のリードモデルと `read_model_positions` を更新します。起動時には head を照合します。HTTP や GraphQL は提供しません。Compose の `18081:8080` の割当や RMU の TOML の `api` 節があっても待受は作られません。ローカルのエントリポイントは `bin/local.rs` で、Lambda 用の `src/main.rs` とは異なります。

## 設定の参照元と必須値

各ローダーはプロセスの作業ディレクトリを基準に、任意の `config/<アプリ名>.toml` を読み、その後 `APP__` 環境変数で上書きします。二重アンダースコアは設定の階層を区切ります。ホストで TOML を使う場合の作業ディレクトリはリポジトリルートです。最終 Docker イメージには TOML をコピーしないため、Compose が環境変数で必須値を渡します。

表の「必須」はマージ後の設定に値が必要という意味です。TOML に値があれば環境変数の指定は不要です。「任意」は Rust のフィールドが省略可能、またはコードに既定値があることを示します。TOML の値はファイル上の既定値であり、コード組込みの代替値ではありません。「同じ」は Compose が同じ値を明示的に指定することを示します。「未指定」は Compose がその変数を設定しないことを示します。

### API 設定

Write と Read に適用します。Compose は RMU にも `APP__API__HOST` と `APP__API__PORT` を渡しますが、RMU は使用しません。

| 環境変数 | 意味と必須性 | TOML の値 | Compose の値 |
| --- | --- | --- | --- |
| `APP__API__HOST` | 必須。待受 IP アドレス | `0.0.0.0` | 同じ |
| `APP__API__PORT` | 必須。待受ポート。ホストの公開ポートの割当とは別 | Write `18080`、Read `18082` | `8080` |
| `APP__API__ALLOW_ORIGINS` | 必須。CORS 許可元。環境変数ではカンマ区切り | `http://127.0.0.1:18080`、`http://127.0.0.1:18082` | `http://localhost:18080,http://localhost:18082,http://localhost:8888` |

### Write の永続化設定

すべて必須で、Write だけが使用します。Compose は TOML と同じ値を明示的に指定します。

| 環境変数 | 意味 | TOML と Compose の値 |
| --- | --- | --- |
| `APP__PERSISTENCE__JOURNAL_TABLE_NAME` | DynamoDB のイベント journal テーブル | `journal` |
| `APP__PERSISTENCE__SNAPSHOT_TABLE_NAME` | DynamoDB の snapshot テーブル | `snapshot` |
| `APP__PERSISTENCE__HEAD_TABLE_NAME` | DynamoDB の集約 head テーブル | `head` |
| `APP__PERSISTENCE__SNAPSHOT_HISTORY_INDEX_NAME` | snapshot 履歴の GSI | `snapshot-active-history-index` |
| `APP__PERSISTENCE__SNAPSHOT_INTERVAL` | snapshot を保存するイベント間隔 | `10` |

テーブル名と GSI 名は DynamoDB の初期化設定と一致させます。RMU の head と journal のテーブル名は Write の値と一致させます。

### AWS 設定

Write と RMU に適用します。Read は TOML の `aws` 節を含め、AWS 設定を使用しません。

| 環境変数 | 意味と必須性 | TOML の値 | Compose の値 |
| --- | --- | --- | --- |
| `APP__AWS__REGION_NAME` | 必須。AWS SDK の既定リージョンプロバイダーチェーンで解決できない場合のリージョン | `ap-northeast-1` | ホストの `${AWS_REGION}` を展開 |
| `APP__AWS__ENDPOINT_URL` | 任意。AWS 接続先の上書き。省略時は SDK が接続先を解決 | `http://localhost:4566` | `http://localstack:4566` |
| `APP__AWS__ACCESS_KEY_ID` | 任意。secret と組み合わせて使う明示的なアクセスキー | `x` | 同じ |
| `APP__AWS__SECRET_ACCESS_KEY` | 任意。アクセスキーと組み合わせて使う明示的な secret | `x` | 同じ |

`x` はローカル用のダミー認証情報です。認証情報の両フィールドに値がある場合、アプリは静的な認証情報プロバイダーを設定します。それ以外は SDK の認証情報プロバイダーチェーンを使います。環境変数を未指定にしても、TOML にある値は削除されません。

Compose の起動スクリプトと e2e スクリプトは、ホストの `AWS_REGION` の既定値を `ap-northeast-1`、`AWS_DEFAULT_REGION` の既定値をそのリージョンにします。Compose を直接実行する場合は、変数展開用のホストの `AWS_REGION` が必要です。サービス内の `AWS_REGION: ap-northeast-1` は別の SDK 向け環境変数です。ホストから LocalStack に接続する場合は `localhost:4566`、コンテナからは `localstack:4566` を使います。

### RMU のストリーム設定

ローカル RMU だけが使用します。共有の設定型では `stream` 節は任意ですが、ローカルバイナリでは必須です。節内の head と journal のテーブル名も必須です。他のフィールドには Rust の既定値があります。

| 環境変数 | 意味 | TOML の値または Rust の既定値 | Compose の値 |
| --- | --- | --- | --- |
| `APP__STREAM__HEAD_TABLE_NAME` | Streams の購読と head の走査に使うテーブル | TOML で `head` | `head` |
| `APP__STREAM__JOURNAL_TABLE_NAME` | イベント再生に使う journal テーブル | TOML で `journal` | `journal` |
| `APP__STREAM__MAX_ITEM_COUNT` | Streams の一回の読込で取得する最大件数 | TOML と Rust で `100` | `32` |
| `APP__STREAM__POLL_INTERVAL_MILLIS` | アイドル時のポーリング初期待機時間。ミリ秒 | TOML と Rust で `1000` | 未指定。Rust の既定値 |
| `APP__STREAM__POLL_MAX_INTERVAL_MILLIS` | アイドル時のポーリング最大待機時間。ミリ秒 | Rust で `30000` | 未指定。Rust の既定値 |
| `APP__STREAM__MAX_EVENTS_PER_TRANSACTION` | 一つのトランザクションで確定するイベント数 | Rust で `1` | 未指定。Rust の既定値 |
| `APP__STREAM__RETRY_MAX_ATTEMPTS` | レコードごとの最大試行回数 | Rust で `5` | 未指定。Rust の既定値 |
| `APP__STREAM__RETRY_MAX_ELAPSED_MILLIS` | 再試行の最大経過時間。ミリ秒 | Rust で `30000` | 未指定。Rust の既定値 |
| `APP__STREAM__RETRY_INITIAL_DELAY_MILLIS` | 再試行の初期待機時間。ミリ秒 | Rust で `200` | 未指定。Rust の既定値 |
| `APP__STREAM__RETRY_MAX_DELAY_MILLIS` | 再試行の最大待機時間。ミリ秒 | Rust で `5000` | 未指定。Rust の既定値 |

### リードモデルのデータベース

`APP__DATABASE__URL` は RMU と Read に必須です。両アプリはマイグレーション済みの同じ MySQL データベースに接続します。Write は使用しません。

| 参照元 | 値 |
| --- | --- |
| RMU と Read のホスト用 TOML | `mysql://ceer:ceer@localhost:3306/ceer` |
| RMU と Read の Compose | `mysql://ceer:ceer@mysql-local:3306/ceer` |

URL はユーザー、パスワード、ホスト、ポート、データベース名を含みます。Compose はユーザー `ceer`、パスワード `ceer`、データベース `ceer` を作り、ホストの `3306` に MySQL を公開します。`refinery` がマイグレーションを実行します。マイグレーションサービスの `DATABASE_URL` は、アプリの `APP__DATABASE__URL` とは別の変数です。これらのアプリは TOML の `redis` 節を使用しません。

## 既存の手順

- [ビルドとテスト](BUILD_AND_TEST.ja.md)に共通環境の準備があります。
- [アプリと e2e のイメージビルド](DEBUG_ON_DOCKER_COMPOSE.ja.md#ビルドイメージ)。
- [Compose の起動](DEBUG_ON_DOCKER_COMPOSE.ja.md#docker-composeの起動)。
- [Compose の停止](DEBUG_ON_DOCKER_COMPOSE.ja.md#docker-composeの停止)。
- [RMU 再起動の検証を含む e2e の実行](DEBUG_ON_DOCKER_COMPOSE.ja.md#e2e)。
- [ホストプロセスの実行とデバッグ](DEBUG_ON_LOCAL_MACHINE.ja.md)。

## 設定の根拠

main `37884e12609924db6b1428136627985fb050d6e1` の実装と照合しています。

- [`config/write-api-server.toml`](../config/write-api-server.toml)
- [`config/read-model-updater.toml`](../config/read-model-updater.toml)
- [`config/read-api-server.toml`](../config/read-api-server.toml)
- [`applications/write-api-server/src/main.rs`](../applications/write-api-server/src/main.rs)
- [`applications/read-api-server/src/lib.rs`](../applications/read-api-server/src/lib.rs)
- [`applications/read-api-server/src/main.rs`](../applications/read-api-server/src/main.rs)
- [`applications/read-model-updater/src/lib.rs`](../applications/read-model-updater/src/lib.rs)
- [`applications/read-model-updater/bin/local.rs`](../applications/read-model-updater/bin/local.rs)
- [`applications/write-api-server/Dockerfile`](../applications/write-api-server/Dockerfile)
- [`applications/read-api-server/Dockerfile`](../applications/read-api-server/Dockerfile)
- [`applications/read-model-updater/Dockerfile.local`](../applications/read-model-updater/Dockerfile.local)
- [`modules/command/interface-adaptor-impl/src/controllers.rs`](../modules/command/interface-adaptor-impl/src/controllers.rs)
- [`modules/query/interface-adaptor/src/controllers.rs`](../modules/query/interface-adaptor/src/controllers.rs)
- [`tools/docker-compose/docker-compose-applications.yml`](../tools/docker-compose/docker-compose-applications.yml)
- [`tools/docker-compose/docker-compose-databases.yml`](../tools/docker-compose/docker-compose-databases.yml)
- [`tools/dynamodb-setup/create-tables.sh`](../tools/dynamodb-setup/create-tables.sh)
- [`tools/scripts/docker-compose-up.sh`](../tools/scripts/docker-compose-up.sh)
- [`tools/scripts/docker-compose-e2e-test.sh`](../tools/scripts/docker-compose-e2e-test.sh)
