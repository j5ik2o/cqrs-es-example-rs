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
