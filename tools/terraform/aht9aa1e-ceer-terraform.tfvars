event_sourcing_enabled           = true
event_sourcing_journal_name      = "journal"
event_sourcing_journal_gsi_name  = "journal-aid-index"
event_sourcing_snapshot_name     = "snapshot"
event_sourcing_snapshot_gsi_name = "snapshot-aid-index"

eks_enabled       = true
eks_version       = "1.27"
eks_auth_roles    = []
eks_auth_users    = []
eks_auth_accounts = []

ecr_enabled = true

datadog_enabled = false
datadog-api-key = "xxxx"

read_model_updater_enabled = false
read_model_updater_tag = "9ed584699fe19cab82121fae2d4ac7f1eee2e49089ba463cdd7378085ccc7b39-amd64"