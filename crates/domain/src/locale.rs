//! Shared locale model for public content localization.

use serde::{Deserialize, Serialize};

/// Public locales currently supported by the API.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
pub enum Locale {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en")]
    En,
}

impl Locale {
    /// Returns the public BCP-47 locale tag used by API responses and headers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    /// Parses supported locale aliases from headers while rejecting encoded or unknown values.
    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "zh" | "zh-cn" | "zh-hans" | "zh-hans-cn" | "cn" => Some(Self::ZhCn),
            "en" | "en-us" | "en-gb" => Some(Self::En),
            _ => None,
        }
    }

    /// Returns locale fallback order for DB localization lookup.
    pub fn fallbacks(self) -> [Self; 2] {
        match self {
            Self::ZhCn => [Self::ZhCn, Self::En],
            Self::En => [Self::En, Self::ZhCn],
        }
    }
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}
