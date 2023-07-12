resource "random_password" "master" {
  length  = 20
  special = false
}

module "aurora" {
  source            = "terraform-aws-modules/rds-aurora/aws"
  name              = "${local.name}-mysql"
  engine            = "aurora-mysql"
  engine_mode       = "serverless"
  storage_encrypted = true
  database_name     = "ceer"
  master_username   = "root"

  vpc_id               = module.vpc.vpc_id
  db_subnet_group_name = module.vpc.database_subnet_group_name
  security_group_rules = {
    vpc_ingress = {
      cidr_blocks = module.vpc.private_subnets_cidr_blocks
    }
  }

  # Serverless v1 clusters do not support managed master user password
  manage_master_user_password = false
  master_password             = random_password.master.result

  monitoring_interval = 60

  apply_immediately   = true
  skip_final_snapshot = true

  # enabled_cloudwatch_logs_exports = # NOT SUPPORTED

  scaling_configuration = {
    auto_pause               = true
    min_capacity             = 2
    max_capacity             = 16
    seconds_until_auto_pause = 300
    timeout_action           = "ForceApplyCapacityChange"
  }

  tags = local.tags
}