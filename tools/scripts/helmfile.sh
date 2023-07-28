#!/usr/bin/env bash

set -eu

OPTS=${OPTS:-}
K8S_CONTEXT=${K8S_CONTEXT:-}

if [[ -z "$K8S_CONTEXT" ]]; then
	echo "K8S_CONTEXT is not set"
	exit 1
fi

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

echo "OPTS=${OPTS}"

export AWS_DEFAULT_REGION=$AWS_REGION

pushd ../helmfile.d

echo "helmfile --namespace ceer ${OPTS} -e ${PREFIX}-${APPLICATION_NAME}-eks $@"

helmfile --namespace ceer ${OPTS} -e "${PREFIX}-${APPLICATION_NAME}-eks" "$@"

popd