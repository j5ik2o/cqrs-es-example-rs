#!/bin/sh

# FYI: https://zenn.dev/kinzal/articles/9ee60ebbebc29c
# FYI: https://github.com/chatwork/ddb-to-kds/blob/main/Dockerfile
# FYI: https://qiita.com/aoyagikouhei/items/4ca1acccb876c5ab60c8

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

docker build \
  --platform=linux/amd64 \
  -t j5ik2o/cqrs-es-example-rs-rmu:latest \
  -f rmu/Dockerfile .

popd