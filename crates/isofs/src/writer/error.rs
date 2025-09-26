#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Serialization error: {0}")]
  Serialize(#[from] crate::serialize::IsoSerializeError),
  #[error("I/O error: {0}")]
  Io(#[from] std::io::Error),
}
