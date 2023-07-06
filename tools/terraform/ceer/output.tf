output "iam_role_arn" {
  value = module.iam_assumable_role_admin.iam_role_arn
}

output "iam_role_name" {
  value = module.iam_assumable_role_admin.iam_role_name
}

output "iam_role_path" {
  value = module.iam_assumable_role_admin.iam_role_path
}
