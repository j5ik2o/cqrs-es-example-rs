#!/usr/bin/env bash

# Runs after the read-model-updater has been restarted. The write below is a
# barrier: head stream records are ordered per aggregate, so once this event is
# projected the updater has replayed every record written before the restart.

set -euo pipefail

source "$(dirname "$0")/common.sh"

wait_for_graphql "Write API" "${WRITE_API_SERVER_BASE_URL}" '{"query": "{ healthCheck }"}' '.data.healthCheck != null'
wait_for_graphql "Read API" "${READ_API_SERVER_BASE_URL}" '{"query": "{ __typename }"}' '.data.__typename == "QueryRoot"'

run_mutation "Create GroupChat after RMU restart(${ADMIN_ID})" "$(jq -n \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation CreateGroupChat($input: CreateGroupChatInput!) { createGroupChat(input: $input) { groupChatId } }", variables: {input: {name: "group-chat-after-restart", executorId: $executorId}}}')"
GROUP_CHAT_ID=$(echo "${MUTATION_RESULT}" | jq -r .data.createGroupChat.groupChatId)
if [ -z "${GROUP_CHAT_ID}" ] || [ "${GROUP_CHAT_ID}" = "null" ]; then
  echo "Error: createGroupChat returned no groupChatId: ${MUTATION_RESULT}"
  exit 1
fi

run_mutation "Add Member after RMU restart(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}, ${ADMIN_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg userAccountId "${USER_ACCOUNT_ID}" \
  --arg executorId "${ADMIN_ID}" \
  '{query: "mutation AddMember($input: AddMemberInput!) { addMember(input: $input) { groupChatId } }", variables: {input: {groupChatId: $groupChatId, userAccountId: $userAccountId, role: "MEMBER", executorId: $executorId}}}')"

run_mutation "Post Message after RMU restart(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID})" "$(jq -n \
  --arg groupChatId "${GROUP_CHAT_ID}" \
  --arg executorId "${USER_ACCOUNT_ID}" \
  '{query: "mutation PostMessage($input: PostMessageInput!) { postMessage(input: $input) { groupChatId, messageId } }", variables: {input: {groupChatId: $groupChatId, content: "Text-after-restart", executorId: $executorId}}}')"
MESSAGE_ID=$(echo "${MUTATION_RESULT}" | jq -r .data.postMessage.messageId)
if [ -z "${MESSAGE_ID}" ] || [ "${MESSAGE_ID}" = "null" ]; then
  echo "Error: postMessage returned no messageId: ${MUTATION_RESULT}"
  exit 1
fi

wait_for_read "Get GroupChat after RMU restart(${GROUP_CHAT_ID}, ${ADMIN_ID})" \
  "$(jq -n --arg groupChatId "${GROUP_CHAT_ID}" --arg userAccountId "${ADMIN_ID}" \
    '{query: "query GetGroupChat($groupChatId: String!, $userAccountId: String!) { getGroupChat(groupChatId: $groupChatId, userAccountId: $userAccountId) { id, name } }", variables: {groupChatId: $groupChatId, userAccountId: $userAccountId}}')" \
  '.data.getGroupChat.name == "group-chat-after-restart"'

wait_for_read "Get Message after RMU restart(${MESSAGE_ID}, ${USER_ACCOUNT_ID})" \
  "$(jq -n --arg messageId "${MESSAGE_ID}" --arg userAccountId "${USER_ACCOUNT_ID}" \
    '{query: "query GetMessage($messageId: String!, $userAccountId: String!) { getMessage(messageId: $messageId, userAccountId: $userAccountId) { id, text } }", variables: {messageId: $messageId, userAccountId: $userAccountId}}')" \
  '.data.getMessage.text == "Text-after-restart"'

echo "After-restart write was projected: GROUP_CHAT_ID=${GROUP_CHAT_ID} MESSAGE_ID=${MESSAGE_ID}"
