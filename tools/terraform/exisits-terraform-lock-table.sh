#!/usr/bin/env sh

set -eu

TABLE_NAME="${PREFIX}-${APPLICATION_NAME}-terraform-lock"

aws --profile ${AWS_PROFILE} dynamodb list-tables | jq -r '.TableNames[]' | grep -E "${TABLE_NAME}" > /dev/null

if [[ $? -eq 0 ]]; then
  echo "exists table:${TABLE_NAME}"
  exit 1
else
  echo "not exists table:${TABLE_NAME}"
  exit 0
fi