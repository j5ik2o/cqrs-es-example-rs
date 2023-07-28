#!/usr/bin/env bash

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export AWS_PAGER=""

BUCKET_NAME="${PREFIX}-${APPLICATION_NAME}-terraform"
echo "BUCKET_NAME=$BUCKET_NAME"

aws --profile ${AWS_PROFILE} s3api list-buckets | grep $BUCKET_NAME
