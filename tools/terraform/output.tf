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

output "akka_persistence_journal_table_name" {
  value = module.akka-persistence.aws_dynamodb_table_journal_table_name
}

output "akka_persistence_journal_gsi_name" {
  value = module.akka-persistence.aws_dynamodb_table_journal_gsi_name
}

output "akka_persistence_snapshot_table_name" {
  value = module.akka-persistence.aws_dynamodb_table_snapshot_table_name
}

output "akka_persistence_snapshot_gsi_name" {
  value = module.akka-persistence.aws_dynamodb_table_snapshot_gsi_name
}

