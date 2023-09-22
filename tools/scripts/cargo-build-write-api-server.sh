#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

cargo build -p write-api-server

popd