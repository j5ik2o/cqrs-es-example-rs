#module "lambda_function" {
#  source = "registry.terraform.io/terraform-aws-modules/lambda/aws"
#
#  function_name = "rmu"
#  description   = "read-model-updater"
#  handler       = "bootstrap"
#  runtime       = "provided.al2"
#
#  source_path = "${path.module}/../../target/lambda/cqrs-es-example-read-model-updater/bootstrap.zip"
#
#  event_source_mapping = {
#    dynamodb = {
#      event_source_arn  = module.event_sourcing.aws_dynamodb_table_journal_stream_arn
#      starting_position = "LATEST"
#    }
#  }
#}