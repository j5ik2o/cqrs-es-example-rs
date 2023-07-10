#!/bin/sh

# FYI: https://zenn.dev/kinzal/articles/9ee60ebbebc29c

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

LOCAL_REPO_NAME=j5ik2o/cqrs-es-example-rs-write-api-server
TAG=latest
LOCAL_URI=${LOCAL_REPO_NAME}:${TAG}
LOCAL_AMD64_URI=${LOCAL_REPO_NAME}:${TAG}-amd64
LOCAL_ARM64_URI=${LOCAL_REPO_NAME}:${TAG}-arm64

REMOTE_MANIFEST_URI=${ECR_REMOTE_URL}/${REMOTE_REPO_NAME}:${TAG}
REMOTE_AMD64_URI=${ECR_REMOTE_URL}/${REMOTE_REPO_NAME}:${TAG}-amd64
REMOTE_ARM64_URI=${ECR_REMOTE_URL}/${REMOTE_REPO_NAME}:${TAG}-arm64

aws ecr get-login-password --region ${AWS_REGION} | docker login --username AWS --password-stdin ${ECR_REMOTE_URL}

docker tag $LOCAL_AMD64_URI $REMOTE_AMD64_URI
docker tag $LOCAL_ARM64_URI $REMOTE_ARM64_URI

docker push ${REMOTE_AMD64_URI}
docker push ${REMOTE_ARM64_URI}

docker manifest create ${REMOTE_MANIFEST_URI} \
    ${REMOTE_AMD64_URI}  \
    ${REMOTE_ARM64_URI}

docker manifest annotate --arch amd64 ${REMOTE_MANIFEST_URI} $REMOTE_AMD64_URI
docker manifest annotate --arch arm64 ${REMOTE_MANIFEST_URI} $REMOTE_ARM64_URI

docker manifest inspect ${REMOTE_MANIFEST_URI}

docker manifest push ${REMOTE_MANIFEST_URI}