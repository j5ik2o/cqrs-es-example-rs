# Docker Compose上でデバッグする

## ビルドイメージ

```shell
$ makers docker-build-local-all
```

## docker-composeの起動

```shell
$ makers docker-compose-up
```

必要なデータベースとテーブルが作成され、アプリケーションも起動します。
開発目的でデータベースだけを起動したい場合は、`docker-compose-up`ではなく`docker-compose-up-db`を実行してください。

## docker-composeの停止

```shell
$ makers docker-compose-down
```

## 動作確認

```shell
$ makers verify-group-chat
```



