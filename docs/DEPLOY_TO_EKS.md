# EKSへのデプロイ

まず、Docker for MacのKubernetesオプションを有効にします（Enable Kubernetes）。
Docker for Macのリソース設定も確認してください。十分なリソースを与える必要があります。

## Dockerイメージをプッシュする

Dockerのローカルリポジトリにイメージをプッシュしてください。

```shell
ceer-root $ ./tools/scripts/docker-build-push.sh
```

## Helmfile の設定ファイルを編集します。

シェル
ceer-root $ vi ./tools/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml
ceer-root # tools/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml

```

コンソールに表示されるタグの値に注目してください。
yamlファイルの以下の項目を適切に設定してください。

- writeApi.writeApiServer.image.repository
- writeApi.writeApiServer.image.tag

## アプリケーションのデプロイ

次にバックエンドのロールをデプロイします。

```shell
tools/scripts $ ./helmfile-apply-eks.sh
```

クラスタが形成されるまでしばらく待ちます。ログにエラーがないことを確認してください。

```shell
$ stern 'write-api-server-*' -n adceet
```

すべてのPodがReady状態になっていることを確認する。

## アプリケーションのチェック

フロントエンドが起動したら、以下のコマンドで動作を確認する。

```shell
$ curl -X GET https://xxxxxx/hello
Hello World！
```

APIを呼び出して動作を確認する。

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ \"accountId： \"01G41J1A2GVT5HE45AH7GP711P\" }" https://xxxxxx/threads
{"threadId":"01GBCN25M496HB4PK9EWQMH28J"}
```

