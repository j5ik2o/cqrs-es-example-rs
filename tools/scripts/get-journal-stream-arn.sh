#!/bin/sh
set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export AWS_PAGER=""
AWS='aws --endpoint-url=http://localhost:4566 --region ap-northeast-1'

$AWS dynamodb describe-table --table-name="journal" | jq -r ".Table.LatestStreamArn"