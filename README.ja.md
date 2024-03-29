# cqrs-es-example-rs

[![Workflow Status](https://github.com/j5ik2o/cqrs-es-example-rs/workflows/ci/badge.svg)](https://github.com/j5ik2o/cqrs-es-example-rs/actions?query=workflow%3A%22ci%22)
[![Renovate](https://img.shields.io/badge/renovate-enabled-brightgreen.svg)](https://renovatebot.com)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![](https://tokei.rs/b1/github/j5ik2o/cqrs-es-example-rs)](https://github.com/XAMPPRocky/tokei)

ステータス:実装中

このリポジトリは、RustでのCQRS/Event Sourcing + GraphQLのサンプル実装です。

Event Sourcingのために[j5ik2o/event-store-adapter-rs](https://github.com/j5ik2o/event-store-adapter-rs)を利用しています。

## 概要

### コンポーネント構成

- Write API (WIP)
    - GraphQLサーバ(Mutation)
- Read Model Updater
    - ジャーナルを基にリードモデル構築するLambda
    - ローカルではLambdaの動作をエミュレーションするコードを実行する(local-rmu)
- Read API
    - GraphQLサーバ(Query)

### システム構成図

![](docs/images/system-layout.png)

## 開発環境

- [ツールのセットアップ](docs/TOOLS_INSTALLATION.ja.md)
- [ビルドとテスト](docs/BUILD_AND_TEST.ja.md)

### ローカル環境

- [ローカルマシンでのデバッグ](docs/DEBUG_ON_LOCAL_MACHINE.ja.md)
- [Docker Composeでのデバッグ](docs/DEBUG_ON_DOCKER_COMPOSE.ja.md)

### AWS環境

- [AWSのセットアップ](docs/TOOLS_INSTALLATION_AWS.ja.md)
- [EKSへのデプロイ](docs/DEPLOY_TO_EKS.ja.md)

## リンク

- [Go版](https://github.com/j5ik2o/cqrs-es-example-go)
- [TypeScript版](https://github.com/j5ik2o/cqrs-es-example-js)
- [共通ドキュメント](https://github.com/j5ik2o/cqrs-es-example-docs)
