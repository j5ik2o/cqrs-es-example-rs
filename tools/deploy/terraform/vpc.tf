module "vpc" {
  source = "registry.terraform.io/terraform-aws-modules/vpc/aws"

  name = "${var.prefix}-vpc-${var.name}"
  cidr = var.vpc_cidr
  azs  = data.aws_availability_zones.available.names

  public_subnets  = var.aws_subnet_public
  private_subnets = var.aws_subnet_private

  database_subnets = var.aws_subnet_database

  enable_nat_gateway   = true
  single_nat_gateway   = true
  enable_dns_hostnames = true

  manage_default_security_group  = true
  default_security_group_ingress = []
  default_security_group_egress  = []

  tags = {
    Name        = "${var.prefix}-vpc-${var.name}"
    Environment = var.prefix
  }

  public_subnet_tags = {
    "kubernetes.io/cluster/${local.eks_cluster_name}" = "shared"
    "kubernetes.io/role/elb"                          = "1"
  }

  private_subnet_tags = {
    "kubernetes.io/cluster/${local.eks_cluster_name}" = "shared"
    "kubernetes.io/role/internal-elb"                 = "1"
  }
}

resource "aws_vpc_endpoint" "dynamodb" {
  vpc_id            = module.vpc.vpc_id
  service_name      = "com.amazonaws.${data.aws_region.current.name}.dynamodb"
  vpc_endpoint_type = "Gateway"
  route_table_ids   = module.vpc.private_route_table_ids

  tags = local.tags
}
