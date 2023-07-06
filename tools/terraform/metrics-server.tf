resource "helm_release" "metrics-server" {
  name      = "metrics-server"
  chart     = "https://github.com/kubernetes-sigs/metrics-server/releases/download/metrics-server-helm-chart-3.10.0/metrics-server-3.10.0.tgz"
  namespace = "kube-system"
  lifecycle {
    create_before_destroy = true
  }

  depends_on = [
    module.eks
  ]
}
