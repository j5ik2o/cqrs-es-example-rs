# Deploy to Minikube

First, Give minikube driver(docker, virtualbox, ...) enough resources.

## Launch Minikube

Run the following script to start minikube.

```shell
ceer-root $ ./tools/scripts/minikube-start.sh
```

The dirver, system resources, etc. are specified in `minikube-start.sh`, so modify them to suit your preferences (but be
sure to reserve the resources required by Deployment).

Change the Docker client connection to minikube.

```shell
ceer-root $ eval $(minikube docker-env default)
```

## Push the Docker Image

Please push the image to docker registry on minikube.

```shell
ceer-root $ ./tools/scripts/docker-build-all.sh
```

## Edit the Configuration file of Helmfile

```shell
ceer-root $ vi ./tools/config/environments/${PREFIX}-${APPLICATION_NAME}-local.yaml
ceer-root # tools/config/environments/${PREFIX}-${APPLICATION_NAME}-local.yaml
```

Notice the tag value displayed in the console; reflect it in xxx.image.tag.
Please set the following items in the yaml file appropriately

- writeApi.writeApiServer.image.repository
- writeApi.writeApiServer.image.tag

Set the following items in the yaml file appropriately(if you use Read API Server)

- readModelUpdater.image.repository
- readModelUpdater.image.tag
- readApiServer.image.repository
- readApiServer.image.tag

---

**NOTE**

All components can be deployed with a single command below, but it is recommended that you run each step at least once
to get a feel for the process.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-all.sh
```

---

## Prepare DynamoDB tabels

Next deploy dynamodb local.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-dynamodb.sh
```

Create the necessary tables.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-dynamodb-setup.sh
```

Open `http://127.0.0.1:31567/` if you want to use DynamoDB Admin.

## Prepare MySQL tabels

Next deploy mysql.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-mysql.sh
```

Create the necessary tables.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-flyway.sh
```

## [About akka-cluster roles](DEBUG_ON_LOCAL_K8S.md#about-akka-cluster-roles)

## Deploy the Backend role

Next deploy the backend roles.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local.sh
```

Wait a few moments for the cluster to form. Make sure there are no errors in the log.

```shell
$ stern 'write-api-server-*' -n adceet
```

Make sure all pods are in Ready status.

## Deploy Read Model Updater (if you need)

Next deploy Read Model Updater.

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-rmu.sh
```

Wait a few moments. Make sure there are no errors in the log.

```shell
$ stern 'read-model-updater-*' -n adceet
```

## Deploy Read API Server (if you need)

Next deploy Read API Server

```shell
ceer-root $ ./tools/scripts/helmfile-apply-local-read-api.sh
```

Wait a few moments. Make sure there are no errors in the log.

```shell
$ stern 'read-api-server-*' -n adceet
```

## Check the applications

After frontend is started, check the operation with the following commands.

```shell
$ curl -X GET http://127.0.0.1:30031/hello
Hello, World!
```

Call API to check operation.

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ \"accountId\": \"01G41J1A2GVT5HE45AH7GP711P\" }" http://127.0.0.1:30031/threads
{"threadId":"01GBCN25M496HB4PK9EWQMH28J"}
```

**NOTE: In a local environment, the first event may not consume well. If this is the case, try sending the command
again.**

Execute the following command if you use RMU and Read API Server.

```shell
$ curl -v -H "Content-Type: application/json" http://127.0.0.1:30033/threads?owner_id=01G41J1A2GVT5HE45AH7GP711P
[{"id":"01GG72CT9B62DRMH31F8SQX3H9","owner_id":"01G41J1A2GVT5HE45AH7GP711P","created_at":"2022-10-25T07:58:31.096808590Z"}]%
```
