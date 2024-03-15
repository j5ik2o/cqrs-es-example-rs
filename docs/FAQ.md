## FAQ

## I want to check the API specification.

Please refer to [API_SPEC.md](API_SPEC.md).

## I want to check the operation

There are two ways to check the operation.

- [Start from docker-compose](. /DEBUG_ON_DOCKER_COMPOSE.md)
- [Start from IntelliJ](. /DEBUG_ON_LOCAL_MACHINE.md)

## I want to check the contents of the database

1. start the database with `makers docker-compose-up` or `makers docker-compose-up-db`.
1. open http://localhost:8003/ for DynamoDB
1. open http://localhost:4040/ for Aurora

## I want to know the list of tasks for cargo-make(makers)

```shell
$ makers --list-all-steps
```