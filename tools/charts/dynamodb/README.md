# Dynamodb helm chart

This helm chart
converts [Amazon's dynamodb-local](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.html)
into a k8s deployable application. It creates a 2 container pod with one running a dynamodb table and the other running
a [dynamodb admin app](https://github.com/aaronshaf/dynamodb-admin)

The intent is to provide a quick easy way to enable a local dynamo db instance for local development/testing.

## prerequisites

* helm 3
* a modern working k8s cluster

## Install

```bash
helm repo add keyporttech https://keyporttech.github.io/helm-charts/
helm install my-release keyporttech/dynamo-db
```

or clone this repo and install from the file system.

## Contributing

Please
see [keyporttech charts contribution guidelines](https://github.com/keyporttech/helm-charts/blob/master/CONTRIBUTING.md)

### Running the cicd tooling locally

This chart uses a Makefile to run CICD. To run:

```bash
make build
```

## Values.yaml Configuration

Most configurable values are similar to other helm charts generated via helm create. The configurations specific to this
chart are listed below.

### Ingress controller

When the ingress controller is enabled the admin UI is available via:

http(s)://host.domain.com/dynamodb

Example with with using nginx controller and CertificateManager letsencrypt TLS issuer:

```yaml
ingress:
  enabled: true
  host: dynamodb.myhost.com
  annotations:
    kubernetes.io/ingress.class: nginx
    kubernetes.io/ingress.allow-http: "false"
    nginx.ingress.kubernetes.io/proxy-body-size: "0"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "600"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    cert-manager.io/cluster-issuer: cert-issuer-letsencrypt-prod
  tls:
  # Secrets must be manually created in the namespace.
  - secretName: dynamodb-tls
    hosts:
      - dynamodb.myhost.com
```

### Storage

This chart allows for 3 types of storage: pvc, directVolume, and emptyDir set via: storageType.

If no configuration is provided the chart will store data on the node using emptyDir.

If storageType is set to directVolume then directVolume msut be set in the chart vaules.

Example:

```yaml
directVolume:
  nfs:
    server: 10.10.10.10
    path: "/dynamodb-data"
```

The following is an example of storage type pvc:

```yaml
storage: "500Mi"
storageClassName: ""
```

| Parameter                                       | Description                                   | Default                    |
|-------------------------------------------------|-----------------------------------------------|----------------------------|
| `dynamodb.image.repository`                     | `dynamo db local image`                       | `amazon/dynamodb-local`    |
| `dynamodb.image.pullPolicy`                     | `image pull policy`                           | `IfNotPresent`             |  
| `dynamodb.image.tag`                            | `image tag`                                   | `1.12.0`                   |
| `admin.image.repository`                        | `admin web UI image`                          | `aaronshaf/dynamodb-admin` |
| `admin.image.pullPolicy`                        | `image pull policy`                           | `IfNotPresent`             |  
| `admin.image.tag`                               | `image tag`                                   | `latest`                   |
| `imagePullSecrets`                              | `image pull secrets`                          | `-`                        |
| `nameOverride`                                  | `name override`                               | `-`                        |
| `fullnameOverride`                              | `fullname override`                           | `-`                        |
| `serviceAccount.create`                         | `creates service account if true`             | `true`                     |
| `serviceAccount.annotations`                    | `service account annotations if created`      | `-`                        |
| `serviceAccount.name`                           | `service account name if created`             | `-`                        |
| `podAnnotations`                                | `pod annotations`                             | `-`                        |
| `podSecurityContext`                            | `pod security context`                        | `-`                        |
| `securityContext`                               | `security context`                            | `-`                        |
| `service.type`                                  | `k8s service type`                            | `ClusterIP`                |
| `ingress.enabled`                               | `enable ingress`                              | `false`                    |
| `ingress.annotations`                           | `ingress annotations`                         | `{}`                       |
| `ingress.tls`                                   | `list of tls secret names and known_hosts`    | `[]`                       |
| `ingress.host`                                  | `ingress host`                                | `-`                        |
| `autoscaling.enabled`                           | `enable autoscaling`                          | `false`                    |
| `autoscaling.minReplicas`                       | `min replicas`                                | `1`                        |
| `autoscaling.maxReplicas`                       | `max replicas`                                | `100`                      |
| `autoscaling.targetCPUUtilizationPercentage`    | `autoscaling target CPU`                      | `80`                       |
| `autoscaling.targetMemoryUtilizationPercentage` | `autoscaling target memory`                   | `unset`                    |
| `resources`                                     | `pod resources`                               | `[]`                       |
| `nodeSelector`                                  | `node selector`                               | `{}`                       |
| `tolerations`                                   | `node selector`                               | `[]`                       |
| `affinity`                                      | `affinity`                                    | `{}`                       |
| `storageType`                                   | `type of storage pvc, directVolume, emptyDir` | `emptyDir`                 |
| `storage`                                       | `size of pvc storage`                         | `unset`                    |
| `storageClassName`                              | `pvc storage class name`                      | `unset`                    |
| `directVolume`                                  | `yaml definings k8s volume`                   | `unset`                    |
