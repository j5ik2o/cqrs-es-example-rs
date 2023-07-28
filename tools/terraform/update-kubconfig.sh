#!/usr/bin/env bash

set -eu

cd $(dirname "$0") || exit

. ./terraform-env.sh

NAME=$(terraform output --state="${TF_STATE_NAME}" -raw eks_cluster_name)

# shellcheck disable=SC2046
aws --profile ${AWS_PROFILE} eks update-kubeconfig --name ${NAME}
