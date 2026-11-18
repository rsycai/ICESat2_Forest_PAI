use crate::env_load::defaults::{FILE_DEFAULTS, USE_FILE_DEFAULTS};
use serde::de::DeserializeOwned;
use std::ffi::OsString;
use std::str::FromStr;
use xt_error::{Error, ErrorKind, Result};

pub(super) fn first(keys: &[&str]) -> Option<(String, OsString)> {
    keys.iter()
        .find_map(|key| std::env::var_os(key).map(|value| ((*key).to_owned(), value)))
        .or_else(|| {
            if !USE_FILE_DEFAULTS.get() {
                return None;
            }
            let defaults = FILE_DEFAULTS.read().ok()?;
            keys.iter().find_map(|key| {
                defaults
                    .get(*key)
                    .map(|value| ((*key).to_owned(), OsString::from(value)))
            })
        })
}

#[doc(hidden)]
pub fn read_os(keys: &[&str]) -> Result<Option<OsString>> {
    Ok(first(keys).map(|(_, value)| value))
}

pub(super) fn text(keys: &[&str]) -> Result<Option<(String, String)>> {
    first(keys)
        .map(|(key, value)| {
            value
                .into_string()
                .map(|value| (key.clone(), value))
                .map_err(|_| {
                    Error::new(
                        ErrorKind::Configuration,
                        format!("Environment variable {key} must be valid Unicode text"),
                    )
                })
        })
        .transpose()
}

#[doc(hidden)]
pub fn read<T: FromStr>(keys: &[&str]) -> Result<Option<T>> {
    text(keys)?
        .map(|(key, value)| {
            value.parse().map_err(|_| {
                Error::new(
                    ErrorKind::Configuration,
                    format!("Cannot parse environment variable {key} as {}", std::any::type_name::<T>()),
                )
            })
        })
        .transpose()
}

#[doc(hidden)]
pub fn read_json<T: DeserializeOwned>(keys: &[&str]) -> Result<Option<T>> {
    text(keys)?
        .map(|(key, value)| {
            serde_json::from_str(&value).map_err(|_| {
                Error::new(
                    ErrorKind::Configuration,
                    format!("Environment variable {key} is not valid JSON for the field type"),
                )
            })
        })
        .transpose()
}
