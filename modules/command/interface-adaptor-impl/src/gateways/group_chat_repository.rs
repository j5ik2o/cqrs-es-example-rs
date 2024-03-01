use anyhow::Result;
use event_store_adapter_rs::types::{Aggregate, Event, EventStore};
use std::collections::{HashMap, VecDeque};

use command_domain::group_chat::GroupChatEvent;
use command_domain::group_chat::{GroupChat, GroupChatId};
use command_interface_adaptor_if::GroupChatRepository;

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
  async fn store(&mut self, event: &GroupChatEvent, snapshot: &GroupChat) -> Result<()> {
    self
      .events
      .entry(event.aggregate_id().clone())
      .or_insert_with(VecDeque::new)
      .push_back(event.clone());
    *self
      .snapshot
      .entry(event.aggregate_id().clone())
      .or_insert(Some(snapshot.clone())) = Some(snapshot.clone());
    Ok(())
  }

  async fn find_by_id(&self, id: &GroupChatId) -> Result<Option<GroupChat>> {
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
pub struct GroupChatRepositoryImpl<ES: EventStore<AID = GroupChatId, AG = GroupChat, EV = GroupChatEvent>> {
  event_store: ES,
  snapshot_interval: usize,
}

unsafe impl<ES: EventStore<AID = GroupChatId, AG = GroupChat, EV = GroupChatEvent>> Sync
  for GroupChatRepositoryImpl<ES>
{
}

unsafe impl<ES: EventStore<AID = GroupChatId, AG = GroupChat, EV = GroupChatEvent>> Send
  for GroupChatRepositoryImpl<ES>
{
}

impl<ES: EventStore<AID = GroupChatId, AG = GroupChat, EV = GroupChatEvent>> GroupChatRepositoryImpl<ES> {
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
impl<ES: EventStore<AID = GroupChatId, AG = GroupChat, EV = GroupChatEvent>> GroupChatRepository
  for GroupChatRepositoryImpl<ES>
{
  async fn store(&mut self, event: &GroupChatEvent, snapshot: &GroupChat) -> Result<()> {
    match Self::resolve_snapshot(self.snapshot_interval, event.is_created(), snapshot) {
      Some(snapshot) => self.event_store.persist_event_and_snapshot(event, snapshot).await?,
      None => self.event_store.persist_event(event, snapshot.version()).await?,
    }
    Ok(())
  }

  async fn find_by_id(&self, id: &GroupChatId) -> Result<Option<GroupChat>> {
    let snapshot_opt = self.event_store.get_latest_snapshot_by_id(id).await?;
    match snapshot_opt {
      None => Ok(None),
      Some(snapshot) => {
        let events = self
          .event_store
          .get_events_by_id_since_seq_nr(id, snapshot.seq_nr())
          .await?;
        let result = GroupChat::replay(events, snapshot.clone());
        Ok(Some(result))
      }
    }
  }
}
