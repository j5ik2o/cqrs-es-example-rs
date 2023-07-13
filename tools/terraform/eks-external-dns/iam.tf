module "iam_assumable_role_admin" {
  source                        = "registry.terraform.io/terraform-aws-modules/iam/aws//modules/iam-assumable-role-with-oidc"
  create_role                   = var.create
  role_name                     = local.iam_role_name
  provider_url                  = replace(var.eks_cluster_oidc_issuer_url, "https://", "")
  role_policy_arns              = concat(aws_iam_policy.external-dns.*.arn, [""])
  oidc_fully_qualified_subjects = [
    "system:serviceaccount:${local.k8s_service_namespace}:${local.k8s_service_account_name}"
  ]

  number_of_role_policy_arns = length(aws_iam_policy.external-dns)

  depends_on = [
    kubernetes_service_account.external-dns
  ]
}

resource "aws_iam_policy" "external-dns" {
  count  = var.create ? 1 : 0
  name   = local.iam_policy_name_prefix
  policy = <<-EOT
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "route53:ChangeResourceRecordSets"
      ],
      "Resource": [
        "arn:aws:route53:::hostedzone/*"
      ]
    },
    {
      "Effect": "Allow",
      "Action": [
        "route53:ListHostedZones",
        "route53:ListResourceRecordSets",
        "route53:ListTagsForResource"
      ],
      "Resource": [
        "*"
      ]
    }
  ]
}
EOT
}