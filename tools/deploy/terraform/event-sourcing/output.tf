output "aws_dynamodb_table_journal_table_name" {
  value = concat(aws_dynamodb_table.journal-table.*.name, [""])[0]
}

output "aws_dynamodb_table_snapshot_table_name" {
  value = concat(aws_dynamodb_table.snapshot-table.*.name, [""])[0]
}

output "aws_dynamodb_table_snapshot_history_index_name" {
  value = "${var.prefix}-${var.snapshot_history_index_name}"
}

output "aws_dynamodb_table_head_table_name" {
  value = concat(aws_dynamodb_table.head-table.*.name, [""])[0]
}

output "aws_dynamodb_table_head_stream_arn" {
  value = concat(aws_dynamodb_table.head-table.*.stream_arn, [""])[0]
}
