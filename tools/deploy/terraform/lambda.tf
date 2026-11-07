module "lambda_function" {
  source = "registry.terraform.io/terraform-aws-modules/lambda/aws"

  create = var.read_model_updater_enabled

  function_name = "${var.prefix}-lambda-read-model-updater"
  description   = "read-model-updater"

  create_package = false

  timeout = 60

  image_uri     = "${module.ceer-ecr-read-model-updater.aws_ecr_repository_url}:${var.read_model_updater_tag}"
  package_type  = "Image"
  architectures = ["x86_64"]

  #  cargo-lambda-build-read-model-updater.shを使う場合の設定
  #  handler                 = "bootstrap"
  #  runtime                 = "provided.al2"
  #  local_existing_package  = "${path.module}/../../target/lambda/read-model-updater/bootstrap-0d385892717f221ea18475226115fedc.zip"
  #  ignore_source_code_hash = true

  environment_variables = {
    "RUST_LOG"                        = "debug"
    "RUST_BACKTRACE"                  = "full"
    "APP__DATABASE__URL"              = "mysql://${module.aurora.cluster_master_username}:${module.aurora.cluster_master_password}@${module.aurora.cluster_endpoint}/${module.aurora.cluster_database_name}"
    "APP__AWS__REGION_NAME"           = "ap-northeast-1"
    "APP__STREAM__HEAD_TABLE_NAME"    = module.event_sourcing.aws_dynamodb_table_head_table_name
    "APP__STREAM__JOURNAL_TABLE_NAME" = module.event_sourcing.aws_dynamodb_table_journal_table_name
  }

  event_source_mapping = {
    dynamodb = {
      event_source_arn  = module.event_sourcing.aws_dynamodb_table_head_stream_arn
      starting_position = "LATEST"
    }
  }

  allowed_triggers = {
    dynamodb = {
      principal  = "dynamodb.amazonaws.com"
      source_arn = module.event_sourcing.aws_dynamodb_table_head_stream_arn
    }
  }
  create_current_version_allowed_triggers = false

  attach_network_policy  = true
  vpc_security_group_ids = [aws_security_group.lambda.id]
  vpc_subnet_ids         = module.vpc.private_subnets

  attach_policies    = true
  number_of_policies = 2
  policies = [
    "arn:aws:iam::aws:policy/service-role/AWSLambdaDynamoDBExecutionRole",
    "arn:aws:iam::aws:policy/AmazonRDSDataFullAccess",
  ]

  # AWSLambdaDynamoDBExecutionRole grants Streams reads only. The updater also
  # queries the journal to fill seq_nr gaps.
  attach_policy_statements = true
  policy_statements = {
    journal_read = {
      effect    = "Allow"
      actions   = ["dynamodb:Query"]
      resources = [module.event_sourcing.aws_dynamodb_table_journal_table_arn]
    }
  }

  depends_on = [
    module.aurora,
  ]
}

resource "aws_security_group" "lambda" {
  name_prefix = "${var.prefix}-lambda-read-model-updater"
  description = "Allow MySQL outbound traffic"
  vpc_id      = module.vpc.vpc_id

  egress {
    from_port   = 3306
    to_port     = 3306
    protocol    = "tcp"
    cidr_blocks = [module.vpc.vpc_cidr_block]
  }

  egress {
    from_port       = 443
    to_port         = 443
    protocol        = "tcp"
    prefix_list_ids = [aws_vpc_endpoint.dynamodb.prefix_list_id]
  }

  tags = local.tags
}