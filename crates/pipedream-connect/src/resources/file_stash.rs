//! File Stash download operations without redirect following.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use http::header::LOCATION;
use pipedream_connect_types::DownloadFileStashRequest;
use url::Url;

use crate::client::RequestDispatcher;
use crate::error::{Error, Result, WireFailure};

use super::admin_operations;
use super::operation;

/// A file download returned directly or as a presigned redirect location.
#[derive(Clone, Eq, PartialEq)]
pub enum FileStashDownload {
    /// File contents returned in the response body.
    Bytes(Vec<u8>),
    /// Presigned location returned by a redirect response.
    Redirect(String),
}

impl fmt::Debug for FileStashDownload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("FileStashDownload");
        match self {
            Self::Bytes(bytes) => debug.field("bytes", &bytes.len()),
            Self::Redirect(location) => {
                let safe_location = Url::parse(location)
                    .map(|mut url| {
                        url.set_query(None);
                        url.to_string()
                    })
                    .unwrap_or_else(|_| "[REDACTED]".to_owned());
                debug.field("url", &safe_location)
            }
        };
        debug.finish()
    }
}

/// File Stash operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = FileStashApiMock)
)]
#[async_trait]
pub trait FileStashApi: Send + Sync {
    /// Downloads a file or returns its redirect location without following it.
    async fn download(&self, request: DownloadFileStashRequest) -> Result<FileStashDownload>;
}

/// A File Stash handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct FileStash {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl FileStash {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl FileStashApi for FileStash {
    async fn download(&self, request: DownloadFileStashRequest) -> Result<FileStashDownload> {
        let response = operation::dispatch(
            &self.dispatcher,
            admin_operations::download_file_stash(&request),
        )
        .await?;
        if response.status.is_redirection() {
            let location = match response.headers.get(LOCATION) {
                Some(value) => match value.to_str() {
                    Ok(value) => value.to_owned(),
                    Err(_) => return Err(Error::internal(WireFailure::MissingLocation)),
                },
                None => return Err(Error::internal(WireFailure::MissingLocation)),
            };
            Ok(FileStashDownload::Redirect(location))
        } else {
            Ok(FileStashDownload::Bytes(response.body))
        }
    }
}

#[cfg(test)]
#[path = "_tests_/file_stash_tests.rs"]
mod file_stash_tests;
