use command_interface_adaptor_if::GroupChatReadModelUpdateDaoError;

use crate::journal::JournalReadError;
use crate::UpdateReadModelError;

/// The reason a record could not be applied. It is stored with a recovery task
/// so an operator can tell a data problem from a storage outage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
  EventPayload,
  UnsupportedEvent,
  JournalGap,
  ReadModelConstraint,
  Unknown,
}

impl FailureKind {
  pub fn as_str(&self) -> &'static str {
    match self {
      FailureKind::EventPayload => "event_payload",
      FailureKind::UnsupportedEvent => "unsupported_event",
      FailureKind::JournalGap => "journal_gap",
      FailureKind::ReadModelConstraint => "read_model_constraint",
      FailureKind::Unknown => "unknown",
    }
  }

  pub fn from_name(value: &str) -> Self {
    match value {
      "event_payload" => FailureKind::EventPayload,
      "unsupported_event" => FailureKind::UnsupportedEvent,
      "journal_gap" => FailureKind::JournalGap,
      "read_model_constraint" => FailureKind::ReadModelConstraint,
      _ => FailureKind::Unknown,
    }
  }
}

/// Returns the failure kind when retrying the same event cannot fix it.
///
/// A `None` return means the caller may retry the operation. The distinction
/// decides whether a recovery task is written or the storage is retried.
pub fn classify_update_error(error: &UpdateReadModelError) -> Option<FailureKind> {
  match error {
    UpdateReadModelError::UnexpectedEventName(_)
    | UpdateReadModelError::MissingNewImage
    | UpdateReadModelError::MissingAttribute(_)
    | UpdateReadModelError::UnexpectedAttributeType(_)
    | UpdateReadModelError::InvalidNumber { .. }
    | UpdateReadModelError::InvalidOccurredAt(_)
    | UpdateReadModelError::InvalidAggregateId { .. }
    | UpdateReadModelError::Payload(_) => Some(FailureKind::EventPayload),
    UpdateReadModelError::JournalRead(error) => match error {
      JournalReadError::Query(_) => None,
      JournalReadError::MissingAttribute(_)
      | JournalReadError::UnexpectedAttributeType(_)
      | JournalReadError::InvalidNumber { .. }
      | JournalReadError::InvalidOccurredAt(_)
      | JournalReadError::Payload(_) => Some(FailureKind::JournalGap),
    },
    UpdateReadModelError::JournalGap { .. } => Some(FailureKind::JournalGap),
    UpdateReadModelError::ReadModelUpdate(error) => classify_dao_error(error),
    UpdateReadModelError::RecoveryTaskStore(_) | UpdateReadModelError::RecoveryRecord { .. } => None,
    UpdateReadModelError::Recovery(_)
    | UpdateReadModelError::HeadScan(_)
    | UpdateReadModelError::PositionStalled { .. } => None,
    UpdateReadModelError::RetryExhausted { source, .. } => classify_update_error(source),
  }
}

/// A retryable failure is one for which the same operation may succeed later.
pub fn is_retryable_update_error(error: &UpdateReadModelError) -> bool {
  classify_update_error(error).is_none()
}

pub fn classify_dao_error(error: &GroupChatReadModelUpdateDaoError) -> Option<FailureKind> {
  match error {
    GroupChatReadModelUpdateDaoError::Transient(_) => None,
    GroupChatReadModelUpdateDaoError::Permanent(_) => Some(FailureKind::ReadModelConstraint),
    GroupChatReadModelUpdateDaoError::AcquireConnectionError
    | GroupChatReadModelUpdateDaoError::BeginTransactionError
    | GroupChatReadModelUpdateDaoError::CommitTransactionError
    | GroupChatReadModelUpdateDaoError::ReadPositionError
    | GroupChatReadModelUpdateDaoError::UpdatePositionError => None,
    GroupChatReadModelUpdateDaoError::UnexpectedSeqNr { .. } => Some(FailureKind::JournalGap),
    GroupChatReadModelUpdateDaoError::InsertGroupChatError
    | GroupChatReadModelUpdateDaoError::DeleteGroupChatError
    | GroupChatReadModelUpdateDaoError::RenameGroupChatError
    | GroupChatReadModelUpdateDaoError::InsertMemberError
    | GroupChatReadModelUpdateDaoError::DeleteMemberError
    | GroupChatReadModelUpdateDaoError::InsertMessageError
    | GroupChatReadModelUpdateDaoError::UpdateMessageError
    | GroupChatReadModelUpdateDaoError::DeleteMessageError => Some(FailureKind::ReadModelConstraint),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_classify_payload_and_gap_failures_as_permanent() {
    assert_eq!(
      Some(FailureKind::EventPayload),
      classify_update_error(&UpdateReadModelError::Payload("bad json".to_string()))
    );
    assert_eq!(
      Some(FailureKind::JournalGap),
      classify_update_error(&UpdateReadModelError::JournalGap {
        aggregate_id: "GroupChat-1".to_string(),
        from_seq_nr: 1,
        to_seq_nr: 2,
      })
    );
  }

  #[test]
  fn should_keep_connection_failures_retryable() {
    assert!(is_retryable_update_error(&UpdateReadModelError::ReadModelUpdate(
      GroupChatReadModelUpdateDaoError::AcquireConnectionError
    )));
    assert!(!is_retryable_update_error(&UpdateReadModelError::ReadModelUpdate(
      GroupChatReadModelUpdateDaoError::Permanent("constraint".to_string())
    )));
  }

  #[test]
  fn should_treat_a_lost_recovery_task_as_retryable() {
    let error = UpdateReadModelError::RecoveryRecord {
      apply: "constraint".to_string(),
      record: "table unavailable".to_string(),
    };
    assert!(is_retryable_update_error(&error));
  }
}
