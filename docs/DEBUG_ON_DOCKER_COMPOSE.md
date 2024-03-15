# Debugging on Docker Compose

## Build the image

```shell
$ makers docker-build-local-all
```

## Start docker-compose

```shell
$ makers docker-compose-up
```

The required database and tables will be created and the application will be started.
If you want to start only the database for development purposes, run `docker-compose-up-db` instead
of `docker-compose-up`.

### Stop docker-compose

```shell
$ makers docker-compose-down
```

## Verification

```shell
$ makers verify-group-chat
```



