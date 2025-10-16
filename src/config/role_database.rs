use super::role::RoleValidate;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Role Database Level.
///
/// For example:
///
/// ```yaml
/// - name: role_database_level
///   type: database
///   grants:
///     - CREATE
///     - TEMP
///   databases:
///     - db1
///     - db2
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RoleDatabaseLevel {
    pub name: String,
    pub grants: Vec<String>,
    pub databases: Vec<String>,
}

impl RoleDatabaseLevel {
    /// Escape and quote a PostgreSQL identifier to prevent SQL injection
    fn escape_identifier(ident: &str) -> String {
        // PostgreSQL identifiers are quoted with double quotes
        // Escape double quotes by doubling them
        format!("\"{}\"", ident.replace("\"", "\"\""))
    }

    /// Generate role database to SQL.
    ///
    /// ```sql
    /// { GRANT | REVOKE } { { CREATE | TEMPORARY | TEMP } [,...] | ALL [ PRIVILEGES ] }
    /// ON DATABASE db_name [, ...]
    /// TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
    /// ```
    pub fn to_sql(&self, user: &str) -> String {
        // grant all if no grants specified or contains "ALL"
        let grants = if self.grants.is_empty() || self.grants.contains(&"ALL".to_string()) {
            "ALL PRIVILEGES".to_string()
        } else {
            self.grants.join(", ")
        };

        // escape database and user identifiers to prevent SQL injection
        let escaped_databases = self
            .databases
            .iter()
            .map(|db| Self::escape_identifier(db))
            .collect::<Vec<_>>()
            .join(", ");
        let escaped_user = Self::escape_identifier(user);

        // grant on databases to user
        let sql = format!(
            "GRANT {} ON DATABASE {} TO {};",
            grants, escaped_databases, escaped_user
        );

        sql
    }
}

impl RoleValidate for RoleDatabaseLevel {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("role name is empty"));
        }

        if self.databases.is_empty() {
            return Err(anyhow!("role databases is empty"));
        }

        // Check valid grants: CREATE, TEMP, TEMPORARY, ALL
        let valid_grants = vec!["CREATE", "TEMP", "TEMPORARY", "ALL"];
        let mut grants = HashSet::new();
        for grant in &self.grants {
            if !valid_grants.contains(&&grant[..]) {
                return Err(anyhow!(
                    "invalid grant: {}, expected: {:?}",
                    grant,
                    valid_grants
                ));
            }
            grants.insert(grant.to_string());
        }

        if self.grants.is_empty() {
            return Err(anyhow!("role grants is empty"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_database_level() {
        let role = RoleDatabaseLevel {
            name: "role_database_level".to_string(),
            grants: vec!["CREATE".to_string(), "TEMP".to_string()],
            databases: vec!["db1".to_string(), "db2".to_string()],
        };

        assert!(role.validate().is_ok());
        assert_eq!(
            role.to_sql("user"),
            "GRANT CREATE, TEMP ON DATABASE \"db1\", \"db2\" TO \"user\";"
        );
    }

    #[test]
    fn test_sql_injection_prevention() {
        let role = RoleDatabaseLevel {
            name: "test".to_string(),
            grants: vec!["CREATE".to_string()],
            databases: vec!["db1\"; DROP DATABASE postgres; --".to_string()],
        };

        let sql = role.to_sql("user\"; DROP USER postgres; --");
        // Verify the injection is properly escaped
        assert!(sql.contains("\"db1\"\"; DROP DATABASE postgres; --\""));
        assert!(sql.contains("\"user\"\"; DROP USER postgres; --\""));
    }
}
