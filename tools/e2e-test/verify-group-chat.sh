#!/usr/bin/env bash

set -ue

ADMIN_ID=${ADMIN_ID:-UserAccount-01H42K4ABWQ5V2XQEP3A48VE0Z}
USER_ACCOUNT_ID=${USER_ACCOUNT_ID:-UserAccount-01H7C6DWMK1BKS1JYH1XZE529M}
WRITE_API_SERVER_BASE_URL=${WRITE_API_SERVER_BASE_URL:-http://localhost:28080}
READ_API_SERVER_BASE_URL=${READ_API_SERVER_BASE_URL:-http://localhost:28082}

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
      "executorId": "UserAccount-01H42K4ABWQ5V2XQEP3A48VE0Z"
    }
  }
}
EOS
)

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

echo "Result: $POST_MESSAGE_RESULT"
MESSAGE_ID=$(echo $POST_MESSAGE_RESULT | jq -r .data.postMessage.messageId)

sleep 1

# グループチャット取得
group_chat=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChat(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
echo $group_chat | jq .

# グループチャットリスト取得
group_list_chat=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChats(userAccountId: \"${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet GroupChats(${ADMIN_ID}):"
echo $group_list_chat | jq .

# グループチャット名の変更
echo -e "\nRename GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
curl -s -X POST -H "Content-Type: application/json" \
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

sleep 1

# グループチャット取得
group_chat=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChat(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
echo $group_chat | jq .

# メンバー取得
member=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMember(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, userAccountId, role, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
echo $member | jq .

# メンバーリスト取得
member_list=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMembers(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, userAccountId, role, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet Members(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
echo $member_list | jq .

# メッセージ取得
message=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMessage(messageId: \"${MESSAGE_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, text, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet Message(${MESSAGE_ID}, ${USER_ACCOUNT_ID}):"
echo $message | jq .

# メッセージリスト取得
message_list=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getMessages(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"${USER_ACCOUNT_ID}\") { id, groupChatId, text, createdAt, updatedAt } }" }
EOS
)

echo -e "\nGet Messages(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}):"
echo $message_list | jq .

# メッセージの削除
echo -e "\nDelete Message(${GROUP_CHAT_ID}, ${MESSAGE_ID}, ${USER_ACCOUNT_ID}):"
curl -s -X POST -H "Content-Type: application/json" \
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

# メンバーの削除
echo -e "\nRemove Member(${GROUP_CHAT_ID}, ${USER_ACCOUNT_ID}, ${ADMIN_ID}):"
curl -s -X POST -H "Content-Type: application/json" \
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

# ルームの削除
echo -e "\nDelete GroupChat(${GROUP_CHAT_ID}, ${ADMIN_ID}):"
curl -s -X POST -H "Content-Type: application/json" \
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
