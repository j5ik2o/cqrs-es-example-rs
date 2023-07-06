#resource "kubernetes_secret" "external-dns" {
#  metadata {
#    name        = local.k8s_service_account_name
#    namespace   = local.k8s_service_namespace
#    annotations = {
#      "kubernetes.io/service-account.name"      = local.k8s_service_account_name
#      "kubernetes.io/service-account.namespace" = local.k8s_service_namespace
#    }
#  }
#  type = "kubernetes.io/service-account-token"
#
#  lifecycle {
#    create_before_destroy = true
#  }
#}

resource "kubernetes_service_account" "external-dns" {
  metadata {
    name        = local.k8s_service_account_name
    namespace   = local.k8s_service_namespace
    annotations = {
      "eks.amazonaws.com/role-arn" = "arn:aws:iam::${data.aws_caller_identity.self.account_id}:role/${local.iam_role_name}"
    }
  }
  # automount_service_account_token = false

  lifecycle {
    create_before_destroy = true
  }
}