//! Known property kinds and raw definitions for new provider kinds.

use serde_json::Value;

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
    pub(super) fn type_name(&self) -> &str {
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

    pub(super) fn from_type(type_name: String, raw_definition: Value) -> Self {
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
