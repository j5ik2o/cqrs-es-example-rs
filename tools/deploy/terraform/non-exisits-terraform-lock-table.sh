#!/usr/bin/env bash

TABLE_NAME="${PREFIX}-${APPLICATION_NAME}-terraform-lock"
echo "TABLE_NAME=$TABLE_NAME"

RESULT=$(aws --profile ${AWS_PROFILE} dynamodb list-tables | jq -r '.TableNames[]' | grep -E "${TABLE_NAME}")
echo "RESULT=$RESULT"

if [ -z "${RESULT}" ]; then
  exit 0
else
  exit 1
fi