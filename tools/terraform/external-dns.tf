module "external-dns" {
  source                      = "./eks-external-dns"
  create = true
  eks_cluster_oidc_issuer_url = module.eks.cluster_oidc_issuer_url
  zone_id                     = var.zone_id
  zone_name                   = var.zone_name

  policy = "upsert-only"
  interval = "30m"
  trigger_loop_on_event = true

  depends_on = [
    module.eks
  ]
}
