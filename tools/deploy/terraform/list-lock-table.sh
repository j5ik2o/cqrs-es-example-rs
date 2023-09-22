#!/usr/bin/env bash

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export AWS_PAGER=""

TABLE_NAME="${PREFIX}-${APPLICATION_NAME}-terraform-lock"
echo "TABLE_NAME=$TABLE_NAME"

aws --profile "$AWS_PROFILE" dynamodb list-table
