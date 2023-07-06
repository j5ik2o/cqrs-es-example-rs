resource "helm_release" "cluster-autoscaler" {
  count     = var.create ? 1 : 0
  name      = "cluster-autoscaler"
  namespace = "kube-system"
  chart     = "https://github.com/kubernetes/autoscaler/releases/download/cluster-autoscaler-chart-9.29.1/cluster-autoscaler-9.29.1.tgz"

  set {
    name  = "awsRegion"
    value = var.aws_region
  }

  set {
    name  = "rbac.create"
    value = true
  }

  set {
    name  = "rbac.serviceAccount.create"
    value = true
  }

  set {
    name  = "rbac.serviceAccount.name"
    value = local.k8s_service_account_name
  }

  set {
    name  = "rbac.serviceAccount.annotations.eks\\.amazonaws\\.com/role-arn"
    value = "arn:aws:iam::${data.aws_caller_identity.self.account_id}:role/${local.iam_role_name}"
    type  = "string"
  }

  set {
    name  = "autoDiscovery.enabled"
    value = true
  }

  set {
    name  = "autoDiscovery.clusterName"
    value = var.eks_cluster_id
  }

  set {
    name  = "extraArgs.expander"
    value = "least-waste"
  }

  set {
    name  = "extraArgs.balance-similar-node-groups"
    value = true
  }

  set {
    name  = "extraArgs.skip-nodes-with-system-pods"
    value = false
  }

  set {
    name  = "podAnnotations.cluster-autoscaler\\kubernetes\\io/safe-to-evict"
    value = false
    type  = "string"
  }

  depends_on = [
    module.iam_assumable_role_admin
  ]
}