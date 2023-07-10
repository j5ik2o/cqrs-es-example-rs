#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export AWS_PAGER=""
AWS='aws --endpoint-url=http://localhost:4566 --region ap-northeast-1'

ARN=$(sh ./get-journal-stream-arn.sh)

pushd ../../

$AWS lambda create-event-source-mapping \
  --function-name read-model-updater \
  --batch-size 100 \
  --maximum-batching-window-in-seconds 30 \
  --starting-position LATEST \
  --event-source-arn $ARN

popd