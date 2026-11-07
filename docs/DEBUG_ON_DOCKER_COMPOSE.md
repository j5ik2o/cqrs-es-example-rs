# Debugging on Docker Compose

## Build the image

```shell
$ makers docker-build-local-all
```

When running the e2e, build the verification image as well.

```shell
$ makers docker-build-e2e-test
```

## Start docker-compose

```shell
$ makers docker-compose-up
```

This starts LocalStack, MySQL, the migration, the Write API, the Read API, and the local Read Model Updater. The updater subscribes to the aggregate head DynamoDB Streams and updates the read model and `read_model_positions`.

If you want to start only the database for development purposes, run `docker-compose-up-db` instead of `docker-compose-up`.

The Lambda Read Model Updater is not deployed by default. Deploy it manually with:

```shell
$ makers deploy-read-model-updater-localstack
```

To deploy the Lambda when `docker-compose-up` starts, set `DOCKER_COMPOSE_UP_DEPLOY_LAMBDA=1`. The host-side variable for LocalStack services is `LOCALSTACK_SERVICES`; its default value includes `lambda`. The e2e script excludes `lambda` and starts only DynamoDB and Streams.

```shell
$ DOCKER_COMPOSE_UP_DEPLOY_LAMBDA=1 makers docker-compose-up
```

### Stop docker-compose

```shell
$ makers docker-compose-down
```

## e2e

```shell
$ makers docker-compose-e2e-test
```

The scenario checks the following flow.

1. Write a group chat, a member, and a message through the Write API.
2. Wait until the Read Model Updater processes the head Streams and updates the MySQL read model.
3. Check that the six Read API queries return the written data.
4. Restart the Read Model Updater.
5. Write a new group chat and message after the restart and check that the read model reflects them. Also check that the existing `read_model_positions` values do not change.

Waits poll the GraphQL responses instead of relying on a fixed sleep. A 404 is retried as "not projected yet"; any other GraphQL error fails immediately. On failure, the script prints the e2e and application logs.

## Check the logs

```shell
$ docker compose -p cqrs-es-example-rs \
    -f tools/docker-compose/docker-compose-databases.yml \
    -f tools/docker-compose/docker-compose-applications.yml \
    logs --no-color --tail=200 read-model-updater-1
```
