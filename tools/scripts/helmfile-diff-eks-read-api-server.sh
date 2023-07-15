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

export AWS_DEFAULT_REGION=$AWS_REGION

pushd ../helmfile.d

helmfile --namespace ceer --selector group=read-api-server -e "${PREFIX}-${APPLICATION_NAME}-eks" diff

popd