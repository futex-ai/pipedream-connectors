//! Replacement of credential-class values before a result reaches standard output.

use pipedream_connect_types::{
    Account, App, AppCategory, Component, Data, DeployedComponent, Emitter, Page, ProjectWebhook,
    RunActionResponse, SecretJson, SecretString, TriggerWebhook, TriggerWebhooks,
    WebhookWithOptionalSigningKey, WebhookWithSigningKey,
};
use serde_json::Value;

/// The text that replaces each credential-class value.
pub(crate) const REDACTED: &str = "[REDACTED]";

/// A result model that can replace its credential-class values.
///
/// Every model that a command prints implements this trait, so a new model
/// cannot reach standard output before its credentials are considered.
pub(crate) trait Redact {
    /// Replaces every credential-class value with [`REDACTED`].
    fn redact(&mut self);
}

impl Redact for SecretString {
    fn redact(&mut self) {
        *self = SecretString::new(REDACTED);
    }
}

impl Redact for SecretJson {
    fn redact(&mut self) {
        *self = SecretJson::new(Value::String(REDACTED.to_owned()));
    }
}

impl<T: Redact> Redact for Option<T> {
    fn redact(&mut self) {
        if let Some(value) = self {
            value.redact();
        }
    }
}

impl<T: Redact> Redact for Vec<T> {
    fn redact(&mut self) {
        for value in self {
            value.redact();
        }
    }
}

impl<T: Redact> Redact for Page<T> {
    fn redact(&mut self) {
        self.data.redact();
    }
}

impl<T: Redact> Redact for Data<T> {
    fn redact(&mut self) {
        self.data.redact();
    }
}

impl Redact for Account {
    fn redact(&mut self) {
        self.credentials.redact();
    }
}

impl Redact for TriggerWebhook {
    fn redact(&mut self) {
        self.signing_key.redact();
    }
}

impl Redact for TriggerWebhooks {
    fn redact(&mut self) {
        self.webhooks.redact();
    }
}

impl Redact for DeployedComponent {
    fn redact(&mut self) {
        self.webhook_signing_key.redact();
        self.webhooks.redact();
    }
}

impl Redact for Emitter {
    fn redact(&mut self) {
        if let Self::DeployedComponent(component) = self {
            component.redact();
        }
    }
}

impl Redact for ProjectWebhook {
    fn redact(&mut self) {
        self.signing_key.redact();
    }
}

impl Redact for WebhookWithOptionalSigningKey {
    fn redact(&mut self) {
        self.signing_key.redact();
    }
}

impl Redact for WebhookWithSigningKey {
    fn redact(&mut self) {
        self.signing_key.redact();
    }
}

/// Implements [`Redact`] for models that hold no credential-class field.
macro_rules! no_credentials {
    ($($model:ty),+ $(,)?) => {
        $(
            impl Redact for $model {
                fn redact(&mut self) {}
            }
        )+
    };
}

no_credentials!(App, AppCategory, Component, RunActionResponse);

#[cfg(test)]
#[path = "_tests_/redact_tests.rs"]
mod redact_tests;
