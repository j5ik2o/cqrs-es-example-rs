# NOTE: AWS環境の構築手順。今回は対象外。

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
event_sourcing_enabled = true
event_sourcing_journal_name      = "journal"
event_sourcing_journal_gsi_name  = "jounral-aid-index"
event_sourcing_snapshot_name     = "snapshot"
event_sourcing_snapshot_gsi_name = "snapshot-aid-index"

eks_enabled = true
eks_version = "1.71"
eks_auth_roles = []
eks_auth_users = []
eks_auth_accounts = []

ecr_enabled = true

datadog-api-key = "xxxx"

read_model_updater_tag     = "9ed584699fe19cab82121fae2d4ac7f1eee2e49089ba463cdd7378085ccc7b39-amd64"
```

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