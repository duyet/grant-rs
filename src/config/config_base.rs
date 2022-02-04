use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::{fmt, fs};

pub use super::connection::{Connection, ConnectionType};
pub use super::User;
pub use super::{Role, RoleLevelType};

/// Configuration contains all the information needed to connect to a database, the roles and
/// users.
///  - `connection`: the connection to the database, including the type of connection and the URL.
///  - `roles`: the roles of the users. The roles are used to determine the permissions of the
///  users. A role can be a [RoleDatabaseLevel], [RoleSchemaLevel] or [RoleTableLevel].
///  - `users`: the users.
///
/// [RoleDatabaseLevel]: crate::config::role::RoleDatabaseLevel
/// [RoleSchemaLevel]: crate::config::role::RoleSchemaLevel
/// [RoleTableLevel]: crate::config::role::RoleTableLevel
///
/// For example:
///
/// ```yaml
/// connection:
///   type: postgres
///   url: postgres://user:password@host:port/database
///
/// roles:
///   - name: role_database_level
///     type: databases
///     grants:
///     - CREATE
///     - TEMP
///     databases:
///     - db1
///     - db2
///     - db3
///  users:
///  - name: user1
///    password: password1
///    roles:
///    - role_database_level
///    - role_schema_level
///    - role_table_level
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Config {
    pub connection: Connection,
    pub roles: Vec<Role>,
    pub users: Vec<User>,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_yaml::to_string(&self).unwrap())
    }
}

impl std::str::FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let config: Config = serde_yaml::from_str(s)?;

        // Validate
        config.validate()?;

        Ok(config)
    }
}

impl Config {
    pub fn new(config_path: &Path) -> Result<Self> {
        let config_path = config_path.to_path_buf();
        let config_str = fs::read_to_string(&config_path).context("failed to read config file")?;
        let config: Config = serde_yaml::from_str(&config_str)?;

        config.validate()?;

        // expand env variables
        let config = config.expand_env_vars()?;

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
                // role name can contain '-' at the first position
                let role_name = if let Some(without_sign) = role.strip_prefix('-') {
                    without_sign
                } else {
                    role
                };

                if !self.roles.iter().any(|r| r.get_name() == role_name) {
                    return Err(anyhow!("user role {} is not available", role));
                }
            }
        }

        Ok(())
    }

    // Expand env variables in config
    fn expand_env_vars(&self) -> Result<Self> {
        let mut config = self.clone();

        // expand connection
        config.connection = config.connection.expand_env_vars()?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::io::Write;
    use std::path::PathBuf;
    use std::str::FromStr;
    use tempfile::NamedTempFile;

    #[test]
    #[should_panic(expected = "failed to get content: invalid type: string")]
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

    // Test Config::from_str
    #[test]
    fn test_read_config_from_str() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles: []
                 users: []
             "};

        Config::from_str(_text).expect("failed to get content");
    }

    // Config::from_str and Config::new should return the same result
    #[test]
    fn test_read_config_from_str_and_new() {
        let _text = indoc! {"
             connection:
               type: postgres
               url: postgres://localhost:5432/postgres
             roles: []
             users: []
        "};

        let config_1 = Config::from_str(_text).expect("failed to get content");

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());
        let config_2 = Config::new(&path).expect("failed to get content");

        assert_eq!(config_1, config_2);
    }

    // Test config with url contains environement variable
    #[test]
    fn test_read_config_with_env_var() {
        envmnt::set("POSTGRES_HOST", "duyet");

        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://${POSTGRES_HOST}:5432/postgres
                 roles: []
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to get content");

        assert_eq!(config.connection.url, "postgres://duyet:5432/postgres");

        envmnt::remove("POSTGRES_HOST");
    }

    // Test expand environement variables but not available
    #[test]
    fn test_read_config_with_env_var_not_available() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://${POSTGRES_HOST:duyet}:5432/${POSTGRES_ABC}
                 roles: []
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to get content");

        assert_eq!(
            config.connection.url,
            "postgres://duyet:5432/${POSTGRES_ABC}"
        );
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
            config.roles[0].to_sql("duyet"),
            "GRANT CREATE, TEMP ON DATABASE db1, db2, db3 TO duyet;".to_string()
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
            config.roles[1].to_sql("duyet"),
            "GRANT ALL PRIVILEGES ON DATABASE db1, db2, db3 TO duyet;".to_string()
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
            config.roles[0].to_sql("duyet"),
            "GRANT CREATE, USAGE ON SCHEMA schema1, schema2, schema3 TO duyet;".to_string()
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
            config.roles[1].to_sql("duyet"),
            "GRANT ALL PRIVILEGES ON SCHEMA schema1, schema2, schema3 TO duyet;".to_string()
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
            config.roles[0].to_sql("duyet"),
            "GRANT SELECT, INSERT ON schema1.table1, schema1.table2, schema1.table3 TO duyet;"
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
            config.roles[1].to_sql("duyet"),
            "GRANT ALL PRIVILEGES ON schema1.table1, schema1.table2, schema1.table3 TO duyet;"
                .to_string()
        );
    }

    // Test config role type table level with table name is `ALL`
    #[test]
    fn test_read_config_role_type_table_level_all_tables() {
        let _text = indoc! {"
                 connection:
                   type: postgres
                   url: postgres://localhost:5432/postgres
                 roles:
                 - type: table
                   name: role_table_level_1
                   grants:
                     - SELECT
                   schemas:
                     - schema1
                   tables:
                     - ALL
                 - type: table
                   name: role_table_level_2
                   grants:
                     - SELECT
                   schemas:
                     - schema1
                   tables:
                     - ALL
                     - another_table_should_be_included_in_all_too
                 - type: table
                   name: role_table_level_3
                   grants:
                     - SELECT
                   schemas:
                     - schema1
                   tables:
                     - ALL
                     - -but_excluded_me
                 - type: table
                   name: role_table_level_4
                   grants:
                     - SELECT
                   schemas:
                     - schema1
                   tables:
                     - table_a
                     - -table_b
                 - type: table
                   name: role_table_level_5
                   grants:
                     - SELECT
                   schemas:
                     - schema1
                   tables:
                     - -table_a
                     - -table_b
                 - type: table
                   name: role_table_level_6
                   grants:
                     - SELECT
                   schemas:
                     - schema1
                   tables:
                     - -ALL
                 users: []
             "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to parse config");
        assert_eq!(config.roles.len(), 6);

        assert_eq!(
            config.roles[0].to_sql("duyet"),
            "GRANT SELECT ON ALL TABLES IN SCHEMA schema1 TO duyet;"
        );
        assert_eq!(
            config.roles[1].to_sql("duyet"),
            "GRANT SELECT ON ALL TABLES IN SCHEMA schema1 TO duyet;"
        );
        assert_eq!(
            config.roles[2].to_sql("duyet"),
            "GRANT SELECT ON ALL TABLES IN SCHEMA schema1 TO duyet; REVOKE SELECT ON schema1.but_excluded_me FROM duyet;"
        );
        assert_eq!(
            config.roles[3].to_sql("duyet"),
            "GRANT SELECT ON schema1.table_a TO duyet; REVOKE SELECT ON schema1.table_b FROM duyet;"
        );
        assert_eq!(
            config.roles[4].to_sql("duyet"),
            "REVOKE SELECT ON schema1.table_a, schema1.table_b FROM duyet;"
        );
        assert_eq!(
            config.roles[5].to_sql("duyet"),
            "REVOKE SELECT ON ALL TABLES IN SCHEMA schema1 FROM duyet;"
        );
    }

    // Test config role type table level with invalid grants
    #[test]
    #[should_panic(expected = "role.grants invalid")]
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
                   url: postgres://postgres:postgres@localhost:5432/postgres
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
                 - name: duyet_without_password
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
        assert_eq!(config.users.len(), 2);

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

        // Test sql create user without password
        assert_eq!(
            config.users[1].to_sql_create(),
            "CREATE USER duyet_without_password;"
        );

        // Test sql drop user
        assert_eq!(config.users[0].to_sql_drop(), "DROP USER IF EXISTS duyet;");
    }

    // Test users config with revoke role by `-role_name`
    #[test]
    fn test_read_config_users_exclude_role_by_minus_role_name() {
        let _text = indoc! {"
             connection:
               type: postgres
               url: postgres://postgres:postgres@localhost:5432/postgres
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
               - -role_database_level
               - -role_schema_level
               - -role_table_level
             - name: duyet_without_password
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
        assert_eq!(config.users.len(), 2);

        // Test user 1
        assert_eq!(config.users[0].get_name(), "duyet");
        assert_eq!(config.users[0].get_password(), "123456");
        assert_eq!(config.users[0].get_roles().len(), 3);
    }

    /// Test find role by name, name can contains `-` in case of exclude role
    #[test]
    fn test_find_role_by_name() {
        let _text = indoc! {"
             connection:
               type: postgres
               url: postgres://postgres:postgres@localhost:5432/postgres
             roles:
             - type: database
               name: role_database_level
               grants:
               - CREATE
               databases:
               - db1
             users: []
        "};

        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write(_text.as_bytes())
            .expect("failed to write to temp file");
        let path = PathBuf::from(file.path().to_str().unwrap());

        let config = Config::new(&path).expect("failed to parse config");

        assert!(config
            .roles
            .iter()
            .find(|r| r.find("role_database_level"))
            .is_some());

        // Test find role by name with `-`
        assert!(config
            .roles
            .iter()
            .find(|r| r.find("-role_database_level"))
            .is_some());
    }
}
