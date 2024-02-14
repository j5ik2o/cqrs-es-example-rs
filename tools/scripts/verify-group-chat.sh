#!/usr/bin/env bash

ADMIN_ID=01H42K4ABWQ5V2XQEP3A48VE0Z
ACCOUNT_ID=01H7C6DWMK1BKS1JYH1XZE529M

WRITE_API_SERVER_BASE_URL=http://localhost:18080/v1
READ_API_SERVER_BASE_URL=http://localhost:18082
ADMIN_ID=01H42K4ABWQ5V2XQEP3A48VE0Z
USER_ACCOUNT_ID=01H7C6DWMK1BKS1JYH1XZE529M

# グループチャット作成
echo -e "\nCreate GroupChat:"
CREATE_GROUP_CHAT_RESULT=$(curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/create" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"name\": \"group-chat-example-1\", \"executor_id\": \"${ADMIN_ID}\" }")
echo "Result: $CREATE_GROUP_CHAT_RESULT"
GROUP_CHAT_ID=$(echo $CREATE_GROUP_CHAT_RESULT | jq -r .group_chat_id)

# メンバー追加
echo -e "\nAdd Member:"
ADD_MEMBER_RESULT=$(curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/add-member" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"role\": \"member\", \"user_account_id\": \"${USER_ACCOUNT_ID}\", \"executor_id\": \"${ADMIN_ID}\" }")
echo "Result: $ADD_MEMBER_RESULT"

# メッセージ投稿
echo -e "\nPost Message:"
POST_MESSAGE_RESULT=$(curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/post-message" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"message\": \"Text1\", \"user_account_id\": \"${USER_ACCOUNT_ID}\", \"executor_id\": \"${USER_ACCOUNT_ID}\"  }")
echo "Result: $POST_MESSAGE_RESULT"
MESSAGE_ID=$(echo $POST_MESSAGE_RESULT | jq -r .message_id)

sleep 1

# グループチャット取得
group_chat=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChat(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"UserAccount-${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS)

echo -e "\nGet GroupChat:"
echo $group_chat | jq .

# グループチャットリスト取得
group_list_chat=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChats(userAccountId: \"UserAccount-${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS)

echo -e "\nGet GroupChats:"
echo $group_list_chat | jq .

# グループチャット名の変更
echo -e "\nRename GroupChat:"
curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/rename" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"name\": \"group-chat-example-2\", \"executor_id\": \"${ADMIN_ID}\" }"

sleep 1

# グループチャット取得
group_chat=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{ "query": "{ getGroupChat(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"UserAccount-${ADMIN_ID}\") { id, name, ownerId, createdAt, updatedAt } }" }
EOS)

echo -e "\nGet GroupChat:"
echo $group_chat | jq .

# メンバー取得
member=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS1
{ "query": "{ getMember(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"UserAccount-${USER_ACCOUNT_ID}\") { id, groupChatId, userAccountId, role, createdAt, updatedAt } }" }
EOS1)

echo -e "\nGet Member:"
echo $member | jq .

# メンバーリスト取得
member_list=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS3
{ "query": "{ getMembers(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"UserAccount-${USER_ACCOUNT_ID}\") { id, groupChatId, userAccountId, role, createdAt, updatedAt } }" }
EOS3)

echo -e "\nGet Members:"
echo $member_list | jq .

# メッセージ取得
message=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS2
{ "query": "{ getMessage(messageId: \"${MESSAGE_ID}\", userAccountId: \"UserAccount-${USER_ACCOUNT_ID}\") { id, groupChatId, text, createdAt, updatedAt } }" }
EOS2)

echo -e "\nGet Message(${MESSAGE_ID}, UserAccount-${USER_ACCOUNT_ID}):"
echo $message | jq .

# メッセージリスト取得
message_list=$(curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/query \
	-d @- <<EOS3
{ "query": "{ getMessages(groupChatId: \"${GROUP_CHAT_ID}\", userAccountId: \"UserAccount-${USER_ACCOUNT_ID}\") { id, groupChatId, text, createdAt, updatedAt } }" }
EOS3)

echo -e "\nGet Messages(${GROUP_CHAT_ID}, UserAccount-${USER_ACCOUNT_ID}):"
echo $message_list | jq .

# メッセージの削除
echo -e "\nDelete Message:"
curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/delete-message" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"message_id\": \"${MESSAGE_ID}\", \"executor_id\": \"${USER_ACCOUNT_ID}\"  }"

# メンバーの削除
echo -e "\nRemove Member:"
curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/remove-member" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"user_account_id\": \"${USER_ACCOUNT_ID}\", \"executor_id\": \"${ADMIN_ID}\"  }"

# ルームの削除
echo -e "\nDelete GroupChat:"
curl -s -X 'POST' \
  "${WRITE_API_SERVER_BASE_URL}/group-chats/delete" \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"executor_id\": \"${ADMIN_ID}\" }"
