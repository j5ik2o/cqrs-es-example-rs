## Setup for AWS

## Add an AWS profile

Edit `~/.aws/credentials` as follows

```
# ...
[ceer]
aws_access_key_id=xxxxxxx
aws_secret_access_key=xxxxxxx
aws_session_token=xxxxxxx
````

Or, for AWS SSO, edit `~/.aws/config` as follows.

````
[profile ceer]
sso_start_url = https://xxxxxx.awsapps.com/start
sso_region = ap-northeast-1
sso_account_id = 1234567890 
sso_role_name = xxxxxxxx
region = ap-northeast-1
````shell

````shell
$ aws sso login --profile ceer
````

Log in with SSO as ````shell $ aws sso login --profile ceer ```.

# Build AWS environment with terraform

Create a new file ``tools/deploy/terraform/${PREFIX}-${APPLICATION_NAME}-terraform.tfvars`` with the following contents.
Changes defined in variables.tf can be overwritten in this `tfvars` file.
You can create only the resources you need. For example, if you only need ecr, just set `ecr_enabled = true`.

````
event_sourcing_enabled = true
event_sourcing_journal_name = "journal"
event_sourcing_snapshot_history_index_name = "snapshot-active-history-index"
event_sourcing_snapshot_name = "snapshot"
event_sourcing_head_name = "head"

eks_enabled = true
eks_version = "1.71"
eks_auth_roles = []
eks_auth_users = []
eks_auth_accounts = []

ecr_enabled = true

datadog-api-key = "xxxx"

read_model_updater_tag = "9ed584699fe19cab82121fae2d4ac7f1eee2e49089ba463cdd7378085ccc7b39-amd64"
```

## Three-table layout and existing data

ESA v5 uses three DynamoDB tables, `journal`, `snapshot`, and `head`, and every table is keyed by `aid`.
The hash and range keys of `journal` and `snapshot` change, so applying this configuration to an
existing environment replaces both tables. Existing events are not kept.

The journal table in `event-sourcing/main.tf` sets `prevent_destroy = true` so Terraform cannot remove
the events without an explicit change. Remove it deliberately when you migrate or reset the
environment.

### Back up before applying

Back up the existing `journal` and `snapshot` tables whether you migrate or reset.

```shell
aws dynamodb create-backup --table-name <journal> --backup-name <prefix>-journal-before-v5
aws dynamodb create-backup --table-name <snapshot> --backup-name <prefix>-snapshot-before-v5
```

If point-in-time recovery is enabled, note a restore point as well.

### Migrate to the v5 layout while keeping data

To keep the existing events, create the new tables under different names, rewrite the items, and
then switch the applications to the new names.

1. Choose new physical names, for example `journal-v5`, `snapshot-v5`, and `head`.
2. Remove the old tables from Terraform state without deleting them.
   ```shell
   terraform state rm 'module.event_sourcing.aws_dynamodb_table.journal-table[0]'
   terraform state rm 'module.event_sourcing.aws_dynamodb_table.snapshot-table[0]'
   ```
   The old environment has no `head` table, so there is nothing to remove for it.
3. Change the `event_sourcing_*` values in the tfvars file.
   ```
   event_sourcing_journal_name                = "journal-v5"
   event_sourcing_snapshot_name               = "snapshot-v5"
   event_sourcing_snapshot_history_index_name = "snapshot-active-history-index"
   event_sourcing_head_name                   = "head"
   ```
4. Run `makers terraform-apply` to create the three new tables.
5. Rewrite the items from the old journal and snapshot into the new tables. The `aid` must be
   `${type-name}-${value}`, and each event must use the v5 envelope (`seq_nr`, `occurred_at`,
   `manifest`, `payload`). Create one head item per aggregate. This sample does not include a
   migration script. Follow the migration procedure in
   `docs/spec/storage/dynamodb.ja.md` of the ESA repository.
6. Switch the applications to the new names.
   - Write API: `APP__PERSISTENCE__JOURNAL_TABLE_NAME`, `APP__PERSISTENCE__SNAPSHOT_TABLE_NAME`,
     `APP__PERSISTENCE__HEAD_TABLE_NAME`, `APP__PERSISTENCE__SNAPSHOT_HISTORY_INDEX_NAME`
   - RMU: `APP__STREAM__HEAD_TABLE_NAME`, `APP__STREAM__JOURNAL_TABLE_NAME`
   - The Lambda managed by Terraform sets these from the `event_sourcing_*` values.
7. Start the Write API and the RMU against the new tables and confirm that the read model is updated.
8. Delete the old tables after the check.

You do not need to rebuild the MySQL read model. The processed position stays in
`read_model_positions`, so keep it consistent with the migrated `seq_nr` values. If it is not
consistent, initialize the read model and `read_model_positions` before restarting the RMU.

### Reset the data

If you do not need the existing data, recreate the tables.

1. Remove `prevent_destroy = true` from the journal `lifecycle` block in `event-sourcing/main.tf`.
2. Run `makers terraform-apply`. Terraform replaces `journal` and `snapshot` and creates `head`.
3. Initialize the MySQL read model. Empty `group_chats`, `members`, `messages`, and
   `read_model_positions`, or recreate the database and run the refinery migrations. An old position
   makes the RMU treat new events as already processed.
4. Restart the Write API and the RMU. The head Streams are recreated, so recreate the Lambda event
   source mapping as well.
5. The LocalStack compose setup recreates the tables and data with `docker compose down -v`; the
   steps above do not apply to it.

With `prevent_destroy` removed, `makers terraform-destroy` deletes the tables. Confirm that you are
targeting the intended environment before you run it.

## terraform init

```shell
$ makers terraform-init
```

The first time you run terraform, S3 buckets and lock tables will be created automatically.

## terraform plan

```shell
$ makers terraform-plan
````

## terraform apply

```shell $ makers terraform-apply
$ makers terraform-apply
```

## Update kubeconfig

Execute the following command to generate kubeconfig(`~/.kube/config`).

```shell
$ makers update-kubeconfig
```

FYI: https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html

Confirm that the context has been switched.

``shell
$ makers k8s-get-contexts
CURRENT NAME CLUSTER AUTHINFO NAMESPACE

* arn:aws:eks:us-east-1:XXXXXXXXXXXXXX:cluster/oce3noy9-eks-adceet arn:aws:eks:us-east-1:XXXXXXXXXXXXXX:
  cluster/oce3noy9-eks-adceet arn: aws:eks:us-east-1:XXXXXXXXXXXXXXXX:cluster/oce3noy9-eks-adceet
  docker-desktop docker-desktop docker-desktop

```

## Check kubernetes-dashboard

## port forward to kubernetes-dashboard

```shell
$ makers k8s-port-forward-dashboard
```

## Change Chrome settings

Open `chrome://flags/#allow-insecure-localhost` 2.
Set `Allow invalid certificates for resources loaded from localhost.` to `Enable` 3.
Restart Chrome 4.
Open `https://localhost:8443`.

## Create a token

``shell
$ makers k8s-create-dashboard-token

```

## Check Aurora (MySQL) connection information

```shell
$ makers get-aurora-cluster-all-info
[cargo-make] INFO - makers 0.36.12
[cargo-make] INFO - Calling cargo metadata to extract project info
[cargo-make] INFO - Cargo metadata done
[cargo-make] INFO - Execute Command: "/bin/sh" "/var/folders/tn/ppwkg3_s603fs3702lmrsn_80000gn/T/fsio_j6Rd16XDN0.sh"
--- Using Environments -----------------
AWS_PROFILE = ceer
AWS_REGION = ap-northeast-1
PREFIX = pah8iobi
APPLICATION_NAME = ceer
----------------------------------------
[cargo-make] INFO - Build File: Makefile.toml
[cargo-make] INFO - Task: get-aurora-cluster-all-info
[cargo-make] INFO - Profile: development
[cargo-make] INFO - Running Task: legacy-migration
[cargo-make] INFO - Running Task: get-aurora-cluster-master-username
MYSQL_USER_NAME = root
[cargo-make] INFO - Running Task: get-aurora-cluster-master-password
MYSQL_USER_PASSWORD = xnLY1KMG14ygUbW2P0Pu
[cargo-make] INFO - Running Task: get-aurora-cluster-endpoint
MYSQL_HOST = pah8iobi-ceer-mysql.cluster-ctywrcabnmgr.ap-northeast-1.rds.amazonaws.com
[cargo-make] INFO - Running Task: get-aurora-cluster-port
MYSQL_PORT = 3306
[cargo-make] INFO - Running Task: get-aurora-cluster-database
MYSQL_DATABASE = ceer
[cargo-make] INFO - Build Done in 10.76 seconds.
````

Set the obtained password to common.env.

```shell
MYSQL_USER_NAME=root
MYSQL_USER_PASSWORD=xxxxxxxxxx
MYSQL_ENDPOINT=${PREFIX}-ceer-mysql.cluster-ctywrcabnmgr.ap-northeast-1.rds.amazonaws.com
MYSQL_PORT=3306
MYSQL_DATABASE=ceer
```