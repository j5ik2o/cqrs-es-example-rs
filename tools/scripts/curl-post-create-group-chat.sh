#!/usr/bin/env bash

curl -s -X POST -H "Content-Type: application/json" \
	${WRITE_API_SERVER_BASE_URL}/query \
	-d @- <<EOS
{
  "query": "mutation CreateGroupChat(\$input: CreateGroupChatInput!) { createGroupChat(input: \$input) { groupChatId } }",
  "variables": {
    "input": {
      "name": "group-chat-example",
      "executorId": "01H42K4ABWQ5V2XQEP3A48VE0Z"
    }
  }
}
EOS