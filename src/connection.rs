use crate::config::{Config, ConnectionType};
use anyhow::{anyhow, Result};
use log::{debug, info};
use postgres::{row::Row, types::ToSql, Client, Config as ConnConfig, NoTls, ToStatement};

// TODO: support multiple adapters

/// Connection to the database, currently only Postgres and Redshift is supported
/// TODO: support multiple adapters
pub struct DbConnection {
    pub connection_info: String,
    pub client: Client,
    conn_config: ConnConfig,
}

/// Presentation for a user in the database
#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub user_createdb: bool,
    pub user_super: bool,
    pub password: String,
}

/// Presentation for a user database privilege in the database
/// which a users has `create` or `temp` on database
#[derive(Debug)]
pub struct UserDatabaseRole {
    pub name: String,
    pub database_name: String,
    pub has_create: bool,
    pub has_temp: bool,
}

impl UserDatabaseRole {
    pub fn perm_to_string(&self, with_name: bool) -> String {
        if with_name {
            return format!("{}({})", self.database_name, self.perm_to_string(false));
        }

        match (self.has_create, self.has_temp) {
            (true, true) => "A".to_string(),
            (true, false) => "C".to_string(),
            (false, true) => "T".to_string(),
            (false, false) => "".to_string(),
        }
    }
}

/// Presentation for a user schema privilege in the database
/// which a users has `create` or `usage` on schema
#[derive(Debug)]
pub struct UserSchemaRole {
    pub name: String,
    pub schema_name: String,
    pub has_create: bool,
    pub has_usage: bool,
}

impl UserSchemaRole {
    pub fn perm_to_string(&self, with_name: bool) -> String {
        if with_name {
            return format!("{}({})", self.schema_name, self.perm_to_string(false));
        }

        match (self.has_create, self.has_usage) {
            (true, true) => "A".to_string(),
            (true, false) => "C".to_string(),
            (false, true) => "U".to_string(),
            _ => "".to_string(),
        }
    }
}

/// Presentation for a user table privilege in the database
/// which a users has `select`, `insert`, `update`, `delete` or `reference` on table
#[derive(Debug)]
pub struct UserTableRole {
    pub name: String,
    pub schema_name: String,
    pub table_name: String,
    pub has_select: bool,
    pub has_insert: bool,
    pub has_update: bool,
    pub has_delete: bool,
    pub has_references: bool,
}

impl UserTableRole {
    pub fn perm_to_string(&self, with_name: bool) -> String {
        if with_name {
            return format!(
                "{}.{}({})",
                self.schema_name,
                self.table_name,
                self.perm_to_string(false)
            );
        }

        if self.has_select
            && self.has_insert
            && self.has_update
            && self.has_delete
            && self.has_references
        {
            return "A".to_string();
        }

        let has_select = if self.has_select { "S" } else { "" };
        let has_insert = if self.has_insert { "I" } else { "" };
        let has_update = if self.has_update { "U" } else { "" };
        let has_delete = if self.has_delete { "D" } else { "" };
        let has_references = if self.has_references { "R" } else { "" };
        format!(
            "{}{}{}{}{}",
            has_select, has_insert, has_update, has_delete, has_references
        )
    }
}

impl DbConnection {
    /// A convenience function which store the connection string into `connection_info` and then connects to the database.
    ///
    /// Refer to <https://rust-lang-nursery.github.io/rust-cookbook/database/postgres.html>
    /// for more information.
    ///
    /// ```rust
    /// use grant::{config::Config, connection::DbConnection};
    /// use std::str::FromStr;
    ///
    /// let config = Config::from_str(
    ///     r#"
    ///       connection:
    ///         type: postgres
    ///         url: "postgresql://postgres:postgres@localhost:5432/postgres"
    ///       roles: []
    ///       users: []
    ///     "#,
    ///    )
    ///    .unwrap();
    ///    let mut db = DbConnection::new(&config)?;
    ///    db.query("SELECT 1", &[]).unwrap();
    /// ```
    pub fn new(config: &Config) -> Result<Self> {
        match config.connection.type_ {
            ConnectionType::Postgres => {
                let connection_info = config.connection.url.clone();
                let mut client = Client::connect(&connection_info, NoTls)
                    .map_err(|e| anyhow!("Failed to connect to database '{}': {}", connection_info, e))?;

                if let Err(e) = client.simple_query("SELECT 1") {
                    return Err(anyhow!("Database connection test failed for '{}': {}", connection_info, e));
                } else {
                    info!("Connected to database: {}", connection_info);
                }

                let conn_config = connection_info.parse::<ConnConfig>()
                    .map_err(|e| anyhow!("Failed to parse connection string '{}': {}", connection_info, e))?;

                Ok(DbConnection {
                    connection_info,
                    client,
                    conn_config,
                })
            }
        }
    }

    /// Get current database name.
    pub fn get_current_database(&self) -> Option<&str> {
        self.conn_config.get_dbname()
    }

    /// Returns the connection_info
    ///
    /// ```rust
    /// use grant::connection::DbConnection;
    /// use std::str::FromStr;
    ///
    /// let connection_info = "postgres://postgres:postgres@localhost:5432/postgres";
    /// let mut client = DbConnection::from_str(connection_info).unwrap();
    /// assert_eq!(client.connection_info(), "postgres://postgres:postgres@localhost:5432/postgres");
    /// ```
    pub fn connection_info(self) -> String {
        self.connection_info
    }

    /// Get the list of users
    pub fn get_users(&mut self) -> Result<Vec<User>> {
        let mut users = vec![];

        // TODO: Get the password from database, currently it only returns *****
        let sql = "SELECT usename, usecreatedb, usesuper, passwd FROM pg_user";
        let stmt = self.client.prepare(sql).unwrap();

        debug!("executing: {}", sql);
        let rows = self.client.query(&stmt, &[]).unwrap();

        for row in rows {
            match (row.get(0), row.get(1), row.get(2), row.get(3)) {
                (Some(name), Some(user_createdb), Some(user_super), Some(password)) => {
                    users.push(User {
                        name,
                        user_createdb,
                        user_super,
                        password,
                    })
                }
                (Some(name), _, _, _) => users.push(User {
                    name,
                    user_createdb: false,
                    user_super: false,
                    password: String::from(""),
                }),
                (_, _, _, _) => (),
            }
        }

        debug!("get_users: {:#?}", users);

        Ok(users)
    }

    /// Get the current database roles for user `user_name` in current database
    /// Returns a list of `RoleDatabaseLevel`
    pub fn get_user_database_privileges(&mut self) -> Result<Vec<UserDatabaseRole>> {
        let mut roles = vec![];

        let sql = r#"
            WITH db AS (
                SELECT d.datname AS database_name
                FROM pg_database d
            ),
            users AS (
                SELECT usename as user_name FROM pg_user
            )
            SELECT
                u.user_name,
                db.database_name,
                pg_catalog.has_database_privilege(u.user_name, database_name, 'CREATE') AS "create",
                pg_catalog.has_database_privilege(u.user_name, database_name, 'TEMP') AS "temp"
            FROM db CROSS JOIN users u;
        "#;

        let stmt = self.client.prepare(sql).unwrap();

        debug!("executing: {}", sql);
        let rows = self.client.query(&stmt, &[])?;
        for row in rows {
            let name: &str = row.get(0);
            let database_name: &str = row.get(1);
            let has_create: bool = row.get(2);
            let has_temp: bool = row.get(3);

            roles.push(UserDatabaseRole {
                name: name.to_string(),
                database_name: database_name.to_string(),
                has_create,
                has_temp,
            })
        }

        Ok(roles)
    }

    /// Get the user schema privileges for current database
    pub fn get_user_schema_privileges(&mut self) -> Result<Vec<UserSchemaRole>> {
        // FIXME it will be empty if the schema doesn't have any tables
        let sql = "
            SELECT
              u.usename AS name,
              s.schemaname AS schema_name,
              has_schema_privilege(u.usename, s.schemaname, 'create') AS has_create,
              has_schema_privilege(u.usename, s.schemaname, 'usage') AS has_usage
            FROM
              pg_user u
              CROSS JOIN (SELECT DISTINCT schemaname FROM pg_tables) s
            WHERE
              1 = 1
              AND s.schemaname != 'pg_catalog'
              AND s.schemaname != 'information_schema';
        ";

        let stmt = self.client.prepare(sql).unwrap();

        debug!("executing: {}", sql);
        let rows = self.client.query(&stmt, &[])?;
        let mut roles = vec![];
        for row in rows {
            let name = row.get(0);
            let schema_name = row.get(1);
            let has_create = row.get(2);
            let has_usage = row.get(3);
            if let (Some(name), Some(schema_name), Some(has_create), Some(has_usage)) =
                (name, schema_name, has_create, has_usage)
            {
                roles.push(UserSchemaRole {
                    name,
                    schema_name,
                    has_create,
                    has_usage,
                })
            }
        }

        Ok(roles)
    }

    /// Get the user table privileges for current database
    pub fn get_user_table_privileges(&mut self) -> Result<Vec<UserTableRole>> {
        let mut roles = vec![];
        let sql = "
            SELECT
              u.usename AS name,
              t.schemaname AS schema_name,
              t.tablename AS table_name,
              has_table_privilege(u.usename, t.schemaname || '.' || t.tablename, 'select') AS has_select,
              has_table_privilege(u.usename, t.schemaname || '.' || t.tablename, 'insert') AS has_insert,
              has_table_privilege(u.usename, t.schemaname || '.' || t.tablename, 'update') AS has_update,
              has_table_privilege(u.usename, t.schemaname || '.' || t.tablename, 'delete') AS has_delete,
              has_table_privilege(u.usename, t.schemaname || '.' || t.tablename, 'references') AS has_references
            FROM
              pg_user u
              CROSS JOIN (SELECT DISTINCT schemaname, tablename FROM pg_tables) t
              WHERE 1 = 1
                AND t.schemaname NOT LIKE 'pg_%'
                AND t.schemaname != 'information_schema';
        ";

        let stmt = self.client.prepare(sql).unwrap();

        debug!("executing: {}", sql);
        let rows = self.client.query(&stmt, &[])?;
        for row in rows {
            let name = row.get(0);
            let schema_name = row.get(1);
            let table_name = row.get(2);
            let has_select = row.get(3);
            let has_insert = row.get(4);
            let has_update = row.get(5);
            let has_delete = row.get(6);
            let has_references = row.get(7);

            if let (
                Some(name),
                Some(schema_name),
                Some(table_name),
                Some(has_select),
                Some(has_insert),
                Some(has_update),
                Some(has_delete),
                Some(has_references),
            ) = (
                name,
                schema_name,
                table_name,
                has_select,
                has_insert,
                has_update,
                has_delete,
                has_references,
            ) {
                roles.push(UserTableRole {
                    name,
                    schema_name,
                    table_name,
                    has_insert,
                    has_select,
                    has_update,
                    has_delete,
                    has_references,
                })
            }
        }

        Ok(roles)
    }

    /// Executes a statement, returning the resulting rows
    /// A statement may contain parameters, specified by `$n` where `n` is the
    /// index of the parameter in the list provided, 1-indexed.
    ///
    /// ```rust
    /// use grant::connection::DbConnection;
    /// use std::str::FromStr;
    ///
    /// let url = "postgresql://postgres:postgres@localhost:5432/postgres";
    /// let mut db = DbConnection::from_str(url).unwrap();
    /// let rows = db.query("SELECT 1 as t", &[]).unwrap();
    /// println!("test_query: {:?}", rows);
    ///
    /// assert_eq!(rows.len(), 1);
    /// assert_eq!(rows.get(0).unwrap().len(), 1);
    ///
    /// let t: i32 = rows.get(0).unwrap().get("t");
    /// assert_eq!(t, 1);
    /// ```
    pub fn query<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>>
    where
        T: ?Sized + ToStatement,
    {
        let ri = self.client.query(query, params)?;
        Ok(ri)
    }

    /// Executes a statement, returning the number of rows modified.
    ///
    /// If the statement does not modify any rows (e.g. SELECT), 0 is returned.
    ///
    /// ```rust
    /// use grant::connection::DbConnection;
    /// use std::str::FromStr;
    ///
    /// let url = "postgresql://postgres:postgres@localhost:5432/postgres";
    /// let mut db = DbConnection::from_str(url).unwrap();
    /// let nrows = db.execute("SELECT 1 as t", &[]).unwrap();
    ///
    /// println!("test_execute: {:?}", nrows);
    /// assert_eq!(nrows, 1);
    /// ```
    pub fn execute(&mut self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<i64> {
        // Support multiple query statements by splitting on semicolons
        // and executing each one separately (if any)
        // This is a bit of a hack, but it's the only way to support
        // multiple statements in the execute method without having
        // to rewrite the entire method
        // should split params into multiple slices as well
        let queries = query.split(';');
        let mut rows_affected = 0;

        for query in queries {
            let query = query.trim();
            if query.is_empty() {
                continue;
            }

            let stmt = self.client.prepare(query)?;
            let rows = self.client.execute(&stmt, params)?;
            rows_affected += rows;
        }

        rows_affected.try_into()
            .map_err(|e| anyhow!("Row count {} exceeds i64::MAX: {}", rows_affected, e))
    }
}

impl std::str::FromStr for DbConnection {
    type Err = anyhow::Error;

    /// Connection by a connection string.
    ///
    /// ```
    /// use grant::connection::DbConnection;
    /// use std::str::FromStr;
    ///
    /// let connection_info = "postgres://postgres:postgres@localhost:5432/postgres";
    /// let mut client = DbConnection::from_str(connection_info).unwrap();
    /// client.query("SELECT 1", &[]).unwrap();
    /// ```
    fn from_str(connection_info: &str) -> Result<Self> {
        let client = Client::connect(connection_info, NoTls).unwrap();
        let conn_config = connection_info.parse::<ConnConfig>().unwrap();

        Ok(Self {
            connection_info: connection_info.to_owned(),
            client,
            conn_config,
        })
    }
}

// Test DbConnection
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::str::FromStr;

    fn drop_user(db: &mut DbConnection, name: &str) {
        let sql = &format!("DROP USER IF EXISTS {}", name);
        db.execute(sql, &[]).unwrap();
    }

    fn create_user(db: &mut DbConnection, user: &User) {
        let mut sql = format!("CREATE USER {} ", user.name);
        if user.user_createdb {
            sql += "CREATEDB"
        }
        if !user.password.is_empty() {
            sql += &format!(" PASSWORD '{}'", user.password)
        }

        db.execute(&sql, &[]).unwrap();
    }

    #[test]
    fn test_drop_user() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::from_str(url).unwrap();

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };

        drop_user(&mut db, &name);
        create_user(&mut db, &user);
        drop_user(&mut db, &name);

        let users = db.get_users().unwrap_or_default();
        assert_eq!(users.iter().any(|u| u.name == name), false);

        // Clean up
        drop_user(&mut db, &name);
    }

    #[test]
    fn test_drop_create_user() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::from_str(url).unwrap();

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        drop_user(&mut db, &name);
        create_user(&mut db, &user);

        let users = db.get_users().unwrap();

        assert_eq!(users.iter().any(|u| u.name == name), true);

        // Clean up
        drop_user(&mut db, &name);
    }

    #[test]
    fn test_get_schema_roles() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::from_str(url).unwrap();

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        drop_user(&mut db, &name);
        create_user(&mut db, &user);

        // get user roles
        let user_schema_privileges = db.get_user_schema_privileges().unwrap_or_default();

        // FIXME it will be empty if the schema doesn't have any tables
        if !user_schema_privileges.is_empty() {
            // new user, that user will don't have any priviledge
            assert_eq!(
                user_schema_privileges
                    .iter()
                    .any(|u| u.name == name && !u.has_usage && !u.has_create),
                true
            );
        }

        // Clean up
        drop_user(&mut db, &name);
    }

    // Test get_user_database_privileges
    #[test]
    fn test_get_user_database_privileges() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::from_str(url).unwrap();

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        drop_user(&mut db, &name);
        create_user(&mut db, &user);

        // get user roles
        let user_database_privileges = db.get_user_database_privileges().unwrap_or_default();

        // Check if user_database_privileges contains current users
        // is empty if the user doesn't have any database privileges
        assert_eq!(
            user_database_privileges
                .iter()
                .any(|u| u.name == name && u.has_create),
            false
        );

        // FIXME seriously test this function

        // Clean up
        drop_user(&mut db, &name);
    }

    // Test get_user_schema_privileges
    #[test]
    fn test_get_user_schema_privileges() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::from_str(url).unwrap();

        let name = random_str();
        let password = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password,
        };
        drop_user(&mut db, &name);
        create_user(&mut db, &user);

        // get user roles
        let user_schema_privileges = db.get_user_schema_privileges().unwrap_or_default();
        println!("{:?}", user_schema_privileges);

        // Check if user_schema_privileges contains current users
        // is empty if the user doesn't have any schema privileges
        // assert_eq!(user_schema_privileges.iter().any(|u| u.name == name), false);

        // FIXME seriously test this function

        // Clean up
        drop_user(&mut db, &name);
    }

    // Test get_user_tables_privileges
    #[test]
    fn test_get_user_table_privileges() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::from_str(url).unwrap();

        let name = random_str();
        let password = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password,
        };
        drop_user(&mut db, &name);
        create_user(&mut db, &user);

        // get user roles
        let user_table_privileges = db.get_user_table_privileges().unwrap_or_default();

        // Check if user_tables_privileges contains current users
        // is empty if the user doesn't have any tables privileges
        assert_eq!(
            user_table_privileges
                .iter()
                .any(|u| u.name == name && u.has_select),
            false
        );

        // FIXME seriously test this function

        // Clean up
        drop_user(&mut db, &name);
    }

    fn random_str() -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        let mut rng = thread_rng();

        let name: String = (0..10)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        name
    }
}
