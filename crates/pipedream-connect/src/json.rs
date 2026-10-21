//! JSON conversion that retains only safe failure metadata.

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result, WireFailure};

#[track_caller]
pub(crate) fn decode<T: DeserializeOwned>(body: &[u8]) -> Result<T> {
    match serde_json::from_slice(body) {
        Ok(value) => Ok(value),
        Err(source) => Err(Error::internal(WireFailure::Decode {
            line: source.line(),
            column: source.column(),
        })),
    }
}

#[track_caller]
pub(crate) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    match serde_json::to_vec(value) {
        Ok(body) => Ok(body),
        Err(_) => Err(Error::internal(WireFailure::Encode)),
    }
}
