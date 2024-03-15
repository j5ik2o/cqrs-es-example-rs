# Deploy to EKS

## Build the Docker image and push it to ECR

```shell
$ makers docker-ecr-build-push-all
```

## Edit the Helmfile configuration files for the Write API Server and Read API Server.

```shell
$ vi . /tools/deploy/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml
```

Set the following entries in the yaml file appropriately for the tag values displayed in the console

- writeApiServer.image.tag
- readApiServer.image.tag

## Edit the terraform configuration file.

``shell
$ vi . /tools/deploy/terraform/${PREFIX}-${APPLICATION_NAME}-terraform.tfvars

```

Set the following items appropriately for the tag values displayed in the console.

````tfvars
read_model_updater_tag = "9ed584699fe19cab82121fae2d4ac7f1eee2e49089ba463cdd7378085ccc7b39-amd64"
```

## Deploying the application

Next, deploy the application (write-api-server/read-api-server).

```shell
$ makers helmfile-apply-all
```

Wait for a while until the cluster is formed. Make sure there are no errors in the log.

```shell
$ stern '*' -n ceer
````

Verify that all pods are in Ready state.

Make sure that the hostname is attached to the Address of the Ingress.

```shell
$ makers kubectl-get-ingress-write-api-server
# snip
NAME CLASS HOSTS ADDRESS PORTS AGE
write-api-server alb write-ceer-j5ik2o.xxxxxxxx.info k8s-ceer-writeapi-f8152916e6-1353305610.ap-northeast-1.elb.amazonaws.com 80 145m
````

````shell
$ makers update-dns-write-api-server
--- Using Environments -----------------
AWS_PROFILE = ceer
AWS_REGION = ap-northeast-1
PREFIX = aht9aa1e
APPLICATION_NAME = ceer
-------------------