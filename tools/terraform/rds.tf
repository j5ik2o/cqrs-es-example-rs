resource "random_password" "master" {
  length  = 20
  special = false
}

module "aurora" {
  source            = "registry.terraform.io/terraform-aws-modules/rds-aurora/aws"
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
      source_security_group_id = aws_security_group.lambda.id
    }
    vpc_ingress = {
      cidr_blocks = [module.vpc.vpc_cidr_block]
    }
    vpc2_ingress = {
      source_security_group_id = module.vpc.default_security_group_id
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

resource "aws_security_group" "rds" {
  name_prefix = "${local.name}-rds"
  description = "Allow MySQL inbound traffic"
  vpc_id      = module.vpc.vpc_id

  ingress {
    description = "TLS from VPC"
    from_port   = 3306
    to_port     = 3306
    protocol    = "tcp"
    cidr_blocks = [module.vpc.vpc_cidr_block]
  }

  tags = local.tags
}