# Debug on Local Machine

## Run the Docker Compose for DynamoDB Local & DynamoDB Admin

Launch dynamodb-local & dynamodb-admin as docker-compose.

```shell
$ ./tools/scripts/docker-compose-up.sh -d
```

## Debug by using IntelliJ IDEA

Create a configuration for all three and run it in IntelliJ IDEA. If you want to debug, run any one of the projects in
debug.

## Operation verification

### Check the applications

Check the operation with the following commands.

```shell
$ curl -s -X GET http://localhost:18080/hello
Hello World!
```

Call API to check operation.

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ \"accountId\": \"01G41J1A2GVT5HE45AH7GP711P\" }" http://127.0.0.1:18080/threads
{"threadId":"01GBCN25M496HB4PK9EWQMH28J"}
```
