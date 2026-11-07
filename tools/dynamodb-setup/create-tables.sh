#!/bin/sh

# shellcheck disable=SC2046
cd $(dirname "$0") && pwd

# shellcheck disable=SC2039
if [ $# == 0 ]; then
  echo "Parameters are empty."
  exit 1
fi

while getopts e: OPT; do
  # shellcheck disable=SC2220
  case ${OPT} in
  "e") ENV_NAME="$OPTARG" ;;
  esac
done

export AWS_DEFAULT_REGION=${AWS_REGION:-${AWS_DEFAULT_REGION:-ap-northeast-1}}
export AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID:-x}
export AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY:-x}
export AWS_PAGER=""

ENDPOINT_URL_OPTION=""
DYNAMODB_ENDPOINT=${DYNAMODB_ENDPOINT:-localhost:31566}

JOURNAL_TABLE_NAME=${JOURNAL_TABLE_NAME:-"${PREFIX}-journal"}
SNAPSHOT_TABLE_NAME=${SNAPSHOT_TABLE_NAME:-"${PREFIX}-snapshot"}
SNAPSHOT_HISTORY_INDEX_NAME=${SNAPSHOT_HISTORY_INDEX_NAME:-"${PREFIX}-snapshot-active-history-index"}
HEAD_TABLE_NAME=${HEAD_TABLE_NAME:-"${PREFIX}-head"}

echo "DYNAMODB_ENDPOINT = ${DYNAMODB_ENDPOINT}"
echo "JOURNAL_TABLE_NAME = ${JOURNAL_TABLE_NAME}"
echo "SNAPSHOT_TABLE_NAME = ${SNAPSHOT_TABLE_NAME}"
echo "SNAPSHOT_HISTORY_INDEX_NAME = ${SNAPSHOT_HISTORY_INDEX_NAME}"
echo "HEAD_TABLE_NAME = ${HEAD_TABLE_NAME}"

if [ "${ENV_NAME}" = "dev" ]; then
  # shellcheck disable=SC2034
  ENDPOINT_URL_OPTION=" --endpoint-url http://${DYNAMODB_ENDPOINT} "
fi

# journal はイベントの追記ログ。集約単位の順序を保証しないので Streams は有効にしない。
# shellcheck disable=SC2086
aws dynamodb create-table \
  ${ENDPOINT_URL_OPTION} \
  --table-name "${JOURNAL_TABLE_NAME}" \
  --attribute-definitions \
    AttributeName=aid,AttributeType=S \
    AttributeName=seq_nr,AttributeType=N \
  --key-schema \
    AttributeName=aid,KeyType=HASH \
    AttributeName=seq_nr,KeyType=RANGE \
  --provisioned-throughput \
    ReadCapacityUnits=10,WriteCapacityUnits=10

# snapshot は集約状態のキャッシュ。現在値は skey=0、履歴は skey=seq_nr。
# 保持処理は疎な GSI (aid, active_history_seq_nr) を KEYS_ONLY で読む。
# shellcheck disable=SC2086
aws dynamodb create-table \
  ${ENDPOINT_URL_OPTION} \
  --table-name "${SNAPSHOT_TABLE_NAME}" \
  --attribute-definitions \
    AttributeName=aid,AttributeType=S \
    AttributeName=skey,AttributeType=N \
    AttributeName=active_history_seq_nr,AttributeType=N \
  --key-schema \
    AttributeName=aid,KeyType=HASH \
    AttributeName=skey,KeyType=RANGE \
  --provisioned-throughput \
    ReadCapacityUnits=10,WriteCapacityUnits=10 \
  --global-secondary-indexes \
  "[
    {
      \"IndexName\": \"${SNAPSHOT_HISTORY_INDEX_NAME}\",
      \"KeySchema\": [{\"AttributeName\":\"aid\",\"KeyType\":\"HASH\"},
                      {\"AttributeName\":\"active_history_seq_nr\",\"KeyType\":\"RANGE\"}],
      \"Projection\":{
        \"ProjectionType\":\"KEYS_ONLY\"
      },
      \"ProvisionedThroughput\": {
        \"ReadCapacityUnits\": 10,
        \"WriteCapacityUnits\": 10
      }
    }
  ]"

# shellcheck disable=SC2086
aws dynamodb update-time-to-live \
  ${ENDPOINT_URL_OPTION} \
  --table-name "${SNAPSHOT_TABLE_NAME}" \
  --time-to-live-specification "Enabled=true, AttributeName=ttl"

# head は集約ごとの最新 seq_nr と直近イベントを持つ。変更フィードはこのテーブルの
# Streams だけを使う。Streams は項目単位で順序を保証するので集約の順序が守られる。
# shellcheck disable=SC2086
aws dynamodb create-table \
  ${ENDPOINT_URL_OPTION} \
  --table-name "${HEAD_TABLE_NAME}" \
  --attribute-definitions \
    AttributeName=aid,AttributeType=S \
  --key-schema \
    AttributeName=aid,KeyType=HASH \
  --provisioned-throughput \
    ReadCapacityUnits=10,WriteCapacityUnits=10 \
  --stream-specification StreamEnabled=true,StreamViewType=NEW_IMAGE
