use crate::config::{Config, ConnectionType};
use anyhow::Result;
use log::debug;
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

    pub fn set_password(&mut self, password: &str) -> &mut Self {
        self.password = password.to_string();

        self
    }
}

#[derive(Debug)]
pub struct UserSchemaRole {
    name: String,
    schema_name: String,
    has_create: bool,
    has_usage: bool,
}

impl DbConnection {
    /// A convenience function which store the connection string into `connection_info` and then connects to the database.
    ///
    /// Refer to <https://rust-lang-nursery.github.io/rust-cookbook/database/postgres.html>
    pub fn connect(config: &Config) -> Self {
        match config.connection.type_ {
            ConnectionType::Postgres => {
                let connection_info = config.connection.url.clone();
                let client = Client::connect(&connection_info, NoTls).unwrap();
                DbConnection {
                    connection_info,
                    client,
                }
            }
            _ => panic!("Unsupported connection type: {:?}", config.connection.type_),
        }
    }

    /// Connection by a connection string.
    pub fn connect_by_string(connection_info: String) -> Self {
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

    /// Ping the database
    pub fn ping(&mut self) -> Result<bool> {
        let stmt = self.client.prepare("SELECT 1").unwrap();
        let rows = self.client.execute(&stmt, &[]).unwrap();
        assert_eq!(rows, 1, "should be 1");

        Ok(true)
    }

    /// Drop a user
    pub fn drop_user(&mut self, user: &User) {
        let sql: String = format!("DROP USER IF EXISTS {}", user.name).to_owned();
        debug!("drop_user: {}", sql);

        self.client.execute(&sql, &[]).expect("could not drop user");
    }

    /// Try to drop a user, this fn will not panic
    pub fn try_drop_user(&mut self, user: &User) {
        let sql: String = format!("DROP USER IF EXISTS {}", user.name).to_owned();
        debug!("try_drop_user: {}", sql);

        self.client.execute(&sql, &[]).unwrap_or_else(|_| 1);
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

        debug!("create user: {}", sql);
        let stmt = self.client.prepare(&sql).unwrap();
        self.client
            .execute(&stmt, &[])
            .expect("could not create user");
    }

    /// Update user password
    pub fn update_user_password(&mut self, user: &User) {
        let sql: String =
            format!("ALTER USER {} PASSWORD '{}'", user.name, user.password).to_owned();
        debug!("update user password: {}", sql);
        let stmt = self.client.prepare(&sql).unwrap();
        self.client
            .execute(&stmt, &[])
            .expect("could not update user password");
    }

    /// Get the list of users
    pub fn get_users(&mut self) -> Result<Vec<User>> {
        let mut users = vec![];
        let sql = "SELECT usename, usecreatedb, usesuper, passwd FROM pg_user";
        for row in self.client.query(sql, &[])? {
            match (row.get(0), row.get(1), row.get(2), row.get(3)) {
                (Some(name), Some(user_createdb), Some(user_super), Some(password)) => users.push(User {
                    name,
                    user_createdb,
                    user_super,
                    password,
                }),
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

    /// Get the current schema roles from cluster
    pub fn get_schema_roles(&mut self) -> Result<Vec<UserSchemaRole>> {
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

        let mut user_schema_roles = vec![];
        for row in self.client.query(sql, &[])? {
            match (row.get(0), row.get(1), row.get(2), row.get(3)) {
                (Some(name), Some(schema_name), Some(has_create), Some(has_usage)) => {
                    user_schema_roles.push(UserSchemaRole {
                        name,
                        schema_name,
                        has_create,
                        has_usage,
                    })
                }
                (_, _, _, _) => (),
            }
        }

        Ok(user_schema_roles)
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
              url: "postgresql://postgres@localhost:5432/postgres"
            roles: []
            users: []
            "#,
        )
        .unwrap();
        let mut db = DbConnection::connect(&config);
        db.ping().unwrap();
    }

    #[test]
    fn test_connect_from_string() {
        let url = "postgres://postgres@localhost:5432/postgres".to_string();
        let mut db = DbConnection::connect_by_string(url);
        db.ping().unwrap();
    }

    #[test]
    fn test_drop_user() {
        let url = "postgres://postgres@localhost:5432/postgres".to_string();
        let mut db = DbConnection::connect_by_string(url);

        let name = random_username();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);
        db.drop_user(&user);

        let users = db.get_users().unwrap();

        assert_eq!(users.iter().any(|u| u.name == name), false);

        // Clean up
        db.drop_user(&user);
    }

    #[test]
    fn test_drop_create_user() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::connect_by_string(url.to_string());

        let name = random_username();
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
        let mut db = DbConnection::connect_by_string(url.to_string());

        let name = random_username();
        let user = User {
            name: name.to_owned(),
            user_createdb: false,
            user_super: false,
            password: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        // get user roles
        let user_schema_roles = db.get_schema_roles().unwrap();
        println!("xxx {:#?}", user_schema_roles);

        // FIXME it will be empty if the schema doesn't have any tables
        if user_schema_roles.len() > 0 {
            // new user, that user will don't have any priviledge
            assert_eq!(
                user_schema_roles.iter().any(|u| u.name == name
                    && u.has_usage == false
                    && u.has_create == false),
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
        let mut db = DbConnection::connect_by_string(url.to_string());
        let rows = db.query("SELECT 1 as t", &[]).unwrap();
        debug!("test_query: {:?}", rows);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows.get(0).unwrap().len(), 1);

        let t: i32 = rows.get(0).unwrap().get("t");
        assert_eq!(t, 1);
    }

    fn random_username() -> String {
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
