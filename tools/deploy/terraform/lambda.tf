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
      event_source_arn = module.event_sourcing.aws_dynamodb_table_head_stream_arn

      # TRIM_HORIZON replays the retained records after a restart. The
      # read_model_positions table drops the records that were already applied.
      starting_position             = "TRIM_HORIZON"
      maximum_retry_attempts        = 5
      maximum_record_age_in_seconds = 86400

      # Split a failed batch so healthy records are not replayed, and use the
      # partial batch response from the handler.
      bisect_batch_on_function_error = true
      function_response_types        = ["ReportBatchItemFailures"]

      # The recovery task table is the durable DLQ. This queue is the safety net
      # for records that exhaust the event source mapping's retries.
      destination_arn_on_failure = aws_sqs_queue.read_model_updater_dlq.arn
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
    recovery_dlq_send = {
      effect    = "Allow"
      actions   = ["sqs:SendMessage"]
      resources = [aws_sqs_queue.read_model_updater_dlq.arn]
    }
  }

  depends_on = [
    module.aurora,
  ]
}

resource "aws_sqs_queue" "read_model_updater_dlq" {
  name                      = "${var.prefix}-read-model-updater-dlq"
  message_retention_seconds = 1209600
  sqs_managed_sse_enabled   = true
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