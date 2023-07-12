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


./docker-build-write-api-server.sh $F_OPTION
./docker-build-read-api-server.sh $F_OPTION
./docker-build-local-rmu.sh $F_OPTION