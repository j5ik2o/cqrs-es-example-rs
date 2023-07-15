#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

F_OPTION=""
while getopts f OPT; do
  # shellcheck disable=SC2220
  case ${OPT} in
  "f") F_OPTION="-f" ;;
  esac
done


PIDS=()

./docker-build-write-api-server.sh $F_OPTION &
PIDS+=($!)
./docker-build-read-api-server.sh $F_OPTION &
PIDS+=($!)
./docker-build-read-model-updater-local.sh $F_OPTION &
PIDS+=($!)
./docker-build-read-model-updater.sh $F_OPTION &
PIDS+=($!)

for PID in "${PIDS[@]}"; do
    wait "$PID"
done