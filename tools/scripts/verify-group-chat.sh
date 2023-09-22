#!/usr/bin/env bash

ADMIN_ID=01H42K4ABWQ5V2XQEP3A48VE0Z
ACCOUNT_ID=01H7C6DWMK1BKS1JYH1XZE529M

# グループチャット作成
GROUP_CHAT_ID=$(curl -s -X 'POST' \
  'http://localhost:18080/group-chats/create' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"executor_id\": \"${ADMIN_ID}\", \"name\": \"group-chat-example-1\" }" | jq -r .group_chat_id)

# メンバー追加
curl -s -X 'POST' \
  'http://localhost:18080/group-chats/add-member' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"executor_id\": \"${ADMIN_ID}\", \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"role\": \"member\", \"user_account_id\": \"${ACCOUNT_ID}\" }"

# メッセージ投稿1
curl -s -X 'POST' \
  'http://localhost:18080/group-chats/post-message' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"executor_id\": \"${ACCOUNT_ID}\", \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"message\": \"Text1\", \"user_account_id\": \"${ACCOUNT_ID}\" }"

# メッセージ投稿2
curl -s -X 'POST' \
  'http://localhost:18080/group-chats/post-message' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{ \"executor_id\": \"${ACCOUNT_ID}\", \"group_chat_id\": \"${GROUP_CHAT_ID}\", \"message\": \"Text2\", \"user_account_id\": \"${ACCOUNT_ID}\" }"
