//! Recursive assertions for the unknown-field maps of response models.

use pipedream_connect_types::*;

pub trait HasExtra {
    fn check_extra(&self);
}

fn extra(fields: &UnknownFields) {
    for (key, value) in fields {
        assert!(
            key == "__future" || value.is_null(),
            "documented field in extra: {key}: {value}"
        );
    }
}

macro_rules! model {
    ($ty:ty $(, $child:ident)* $(,)?) => {
        impl HasExtra for $ty {
            fn check_extra(&self) {
                extra(&self.extra);
                $(self.$child.check_extra();)*
            }
        }
    };
}

impl<T: HasExtra> HasExtra for Option<T> {
    fn check_extra(&self) {
        if let Some(value) = self {
            value.check_extra();
        }
    }
}
impl<T: HasExtra> HasExtra for Vec<T> {
    fn check_extra(&self) {
        for value in self {
            value.check_extra();
        }
    }
}
impl<T: HasExtra> HasExtra for Data<T> {
    fn check_extra(&self) {
        extra(&self.extra);
        self.data.check_extra();
    }
}
impl<T: HasExtra> HasExtra for Page<T> {
    fn check_extra(&self) {
        extra(&self.extra);
        self.page_info.check_extra();
        self.data.check_extra();
    }
}

model!(Account, app);
model!(App, scope_profiles, connect);
model!(ScopeProfile);
model!(ProxySupport);
model!(AppCategory);
model!(AppOverride, app);
model!(Component, configurable_props, annotations);
model!(ToolAnnotations);
model!(PageInfo);
model!(ConnectTokenResponse);
model!(ConfigurePropResponse, options, observations);
model!(ConnectUsageResponse, data);
model!(UsageRecord);
model!(OAuthTokenResponse);
model!(OauthApp, scope_profiles);
model!(Project);
model!(ProjectInfo, apps);
model!(ProjectInfoApp);
model!(ErrorResponse);
model!(ExternalUser);
model!(ReloadPropsResponse, observations, dynamic_props);
model!(DynamicProps, configurable_props);
model!(RunActionResponse, os);
model!(Observation, err);
model!(ObservationError);
model!(TriggerWebhooks, webhooks);
model!(TriggerWebhook);
model!(TriggerWorkflows);
model!(EmittedEvent);
model!(Webhook);
model!(WebhookWithSigningKey);
model!(WebhookWithOptionalSigningKey);
model!(ValidateTokenResponse, app);
model!(
    DeployedComponent,
    configurable_props,
    configured_props,
    webhooks
);
model!(HttpInterface);
model!(TimerInterface);

impl HasExtra for Emitter {
    fn check_extra(&self) {
        match self {
            Self::DeployedComponent(value) => value.check_extra(),
            Self::HttpInterface(value) => value.check_extra(),
            Self::TimerInterface(value) => value.check_extra(),
            Self::Other(_) => {}
        }
    }
}
impl HasExtra for PropOption {
    fn check_extra(&self) {
        match self {
            Self::Labelled { extra: fields, .. } => extra(fields),
            Self::Nested {
                option,
                extra: fields,
            } => {
                extra(fields);
                option.check_extra();
            }
            Self::Scalar(_) => {}
        }
    }
}
impl HasExtra for ConfiguredPropValue {
    fn check_extra(&self) {
        match self {
            Self::App { extra: fields, .. } | Self::Sql { extra: fields, .. } => extra(fields),
            _ => {}
        }
    }
}
impl HasExtra for ConfiguredProps {
    fn check_extra(&self) {
        for (_, value) in self.iter() {
            value.check_extra();
        }
    }
}
impl HasExtra for ConfigurableProp {
    fn check_extra(&self) {
        let details = [
            "alertType",
            "content",
            "static",
            "appProp",
            "eventNames",
            "remote",
            "min",
            "max",
            "customResponse",
            "accessMode",
            "sync",
            "auth",
            "baseIdProp",
            "tableIdProp",
            "secret",
            "format",
        ];
        for (key, value) in &self.extra {
            assert!(
                key == "__future" || value.is_null() || details.contains(&key.as_str()),
                "prop common field in extra: {key}"
            );
        }
        self.default.check_extra();
        self.options.check_extra();
    }
}
