resource "aws_dynamodb_table" "journal-table" {
  count = var.enabled ? 1 : 0
  name  = "${var.prefix}-${var.journal_name}"

  hash_key  = "pkey"
  range_key = "skey"

  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "pkey"
    type = "S"
  }

  attribute {
    name = "skey"
    type = "S"
  }

  attribute {
    name = "aid"
    type = "S"
  }

  attribute {
    name = "seq_nr"
    type = "N"
  }

  global_secondary_index {
    name            = "${var.prefix}-${var.journal_gsi_name}"
    hash_key        = "aid"
    range_key       = "seq_nr"
    projection_type = "ALL"
  }

  stream_enabled   = true
  stream_view_type = "NEW_IMAGE"

  tags = {
    Name  = "${var.prefix}-${var.journal_name}"
    Owner = var.owner
  }

}

resource "aws_dynamodb_table" "snapshot-table" {
  count = var.enabled ? 1 : 0
  name  = "${var.prefix}-${var.snapshot_name}"

  hash_key  = "pkey"
  range_key = "skey"

  billing_mode = "PAY_PER_REQUEST"

  attribute {
    name = "pkey"
    type = "S"
  }

  attribute {
    name = "skey"
    type = "S"
  }

  attribute {
    name = "aid"
    type = "S"
  }

  attribute {
    name = "seq_nr"
    type = "N"
  }

  global_secondary_index {
    name            = "${var.prefix}-${var.snapshot_gsi_name}"
    hash_key        = "aid"
    range_key       = "seq_nr"
    projection_type = "ALL"
  }

  tags = {
    Name  = "${var.prefix}-${var.snapshot_name}"
    Owner = var.owner
  }

}

#module "s3_bucket" {
#  source = "terraform-aws-modules/s3-bucket/aws"
#
#  bucket = "${var.prefix}-${var.snapshot_name}"
#  acl    = "private"
#
#  tags = {
#    Name  = "${var.prefix}-${var.snapshot_name}"
#    Owner = var.owner
#  }
#
#}