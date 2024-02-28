#!/usr/bin/env bash

set -ue

ADMIN_ID=${ADMIN_ID:-UserAccount-01H42K4ABWQ5V2XQEP3A48VE0Z}
USER_ACCOUNT_ID=${USER_ACCOUNT_ID:-UserAccount-01H7C6DWMK1BKS1JYH1XZE529M}
WRITE_API_SERVER_BASE_URL=${WRITE_API_SERVER_BASE_URL:-http://localhost:38080}
READ_API_SERVER_BASE_URL=${READ_API_SERVER_BASE_URL:-http://localhost:38082}

# グループチャット作成
echo -e "\nCreate GroupChat(${ADMIN_ID}):"
CREATE_GROUP_CHAT_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation CreateGroupChat(\$input: CreateGroupChatInput!) { createGroupChat(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "name": "group-chat-example",
      "executorId": "${ADMIN_ID}"
    }
  }
}
EOS
)

if echo $CREATE_GROUP_CHAT_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $CREATE_GROUP_CHAT_RESULT"
  exit 1
fi

echo "Result: $CREATE_GROUP_CHAT_RESULT"

GROUP_CHAT_ID=$(echo $CREATE_GROUP_CHAT_RESULT | jq -r .data.createGroupChat.groupChatId)

# メンバー追加
echo -e "\nAdd Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}, ${ADMIN_ID}):"
ADD_MEMBER_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation AddMember(\$input: AddMemberInput!) { addMember(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "groupChatId": "${GROUP_CHAT_ID}",
      "userAccountId": "${USER_ACCOUNT_ID}",
      "role": "MEMBER",
      "executorId": "${ADMIN_ID}"
    }
  }
}
EOS
)

if echo $ADD_MEMBER_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $ADD_MEMBER_RESULT"
  exit 1
fi

echo "Result: $ADD_MEMBER_RESULT"

# メッセージ投稿
echo -e "\nPost Message(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
POST_MESSAGE_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation PostMessage(\$input: PostMessageInput!) { postMessage(input: \$input) { groupChatId, messageId } }",
  "variables": {
    "input": {
      "groupChatId": "${GROUP_CHAT_ID}",
      "content": "Text1",
      "executorId": "${USER_ACCOUNT_ID}"
    }
  }
}
EOS
)

if echo $POST_MESSAGE_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $POST_MESSAGE_RESULT"
  exit 1
fi

echo "Result: $POST_MESSAGE_RESULT"

MESSAGE_ID=$(echo $POST_MESSAGE_RESULT | jq -r .data.postMessage.messageId)

sleep 1

# グループチャット取得
echo -e "\nGet GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
GET_GROUP_CHAT_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChat(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS
)

if echo $GET_GROUP_CHAT_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_GROUP_CHAT_RESULT"
  exit 1
fi

echo "Result: $GET_GROUP_CHAT_RESULT"

# グループチャットリスト取得
echo -e "\nGet GroupChats(${ADMIN_ID}):"
GET_GROUP_CHATS_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChats(userAccountId: \"${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS
)

if echo $GET_GROUP_CHATS_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_GROUP_CHATS_RESULT"
  exit 1
fi

echo "Result: $GET_GROUP_CHATS_RESULT"

# グループチャット名の変更
echo -e "\nRename GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
RENAME_GROUP_CHAT_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation RenameGroupChat(\$input: RenameGroupChatInput!) { renameGroupChat(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "groupChatId": "${GROUP_CHAT_ID}",
      "name": "group-chat-example-2",
      "executorId": "${ADMIN_ID}"
    }
  }
}
EOS
)

if echo $RENAME_GROUP_CHAT_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $RENAME_GROUP_CHAT_RESULT"
  exit 1
fi

echo "Result: $RENAME_GROUP_CHAT_RESULT"

sleep 1

# グループチャット取得
echo -e "\nGet GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
GET_GROUP_CHAT_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChat(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS
)

if echo $GET_GROUP_CHAT_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_GROUP_CHAT_RESULT "
  exit 1
fi

echo "Result: $GET_GROUP_CHAT_RESULT"

# メンバー取得
echo -e "\nGet Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
GET_MEMBER_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMember(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, userAccountId, role, createdAt, updatedAt } }" }
EOS
)

if echo $GET_MEMBER_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_MEMBER_RESULT "
  exit 1
fi

echo "Result: $GET_MEMBER_RESULT"

# メンバーリスト取得
echo -e "\nGet Members(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
GET_MEMBERS_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMembers(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, userAccountId, role, createdAt, updatedAt } }" }
EOS
)

if echo $GET_MEMBERS_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_MEMBERS_RESULT "
  exit 1
fi

echo "Result: $GET_MEMBERS_RESULT"

# メッセージ取得
echo -e "\nGet Message(${MESSAGE_ID}, ${USER_ACCOUNT_ID}):"
GET_MESSAGE_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMessage(messageId: \"${MESSAGE_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, text, createdAt, updatedAt } }" }
EOS
)

if echo $GET_MESSAGE_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_MESSAGE_RESULT "
  exit 1
fi

echo "Result: $GET_MESSAGE_RESULT"

# メッセージリスト取得
echo -e "\nGet Messages(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
GET_MESSAGES_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMessages(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, text, createdAt, updatedAt } }" }
EOS
)

if echo $GET_MESSAGES_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $GET_MESSAGES_RESULT"
  exit 1
fi

echo "Result: $GET_MESSAGES_RESULT"

# メッセージの削除
echo -e "\nDelete Message(${GROUP_CHAT_ID}, ${MESSAGE_ID}, ${USER_ACCOUNT_ID}):"
DELETE_MESSAGE_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation DeleteMessage(\$input: DeleteMessageInput!) { deleteMessage(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "groupChatId": "${GROUP_CHAT_ID}",
      "messageId": "${MESSAGE_ID}",
      "executorId": "${USER_ACCOUNT_ID}"
    }
  }
}
EOS
)

if echo $DELETE_MESSAGE_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $DELETE_MESSAGE_RESULT"
  exit 1
fi

echo "Result: $DELETE_MESSAGE_RESULT"

# メンバーの削除
echo -e "\nRemove Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}, ${ADMIN_ID}):"
REMOVE_MEMBER_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation RemoveMember(\$input: RemoveMemberInput!) { removeMember(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "groupChatId": "${GROUP_CHAT_ID}",
      "userAccountId": "${USER_ACCOUNT_ID}",
      "executorId": "${ADMIN_ID}"
    }
  }
}
EOS
)

if echo $REMOVE_MEMBER_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $REMOVE_MEMBER_RESULT"
  exit 1
fi

echo "Result: $REMOVE_MEMBER_RESULT"

# ルームの削除
echo -e "\nDelete GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
DELETE_GROUP_CHAT_RESULT=$(curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation DeleteGroupChat(\$input: DeleteGroupChatInput!) { deleteGroupChat(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "groupChatId": "${GROUP_CHAT_ID}",
      "executorId": "${ADMIN_ID}"
    }
  }
}
EOS
)

if echo $DELETE_GROUP_CHAT_RESULT | jq -e .errors > /dev/null; then
  echo "Error: $DELETE_GROUP_CHAT_RESULT"
  exit 1
fi

echo "Result: $DELETE_GROUP_CHAT_RESULT"