//! Command results as complete JSON documents with redacted credentials.

use pipedream_connect_types::PageRequest;
use serde::Serialize;

use crate::error::{Error, Result};
use crate::redact::Redact;
use crate::target::Target;

/// Whether a result may show credential-class values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Secrets {
    /// Replace each credential-class value with the redaction marker.
    Redact,
    /// Show credential-class values, because the caller passed `--show-secrets`.
    Show,
}

/// One pretty-printed JSON document, encoded completely before any byte is written.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Document {
    json: Vec<u8>,
}

impl Document {
    /// Returns the encoded JSON without a trailing newline.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.json
    }
}

/// The result of a command whose response has no resource body.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub(crate) struct Acknowledgement {
    status: AcknowledgedStatus,
}

/// What Pipedream acknowledged.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum AcknowledgedStatus {
    /// Pipedream accepted a run and completes it later.
    Accepted,
    /// Pipedream deleted the resource.
    Deleted,
}

impl Acknowledgement {
    /// A run that Pipedream accepted for later completion.
    pub(crate) fn accepted() -> Self {
        Self {
            status: AcknowledgedStatus::Accepted,
        }
    }

    /// A resource that Pipedream deleted.
    pub(crate) fn deleted() -> Self {
        Self {
            status: AcknowledgedStatus::Deleted,
        }
    }
}

impl Redact for Acknowledgement {
    fn redact(&mut self) {}
}

/// A fact about a successful result that the caller needs to know.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Notice {
    /// `--all` stopped at its page budget. `next` continues the listing.
    PageLimit {
        /// The number of pages read.
        pages: usize,
        /// The request for the next page.
        next: PageRequest,
    },
    /// Pipedream repeated a page cursor, so the listing stopped early.
    StuckCursor,
}

/// A finished command: the target of its request, its result, and its notices.
#[derive(Debug)]
pub(crate) struct Output {
    /// The resource of the request that produced the result.
    pub(crate) target: Target,
    /// The result for standard output.
    pub(crate) document: Document,
    /// Notices for standard error.
    pub(crate) notices: Vec<Notice>,
}

impl Output {
    /// Encodes the result of a request on `target`, with its credentials redacted
    /// unless `secrets` is `Show`.
    pub(crate) fn redacted<T: Serialize + Redact>(
        target: Target,
        mut value: T,
        secrets: Secrets,
    ) -> Result<Self> {
        if secrets == Secrets::Redact {
            value.redact();
        }
        Self::revealed(target, &value)
    }

    /// Encodes the result of a request on `target` as it is. Only a command that
    /// exists to produce credentials uses it.
    pub(crate) fn revealed<T: Serialize>(target: Target, value: &T) -> Result<Self> {
        match serde_json::to_vec_pretty(value) {
            Ok(json) => Ok(Self {
                target,
                document: Document { json },
                notices: Vec::new(),
            }),
            Err(cause) => Err(Error::Unencodable { target, cause }),
        }
    }

    /// Adds the notices that standard error shows after the result.
    pub(crate) fn with_notices(mut self, notices: Vec<Notice>) -> Self {
        self.notices = notices;
        self
    }
}

#[cfg(test)]
#[path = "_tests_/document_tests.rs"]
mod document_tests;
