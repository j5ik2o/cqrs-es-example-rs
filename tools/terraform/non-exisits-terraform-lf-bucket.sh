#!/usr/bin/env sh

BUCKET_NAME="${PREFIX}-${APPLICATION_NAME}-terraform"
echo "BUCKET_NAME=$BUCKET_NAME"

RESULT=$(aws --profile ${AWS_PROFILE} s3api list-buckets | jq -r '.Buckets[].Name' | grep -E "${BUCKET_NAME}")
echo "RESULT=$RESULT"

if [[ -z "$RESULT" ]]; then
  exit 0
else
  exit 1
fi