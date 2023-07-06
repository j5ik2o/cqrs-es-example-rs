#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

if [[ ! -e ./terraform-env.sh ]]; then
    echo "terraform-env.sh is not found."
    exit 1
fi

# shellcheck disable=SC2034
OUTPUT_ENV=0

. ./terraform-env.sh

terraform output --state="${TF_STATE_NAME}" "$@"