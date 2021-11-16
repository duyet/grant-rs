use anyhow::Result;
use log::debug;
use postgres::row::Row;
use postgres::types::ToSql;
use postgres::{Client, NoTls, ToStatement};

// TODO: support multiple adapters

pub struct DbConnection {
    connection_info: String,
    client: Client,
}

#[derive(Debug)]
pub struct User {
    user_name: String,
    user_createdb: bool,
    user_super: bool,
    passwd: String,
}

#[derive(Debug)]
pub struct UserSchemaRole {
    user_name: String,
    schema_name: String,
    has_create: bool,
    has_usage: bool,
}

impl DbConnection {
    /// A convenience function which store the connection string into `connection_info` and then connects to the database.
    ///
    /// Refer to https://rust-lang-nursery.github.io/rust-cookbook/database/postgres.html
    pub fn connect(conn: &str) -> Self {
        let connection_info = conn.to_string();
        let client = Client::connect(conn, NoTls)
            .unwrap_or_else(|err| panic!("could not connect to {}: {:?}", conn, err));

        Self {
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
        let sql: String = format!("DROP USER IF EXISTS {}", user.user_name).to_owned();
        debug!("drop_user: {}", sql);

        self.client.execute(&sql, &[]).expect("could not drop user");
    }

    /// Try to drop a user, this fn will not panic
    pub fn try_drop_user(&mut self, user: &User) {
        let sql: String = format!("DROP USER IF EXISTS {}", user.user_name).to_owned();
        debug!("try_drop_user: {}", sql);

        self.client.execute(&sql, &[]).unwrap_or_else(|_| 1);
    }

    /// Create user
    pub fn create_user(&mut self, user: &User) {
        let mut sql: String = format!("CREATE USER {} ", user.user_name).to_owned();
        if user.user_createdb {
            sql += "CREATEDB"
        }
        if !user.passwd.is_empty() {
            sql += &format!(" PASSWORD '{}'", user.passwd).to_string()
        }

        debug!("create user: {}", sql);
        let stmt = self.client.prepare(&sql).unwrap();
        self.client
            .execute(&stmt, &[])
            .expect("could not create user");
    }

    /// Get the list of users
    pub fn get_users(&mut self) -> Result<Vec<User>> {
        let mut users = vec![];
        let sql = "SELECT usename, usecreatedb, usesuper FROM pg_user";
        for row in self.client.query(sql, &[])? {
            match (row.get(0), row.get(1), row.get(2)) {
                (Some(user_name), Some(user_createdb), Some(user_super)) => users.push(User {
                    user_name,
                    user_createdb,
                    user_super,
                    passwd: String::from(""),
                }),
                (Some(user_name), _, _) => users.push(User {
                    user_name,
                    user_createdb: false,
                    user_super: false,
                    passwd: String::from(""),
                }),
                (_, _, _) => (),
            }
        }

        debug!("get_users: {:#?}", users);

        Ok(users)
    }

    /// Get the current schema roles from cluster
    pub fn get_schema_roles(&mut self) -> Result<Vec<UserSchemaRole>> {
        let sql = "
            SELECT
              u.usename AS user_name,
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
                (Some(user_name), Some(schema_name), Some(has_create), Some(has_usage)) => {
                    user_schema_roles.push(UserSchemaRole {
                        user_name,
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
    fn test_connect() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::connect(url);
        db.ping().expect("cannot ping");
    }

    #[test]
    fn test_drop_user() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::connect(url);

        let user_name = random_username();
        let user = User {
            user_name: user_name.to_owned(),
            user_createdb: false,
            user_super: false,
            passwd: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);
        db.drop_user(&user);

        let users = db.get_users().unwrap();

        assert_eq!(users.iter().any(|u| u.user_name == user_name), false);

        // Clean up
        db.drop_user(&user);
    }

    #[test]
    fn test_drop_create_user() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::connect(url);

        let user_name = random_username();
        let user = User {
            user_name: user_name.to_owned(),
            user_createdb: false,
            user_super: false,
            passwd: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        let users = db.get_users().unwrap();

        assert_eq!(users.iter().any(|u| u.user_name == user_name), true);

        // Clean up
        db.drop_user(&user);
    }

    #[test]
    fn test_get_schema_roles() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::connect(url);

        let user_name = random_username();
        let user = User {
            user_name: user_name.to_owned(),
            user_createdb: false,
            user_super: false,
            passwd: "duyet".to_string(),
        };
        db.drop_user(&user);
        db.create_user(&user);

        let user_schema_roles = db.get_schema_roles().unwrap();

        // Create new user, that user will don't have any priviledge
        assert_eq!(
            user_schema_roles
                .iter()
                .any(|u| u.user_name == user_name && u.has_usage == false && u.has_create == false),
            true
        );

        // Clean up
        db.drop_user(&user);
    }

    // Test query_raw
    #[test]
    fn test_query() {
        let url = "postgresql://postgres:postgres@localhost:5432/postgres";
        let mut db = DbConnection::connect(url);
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

        let user_name: String = (0..10)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        user_name
    }
}
