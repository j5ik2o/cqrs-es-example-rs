module "lambda_function" {
  source = "registry.terraform.io/terraform-aws-modules/lambda/aws"

  create = false

  function_name = "read-model-updater"
  description   = "read-model-updater"

  create_package = false

  image_uri     = "738575627980.dkr.ecr.ap-northeast-1.amazonaws.com/aht9aa1e-ecr-ceer-read-model-updater:latest-amd64"
  package_type  = "Image"
  architectures = ["x86_64"]

  environment_variables = {
    "RUST_LOG"           = "debug"
    "RUST_BACKTRACE"     = "full"
    "APP__DATABASE__URL" = "mysql://${module.aurora.cluster_master_username}:${module.aurora.cluster_master_password}@${module.aurora.cluster_endpoint}/${module.aurora.cluster_database_name}"
  }

  attach_policy = true
  policy        = "arn:aws:iam::aws:policy/AmazonDynamoDBFullAccess"

  event_source_mapping = {
    dynamodb = {
      event_source_arn  = module.event_sourcing.aws_dynamodb_table_journal_stream_arn
      starting_position = "LATEST"
    }
  }

  depends_on = [
    module.aurora,
  ]
}
