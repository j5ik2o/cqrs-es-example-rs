use anyhow::Result;
use command_domain::aggregate::Aggregate;
use std::collections::{HashMap, VecDeque};

use command_domain::group_chat::GroupChatEvent;
use command_domain::group_chat::{GroupChat, GroupChatId};
use command_domain::Event;
use command_interface_adaptor_if::GroupChatRepository;

use crate::gateways::EventPersistenceGateway;

#[derive(Debug, Clone)]
pub struct MockGroupChatRepository {
  events: HashMap<GroupChatId, VecDeque<GroupChatEvent>>,
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
  async fn store(&mut self, event: &GroupChatEvent, _version: usize, snapshot: Option<&GroupChat>) -> Result<()> {
    self
      .events
      .entry(event.aggregate_id().clone())
      .or_insert_with(VecDeque::new)
      .push_back(event.clone());
    *self
      .snapshot
      .entry(event.aggregate_id().clone())
      .or_insert(snapshot.cloned()) = snapshot.cloned();
    Ok(())
  }

  async fn find_by_id(&self, id: &GroupChatId) -> Result<GroupChat> {
    let events = self.events.get(id).unwrap().clone();
    let snapshot = self.snapshot.get(id).unwrap().clone();
    let result = GroupChat::replay(events.into(), snapshot, 0);
    Ok(result)
  }
}

#[derive(Debug, Clone)]
pub struct GroupChatRepositoryImpl<EPG: EventPersistenceGateway> {
  event_persistence_gateway: EPG,
  snapshot_interval: usize,
}

unsafe impl<EPG: EventPersistenceGateway> Sync for GroupChatRepositoryImpl<EPG> {}

unsafe impl<EPG: EventPersistenceGateway> Send for GroupChatRepositoryImpl<EPG> {}

impl<EPG: EventPersistenceGateway> GroupChatRepositoryImpl<EPG> {
  /// コンストラクタ。
  ///
  /// # 引数
  /// - `event_persistence_gateway` - イベント永続化ゲートウェイ
  pub fn new(event_persistence_gateway: EPG, snapshot_interval: usize) -> Self {
    Self {
      event_persistence_gateway,
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
  fn resolve_snapshot(snapshot_interval: usize, created: bool, group_chat: Option<&GroupChat>) -> Option<&GroupChat> {
    match group_chat {
      Some(gc) if created => Some(gc),
      Some(gc) => {
        if gc.seq_nr() % snapshot_interval == 0 {
          Some(gc)
        } else {
          None
        }
      }
      None => None,
    }
  }
}

#[async_trait::async_trait]
impl<EPG: EventPersistenceGateway> GroupChatRepository for GroupChatRepositoryImpl<EPG> {
  async fn store(&mut self, event: &GroupChatEvent, version: usize, snapshot: Option<&GroupChat>) -> Result<()> {
    self
      .event_persistence_gateway
      .store_event_with_snapshot_opt(
        event,
        version,
        Self::resolve_snapshot(self.snapshot_interval, event.is_created(), snapshot),
      )
      .await
  }

  async fn find_by_id(&self, id: &GroupChatId) -> Result<GroupChat> {
    let (snapshot, seq_nr, version) = self
      .event_persistence_gateway
      .get_snapshot_by_id::<GroupChatEvent, GroupChat, GroupChatId>(id)
      .await?;
    let events = self
      .event_persistence_gateway
      .get_events_by_id_and_seq_nr::<GroupChatEvent, GroupChatId>(id, seq_nr)
      .await?;
    let result = GroupChat::replay(events, Some(snapshot), version);
    Ok(result)
  }
}
