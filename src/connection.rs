use crate::config::{Config, ConnectionType, RoleDatabaseLevel};
use anyhow::Result;
use log::{debug, error, info};
use postgres::row::Row;
use postgres::types::ToSql;
use postgres::{Client, NoTls, ToStatement};

// TODO: support multiple adapters

pub struct DbConnection {
    pub connection_info: String,
    pub client: Client,
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub user_createdb: bool,
    pub user_super: bool,
    pub password: String,
}

impl User {
    pub fn new(name: String, user_createdb: bool, user_super: bool, password: String) -> User {
        User {
            name,
            user_createdb,
            user_super,
            password,
        }
    }
}

#[derive(Debug)]
pub struct UserDatabaseRole {
    pub name: String,
    pub database_name: String,
    pub has_create: bool,
    pub has_temp: bool,
}

#[derive(Debug)]
pub struct UserSchemaRole {
    pub name: String,
    pub schema_name: String,
    pub has_create: bool,
    pub has_usage: bool,
}

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

impl DbConnection {
    /// A convenience function which store the connection string into `connection_info` and then connects to the database.
    ///
    /// Refer to <https://rust-lang-nursery.github.io/rust-cookbook/database/postgres.html>
    pub fn new(config: &Config) -> Self {
        match config.connection.type_ {
            ConnectionType::Postgres => {
                let connection_info = config.connection.url.clone();
                let mut client = Client::connect(&connection_info, NoTls)
                    .expect("failed to connect to database");

                if let Err(e) = client.simple_query("SELECT 1") {
                    error!("Failed to connect to database: {}", e);
                } else {
                    info!("Connected to database: {}", connection_info);
                }

                DbConnection {
                    connection_info,
                    client,
                }
            }
        }
    }

    /// Connection by a connection string.
    pub fn new_from_string(connection_info: String) -> Self {
        let client = Client::connect(&connection_info, NoTls).unwrap();
        DbConnection {
            connection_info,
            client,
        }
    }

    /// Returns the connection_info
    pub fn connection_info(self) -> String {
        self.connection_info
    }

    /// Drop a user
    pub fn drop_user(&mut self, user: &User) {
        let sql: String = format!("DROP USER IF EXISTS {}", user.name).to_owned();
        debug!("drop_user: {}", sql);

        self.client.execute(&sql, &[]).expect("could not drop user");
    }

    /// Create user
    pub fn create_user(&mut self, user: &User) {
        let mut sql: String = format!("CREATE USER {} ", user.name).to_owned();
        if user.user_createdb {
            sql += "CREATEDB"
        }
        if !user.password.is_empty() {
            sql += &format!(" PASSWORD '{}'", user.password).to_string()
        }

        let stmt = self.client.prepare(&sql).unwrap();

        info!("executing: {}", sql);
        self.client
            .execute(&stmt, &[])
            .expect("could not create user");
    }

    /// Update user password
    pub fn update_user_password(&mut self, user: &User) {
        let sql: String =
            format!("ALTER USER {} PASSWORD '{}'", user.name, user.password).to_owned();
        let stmt = self.client.prepare(&sql).unwrap();

        info!("executing: {}", sql);
        self.client
            .execute(&stmt, &[])
            .expect("could not update user password");
    }

    /// Get the list of users
    pub fn get_users(&mut self) -> Result<Vec<User>> {
        let mut users = vec![];
        let sql = "SELECT usename, usecreatedb, usesuper, passwd FROM pg_user";
        let stmt = self.client.prepare(&sql).unwrap();

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

        let stmt = self.client.prepare(&sql).unwrap();

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

        let stmt = self.client.prepare(&sql).unwrap();

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
              has_table_privilege(u.usename, CONCAT(t.schemaname, '.', t.tablename), 'select') AS has_select,
              has_table_privilege(u.usename, CONCAT(t.schemaname, '.', t.tablename), 'insert') AS has_insert,
              has_table_privilege(u.usename, CONCAT(t.schemaname, '.', t.tablename), 'update') AS has_update,
              has_table_privilege(u.usename, CONCAT(t.schemaname, '.', t.tablename), 'delete') AS has_delete,
              has_table_privilege(u.usename, CONCAT(t.schemaname, '.', t.tablename), 'references') AS has_references
            FROM
                pg_user u
                CROSS JOIN (SELECT DISTINCT schemaname, tablename FROM pg_tables) t
                WHERE
                1 = 1
                AND t.schemaname != 'pg_catalog'
                AND t.schemaname != 'information_schema';
        ";

        let stmt = self.client.prepare(&sql).unwrap();

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

    pub fn query<T>(&mut self, query: &T, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<Row>>
    where
        T: ?Sized + ToStatement,
    {
        let ri = self.client.query(query, params)?;
        Ok(ri)
    }
}

// Test DbConnection
#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_connect_from_config() {
        let config = Config::from_str(
            r#"
            connection:
              type: postgres
              url: "postgresql://postgres:postgres@localhost:5432/postgres"
            roles: []
            users: []
            "#,
        )
        .unwrap();
        let mut db = DbConnection::new(&config);
        db.query("SELECT 1", &[]).unwrap();
    }

    #[test]
    fn test_connect_from_string() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres".to_string();
        let mut db = DbConnection::new_from_string(url);
        db.query("SELECT 1", &[]).unwrap();
    }

    #[test]
    fn test_drop_user() {
        let url = "postgres://postgres:postgres@localhost:5432/postgres".to_string();
        let mut db = DbConnection::new_from_string(url);

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);
        db.drop_user(&user);

        let users = db.get_users().unwrap_or(vec![]);
        assert_eq!(users.iter().any(|u| u.name == name), false);

        // Clean up
        db.drop_user(&user);
    }

    #[test]
    fn test_drop_create_user() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::new_from_string(url.to_string());

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        let users = db.get_users().unwrap();

        assert_eq!(users.iter().any(|u| u.name == name), true);

        // Clean up
        db.drop_user(&user);
    }

    #[test]
    fn test_get_schema_roles() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::new_from_string(url.to_string());

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        // get user roles
        let user_schema_privileges = db.get_user_schema_privileges().unwrap_or(vec![]);

        // FIXME it will be empty if the schema doesn't have any tables
        if user_schema_privileges.len() > 0 {
            // new user, that user will don't have any priviledge
            assert_eq!(
                user_schema_privileges
                    .iter()
                    .any(|u| u.name == name && u.has_usage == false && u.has_create == false),
                true
            );
        }

        // Clean up
        db.drop_user(&user);
    }

    // Test query_raw
    #[test]
    fn test_query() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::new_from_string(url.to_string());
        let rows = db.query("SELECT 1 as t", &[]).unwrap();
        debug!("test_query: {:?}", rows);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows.get(0).unwrap().len(), 1);

        let t: i32 = rows.get(0).unwrap().get("t");
        assert_eq!(t, 1);
    }

    // Test get_user_database_privileges
    #[test]
    fn test_get_user_database_privileges() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::new_from_string(url.to_string());

        let name = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        // get user roles
        let user_database_privileges = db.get_user_database_privileges().unwrap_or(vec![]);

        // Check if user_database_privileges contains current users
        // is empty if the user doesn't have any database privileges
        assert_eq!(
            user_database_privileges
                .iter()
                .any(|u| u.name == name && u.has_create == true),
            false
        );

        // FIXME seriously test this function

        // Clean up
        db.drop_user(&user);
    }

    // Test get_user_schema_privileges
    #[test]
    fn test_get_user_schema_privileges() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::new_from_string(url.to_string());

        let name = random_str();
        let password = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: password.to_owned(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        // get user roles
        let user_schema_privileges = db.get_user_schema_privileges().unwrap_or(vec![]);
        println!("{:?}", user_schema_privileges);

        // Check if user_schema_privileges contains current users
        // is empty if the user doesn't have any schema privileges
        // assert_eq!(user_schema_privileges.iter().any(|u| u.name == name), false);

        // FIXME seriously test this function

        // Clean up
        db.drop_user(&user);
    }

    // Test get_user_tables_privileges
    #[test]
    fn test_get_user_table_privileges() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::new_from_string(url.to_string());

        let name = random_str();
        let password = random_str();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: password.to_owned(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        // get user roles
        let user_table_privileges = db.get_user_table_privileges().unwrap_or(vec![]);

        // Check if user_tables_privileges contains current users
        // is empty if the user doesn't have any tables privileges
        assert_eq!(
            user_table_privileges
                .iter()
                .any(|u| u.name == name && u.has_select == true),
            false
        );

        // FIXME seriously test this function

        // Clean up
        db.drop_user(&user);
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
