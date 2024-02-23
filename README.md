# cqrs-es-example-rs

[![Workflow Status](https://github.com/j5ik2o/cqrs-es-example-rs/workflows/ci/badge.svg)](https://github.com/j5ik2o/cqrs-es-example-rs/actions?query=workflow%3A%22ci%22)
[![Renovate](https://img.shields.io/badge/renovate-enabled-brightgreen.svg)](https://renovatebot.com)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![](https://tokei.rs/b1/github/j5ik2o/cqrs-es-example-rs)](https://github.com/XAMPPRocky/tokei)

Status: In Implementation

This repository is a sample implementation of CQRS/ES in Rust.

It uses [j5ik2o/event-store-adapter-rs](https://github.com/j5ik2o/event-store-adapter-rs) for Event Sourcing.

[日本語](./README.ja.md)

## Overview

### Component Composition

- Write API Server
    - GraphQL server (Mutation)
- Read Model Updater
    - Lambda to build read models based on journals
- Read API Server
    - GraphQL server (Query, Subscription)

### System Architecture Diagram

![](docs/images/system-layout.png)

## Development Environment

- [Tool Setup](docs/TOOLS_INSTALLATION.md)

### Local Environment

- [Debugging on Local Machine](docs/DEBUG_ON_LOCAL_MACHINE.md)
- [Debugging with Docker Compose](docs/DEBUG_ON_DOCKER_COMPOSE.md)
- [Deploying to Local Kubernetes](docs/DEPLOY_TO_LOCAL_K8S.md)
- [Deploying to Minikube](docs/DEPLOY_TO_MINIKUBE.md)

### AWS Environment

- [AWS Setup](docs/AWS_SETUP.md)
- [Deploying to EKS](docs/DEPLOY_TO_EKS.md)


## Links

- [Go Version](https://github.com/j5ik2o/cqrs-es-example-go)
- [TypeScript Version](https://github.com/j5ik2o/cqrs-es-example-js)
- [Common Documents](https://github.com/j5ik2o/cqrs-es-example-docs)
