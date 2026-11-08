resource "aws_dynamodb_table" "journal-table" {
  count = var.enabled ? 1 : 0
  name  = "${var.prefix}-${var.journal_name}"

  hash_key  = "aid"
  range_key = "seq_nr"

  lifecycle {
    # The v5 key schema replaces the table. Require an explicit opt-out before
    # an apply or destroy can remove the events.
    prevent_destroy = true
  }

  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "aid"
    type = "S"
  }

  attribute {
    name = "seq_nr"
    type = "N"
  }

  tags = {
    Name  = "${var.prefix}-${var.journal_name}"
    Owner = var.owner
  }
}

resource "aws_dynamodb_table" "snapshot-table" {
  count = var.enabled ? 1 : 0
  name  = "${var.prefix}-${var.snapshot_name}"

  hash_key  = "aid"
  range_key = "skey"

  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "aid"
    type = "S"
  }

  attribute {
    name = "skey"
    type = "N"
  }

  attribute {
    name = "active_history_seq_nr"
    type = "N"
  }

  global_secondary_index {
    name            = "${var.prefix}-${var.snapshot_history_index_name}"
    hash_key        = "aid"
    range_key       = "active_history_seq_nr"
    projection_type = "KEYS_ONLY"
  }

  ttl {
    attribute_name = "ttl"
    enabled        = true
  }

  tags = {
    Name  = "${var.prefix}-${var.snapshot_name}"
    Owner = var.owner
  }
}

resource "aws_dynamodb_table" "head-table" {
  count = var.enabled ? 1 : 0
  name  = "${var.prefix}-${var.head_name}"

  hash_key = "aid"

  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "aid"
    type = "S"
  }

  stream_enabled   = true
  stream_view_type = "NEW_IMAGE"

  tags = {
    Name  = "${var.prefix}-${var.head_name}"
    Owner = var.owner
  }
}
