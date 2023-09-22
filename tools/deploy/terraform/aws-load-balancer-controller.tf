resource "helm_release" "alb-controller-crds" {
  name  = "alb-controller-crds"
  chart = "../charts/alb-controller-crds"

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [
    module.eks
  ]
}

module "aws-load-balancer-controller" {
  source                      = "./aws-load-balancer-controller"
  create                      = var.eks_enabled
  aws_region                  = var.aws_region
  vpc_id                      = module.vpc.vpc_id
  prefix                      = var.prefix
  eks_cluster_id              = module.eks.cluster_name
  eks_cluster_version         = module.eks.cluster_version
  eks_cluster_oidc_issuer_url = module.eks.cluster_oidc_issuer_url

  depends_on = [
    helm_release.alb-controller-crds
  ]
}
