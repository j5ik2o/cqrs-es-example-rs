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

### ルートパスにアクセスする

```shell
$ makers curl-get-root
[cargo-make] INFO - makers 0.36.12
[cargo-make] INFO - Calling cargo metadata to extract project info
[cargo-make] INFO - Cargo metadata done
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = ap-northeast-1
PREFIX           = pah8iobi
APPLICATION_NAME = ceer
----------------------------------------
[cargo-make] INFO - Build File: Makefile.toml
[cargo-make] INFO - Task: curl-get-root
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: curl-get-root-write-api-server
Hello, Write API!
[cargo-make] INFO - Running Task: curl-get-root-read-api-server
Hello, Read API!
[cargo-make] INFO - Build Done in 1.11 seconds.
```

### グループチャットを作成した後、グループチャットリードモデルを取得する

```shell
$ makers create-and-get-group-chat
[cargo-make] INFO - makers 0.36.12
[cargo-make] INFO - Calling cargo metadata to extract project info
[cargo-make] INFO - Cargo metadata done
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = ap-northeast-1
PREFIX           = pah8iobi
APPLICATION_NAME = ceer
----------------------------------------
[cargo-make] INFO - Build File: Makefile.toml
[cargo-make] INFO - Task: create-and-get-group-chat
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: create-and-get-group-chat
create-group-chat: GROUP_CHAT_ID=01H6SXQH4HK1GRSD7ZYRR23N9R
get-group-chat: ACTUAL_GROUP_CHAT_ID=01H6SXQH4HK1GRSD7ZYRR23N9R
OK
[cargo-make] INFO - Build Done in 3.29 seconds.
```