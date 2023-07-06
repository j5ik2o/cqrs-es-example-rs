locals {
  application_name         = "external-dns"
  k8s_service_namespace    = var.k8s_service_namespace
  k8s_service_account_name = var.k8s_service_account_name
  iam_policy_name_prefix   = "${var.prefix}-${local.application_name}"
  iam_role_name            = "${var.prefix}-${local.application_name}"
}