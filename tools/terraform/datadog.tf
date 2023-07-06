resource "helm_release" "datadog" {
  count     = var.datadog_enabled ? 1 : 0
  name      = "datadog"
  namespace = "kube-system"
  chart     = "https://github.com/DataDog/helm-charts/releases/download/datadog-2.35.3/datadog-2.35.3.tgz"

  lifecycle {
    create_before_destroy = true
  }

  set_sensitive {
    name  = "datadog.apiKey"
    value = var.datadog-api-key
  }

  set {
    name  = "datadog.logLevel"
    value = "INFO"
  }

  set {
    name  = "datadog.kubeStateMetricsEnabled"
    value = false
  }

  set {
    name  = "datadog.dogstatsd.nonLocalTraffic"
    value = true
  }

  set {
    name  = "apm.enabled"
    value = true
  }

  set {
    name  = "logs.enabled"
    value = false
  }

  set {
    name  = "logs.containerCollectAll"
    value = false
  }

  set {
    name  = "agents.useHostNetwork"
    value = true
  }

}