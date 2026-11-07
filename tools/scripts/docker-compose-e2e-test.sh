#!/usr/bin/env bash

set -euo pipefail

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || (cd .. && pwd))

# Ensure env file exists for makers/scripts that rely on it
if [ ! -f "$ROOT_DIR/common.env" ]; then
  cp "$ROOT_DIR/common.env.default" "$ROOT_DIR/common.env"
fi

export AWS_REGION=${AWS_REGION:-ap-northeast-1}
export AWS_DEFAULT_REGION=${AWS_DEFAULT_REGION:-$AWS_REGION}

export ARCH=$(uname -m)
echo "ARCH=${ARCH}"

if [ "$ARCH" = "x86_64" ]; then
  ARCH="amd64"
fi

if [ "$ARCH" = "aarch64" ]; then
  ARCH="arm64"
fi

# The e2e uses the containerized read model updater, which subscribes to the
# head Streams and replays from TRIM_HORIZON on restart. Lambda is the optional
# path and is not part of this scenario, so do not load it in LocalStack.
export LOCALSTACK_SERVICES=${LOCALSTACK_SERVICES:-cloudwatch,dynamodb,dynamodbstreams}
export LOCALSTACK_DEBUG=${LOCALSTACK_DEBUG:-0}
export LOCALSTACK_EAGER_SERVICE_LOADING=${LOCALSTACK_EAGER_SERVICE_LOADING:-0}

PROJECT_NAME=${COMPOSE_PROJECT_NAME:-cqrs-es-example-rs}
F_OPTION="-f ../docker-compose/docker-compose-applications.yml -f ../docker-compose/docker-compose-e2e-test.yml"
RUN_E2E=1
SERVICES="localstack mysql refinery dynamodb-setup dynamodb-admin write-api-server-1 read-model-updater-1 read-api-server-1"

while getopts d OPT; do
  # shellcheck disable=SC2220
  case ${OPT} in
  "d")
    F_OPTION=""
    RUN_E2E=0
    SERVICES="localstack mysql refinery dynamodb-setup dynamodb-admin"
    ;;
  esac
done

# Remove processed options from $@
shift $(($OPTIND - 1))

compose() {
  # shellcheck disable=SC2086
  docker compose -p "${PROJECT_NAME}" -f ../docker-compose/docker-compose-databases.yml ${F_OPTION} "$@"
}

dump_logs() {
  echo "=== Compose ps ===" >&2
  compose ps >&2 || true
  echo "=== Compose logs ===" >&2
  compose logs --no-color --tail=200 >&2 || true
}

on_exit() {
  local status=$?
  if [ "${status}" -ne 0 ]; then
    echo "e2e failed with exit code ${status}; collecting compose logs" >&2
    dump_logs
  fi
  exit "${status}"
}
trap on_exit EXIT

if [ "${RUN_E2E}" -eq 1 ]; then
  for image in \
    "write-api-server:latest-${ARCH}" \
    "read-model-updater-local:latest-${ARCH}" \
    "read-api-server:latest-${ARCH}"; do
    if ! docker image inspect "${image}" >/dev/null 2>&1; then
      echo "Error: ${image} is not built. Run makers docker-build-local-all first." >&2
      exit 1
    fi
  done
  if ! docker image inspect e2e-test-rs:latest >/dev/null 2>&1; then
    echo "Building the e2e test image"
    (cd ../e2e-test && make build)
  fi
fi

compose down -v --remove-orphans
# shellcheck disable=SC2086
compose up --remove-orphans --force-recreate --renew-anon-volumes -d ${SERVICES} "$@"

if [ "${RUN_E2E}" -eq 0 ]; then
  echo "Started databases only (-d); skipping the end-to-end verification"
  exit 0
fi

echo "Running the Write API -> read model updater -> Read API scenario"
compose run -T --rm --no-deps e2e-test

mysql_query() {
  compose exec -T mysql mysql --protocol=TCP -h127.0.0.1 -uceer -pceer -N -B ceer -e "$1" 2>/dev/null
}

positions_before=$(mysql_query "SELECT aggregate_id, seq_nr FROM read_model_positions ORDER BY aggregate_id")
group_chats_before=$(mysql_query "SELECT COUNT(*) FROM group_chats")
messages_before=$(mysql_query "SELECT COUNT(*) FROM messages")
if [ -z "${positions_before}" ]; then
  echo "Error: read_model_positions is empty before restart" >&2
  exit 1
fi

rmu_insert_errors_before=$(compose logs --no-color --no-log-prefix read-model-updater-1 2>/dev/null | grep -cE "Failed to insert|Duplicate entry" || true)
echo "Restarting read-model-updater-1 to exercise head Streams replay"
compose restart read-model-updater-1

echo "Running a new Write API -> Read API scenario after the restart"
if ! compose run -T --rm --no-deps --entrypoint ./verify-after-restart.sh e2e-test; then
  echo "Post-restart verification failed" >&2
  compose logs --no-color --tail=200 read-model-updater-1 >&2
  exit 1
fi

positions_after=$(mysql_query "SELECT aggregate_id, seq_nr FROM read_model_positions ORDER BY aggregate_id")
group_chats_after=$(mysql_query "SELECT COUNT(*) FROM group_chats")
messages_after=$(mysql_query "SELECT COUNT(*) FROM messages")

if [ "${group_chats_after}" -ne $((group_chats_before + 1)) ]; then
  echo "Error: group_chats count after restart is ${group_chats_after}, expected $((group_chats_before + 1))" >&2
  exit 1
fi
if [ "${messages_after}" -ne $((messages_before + 1)) ]; then
  echo "Error: messages count after restart is ${messages_after}, expected $((messages_before + 1))" >&2
  exit 1
fi

# Every aggregate that existed before the restart keeps its position: the
# replayed head records are dropped as duplicates instead of being applied again.
while read -r aggregate_id seq_nr; do
  [ -z "${aggregate_id}" ] && continue
  after_seq_nr=$(echo "${positions_after}" | awk -v aggregate_id="${aggregate_id}" '$1 == aggregate_id { print $2 }')
  if [ "${after_seq_nr}" != "${seq_nr}" ]; then
    echo "Error: read_model_positions for ${aggregate_id} changed from ${seq_nr} to ${after_seq_nr} after restart" >&2
    exit 1
  fi
done <<< "${positions_before}"

# A replay that bypassed the committed positions would fail with a duplicate
# primary key. Fail if the restart added such errors to the updater log.
rmu_insert_errors_after=$(compose logs --no-color --no-log-prefix read-model-updater-1 2>/dev/null | grep -cE "Failed to insert|Duplicate entry" || true)
if [ "${rmu_insert_errors_after}" -gt "${rmu_insert_errors_before}" ]; then
  echo "Error: the read model updater logged new insert errors after restart (${rmu_insert_errors_before} -> ${rmu_insert_errors_after})" >&2
  compose logs --no-color --tail=200 read-model-updater-1 >&2
  exit 1
fi

echo "e2e-test and post-restart replay succeeded"
