module "ceer" {
  source                      = "./ceer"
  create                      = var.eks_enabled
  prefix                      = var.prefix
  k8s_service_namespace       = "ceer"
  k8s_service_account_name    = "ceer"
  eks_cluster_id              = module.eks.cluster_id
  eks_cluster_version         = module.eks.cluster_version
  eks_cluster_oidc_issuer_url = module.eks.cluster_oidc_issuer_url

  depends_on = [
    module.eks
  ]
}

