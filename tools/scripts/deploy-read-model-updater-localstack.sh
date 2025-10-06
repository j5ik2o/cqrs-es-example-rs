#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
DIST_DIR="$ROOT_DIR/dist/lambda/read-model-updater"
ZIP_PATH="$DIST_DIR/bootstrap.zip"

if [[ ! -f "$ZIP_PATH" ]]; then
  echo "エラー: $ZIP_PATH が見つかりません。先に build-read-model-updater-lambda を実行してください。" >&2
  exit 1
fi

AWS_REGION=${AWS_REGION:-ap-northeast-1}
LOCALSTACK_ENDPOINT_URL=${LOCALSTACK_ENDPOINT_URL:-http://localhost:4566}
FUNCTION_NAME=${READ_MODEL_UPDATER_LAMBDA_FUNCTION_NAME:-read-model-updater}
ROLE_ARN=${READ_MODEL_UPDATER_LAMBDA_ROLE_ARN:-arn:aws:iam::000000000000:role/service-role/read-model-updater-role}

export AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:-test}
export AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY:-test}
export AWS_SESSION_TOKEN=${AWS_SESSION_TOKEN:-}
export AWS_REGION
export AWS_DEFAULT_REGION=${AWS_DEFAULT_REGION:-$AWS_REGION}
unset AWS_PROFILE
unset AWS_DEFAULT_PROFILE

APP_AWS_REGION_NAME=${APP__AWS__REGION_NAME:-$AWS_REGION}
APP_AWS_ENDPOINT_URL=${APP__AWS__ENDPOINT_URL:-$LOCALSTACK_ENDPOINT_URL}
APP_AWS_ACCESS_KEY_ID=${APP__AWS__ACCESS_KEY_ID:-${AWS_ACCESS_KEY_ID:-test}}
APP_AWS_SECRET_ACCESS_KEY=${APP__AWS__SECRET_ACCESS_KEY:-${AWS_SECRET_ACCESS_KEY:-test}}
APP_STREAM_JOURNAL_TABLE_NAME=${APP__STREAM__JOURNAL_TABLE_NAME:-journal}
APP_STREAM_MAX_ITEM_COUNT=${APP__STREAM__MAX_ITEM_COUNT:-32}
APP_DATABASE_URL=${APP__DATABASE__URL:-mysql://ceer:ceer@mysql-local:3306/ceer}
RUST_LOG_VALUE=${RUST_LOG:-info}

# AWS CLI が必要
if ! command -v aws >/dev/null 2>&1; then
  echo "エラー: aws コマンドが見つかりません。AWS CLI をインストールしてください。" >&2
  exit 1
fi

ENV_FILE=$(mktemp)
trap 'rm -f "$ENV_FILE"' EXIT

APP_AWS_REGION_NAME="$APP_AWS_REGION_NAME" \
APP_AWS_ENDPOINT_URL="$APP_AWS_ENDPOINT_URL" \
APP_AWS_ACCESS_KEY_ID="$APP_AWS_ACCESS_KEY_ID" \
APP_AWS_SECRET_ACCESS_KEY="$APP_AWS_SECRET_ACCESS_KEY" \
APP_STREAM_JOURNAL_TABLE_NAME="$APP_STREAM_JOURNAL_TABLE_NAME" \
APP_STREAM_MAX_ITEM_COUNT="$APP_STREAM_MAX_ITEM_COUNT" \
APP_DATABASE_URL="$APP_DATABASE_URL" \
RUST_LOG_VALUE="$RUST_LOG_VALUE" \
python3 - "$ENV_FILE" <<'PY'
import json
import os
import sys

env = {
    "APP__AWS__REGION_NAME": os.environ["APP_AWS_REGION_NAME"],
    "APP__AWS__ENDPOINT_URL": os.environ["APP_AWS_ENDPOINT_URL"],
    "APP__AWS__ACCESS_KEY_ID": os.environ["APP_AWS_ACCESS_KEY_ID"],
    "APP__AWS__SECRET_ACCESS_KEY": os.environ["APP_AWS_SECRET_ACCESS_KEY"],
    "APP__STREAM__JOURNAL_TABLE_NAME": os.environ["APP_STREAM_JOURNAL_TABLE_NAME"],
    "APP__STREAM__MAX_ITEM_COUNT": os.environ["APP_STREAM_MAX_ITEM_COUNT"],
    "APP__DATABASE__URL": os.environ["APP_DATABASE_URL"],
    "RUST_LOG": os.environ["RUST_LOG_VALUE"],
}
with open(sys.argv[1], "w", encoding="utf-8") as fh:
    json.dump({"Variables": env}, fh)
PY

set +e
aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda get-function \
  --function-name "$FUNCTION_NAME" >/dev/null 2>&1
FOUND=$?
set -e

if [[ $FOUND -eq 0 ]]; then
  echo "既存の関数を更新します: $FUNCTION_NAME"
  aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda update-function-code \
    --function-name "$FUNCTION_NAME" \
    --zip-file "fileb://$ZIP_PATH" >/dev/null
  aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda update-function-configuration \
    --function-name "$FUNCTION_NAME" \
    --runtime provided.al2 \
    --handler bootstrap \
    --memory-size 512 \
    --timeout 30 \
    --environment "file://$ENV_FILE" >/dev/null
else
  echo "新しい関数を作成します: $FUNCTION_NAME"
  aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda create-function \
    --function-name "$FUNCTION_NAME" \
    --runtime provided.al2 \
    --role "$ROLE_ARN" \
    --handler bootstrap \
    --zip-file "fileb://$ZIP_PATH" \
    --memory-size 512 \
    --timeout 30 \
    --environment "file://$ENV_FILE" >/dev/null
fi

STREAM_ARN=$(aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" dynamodb describe-table \
  --table-name "$APP_STREAM_JOURNAL_TABLE_NAME" \
  --query 'Table.LatestStreamArn' \
  --output text)

if [[ -z "$STREAM_ARN" || "$STREAM_ARN" == "None" ]]; then
  echo "エラー: DynamoDBテーブル $APP_STREAM_JOURNAL_TABLE_NAME のストリームARNを取得できませんでした。ストリームが有効化されているか確認してください。" >&2
  exit 1
fi

echo "既存のイベントソースマッピングを削除します"
EXISTING_UUIDS=$(aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda list-event-source-mappings \
  --function-name "$FUNCTION_NAME" \
  --query 'EventSourceMappings[].UUID' \
  --output text || true)

if [[ -n "$EXISTING_UUIDS" && "$EXISTING_UUIDS" != "None" ]]; then
  for uuid in $EXISTING_UUIDS; do
    aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda delete-event-source-mapping \
      --uuid "$uuid" >/dev/null
  done
fi

echo "イベントソースマッピングを作成します"
aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" lambda create-event-source-mapping \
  --function-name "$FUNCTION_NAME" \
  --event-source-arn "$STREAM_ARN" \
  --starting-position LATEST \
  --batch-size ${READ_MODEL_UPDATER_LAMBDA_BATCH_SIZE:-64} >/dev/null

echo "デプロイが完了しました"
