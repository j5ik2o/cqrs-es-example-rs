#!/usr/bin/env bash

DYNAMODB_LOCAL=${DYNAMODB_LOCAL:-localhost}

echo "host = ${DYNAMODB_LOCAL}"

AWS_PAGER="" AWS_ACCESS_KEY_ID=x AWS_SECRET_ACCESS_KEY=x \
  aws dynamodb --endpoint-url http://${DYNAMODB_LOCAL}:8000 scan --table-name Account
