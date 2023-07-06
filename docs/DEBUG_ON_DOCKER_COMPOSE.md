# Debug on Docker Compose

## Build image

```shell
$ ./tools/scripts/docker-build-all.sh
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = us-east-1
PREFIX           = om2eep1k
APPLICATION_NAME = ceer
----------------------------------------
...
```

The image name is `ceer-write-api-server` as fixed.

## Run the Docker Compose

```shell
$ ./tools/scripts/docker-compose-up.sh
```

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
{"threadId":"01GBRWPCHEZKHX8QCR3226AGAM"}
```


