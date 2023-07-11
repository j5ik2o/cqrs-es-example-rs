#!/bin/sh

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

docker run -v $(pwd)/../rdb-migration:/app \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  -e DATABASE_URL=mysql://ceer:ceer@127.0.0.1:3306/ceer \
  ghcr.io/rust-db/refinery:main \
    migrate -e DATABASE_URL -p /app/migrations
