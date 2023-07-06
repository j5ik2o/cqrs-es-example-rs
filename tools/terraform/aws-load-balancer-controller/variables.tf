variable "create" {
  default = true
}

variable "dependencies" {
  type    = list(string)
  default = []
}

variable "vpc_id" {}

variable "aws_region" {}

variable "prefix" {
  default = "sg"
}

variable "k8s_service_account_name" {
  default = "aws-load-balancer-controller"
}

variable "k8s_service_namespace" {
  default = "kube-system"
}

variable "eks_cluster_id" {
}

variable "eks_cluster_version" {
}

variable "eks_cluster_oidc_issuer_url" {
}
