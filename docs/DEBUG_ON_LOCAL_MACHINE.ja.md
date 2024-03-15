# ローカルマシンでデバッグ

## docker-composeにてデータベースだけを実行する。

```shell
$ makers docker-compose-up-db
```

## IntelliJ IDEAを使ってデバッグする。

アプリケーションは動作していないので必要に応じて起動してデバッグしてください。

- write-api-server
    - `src/main.rs`
- read-model-updater
    - `src/main.rs`はAWS Lambda用なので、ローカルでは`bin/local.rs`を使います。
- read-api-server
    - `src/main.rs`

環境変数を指定せずに起動した場合は、config/以下の設定ファイルの値で起動します。HTTPのポートはdocker-compose時と同じ番号が指定されていますので、以下の動作確認コマンドがそのまま使えます。

## 動作確認

```shell
$ makers verify-group-chat
```
