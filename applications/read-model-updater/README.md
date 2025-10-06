# read-model-updater

write-api-serverで発生するドメインイベントを使って、read-api-serverが利用するリードモデルを更新するプロセス。

## Dockerfile

- Dockerfile: 本番用
- Dockerfile.local: ローカル用

## エントリポイント

- `src/main.rs`: 本番用
- `bin/local.rs`: ローカル用

## LocalStack を使った Lambda デプロイ手順

1. `common.env` に `LOCALSTACK_ENDPOINT_URL` などの値を設定する。
2. `makers build-read-model-updater-lambda` で Lambda 用バイナリと ZIP (`dist/lambda/read-model-updater/bootstrap.zip`) を生成する。
3. `makers deploy-read-model-updater-localstack` を実行すると、LocalStack 上に `read-model-updater` 関数が作成され DynamoDB Streams (`journal`) とイベントソースマッピングが張られる。
4. `makers create-and-get-group-chat` などでイベントを発生させると、LocalStack の Lambda 経由で MySQL リードモデルが更新される。

デプロイスクリプト (`tools/scripts/deploy-read-model-updater-localstack.sh`) は AWS CLI を利用するため、実行前に AWS CLI のインストールを確認すること。
