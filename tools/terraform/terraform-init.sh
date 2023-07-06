#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

if [[ ! -e ./terraform-env.sh ]]; then
    echo "terraform-env.sh is not found."
    exit 1
fi

# shellcheck disable=SC2034
OUTPUT_ENV=1

. ./terraform-env.sh

terraform init -backend=true \
  -backend-config="region=${AWS_REGION}" \
  -backend-config="profile=${AWS_PROFILE}" \
  -backend-config="bucket=${TF_BUCKET_NAME}" \
  -backend-config="key=${TF_STATE_NAME}" \
  -backend-config="dynamodb_table=${TF_LOCK_TABLE_NAME}" \
  -backend-config="encrypt=true" \
  --var-file="${TF_VAR_FILE}" "$@"
