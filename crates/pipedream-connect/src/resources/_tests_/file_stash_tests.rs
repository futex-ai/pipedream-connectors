//! File Stash byte and redirect response tests.

use http::HeaderValue;
use pipedream_connect_types::DownloadFileStashRequest;
use serde_json::json;

use crate::resources::file_stash::FileStashDownload;
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn file_stash_returns_bytes_and_does_not_follow_redirects() {
    let mut redirect = response(302, json!(null));
    redirect.headers.insert(
        "location",
        HeaderValue::from_static("https://signed.example/file"),
    );
    let mut bytes = response(200, json!(null));
    bytes.body = vec![0, 1, 255];
    let harness = Harness::script(static_config(), vec![Ok(bytes), Ok(redirect)]);
    let request = DownloadFileStashRequest {
        s3_key: "1day/a/b/c/u/file".into(),
    };
    assert_eq!(
        harness
            .client
            .file_stash()
            .download(request.clone())
            .await
            .unwrap(),
        FileStashDownload::Bytes(vec![0, 1, 255])
    );
    assert_eq!(
        harness.client.file_stash().download(request).await.unwrap(),
        FileStashDownload::Redirect("https://signed.example/file".into())
    );
    assert_eq!(harness.requests.lock().unwrap().len(), 2);
    harness.assert_requests(2);
}
