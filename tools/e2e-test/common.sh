#!/usr/bin/env bash

# Shared GraphQL helpers for the docker-compose e2e scripts.

ADMIN_ID=${ADMIN_ID:-UserAccount-01H42K4ABWQ5V2XQEP3A48VE0Z}
USER_ACCOUNT_ID=${USER_ACCOUNT_ID:-UserAccount-01H7C6DWMK1BKS1JYH1XZE529M}
WRITE_API_SERVER_BASE_URL=${WRITE_API_SERVER_BASE_URL:-http://localhost:18080}
READ_API_SERVER_BASE_URL=${READ_API_SERVER_BASE_URL:-http://localhost:18082}
POLL_ATTEMPTS=${POLL_ATTEMPTS:-90}
POLL_INTERVAL_SECONDS=${POLL_INTERVAL_SECONDS:-2}

MUTATION_RESULT=""

post_graphql() {
  local base_url="$1"
  local body="$2"
  # A refused connection during startup returns an empty body so the polling
  # loops can retry instead of aborting the scenario.
  curl -sS --max-time 30 -X POST -H "Content-Type: application/json" "${base_url}/query" -d "${body}" || true
}

# run_mutation <label> <body>. Stores the raw response in MUTATION_RESULT.
run_mutation() {
  local label="$1"
  local body="$2"
  echo -e "\n${label}:"
  MUTATION_RESULT=$(post_graphql "${WRITE_API_SERVER_BASE_URL}" "${body}")
  if ! echo "${MUTATION_RESULT}" | jq -e '.data != null and ((.errors // []) | length == 0)' > /dev/null 2>&1; then
    echo "Error: ${MUTATION_RESULT}"
    exit 1
  fi
  echo "Result: ${MUTATION_RESULT}"
}

# wait_for_graphql <label> <base_url> <body> <jq-filter>. Waits until an API
# answers before the scenario starts.
wait_for_graphql() {
  local label="$1"
  local base_url="$2"
  local body="$3"
  local filter="$4"
  echo -e "\nWait for ${label}:"
  local attempt=0
  local result=""
  while [ "${attempt}" -lt "${POLL_ATTEMPTS}" ]; do
    result=$(post_graphql "${base_url}" "${body}")
    if echo "${result}" | jq -e "${filter}" > /dev/null 2>&1; then
      echo "Ready: ${result}"
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 1
  done
  echo "Error: timed out waiting for ${label}: ${result}"
  exit 1
}

# wait_for_read <label> <body> <jq-filter>. Polls the read model until the
# projection catches up. A 404 means the RMU has not projected the event yet.
# Any other GraphQL error fails immediately so the cause is not hidden.
wait_for_read() {
  local label="$1"
  local body="$2"
  local filter="$3"
  echo -e "\n${label}:"
  local attempt=0
  local result=""
  while [ "${attempt}" -lt "${POLL_ATTEMPTS}" ]; do
    result=$(post_graphql "${READ_API_SERVER_BASE_URL}" "${body}")
    if echo "${result}" | jq -e '.errors // [] | length > 0' > /dev/null 2>&1; then
      if echo "${result}" | jq -e '.errors | all(.extensions.code == "404")' > /dev/null 2>&1; then
        attempt=$((attempt + 1))
        sleep "${POLL_INTERVAL_SECONDS}"
        continue
      fi
      echo "Error: unexpected GraphQL error while waiting for ${label}: ${result}"
      exit 1
    fi
    if echo "${result}" | jq -e "${filter}" > /dev/null 2>&1; then
      echo "Result: ${result}"
      return 0
    fi
    attempt=$((attempt + 1))
    sleep "${POLL_INTERVAL_SECONDS}"
  done
  echo "Error: timed out waiting for ${label}: ${result}"
  exit 1
}

# wait_for_error_code <label> <body> <code>. Polls until the read API returns
# errors that all carry the given code. Any other error fails immediately.
wait_for_error_code() {
  local label="$1"
  local body="$2"
  local code="$3"
  echo -e "\n${label}:"
  local attempt=0
  local result=""
  while [ "${attempt}" -lt "${POLL_ATTEMPTS}" ]; do
    result=$(post_graphql "${READ_API_SERVER_BASE_URL}" "${body}")
    if echo "${result}" | jq -e --arg code "${code}" '.errors // [] | length > 0 and all(.[]; .extensions.code == $code)' > /dev/null 2>&1; then
      echo "Result: ${result}"
      return 0
    fi
    if echo "${result}" | jq -e '.errors // [] | length > 0' > /dev/null 2>&1; then
      echo "Error: unexpected GraphQL error while waiting for ${label} to return ${code}: ${result}"
      exit 1
    fi
    attempt=$((attempt + 1))
    sleep "${POLL_INTERVAL_SECONDS}"
  done
  echo "Error: timed out waiting for ${label}: ${result}"
  exit 1
}
