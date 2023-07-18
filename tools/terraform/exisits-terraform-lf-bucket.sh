#!/usr/bin/env sh

set -eu

BUCKET_NAME="${PREFIX}-${APPLICATION_NAME}-terraform"

aws --profile ${AWS_PROFILE} s3api list-buckets | jq -r '.Buckets[].Name' | grep -E "${BUCKET_NAME}" > /dev/null

if [[ $? -eq 0 ]]; then
  echo "exists bucket:${BUCKET_NAME}"
  exit 1
else
  echo "not exists bucket:${BUCKET_NAME}"
  exit 0
fi