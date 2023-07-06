# Setup for AWS

## Add an aws profile

Edit `~/.aws/credentials` as follows.

### If AWS IAM Identity Center(AWS SSO) is not used

Define just one entry.

```
# ...
[ceer]
aws_access_key_id=xxxxx
aws_secret_access_key=xxxxx
aws_session_token=xxxxx
```

### If AWS IAM Identity Center(AWS SSO) is used.

Define one for terraform and one for SSO separately.

`~/.aws/config`

```
[profile ceer-sso]
sso_start_url = https://xxxxx.awsapps.com/start
sso_region = us-east-1
sso_account_id = 1234567890
sso_role_name = XxxxOwnerAccess
region = us-east-1
```

`~/.aws/credentials`

```
# ...
[ceer]
aws_access_key_id=xxxxx
aws_secret_access_key=xxxxx
aws_session_token=xxxxx
# ...
# ...
# ...
[ceer-sso]
aws_access_key_id=xxxxx
aws_secret_access_key=xxxxx
aws_session_token=xxxxx
```

## copy env.sh.default as env.sh, and edit it

```shell
$ cp env.sh.default env.sh
```

Modify PREFIX, APPLICATION_NAME as appropriate.
If you want to create a personal environment, change PREFIX.

### If AWS IAM Identity Center(AWS SSO) is not used

Should specify a profile name to `AWS_PROFILE`.

```shell
export AWS_PROFILE=ceer
export AWS_REGION=us-east-1
export AWS_ACCOUNT_ID=1234567890

export PREFIX=om2eep1k
export APPLICATION_NAME=ceer

if [[ "$OUTPUT_ENV" == 1 ]]; then
echo "--- Using Environments -----------------"
echo "AWS_PROFILE      = $AWS_PROFILE"
echo "AWS_REGION       = $AWS_REGION"
echo "PREFIX           = $PREFIX"
echo "APPLICATION_NAME = $APPLICATION_NAME"
echo "----------------------------------------"
fi
```

### If AWS IAM Identity Center(AWS SSO) is used

Should specify a profile name for AWS SSO to `AWS_PROFILE`.

```shell
export AWS_PROFILE=ceer
```

# Building an AWS environment with terraform

Create a new file `tools/terraform/${PREFIX}-${APPLICATION_NAME}-terraform.tfvars` with the following.
Changes defined in variables.tf can be overridden in this tfvars file.
You can create only the resources you need. For example, just set `ecr_enabled = true` if you only need ecr.

```
akka_persistence_enabled = true
akka_persistence_journal_name      = "journal"
akka_persistence_journal_gsi_name  = "jounral-aid-index"
akka_persistence_snapshot_name     = "snapshot"
akka_persistence_snapshot_gsi_name = "snapshot-aid-index"

eks_enabled = true
eks_version = "1.71"
eks_auth_roles = []
eks_auth_users = []
eks_auth_accounts = []

ecr_enabled = true

datadog-api-key = "xxxx"
```

## Create a lock table

At first time only, Create a lock table for terraform on DynamoDB.

```shell
tools/terraform $ ./create-lock-table.sh
```

## Create a s3 bucket for tfstate

At first time only, Create an s3 bucket to store tfstate.

```shell
tools/terraform $ ./create-tf-bucket.sh
```

## terraform init

```shell
tools/terraform $ ./terraform-init.sh
```

## terraform plan

```shell
tools/terraform $ ./terraform-plan.sh
```

## terraform apply

```shell
tools/terraform $ ./terraform-apply.sh
```

## Update kubeconfig

Execute the following command to generate kubeconfig(`~/.kube/config`).

```shell
tools/terraform $ ./update-kubeconfig.sh
```

FYI: https://docs.aws.amazon.com/eks/latest/userguide/create-kubeconfig.html

Verify that the context has switched.

```shell
tools/terraform $ kubectl config get-contexts
CURRENT   NAME                                                             CLUSTER                                                          AUTHINFO                                                         NAMESPACE
*         arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet   arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet   arn:aws:eks:us-east-1:XXXXXXXXXXXX:cluster/oce3noy9-eks-adceet
          docker-desktop                                                   docker-desktop                                                   docker-desktop
```

# Confirm kubernetes-dashboard

## Port forward to kubernetes-dashboard

```shell
$ DASHBOARD_NS=kubernetes-dashboard
$ export POD_NAME=$(kubectl get pods -n $DASHBOARD_NS -l "app.kubernetes.io/name=kubernetes-dashboard,app.kubernetes.io/instance=kubernetes-dashboard" -o jsonpath="{.items[0].metadata.name}")
$ kubectl -n $DASHBOARD_NS port-forward $POD_NAME 8443:8443
```

## Configuring Chrome

1. Open `chrome://flags/#allow-insecure-localhost`
2. `Allow invalid certificates for resources loaded from localhost.` is `Enable`
3. Relaunch
4. Open `https://localhost:8443`

## Get token

```shell
$ kubectl -n kubernetes-dashboard create token kubernetes-dashboard
eyJXXXXXXXXXXXXXXXXXXXXXXXXXXxYTZmYzlkOGZkN2M5ZjMifQ.eyJhdWQiOlsiaHR0cHM6Ly9rdWJlcm5ldGVzLmRlZmF1bHQuc3ZjIl0sImV4cCI6MTY4ODY5NzE0MSwiaWF0IjoxNjg4NjkzNTQxLCJpc3MiOiJodHRwczovL29pZGMuZWtzLmFwLW5vcnRoZWFzdC0xLmFtYXpvbmF3cy5jb20vaWQvQjcyNjkwQkY2NEUyQUM5RjA0RkIxRDQxMTZDNkMwMDQiLCJrdWJlcm5ldGVzLmlvIjp7Im5hbWVzcGFjZSI6Imt1YmVybmV0ZXMtZGFzaGJvYXJkIiwic2VydmljZWFjY291bnQiOnsibmFtZSI6Imt1YmVybmV0ZXMtZGFzaGJvYXJkIiwidWlkIjoiMWJlNjViMmYtYmZkMy00NmYwLWE0MjktODlkY2E4Yzk4MWU5In19LCJuYmYiOjE2ODg2OTM1NDEsInN1YiI6InN5c3RlbTpzZXJ2aWNlYWNjb3VudDprdWJlcm5ldGVzLWRhc2hib2FyZDprdWJlcm5ldGVzLWRhc2hib2FyZCJ9.agCS7Zf7WlzFW32lRhNlNHzzvEoGUJALq0X_VaOW9WuWdjhY90tvDzCkul9YGygGwHKTuGMrkDJIfcJf92c3hUpXLu30HYRRBCpth-cC4YUd51Eccm5OJ6HJwuK-HaFO6ZtGfOl5uPO2qJzFyLEHAEePEiH6MVWMcx3BWvDDxFColFleasF1BtkB_64-k-fgaOBAnGM3paTQ8ug4taqsRaisSChLreYYs7zUI7YXnQMJ0LswWQyMOjVC52BfjwqNvP2A7bg8hqOPaO66lN4x89TAD_d0yjKQM3CuXlGKGPKqYtGj7KRvWNjvXN5VNEeQnhv9LYzs6n1SyKD7KsRCWA
```
