variable "create" {
  default = true
}

variable "dependencies" {
  type    = list(string)
  default = []
}

variable "prefix" {
  default = "sg"
}

variable "zone_id" {}
variable "zone_name" {}

variable "k8s_service_account_name" {
  default = "external-dns"
}

variable "k8s_service_namespace" {
  default = "kube-system"
}

variable "eks_cluster_oidc_issuer_url" {
}

variable "interval" {
  default = "30m"
}

variable "triggerLoopOnEvent" {
  default = true
}

variable "policy" {
  default = "upsert-only"
}