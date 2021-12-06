use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashSet;
use std::path::PathBuf;
use std::{fmt, fs};

/// Connection type. Supported values: Postgres
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectionType {
    #[serde(rename = "postgres")]
    Postgres,
}

// Connection configuration section.
// The user on the connection should have the permission to grant privileges.
// For example:
//
// ```yaml
// connection:
//  type: postgres
//  url: postgres://user:password@host:port/database
// ````
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    #[serde(rename = "type")]
    pub type_: ConnectionType,
    pub url: String,
}

impl Connection {
    pub fn new(type_: ConnectionType, url: String) -> Self {
        Self { type_, url }
    }

    pub fn validate(&self) -> Result<()> {
        match self.type_ {
            ConnectionType::Postgres => Ok(()),
        }
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

impl RoleLevelType {
    pub fn validate(&self) -> Result<()> {
        match self {
            RoleLevelType::Database => Ok(()),
            RoleLevelType::Schema => Ok(()),
            RoleLevelType::Table => Ok(()),
        }
    }

    pub fn get_level_type(&self) -> String {
        match self {
            RoleLevelType::Database => "database".to_string(),
            RoleLevelType::Schema => "schema".to_string(),
            RoleLevelType::Table => "table".to_string(),
        }
    }

    pub fn is_none(&self) -> bool {
        false
    }
}

// a Role config item, for example:
//
// ```yaml
// - name: role_database_level
//   type: database
//   databases:
//     - db1
//     - db2
//   grants:
//     - CREATE
//     - TEMP
// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleDatabaseLevel {
    pub name: String,
    pub databases: Vec<String>,
    pub grants: Vec<String>,
}

impl RoleDatabaseLevel {
    // { GRANT | REVOKE } { { CREATE | TEMPORARY | TEMP } [,...] | ALL [ PRIVILEGES ] }
    // ON DATABASE db_name [, ...]
    // TO { username [ WITH GRANT OPTION ] | GROUP group_name | PUBLIC } [, ...]
    pub fn to_sql(&self, user: String, grant: bool) -> String {
        let sql = if grant { "GRANT" } else { "REVOKE" };
        let from_to = if grant { "TO" } else { "FROM" };

        // grant all if no grants specified or contains "ALL"
        let grants = if self.grants.is_empty() || self.grants.contains(&"ALL".to_string()) {
            "ALL PRIVILEGES".to_string()
        } else {
            self.grants.join(", ")
        };

        // grant on databases to user
        let sql = format!(
            "{} {} ON DATABASE {} {} {};",
            sql,
            grants,
            self.databases.join(", "),
            from_to,
            user
        );

        sql
    }

    pub fn to_sql_grant(&self, user: String) -> String {
        self.to_sql(user, true)
    }

    pub fn to_sql_revoke(&self, user: String) -> String {
        self.to_sql(user, false)
    }

    pub fn validate(&self) -> Result<()> {
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

// a Role config item, for example:
//
// ```yaml
// - name: role_schema_level
//  type: SCHEMA
//  grants:
//  - CREATE
//  - TEMP
//  schemas:
//  - schema1
//  - schema2
//  ```
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

    pub fn to_sql_revoke(&self, user: String) -> String {
        self.to_sql(user, false)
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

// a Role config item
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoleTableLevel {
    pub name: String,
    pub grants: Vec<String>,
    pub schemas: Vec<String>,
    pub tables: Vec<String>,
}

impl RoleTableLevel {
    // {GRANT | REVOKE} { { SELECT | INSERT | UPDATE | DELETE | DROP | REFERENCES } [,...] | ALL [ PRIVILEGES ] }
    // ON { [ TABLE ] table_name [, ...] | ALL TABLES IN SCHEMA schema_name [, ...] }
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

        // grant to all tables if no tables are specified or if tables contains "ALL"
        let tables = if self.tables.is_empty() || self.tables.contains(&"ALL".to_string()) {
            format!("ALL TABLES IN SCHEMA {}", self.schemas.join(", "))
        } else {
            self.schemas
                .iter()
                .map(|s| {
                    self.tables
                        .iter()
                        .map(|t| format!("{}.{}", s, t))
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        let sql = format!("{} {} ON {} {} {};", sql, grants, tables, from_to, user);

        sql
    }

    pub fn to_sql_grant(&self, user: String) -> String {
        self.to_sql(user, true)
    }

    pub fn to_sql_revoke(&self, user: String) -> String {
        self.to_sql(user, false)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("role name is empty"));
        }

        if self.schemas.is_empty() {
            return Err(anyhow!("role schemas is empty"));
        }

        if self.tables.is_empty() {
            return Err(anyhow!("role tables is empty"));
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub password: String,
    pub roles: Vec<String>,
}

impl User {
    pub fn to_sql_create(&self) -> String {
        let sql = format!(
            "CREATE USER {} WITH PASSWORD '{}';",
            self.name, self.password
        );
        sql
    }

    pub fn to_sql_drop(&self) -> String {
        let sql = format!("DROP USER IF EXISTS {};", self.name);
        sql
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("user name is empty"));
        }

        if self.password.is_empty() {
            return Err(anyhow!("user password is empty"));
        }

        Ok(())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_password(&self) -> String {
        self.password.clone()
    }

    pub fn get_roles(&self) -> Vec<String> {
        self.roles.clone()
    }
}

// Role type.
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
    pub fn to_sql(&self, user: String, grant: bool) -> String {
        match self {
            Role::Database(role) => role.to_sql(user, grant),
            Role::Schema(role) => role.to_sql(user, grant),
            Role::Table(role) => role.to_sql(user, grant),
        }
    }

    pub fn to_sql_grant(&self, user: String) -> String {
        self.to_sql(user, true)
    }

    pub fn to_sql_revoke(&self, user: String) -> String {
        self.to_sql(user, false)
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

// Config file structure
// Role can be either RoleDatabaseLevel or RoleSchemaLevel
// for example:
// ```yaml
// connection:
//   type: postgres
//   url: postgres://user:password@host:port/database
//
// roles:
//   - name: role_database_level
//     type: databases
//     grants:
//     - CREATE
//     - TEMP
//     databases:
//     - db1
//     - db2
//     - db3
//  users:
//  - name: user1
//    password: password1
//    roles:
//    - role_database_level
//    - role_schema_level
//    - role_table_level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub connection: Connection,
    pub roles: Vec<Role>,
    pub users: Vec<User>,
}

impl Config {
    pub fn new(config_path: &PathBuf) -> Result<Self> {
        let config_str = fs::read_to_string(&config_path).context("failed to read config file")?;
        let config: Config =
            serde_yaml::from_str(&config_str).context("failed to parse yaml content")?;

        config.validate()?;

        Ok(config)
    }

    pub fn from_str(config_str: &str) -> Result<Self> {
        let config: Config =
            serde_yaml::from_str(config_str).context("failed to parse yaml content")?;

        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate connection
        self.connection.validate()?;

        // Validate roles
        for role in &self.roles {
            role.validate()?;
        }
        // Validate role name are unique by name
        let mut role_names = HashSet::new();
        for role in &self.roles {
            if role_names.contains(&role.get_name()) {
                return Err(anyhow!("duplicated role name: {}", role.get_name()));
            }
            role_names.insert(role.get_name());
        }

        // Validate users
        for user in &self.users {
            user.validate()?;
        }
        // Validate users are unique by name
        let mut user_names: HashSet<String> = HashSet::new();
        for user in &self.users {
            if user_names.contains(&user.name) {
                return Err(anyhow!("duplicated user: {}", user.name));
            }
            user_names.insert(user.name.clone());
        }
        // Validate users roles are available in roles
        for user in &self.users {
            for role in &user.roles {
                if !self.roles.iter().any(|r| r.get_name() == role.to_string()) {
                    return Err(anyhow!("user role {} is not available", role));
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_yaml::to_string(&self).unwrap())
    }
}

// Implement default values for Config
impl Default for Config {
    fn default() -> Self {
        Config {
            connection: Connection::default(),
            roles: vec![],
            users: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    #[should_panic(expected = "failed to parse yaml")]
    fn test_with_basic_config() {
        let _text = "bad yaml content";
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to get content");
    }

    // Test config with minimum valid YAML
    #[test]
    fn test_read_config_basic_config() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles: []
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to get content");
    }

    // Test config with invalid connection type
    #[test]
    #[should_panic(expected = "connection.type: unknown variant `invalid`")]
    fn test_read_config_invalid_connection_type() {
        let _text = indoc! {"
                 connection:
                   type: invalid
                   url: postgres://postgres@localhost:5432/postgres
                 roles: []
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to parse config");
    }

    // Test config with role database level
    #[test]
    fn test_read_config_one_role_database_level() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: database
                   name: role_database_level_1
                   grants:
                   - CREATE
                   - TEMP
                   databases:
                   - db1
                   - db2
                   - db3
                 - type: database
                   name: role_database_level_2
                   grants:
                   - ALL
                   databases:
                   - db1
                   - db2
                   - db3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to parse config");
        assert_eq!(config.roles.len(), 2);

        // Test role 1
        assert_eq!(config.roles[0].get_name(), "role_database_level_1");
        assert_eq!(config.roles[0].get_level(), RoleLevelType::Database);
        assert_eq!(config.roles[0].get_grants().len(), 2);
        assert_eq!(config.roles[0].get_grants()[0], "CREATE");
        assert_eq!(config.roles[0].get_grants()[1], "TEMP");
        assert_eq!(config.roles[0].get_databases().len(), 3);
        assert_eq!(config.roles[0].get_databases()[0], "db1");
        assert_eq!(config.roles[0].get_databases()[1], "db2");
        assert_eq!(config.roles[0].get_databases()[2], "db3");
        assert_eq!(
            config.roles[0].to_sql_grant("duyet".to_string()),
            "GRANT CREATE, TEMP ON DATABASE db1, db2, db3 TO duyet;".to_string()
        );
        assert_eq!(
            config.roles[0].to_sql_revoke("duyet".to_string()),
            "REVOKE CREATE, TEMP ON DATABASE db1, db2, db3 FROM duyet;".to_string()
        );

        // Test role 2
        assert_eq!(config.roles[1].get_name(), "role_database_level_2");
        assert_eq!(config.roles[1].get_level(), RoleLevelType::Database);
        assert_eq!(config.roles[1].get_grants().len(), 1);
        assert_eq!(config.roles[1].get_grants()[0], "ALL");
        assert_eq!(config.roles[1].get_databases().len(), 3);
        assert_eq!(config.roles[1].get_databases()[0], "db1");
        assert_eq!(config.roles[1].get_databases()[1], "db2");
        assert_eq!(config.roles[1].get_databases()[2], "db3");
        assert_eq!(
            config.roles[1].to_sql_grant("duyet".to_string()),
            "GRANT ALL PRIVILEGES ON DATABASE db1, db2, db3 TO duyet;".to_string()
        );
        assert_eq!(
            config.roles[1].to_sql_revoke("duyet".to_string()),
            "REVOKE ALL PRIVILEGES ON DATABASE db1, db2, db3 FROM duyet;".to_string()
        );
    }

    // Test config role type database level with invalid grants
    #[test]
    #[should_panic(expected = "invalid grant: invalid")]
    fn test_read_config_role_type_database_level_invalid_grants() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: database
                   name: role_database_level
                   grants:
                   - invalid
                   databases:
                   - db1
                   - db2
                   - db3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to parse config");
    }

    // Test config with role schema level
    #[test]
    fn test_read_config_one_role_schema_level() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: schema
                   name: role_schema_level_1
                   grants:
                   - CREATE
                   - USAGE
                   schemas:
                   - schema1
                   - schema2
                   - schema3
                 - type: schema
                   name: role_schema_level_2
                   grants:
                   - ALL
                   schemas:
                   - schema1
                   - schema2
                   - schema3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to parse config");
        assert_eq!(config.roles.len(), 2);

        // Test role 1
        assert_eq!(config.roles[0].get_name(), "role_schema_level_1");
        assert_eq!(config.roles[0].get_level(), RoleLevelType::Schema);
        assert_eq!(config.roles[0].get_grants().len(), 2);
        assert_eq!(config.roles[0].get_grants()[0], "CREATE");
        assert_eq!(config.roles[0].get_grants()[1], "USAGE");
        assert_eq!(config.roles[0].get_schemas().len(), 3);
        assert_eq!(config.roles[0].get_schemas()[0], "schema1");
        assert_eq!(config.roles[0].get_schemas()[1], "schema2");
        assert_eq!(config.roles[0].get_schemas()[2], "schema3");
        assert_eq!(
            config.roles[0].to_sql_grant("duyet".to_string()),
            "GRANT CREATE, USAGE ON SCHEMA schema1, schema2, schema3 TO duyet;".to_string()
        );
        assert_eq!(
            config.roles[0].to_sql_revoke("duyet".to_string()),
            "REVOKE CREATE, USAGE ON SCHEMA schema1, schema2, schema3 FROM duyet;".to_string()
        );

        // Test role 2
        assert_eq!(config.roles[1].get_name(), "role_schema_level_2");
        assert_eq!(config.roles[1].get_level(), RoleLevelType::Schema);
        assert_eq!(config.roles[1].get_grants().len(), 1);
        assert_eq!(config.roles[1].get_grants()[0], "ALL");
        assert_eq!(config.roles[1].get_schemas().len(), 3);
        assert_eq!(config.roles[1].get_schemas()[0], "schema1");
        assert_eq!(config.roles[1].get_schemas()[1], "schema2");
        assert_eq!(config.roles[1].get_schemas()[2], "schema3");
        assert_eq!(
            config.roles[1].to_sql_grant("duyet".to_string()),
            "GRANT ALL PRIVILEGES ON SCHEMA schema1, schema2, schema3 TO duyet;".to_string()
        );
        assert_eq!(
            config.roles[1].to_sql_revoke("duyet".to_string()),
            "REVOKE ALL PRIVILEGES ON SCHEMA schema1, schema2, schema3 FROM duyet;".to_string()
        );
    }

    // Test config role type schema level with invalid grants
    #[test]
    #[should_panic(expected = "invalid grant: invalid")]
    fn test_read_config_role_type_schema_level_invalid_grants() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: schema
                   name: role_schema_level
                   grants:
                   - invalid
                   schemas:
                   - schema1
                   - schema2
                   - schema3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to parse config");
    }

    // Test config with one role table level
    #[test]
    fn test_read_config_one_role_table_level() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: table
                   name: role_table_level_1
                   grants:
                   - SELECT
                   - INSERT
                   schemas:
                   - schema1
                   tables:
                   - table1
                   - table2
                   - table3
                 - type: table
                   name: role_table_level_2
                   grants:
                   - ALL
                   schemas:
                   - schema1
                   tables:
                   - table1
                   - table2
                   - table3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to parse config");
        assert_eq!(config.roles.len(), 2);

        // Test role 1
        assert_eq!(config.roles[0].get_name(), "role_table_level_1");
        assert_eq!(config.roles[0].get_level(), RoleLevelType::Table);
        assert_eq!(config.roles[0].get_grants().len(), 2);
        assert_eq!(config.roles[0].get_grants()[0], "SELECT");
        assert_eq!(config.roles[0].get_grants()[1], "INSERT");
        assert_eq!(config.roles[0].get_schemas().len(), 1);
        assert_eq!(config.roles[0].get_schemas()[0], "schema1");
        assert_eq!(config.roles[0].get_tables().len(), 3);
        assert_eq!(config.roles[0].get_tables()[0], "table1");
        assert_eq!(config.roles[0].get_tables()[1], "table2");
        assert_eq!(config.roles[0].get_tables()[2], "table3");
        assert_eq!(
            config.roles[0].to_sql_grant("duyet".to_string()),
            "GRANT SELECT, INSERT ON schema1.table1, schema1.table2, schema1.table3 TO duyet;"
        );
        assert_eq!(
            config.roles[0].to_sql_revoke("duyet".to_string()),
            "REVOKE SELECT, INSERT ON schema1.table1, schema1.table2, schema1.table3 FROM duyet;"
        );

        // Test role 2
        assert_eq!(config.roles[1].get_name(), "role_table_level_2");
        assert_eq!(config.roles[1].get_level(), RoleLevelType::Table);
        assert_eq!(config.roles[1].get_grants().len(), 1);
        assert_eq!(config.roles[1].get_grants()[0], "ALL");
        assert_eq!(config.roles[1].get_schemas().len(), 1);
        assert_eq!(config.roles[1].get_schemas()[0], "schema1");
        assert_eq!(config.roles[1].get_tables().len(), 3);
        assert_eq!(config.roles[1].get_tables()[0], "table1");
        assert_eq!(config.roles[1].get_tables()[1], "table2");
        assert_eq!(config.roles[1].get_tables()[2], "table3");
        assert_eq!(
            config.roles[1].to_sql_grant("duyet".to_string()),
            "GRANT ALL PRIVILEGES ON schema1.table1, schema1.table2, schema1.table3 TO duyet;"
                .to_string()
        );
        assert_eq!(
            config.roles[1].to_sql_revoke("duyet".to_string()),
            "REVOKE ALL PRIVILEGES ON schema1.table1, schema1.table2, schema1.table3 FROM duyet;"
                .to_string()
        );
    }

    // Test config role type table level with invalid grants
    #[test]
    #[should_panic(expected = "invalid grant: invalid")]
    fn test_read_config_role_type_table_level_invalid_grants() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: table
                   name: role_table_level
                   grants:
                   - invalid
                   schemas:
                   - schema1
                   tables:
                   - table1
                   - table2
                   - table3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to parse config");
    }

    // Test two role duplicated name error
    #[test]
    #[should_panic(expected = "duplicated role name: role_table_level")]
    fn test_read_config_two_role_duplicated_name() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: table
                   name: role_table_level
                   grants:
                   - SELECT
                   - INSERT
                   schemas:
                   - schema1
                   tables:
                   - table1
                   - table2
                   - table3
                 - type: table
                   name: role_table_level
                   grants:
                   - ALL
                   schemas:
                   - schema1
                   tables:
                   - table1
                   - table2
                   - table3
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        Config::new(&path).expect("failed to parse config");
    }

    // Test users config
    #[test]
    fn test_read_config_users() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: database
                   name: role_database_level
                   grants:
                   - CREATE
                   - TEMP
                   databases:
                   - db1
                   - db2
                   - db3
                 - type: schema
                   name: role_schema_level
                   grants:
                   - ALL
                   schemas:
                   - schema1
                   - schema2
                   - schema3
                 - type: table
                   name: role_table_level
                   grants:
                   - SELECT
                   - INSERT
                   schemas:
                   - schema1
                   tables:
                   - table1
                   - table2
                   - table3
                 users:
                 - name: duyet
                   password: 123456
                   roles:
                   - role_database_level
                   - role_schema_level
                   - role_table_level
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to parse config");
        assert_eq!(config.users.len(), 1);

        // Test user 1
        assert_eq!(config.users[0].get_name(), "duyet");
        assert_eq!(config.users[0].get_password(), "123456");
        assert_eq!(config.users[0].get_roles().len(), 3);
        assert_eq!(config.users[0].get_roles()[0], "role_database_level");
        assert_eq!(config.users[0].get_roles()[1], "role_schema_level");
        assert_eq!(config.users[0].get_roles()[2], "role_table_level");

        // Test sql create user
        assert_eq!(
            config.users[0].to_sql_create(),
            "CREATE USER duyet WITH PASSWORD '123456';"
        );

        // Test sql drop user
        assert_eq!(config.users[0].to_sql_drop(), "DROP USER IF EXISTS duyet;");
    }
}