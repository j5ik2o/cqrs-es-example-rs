module "lambda_function" {
  source = "registry.terraform.io/terraform-aws-modules/lambda/aws"

  create = true

  create_function = true

  function_name = "${var.prefix}-lambda-read-model-updater"
  description   = "read-model-updater"

  create_package = false

  timeout = 60

#  image_uri     = "738575627980.dkr.ecr.ap-northeast-1.amazonaws.com/aht9aa1e-ecr-ceer-read-model-updater:latest-amd64"
#  package_type  = "Image"
  architectures = ["x86_64"]

  handler                 = "bootstrap"
  runtime                 = "provided.al2"
  local_existing_package  = "${path.module}/../../target/lambda/cqrs-es-example-read-model-updater/bootstrap-0d385892717f221ea18475226115fedc.zip"
  ignore_source_code_hash = true

  environment_variables = {
    "RUST_LOG"              = "debug"
    "RUST_BACKTRACE"        = "full"
    "APP__DATABASE__URL"    = "mysql://${module.aurora.cluster_master_username}:${module.aurora.cluster_master_password}@${module.aurora.cluster_endpoint}/${module.aurora.cluster_database_name}"
    "APP__AWS__REGION_NAME" = "ap-northeast-1"
  }

  event_source_mapping = {
    dynamodb = {
      event_source_arn  = module.event_sourcing.aws_dynamodb_table_journal_stream_arn
      starting_position = "LATEST"
    }
  }

  allowed_triggers = {
    dynamodb = {
      principal  = "dynamodb.amazonaws.com"
      source_arn = module.event_sourcing.aws_dynamodb_table_journal_stream_arn
    }
  }
  create_current_version_allowed_triggers = false
  attach_network_policy = true


  vpc_subnet_ids         = module.vpc.private_subnets
  vpc_security_group_ids = [module.vpc.default_security_group_id]

  attach_policies    = true
  number_of_policies = 1

  policies = [
    "arn:aws:iam::aws:policy/service-role/AWSLambdaDynamoDBExecutionRole",
  ]

  depends_on = [
    module.aurora,
  ]
}
