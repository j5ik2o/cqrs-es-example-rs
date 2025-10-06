## `common.env.default` を `common.env` にコピーして編集する

```shell
$ cp common.env.default common.env
```

環境変数の PREFIX（英数字のみ。英字は小文字）と APPLICATION_NAME を適宜変更してください。PREFIX は `pwgen -A` などで生成すると便利です。

```shell
$ pwgen -A
```

`pwgen` が未インストールの場合は `brew install pwgen` で導入できます。

## ビルド方法

ビルドを実行する前に Docker を必ず起動しておきます。

sqlx が DB に接続できるよう、先に DB だけ起動します。

```shell
$ makers docker-compose-up-db
```

```shell
$ makers build
```

補足: sqlx で実行する SQL を追加・変更した場合は、各プロジェクト直下で `cargo sqlx prepare` を実行する必要があります（`.sqlx/` に JSON が生成されます）。このリポジトリでは `makers build` 中に自動で `cargo sqlx prepare` が走るようにしているため、SQL 変更時は必ず `makers build` を実行し、生成された `.sqlx/*.json` を Git 管理下に含めてください。なお DB 接続が必要なので、あらかじめ `makers docker-compose-up-db` を実行しておいてください。

## LocalStack 上での Lambda デプロイ

`makers docker-compose-up` を実行すると、コンテナ起動後に LocalStack へ Lambda が自動デプロイされます。手動で再デプロイしたい場合は次のコマンドを利用します。

```shell
$ makers build-read-model-updater-lambda
$ makers deploy-read-model-updater-localstack
```

`build-read-model-updater-lambda` は Docker 上で Lambda 互換バイナリをビルドし、`dist/lambda/read-model-updater/bootstrap.zip` を生成します。`deploy-read-model-updater-localstack` は AWS CLI を用いて LocalStack に関数を作成（または更新）し、`journal` テーブルのストリームとイベントソースマッピングを設定します。設定値は `common.env` から読み込まれるため、LocalStack のエンドポイントやダミー資格情報を事前に記入してください。

## テスト方法

テストを実行する前に Docker を必ず起動してください。

```shell
$ makers test
```

補足: `cargo test` でもテスト可能ですが、testcontainers を利用している関係で同時実行数を環境変数（`RUST_TEST_THREADS=1`）で制限しています。`RUST_TEST_THREADS=1 cargo test` 以外では正しく動作しないため注意してください。`makers test` ではデフォルトで `RUST_TEST_THREADS=1` が設定されているので、そのまま実行すれば問題ありません。
