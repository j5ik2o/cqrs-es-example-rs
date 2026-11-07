use std::sync::Arc;

use command_domain::group_chat::GroupChatId;
use command_interface_adaptor_if::{GroupChatReadModelUpdateDao, ReadModelEvent};

use crate::journal::JournalReader;
use crate::UpdateReadModelError;

/// Applies events with the one position rule: only the event at position + 1 is
/// applied, and the read model change and the position advance commit together.
/// The stream path, resync, and recovery all use this type.
pub struct Applier {
  dao: Arc<dyn GroupChatReadModelUpdateDao>,
  journal: Arc<dyn JournalReader>,
  max_events_per_transaction: usize,
  read_ahead_events: usize,
}

impl Applier {
  pub fn new(
    dao: Arc<dyn GroupChatReadModelUpdateDao>,
    journal: Arc<dyn JournalReader>,
    max_events_per_transaction: usize,
  ) -> Self {
    Self {
      dao,
      journal,
      max_events_per_transaction: max_events_per_transaction.max(1),
      read_ahead_events: 100,
    }
  }

  pub async fn position(&self, aggregate_id: &GroupChatId) -> Result<u64, UpdateReadModelError> {
    self
      .dao
      .get_position(aggregate_id)
      .await
      .map_err(UpdateReadModelError::ReadModelUpdate)
  }

  /// Applies a head record. The journal fills a gap below the record's seq_nr;
  /// the record itself carries the newest event.
  pub async fn apply_head_event(
    &self,
    aggregate_id: &GroupChatId,
    event: ReadModelEvent,
  ) -> Result<(), UpdateReadModelError> {
    apply_head_event_with(
      self.dao.as_ref(),
      self.journal.as_ref(),
      self.max_events_per_transaction,
      aggregate_id,
      event,
    )
    .await
  }

  /// Replays the journal from the current position up to `target_seq_nr`. It is
  /// the shared body of resync and recovery and never uses the head payload, so
  /// a repaired journal entry is picked up on the next run.
  pub async fn apply_through(
    &self,
    aggregate_id: &GroupChatId,
    target_seq_nr: u64,
  ) -> Result<(), UpdateReadModelError> {
    apply_through_with(
      self.dao.as_ref(),
      self.journal.as_ref(),
      self.max_events_per_transaction,
      self.read_ahead_events,
      aggregate_id,
      target_seq_nr,
    )
    .await
  }
}

/// Applies a head event using borrowed DAO and journal implementations. The
/// free functions keep the existing `process_records` test seam while the
/// `Applier` owns the dependencies in production.
pub(crate) async fn apply_head_event_with(
  dao: &dyn GroupChatReadModelUpdateDao,
  journal: &dyn JournalReader,
  max_events_per_transaction: usize,
  aggregate_id: &GroupChatId,
  event: ReadModelEvent,
) -> Result<(), UpdateReadModelError> {
  let position = dao.get_position(aggregate_id).await?;
  if event.seq_nr() <= position {
    return Ok(());
  }
  let mut pending = Vec::new();
  if event.seq_nr() > position + 1 {
    let from_seq_nr = position + 1;
    let to_seq_nr = event.seq_nr() - 1;
    let journal_events = journal.read_events(aggregate_id, from_seq_nr, to_seq_nr).await?;
    require_contiguous(aggregate_id, from_seq_nr, to_seq_nr, &journal_events)?;
    pending.extend(journal_events);
  }
  pending.push(event);
  apply_events_with(dao, max_events_per_transaction, aggregate_id, &pending).await
}

pub(crate) async fn apply_through_with(
  dao: &dyn GroupChatReadModelUpdateDao,
  journal: &dyn JournalReader,
  max_events_per_transaction: usize,
  read_ahead_events: usize,
  aggregate_id: &GroupChatId,
  target_seq_nr: u64,
) -> Result<(), UpdateReadModelError> {
  if target_seq_nr == 0 {
    return Ok(());
  }
  let read_ahead_events = read_ahead_events.max(1) as u64;
  loop {
    let current_seq_nr = dao.get_position(aggregate_id).await?;
    if current_seq_nr >= target_seq_nr {
      return Ok(());
    }
    let end_seq_nr = (current_seq_nr + read_ahead_events).min(target_seq_nr);
    let events = journal
      .read_events(aggregate_id, current_seq_nr + 1, end_seq_nr)
      .await?;
    require_contiguous(aggregate_id, current_seq_nr + 1, end_seq_nr, &events)?;
    apply_events_with(dao, max_events_per_transaction, aggregate_id, &events).await?;
    let after_seq_nr = dao.get_position(aggregate_id).await?;
    if after_seq_nr <= current_seq_nr {
      return Err(UpdateReadModelError::PositionStalled {
        aggregate_id: aggregate_id.to_string(),
        position: after_seq_nr,
        target_seq_nr,
      });
    }
  }
}

async fn apply_events_with(
  dao: &dyn GroupChatReadModelUpdateDao,
  max_events_per_transaction: usize,
  aggregate_id: &GroupChatId,
  events: &[ReadModelEvent],
) -> Result<(), UpdateReadModelError> {
  let chunk_size = max_events_per_transaction.max(1);
  for chunk in events.chunks(chunk_size) {
    dao.apply_events(aggregate_id, chunk).await?;
  }
  Ok(())
}

fn require_contiguous(
  aggregate_id: &GroupChatId,
  from_seq_nr: u64,
  to_seq_nr: u64,
  events: &[ReadModelEvent],
) -> Result<(), UpdateReadModelError> {
  let mut expected = from_seq_nr;
  for event in events {
    if event.seq_nr() != expected {
      return Err(UpdateReadModelError::JournalGap {
        aggregate_id: aggregate_id.to_string(),
        from_seq_nr,
        to_seq_nr,
      });
    }
    expected += 1;
  }
  if expected != to_seq_nr + 1 {
    return Err(UpdateReadModelError::JournalGap {
      aggregate_id: aggregate_id.to_string(),
      from_seq_nr,
      to_seq_nr,
    });
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::journal::JournalReadError;

  #[test]
  fn should_report_a_missing_journal_range_as_a_gap() {
    let aggregate_id = GroupChatId::new();
    let error = require_contiguous(&aggregate_id, 1, 3, &[]).unwrap_err();
    assert!(matches!(
      error,
      UpdateReadModelError::JournalGap {
        from_seq_nr: 1,
        to_seq_nr: 3,
        ..
      }
    ));
  }

  #[test]
  fn should_keep_journal_query_failures_retryable() {
    let error = UpdateReadModelError::JournalRead(JournalReadError::Query("throttled".to_string()));
    assert!(crate::failure::is_retryable_update_error(&error));
  }
}
