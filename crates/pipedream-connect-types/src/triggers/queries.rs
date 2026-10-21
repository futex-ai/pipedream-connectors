//! Trigger deployment models and requests.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Error, ExternalUserId, Result, ValidationField};

/// Trigger event history query.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ListTriggerEventsRequest {
    /// External user that owns the deployed trigger.
    pub external_user_id: ExternalUserId,
    /// Maximum event count.
    #[serde(rename = "n")]
    pub limit: u16,
}

impl ListTriggerEventsRequest {
    /// Creates a bounded event history request.
    pub fn new(external_user_id: ExternalUserId, limit: u16) -> Result<Self> {
        if !(1..=100).contains(&limit) {
            return Err(Error::OutOfRange {
                field: ValidationField::TriggerEventLimit,
                minimum: 1,
                maximum: 100,
            });
        }
        Ok(Self {
            external_user_id,
            limit,
        })
    }
}

impl<'de> Deserialize<'de> for ListTriggerEventsRequest {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            external_user_id: ExternalUserId,
            n: u16,
        }

        let wire = Wire::deserialize(deserializer)?;
        match Self::new(wire.external_user_id, wire.n) {
            Ok(request) => Ok(request),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

/// Delete deployed trigger options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeleteTriggerRequest {
    /// External user that owns the deployed trigger.
    pub external_user_id: ExternalUserId,
    /// Ignore hook errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_hook_errors: Option<bool>,
}

/// External user query shared by deployed-trigger operations.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DeployedTriggerQuery {
    /// External user that owns the deployed trigger.
    pub external_user_id: ExternalUserId,
}
