use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

pub use super::role_database::RoleDatabaseLevel;
pub use super::role_schema::RoleSchemaLevel;
pub use super::role_table::RoleTableLevel;

/// Level type for role.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "level")]
pub enum RoleLevelType {
    Database,
    Schema,
    Table,
}

impl fmt::Display for RoleLevelType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RoleLevelType::Database => write!(f, "database"),
            RoleLevelType::Schema => write!(f, "schema"),
            RoleLevelType::Table => write!(f, "table"),
        }
    }
}

/// Configuration for a role.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Role {
    #[serde(rename = "database")]
    Database(RoleDatabaseLevel),
    #[serde(rename = "schema")]
    Schema(RoleSchemaLevel),
    #[serde(rename = "table")]
    Table(RoleTableLevel),
}

impl Role {
    pub fn to_sql(&self, user: String) -> String {
        match self {
            Role::Database(role) => role.to_sql(user, true),
            Role::Schema(role) => role.to_sql(user, true),
            Role::Table(role) => role.to_sql(user),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Role::Database(role) => role.validate(),
            Role::Schema(role) => role.validate(),
            Role::Table(role) => role.validate(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Role::Database(role) => role.name.clone(),
            Role::Schema(role) => role.name.clone(),
            Role::Table(role) => role.name.clone(),
        }
    }

    pub fn find(&self, name: &str) -> bool {
        // role name can contain '-', so we need to remove it before comparing
        let name = name.replace("-", "");

        match self {
            Role::Database(role) => role.name == name,
            Role::Schema(role) => role.name == name,
            Role::Table(role) => role.name == name,
        }
    }

    pub fn get_level(&self) -> RoleLevelType {
        match self {
            Role::Database(_role) => RoleLevelType::Database,
            Role::Schema(_role) => RoleLevelType::Schema,
            Role::Table(_role) => RoleLevelType::Table,
        }
    }

    pub fn get_grants(&self) -> Vec<String> {
        match self {
            Role::Database(role) => role.grants.clone(),
            Role::Schema(role) => role.grants.clone(),
            Role::Table(role) => role.grants.clone(),
        }
    }

    pub fn get_databases(&self) -> Vec<String> {
        match self {
            Role::Database(role) => role.databases.clone(),
            Role::Schema(_) => vec![],
            Role::Table(_) => vec![],
        }
    }

    pub fn get_schemas(&self) -> Vec<String> {
        match self {
            Role::Database(_) => vec![],
            Role::Schema(role) => role.schemas.clone(),
            Role::Table(role) => role.schemas.clone(),
        }
    }

    pub fn get_tables(&self) -> Vec<String> {
        match self {
            Role::Database(_) => vec![],
            Role::Schema(_) => vec![],
            Role::Table(role) => role.tables.clone(),
        }
    }
}
