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

Before you apply it to an existing environment:

1. Back up the existing `journal` and `snapshot` tables. For an on-demand backup, run
   `aws dynamodb create-backup --table-name <table> --backup-name <name>`.
2. Decide whether the existing data must be kept. This sample does not ship a migration script and
   assumes the tables are recreated, as they are in LocalStack.
3. If the data must be kept, create the three new tables and rewrite the items with the new keys.
   Do not delete the old tables until the rewrite is complete.
4. Run `makers terraform-plan` and confirm that `journal` and `snapshot` are replacements, then run
   `makers terraform-apply`.

You may add `prevent_destroy = true` to the `journal` table to prevent accidental deletion. In that
case `makers terraform-destroy` fails; remove the setting only when you intend to destroy the
environment.

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