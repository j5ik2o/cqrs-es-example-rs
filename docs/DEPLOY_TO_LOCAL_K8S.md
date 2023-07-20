以下の機能はまだ準備できていません。

# Kubernetesへのデプロイ（Docker for Macの場合）

まず、Docker for MacのKubernetesオプションを有効にします(Enable Kubernetes)。
Docker for Macのリソース設定も確認してください。十分なリソースを与える必要があります。

## Dockerイメージをプッシュする

Dockerのローカルリポジトリにイメージをプッシュしてください。

```shell
ceer-root $ ./tools/scripts/docker-build-all.sh
```

## Helmfile の設定ファイルを編集する。

```shell
ceer-root $ vi ./tools/config/environments/${PREFIX}-${APPLICATION_NAME}-local.yaml
ceer-root # tools/config/environments/${PREFIX}-${APPLICATION_NAME}-local.yaml
```

コンソールに表示されるタグの値に注目してください。
yamlファイルの以下の項目を適切に設定してください。

- writeApi.writeApiServer.image.repository
- writeApi.writeApiServer.image.tag
- readApiServer.image.repository
- readApiServer.image.tag
- readModelUpdater.image.repository
- readModelUpdater.image.tag

---

**NOTE**

すべてのコンポーネントは以下のコマンド1つでデプロイできますが、各ステップを少なくとも1回は実行して、プロセスの感触をつかむことをお勧めします
を実行することを推奨する。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-all.sh
```

---

## DynamoDBタブを準備する

次にdynamodb localをデプロイします。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-dynamodb.sh
```

必要なテーブルを作成する。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-dynamodb-setup.sh
```

DynamoDB Adminを使用する場合は `http://127.0.0.1:31567/` を開く。

## MySQLタブを準備する。

次にmysqlをデプロイする。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-mysql.sh
```

必要なテーブルを作成する。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-flyway.sh
```

## アプリケーションのデプロイ

次にバックエンドロールをデプロイします。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local.sh
```

クラスタが形成されるまでしばらく待ちます。ログにエラーがないことを確認する。

```shell
$ stern 'write-api-server-*' -n adceet
```

すべてのPodがReady状態になっていることを確認する。

## Read Model Updaterをデプロイする。

次に Read Model Updater をデプロイします。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-read-model-updater-local.sh
```

しばらく待ちます。ログにエラーがないことを確認してください。

```shell
$ stern 'read-model-updater-*' -n adceet
```

## 次にRead API Serverをデプロイする。

次にRead API Serverをデプロイする。

```shell
ceer-ルート $ ./tools/scripts/helmfile-apply-local-read-api.sh
```

しばらく待つ。ログにエラーがないことを確認する。

```shell
$ stern 'read-api-server-*' -n adceet
```

## アプリケーションの動作チェック

フロントエンドが起動したら、以下のコマンドで動作を確認する。

```shell
$ curl -X GET http://localhost:30031/hello
Hello World！
```

APIを呼び出して動作を確認する。

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ \"accountId： \"01G41J1A2GVT5HE45AH7GP711P\" }" http://127.0.0.1:30031/threads
{"threadId":"01GBCN25M496HB4PK9EWQMH28J"}
```

**注：ローカル環境では、最初のイベントはうまく消費されないかもしれない。このような場合は、もう一度コマンドを送信してください。
を再度送信してみてください。

RMUとRead API Serverを使用している場合は、以下のコマンドを実行する。

```shell
$ curl -v -H "Content-Type: application/json" http://127.0.0.1:30033/threads?owner_id=01G41J1A2GVT5HE45AH7GP711P
[{"id":"01GG72CT9B62DRMH31F8SQX3H9","owner_id":"01G41J1A2GVT5HE45AH7GP711P","created_at":"2022-10-25T07:58:31.096808590Z"}]%
```
