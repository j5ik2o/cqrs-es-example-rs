module "ceer-ecr-write-api-server" {
  source           = "./ecr"
  enabled          = var.ecr_enabled
  prefix           = var.prefix
  application_name = var.name
  owner            = var.owner
  name             = "write-api-server"
}

module "ceer-ecr-read-model-updater" {
  source           = "./ecr"
  enabled          = var.ecr_enabled
  prefix           = var.prefix
  application_name = var.name
  owner            = var.owner
  name             = "read-model-updater"
}

module "ceer-ecr-read-api-server" {
  source           = "./ecr"
  enabled          = var.ecr_enabled
  prefix           = var.prefix
  application_name = var.name
  owner            = var.owner
  name             = "read-api-server"
}
