#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

cargo build -p cqrs-es-example-read-api-server

popd