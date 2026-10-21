//! Crate-wide string enum wire values and forward compatibility.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;

use crate::{
    AppAuthType, AppSortKey, ComponentStash, ComponentType, EmitterType, Environment, ErrorOrigin,
    GrantType, RegistryKind, ScopeProfileName, SortDirection,
};

fn check<T: DeserializeOwned + Serialize>(values: &[&str]) {
    for value in values.iter().chain(["future_value"].iter()) {
        let decoded: T = serde_json::from_value(json!(value)).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), json!(value));
    }
}

#[test]
fn every_string_enum_serializes_known_and_unknown_raw_strings() {
    check::<AppAuthType>(&["keys", "oauth", "none"]);
    check::<AppSortKey>(&["name", "name_slug", "featured_weight"]);
    check::<ComponentStash>(&["optional", "required"]);
    check::<ComponentType>(&["action", "trigger"]);
    check::<EmitterType>(&["email", "http", "source", "timer"]);
    check::<Environment>(&["development", "production"]);
    check::<ErrorOrigin>(&[
        "component_code",
        "upstream_api",
        "network_io",
        "response_parsing",
    ]);
    check::<GrantType>(&["client_credentials"]);
    check::<RegistryKind>(&["public", "private", "all"]);
    check::<ScopeProfileName>(&["read_only", "read_write", "admin"]);
    check::<SortDirection>(&["asc", "desc"]);
    let unknown: AppAuthType = serde_json::from_value(json!("future_auth")).unwrap();
    assert_eq!(unknown, AppAuthType::Other("future_auth".into()));
}
