#!/bin/sh

set -ue

curl -s -X POST \
	-H "Content-Type: application/json" \
	-d "{\"name\":\"test\",\"executor_id\":{\"value\":\"01H4J5WDZDXYJ4NWRDT5AR1J6E\"}}" \
	${WRITE_API_SERVER_BASE_URL}/threads/create