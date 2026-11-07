# Docker Compose上でデバッグする

## ビルドイメージ

```shell
$ makers docker-build-local-all
```

e2e を実行する場合は、検証用のイメージもビルドします。

```shell
$ makers docker-build-e2e-test
```

## docker-composeの起動

```shell
$ makers docker-compose-up
```

LocalStack、MySQL、マイグレーション、Write API、Read API、ローカル版 Read Model Updater が起動します。Read Model Updater は集約 head テーブルの DynamoDB Streams を購読し、リードモデルと `read_model_positions` を更新します。

開発目的でデータベースだけを起動したい場合は、`docker-compose-up` ではなく `docker-compose-up-db` を実行してください。

Lambda 版の Read Model Updater は既定ではデプロイしません。手動でデプロイする場合は次のコマンドを実行します。

```shell
$ makers deploy-read-model-updater-localstack
```

`docker-compose-up` の起動時に Lambda もデプロイする場合は、`DOCKER_COMPOSE_UP_DEPLOY_LAMBDA=1` を指定します。このとき LocalStack の `SERVICES` に `lambda` を含めてください。

```shell
$ DOCKER_COMPOSE_UP_DEPLOY_LAMBDA=1 makers docker-compose-up
```

### docker-composeの停止

```shell
$ makers docker-compose-down
```

## e2e

```shell
$ makers docker-compose-e2e-test
```

次の流れを確認します。

1. Write API にグループチャット、メンバー、メッセージを書き込む。
2. Read Model Updater が head Streams からイベントを処理し、MySQL のリードモデルを更新するまで待つ。
3. Read API の 6 つの Query が書き込み結果を返すことを確認する。
4. Read Model Updater を再起動する。
5. 再起動後に新しいグループチャットとメッセージを書き込み、リードモデルに反映されることを確認する。既存の `read_model_positions` が変わらないことも確認する。

待ち合わせは固定時間の sleep ではなく、GraphQL の応答をポーリングします。404 はリードモデルへの反映待ちとして再試行し、それ以外の GraphQL エラーはすぐに失敗します。失敗時は e2e と各サービスのログを出力します。

## ログの確認

```shell
$ docker compose -p cqrs-es-example-rs \
    -f tools/docker-compose/docker-compose-databases.yml \
    -f tools/docker-compose/docker-compose-applications.yml \
    logs --no-color --tail=200 read-model-updater-1
```
