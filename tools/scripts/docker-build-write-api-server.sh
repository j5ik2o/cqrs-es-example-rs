#!/bin/sh

set -eu

cd $(dirname "$0") || exit

if [ "$#" -gt 0 ]; then
    ARCH=$1
else
    ARCH=$(uname -m)
fi

echo "ARCH=${ARCH}"

APP_NAME=write-api-server
LOCAL_REPO_NAME=j5ik2o/cqrs-es-example-rs-${APP_NAME}

TAG=latest
LOCAL_URI=${LOCAL_REPO_NAME}:${TAG}
LOCAL_AMD64_URI=${LOCAL_REPO_NAME}:${TAG}-amd64
LOCAL_ARM64_URI=${LOCAL_REPO_NAME}:${TAG}-arm64

pushd ../../

IS_ALL=0
if [ "$ARCH" == "all" ]; then
    IS_ALL=1
fi

PIDS=()

if [ "$ARCH" == "arm64" ] || [ "$IS_ALL" -eq 1 ]; then

docker buildx build --builder amd-arm --platform linux/arm64 \
  --build-context messense/rust-musl-cross:arm64-musl=docker-image://messense/rust-musl-cross:aarch64-musl \
  -t $LOCAL_ARM64_URI --load -f applications/${APP_NAME}/Dockerfile . &
PIDS+=($!)

fi

if [ "$ARCH" == "amd64" ] || [ "$IS_ALL" -eq 1 ]; then

docker buildx build --builder amd-arm --platform linux/amd64 \
  --build-context messense/rust-musl-cross:amd64-musl=docker-image://messense/rust-musl-cross:x86_64-musl \
  -t $LOCAL_AMD64_URI --load -f applications/${APP_NAME}/Dockerfile . &
PIDS+=($!)

fi

for PID in "${PIDS[@]}"; do
    wait "$PID"
done

popd