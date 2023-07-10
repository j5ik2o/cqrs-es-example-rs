#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

docker run -v $(pwd)/../../config:/config \
  -p 8080:8080 \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  -d j5ik2o/cqrs-es-example-rs-read-api-server:latest-arm64