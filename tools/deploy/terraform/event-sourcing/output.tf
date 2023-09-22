output "aws_dynamodb_table_journal_table_name" {
  value = concat(aws_dynamodb_table.journal-table.*.name, [""])[0]
}

output "aws_dynamodb_table_journal_stream_arn" {
  value = concat(aws_dynamodb_table.journal-table.*.stream_arn, [""])[0]
}

output "aws_dynamodb_table_journal_gsi_name" {
  value = "${var.prefix}-${var.journal_gsi_name}"
}

output "aws_dynamodb_table_snapshot_table_name" {
  value = concat(aws_dynamodb_table.snapshot-table.*.name, [""])[0]
}

output "aws_dynamodb_table_snapshot_gsi_name" {
  value = "${var.prefix}-${var.snapshot_gsi_name}"
}
