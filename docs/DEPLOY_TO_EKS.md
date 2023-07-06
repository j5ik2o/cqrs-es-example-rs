# Deploy to EKS

First, enable the Kubernetes option in Docker for Mac(Enable Kubernetes).
Also check the resource settings for Docker for Mac. You must give it sufficient resources.

## Push the Docker Image

Please push the image to docker local repository.

```shell
ceer-root $ ./tools/scripts/docker-build-push.sh
```

## Edit the Configuration file of Helmfile

```shell
ceer-root $ vi ./tools/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml
ceer-root # tools/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml
```

Notice the tag value displayed in the console; reflect it in xxx.image.tag.
Please set the following items in the yaml file appropriately

- writeApi.writeApiServer.image.repository
- writeApi.writeApiServer.image.tag

## Deploy the applications

Next deploy the backend role.

```shell
tools/scripts $ ./helmfile-apply-eks.sh
```

Wait a few moments for the cluster to form. Make sure there are no errors in the log.

```shell
$ stern 'write-api-server-*' -n adceet
```

Make sure all pods are in Ready status.

## Check the applications

After frontend is started, check the operation with the following commands.

```shell
$ curl -X GET https://xxxxxx/hello
Hello World!
```

Call API to check operation.

```shell
$ curl -v -X POST -H "Content-Type: application/json" -d "{ \"accountId\": \"01G41J1A2GVT5HE45AH7GP711P\" }" https://xxxxxx/threads
{"threadId":"01GBCN25M496HB4PK9EWQMH28J"}
```

