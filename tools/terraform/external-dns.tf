module "external-dns" {
  source                      = "./eks-external-dns"
  eks_cluster_oidc_issuer_url = module.eks.cluster_oidc_issuer_url
  zone_id                     = var.zone_id
  zone_name                   = var.zone_name

  depends_on = [
    module.eks
  ]
}
