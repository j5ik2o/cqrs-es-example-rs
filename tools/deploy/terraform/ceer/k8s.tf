resource "kubernetes_namespace" "ceer" {
  metadata {
    name = local.k8s_service_namespace
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "kubernetes_service_account" "write-api-server" {
  metadata {
    name        = local.k8s_service_account_name
    namespace   = local.k8s_service_namespace
    annotations = {
      "eks.amazonaws.com/role-arn" = "arn:aws:iam::${data.aws_caller_identity.self.account_id}:role/${local.iam_role_name}"
    }
  }

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [
    kubernetes_namespace.ceer
  ]
}