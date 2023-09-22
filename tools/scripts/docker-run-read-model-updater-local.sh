#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

docker run -v $(pwd)/../../config:/config \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=full \
  -e APP__DATABASE__URL=mysql://ceer:ceer@localhost:3306/ceer \
  -e APP__AWS__REGION_NAME=${AWS_REGION} \
  -e APP__AWS__ENDPOINT_URL=http://localstack:4566 \
  -e APP__AWS__ACCESS_KEY_ID=x \
  -e APP__AWS__SECRET_ACCESS_KEY=x \
  -e APP__STREAM_JOURNAL_TABLE_NAME=journal \
  -e APP__STREAM_MAX_ITEM_COUNT=100 \
  -d read-model-updater-local:latest-arm64