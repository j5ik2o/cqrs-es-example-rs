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

### アプリケーションの確認

以下のコマンドで動作確認を行う。

```shell
$ curl -s -X GET http://localhost:8080/
Hello, Write API!%
```

GraphiQL IDEのページが返ってくればOK。

http://localhost:8082/

APIを呼び出して動作を確認する。

```shell
$ ./tools/scripts/create-thread.sh
{"Success":{"id":{"value":"01H541BDRT2XP2QNH93MSPFAMH"}}}
```

GraphiQL IDEから以下のクエリを実行する。threadIdに上記コマンドで作成したID値を指定してください。

```graphql
{
  getThread(threadId: "01H541BDRT2XP2QNH93MSPFAMH") {
    id
    name
    ownerId
    createdAt
  }
}
```

以下のようなレスポンスが返ってくればOK。

```graphql
{
  "data": {
    "getThread": {
      "id": "01H541BDRT2XP2QNH93MSPFAMH",
      "name": "test",
      "ownerId": "01H4J5WDZDXYJ4NWRDT5AR1J6E",
      "createdAt": "2023-07-12T03:12:10"
    }
  }
}
```

curlで実行する場合は以下のようになります。

```shell
curl -X POST -H "Content-Type: application/json" -d '{ "query": "{ getThread(threadId: \"01H541BDRT2XP2QNH93MSPFAMH\"){ id } }" }' http://localhost:8082
{"data":{"getThread":{"id":"01H541BDRT2XP2QNH93MSPFAMH"}}}%
```
