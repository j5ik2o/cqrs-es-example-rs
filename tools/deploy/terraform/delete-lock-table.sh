#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

# shellcheck disable=SC2034
OUTPUT_ENV=1

source ../../env.sh

export AWS_PAGER=""

TABLE_NAME="${PREFIX}-${APPLICATION_NAME}-terraform-lock"
echo "TABLE_NAME=$TABLE_NAME"

aws --profile "$AWS_PROFILE" \
  dynamodb delete-table  --table-name "$TABLE_NAME"