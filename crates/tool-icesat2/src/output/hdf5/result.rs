use std::error::Error as StdError;

pub(super) type Result<T> = std::result::Result<T, Box<dyn StdError + Send + Sync>>;
