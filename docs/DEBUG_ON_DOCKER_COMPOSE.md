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

イメージ名は固定で `ceer-write-api-server` です。

## Docker Composeを実行する

```shell
$ ./tools/scripts/docker-compose-up.sh
```

## 動作確認

### アプリケーションの確認

以下のコマンドで動作確認を行う。

```shell
$ curl -s -X GET http://localhost:18080/hello
Hello World！
```

APIを呼び出して動作を確認する。

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ {"accountId"： \"01G41J1A2GVT5HE45AH7GP711P\" }" http://127.0.0.1:18080/threads
{"threadId":"01GBRWPCHEZKHX8QCR3226AGAM"}
```


