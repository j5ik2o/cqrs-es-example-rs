# Docker Composeでデバッグする

## ビルドイメージ

```shell
$ ./tools/scripts/docker-build-all.sh
--- Using Environments -----------------
AWS_PROFILE = ceer
AWS_REGION = us-east-1
PREFIX = om2eep1k
APPLICATION_NAME = ceer
----------------------------------------
...
```

## Docker Composeを実行する

```shell
$ ./tools/scripts/docker-compose-up.sh
```

## 動作確認

### アプリケーションの確認

以下のコマンドで動作確認を行う。

```shell
$ curl -s -X GET http://localhost:18080/
Hello, Write API!%
```

GraphiQL IDEのページが返ってくればOK。

http://localhost:18082/

APIを呼び出して動作を確認する。

```shell
$ PORT=18080 ./tools/scripts/create-thread.sh
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
curl -X POST -H "Content-Type: application/json" -d '{ "query": "{ getThread(threadId: \"01H541BDRT2XP2QNH93MSPFAMH\"){ id } }" }' http://localhost:18082
{"data":{"getThread":{"id":"01H541BDRT2XP2QNH93MSPFAMH"}}}%
```
