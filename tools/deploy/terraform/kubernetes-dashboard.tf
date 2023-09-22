resource "helm_release" "kubernetes-dashboard" {
  name             = "kubernetes-dashboard"
  chart            = "https://kubernetes.github.io/dashboard/kubernetes-dashboard-6.0.8.tgz"
  namespace        = "kubernetes-dashboard"
  create_namespace = true

  lifecycle {
    create_before_destroy = true
  }

  set {
    name  = "rbac.create"
    value = true
  }

  depends_on = [
    module.eks
  ]
}

resource "helm_release" "k8s-dashboard-crb" {
  name      = "k8s-dashboard-crb"
  chart     = "../charts/k8s-dashboard-crb"
  namespace = "kubernetes-dashboard"

  lifecycle {
    create_before_destroy = true
  }

  set {
    name  = "serviceAccount.name"
    value = "kubernetes-dashboard"
  }

  set {
    name  = "serviceAccount.namespace"
    value = "kubernetes-dashboard"
  }

  depends_on = [
    helm_release.kubernetes-dashboard
  ]
}
