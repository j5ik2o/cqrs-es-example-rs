# read-model-updater

集約 head テーブルの DynamoDB Streams を購読し、write-api-server が発生させたドメインイベントで read-api-server が利用するリードモデルを更新するプロセス。重複と古い `seq_nr` は捨て、欠番は journal から補う。処理位置は `read_model_positions` に記録する。

## Dockerfile

- Dockerfile: 本番用
- Dockerfile.local: ローカル用

## エントリポイント

- `src/main.rs`: 本番用
- `bin/local.rs`: ローカル用

## LocalStack を使った Lambda デプロイ手順

1. `common.env` に `LOCALSTACK_ENDPOINT_URL` などの値を設定する。
2. `makers build-read-model-updater-lambda` で Lambda 用バイナリと ZIP (`dist/lambda/read-model-updater/bootstrap.zip`) を生成する。
3. `makers deploy-read-model-updater-localstack` を実行すると、LocalStack 上に `read-model-updater` 関数が作成され、集約 head テーブル `head` の DynamoDB Streams とイベントソースマッピングが張られる。
4. `makers create-and-get-group-chat` などでイベントを発生させると、LocalStack の Lambda 経由で MySQL リードモデルが更新される。

デプロイスクリプト (`tools/scripts/deploy-read-model-updater-localstack.sh`) は AWS CLI を利用するため、実行前に AWS CLI のインストールを確認すること。
