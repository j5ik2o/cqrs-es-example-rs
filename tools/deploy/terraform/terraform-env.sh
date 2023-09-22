# 新しいAWSアカウントの使用を開始する際、
# TF_BUCKET_NAME に指定するS3バケット
# TF_LOCK_TABLE_NAME に指定するDynamoDBテーブル
# は事前にAWSコンソールから操作するなどして作成しておく必要があります

TF_BUCKET_NAME=$PREFIX-$APPLICATION_NAME-terraform
TF_STATE_NAME=$PREFIX-$APPLICATION_NAME-terraform.tfstate
TF_LOCK_TABLE_NAME=$PREFIX-$APPLICATION_NAME-terraform-lock
TF_VAR_FILE=$PREFIX-$APPLICATION_NAME-terraform.tfvars

export TF_VAR_aws_profile=$AWS_PROFILE
export TF_VAR_aws_region=$AWS_REGION
export TF_VAR_prefix=$PREFIX
export TF_VAR_name=$APPLICATION_NAME
# export TF_VAR_datadog_api_key=$DATADOG_API_KEY
export TF_VAR_read_model_updater_enabled=$READ_MODEL_UPDATER_ENABLED

export TF_LOG=DEBUG
export TF_LOG_PATH=./debug.log
