locals {
  rmu_ops = jsondecode(file("${path.module}/../rmu/ops.json"))
}

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
  name                       = "${var.prefix}-read-model-updater-dlq"
  message_retention_seconds  = local.rmu_ops.recoveryQueue.messageRetentionSeconds
  visibility_timeout_seconds = local.rmu_ops.recoveryQueue.visibilityTimeoutSeconds
  sqs_managed_sse_enabled    = true

  # The recovery worker consumes this queue. A message that still fails after
  # the configured receives is parked in the recovery DLQ.
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.read_model_updater_recovery_dlq.arn
    maxReceiveCount     = local.rmu_ops.recoveryQueue.maxReceiveCount
  })
}

resource "aws_sqs_queue" "read_model_updater_recovery_dlq" {
  name                      = "${var.prefix}-read-model-updater-recovery-dlq"
  message_retention_seconds = 1209600
  sqs_managed_sse_enabled   = true
}

resource "aws_security_group" "lambda" {
  name_prefix = "${var.prefix}-lambda-read-model-updater"
  description = "Allow MySQL, DynamoDB, and CloudWatch outbound traffic"
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

  # The recovery worker publishes custom metrics with PutMetricData. The VPC
  # has a single NAT gateway, so CloudWatch is reached over the internet path.
  egress {
    description = "CloudWatch PutMetricData through the NAT gateway"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = local.tags
}

# The recovery worker uses the same image as the stream updater and selects the
# recovery pass with APP__RECOVERY_WORKER=true. It consumes the OnFailure queue
# and also runs on an EventBridge schedule, so a lost notification still drains
# the pending tasks.
module "lambda_function_recovery" {
  source = "registry.terraform.io/terraform-aws-modules/lambda/aws"

  create = var.read_model_updater_enabled

  function_name = "${var.prefix}-lambda-read-model-updater-recovery"
  description   = "read-model-updater recovery worker"

  create_package = false
  timeout        = local.rmu_ops.recoveryWorker.timeoutSeconds
  image_uri      = "${module.ceer-ecr-read-model-updater.aws_ecr_repository_url}:${var.read_model_updater_tag}"
  package_type   = "Image"
  architectures  = ["x86_64"]

  # One worker avoids two passes racing for the same recovery task.
  reserved_concurrent_executions = local.rmu_ops.recoveryWorker.reservedConcurrentExecutions

  environment_variables = {
    "APP__RECOVERY_WORKER"            = "true"
    "RUST_LOG"                        = "debug"
    "RUST_BACKTRACE"                  = "full"
    "APP__DATABASE__URL"              = "mysql://${module.aurora.cluster_master_username}:${module.aurora.cluster_master_password}@${module.aurora.cluster_endpoint}/${module.aurora.cluster_database_name}"
    "APP__AWS__REGION_NAME"           = "ap-northeast-1"
    "APP__STREAM__HEAD_TABLE_NAME"    = module.event_sourcing.aws_dynamodb_table_head_table_name
    "APP__STREAM__JOURNAL_TABLE_NAME" = module.event_sourcing.aws_dynamodb_table_journal_table_name
  }

  event_source_mapping = {
    sqs = {
      event_source_arn        = aws_sqs_queue.read_model_updater_dlq.arn
      batch_size              = local.rmu_ops.recoveryQueue.batchSize
      function_response_types = ["ReportBatchItemFailures"]
      # SQS retries are governed by the queue's redrive_policy.maxReceiveCount,
      # not by an event source mapping parameter. Concurrency is capped by
      # reserved_concurrent_executions above.
    }
  }

  allowed_triggers = {
    sqs = {
      principal  = "sqs.amazonaws.com"
      source_arn = aws_sqs_queue.read_model_updater_dlq.arn
    }
  }
  create_current_version_allowed_triggers = false

  attach_network_policy  = true
  vpc_security_group_ids = [aws_security_group.lambda.id]
  vpc_subnet_ids         = module.vpc.private_subnets

  attach_policies    = true
  number_of_policies = 1
  policies = [
    "arn:aws:iam::aws:policy/service-role/AWSLambdaSQSQueueExecutionRole",
  ]

  attach_policy_statements = true
  policy_statements = {
    head_scan = {
      effect    = "Allow"
      actions   = ["dynamodb:Scan"]
      resources = [module.event_sourcing.aws_dynamodb_table_head_table_arn]
    }
    journal_read = {
      effect    = "Allow"
      actions   = ["dynamodb:Query"]
      resources = [module.event_sourcing.aws_dynamodb_table_journal_table_arn]
    }
    recovery_metrics = {
      effect    = "Allow"
      actions   = ["cloudwatch:PutMetricData"]
      resources = ["*"]
    }
  }

  depends_on = [
    module.aurora,
  ]
}

resource "aws_cloudwatch_event_rule" "rmu_recovery_pass" {
  count = var.read_model_updater_enabled ? 1 : 0

  name                = "${var.prefix}-rmu-recovery-pass"
  description         = "Periodic read model recovery pass"
  schedule_expression = local.rmu_ops.schedule.recoveryPass
}

resource "aws_cloudwatch_event_target" "rmu_recovery_pass" {
  count = var.read_model_updater_enabled ? 1 : 0

  rule      = aws_cloudwatch_event_rule.rmu_recovery_pass[0].name
  target_id = "read-model-updater-recovery"
  arn       = module.lambda_function_recovery.lambda_function_arn
}

resource "aws_lambda_permission" "rmu_recovery_pass" {
  count = var.read_model_updater_enabled ? 1 : 0

  statement_id  = "AllowExecutionFromEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = module.lambda_function_recovery.lambda_function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.rmu_recovery_pass[0].arn
}

# Alerts are published to SNS. Set var.rmu_ops_alert_email to also receive them
# by email; the topic is otherwise available for another subscription.
resource "aws_sns_topic" "rmu_ops_alerts" {
  name = "${var.prefix}-rmu-ops-alerts"

  tags = local.tags
}

resource "aws_sns_topic_policy" "rmu_ops_alerts" {
  arn = aws_sns_topic.rmu_ops_alerts.arn

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Effect    = "Allow"
      Principal = { Service = "cloudwatch.amazonaws.com" }
      Action    = "sns:Publish"
      Resource  = aws_sns_topic.rmu_ops_alerts.arn
    }]
  })
}

resource "aws_sns_topic_subscription" "rmu_ops_email" {
  count = var.rmu_ops_alert_email != "" ? 1 : 0

  topic_arn = aws_sns_topic.rmu_ops_alerts.arn
  protocol  = "email"
  endpoint  = var.rmu_ops_alert_email
}

locals {
  rmu_alarm_actions = [aws_sns_topic.rmu_ops_alerts.arn]
  rmu_alarm_ok_actions = [aws_sns_topic.rmu_ops_alerts.arn]
  rmu_metric_namespace  = "CqrsEsExample/Rmu"
  rmu_metric_dimensions = { Service = "read-model-updater" }
}

resource "aws_cloudwatch_metric_alarm" "rmu_stream_errors" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-stream-errors"
  alarm_description   = "The head stream updater returned errors."
  namespace           = "AWS/Lambda"
  metric_name         = "Errors"
  dimensions          = { FunctionName = "${var.prefix}-lambda-read-model-updater" }
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = 1
  statistic           = "Sum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.evaluationPeriods
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_stream_throttles" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-stream-throttles"
  alarm_description   = "The head stream updater is being throttled."
  namespace           = "AWS/Lambda"
  metric_name         = "Throttles"
  dimensions          = { FunctionName = "${var.prefix}-lambda-read-model-updater" }
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = 1
  statistic           = "Sum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.evaluationPeriods
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_recovery_worker_errors" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-recovery-worker-errors"
  alarm_description   = "The recovery worker returned errors."
  namespace           = "AWS/Lambda"
  metric_name         = "Errors"
  dimensions          = { FunctionName = "${var.prefix}-lambda-read-model-updater-recovery" }
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = 1
  statistic           = "Sum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = 1
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_on_failure_queue_depth" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-on-failure-queue-depth"
  alarm_description   = "Failed head stream batches are waiting for the recovery worker."
  namespace           = "AWS/SQS"
  metric_name         = "ApproximateNumberOfMessagesVisible"
  dimensions          = { QueueName = aws_sqs_queue.read_model_updater_dlq.name }
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = local.rmu_ops.alarms.queueDepthThreshold
  statistic           = "Maximum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.evaluationPeriods
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_on_failure_queue_age" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-on-failure-queue-age"
  alarm_description   = "A failed head stream batch has been waiting too long."
  namespace           = "AWS/SQS"
  metric_name         = "ApproximateAgeOfOldestMessage"
  dimensions          = { QueueName = aws_sqs_queue.read_model_updater_dlq.name }
  comparison_operator = "GreaterThanThreshold"
  threshold           = local.rmu_ops.alarms.queueAgeSeconds
  statistic           = "Maximum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.evaluationPeriods
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_recovery_dlq_depth" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-recovery-dlq-depth"
  alarm_description   = "The recovery worker exhausted its retries; inspect the recovery DLQ."
  namespace           = "AWS/SQS"
  metric_name         = "ApproximateNumberOfMessagesVisible"
  dimensions          = { QueueName = aws_sqs_queue.read_model_updater_recovery_dlq.name }
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = 1
  statistic           = "Maximum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = 1
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_pending_recovery_tasks" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-pending-recovery-tasks"
  alarm_description   = "Recovery tasks remain pending after the last pass."
  namespace           = local.rmu_metric_namespace
  metric_name         = "RecoveryTasksPending"
  dimensions          = local.rmu_metric_dimensions
  comparison_operator = "GreaterThanThreshold"
  threshold           = local.rmu_ops.alarms.pendingTasksThreshold
  statistic           = "Maximum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.customEvaluationPeriods
  treat_missing_data  = "ignore"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_failed_recovery_tasks" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-failed-recovery-tasks"
  alarm_description   = "The recovery worker could not apply one or more tasks."
  namespace           = local.rmu_metric_namespace
  metric_name         = "RecoveryTasksFailed"
  dimensions          = local.rmu_metric_dimensions
  comparison_operator = "GreaterThanThreshold"
  threshold           = local.rmu_ops.alarms.failedTasksThreshold
  statistic           = "Maximum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.customEvaluationPeriods
  treat_missing_data  = "ignore"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_resync_failures" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-resync-failures"
  alarm_description   = "The recovery pass could not reconcile one or more aggregates."
  namespace           = local.rmu_metric_namespace
  metric_name         = "ResyncFailed"
  dimensions          = local.rmu_metric_dimensions
  comparison_operator = "GreaterThanThreshold"
  threshold           = local.rmu_ops.alarms.resyncFailuresThreshold
  statistic           = "Maximum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = local.rmu_ops.alarms.customEvaluationPeriods
  treat_missing_data  = "ignore"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}

resource "aws_cloudwatch_metric_alarm" "rmu_recovery_schedule_failures" {
  count = var.read_model_updater_enabled ? 1 : 0

  alarm_name          = "${var.prefix}-rmu-recovery-schedule-failures"
  alarm_description   = "The recovery pass schedule could not invoke the worker."
  namespace           = "AWS/Events"
  metric_name         = "FailedInvocations"
  dimensions          = { RuleName = aws_cloudwatch_event_rule.rmu_recovery_pass[0].name }
  comparison_operator = "GreaterThanOrEqualToThreshold"
  threshold           = 1
  statistic           = "Sum"
  period              = local.rmu_ops.alarms.periodSeconds
  evaluation_periods  = 1
  treat_missing_data  = "notBreaching"
  alarm_actions       = local.rmu_alarm_actions
  ok_actions          = local.rmu_alarm_ok_actions
  tags                = local.tags
}