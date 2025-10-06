#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || (cd .. && pwd))

if [ ! -f "$ROOT_DIR/common.env" ]; then
  cp "$ROOT_DIR/common.env.default" "$ROOT_DIR/common.env"
fi

LAMBDA_BOOTSTRAP_PATH="$ROOT_DIR/dist/lambda/read-model-updater/bootstrap"

build_lambda() {
  "$ROOT_DIR/tools/scripts/build-read-model-updater-lambda.sh"
}

if [ "${DOCKER_COMPOSE_UP_BUILD_LAMBDA:-1}" = "1" ]; then
  REBUILD=0
  if [ ! -f "$LAMBDA_BOOTSTRAP_PATH" ]; then
    REBUILD=1
  else
    if [ -n "$(find "$ROOT_DIR/applications/read-model-updater" "$ROOT_DIR/modules/rmu" -type f -newer "$LAMBDA_BOOTSTRAP_PATH" -print -quit)" ]; then
      REBUILD=1
    fi
  fi
  if [ $REBUILD -eq 1 ]; then
    build_lambda
  fi
fi

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

F_OPTION="-f ../docker-compose/docker-compose-applications.yml"

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

if [ "${DOCKER_COMPOSE_UP_DEPLOY_LAMBDA:-1}" = "1" ]; then

  export AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:-test}
  export AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY:-test}
  export AWS_DEFAULT_REGION=${AWS_DEFAULT_REGION:-ap-northeast-1}
  export AWS_REGION=${AWS_REGION:-ap-northeast-1}
  LOCALSTACK_ENDPOINT_URL=${LOCALSTACK_ENDPOINT_URL:-http://localhost:4566}

  RETRY=0
  MAX_RETRY=${LOCALSTACK_WAIT_MAX_RETRY:-30}
  WAIT_INTERVAL=${LOCALSTACK_WAIT_INTERVAL:-2}
  until aws --endpoint-url "$LOCALSTACK_ENDPOINT_URL" --region "$AWS_REGION" dynamodb list-tables >/dev/null 2>&1; do
    RETRY=$((RETRY + 1))
    if [ $RETRY -ge $MAX_RETRY ]; then
      echo "LocalStack is not ready after waiting, aborting Lambda deployment." >&2
      exit 1
    fi
    sleep "$WAIT_INTERVAL"
  done

  "$ROOT_DIR/tools/scripts/deploy-read-model-updater-localstack.sh"
fi
