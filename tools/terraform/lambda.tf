#module "lambda_function" {
#  source = "registry.terraform.io/terraform-aws-modules/lambda/aws"
#  create = false
#  function_name = "read-model-updater"
#  description   = "read-model-updater"
#
#  create_package = false
#
#  image_uri     = "738575627980.dkr.ecr.ap-northeast-1.amazonaws.com/aht9aa1e-ecr-ceer-read-model-updater:latest"
#  package_type  = "Image"
#  architectures = ["x86_64"]
#
#  environment_variables = {
#    AWS_REGION="ap-northeast-1"
#    RUST_LOG="debug"
#    RUST_BACKTRACE="full"
#    APP__API__HOST="0.0.0.0"
#    APP__API__PORT="8080"
#    APP__DATABASE__URL="mysql://ceer:ceer@mysql-local:3306/ceer"
#  }
#
#  event_source_mapping = {
#    dynamodb = {
#      event_source_arn  = module.event_sourcing.aws_dynamodb_table_journal_stream_arn
#      starting_position = "LATEST"
#    }
#  }
#}
