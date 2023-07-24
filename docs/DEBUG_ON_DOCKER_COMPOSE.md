# Docker Compose上でデバッグする

## ビルドイメージ

```shell
$ makers docker-build-local-all
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
$ makers docker-compose-up
```

## 動作確認

### アプリケーションの確認

以下のコマンドで動作確認を行う。

```shell
$ makers curl-get-root
[cargo-make] INFO - makers 0.36.11
[cargo-make] INFO - Calling cargo metadata to extract project info
[cargo-make] INFO - Cargo metadata done
[cargo-make] INFO - Execute Command: "/bin/sh" "/var/folders/tn/ppwkg3_s603fs3702lmrsn_80000gn/T/fsio_P9UT4nidwY.sh"
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = ap-northeast-1
PREFIX           = aht9aa1e
APPLICATION_NAME = ceer
----------------------------------------
[cargo-make] INFO - Build File: Makefile.toml
[cargo-make] INFO - Task: curl-get-root
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: curl-get-root-write-api-server
Note: Unnecessary use of -X or --request, GET is already inferred.
*   Trying 127.0.0.1:18080...
* Connected to localhost (127.0.0.1) port 18080 (#0)
> GET / HTTP/1.1
> Host: localhost:18080
> User-Agent: curl/7.88.1
> Accept: */*
>
< HTTP/1.1 200 OK
< content-type: text/plain; charset=utf-8
< content-length: 17
< date: Wed, 19 Jul 2023 06:07:57 GMT
<
* Connection #0 to host localhost left intact
Hello, Write API!
[cargo-make] INFO - Running Task: curl-get-root-read-api-server
Note: Unnecessary use of -X or --request, GET is already inferred.
*   Trying 127.0.0.1:18082...
* Connected to localhost (127.0.0.1) port 18082 (#0)
> GET / HTTP/1.1
> Host: localhost:18082
> User-Agent: curl/7.88.1
> Accept: */*
>
< HTTP/1.1 200 OK
< content-type: text/plain; charset=utf-8
< content-length: 16
< date: Wed, 19 Jul 2023 06:07:57 GMT
<
* Connection #0 to host localhost left intact
Hello, Read API!
[cargo-make] INFO - Build Done in 1.31 seconds.
```

GraphiQL IDEを開く

```shell
$ makers open-graphiql-ide
```

疎通確認方法は以下の通り。

```shell
$ ./tools/scripts/curl-post-write-api-server-on-local.sh
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
$ curl -X POST -H "Content-Type: application/json" -d '{ "query": "{ getThread(threadId: \"01H541BDRT2XP2QNH93MSPFAMH\"){ id } }" }' http://localhost:18082
{"data":{"getThread":{"id":"01H541BDRT2XP2QNH93MSPFAMH"}}}%
```
