# terraform

terraformの`tfstate`を管理するために専用のS3バケットと、
terraformのロックを実現するためのDynamoDBテーブルを作る必要があります。
以下のコマンドを実行する。

```shell
$ ./create-tf-bucket.sh
$ ./create-lock-table.sh
```

## 初期化

```shell
tools/terraform $ ./terraform-init.sh
```

## デプロイ

- インフラへの変更差分を確認する

```shell
tools/terraform $ ./terraform-plan.sh
```

- インフラへの変更を適用する

```shell
tools/terraform $ ./terraform-apply.sh

# -auto-approveを付けると確認なしで実行します。
# tools/terraform $ ./terraform-apply.sh -auto-approve
```

- インフラを破棄する

```shell
tools/terraform $ ./terraform-destroy.sh 

# -auto-approveを付けると確認なしで実行します。
# tools/terraform $ ./terraform-destroy.sh -auto-approve
```

- ロックを解除するには

```shell
tools/terrform $ terraform force-unlock
```

## kubeconfig

```shell
# ~/.kube/configが出力される
$ ./update-kubeconfig.sh
```

## auth-config-map

```shell
# tools/terraform/eks_aws_auth_config_map
$ ./export-auth-config-map.sh
```

```shell
$ ./apply-auth-config-map.sh
```