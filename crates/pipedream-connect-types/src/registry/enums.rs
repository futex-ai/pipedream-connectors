//! Registry enum values with lossless unknown variants.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! string_enum {
    ($(#[$meta:meta])* pub enum $name:ident { $($(#[$variant_meta:meta])* $variant:ident => $wire:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
            /// A value added by the provider after this SDK version.
            Other(String),
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where S: Serializer {
                let value = match self {
                    $(Self::$variant => $wire,)+
                    Self::Other(value) => value.as_str(),
                };
                serializer.serialize_str(value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where D: Deserializer<'de> {
                let value = String::deserialize(deserializer)?;
                Ok(match value.as_str() {
                    $($wire => Self::$variant,)+
                    _ => Self::Other(value),
                })
            }
        }
    };
}

string_enum! {
    /// Authentication mode advertised by an app.
    pub enum AppAuthType {
        /// API-key authentication.
        Keys => "keys",
        /// Hosted OAuth authentication.
        Oauth => "oauth",
        /// No account authentication.
        None => "none",
    }
}

string_enum! {
    /// Registry component kind.
    pub enum ComponentType {
        /// An executable action.
        Action => "action",
        /// A deployable trigger.
        Trigger => "trigger",
    }
}

string_enum! {
    /// Whether a component requires a File Stash reference.
    pub enum ComponentStash {
        /// A stash reference may be omitted.
        Optional => "optional",
        /// A stash reference is required.
        Required => "required",
    }
}

string_enum! {
    /// App listing sort key.
    pub enum AppSortKey {
        /// Sort by display name.
        Name => "name",
        /// Sort by provider slug.
        NameSlug => "name_slug",
        /// Sort by featured weight.
        FeaturedWeight => "featured_weight",
    }
}

string_enum! {
    /// App listing direction.
    pub enum SortDirection {
        /// Ascending order.
        Asc => "asc",
        /// Descending order.
        Desc => "desc",
    }
}

string_enum! {
    /// Component registry source.
    pub enum RegistryKind {
        /// Public registry components.
        Public => "public",
        /// Private registry components.
        Private => "private",
        /// Both public and private components.
        All => "all",
    }
}

string_enum! {
    /// Named OAuth scope profile.
    pub enum ScopeProfileName {
        /// Read-only access.
        ReadOnly => "read_only",
        /// Read and write access.
        ReadWrite => "read_write",
        /// Administrative access.
        Admin => "admin",
    }
}
