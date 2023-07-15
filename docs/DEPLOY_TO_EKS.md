# EKSへのデプロイ

## DockerイメージをECRにプッシュする

```shell
ceer-root $ ./tools/scripts/docker-ecr-push-with-build.sh
```

## Helmfile の設定ファイルを編集します。

```shell
ceer-root $ vi ./tools/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml
ceer-root # tools/config/environments/${PREFIX}-${APPLICATION_NAME}-eks.yaml
```

コンソールに表示されるタグの値に注目してください。
yamlファイルの以下の項目を適切に設定してください。

- writeApiServer.image.repository
- writeApiServer.image.tag
- readApiServer.image.repository
- readApiServer.image.tag

## アプリケーションのデプロイ

次にデプロイします。

```shell
ceer-root $ ./tools/scripts/helmfile-apply-eks.sh
```

クラスタが形成されるまでしばらく待ちます。ログにエラーがないことを確認してください。

```shell
$ stern 'write-api-server-*' -n adceet
```

すべてのPodがReady状態になっていることを確認する。

IngressのAddressにホスト名が付いていることを確認する。

```shell
$ kubectl -n ceer get ingress write-api-server
NAME               CLASS   HOSTS                           ADDRESS                                                                    PORTS   AGE
write-api-server   alb     write-ceer-j5ik2o.cwtest.info   k8s-ceer-writeapi-f8152916e6-1353305610.ap-northeast-1.elb.amazonaws.com   80      145m
```

```shell
$ ./tools/scripts/aws-route53-upsert-external-dns-of-write-api-server.sh
```

```shell
$ kubectl -n ceer get ingress read-api-server
NAME              CLASS   HOSTS                          ADDRESS                                                                   PORTS   AGE
read-api-server   alb     read-ceer-j5ik2o.cwtest.info   k8s-ceer-readapis-818fc43feb-708519146.ap-northeast-1.elb.amazonaws.com   80      4h26m
```

```shell
$ ./tools/scripts/aws-route53-upsert-external-dns-of-read-api-server.sh
```

## アプリケーションのチェック

フロントエンドが起動したら、以下のコマンドで動作を確認する。

```shell
$ ./tools/scripts/curl-get-root-write-api-server-on-eks.sh
Hello, Write API!%
```

APIを呼び出して動作を確認する。

```shell
$ ./tools/scripts/curl-post-write-api-server-on-eks.sh
{"Success":{"id":{"value":"01H5DAYAN4ENF16AMT6Z6EQ0PC"}}} 
```

