//! Project administration models.

use serde::{Deserialize, Serialize};

use crate::{Timestamp, UnknownFields};

/// Connect usage for one time window.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UsageRecord {
    /// Total credits used.
    pub credits_used: i64,
    /// Action credits.
    #[serde(default)]
    pub action_run_credits_used: Option<i64>,
    /// Proxy credits.
    #[serde(default)]
    pub proxy_credits_used: Option<i64>,
    /// Trigger emission credits.
    #[serde(default)]
    pub source_emit_credits_used: Option<i64>,
    /// Start timestamp in epoch seconds.
    #[serde(deserialize_with = "crate::shared::deserialize_timestamp_seconds")]
    pub usage_start_ts: Timestamp,
    /// End timestamp in epoch seconds.
    #[serde(deserialize_with = "crate::shared::deserialize_timestamp_seconds")]
    pub usage_end_ts: Timestamp,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
