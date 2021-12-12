use anyhow::Result;
use envmnt::{ExpandOptions, ExpansionType};
use log::warn;
use serde::{Deserialize, Serialize};

/// Connection type. Supported values: Postgres
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConnectionType {
    #[serde(rename = "postgres")]
    Postgres,
}

/// Connection configuration section.
/// The user on the connection should have the permission to grant privileges.
///
/// For example:
/// ```yaml
/// connection:
///   type: postgres
///   url: postgres://user:password@host:port/database
/// ```
///
/// The connection type is required.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    #[serde(rename = "type")]
    pub type_: ConnectionType,
    pub url: String,
}

impl Connection {
    pub fn validate(&self) -> Result<()> {
        match self.type_ {
            ConnectionType::Postgres => Ok(()),
        }
    }

    // xpaned environtment variables in the `url` field.
    // Expand environment variables in the `url` field.
    // For example: postgres://user:${PASSWORD}@host:port/database
    pub fn expand_env_vars(&self) -> Result<Self> {
        let mut connection = self.clone();

        let options = ExpandOptions {
            expansion_type: Some(ExpansionType::UnixBracketsWithDefaults),
            default_to_empty: false,
        };

        connection.url = envmnt::expand(&self.url, Some(options));

        // Warning if still have environment variables in the `url` field.
        // Most likely, the user forgot to export the environment variables.
        if connection.url.contains("${") {
            warn!(
                "The connection url may not have fully expanded environment variables: {}",
                connection.url
            );
        }

        Ok(connection)
    }
}

// Implement default values for connection type and url.
impl Default for Connection {
    fn default() -> Self {
        Self {
            type_: ConnectionType::Postgres,
            url: "postgres://postgres:postgres@localhost:5432/postgres".to_string(),
        }
    }
}

// Test Connection.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_validate() {
        let connection = Connection {
            type_: ConnectionType::Postgres,
            url: "postgres://postgres:postgres@localhost:5432/postgres".to_string(),
        };
        assert!(connection.validate().is_ok());
    }

    #[test]
    fn test_connection_expand_env_vars() {
        // backup the original env variables
        let original_env = envmnt::get_or("PASSWORD", "");
        envmnt::set("PASSWORD", "postgres");

        let connection = Connection {
            type_: ConnectionType::Postgres,
            url: "postgres://user:${PASSWORD}@host:port/database".to_string(),
        };
        let expanded_connection = connection.expand_env_vars().unwrap();
        assert_eq!(
            expanded_connection.url,
            "postgres://user:postgres@host:port/database"
        );

        // restore the original env variables
        envmnt::set("PASSWORD", original_env);
    }
}
