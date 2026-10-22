use std::sync::Mutex;

use pipedream_connect::transport::HttpRequest;

#[derive(Default)]
pub struct Recorder {
    requests: Mutex<Vec<HttpRequest>>,
}

impl Recorder {
    pub fn record(&self, request: HttpRequest) {
        self.requests.lock().expect("recording lock").push(request);
    }

    pub fn clear(&self) {
        self.requests.lock().expect("recording lock").clear();
    }

    pub fn take_one(&self, operation: &str) -> HttpRequest {
        let mut requests = self.requests.lock().expect("recording lock");
        assert_eq!(
            requests.len(),
            1,
            "{operation} sent an unexpected request count"
        );
        requests.pop().expect("recorded request")
    }

    pub fn take_oauth(&self) -> HttpRequest {
        let mut requests = self.requests.lock().expect("recording lock");
        let index = requests
            .iter()
            .position(|request| request.url.expose_secret().ends_with("/v1/oauth/token"))
            .expect("OAuth token request");
        requests.remove(index)
    }
}

mod cases;
pub use cases::invoke;
