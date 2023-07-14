resource "helm_release" "external-dns" {
  count     = var.create ? 1 : 0
  name      = "external-dns"
  namespace = local.k8s_service_namespace
  chart     = "https://github.com/kubernetes-sigs/external-dns/releases/download/external-dns-helm-chart-1.12.2/external-dns-1.12.2.tgz"

  lifecycle {
    create_before_destroy = true
  }

  set {
    name  = "serviceAccount.create"
    value = false
  }

  set {
    name  = "serviceAccount.name"
    value = local.k8s_service_account_name
  }

  set {
    name  = "provider"
    value = "aws"
  }

  set {
    name = "interval"
    value = var.interval
  }

  set {
    name = "triggerLoopOnEvent"
    value = var.triggerLoopOnEvent
  }

  set {
    name  = "policy"
    value = var.policy
  }

  set {
    name  = "domainFilters[0]"
    value = var.zone_name
  }

  set {
    name  = "txtOwnerId"
    value = var.zone_id
    type  = "string"
  }

  depends_on = [
    module.iam_assumable_role_admin,
    kubernetes_service_account.external-dns
  ]
}