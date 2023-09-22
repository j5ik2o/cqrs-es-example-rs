# Docker Compose上でデバッグする

## ビルドイメージ

```shell
$ makers docker-build-local-all
--- Using Environments -----------------
AWS_PROFILE = ceer
AWS_REGION = ap-northeast-1
PREFIX = om2eep1k
APPLICATION_NAME = ceer
----------------------------------------
...
```

## docker-composeの起動

```shell
$ makers docker-compose-up
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
[cargo-make] INFO - Task: docker-compose-up
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Execute Command: "./tools/scripts/docker-compose-up.sh"
[+] Running 13/13
// 中略
[cargo-make] INFO - Build Done in 32.64 seconds.
```

必要なデータベースとテーブルが作成され、アプリケーションも起動します。
開発目的でデータベースだけを起動したい場合は、`docker-compose-up`ではなく`docker-compose-up-db`を実行してください。

## docker-composeの停止

```shell
$ makers docker-compose-down
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
[cargo-make] INFO - Task: docker-compose-down
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Execute Command: "./tools/scripts/docker-compose-down.sh"
[+] Running 10/10
// 中略                                                                                                                                         0.1s
[cargo-make] INFO - Build Done in 37.50 seconds.
```

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

## デバッグ

### グループチャットを作成する

```shell
$ makers create-group-chat
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
[cargo-make] INFO - Task: create-group-chat
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: create-group-chat
{
  "Success": {
    "id": {
      "value": "01H6SXT790F3ERNW0JECPZC0P4"
    }
  }
}
[cargo-make] INFO - Build Done in 1.70 seconds.
```

### GraphQL IDEを開く

```shell
$ makers open-graphql-ide
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
[cargo-make] INFO - Task: open-graphql-ide
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: open-graphql-ide
[cargo-make] INFO - Build Done in 1.25 seconds.
```

GraphQL IDEから以下のクエリを実行する。groupChatIdに上記コマンドで作成したID値を指定してください。

```graphql
{
  getGroupChat(groupChatId: "01H541BDRT2XP2QNH93MSPFAMH") {
    id
    name
    ownerId
    createdAt
  }
}
```

以下のようなレスポンスが返ってくればOK。

```json
{
  "data": {
    "getGroupChat": {
      "id": "01H541BDRT2XP2QNH93MSPFAMH",
      "name": "test",
      "ownerId": "01H4J5WDZDXYJ4NWRDT5AR1J6E",
      "createdAt": "2023-07-12T03:12:10"
    }
  }
}
```

