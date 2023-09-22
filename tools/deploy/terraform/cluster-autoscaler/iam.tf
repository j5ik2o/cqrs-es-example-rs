module "iam_assumable_role_admin" {
  source                        = "registry.terraform.io/terraform-aws-modules/iam/aws//modules/iam-assumable-role-with-oidc"
  create_role                   = var.create
  role_name                     = local.iam_role_name
  provider_url                  = replace(var.eks_cluster_oidc_issuer_url, "https://", "")
  role_policy_arns              = concat(aws_iam_policy.cluster_autoscaler.*.arn, [""])
  oidc_fully_qualified_subjects = [
    "system:serviceaccount:${local.k8s_service_namespace}:${local.k8s_service_account_name}"
  ]

  number_of_role_policy_arns = length(aws_iam_policy.cluster_autoscaler)
}

resource "aws_iam_policy" "cluster_autoscaler" {
  count       = var.create ? 1 : 0
  name_prefix = local.iam_policy_name_prefix
  description = "EKS cluster-autoscaler policy for cluster ${var.eks_cluster_id}"
  policy      = element(concat(data.aws_iam_policy_document.cluster_autoscaler.*.json, [""]), 0)
}

data "aws_iam_policy_document" "cluster_autoscaler" {
  count = var.create ? 1 : 0
  statement {
    sid    = "clusterAutoscalerAll"
    effect = "Allow"

    actions = [
      "autoscaling:DescribeAutoScalingGroups",
      "autoscaling:DescribeAutoScalingInstances",
      "autoscaling:DescribeInstances",
      "autoscaling:DescribeLaunchConfigurations",
      "autoscaling:DescribeTags",
      "autoscaling:SetDesiredCapacity",
      "autoscaling:TerminateInstanceInAutoScalingGroup",
      "ec2:DescribeLaunchTemplateVersions",
      "ec2:DescribeInstanceTypes"
    ]

    resources = ["*"]
  }

  statement {
    sid    = "clusterAutoscalerOwn"
    effect = "Allow"

    actions = [
      "autoscaling:SetDesiredCapacity",
      "autoscaling:TerminateInstanceInAutoScalingGroup",
      "autoscaling:UpdateAutoScalingGroup",
    ]

    resources = ["*"]

    condition {
      test     = "StringEquals"
      variable = "autoscaling:ResourceTag/k8s.io/cluster-autoscaler/${var.eks_cluster_id}"
      values   = ["owned"]
    }

    condition {
      test     = "StringEquals"
      variable = "autoscaling:ResourceTag/k8s.io/cluster-autoscaler/enabled"
      values   = ["true"]
    }
  }
}