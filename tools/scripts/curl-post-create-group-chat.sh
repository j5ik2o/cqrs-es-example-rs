#!/usr/bin/env bash

curl -s -X 'POST' \
    "${WRITE_API_SERVER_BASE_URL}/group-chats/create" \
    -H 'accept: application/json' \
    -H 'Content-Type: application/json' \
    -d '{ "executor_id": "01H42K4ABWQ5V2XQEP3A48VE0Z", "name": "group-chat-example" }'