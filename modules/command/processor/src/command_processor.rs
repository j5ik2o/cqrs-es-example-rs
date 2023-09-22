use anyhow::Result;
use event_store_adapter_rs::types::Aggregate;
use command_domain::group_chat::*;
use command_domain::user_account::UserAccountId;
use command_interface_adaptor_if::{GroupChatPresenter, GroupChatRepository};

/// グループチャットへのコマンドを処理するユースケース実装。
///
/// NOTE: コマンドを処理するユースケースをコマンドプロセッサと呼びます(クエリを処理するユースケースはクエリプロセッサとなりますが、今回はGraphQLを採用しているためクエリプロッサは定義されていません)
pub struct GroupChatCommandProcessor<'a, TR: GroupChatRepository> {
  group_chat_repository: &'a mut TR,
}

impl<'a, TR: GroupChatRepository> GroupChatCommandProcessor<'a, TR> {
  /// コンストラクタ。
  ///
  /// # 引数
  /// - `group_chat_repository` - グループチャットリポジトリ
  pub fn new(group_chat_repository: &'a mut TR) -> Self {
    Self { group_chat_repository }
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
  pub async fn create_group_chat<P: GroupChatPresenter>(
    &mut self,
    group_chat_presenter: &mut P,
    name: GroupChatName,
    executor_id: UserAccountId,
  ) -> Result<()> {
    let members = Members::new(executor_id);
    let (t, te) = GroupChat::new(name, members);
    self.group_chat_repository.store(&te, 1, Some(&t)).await?;
    group_chat_presenter.present(te);
    Ok(())
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
  pub async fn rename_group_chat<P: GroupChatPresenter>(
    &mut self,
    group_chat_presenter: &mut P,
    id: GroupChatId,
    name: GroupChatName,
    executor_id: UserAccountId,
  ) -> Result<()> {
    let mut group_chat = self.group_chat_repository.find_by_id(&id).await?;
    let event = group_chat.rename(name, executor_id)?;
    self
      .group_chat_repository
      .store(&event, group_chat.version(), Some(&group_chat))
      .await?;
    group_chat_presenter.present(event);
    Ok(())
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
  pub async fn add_member<P: GroupChatPresenter>(
    &mut self,
    group_chat_presenter: &mut P,
    id: GroupChatId,
    user_account_id: UserAccountId,
    role: MemberRole,
    executor_id: UserAccountId,
  ) -> Result<()> {
    let mut group_chat = self.group_chat_repository.find_by_id(&id).await?;
    let member_id = MemberId::new();
    let event = group_chat.add_member(member_id, user_account_id, role, executor_id)?;
    self
      .group_chat_repository
      .store(&event, group_chat.version(), Some(&group_chat))
      .await?;
    group_chat_presenter.present(event);
    Ok(())
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
  pub async fn remove_member<P: GroupChatPresenter>(
    &mut self,
    group_chat_presenter: &mut P,
    id: GroupChatId,
    user_account_id: UserAccountId,
    executor_id: UserAccountId,
  ) -> Result<()> {
    let mut group_chat = self.group_chat_repository.find_by_id(&id).await?;
    let event = group_chat.remove_member(user_account_id, executor_id)?;
    self
      .group_chat_repository
      .store(&event, group_chat.version(), Some(&group_chat))
      .await?;
    group_chat_presenter.present(event);
    Ok(())
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
  pub async fn delete_group_chat<P: GroupChatPresenter>(
    &mut self,
    group_chat_presenter: &mut P,
    id: GroupChatId,
    executor_id: UserAccountId,
  ) -> Result<()> {
    let mut group_chat = self.group_chat_repository.find_by_id(&id).await?;
    let event = group_chat.delete(executor_id)?;
    self
      .group_chat_repository
      .store(&event, group_chat.version(), Some(&group_chat))
      .await?;
    group_chat_presenter.present(event);
    Ok(())
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
  pub async fn post_message<P: GroupChatPresenter>(
    &mut self,
    _group_chat_presenter: &mut P,
    _id: GroupChatId,
    _message: Message,
    _executor_id: UserAccountId,
  ) -> Result<()> {
    todo!() // 必須課題 難易度:中
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
  pub async fn delete_message<P: GroupChatPresenter>(
    &mut self,
    group_chat_presenter: &mut P,
    id: GroupChatId,
    message_id: MessageId,
    executor_id: UserAccountId,
  ) -> Result<()> {
    let mut group_chat = self.group_chat_repository.find_by_id(&id).await?;
    let event = group_chat.delete_message(message_id, executor_id)?;
    self
      .group_chat_repository
      .store(&event, group_chat.version(), Some(&group_chat))
      .await?;
    group_chat_presenter.present(event);
    Ok(())
  }
}
