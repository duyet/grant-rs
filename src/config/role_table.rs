use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Role Table Level.
///
/// For example:
///
/// ```yaml
/// - name: role_table
///   grants:
///     - SELECT
///     - INSERT
///     - UPDATE
///     - DELETE
///   schemas:
///   - public
///   tables:
///     - ALL
///     - +table1
///     - -table2
///     - -public.table2
/// ```
///
/// The above example grants SELECT, INSERT, UPDATE, DELETE to all tables in the public schema
/// except table2.
/// The ALL is a special keyword that means all tables in the public schema.
/// If the table does not have a schema, it is assumed to be in all schema.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleTableLevel {
    pub name: String,
    pub grants: Vec<String>,
    pub schemas: Vec<String>,
    pub tables: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Table {
    name: String,
    sign: String,
}

impl Table {
    fn new(name: &str) -> Self {
        let sign = match name.chars().nth(0) {
            Some('+') => "+".to_string(),
            Some('-') => "-".to_string(),
            _ => "+".to_string(),
        };
        let name = name.trim_start_matches(&sign).to_string();

        Self { name, sign }
    }
}

impl RoleTableLevel {
    // {GRANT | REVOKE} { { SELECT | INSERT | UPDATE | DELETE | DROP | REFERENCES } [,...] | ALL [ PRIVILEGES ] }
    // ON { [ TABLE ] table_name [, ...] | ALL TABLES IN SCHEMA schema_name [, ...] }
    // TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
    pub fn to_sql(&self, user: String) -> String {
        let mut sqls = vec![];
        let mut tables = self
            .tables
            .iter()
            .map(|t| Table::new(t))
            .collect::<Vec<Table>>();

        // grant all privileges if grants contains "ALL"
        let grants = if self.grants.contains(&"ALL".to_string()) {
            "ALL PRIVILEGES".to_string()
        } else {
            self.grants.join(", ")
        };

        // if `tables` only contains `ALL`
        if let Some(table_named_all) = tables.iter().find(|t| t.name == "ALL") {
            let sql = match table_named_all.sign.as_str() {
                "+" => format!(
                    "GRANT {} ON ALL TABLES IN SCHEMA {} TO {};",
                    grants,
                    self.schemas.join(", "),
                    user
                ),
                "-" => format!(
                    "REVOKE {} ON ALL TABLES IN SCHEMA {} FROM {};",
                    grants,
                    self.schemas.join(", "),
                    user
                ),
                _ => "".to_string(),
            };
            sqls.push(sql);

            // remove name `ALL` and all tables start with `+`
            for table in tables.clone() {
                if table.name == "ALL" || table.sign == "+" {
                    tables.retain(|x| x != &table);
                }
            }
        }

        // grant on tables sign `+`
        let grant_tables = tables.iter().filter(|x| x.sign == "+").collect::<Vec<_>>();
        if grant_tables.len() > 0 {
            let _with_schema = grant_tables
                .iter()
                .flat_map(|t| {
                    if t.name.contains(".") {
                        vec![t.name.clone()]
                    } else {
                        self.schemas
                            .iter()
                            .map(|s| format!("{}.{}", s, &t.name))
                            .collect::<Vec<_>>()
                    }
                })
                .collect::<Vec<String>>()
                .join(", ");

            let sql = format!("GRANT {} ON {} TO {};", grants, _with_schema, user);
            sqls.push(sql);

            // remove all tables start with `+`
            for table in tables.clone() {
                if table.sign == "+" {
                    tables.retain(|x| x != &table);
                }
            }
        }

        // revoke on tables start with `-`
        let revoke_tables = tables.iter().filter(|x| x.sign == "-").collect::<Vec<_>>();
        if revoke_tables.len() > 0 {
            let _with_schema = revoke_tables
                .iter()
                .flat_map(|t| {
                    if t.name.contains(".") {
                        vec![t.name.clone()]
                    } else {
                        self.schemas
                            .iter()
                            .map(|s| format!("{}.{}", s, &t.name))
                            .collect::<Vec<_>>()
                    }
                })
                .collect::<Vec<String>>()
                .join(", ");

            let sql = format!("REVOKE {} ON {} FROM {};", grants, _with_schema, user);
            sqls.push(sql);
        }

        sqls.join(" ")
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("role.name is empty"));
        }

        if self.schemas.is_empty() {
            return Err(anyhow!("role.schemas is empty"));
        }

        // TODO: support schemas=[ALL]
        if self.schemas.contains(&"ALL".to_string()) {
            return Err(anyhow!("role.schemas is not supported yet: ALL"));
        }

        if self.tables.is_empty() {
            return Err(anyhow!("role.tables is empty"));
        }

        if self.grants.is_empty() {
            return Err(anyhow!("role.grants is empty"));
        }

        // Check valid grants: SELECT, INSERT, UPDATE, DELETE, DROP, REFERENCES, ALL
        let valid_grants = vec![
            "SELECT",
            "INSERT",
            "UPDATE",
            "DELETE",
            "DROP",
            "REFERENCES",
            "ALL",
        ];
        let mut grants = HashSet::new();
        for grant in &self.grants {
            if !valid_grants.contains(&&grant[..]) {
                return Err(anyhow!(
                    "role.grants invalid: {}, expected: {:?}",
                    grant,
                    valid_grants
                ));
            }
            grants.insert(grant.to_string());
        }

        Ok(())
    }
}

// Test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_table_level() {
        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string()],
            schemas: vec!["public".to_string()],
            tables: vec!["test".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT ON public.test TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string()],
            tables: vec!["test".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON public.test TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["test".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON public.test, test.test TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["ALL".to_string()],
            schemas: vec!["public".to_string()],
            tables: vec!["test".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT ALL PRIVILEGES ON public.test TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string()],
            tables: vec!["ALL".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON ALL TABLES IN SCHEMA public TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["ALL".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["ALL".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public, test TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["ALL".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON ALL TABLES IN SCHEMA public, test TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["test".to_string(), "test.test2".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON public.test, test.test, test.test2 TO test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["test".to_string(), "-test.test2".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON public.test, test.test TO test; REVOKE SELECT, INSERT ON test.test2 FROM test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["test".to_string(), "-test2".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON public.test, test.test TO test; REVOKE SELECT, INSERT ON public.test2, test.test2 FROM test;"
        );

        let role = RoleTableLevel {
            name: "test".to_string(),
            grants: vec!["SELECT".to_string(), "INSERT".to_string()],
            schemas: vec!["public".to_string(), "test".to_string()],
            tables: vec!["ALL".to_string(), "-test.test2".to_string()],
        };
        assert_eq!(
            role.to_sql("test".to_string()),
            "GRANT SELECT, INSERT ON ALL TABLES IN SCHEMA public, test TO test; REVOKE SELECT, INSERT ON test.test2 FROM test;"
        );
    }
}
