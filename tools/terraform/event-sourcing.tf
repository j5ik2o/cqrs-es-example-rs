module "event_sourcing" {
  source            = "./event-sourcing"
  enabled           = var.event_sourcing_enabled
  prefix            = var.prefix
  owner             = var.owner
  journal_name      = var.event_sourcing_journal_name
  journal_gsi_name  = var.event_sourcing_journal_gsi_name
  snapshot_name     = var.event_sourcing_snapshot_name
  snapshot_gsi_name = var.event_sourcing_snapshot_gsi_name
}