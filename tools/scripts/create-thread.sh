#!/bin/sh

set -ue

PORT=${PORT:-8080}
HOST_URL=${HOST_URL:-http://localhost}

curl -X POST -H "Content-Type: application/json" -d "{\"name\":\"test\",\"executor_id\":{\"value\":\"01H4J5WDZDXYJ4NWRDT5AR1J6E\"}}" ${HOST_URL}:${PORT}/threads/create