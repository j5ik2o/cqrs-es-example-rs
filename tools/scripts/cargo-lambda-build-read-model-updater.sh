#!/bin/sh
set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

cargo lambda build -p cqrs-es-example-read-model-updater --release --output-format zip

popd