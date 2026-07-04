use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::McpError;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct McpServerId(String);

impl McpServerId {
    pub fn new(value: impl Into<String>) -> Result<Self, McpError> {
        let value = value.into();
        if value.is_empty()
            || !value.chars().all(|ch| {
                ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.')
            })
        {
            return Err(McpError::InvalidConfig(format!(
                "invalid server id `{value}`"
            )));
        }

        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for McpServerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for McpServerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct McpServerConfig {
    pub id: McpServerId,
    pub display_name: String,
    pub transport: McpTransportConfig,
    pub enabled: bool,
    pub timeout_ms: u64,
}

impl McpServerConfig {
    pub fn new(
        id: McpServerId,
        display_name: impl Into<String>,
        transport: McpTransportConfig,
    ) -> Result<Self, McpError> {
        let display_name = display_name.into();
        validate_config_parts(&display_name, &transport, 30_000)?;

        Ok(Self {
            id,
            display_name,
            transport,
            enabled: true,
            timeout_ms: 30_000,
        })
    }

    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl<'de> Deserialize<'de> for McpServerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct McpServerConfigWire {
            id: McpServerId,
            display_name: String,
            transport: McpTransportConfig,
            enabled: bool,
            timeout_ms: u64,
        }

        let wire = McpServerConfigWire::deserialize(deserializer)?;
        validate_config_parts(&wire.display_name, &wire.transport, wire.timeout_ms)
            .map_err(D::Error::custom)?;

        Ok(Self {
            id: wire.id,
            display_name: wire.display_name,
            transport: wire.transport,
            enabled: wire.enabled,
            timeout_ms: wire.timeout_ms,
        })
    }
}

fn validate_config_parts(
    display_name: &str,
    transport: &McpTransportConfig,
    timeout_ms: u64,
) -> Result<(), McpError> {
    if display_name.trim().is_empty() {
        return Err(McpError::InvalidConfig(
            "display name cannot be empty".to_string(),
        ));
    }
    if timeout_ms == 0 {
        return Err(McpError::InvalidConfig(
            "timeout_ms must be greater than zero".to_string(),
        ));
    }
    transport.validate()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum McpTransportConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, McpEnvValue>,
    },
    StreamableHttp {
        url: String,
        headers: BTreeMap<String, McpEnvValue>,
    },
}

impl McpTransportConfig {
    #[must_use]
    pub fn stdio(command: impl Into<String>) -> Self {
        Self::Stdio {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
        }
    }

    fn validate(&self) -> Result<(), McpError> {
        match self {
            Self::Stdio { command, .. } if command.trim().is_empty() => Err(
                McpError::InvalidConfig("stdio command cannot be empty".to_string()),
            ),
            Self::StreamableHttp { url, .. }
                if !(url.starts_with("http://") || url.starts_with("https://")) =>
            {
                Err(McpError::InvalidConfig(
                    "streamable http url must start with http:// or https://".to_string(),
                ))
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum McpEnvValue {
    Literal(String),
    SecretRef(String),
}
