#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

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
