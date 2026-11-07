# AWSのためのセットアップ

## AWSプロファイルを追加する

以下のように `~/.aws/credentials` 編集する

```
# ...
[ceer]
aws_access_key_id=xxxxx
aws_secret_access_key=xxxxx
aws_session_token=xxxxx
```

もしくは、AWS SSOの場合は、以下のように `~/.aws/config` 編集する

```
[profile ceer]
sso_start_url = https://xxxxxx.awsapps.com/start
sso_region = ap-northeast-1
sso_account_id = 1234567890 
sso_role_name = xxxxxx
region = ap-northeast-1
```

```shell
$ aws sso login --profile ceer
```

としてSSOにてログインする。

# terraform による AWS 環境の構築

`tools/deploy/terraform/${PREFIX}-${APPLICATION_NAME}-terraform.tfvars`を以下の内容で新規作成する。
variables.tfで定義した変更は、この`tfvars`ファイルで上書きすることができる。
必要なリソースだけを作成することができます。例えば、ecrだけが必要なら`ecr_enabled = true`と設定すればよい。

```
event_sourcing_enabled                     = true
event_sourcing_journal_name                = "journal"
event_sourcing_snapshot_name               = "snapshot"
event_sourcing_snapshot_history_index_name = "snapshot-active-history-index"
event_sourcing_head_name                   = "head"

eks_enabled = true
eks_version = "1.71"
eks_auth_roles = []
eks_auth_users = []
eks_auth_accounts = []

ecr_enabled = true

datadog-api-key = "xxxx"

read_model_updater_tag     = "9ed584699fe19cab82121fae2d4ac7f1eee2e49089ba463cdd7378085ccc7b39-amd64"
```

## 3 テーブル構成への変更と既存データ

ESA v5 では DynamoDB のテーブルが journal / snapshot / head の 3 つになり、キーは `aid` に統一されます。
journal と snapshot のハッシュキーとソートキーが変わるため、既存のテーブルにこの設定を適用すると
Terraform はテーブルを置換します。既存のイベントは残りません。

`event-sourcing/main.tf` の journal には `prevent_destroy = true` を設定しています。置換や削除の前に
Terraform がデータを消さないようにするためです。移行またはリセットを行うときは、次の手順で明示的に外してください。

### 適用前のバックアップ

移行するかリセットするかに関係なく、既存の journal と snapshot のバックアップを取ります。

```shell
aws dynamodb create-backup --table-name <journal> --backup-name <prefix>-journal-before-v5
aws dynamodb create-backup --table-name <snapshot> --backup-name <prefix>-snapshot-before-v5
```

ポイントインタイムリカバリを有効にしている場合は、復元できる時点を確認しておきます。

### データを残す場合の v5 レイアウトへの移行

既存のイベントを残す場合は、新しいテーブルを別名で作り、項目を書き直してから参照先を切り替えます。

1. 新しい物理名を決めます。例は `journal-v5`、`snapshot-v5`、`head` です。
2. 旧テーブルを削除せずに Terraform の管理から外します。
   ```shell
   terraform state rm 'module.event_sourcing.aws_dynamodb_table.journal-table[0]'
   terraform state rm 'module.event_sourcing.aws_dynamodb_table.snapshot-table[0]'
   ```
   旧環境に head はないため、外す必要はありません。
3. `tfvars` の `event_sourcing_*` を新しい名前に変えます。
   ```
   event_sourcing_journal_name                = "journal-v5"
   event_sourcing_snapshot_name               = "snapshot-v5"
   event_sourcing_snapshot_history_index_name = "snapshot-active-history-index"
   event_sourcing_head_name                   = "head"
   ```
4. `makers terraform-apply` で新しい 3 テーブルを作ります。
5. 旧 journal と snapshot の項目を新しいテーブルへ書き直します。`aid` は `${型名}-${値}` の形にし、
   イベントは v5 の封筒（`seq_nr`、`occurred_at`、`manifest`、`payload`）で書きます。ヘッド項目も
   集約ごとに作ります。移行スクリプトはこのサンプルに含みません。ESA の
   `docs/spec/storage/dynamodb.ja.md` の移行手順を参照してください。
6. アプリの参照先を新しい名前に切り替えます。
   - Write API: `APP__PERSISTENCE__JOURNAL_TABLE_NAME`、`APP__PERSISTENCE__SNAPSHOT_TABLE_NAME`、
     `APP__PERSISTENCE__HEAD_TABLE_NAME`、`APP__PERSISTENCE__SNAPSHOT_HISTORY_INDEX_NAME`
   - RMU: `APP__STREAM__HEAD_TABLE_NAME`、`APP__STREAM__JOURNAL_TABLE_NAME`
   - Terraform で管理する Lambda は `event_sourcing_*` の値から環境変数を自動で設定します。
7. 新しいテーブルで Write API と RMU を起動し、リードモデルが更新されることを確認します。
8. 確認が終わってから旧テーブルを削除します。

リードモデル（MySQL）は作り直す必要がありません。処理位置は `read_model_positions` に残るため、
移行したイベントの `seq_nr` と整合するようにしてください。整合しない場合は、リードモデルと
`read_model_positions` を初期化してから RMU を再起動します。

### データをリセットする場合

サンプルのように既存データを残さない場合は、テーブルを作り直します。

1. `event-sourcing/main.tf` の journal の `lifecycle` ブロックから `prevent_destroy = true` を外します。
2. `makers terraform-apply` を実行します。journal と snapshot が置換され、head が作られます。
3. MySQL のリードモデルを初期化します。`group_chats`、`members`、`messages`、`read_model_positions`
   を空にするか、データベースを作り直して refinery のマイグレーションを実行します。古い処理位置が
   残ると、新しいイベントが位置以下と判定されて反映されません。
4. Write API と RMU を再起動します。head の Streams は作り直されるため、Lambda のイベントソース
   マッピングも作り直してください。
5. LocalStack の compose は `docker compose down -v` でテーブルとデータを作り直せるため、この手順は
   不要です。

`prevent_destroy` を外した状態で `makers terraform-destroy` を実行すると、テーブルは削除されます。
削除するときは、対象の環境であることを確認してください。

## terraform init

```shell
$ makers terraform-init
```

初回実行時に、S3バケットやロックテーブルは自動的に作られます。

## terraform plan

```shell
$ makers terraform-plan
```

## terraform apply

```shell
$ makers terraform-apply
```

## Update kubeconfig

以下のコマンドを実行してkubeconfig(`~/.kube/config`)を生成する。

```shell
$ makers update-kubeconfig
```

FYI: https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html

コンテキストが切り替わったことを確認する。

```shell
$ makers k8s-get-contexts
CURRENT   NAME                                                             CLUSTER                                                          AUTHINFO                                                         NAMESPACE
*         arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet   arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet   arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet
          docker-desktop                                                   docker-desktop                                                   docker-desktop
```

# kubernetes-dashboardの確認

## kubernetes-dashboardへのポートフォワード

```shell
$ makers k8s-port-forward-dashboard
```

## Chromeの設定変更

1. `chrome://flags/#allow-insecure-localhost` を開く
2. `Allow invalid certificates for resources loaded from localhost.` を `Enable` にする
3. Chromeを再起動する
4. `https://localhost:8443` を開く

## トークンを作成する

```shell
$ makers k8s-create-dashboard-token
```

# Aurora(MySQL)の接続情報を確認

```shell
$ makers get-aurora-cluster-all-info
[cargo-make] INFO - makers 0.36.12
[cargo-make] INFO - Calling cargo metadata to extract project info
[cargo-make] INFO - Cargo metadata done
[cargo-make] INFO - Execute Command: "/bin/sh" "/var/folders/tn/ppwkg3_s603fs3702lmrsn_80000gn/T/fsio_j6Rd16XDN0.sh"
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = ap-northeast-1
PREFIX           = pah8iobi
APPLICATION_NAME = ceer
----------------------------------------
[cargo-make] INFO - Build File: Makefile.toml
[cargo-make] INFO - Task: get-aurora-cluster-all-info
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: get-aurora-cluster-master-username
MYSQL_USER_NAME = root
[cargo-make] INFO - Running Task: get-aurora-cluster-master-password
MYSQL_USER_PASSWORD = xnLY1KMG14ygUbW2P0Pu
[cargo-make] INFO - Running Task: get-aurora-cluster-endpoint
MYSQL_HOST = pah8iobi-ceer-mysql.cluster-ctywrcabnmgr.ap-northeast-1.rds.amazonaws.com
[cargo-make] INFO - Running Task: get-aurora-cluster-port
MYSQL_PORT = 3306
[cargo-make] INFO - Running Task: get-aurora-cluster-database
MYSQL_DATABASE = ceer
[cargo-make] INFO - Build Done in 10.76 seconds.
```

得たパスワードをcommon.envに設定する。

```shell
MYSQL_USER_NAME=root
MYSQL_USER_PASSWORD=xxxxxxxx
MYSQL_ENDPOINT=${PREFIX}-ceer-mysql.cluster-ctywrcabnmgr.ap-northeast-1.rds.amazonaws.com
MYSQL_PORT=3306
MYSQL_DATABASE=ceer
```