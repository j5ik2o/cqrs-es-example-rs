#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

export ARCH=$(uname -m)
echo "ARCH=${ARCH}"
export TAG_PREFIX="aarch64"

if [ "$ARCH" = "x86_64" ]; then
  TAG_PREFIX="x86_64"
  ARCH="amd64"
fi

if [ "$ARCH" = "aarch64" ]; then
  TAG_PREFIX="aarch64"
  ARCH="arm64"
fi

F_OPTION="-f ../docker-compose/docker-compose-applications.yml -f ../docker-compose/docker-compose-e2e-test.yml"

while getopts d OPT; do
  # shellcheck disable=SC2220
  case ${OPT} in
  "d") F_OPTION="" ;;
  esac
done

# Remove processed options from $@
shift $(($OPTIND - 1))

docker compose -p cqrs-es-example-rs -f ../docker-compose/docker-compose-databases.yml ${F_OPTION} down -v --remove-orphans
docker compose -p cqrs-es-example-rs  -f ../docker-compose/docker-compose-databases.yml ${F_OPTION} up --remove-orphans --force-recreate --renew-anon-volumes -d "$@"
