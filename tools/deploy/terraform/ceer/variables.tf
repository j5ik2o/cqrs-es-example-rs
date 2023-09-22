variable "create" {
  default = true
}

variable "prefix" {
}

variable "k8s_service_account_name" {
  default = "ceer"
}

variable "k8s_service_namespace" {
  default = "ceer"
}

variable "eks_cluster_id" {
}

variable "eks_cluster_version" {
}

variable "eks_cluster_oidc_issuer_url" {
}
