# Local runtime configuration

[日本語](LOCAL_CONFIGURATION.ja.md)

This reference covers the Write API, local Read Model Updater (RMU), and Read API using LocalStack DynamoDB and MySQL.

## Ports and endpoints

| Application | Host port in Compose | Container port | Direct host process, TOML value | GraphQL |
| --- | --- | --- | --- | --- |
| Write API | 18080 | 8080 | 18080 | `http://localhost:18080/query` |
| Local RMU | 18081 mapping only | 8080 mapping only | No HTTP listener | None |
| Read API | 18082 | 8080 | 18082 | `http://localhost:18082/query` |

Both APIs accept GraphQL POST requests at `/query` and serve GraphiQL on GET at the same path. The RMU consumes the `head` table's DynamoDB Streams, reads journal events, and updates the MySQL read model and `read_model_positions`. It reconciles heads at startup. It does not serve HTTP or GraphQL. Compose's `18081:8080` mapping and the RMU TOML's `api` section do not create a listener. The local entry point is `bin/local.rs`, not the Lambda `src/main.rs`.

## Configuration sources and required values

Each loader reads its optional `config/<application>.toml` relative to the process working directory, then overrides it with `APP__` environment variables. Double underscores separate nested keys. Run host processes from the repository root to use these files. The final Docker images do not copy the TOML files, so Compose supplies required settings through environment variables.

In the tables, **required** means the merged configuration must contain the value, not that you must export it when TOML supplies it. **Optional** means the Rust field accepts absence or has a code default. TOML values are file defaults, not built-in fallbacks. A value shown as “same” is explicitly supplied by Compose. “Absent” means Compose leaves the variable unset.

### API settings

These variables apply to Write and Read only. RMU ignores `APP__API__HOST` and `APP__API__PORT` even though Compose supplies them.

| Variable | Meaning and requirement | TOML value | Compose value |
| --- | --- | --- | --- |
| `APP__API__HOST` | Required bind IP address | `0.0.0.0` | Same |
| `APP__API__PORT` | Required listener port, not the host port mapping | Write `18080`, Read `18082` | `8080` |
| `APP__API__ALLOW_ORIGINS` | Required CORS origin list, comma-separated in the environment | `http://127.0.0.1:18080`, `http://127.0.0.1:18082` | `http://localhost:18080,http://localhost:18082,http://localhost:8888` |

### Write persistence settings

All are required and apply only to Write. Compose explicitly supplies the same values as TOML.

| Variable | Meaning | TOML and Compose value |
| --- | --- | --- |
| `APP__PERSISTENCE__JOURNAL_TABLE_NAME` | DynamoDB event journal table | `journal` |
| `APP__PERSISTENCE__SNAPSHOT_TABLE_NAME` | DynamoDB snapshot table | `snapshot` |
| `APP__PERSISTENCE__HEAD_TABLE_NAME` | DynamoDB aggregate head table | `head` |
| `APP__PERSISTENCE__SNAPSHOT_HISTORY_INDEX_NAME` | Snapshot history GSI | `snapshot-active-history-index` |
| `APP__PERSISTENCE__SNAPSHOT_INTERVAL` | Event interval for snapshots | `10` |

Table and GSI names must match the DynamoDB setup. RMU's head and journal names must match Write's names.

### AWS settings

These variables apply to Write and RMU. Read does not use AWS, including the `aws` section in its TOML.

| Variable | Meaning and requirement | TOML value | Compose value |
| --- | --- | --- | --- |
| `APP__AWS__REGION_NAME` | Required region fallback after the AWS SDK default region provider chain | `ap-northeast-1` | Host `${AWS_REGION}` interpolation |
| `APP__AWS__ENDPOINT_URL` | Optional AWS endpoint override, SDK endpoint resolution if absent | `http://localhost:4566` | `http://localstack:4566` |
| `APP__AWS__ACCESS_KEY_ID` | Optional explicit access key, used together with secret | `x` | Same |
| `APP__AWS__SECRET_ACCESS_KEY` | Optional explicit secret, used together with access key | `x` | Same |

The `x` credentials are local dummy values. If both credential fields are present, the application installs a static credentials provider. Otherwise it uses the SDK credential provider chain. An unset environment variable does not remove a value already supplied by TOML.

The Compose start and e2e scripts default the host `AWS_REGION` to `ap-northeast-1` and `AWS_DEFAULT_REGION` to that region. Direct Compose invocation needs host `AWS_REGION` for interpolation. The service-level `AWS_REGION: ap-northeast-1` is a separate SDK environment setting. `localhost:4566` reaches LocalStack from the host; containers use `localstack:4566`.

### RMU stream settings

The local RMU requires a `stream` section. The Lambda updater entry point also requires this section. Write and Read do not use it. Head and journal names are required within that section. Other fields have Rust defaults.

| Variable | Meaning | TOML value or Rust default | Compose value |
| --- | --- | --- | --- |
| `APP__STREAM__HEAD_TABLE_NAME` | Head table whose stream is consumed and whose heads are scanned | `head` in TOML | `head` |
| `APP__STREAM__JOURNAL_TABLE_NAME` | Journal table for event replay | `journal` in TOML | `journal` |
| `APP__STREAM__MAX_ITEM_COUNT` | Maximum records per Streams read | `100` in TOML and Rust | `32` |
| `APP__STREAM__POLL_INTERVAL_MILLIS` | Initial idle polling delay in milliseconds | `1000` in TOML and Rust | Absent, Rust default |
| `APP__STREAM__POLL_MAX_INTERVAL_MILLIS` | Maximum idle polling delay in milliseconds | `30000` in Rust | Absent, Rust default |
| `APP__STREAM__MAX_EVENTS_PER_TRANSACTION` | Events committed per transaction | `1` in Rust | Absent, Rust default |
| `APP__STREAM__RETRY_MAX_ATTEMPTS` | Maximum attempts per record | `5` in Rust | Absent, Rust default |
| `APP__STREAM__RETRY_MAX_ELAPSED_MILLIS` | Elapsed-time budget in milliseconds, checked before scheduling another retry. Running operations are not cancelled and can exceed this budget. `0` disables the time limit | `30000` in Rust | Absent, Rust default |
| `APP__STREAM__RETRY_INITIAL_DELAY_MILLIS` | Initial retry delay in milliseconds | `200` in Rust | Absent, Rust default |
| `APP__STREAM__RETRY_MAX_DELAY_MILLIS` | Maximum retry delay in milliseconds | `5000` in Rust | Absent, Rust default |

### Read model database

`APP__DATABASE__URL` is required for RMU and Read. Both must point to the same migrated MySQL database. Write does not use this setting.

| Source | Value |
| --- | --- |
| Host TOML for RMU and Read | `mysql://ceer:ceer@localhost:3306/ceer` |
| Compose for RMU and Read | `mysql://ceer:ceer@mysql-local:3306/ceer` |

The URL contains user, password, host, port, and database. Compose creates user `ceer`, password `ceer`, and database `ceer`, publishes MySQL on host `3306`, and runs migrations through `refinery`. The `DATABASE_URL` in the migration service is separate from the applications' `APP__DATABASE__URL`. The `redis` TOML sections are not consumed by these applications.

## Existing procedures

- [Build and test](BUILD_AND_TEST.md) covers the shared environment setup.
- [Build application and e2e images](DEBUG_ON_DOCKER_COMPOSE.md#build-the-image).
- [Start Compose](DEBUG_ON_DOCKER_COMPOSE.md#start-docker-compose).
- [Stop Compose](DEBUG_ON_DOCKER_COMPOSE.md#stop-docker-compose).
- [Run e2e, including RMU restart verification](DEBUG_ON_DOCKER_COMPOSE.md#e2e).
- [Run and debug host processes](DEBUG_ON_LOCAL_MACHINE.md).

## Source references

Values were checked against main `37884e12609924db6b1428136627985fb050d6e1`.

- [`config/write-api-server.toml`](../config/write-api-server.toml)
- [`config/read-model-updater.toml`](../config/read-model-updater.toml)
- [`config/read-api-server.toml`](../config/read-api-server.toml)
- [`applications/write-api-server/src/main.rs`](../applications/write-api-server/src/main.rs)
- [`applications/read-api-server/src/lib.rs`](../applications/read-api-server/src/lib.rs)
- [`applications/read-api-server/src/main.rs`](../applications/read-api-server/src/main.rs)
- [`applications/read-model-updater/src/lib.rs`](../applications/read-model-updater/src/lib.rs)
- [`applications/read-model-updater/bin/local.rs`](../applications/read-model-updater/bin/local.rs)
- [`applications/write-api-server/Dockerfile`](../applications/write-api-server/Dockerfile)
- [`applications/read-api-server/Dockerfile`](../applications/read-api-server/Dockerfile)
- [`applications/read-model-updater/Dockerfile.local`](../applications/read-model-updater/Dockerfile.local)
- [`modules/command/interface-adaptor-impl/src/controllers.rs`](../modules/command/interface-adaptor-impl/src/controllers.rs)
- [`modules/query/interface-adaptor/src/controllers.rs`](../modules/query/interface-adaptor/src/controllers.rs)
- [`tools/docker-compose/docker-compose-applications.yml`](../tools/docker-compose/docker-compose-applications.yml)
- [`tools/docker-compose/docker-compose-databases.yml`](../tools/docker-compose/docker-compose-databases.yml)
- [`tools/dynamodb-setup/create-tables.sh`](../tools/dynamodb-setup/create-tables.sh)
- [`tools/scripts/docker-compose-up.sh`](../tools/scripts/docker-compose-up.sh)
- [`tools/scripts/docker-compose-e2e-test.sh`](../tools/scripts/docker-compose-e2e-test.sh)
