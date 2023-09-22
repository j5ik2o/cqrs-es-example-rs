#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

./terraform-output.sh -raw eks_aws_auth_config_map > ./eks_aws_auth_config_map.yaml