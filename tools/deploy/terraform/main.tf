terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.100.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.21.1"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.10.1"
    }
  }
  backend "s3" {
  }
}

data "aws_region" "current" {
}

resource "random_string" "suffix" {
  length  = 6
  special = false
}

data "aws_availability_zones" "available" {
}

data "aws_caller_identity" "current" {
}
