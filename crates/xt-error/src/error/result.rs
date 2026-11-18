use crate::error::context::Error;
use std::error::Error as StdError;

pub type Result<T> = std::result::Result<T, Error>;

pub(super) type Source = Box<dyn StdError + Send + Sync + 'static>;
