#!/bin/sh

# FYI: https://zenn.dev/kinzal/articles/9ee60ebbebc29c

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

docker buildx build \
  --push \
  --platform linux/amd64,linux/arm64 \
  --build-context messense/rust-musl-cross:arm64-musl=docker-image://messense/rust-musl-cross:aarch64-musl \
  --build-context messense/rust-musl-cross:amd64-musl=docker-image://messense/rust-musl-cross:x86_64-musl \
  -t j5ik2o/cqrs-es-example-rs-write-api:latest \
  -f command/write-api/Dockerfile .

popd