output "aws_ecr_repository_arn" {
  value = concat(aws_ecr_repository.this.*.arn, [""])[0]
}

output "aws_ecr_repository_url" {
  value = concat(aws_ecr_repository.this.*.repository_url, [""])[0]
}