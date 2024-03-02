use anyhow::Result;
use event_store_adapter_rs::types::Event;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

use command_domain::group_chat::*;
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::GroupChatRepository;

#[derive(Error, Debug)]
pub enum CommandProcessError {
  #[error("GroupChat not found.")]
  NotFoundError,
}

/// グループチャットへのコマンドを処理するユースケース実装。
///
/// NOTE: コマンドを処理するユースケースをコマンドプロセッサと呼びます(クエリを処理するユースケースはクエリプロセッサとなりますが、今回はGraphQLを採用しているためクエリプロッサは定義されていません)
pub struct GroupChatCommandProcessor<TR: GroupChatRepository> {
  group_chat_repository: Arc<Mutex<TR>>,
}

impl<TR: GroupChatRepository> GroupChatCommandProcessor<TR> {
  /// コンストラクタ。
  ///
  /// # 引数
  /// - `group_chat_repository` - グループチャットリポジトリ
  pub fn new(group_chat_repository: TR) -> Self {
    Self {
      group_chat_repository: Arc::new(Mutex::new(group_chat_repository)),
    }
  }

  /// グループチャットを作成する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `name` - グループチャット名
  /// - `executor_id` - 実行者のユーザーアカウントID
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn create_group_chat(&mut self, name: GroupChatName, executor_id: UserAccountId) -> Result<GroupChatId> {
    let mut repository_mg = self.group_chat_repository.lock().await;

    let members = Members::new(executor_id);
    let (group_chat, group_chat_event) = GroupChat::new(name, members);

    repository_mg.store(&group_chat_event, &group_chat).await?;

    Ok(group_chat_event.aggregate_id().clone())
  }

  /// グループチャットの名前を変更する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `id` - グループチャットID
  /// - `name` - グループチャット名
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn rename_group_chat(
    &mut self,
    id: GroupChatId,
    name: GroupChatName,
    executor_id: UserAccountId,
  ) -> Result<GroupChatId> {
    let mut rg = self.group_chat_repository.lock().await;

    let mut group_chat = rg.find_by_id(&id).await?.ok_or(CommandProcessError::NotFoundError)?;

    let group_chat_event = group_chat.rename(name, executor_id)?;
    rg.store(&group_chat_event, &group_chat).await?;

    Ok(group_chat_event.aggregate_id().clone())
  }

  /// グループチャットにメンバーを追加する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `id` - グループチャットID
  /// - `user_account_id` - ユーザーアカウントID
  /// - `role` - メンバーの役割
  /// - `executor_id` - 実行者のユーザーアカウントID
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn add_member(
    &mut self,
    id: GroupChatId,
    user_account_id: UserAccountId,
    role: MemberRole,
    executor_id: UserAccountId,
  ) -> Result<GroupChatId> {
    let mut repository_mg = self.group_chat_repository.lock().await;

    let mut group_chat = repository_mg
      .find_by_id(&id)
      .await?
      .ok_or(CommandProcessError::NotFoundError)?;

    let member_id = MemberId::new();
    let group_chat_event = group_chat.add_member(member_id, user_account_id, role, executor_id)?;

    repository_mg.store(&group_chat_event, &group_chat).await?;

    Ok(group_chat_event.aggregate_id().clone())
  }

  /// グループチャットからメンバーを削除する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `id` - グループチャットID
  /// - `user_account_id` - ユーザーアカウントID
  /// - `executor_id` - 実行者のユーザーアカウントID
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn remove_member(
    &mut self,
    id: GroupChatId,
    user_account_id: UserAccountId,
    executor_id: UserAccountId,
  ) -> Result<GroupChatId> {
    let mut repository_mg = self.group_chat_repository.lock().await;

    let mut group_chat = repository_mg
      .find_by_id(&id)
      .await?
      .ok_or(CommandProcessError::NotFoundError)?;

    let group_chat_event = group_chat.remove_member(user_account_id, executor_id)?;

    repository_mg.store(&group_chat_event, &group_chat).await?;

    Ok(group_chat_event.aggregate_id().clone())
  }

  /// グループチャットを削除する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `id` - グループチャットID
  /// - `executor_id` - 実行者のユーザーアカウントID
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn delete_group_chat(&mut self, id: GroupChatId, executor_id: UserAccountId) -> Result<GroupChatId> {
    let mut repository_mg = self.group_chat_repository.lock().await;

    let mut group_chat = repository_mg
      .find_by_id(&id)
      .await?
      .ok_or(CommandProcessError::NotFoundError)?;

    let group_chat_event = group_chat.delete(executor_id)?;

    repository_mg.store(&group_chat_event, &group_chat).await?;

    Ok(group_chat_event.aggregate_id().clone())
  }

  /// グループチャットにメッセージを投稿する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `id` - グループチャットID
  /// - `message` - メッセージ
  /// - `executor_id` - 実行者のユーザーアカウントID
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn post_message(
    &mut self,
    id: GroupChatId,
    message: Message,
    executor_id: UserAccountId,
  ) -> Result<(GroupChatId, MessageId)> {
    let mut repository_mg = self.group_chat_repository.lock().await;

    let mut group_chat = repository_mg
      .find_by_id(&id)
      .await?
      .ok_or(CommandProcessError::NotFoundError)?;

    let group_chat_event = group_chat.post_message(message.clone(), executor_id)?;

    repository_mg.store(&group_chat_event, &group_chat).await?;

    Ok((
      group_chat_event.aggregate_id().clone(),
      message.breach_encapsulation_of_id().clone(),
    ))
  }

  /// グループチャットのメッセージを削除する。
  ///
  /// # 引数
  /// - `group_chat_presenter` - グループチャットプレゼンター
  /// - `id` - グループチャットID
  /// - `message_id` - メッセージID
  /// - `executor_id` - 実行者のユーザーアカウントID
  ///
  /// # 戻り値
  /// - 成功した場合はOk, 失敗した場合はErrを返す。
  pub async fn delete_message(
    &mut self,
    id: GroupChatId,
    message_id: MessageId,
    executor_id: UserAccountId,
  ) -> Result<GroupChatId> {
    let mut repository_mg = self.group_chat_repository.lock().await;

    let mut group_chat = repository_mg
      .find_by_id(&id)
      .await?
      .ok_or(CommandProcessError::NotFoundError)?;

    let group_chat_event = group_chat.delete_message(message_id, executor_id)?;

    repository_mg.store(&group_chat_event, &group_chat).await?;

    Ok(group_chat_event.aggregate_id().clone())
  }
}
