# EKSへのデプロイ

## DockerイメージをECRにプッシュする

```shell
ceer-root $ makers docker-ecr-build-push-all
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
ceer-root $ makers helmfile-apply
```

クラスタが形成されるまでしばらく待ちます。ログにエラーがないことを確認してください。

```shell
$ stern 'write-api-server-*' -n ceer
```

すべてのPodがReady状態になっていることを確認する。

IngressのAddressにホスト名が付いていることを確認する。

```shell
$ makers kubectl-get-ingress-write-api-server
# snip
NAME               CLASS   HOSTS                           ADDRESS                                                                    PORTS   AGE
write-api-server   alb     write-ceer-j5ik2o.xxxxxx.info   k8s-ceer-writeapi-f8152916e6-1353305610.ap-northeast-1.elb.amazonaws.com   80      145m
```

```shell
$ makers update-dns-write-api-server
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = ap-northeast-1
PREFIX           = aht9aa1e
APPLICATION_NAME = ceer
----------------------------------------
DNS_NAME=k8s-ceer-writeapi-f8152916e6-1353305610.ap-northeast-1.elb.amazonaws.com
ALB_NAME=k8s-ceer-writeapi-f8152916e6
ALB_ARN=arn:aws:elasticloadbalancing:ap-northeast-1:738575627980:loadbalancer/app/k8s-ceer-writeapi-f8152916e6/1bc1fd361b50b9ef
HOSTED_ZONE_ID=Z14GRHDCWA56QT
{
    "ChangeInfo": {
        "Id": "/change/C0690022GGDQ5KIQBOP3",
        "Status": "PENDING",
        "SubmittedAt": "2023-07-15T18:09:49.123000+00:00"
    }
}
```

```shell
$ makers kubectl-get-ingress-read-api-server
# snip
NAME              CLASS   HOSTS                          ADDRESS                                                                   PORTS   AGE
read-api-server   alb     read-ceer-j5ik2o.xxxxxx.info   k8s-ceer-readapis-818fc43feb-708519146.ap-northeast-1.elb.amazonaws.com   80      4h26m
```

```shell
$ makers update-dns-read-api-server
--- Using Environments -----------------
AWS_PROFILE      = ceer
AWS_REGION       = ap-northeast-1
PREFIX           = aht9aa1e
APPLICATION_NAME = ceer
----------------------------------------
DNS_NAME=k8s-ceer-readapis-818fc43feb-708519146.ap-northeast-1.elb.amazonaws.com
ALB_NAME=k8s-ceer-readapis-818fc43feb
ALB_ARN=arn:aws:elasticloadbalancing:ap-northeast-1:738575627980:loadbalancer/app/k8s-ceer-readapis-818fc43feb/20c6df4385875307
HOSTED_ZONE_ID=Z14GRHDCWA56QT
{
    "ChangeInfo": {
        "Id": "/change/C08808643UWPIGM7L0DA9",
        "Status": "PENDING",
        "SubmittedAt": "2023-07-15T18:09:27.854000+00:00"
    }
}
```

## アプリケーションの動作チェック

フロントエンドが起動したら、以下のコマンドで動作を確認する。

```shell
$ makers --profile production curl-get-root
# snip
Hello, Write API!
# snip
Hello, Read API!
# snip
```

APIを呼び出して動作を確認する。

```shell
$ makers create-and-get-thread
# snip
create-thread: THREAD_ID=01H5Q476STTQ78D45ZR4EKXF0B
get-thread: ACTUAL_THREAD_ID=01H5Q476STTQ78D45ZR4EKXF0B
OK
# snip
```

