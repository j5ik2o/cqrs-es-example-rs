#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

LOCAL_REPO_NAME=write-api-server
TAG=latest
LOCAL_URI=${LOCAL_REPO_NAME}:${TAG}
LOCAL_AMD64_URI=${LOCAL_REPO_NAME}:${TAG}-amd64
LOCAL_ARM64_URI=${LOCAL_REPO_NAME}:${TAG}-arm64

pushd ../../

ECR_BASE_URL=$(./tools/deploy/terraform/terraform-output.sh -raw ecr_write_api_server_repository_url)

popd

TAG=$(docker images --no-trunc --quiet $LOCAL_AMD64_URI | grep sha256 | cut -d ":" -f2)

REMOTE_MANIFEST_URI=${ECR_BASE_URL}:${TAG}
REMOTE_AMD64_URI=${ECR_BASE_URL}:${TAG}-amd64
REMOTE_ARM64_URI=${ECR_BASE_URL}:${TAG}-arm64

echo ">>> docker tag"
docker tag ${LOCAL_AMD64_URI} ${REMOTE_AMD64_URI}
docker tag ${LOCAL_ARM64_URI} ${REMOTE_ARM64_URI}

echo ">>> docker push"
docker push ${REMOTE_AMD64_URI}
docker push ${REMOTE_ARM64_URI}

echo ">>> docker manifest create"
docker manifest create --amend ${REMOTE_MANIFEST_URI} ${REMOTE_AMD64_URI} ${REMOTE_ARM64_URI}
echo ">>> docker manifest annotate"
docker manifest annotate --arch amd64 ${REMOTE_MANIFEST_URI} $REMOTE_AMD64_URI
docker manifest annotate --arch arm64 ${REMOTE_MANIFEST_URI} $REMOTE_ARM64_URI
echo ">>> docker manifest inspect"
docker manifest inspect ${REMOTE_MANIFEST_URI}
echo ">>> docker manifest push"
docker manifest push ${REMOTE_MANIFEST_URI}

echo "HASH=${TAG}"