#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || (cd .. && pwd))

# Ensure env file exists for makers/scripts that rely on it
if [ ! -f "$ROOT_DIR/common.env" ]; then
  cp "$ROOT_DIR/common.env.default" "$ROOT_DIR/common.env"
fi

# Build Lambda artifact ahead of time so LocalStack deploy succeeds in CI/e2e runs
"$ROOT_DIR/tools/scripts/build-read-model-updater-lambda.sh"

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

# Wait for LocalStack to accept connections before deploying the Lambda
export AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:-test}
export AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY:-test}
export AWS_DEFAULT_REGION=${AWS_DEFAULT_REGION:-ap-northeast-1}
export AWS_REGION=${AWS_REGION:-ap-northeast-1}
LOCALSTACK_ENDPOINT_URL=${LOCALSTACK_ENDPOINT_URL:-http://localhost:4566}

RETRY=0
until aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" dynamodb list-tables >/dev/null 2>&1; do
  RETRY=$((RETRY + 1))
  if [ $RETRY -ge 30 ]; then
    echo "LocalStack is not ready after waiting, aborting." >&2
    exit 1
  fi
  sleep 2
done

# Deploy read-model-updater Lambda into LocalStack so e2e tests can consume the stream
"$ROOT_DIR/tools/scripts/deploy-read-model-updater-localstack.sh"
