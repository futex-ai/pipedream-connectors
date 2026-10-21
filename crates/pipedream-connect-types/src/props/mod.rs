//! Lossless configurable property definitions returned by registry endpoints.

mod values;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::{AppSlug, PropName, UnknownFields};

pub use values::{ConfiguredPropValue, ConfiguredProps, PropOption, PropOptionValue, TimerValue};

/// A known or provider-defined property kind.
#[derive(Clone, Debug, PartialEq)]
pub enum PropKind {
    /// An informational alert.
    Alert,
    /// Arbitrary JSON.
    Any,
    /// An account selector.
    App,
    /// An app webhook selector.
    Apphook,
    /// Airtable base selector.
    AirtableBaseId,
    /// Airtable field selector.
    AirtableFieldId,
    /// Airtable table selector.
    AirtableTableId,
    /// Airtable view selector.
    AirtableViewId,
    /// Boolean value.
    Boolean,
    /// Discord channel selector.
    DiscordChannel,
    /// Discord channel array selector.
    DiscordChannelArray,
    /// Data store service.
    DataStore,
    /// File Stash directory.
    Dir,
    /// HTTP interface.
    Http,
    /// HTTP request configuration.
    HttpRequest,
    /// Integer value.
    Integer,
    /// Integer array.
    IntegerArray,
    /// Object value.
    Object,
    /// SQL query configuration.
    Sql,
    /// String value.
    String,
    /// String array.
    StringArray,
    /// Timer interface.
    Timer,
    /// Database service.
    Database,
    /// A provider kind unknown to this SDK, with its complete raw definition.
    Other {
        /// Unknown type tag.
        type_name: String,
        /// Complete property definition.
        raw_definition: Value,
    },
}

impl PropKind {
    fn type_name(&self) -> &str {
        match self {
            Self::Alert => "alert",
            Self::Any => "any",
            Self::App => "app",
            Self::Apphook => "$.interface.apphook",
            Self::AirtableBaseId => "$.airtable.baseId",
            Self::AirtableFieldId => "$.airtable.fieldId",
            Self::AirtableTableId => "$.airtable.tableId",
            Self::AirtableViewId => "$.airtable.viewId",
            Self::Boolean => "boolean",
            Self::DiscordChannel => "$.discord.channel",
            Self::DiscordChannelArray => "$.discord.channel[]",
            Self::DataStore => "data_store",
            Self::Dir => "dir",
            Self::Http => "$.interface.http",
            Self::HttpRequest => "http_request",
            Self::Integer => "integer",
            Self::IntegerArray => "integer[]",
            Self::Object => "object",
            Self::Sql => "sql",
            Self::String => "string",
            Self::StringArray => "string[]",
            Self::Timer => "$.interface.timer",
            Self::Database => "$.service.db",
            Self::Other { type_name, .. } => type_name,
        }
    }

    fn from_type(type_name: String, raw_definition: Value) -> Self {
        match type_name.as_str() {
            "alert" => Self::Alert,
            "any" => Self::Any,
            "app" => Self::App,
            "$.interface.apphook" => Self::Apphook,
            "$.airtable.baseId" => Self::AirtableBaseId,
            "$.airtable.fieldId" => Self::AirtableFieldId,
            "$.airtable.tableId" => Self::AirtableTableId,
            "$.airtable.viewId" => Self::AirtableViewId,
            "boolean" => Self::Boolean,
            "$.discord.channel" => Self::DiscordChannel,
            "$.discord.channel[]" => Self::DiscordChannelArray,
            "data_store" => Self::DataStore,
            "dir" => Self::Dir,
            "$.interface.http" => Self::Http,
            "http_request" => Self::HttpRequest,
            "integer" => Self::Integer,
            "integer[]" => Self::IntegerArray,
            "object" => Self::Object,
            "sql" => Self::Sql,
            "string" => Self::String,
            "string[]" => Self::StringArray,
            "$.interface.timer" => Self::Timer,
            "$.service.db" => Self::Database,
            _ => Self::Other {
                type_name,
                raw_definition,
            },
        }
    }
}

/// A configurable property with common fields and lossless type details.
#[derive(Clone, Debug, PartialEq)]
pub struct ConfigurableProp {
    /// Exact property name.
    pub name: PropName,
    /// Property type and unknown raw definition when needed.
    pub kind: PropKind,
    /// Display label.
    pub label: Option<String>,
    /// User-facing description.
    pub description: Option<String>,
    /// Whether the property may be omitted.
    pub optional: Option<bool>,
    /// Whether the property is disabled.
    pub disabled: Option<bool>,
    /// Whether the property is read-only.
    pub read_only: Option<bool>,
    /// Whether the property is hidden.
    pub hidden: Option<bool>,
    /// Whether remote options should be loaded.
    pub remote_options: Option<bool>,
    /// Whether changing this property reloads properties.
    pub reload_props: Option<bool>,
    /// Whether labels must be retained with values.
    pub with_label: Option<bool>,
    /// App slug used by account selector properties.
    pub app: Option<AppSlug>,
    /// Typed default value, when supplied.
    pub default: Option<ConfiguredPropValue>,
    /// Labelled or scalar static choices.
    pub options: Vec<PropOption>,
    /// Unknown fields from the provider.
    pub extra: UnknownFields,
}

impl Serialize for ConfigurableProp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let PropKind::Other { raw_definition, .. } = &self.kind {
            return raw_definition.serialize(serializer);
        }
        let mut object = Map::new();
        object.insert("name".into(), Value::String(self.name.as_str().to_owned()));
        object.insert("type".into(), Value::String(self.kind.type_name().into()));
        insert_option(&mut object, "label", &self.label);
        insert_option(&mut object, "description", &self.description);
        insert_option(&mut object, "optional", &self.optional);
        insert_option(&mut object, "disabled", &self.disabled);
        insert_option(&mut object, "readOnly", &self.read_only);
        insert_option(&mut object, "hidden", &self.hidden);
        insert_option(&mut object, "remoteOptions", &self.remote_options);
        insert_option(&mut object, "reloadProps", &self.reload_props);
        insert_option(&mut object, "withLabel", &self.with_label);
        insert_option(&mut object, "app", &self.app);
        insert_option(&mut object, "default", &self.default);
        if !self.options.is_empty() {
            let options = match serde_json::to_value(&self.options) {
                Ok(options) => options,
                Err(error) => return Err(serde::ser::Error::custom(error)),
            };
            object.insert("options".into(), options);
        }
        for (key, value) in &self.extra {
            object.entry(key.clone()).or_insert_with(|| value.clone());
        }
        Value::Object(object).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ConfigurableProp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Value::deserialize(deserializer)?;
        let mut object = match raw.clone() {
            Value::Object(object) => object,
            _ => {
                return Err(serde::de::Error::custom(
                    "configurable prop must be an object",
                ));
            }
        };
        let name_value = object
            .get("name")
            .cloned()
            .ok_or_else(|| serde::de::Error::custom("configurable prop name is missing"))?;
        let name = match serde_json::from_value(name_value) {
            Ok(name) => name,
            Err(error) => return Err(serde::de::Error::custom(error)),
        };
        let type_name = object
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("configurable prop type is missing"))?
            .to_owned();
        let kind = PropKind::from_type(type_name, raw.clone());
        macro_rules! field {
            ($type:ty, $key:literal) => {
                match take::<$type>(&mut object, $key) {
                    Ok(value) => value,
                    Err(error) => return Err(serde::de::Error::custom(error)),
                }
            };
        }
        let label = field!(Option<String>, "label");
        let description = field!(Option<String>, "description");
        let optional = field!(Option<bool>, "optional");
        let disabled = field!(Option<bool>, "disabled");
        let read_only = field!(Option<bool>, "readOnly");
        let hidden = field!(Option<bool>, "hidden");
        let remote_options = field!(Option<bool>, "remoteOptions");
        let reload_props = field!(Option<bool>, "reloadProps");
        let with_label = field!(Option<bool>, "withLabel");
        let app = field!(Option<AppSlug>, "app");
        let default = field!(Option<ConfiguredPropValue>, "default");
        let options = field!(Option<Vec<PropOption>>, "options").unwrap_or_default();
        object.remove("name");
        object.remove("type");
        Ok(Self {
            name,
            kind,
            label,
            description,
            optional,
            disabled,
            read_only,
            hidden,
            remote_options,
            reload_props,
            with_label,
            app,
            default,
            options,
            extra: object.into_iter().collect(),
        })
    }
}

fn insert_option<T: Serialize>(object: &mut Map<String, Value>, key: &str, value: &Option<T>) {
    if let Some(value) = value
        && let Ok(value) = serde_json::to_value(value)
    {
        object.insert(key.to_owned(), value);
    }
}

fn take<T: for<'de> Deserialize<'de>>(
    object: &mut Map<String, Value>,
    key: &str,
) -> std::result::Result<T, serde_json::Error> {
    match object.remove(key) {
        Some(value) => match serde_json::from_value(value) {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        },
        None => match serde_json::from_value(Value::Null) {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        },
    }
}
