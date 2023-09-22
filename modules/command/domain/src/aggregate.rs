/// 集約のIDを表すトレイト。
pub trait AggregateId: std::fmt::Display + std::fmt::Debug + Send + Sync {
  /// 集約の種別名を返す。
  fn type_name(&self) -> String;
  /// 集約のIDを文字列として返す
  fn value(&self) -> String;
}

/// 集約を表すトレイト。
pub trait Aggregate: std::fmt::Debug + Send + Sync {
  type ID: AggregateId;
  /// IDを返す。
  fn id(&self) -> &Self::ID;
  /// シーケンス番号を返す。
  fn seq_nr(&self) -> usize;
  /// バージョンを返す。
  fn version(&self) -> usize;
  /// シーケンス番号を設定する。
  fn set_version(&mut self, version: usize);
}
