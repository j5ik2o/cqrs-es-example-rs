#!/bin/sh

curl -s -X POST -H "Content-Type: application/json" \
	${READ_API_SERVER_BASE_URL}/graphql \
	-d @- <<EOS
{ "query": "{ getThread(threadId: \"${THREAD_ID}\") { id } }" }
EOS