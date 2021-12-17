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
    // { GRANT | REVOKE } { { CREATE | TEMPORARY | TEMP } [,...] | ALL [ PRIVILEGES ] }
    // ON DATABASE db_name [, ...]
    // TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
    pub fn to_sql(&self, user: &str) -> String {
        // grant all if no grants specified or contains "ALL"
        let grants = if self.grants.is_empty() || self.grants.contains(&"ALL".to_string()) {
            "ALL PRIVILEGES".to_string()
        } else {
            self.grants.join(", ")
        };

        // grant on databases to user
        let sql = format!(
            "GRANT {} ON DATABASE {} TO {};",
            grants,
            self.databases.join(", "),
            user
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

// Test
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
            "GRANT CREATE, TEMP ON DATABASE db1, db2 TO user;"
        );
    }
}
