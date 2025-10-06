## Copy and edit `common.env.default` as `common.env

```shell
$ cp common.env.default common.env
```

PREFIX of environment variables (All alphanumeric characters. Change PREFIX (all alphanumeric characters, lower case
only) and APPLICATION_NAME as appropriate.
It is recommended to generate the PREFIX with `pwgen -A`.

```shell
$ pwgen -A
```

If you do not have `pwgen`, install it with `brew install pwgen`.

## ビルド方法

Be sure to start Docker before performing the build.

Start DB only with the following command before performing the build. (to allow sqlx to connect to the DB)

```shell
$ makers docker-compose-up-db
```

```shell
$ makers build
```

Note: If you add or modify SQL to be issued by sqlx, you will need to run `cargo sqlx prepare` directly under each
project (a JSON file will be generated in `.sqlx/`), but you can set up `cargo sqlx prepare` is set to run.
So, please be sure to run `makers build` when you add or modify SQL. Please include the generated `.sqlx/*.json` file
under git's control. At this time, you need to connect to the DB server, so you need to
run `makers docker-compose-up-db`.

## テスト方法

Be sure to start Docker before running the test.

```shell
$ makers test
```

Note: You can also test with `cargo test`, but this time the number of tests to run simultaneously is limited by an
environment variable (`RUST_TEST_THREADS=1`) due to the use of testcontainers.
In other words, please note that tests cannot be run properly unless `RUST_TEST_THREADS=1 cargo test` is used.
With `makers test`, the default is `RUST_TEST_THREADS=1`, so you can run the test as it is.
## Deploying the Lambda to LocalStack

To emulate the DynamoDB Streams → Lambda → MySQL flow locally, run the following:

```shell
$ makers build-read-model-updater-lambda
$ makers deploy-read-model-updater-localstack
```

`build-read-model-updater-lambda` builds the Lambda-compatible binary inside Docker and places the ZIP at `dist/lambda/read-model-updater/bootstrap.zip`.
`deploy-read-model-updater-localstack` relies on the AWS CLI to create or update the function on LocalStack and wires it to the `journal` table stream.
Configuration values are read from `common.env`, so configure the LocalStack endpoint and dummy credentials before executing the commands.
