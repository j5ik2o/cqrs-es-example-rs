#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export AWS_PAGER=""
AWS='aws --endpoint-url=http://localhost:4566 --region ap-northeast-1'

pushd ../../

$AWS lambda create-function \
  --architectures arm64 \
  --function-name rmu \
  --handler bootstrap \
  --zip-file fileb://./target/lambda/cqrs-es-example-rmu/bootstrap.zip \
  --runtime provided.al2 \
  --role arn:aws:iam::000000000000:role/lambda-r1 \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active

popd