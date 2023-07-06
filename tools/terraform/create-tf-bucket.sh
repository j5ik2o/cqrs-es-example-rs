#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

if [[ ! -e ../../env.sh ]]; then
    echo "env.sh is not found."
    exit 1
fi

# shellcheck disable=SC2034
OUTPUT_ENV=1

source ../../env.sh

export AWS_PAGER=""

BUCKET_NAME="${PREFIX}-${APPLICATION_NAME}-terraform"
echo "BUCKET_NAME=$BUCKET_NAME"

aws --profile "${AWS_PROFILE}" s3 mb s3://"$BUCKET_NAME"
