#!/bin/sh

set -eu

# shellcheck disable=SC2046
cd $(dirname "$0") || exit

if [[ ! -e ../../env.sh ]]; then
    echo "env.sh is not found."
    exit 1
fi

# shellcheck disable=SC2034
OUTPUT_ENV=1

source ../../env.sh

pushd ../../

ECR_BASE_URL=$(./tools/terraform/terraform-output.sh -raw ecr_read_api_server_repository_url)

popd

echo ">>> ecr login"
aws --profile ${AWS_PROFILE} ecr get-login-password --region ${AWS_REGION} | docker login --username AWS --password-stdin ${ECR_BASE_URL}