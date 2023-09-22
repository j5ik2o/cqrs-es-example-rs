#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../query/read-api-server

cargo run -p read-api-server --bin export-sdl

popd