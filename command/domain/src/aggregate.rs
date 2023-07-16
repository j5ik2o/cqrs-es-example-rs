pub trait AggregateId: std::fmt::Display {
  fn type_name(&self) -> String;
  fn value(&self) -> String;
}

pub trait Aggregate: std::fmt::Debug {
  type ID: AggregateId;
  fn id(&self) -> &Self::ID;
  fn seq_nr(&self) -> usize;
  fn version(&self) -> usize;
  fn set_version(&mut self, version: usize);
}
