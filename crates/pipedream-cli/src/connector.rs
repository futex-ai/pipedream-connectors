//! Composition of the SDK client behind the resource factory trait.

use std::sync::Arc;

use pipedream_connect::PipedreamClient;
use pipedream_connect::reqwest_transport::ReqwestTransport;
use pipedream_connect::resources::ConnectResources;
use pipedream_connect::runtime::{SystemClock, TokioSleeper};

use crate::settings::Settings;

/// Builds the resource factory once per command. Construction sends no request.
#[cfg_attr(test, unimock::unimock(api = [ConnectMock]))]
pub(crate) trait Connector: Send + Sync {
    /// Returns the resource factory for `settings`.
    fn connect(
        &self,
        settings: &Settings,
    ) -> pipedream_connect::error::Result<Arc<dyn ConnectResources + Send + Sync>>;
}

/// Builds the SDK client with the reqwest transport, the system clock, and the Tokio sleeper.
pub(crate) struct SdkConnector;

impl Connector for SdkConnector {
    fn connect(
        &self,
        settings: &Settings,
    ) -> pipedream_connect::error::Result<Arc<dyn ConnectResources + Send + Sync>> {
        let config = settings.client_config()?;
        let transport = Arc::new(ReqwestTransport::new(config.http_options())?);
        let client = PipedreamClient::new(
            config,
            transport,
            Arc::new(SystemClock::default()),
            Arc::new(TokioSleeper),
        )?;
        Ok(Arc::new(client))
    }
}
