#!/bin/sh

set -eu

docker buildx create --name amd-arm --driver docker-container --platform linux/arm64,linux/amd64