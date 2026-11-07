module "event_sourcing" {
  source                      = "./event-sourcing"
  enabled                     = var.event_sourcing_enabled
  prefix                      = var.prefix
  owner                       = var.owner
  journal_name                = var.event_sourcing_journal_name
  snapshot_name               = var.event_sourcing_snapshot_name
  snapshot_history_index_name = var.event_sourcing_snapshot_history_index_name
  head_name                   = var.event_sourcing_head_name
}
