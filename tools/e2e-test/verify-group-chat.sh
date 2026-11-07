#!/usr/bin/env bash

set -euo pipefail

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
  curl -s -X POST -H "Content-Type: application/json" "${base_url}/query" -d "${body}" || true
}

# run_mutation <label> <body>. Stores the raw response in MUTATION_RESULT.
run_mutation() {
  local label="$1"
  local body="$2"
  echo -e "\n${label}:"
  MUTATION_RESULT=$(post_graphql "${WRITE_API_SERVER_BASE_URL}" "${body}")
  if ! echo "${MUTATION_RESULT}" | jq -e '.data != null and (.errors == null)' > /dev/null 2>&1; then
    echo "Error: ${MUTATION_RESULT}"
    exit 1
  fi
  echo "Result: ${MUTATION_RESULT}"
}

# wait_for_read <label> <body> <jq-filter>. Polls the read model until the filter
# matches, so a fixed sleep is not the only thing separating a write from its
# projection. A 404 means the RMU has not projected the event yet.
wait_for_read() {
  local label="$1"
  local body="$2"
  local filter="$3"
  echo -e "\n${label}:"
  local attempt=0
  local result=""
  while [ "${attempt}" -lt "${POLL_ATTEMPTS}" ]; do
    result=$(post_graphql "${READ_API_SERVER_BASE_URL}" "${body}")
    if ! echo "${result}" | jq -e .errors > /dev/null 2>&1 && echo "${result}" | jq -e "${filter}" > /dev/null 2>&1; then
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
# errors that all carry the given code.
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
    attempt=$((attempt + 1))
    sleep "${POLL_INTERVAL_SECONDS}"
  done
  echo "Error: timed out waiting for ${label}: ${result}"
  exit 1
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

wait_for_graphql "Write API" "${WRITE_API_SERVER_BASE_URL}" '{"query": "{ healthCheck }"}' '.data.healthCheck != null'
wait_for_graphql "Read API" "${READ_API_SERVER_BASE_URL}" '{"query": "{ __typename }"}' '.data.__typename == "QueryRoot"'

# グループチャット作成
run_mutation "Create GroupChat(${ADMIN_ID})" "$(jq -n \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation CreateGroupChat($input: CreateGroupChatInput!) { createGroupChat(input: $input) { groupChatId } }", variables: {input: {name: "group-chat-example", executorId: $executorId}}}')"
GROUP_CHAT_ID=$(echo "${MUTATION_RESULT}" | jq -r .data.createGroupChat.groupChatId)

# メンバー追加
run_mutation "Add Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}, ${ADMIN_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg userAccountId "${USER_ACCOUNT_ID}" \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation AddMember($input: AddMemberInput!) { addMember(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, userAccountId: $userAccountId, role: "MEMBER", executorId: $executorId}}}')"

# メッセージ投稿
run_mutation "Post Message(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg executorId "${USER_ACCOUNT_ID}" \
  '{query: "mutation PostMessage($input: PostMessageInput!) { postMessage(input: $input) { groupChatId, messageId } }", variables: {input: {groupChatId: $groupChatId, content: "Text1", executorId: $executorId}}}')"
MESSAGE_ID=$(echo "${MUTATION_RESULT}" | jq -r .data.postMessage.messageId)

# グループチャット取得
wait_for_read "Get GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID})" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${ADMIN_ID}" \
    '{query: "query GetGroupChat($groupChatId: String!, $userAccountId: String!) { getGroupChat(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, name, ownerId, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getGroupChat.id != null'

# グループチャットリスト取得
wait_for_read "Get GroupChats(${ADMIN_ID})" \
  "$(jq -n --arg userAccountId "${ADMIN_ID}" \
    '{query: "query GetGroupChats($userAccountId: String!) { getGroupChats(userAccountId: $userAccountId) { id, name, ownerId, createdAt, updatedAt } }", variables: {userAccountId: $userAccountId}}')" \
  '.data.getGroupChats | length > 0'

# メンバー取得
wait_for_read "Get Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID})" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMember($groupChatId: String!, $userAccountId: String!) { getMember(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, groupChatId, userAccountId, role, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getMember.userAccountId != null'

wait_for_read "Get Members(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID})" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMembers($groupChatId: String!, $userAccountId: String!) { getMembers(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, groupChatId, userAccountId, role, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getMembers | length == 2'

# メッセージ取得
wait_for_read "Get Message(${MESSAGE_ID}, ${USER_ACCOUNT_ID})" \
  "$(jq -n --arg messageId "${MESSAGE_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMessage($messageId: String!, $userAccountId: String!) { getMessage(messageId: $messageId, userAccountId: $userAccountId) { id, groupChatId, text, createdAt, updatedAt } }", variables: {messageId: $messageId, userAccountId: $userAccountId}}')" \
  '.data.getMessage.text == "Text1"'

wait_for_read "Get Messages(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID})" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMessages($groupChatId: String!, $userAccountId: String!) { getMessages(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, groupChatId, text, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getMessages | length == 1'

# グループチャット名の変更
run_mutation "Rename GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation RenameGroupChat($input: RenameGroupChatInput!) { renameGroupChat(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, name: "group-chat-example-2", executorId: $executorId}}}')"

wait_for_read "Get GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}) after rename" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${ADMIN_ID}" \
    '{query: "query GetGroupChat($groupChatId: String!, $userAccountId: String!) { getGroupChat(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, name, ownerId, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getGroupChat.name == "group-chat-example-2"'

# メッセージ編集
run_mutation "Edit Message(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg messageId "${MESSAGE_ID}" \
  --arg executorId "${USER_ACCOUNT_ID}" \
  '{query: "mutation EditMessage($input: EditMessageInput!) { editMessage(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, messageId: $messageId, content: "Text2", executorId: $executorId}}}')"

wait_for_read "Get Message(${MESSAGE_ID}, ${USER_ACCOUNT_ID}) after edit" \
  "$(jq -n --arg messageId "${MESSAGE_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMessage($messageId: String!, $userAccountId: String!) { getMessage(messageId: $messageId, userAccountId: $userAccountId) { id, groupChatId, text, createdAt, updatedAt } }", variables: {messageId: $messageId, userAccountId: $userAccountId}}')" \
  '.data.getMessage.text == "Text2"'

# メッセージの削除
run_mutation "Delete Message(${GROUP_CHAT_ID}, ${MESSAGE_ID}, ${USER_ACCOUNT_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg messageId "${MESSAGE_ID}" \
  --arg executorId "${USER_ACCOUNT_ID}" \
  '{query: "mutation DeleteMessage($input: DeleteMessageInput!) { deleteMessage(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, messageId: $messageId, executorId: $executorId}}}')"

wait_for_read "Get Messages(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}) after delete" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMessages($groupChatId: String!, $userAccountId: String!) { getMessages(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, groupChatId, text, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getMessages | length == 0'

# メンバーの削除
run_mutation "Remove Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}, ${ADMIN_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg userAccountId "${USER_ACCOUNT_ID}" \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation RemoveMember($input: RemoveMemberInput!) { removeMember(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, userAccountId: $userAccountId, executorId: $executorId}}}')"

wait_for_read "Get Members(${GROUP_CHAT_ID}, ${ADMIN_ID}) after remove" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${ADMIN_ID}" \
    '{query: "query GetMembers($groupChatId: String!, $userAccountId: String!) { getMembers(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, groupChatId, userAccountId, role, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getMembers | length == 1'

# ルームの削除
run_mutation "Delete GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation DeleteGroupChat($input: DeleteGroupChatInput!) { deleteGroupChat(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, executorId: $executorId}}}')"

wait_for_error_code "Get GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}) after delete" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${ADMIN_ID}" \
    '{query: "query GetGroupChat($groupChatId: String!, $userAccountId: String!) { getGroupChat(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, name, ownerId, createdAt, updatedAt } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  "404"
