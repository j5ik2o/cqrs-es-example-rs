pub trait Aggregate {
  type ID: std::fmt::Display;
  fn id(&self) -> &Self::ID;
  fn version(&self) -> usize;
}
