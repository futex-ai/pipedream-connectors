//! Shared helpers for lossless property objects.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::UnknownFields;

pub(super) fn insert_extra(object: &mut Map<String, Value>, extra: &UnknownFields) {
    for (key, value) in extra {
        object.entry(key.clone()).or_insert_with(|| value.clone());
    }
}

pub(super) fn encode<T: Serialize, E: serde::ser::Error>(value: &T) -> Result<Value, E> {
    match serde_json::to_value(value) {
        Ok(value) => Ok(value),
        Err(error) => Err(E::custom(error)),
    }
}

pub(super) fn decode<T: DeserializeOwned, E: serde::de::Error>(value: Value) -> Result<T, E> {
    match serde_json::from_value(value) {
        Ok(value) => Ok(value),
        Err(error) => Err(E::custom(error)),
    }
}
