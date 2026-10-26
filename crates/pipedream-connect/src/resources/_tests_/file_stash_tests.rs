//! File Stash byte and redirect response tests.

use http::HeaderValue;
use pipedream_connect_types::DownloadFileStashRequest;
use serde_json::json;

use crate::resources::file_stash::FileStashDownload;
use crate::resources::test_support::assert_wire;
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
    for (index, url) in [
        "https://api.pipedream.com/v1/connect/proj_Test123/file_stash/download?s3_key=1day%2Fa%2Fb%2Fc%2Fu%2Ffile",
        "https://api.pipedream.com/v1/connect/proj_Test123/file_stash/download?s3_key=1day%2Fa%2Fb%2Fc%2Fu%2Ffile",
    ]
    .into_iter()
    .enumerate()
    {
        assert_wire(
            &harness.request(index),
            http::Method::GET,
            url,
            &[
                ("authorization", "Bearer static-secret"),
                ("x-pd-environment", "development"),
                ("user-agent", concat!("pipedream-connect/", env!("CARGO_PKG_VERSION"))),
            ],
            None,
        );
    }
    assert_eq!(harness.requests.lock().unwrap().len(), 2);
    harness.assert_requests(2);
}

#[test]
fn file_stash_debug_hides_body_bytes_and_presigned_query() {
    let bytes = format!("{:?}", FileStashDownload::Bytes(vec![1, 2, 3, 4]));
    assert!(bytes.contains("4"));
    assert!(!bytes.contains("1, 2, 3, 4"));
    let redirect = format!(
        "{:?}",
        FileStashDownload::Redirect(
            "https://signed.example/file?signature=secret&expires=10".into(),
        )
    );
    assert!(redirect.contains("https://signed.example/file"));
    assert!(!redirect.contains("signature"));
    assert!(!redirect.contains("secret"));
}
