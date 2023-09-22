resource "aws_ecr_repository" "this" {
  name  = "${var.prefix}-ecr-${var.application_name}-${var.name}"
  count = var.enabled ? 1 : 0
  lifecycle {
    create_before_destroy = true
  }
  tags = {
    Name  = "${var.prefix}-ecr-${var.application_name}-${var.name}"
    Owner = var.owner
  }
}

resource "aws_ecr_repository_policy" "this" {
  count  = var.enabled ? 1 : 0
  policy = <<EOF
{
    "Version": "2008-10-17",
    "Statement": [
        {
            "Sid": "${aws_ecr_repository.this.0.name}-policy",
            "Effect": "Allow",
            "Principal": "*",
            "Action": [
                "ecr:GetDownloadUrlForLayer",
                "ecr:BatchGetImage",
                "ecr:BatchCheckLayerAvailability",
                "ecr:PutImage",
                "ecr:InitiateLayerUpload",
                "ecr:UploadLayerPart",
                "ecr:CompleteLayerUpload",
                "ecr:DescribeRepositories",
                "ecr:GetRepositoryPolicy",
                "ecr:ListImages",
                "ecr:DeleteRepository",
                "ecr:BatchDeleteImage",
                "ecr:SetRepositoryPolicy",
                "ecr:DeleteRepositoryPolicy"
            ]
        }
    ]
}
EOF

  repository = aws_ecr_repository.this.0.name
}