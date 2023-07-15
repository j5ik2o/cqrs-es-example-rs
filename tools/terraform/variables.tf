variable "aws_region" {
  default = ""
}

variable "aws_profile" {
  default = "test1"
}

variable "prefix" {
  default = "test1"
}

variable "name" {
  default = "akka-app"
}

variable "owner" {
  default = "test1"
}

variable "vpc_cidr" {
  default = "10.0.0.0/16"
}

variable "aws_subnet_public" {
  type    = list(string)
  default = [
    "10.0.1.0/24",
    "10.0.2.0/24",
    "10.0.3.0/24"
  ]
}

variable "aws_subnet_private" {
  type    = list(string)
  default = [
    "10.0.4.0/24",
    "10.0.5.0/24",
    "10.0.6.0/24"
  ]
}

variable "aws_subnet_database" {
  type    = list(string)
  default = [
    "10.0.20.0/24",
    "10.0.21.0/24"
  ]
}

variable "event_sourcing_enabled" {
  type = bool
}

variable "event_sourcing_journal_name" {
  type = string
}

variable "event_sourcing_journal_gsi_name" {
  type = string
}

variable "event_sourcing_snapshot_name" {
  type = string
}

variable "event_sourcing_snapshot_gsi_name" {
  type = string
}

variable "datadog-api-key" {
  type = string
}

variable "eks_enabled" {
  default = false
}

variable "eks_version" {
  default = "1.27"
}

variable "ecr_enabled" {
  default = false
}

variable "eks_node_instance_type" {
  default = "t2.medium"
}


variable "eks_asg_desired_capacity" {
  default = 3
}

variable "eks_asg_min_size" {
  default = 3
}

variable "eks_asg_max_size" {
  default = 10
}

variable "eks_auth_accounts" {
  description = "Additional AWS account numbers to add to the aws-auth configmap."
  type        = list(string)

  default = [
    "777777777777",
    "888888888888",
  ]
}

variable "eks_auth_roles" {
  description = "Additional IAM roles to add to the aws-auth configmap."
  type        = list(object({
    rolearn  = string
    username = string
    groups   = list(string)
  }))

  default = [
    {
      rolearn  = "arn:aws:iam::66666666666:role/role1"
      username = "role1"
      groups   = [
        "system:masters"
      ]
    },
  ]
}

variable "eks_auth_users" {
  description = "Additional IAM users to add to the aws-auth configmap."
  type        = list(object({
    userarn  = string
    username = string
    groups   = list(string)
  }))

  default = [
    {
      userarn  = "arn:aws:iam::66666666666:user/user1"
      username = "user1"
      groups   = [
        "system:masters"
      ]
    },
    {
      userarn  = "arn:aws:iam::66666666666:user/user2"
      username = "user2"
      groups   = [
        "system:masters"
      ]
    },
  ]
}

variable "eks_root_volume_type" {
  default = "gp2"
}

variable "datadog_enabled" {
  default = false
}

variable "zone_id" {}
variable "zone_name" {}

variable "read_model_updater_enabled" {
  default = false
}
variable "read_model_updater_tag" {}