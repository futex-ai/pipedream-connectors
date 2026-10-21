//! Extra-field assertions for conformance model values.

use pipedream_connect_types::{
    App, AppCategory, AppOverride, ConnectTokenResponse, Data, ErrorResponse, OAuthTokenResponse,
    OauthApp, PageEnvelope, Project, ProjectInfo, ReloadPropsResponse, RunActionResponse,
    UnknownFields, ValidateTokenResponse,
};

pub trait HasExtra {
    fn extra(&self) -> Option<&UnknownFields>;
}

macro_rules! extra_impl {
    ($($type:ty),+ $(,)?) => {$ (
        impl HasExtra for $type {
            fn extra(&self) -> Option<&UnknownFields> { Some(&self.extra) }
        }
    )+ };
}

extra_impl!(
    pipedream_connect_types::Account,
    App,
    AppOverride,
    pipedream_connect_types::ConfigurePropResponse,
    pipedream_connect_types::ConnectUsageResponse,
    OAuthTokenResponse,
    ConnectTokenResponse,
    ErrorResponse,
    AppCategory,
    OauthApp,
    Project,
    ProjectInfo,
    ReloadPropsResponse,
    RunActionResponse,
    pipedream_connect_types::TriggerWebhooks,
    pipedream_connect_types::TriggerWorkflows,
    ValidateTokenResponse,
);

impl<T> HasExtra for Data<T> {
    fn extra(&self) -> Option<&UnknownFields> {
        Some(&self.extra)
    }
}

impl<T> HasExtra for PageEnvelope<T> {
    fn extra(&self) -> Option<&UnknownFields> {
        Some(&self.extra)
    }
}

impl HasExtra for Vec<AppCategory> {
    fn extra(&self) -> Option<&UnknownFields> {
        None
    }
}
