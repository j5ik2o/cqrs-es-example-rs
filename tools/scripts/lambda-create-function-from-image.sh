#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export AWS_PAGER=""
AWS='aws --endpoint-url=http://localhost:4566 --region ap-northeast-1'

pushd ../../

$AWS lambda create-function \
  --function-name rmu \
  --handler bootstrap \
  --package-type Image \
  --code ImageUri=j5ik2o/cqrs-es-example-rs-rmu:latest \
  --role arn:aws:iam::000000000000:role/lambda-r1 \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active

popd