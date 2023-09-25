# cqrs-es-example-rs

[![Workflow Status](https://github.com/j5ik2o/cqrs-es-example-rs/workflows/ci/badge.svg)](https://github.com/j5ik2o/cqrs-es-example-rs/actions?query=workflow%3A%22ci%22)
[![Renovate](https://img.shields.io/badge/renovate-enabled-brightgreen.svg)](https://renovatebot.com)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![](https://tokei.rs/b1/github/j5ik2o/cqrs-es-example-rs)](https://github.com/XAMPPRocky/tokei)

ステータス:実装中

このリポジトリは、RustでのCQRS/ESのサンプル実装です。

## 概要

### コンポーネント構成

- Write API (WIP)
    - 書き込み専用のWeb API
- Read Model Updater
    - ジャーナルを基にリードモデル構築するLambda
- Read API
    - GraphQLサーバ(Query,Subscription)

### システム構成図

![](./system-layout.png)

## 開発環境

- [ツールのセットアップ](docs/TOOLS_INSTALLATION.md)

### ローカル環境

- [ローカルマシンでのデバッグ](docs/DEBUG_ON_LOCAL_MACHINE.md)
- [Docker Composeでのデバッグ](docs/DEBUG_ON_DOCKER_COMPOSE.md)
- [ローカルKubernetesでのデプロイ](docs/DEPLOY_TO_LOCAL_K8S.md)
- [Minikubeへのデプロイ](docs/DEPLOY_TO_MINIKUBE.md)

### AWS環境

- [AWSのセットアップ](docs/AWS_SETUP.md)
- [EKSへのデプロイ](docs/DEPLOY_TO_EKS.md)
