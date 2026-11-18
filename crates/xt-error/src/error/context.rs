use crate::error::kind::ErrorKind;
use crate::error::result::Source;
use crate::error::retry::RetryAdvice;
use std::error::Error as StdError;
use std::time::Duration;
use std::{fmt, io};

#[derive(Debug)]
pub struct Error {
    pub(super) kind: ErrorKind,
    pub(super) message: String,
    pub(super) contexts: Vec<String>,
    pub(super) source: Option<Source>,
    pub(super) retry_after: Option<Duration>,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            contexts: Vec::new(),
            source: None,
            retry_after: None,
        }
    }

    pub fn with_source(mut self, source: impl StdError + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.contexts.push(context.into());
        self
    }

    pub fn with_retry_after(mut self, delay: Duration) -> Self {
        self.retry_after = Some(delay);
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    pub fn code(&self) -> &'static str {
        self.kind.code()
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn contexts(&self) -> &[String] {
        &self.contexts
    }

    pub fn retry_advice(&self) -> RetryAdvice {
        match (self.kind.retry_advice(), self.retry_after) {
            (RetryAdvice::Backoff, Some(delay)) => RetryAdvice::After(delay),
            (advice, _) => advice,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] ", self.code())?;
        for context in self.contexts.iter().rev() {
            write!(f, "{context}: ")?;
        }
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn StdError + 'static))
    }
}

impl From<io::Error> for Error {
    fn from(source: io::Error) -> Self {
        Self::new(ErrorKind::Io(source.kind()), source.to_string()).with_source(source)
    }
}
