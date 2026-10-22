//! Query models for project administration.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{AppSlug, PageRequest};

/// Optional body and content type for workflow invocation.
#[derive(Clone, Default, PartialEq)]
pub struct WorkflowRequest {
    /// Exact request bytes, when the workflow call has a body.
    pub body: Option<Vec<u8>>,
    /// Content type for the body, when supplied.
    pub content_type: Option<String>,
}

impl fmt::Debug for WorkflowRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkflowRequest")
            .field("body_bytes", &self.body.as_ref().map(Vec::len))
            .field("content_type", &self.content_type)
            .finish()
    }
}

/// Project list query.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ListProjectsRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Project search sent as `q`.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

/// Custom OAuth app list query.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ListOauthAppsRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Provider app filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppSlug>,
    /// Search sent as `q`.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

/// App override list query.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ListAppOverridesRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Provider app filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppSlug>,
}

/// Connect usage time-window query.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ListUsageRequest {
    /// Inclusive start time in epoch seconds.
    pub start_ts: i64,
    /// End time in epoch seconds.
    pub end_ts: i64,
}

/// File Stash download query.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DownloadFileStashRequest {
    /// Exact S3 key supplied by Pipedream.
    pub s3_key: String,
}
