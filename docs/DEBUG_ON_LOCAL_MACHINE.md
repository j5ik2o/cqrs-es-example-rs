# Debug on local machine.

## Run only database in docker-compose.

```shell
$ makers docker-compose-up-db
```

## Debug using IntelliJ IDEA.

The application is not running, so start it and debug it if necessary.

- write-api-server
    - `src/main.rs`
- read-model-updater
    - Since `src/main.rs` is for AWS Lambda, use `bin/local.rs` locally.
- read-api-server
    - `src/main.rs`.

If you start without specifying environment variables, it will start with the value of the configuration file under
config/. The HTTP port number is the same as that of docker-compose, so you can use the following commands to check the
operation.

## check

```shell
$ makers verify-group-chat
```