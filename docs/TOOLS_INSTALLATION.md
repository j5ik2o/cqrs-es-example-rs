# ツールのインストール 

## asdf

```sh
$ brew install asdf
```

## jq

```shell
$ asdf plugin-add jq https://github.com/AZMCode/asdf-jq.git
$ asdf install jq 1.6
$ asdf local jq 1.6
$ jq --version
jq-1.6
```

## awscli

```shell
$ asdf plugin add awscli
$ asdf install awscli 2.12.6
$ asdf local  awscli 2.12.6
$ aws --version
aws-cli/2.12.6 Python/3.11.4 Darwin/22.5.0 exe/x86_64 prompt/off
```

## terraform

https://github.com/hashicorp/terraform

```shell
$ asdf plugin-add terraform https://github.com/asdf-community/asdf-hashicorp.git
$ asdf install terraform 1.5.2
$ asdf local terraform 1.5.2
$ terraform version
Terraform v1.5.2
on darwin_arm64
+ provider registry.terraform.io/hashicorp/aws v5.0.1
+ provider registry.terraform.io/hashicorp/cloudinit v2.3.2
+ provider registry.terraform.io/hashicorp/helm v2.5.1
+ provider registry.terraform.io/hashicorp/kubernetes v2.11.0
+ provider registry.terraform.io/hashicorp/random v3.5.1
+ provider registry.terraform.io/hashicorp/time v0.9.1
+ provider registry.terraform.io/hashicorp/tls v4.0.4
```

## terraformer(オプション)

datadogに設定をインポートするためのツール。

https://github.com/GoogleCloudPlatform/terraformer

```shell
$ asdf plugin add terraformer https://github.com/grimoh/asdf-terraformer.git
$ asdf install terraformer 0.8.24
$ asdf local terraformer 0.8.24
$ mkdir ./temp
ceer-root/temp $ export GODEBUG=asyncpreemptoff=1
ceer-root/temp $ DATADOG_API_KEY="xxx" DATADOG_APP_KEY="xxx" terraformer import datadog --resources=dashboard --filter=datadog_dashboard=XXXXX
```

## Kubernetes

### kubectl

トラブルを避けるため、サーバ側と同じバージョンのkubectlをインストールしてください。

```shell
$ KUBECTL_VERSION=1.27.3
$ asdf plugin-add kubectl https://github.com/asdf-community/asdf-kubectl.git
$ asdf install kubectl $KUBECTL_VERSION
$ asdf local kubectl $KUBECTL_VERSION # Always set up in the project root.
$ kubectl version
Client Version: version.Info{Major:"1", Minor:"21", GitVersion:"v1.21.13", GitCommit:"80ec6572b15ee0ed2e6efa97a4dcd30f57e68224", GitTreeState:"clean", BuildDate:"2022-05-24T12:40:44Z", GoVersion:"go1.16.15", Compiler:"gc", Platform:"darwin/arm64"}
Server Version: version.Info{Major:"1", Minor:"21+", GitVersion:"v1.21.12-eks-a64ea69", GitCommit:"d4336843ba36120e9ed1491fddff5f2fec33eb77", GitTreeState:"clean", BuildDate:"2022-05-12T18:29:27Z", GoVersion:"go1.16.15", Compiler:"gc", Platform:"linux/amd64"}
```

### stern

```shell
$ asdf plugin-add stern https://github.com/looztra/asdf-stern
$ asdf install stern 1.25.0
$ asdf local stern 1.25.0
$ stern --version
version: 1.25.0
commit: f13bde422e977c7a69ec0827c0337b2bc8e44444
built at: 2023-04-13T23:06:41Z
```

### helm

```shell
$ asdf plugin-add helm https://github.com/Antiarchitect/asdf-helm.git
$ asdf install helm 3.12.1
$ asdf local helm 3.12.1
$ helm version
version.BuildInfo{Version:"v3.12.1", GitCommit:"f32a527a060157990e2aa86bf45010dfb3cc8b8d", GitTreeState:"clean", GoVersion:"go1.20.4"}
# Required plug-ins in helmfile
$ helm plugin install https://github.com/databus23/helm-diff
```

### helmfile

```shell
$ asdf plugin-add helmfile https://github.com/feniix/asdf-helmfile.git
$ asdf install helmfile 0.155.0
$ asdf local helmfile 0.155.0
$ helmfile --version
helmfile version v0.155.0
```

### minikube (オプション)

```shell
$ asdf plugin-add minikube https://github.com/alvarobp/asdf-minikube.git
$ asdf install minikube 1.30.1
$ asdf local minikube 1.30.1
$ minikube version
minikube version: v1.30.1
commit: 08896fd1dc362c097c925146c4a0d0dac715ace0
```

### cargo-make

```shell
$ cargo install cargo-make
```

### sqlx-cli

```shell
ceer-root $ cargo install sqlx-cli
```

### docker buildx

```shell
ceer-root $ ./docker-buildx-create.sh
```