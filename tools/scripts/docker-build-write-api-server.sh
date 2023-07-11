#!/bin/sh

# FYI: https://zenn.dev/kinzal/articles/9ee60ebbebc29c

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

F_OPTION=""
while getopts f OPT; do
  # shellcheck disable=SC2220
  case ${OPT} in
  "f") F_OPTION=1 ;;
  esac
done

LOCAL_REPO_NAME=j5ik2o/cqrs-es-example-rs-write-api-server
TAG=latest
LOCAL_URI=${LOCAL_REPO_NAME}:${TAG}
LOCAL_AMD64_URI=${LOCAL_REPO_NAME}:${TAG}-amd64
LOCAL_ARM64_URI=${LOCAL_REPO_NAME}:${TAG}-arm64

pushd ../../

docker buildx build --builder amd-arm --platform linux/arm64 \
  --build-context messense/rust-musl-cross:arm64-musl=docker-image://messense/rust-musl-cross:aarch64-musl \
  -t $LOCAL_ARM64_URI --load -f command/write-api-server/Dockerfile .

if [[ "$F_OPTION" == 1 ]]; then
docker buildx build --builder amd-arm --platform linux/amd64 \
  --build-context messense/rust-musl-cross:amd64-musl=docker-image://messense/rust-musl-cross:x86_64-musl \
  -t $LOCAL_AMD64_URI --load -f command/write-api-server/Dockerfile .
fi

popd