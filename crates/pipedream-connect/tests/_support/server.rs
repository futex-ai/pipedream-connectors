//! Local HTTP servers and requests shared by transport integration tests.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::hang_guard::guard;

pub(crate) async fn server(response: Vec<u8>) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/request", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        guard(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut bytes = [0; 1024];
                let read = socket.read(&mut bytes).await.unwrap();
                assert_ne!(read, 0);
                request.extend_from_slice(&bytes[..read]);
                assert!(request.len() < 16384);
                let Some(end) = request.windows(4).position(|value| value == b"\r\n\r\n") else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&request[..end]);
                let length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if request.len() >= end + 4 + length {
                    break;
                }
            }
            let _ = socket.write_all(&response).await;
            request
        })
        .await
    });
    (url, task)
}
