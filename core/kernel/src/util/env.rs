use error_stack::{Context, Result, ResultExt};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get environment variables")]
    GetFailed,
    #[error("Failed to parse variable {0:?}")]
    Parse(&'static str),
    #[error("{0:?} is required")]
    NotPresent(&'static str),
}

#[track_caller]
pub fn var(key: &'static str) -> Result<Option<String>, Error> {
    match dotenvy::var(key) {
        Ok(content) => Ok(Some(content)),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(error) => Err(error).change_context(Error::GetFailed),
    }
}

#[track_caller]
pub fn required_var(key: &'static str) -> Result<String, Error> {
    match dotenvy::var(key) {
        Ok(content) => Ok(content),
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => {
            Err(Error::NotPresent(key).into())
        },
        Err(error) => Err(error).change_context(Error::GetFailed),
    }
}

#[track_caller]
pub fn var_parse<T: FromStr>(key: &'static str) -> Result<Option<T>, Error>
where
    T::Err: Context,
{
    match dotenvy::var(key) {
        Ok(content) => {
            Ok(Some(content.parse::<T>().change_context(Error::Parse(key))?))
        },
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => Ok(None),
        Err(error) => Err(error).change_context(Error::GetFailed),
    }
}

#[track_caller]
pub fn required_var_parse<T: FromStr>(key: &'static str) -> Result<T, Error>
where
    T::Err: Context,
{
    match dotenvy::var(key) {
        Ok(content) => {
            Ok(content.parse::<T>().change_context(Error::Parse(key))?)
        },
        Err(dotenvy::Error::EnvVar(std::env::VarError::NotPresent)) => {
            Err(Error::NotPresent(key).into())
        },
        Err(error) => Err(error).change_context(Error::GetFailed),
    }
}
