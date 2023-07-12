output "vpc_id" {
  value = module.vpc.vpc_id
}

output "vpc_public_subnets_ids" {
  value = module.vpc.public_subnets
}

output "vpc_private_subnet_ids" {
  value = module.vpc.private_subnets
}

output "vpc_public_subnet_cidr_blocks" {
  value = module.vpc.public_subnets_cidr_blocks
}

output "vpc_private_subnet_cidr_blocks" {
  value = module.vpc.private_subnets_cidr_blocks
}

output "vpc_subnet_azs" {
  value = module.vpc.azs
}

output "vpc_cidr_block" {
  value = module.vpc.vpc_cidr_block
}

#output "vpc_subnet_groups" {
#  value = {
#    "public" = [
#    for i in range(length(module.vpc.azs)):
#    {
#      cidr = module.vpc.public_subnets_cidr_blocks[i],
#      az = module.vpc.azs[i],
#      id = module.vpc.public_subnets[i],
#    }
#    ],
#    "private" = [
#    for i in range(length(module.vpc.azs)):
#    {
#      cidr = module.vpc.private_subnets_cidr_blocks[i],
#      az = module.vpc.azs[i],
#      id = module.vpc.private_subnets[i],
#    }
#    ]
#  }
#}

output "eks_cluster_name" {
  value = local.eks_cluster_name
}

output "eks_aws_auth_config_map" {
  value = module.eks.aws_auth_configmap_yaml
}

output "event_sourcing_journal_table_name" {
  value = module.event_sourcing.aws_dynamodb_table_journal_table_name
}

output "event_sourcing_journal_gsi_name" {
  value = module.event_sourcing.aws_dynamodb_table_journal_gsi_name
}

output "event_sourcing_snapshot_table_name" {
  value = module.event_sourcing.aws_dynamodb_table_snapshot_table_name
}

output "event_sourcing_snapshot_gsi_name" {
  value = module.event_sourcing.aws_dynamodb_table_snapshot_gsi_name
}

output "ecr_write_api_server_repository_url" {
  value = module.ceer-ecr-write-api-server.aws_ecr_repository_url
}

output "ecr_read_model_updater_repository_url" {
  value = module.ceer-ecr-read-model-updater.aws_ecr_repository_url
}

output "ecr_read_api_server_repository_url" {
  value = module.ceer-ecr-read-api-server.aws_ecr_repository_url
}

output "aurora_cluster_endpoint" {
  description = "Writer endpoint for the cluster"
  value       = module.aurora.cluster_endpoint
}

output "aurora_cluster_reader_endpoint" {
  description = "A read-only endpoint for the cluster, automatically load-balanced across replicas"
  value       = module.aurora.cluster_reader_endpoint
}

output "aurora_cluster_engine_version_actual" {
  description = "The running version of the cluster database"
  value       = module.aurora.cluster_engine_version_actual
}

output "aurora_cluster_database_name" {
  description = "Name for an automatically created database on cluster creation"
  value       = module.aurora.cluster_database_name
}

output "aurora_cluster_port" {
  description = "The database port"
  value       = module.aurora.cluster_port
}

output "aurora_cluster_master_password" {
  description = "The database master password"
  value       = module.aurora.cluster_master_password
  sensitive   = true
}

output "aurora_cluster_master_username" {
  description = "The database master username"
  value       = module.aurora.cluster_master_username
  sensitive   = true
}

