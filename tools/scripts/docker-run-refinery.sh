#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

# shellcheck disable=SC2034
OUTPUT_ENV=1

source ../../env.sh

IMAGE=ghcr.io/rust-db/refinery:main
# IMAGE=refinery_ceer:latest

docker run -v $(pwd)/../refinery:/app \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  -e DATABASE_URL=mysql://ceer:ceer@host.docker.internal:3306/ceer \
  ${IMAGE} \
    migrate -e DATABASE_URL -p /app/migrations
