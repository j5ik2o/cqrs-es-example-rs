use chrono::Utc;
use event_store_adapter_rs::{EventEnvelope, EventStore, SeqNr, SnapshotEnvelope, SEQ_NR_MAX};
use std::collections::{HashMap, VecDeque};

use command_domain::group_chat::GroupChatEvent;
use command_domain::group_chat::{EventRecord, GroupChat, GroupChatId};
use command_interface_adaptor_if::{GroupChatRepository, GroupChatRepositoryError};

/// スナップショット envelope に載せる manifest。
pub const SNAPSHOT_MANIFEST: &str = "GroupChat@1";

#[derive(Debug, Clone)]
pub struct MockGroupChatRepository {
  events: HashMap<GroupChatId, VecDeque<EventRecord>>,
  snapshot: HashMap<GroupChatId, Option<GroupChat>>,
}

impl MockGroupChatRepository {
  pub fn new() -> Self {
    Self {
      events: HashMap::new(),
      snapshot: HashMap::new(),
    }
  }
}

#[async_trait::async_trait]
impl GroupChatRepository for MockGroupChatRepository {
  async fn store(&mut self, event: &GroupChatEvent, snapshot: &GroupChat) -> Result<(), GroupChatRepositoryError> {
    self
      .events
      .entry(snapshot.id().clone())
      .or_insert_with(VecDeque::new)
      .push_back(EventRecord::new(
        event.clone(),
        snapshot.seq_nr(),
        *snapshot.last_updated_at(),
      ));
    *self
      .snapshot
      .entry(snapshot.id().clone())
      .or_insert(Some(snapshot.clone())) = Some(snapshot.clone());
    Ok(())
  }

  async fn find_by_id(&self, id: &GroupChatId) -> Result<Option<GroupChat>, GroupChatRepositoryError> {
    let events = self.events.get(id).unwrap().clone();
    let snapshot_opt = self.snapshot.get(id).unwrap().clone();
    if let Some(snapshot) = snapshot_opt {
      let result = GroupChat::replay(events.into(), snapshot);
      Ok(Some(result))
    } else {
      Ok(None)
    }
  }
}

#[derive(Debug, Clone)]
pub struct GroupChatRepositoryImpl<ES: EventStore<AID = GroupChatId, A = GroupChat, P = GroupChatEvent>> {
  event_store: ES,
  snapshot_interval: usize,
}

unsafe impl<ES: EventStore<AID = GroupChatId, A = GroupChat, P = GroupChatEvent>> Sync for GroupChatRepositoryImpl<ES> {}

unsafe impl<ES: EventStore<AID = GroupChatId, A = GroupChat, P = GroupChatEvent>> Send for GroupChatRepositoryImpl<ES> {}

impl<ES: EventStore<AID = GroupChatId, A = GroupChat, P = GroupChatEvent>> GroupChatRepositoryImpl<ES> {
  /// コンストラクタ。
  ///
  /// # 引数
  /// - `event_persistence_gateway` - イベント永続化ゲートウェイ
  pub fn new(event_store: ES, snapshot_interval: usize) -> Self {
    Self {
      event_store,
      snapshot_interval,
    }
  }

  /// スナップショットを永続化するかどうかを判定する。
  ///
  /// # 引数
  /// - `snapshot_interval` - スナップショットを永続化する間隔
  /// - `created` - グループチャットが作成されたかどうか
  /// - `group_chat` - グループチャット
  ///
  /// # 戻り値
  /// スナップショットを永続化する場合は `Some` 、そうでない場合は `None` 。
  fn resolve_snapshot(snapshot_interval: usize, created: bool, group_chat: &GroupChat) -> Option<&GroupChat> {
    if created || group_chat.seq_nr() % snapshot_interval == 0 {
      Some(group_chat)
    } else {
      None
    }
  }
}

#[async_trait::async_trait]
impl<ES: EventStore<AID = GroupChatId, A = GroupChat, P = GroupChatEvent>> GroupChatRepository
  for GroupChatRepositoryImpl<ES>
{
  async fn store(&mut self, event: &GroupChatEvent, snapshot: &GroupChat) -> Result<(), GroupChatRepositoryError> {
    // v5 の envelope はメタデータを payload から分離する。集約 ID と seq_nr は
    // イベントを適用した後の集約状態から取り、occurred_at は書き込み時に決める。
    let event_envelope = EventEnvelope::new(
      snapshot.id().clone(),
      snapshot.seq_nr() as SeqNr,
      Utc::now(),
      event.clone(),
    )
    .with_manifest(event.manifest());
    let result = match Self::resolve_snapshot(self.snapshot_interval, event.is_created(), snapshot) {
      Some(snapshot) => {
        let snapshot_envelope =
          SnapshotEnvelope::new(snapshot.clone(), snapshot.seq_nr() as SeqNr).with_manifest(SNAPSHOT_MANIFEST);
        self
          .event_store
          .persist_event_and_snapshot(event_envelope, snapshot_envelope)
          .await
      }
      None => self.event_store.persist_event(event_envelope).await,
    };
    match result {
      Ok(_) => Ok(()),
      Err(error) => Err(GroupChatRepositoryError::StoreError(snapshot.clone(), error)),
    }
  }

  async fn find_by_id(&self, id: &GroupChatId) -> Result<Option<GroupChat>, GroupChatRepositoryError> {
    let snapshot_opt = self.event_store.get_latest_snapshot_by_id(id).await;
    match snapshot_opt {
      Ok(None) => Ok(None),
      Ok(Some(snapshot_read)) => {
        let (snapshot_envelope_opt, _head_seq_nr) = snapshot_read.into_parts();
        match snapshot_envelope_opt {
          None => {
            // スナップショットがなくてもヘッドがあれば、作成イベントから復元する。
            let events = self.event_store.get_events_by_id_since_seq_nr(id, 1).await;
            match events {
              Ok(events) => {
                let records = into_event_records(events);
                Ok(GroupChat::replay_from_events(id.clone(), records))
              }
              Err(error) => Err(GroupChatRepositoryError::FindByIdError(id.clone(), error)),
            }
          }
          Some(snapshot_envelope) => {
            // get_events_by_id_since_seq_nr は下限を含む。スナップショット自身のイベントを
            // 再適用しないよう、次の seq_nr から読む。上限を超える seq_nr は渡せない。
            let events = match snapshot_envelope.seq_nr().checked_add(1) {
              Some(start) if start <= SEQ_NR_MAX => self.event_store.get_events_by_id_since_seq_nr(id, start).await,
              _ => Ok(Vec::new()),
            };
            match events {
              Ok(events) => {
                let records = into_event_records(events);
                let result = GroupChat::replay(records, snapshot_envelope.into_aggregate());
                Ok(Some(result))
              }
              Err(error) => Err(GroupChatRepositoryError::FindByIdError(id.clone(), error)),
            }
          }
        }
      }
      Err(error) => Err(GroupChatRepositoryError::FindByIdError(id.clone(), error)),
    }
  }
}

fn into_event_records(events: Vec<EventEnvelope<GroupChatId, GroupChatEvent>>) -> Vec<EventRecord> {
  events
    .into_iter()
    .map(|envelope| {
      let seq_nr = envelope.seq_nr() as usize;
      let occurred_at = *envelope.occurred_at();
      EventRecord::new(envelope.into_payload(), seq_nr, occurred_at)
    })
    .collect()
}
