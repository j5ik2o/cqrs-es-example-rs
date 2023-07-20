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

## `env.sh.default`を`env.sh`としてコピーし編集する

```shell
$ cp env.sh.default env.sh
```

環境変数のPREFIX、APPLICATION_NAMEを適宜変更する。
個人的な環境を作りたい場合は、PREFIXを変更してください。

# terraform による AWS 環境の構築

`tools/terraform/${PREFIX}/${APPLICATION_NAME}-terraform.tfvars`を以下の内容で新規作成する。
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
```

## ロックテーブルの作成

初回のみ、DynamoDBにterraform用のロックテーブルを作成する。

```shell
ceer-root $ makers terraform-create-lock-table
```

## tfstate用のs3バケットを作る

初回のみ、tfstateを保存するs3バケットを作成する。

```shell
ceer-root $ makers terraform-create-tf-bucket
```

## terraform init

```shell
ceer-root $ makers terraform-init
```

## terraform plan

```shell
ceer-root $ makers terraform-plan
```

## terraform apply

```shell
ceer-root $ makers terraform-apply
```

## Update kubeconfig

以下のコマンドを実行してkubeconfig(`~/.kube/config`)を生成する。

```shell
ceer-root $ makers update-kubeconfig
```

FYI: https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html

コンテキストが切り替わったことを確認する。

```shell
tools/terraform $ kubectl config get-contexts
CURRENT   NAME                                                             CLUSTER                                                          AUTHINFO                                                         NAMESPACE
*         arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet   arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet   arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet
          docker-desktop                                                   docker-desktop                                                   docker-desktop
```

# kubernetes-dashboardの確認

## kubernetes-dashboardへのポートフォワード

```shell
$ DASHBOARD_NS=kubernetes-dashboard
$ export POD_NAME=$(kubectl get pods -n $DASHBOARD_NS -l "app.kubernetes.io/name=kubernetes-dashboard,app.kubernetes.io/instance=kubernetes-dashboard" -o jsonpath="{.items[0].metadata.name}")
$ kubectl -n $DASHBOARD_NS port-forward $POD_NAME 8443:8443
```

## Chromeの設定変更

1. `chrome://flags/#allow-insecure-localhost` を開く
2. `Allow invalid certificates for resources loaded from localhost.` を `Enable` にする
3. Chromeを再起動する
4. `https://localhost:8443` を開く

## トークンを作成する

```shell
$ kubectl -n kubernetes-dashboard create token kubernetes-dashboard
eyJXXXXXXXXXXXXXXXXXXXXXXXXXXxYTZmYzlkOGZkN2M5ZjMifQ.eyJhdWQiOlsiaHR0cHM6Ly9rdWJlcm5ldGVzLmRlZmF1bHQuc3ZjIl0sImV4cCI6MTY4ODY5NzE0MSwiaWF0IjoxNjg4NjkzNTQxLCJpc3MiOiJodHRwczovL29pZGMuZWtzLmFwLW5vcnRoZWFzdC0xLmFtYXpvbmF3cy5jb20vaWQvQjcyNjkwQkY2NEUyQUM5RjA0RkIxRDQxMTZDNkMwMDQiLCJrdWJlcm5ldGVzLmlvIjp7Im5hbWVzcGFjZSI6Imt1YmVybmV0ZXMtZGFzaGJvYXJkIiwic2VydmljZWFjY291bnQiOnsibmFtZSI6Imt1YmVybmV0ZXMtZGFzaGJvYXJkIiwidWlkIjoiMWJlNjViMmYtYmZkMy00NmYwLWE0MjktODlkY2E4Yzk4MWU5In19LCJuYmYiOjE2ODg2OTM1NDEsInN1YiI6InN5c3RlbTpzZXJ2aWNlYWNjb3VudDprdWJlcm5ldGVzLWRhc2hib2FyZDprdWJlcm5ldGVzLWRhc2hib2FyZCJ9.agCS7Zf7WlzFW32lRhNlNHzzvEoGUJALq0X_VaOW9WuWdjhY90tvDzCkul9YGygGwHKTuGMrkDJIfcJf92c3hUpXLu30HYRRBCpth-cC4YUd51Eccm5OJ6HJwuK-HaFO6ZtGfOl5uPO2qJzFyLEHAEePEiH6MVWMcx3BWvDDxFColFleasF1BtkB_64-k-fgaOBAnGM3paTQ8ug4taqsRaisSChLreYYs7zUI7YXnQMJ0LswWQyMOjVC52BfjwqNvP2A7bg8hqOPaO66lN4x89TAD_d0yjKQM3CuXlGKGPKqYtGj7KRvWNjvXN5VNEeQnhv9LYzs6n1SyKD7KsRCWA
```
