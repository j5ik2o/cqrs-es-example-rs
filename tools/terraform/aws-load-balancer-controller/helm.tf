resource "helm_release" "aws-load-balancer-controller" {
  count     = var.create ? 1 : 0
  name      = "aws-load-balancer-controller"
  namespace = "kube-system"
  chart     = "https://aws.github.io/eks-charts/aws-load-balancer-controller-1.5.4.tgz"

  lifecycle {
    create_before_destroy = true
  }

  set {
    name  = "clusterName"
    value = var.eks_cluster_id
  }

  set {
    name  = "region"
    value = var.aws_region
  }

  set {
    name  = "vpcId"
    value = var.vpc_id
  }

  set {
    name  = "serviceAccount.create"
    value = true
  }

  set {
    name  = "serviceAccount.name"
    value = local.k8s_service_account_name
  }

  set {
    name  = "serviceAccount.annotations.eks\\.amazonaws\\.com/role-arn"
    value = "arn:aws:iam::${data.aws_caller_identity.self.account_id}:role/${local.iam_role_name}"
    type  = "string"
  }

  depends_on = [
    module.iam_assumable_role_admin
  ]
}