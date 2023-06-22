#!/bin/sh

curl -X POST -H "Content-Type: application/json" -d "{\"name\":\"test\",\"executor_id\":{\"value\":\"01H4J5WDZDXYJ4NWRDT5AR1J6E\"}}" http://localhost:8080/threads/create