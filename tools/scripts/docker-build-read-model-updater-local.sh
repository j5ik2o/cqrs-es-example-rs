#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

if [ "$#" -gt 0 ]; then
    ARCH=$1
else
    ARCH=$(uname -m)
fi

echo "ARCH=${ARCH}"

LOCAL_REPO_NAME=read-model-updater-local
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

if [ "$ARCH" == "arm64" ] || [ "$ARCH" == "aarch64" ] || [ "$IS_ALL" -eq 1 ]; then

docker buildx build --builder amd-arm --platform linux/arm64 --build-arg TAG_PREFIX=arm64 \
  --build-context messense/rust-musl-cross:arm64-musl=docker-image://messense/rust-musl-cross:aarch64-musl \
  -t $LOCAL_ARM64_URI --load -f applications/read-model-updater/Dockerfile.local . &
PIDS+=($!)

fi

if [ "$ARCH" == "x86_64" ] || [ "$IS_ALL" -eq 1 ]; then

docker buildx build --builder amd-arm --platform linux/amd64 --build-arg TAG_PREFIX=amd64 \
  --build-context messense/rust-musl-cross:amd64-musl=docker-image://messense/rust-musl-cross:x86_64-musl \
  -t $LOCAL_AMD64_URI --load -f applications/read-model-updater/Dockerfile.local . &
PIDS+=($!)

fi

for PID in "${PIDS[@]}"; do
    wait "$PID"
done

popd
