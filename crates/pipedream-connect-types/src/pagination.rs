//! Bounded cursor pagination models and forward or backward traversal.

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{Error, Result, ValidationField};
use crate::{Cursor, UnknownFields};

/// A page request with one bounded cursor direction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PageRequest {
    /// Maximum number of records to request.
    pub limit: u16,
    /// Cursor after which records are requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Cursor>,
    /// Cursor before which records are requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Cursor>,
}

impl PageRequest {
    /// Creates a request with the default limit of 100.
    pub fn new() -> Self {
        Self {
            limit: 100,
            after: None,
            before: None,
        }
    }

    /// Creates a request with a validated limit.
    pub fn with_limit(limit: u16) -> Result<Self> {
        let request = Self {
            limit,
            ..Self::new()
        };
        request.validate().map(|()| request)
    }

    /// Validates the limit and cursor direction.
    pub fn validate(&self) -> Result<()> {
        if !(1..=200).contains(&self.limit) {
            return Err(Error::OutOfRange {
                field: ValidationField::Limit,
                minimum: 1,
                maximum: 200,
            });
        }
        if self.after.is_some() && self.before.is_some() {
            return Err(Error::ConflictingCursors);
        }
        Ok(())
    }

    /// Returns the next request and rejects a provider cursor that does not advance.
    pub fn next(&self, page: &PageInfo) -> Result<Option<Self>> {
        self.validate()?;
        if page.count == Some(0) {
            return Ok(None);
        }
        let (cursor, forward) = match (&self.after, &self.before) {
            (Some(_), None) | (None, None) => (page.end_cursor.as_ref(), true),
            (None, Some(_)) => (page.start_cursor.as_ref(), false),
            (Some(_), Some(_)) => return Err(Error::ConflictingCursors),
        };
        let Some(cursor) = cursor else {
            return Ok(None);
        };
        let previous = if forward { &self.after } else { &self.before };
        if previous.as_ref() == Some(cursor) {
            return Err(Error::StuckCursor);
        }
        Ok(Some(Self {
            limit: self.limit,
            after: forward.then(|| cursor.clone()),
            before: (!forward).then(|| cursor.clone()),
        }))
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de> Deserialize<'de> for PageRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default = "default_limit")]
            limit: u16,
            after: Option<Cursor>,
            before: Option<Cursor>,
        }

        let wire = Wire::deserialize(deserializer)?;
        let request = Self {
            limit: wire.limit,
            after: wire.after,
            before: wire.before,
        };
        match request.validate() {
            Ok(()) => Ok(request),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

fn default_limit() -> u16 {
    100
}

/// Cursor metadata returned by a paged response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct PageInfo {
    /// Number of records in the response.
    #[serde(default)]
    pub count: Option<u64>,
    /// Total number of matching records, when reported.
    #[serde(default)]
    pub total_count: Option<u64>,
    /// Cursor for the first record.
    #[serde(default)]
    pub start_cursor: Option<Cursor>,
    /// Cursor for the last record.
    #[serde(default)]
    pub end_cursor: Option<Cursor>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A typed paged response with preserved provider fields.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Page<T> {
    /// Records in provider order.
    pub data: Vec<T>,
    /// Cursor metadata.
    pub page_info: PageInfo,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

#[cfg(test)]
#[path = "_tests_/pagination_tests.rs"]
mod pagination_tests;
