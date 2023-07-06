module "iam_assumable_role_admin" {
  source       = "registry.terraform.io/terraform-aws-modules/iam/aws//modules/iam-assumable-role-with-oidc"
  create_role  = var.create
  role_name    = local.iam_role_name
  provider_url = replace(var.eks_cluster_oidc_issuer_url, "https://", "")

  # ポリシーを追加したら number_of_role_policy_arns も変更しないと、追加したポリシーがアタッチされないことに注意！！
  role_policy_arns = [
    "arn:aws:iam::aws:policy/AmazonDynamoDBFullAccess",
    "arn:aws:iam::aws:policy/AmazonS3FullAccess",
    "arn:aws:iam::aws:policy/AmazonRDSFullAccess",
    "arn:aws:iam::aws:policy/CloudWatchFullAccess"
  ]
  number_of_role_policy_arns = 4

  oidc_fully_qualified_subjects = ["system:serviceaccount:${var.k8s_service_namespace}:${var.k8s_service_account_name}"]
}
