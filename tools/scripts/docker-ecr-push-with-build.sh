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

pushd ../../

./tools/scripts/docker-build-write-api-server.sh all &
PID1=$!
./tools/scripts/docker-build-read-api-server.sh all &
PID2=$!

wait $PID1 $PID2

./tools/scripts/docker-ecr-push-write-api-server.sh &
PID1=$!
./tools/scripts/docker-ecr-push-read-api-server.sh &
PID2=$!

wait $PID1 $PID2

popd