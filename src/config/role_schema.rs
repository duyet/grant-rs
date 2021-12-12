use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Role Schema Level.
///
/// For example:
///
/// ```yaml
/// - name: role_schema_level
///   type: SCHEMA
///   grants:
///     - CREATE
///     - TEMP
///   schemas:
///     - schema1
///     - schema2
/// ```
///
///  The above example will grant CREATE and TEMP privileges on schema1 and schema2.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleSchemaLevel {
    pub name: String,
    pub grants: Vec<String>,
    pub schemas: Vec<String>,
}

impl RoleSchemaLevel {
    // { GRANT | REVOKE } { { CREATE | USAGE } [,...] | ALL [ PRIVILEGES ] }
    // ON SCHEMA schema_name [, ...]
    // TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
    pub fn to_sql(&self, user: String, grant: bool) -> String {
        let sql = if grant { "GRANT" } else { "REVOKE" };
        let from_to = if grant { "TO" } else { "FROM" };

        // grant all privileges if no grants are specified or if grants contains "ALL"
        let grants = if self.grants.is_empty() || self.grants.contains(&"ALL".to_string()) {
            "ALL PRIVILEGES".to_string()
        } else {
            self.grants.join(", ")
        };

        // grant on schemas to user
        let sql = format!(
            "{} {} ON SCHEMA {} {} {};",
            sql,
            grants,
            self.schemas.join(", "),
            from_to,
            user
        );

        sql
    }

    pub fn to_sql_grant(&self, user: String) -> String {
        self.to_sql(user, true)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("role name is empty"));
        }

        if self.schemas.is_empty() {
            return Err(anyhow!("role schemas is empty"));
        }

        // Check valid grants: CREATE, USAGE, ALL
        let valid_grants = vec!["CREATE", "USAGE", "ALL"];
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
    fn test_role_schema_level() {
        let role_schema_level = RoleSchemaLevel {
            name: "role_schema_level".to_string(),
            grants: vec!["CREATE".to_string(), "TEMP".to_string()],
            schemas: vec!["schema1".to_string(), "schema2".to_string()],
        };

        role_schema_level.validate().ok();

        let sql = role_schema_level.to_sql_grant("user".to_string());
        assert_eq!(
            sql,
            "GRANT CREATE, TEMP ON SCHEMA schema1, schema2 TO user;"
        );
    }
}
