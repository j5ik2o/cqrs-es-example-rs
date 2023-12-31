use crate::group_chat::message::Message;
use crate::group_chat::message_id::MessageId;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// [GroupChat]内でやりとりする[Message]の集合。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Messages(Vec<Message>);

impl Messages {
  /// コンストラクタ
  pub fn new(values: impl IntoIterator<Item = Message>) -> Self {
    Self(values.into_iter().collect())
  }

  /// [Message]の件数を返す。
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// 指定したインデックスの[Message]への参照を返す。
  ///
  /// # 引数
  /// - `index` - 取得する[Message]のインデックス
  ///
  /// # 戻り値
  /// - 指定したインデックスの[Message]への参照を返す。
  pub fn get_at(&self, index: usize) -> Option<&Message> {
    self.0.get(index)
  }

  /// [Message]のイテレータを返す。
  pub fn iter(&self) -> impl Iterator<Item = &Message> {
    self.0.iter()
  }

  /// 指定した[MessageId]を持つ[Message]が含まれているかどうかを返す。
  ///
  /// # 引数
  /// - `message_id` - 検索する[Message]のID
  ///
  /// # 戻り値
  /// - 指定した[MessageId]を持つ[Message]が含まれている場合は`true`を返す。
  pub fn contains(&self, message_id: &MessageId) -> bool {
    self
      .0
      .iter()
      .any(|message| *message.breach_encapsulation_of_id() == *message_id)
  }

  /// 指定した[MessageId]を持つ[Message]を返す。
  ///
  /// # 引数
  /// - `message_id` - 検索する[Message]のID
  ///
  /// # 戻り値
  /// - 指定した[MessageId]を持つ[Message]が含まれている場合は[Message]への参照を返す。
  pub fn get(&self, message_id: &MessageId) -> Option<&Message> {
    self
      .0
      .iter()
      .find(|message| *message.breach_encapsulation_of_id() == *message_id)
  }

  /// [Message]を追加する。
  ///
  /// # 引数
  /// - `message` - 追加する[Message]
  pub fn add(&mut self, message: Message) {
    self.0.push(message);
  }

  /// 指定した[MessageId]を持つ[Message]を削除する。
  ///
  /// # 引数
  /// - `message_id` - 削除する[Message]のID
  ///
  /// # 戻り値
  /// - 削除に失敗した場合は`Err(anyhow!("Message not found"))`を返す。
  /// - 削除に成功した場合は`Ok(())`を返す。
  pub fn remove(&mut self, message_id: &MessageId) -> Result<()> {
    let index = self
      .0
      .iter()
      .position(|message| *message.breach_encapsulation_of_id() == *message_id)
      .ok_or(anyhow!("Message not found"))?;
    self.0.remove(index);
    Ok(())
  }
}
