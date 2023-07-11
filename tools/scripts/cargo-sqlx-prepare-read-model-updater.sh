#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../read-model-updater

cargo sqlx prepare -D mysql://ceer:ceer@localhost:3306/ceer

popd