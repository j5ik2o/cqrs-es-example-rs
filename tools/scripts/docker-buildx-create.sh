#!/usr/bin/env bash

EXISTS=$(docker buildx ls | grep amd-arm)

if [ -z "$EXISTS" ]; then
  echo "Creating buildx instance"
  docker buildx create --name amd-arm --driver docker-container --platform linux/arm64,linux/amd64
else
  echo "Buildx instance already exists"
  exit 0
fi