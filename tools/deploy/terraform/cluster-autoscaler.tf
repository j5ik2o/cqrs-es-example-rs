module "cluster-autoscaler" {
  source                      = "./cluster-autoscaler"
  create                      = var.eks_enabled
  aws_region                  = var.aws_region
  prefix                      = var.prefix
  eks_cluster_id              = module.eks.cluster_name
  eks_cluster_version         = module.eks.cluster_version
  eks_cluster_oidc_issuer_url = module.eks.cluster_oidc_issuer_url
  depends_on                  = [
    module.eks
  ]
}