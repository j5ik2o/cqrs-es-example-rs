# ローカルマシンでデバッグ

## DynamoDB LocalとDynamoDB AdminのDocker Composeを実行する。

docker-composeとしてdynamodb-localとdynamodb-adminを起動します。

```shell
./tools/scripts/docker-compose-up.sh -d
```

## IntelliJ IDEAを使ってデバッグする。

この3つの設定を作成し、IntelliJ IDEAで実行する。デバッグしたい場合は、プロジェクトのどれかを
を実行する。

## 動作確認

### アプリケーションの動作確認

以下のコマンドで動作確認を行う。

```shell
$ curl -s -X GET http://localhost:18080/hello
Hello World！
```

APIを呼び出して動作を確認する。

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ {"accountId"： \"01G41J1A2GVT5HE45AH7GP711P\" }" http://127.0.0.1:18080/threads
{"threadId":"01GBCN25M496HB4PK9EWQMH28J"}
```
