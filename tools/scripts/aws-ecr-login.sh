#!/usr/bin/env bash

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

pushd ../../

ECR_BASE_URL=$(./tools/deploy/terraform/terraform-output.sh -raw ecr_read_api_server_repository_url)

popd

echo ">>> ecr login"
aws --profile ${AWS_PROFILE} ecr get-login-password --region ${AWS_REGION} | docker login --username AWS --password-stdin ${ECR_BASE_URL}