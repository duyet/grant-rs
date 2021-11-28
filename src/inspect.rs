use crate::config::{Config, Role, User as UserInConfig};
use crate::connection::{DbConnection, User};
use anyhow::{bail, Result};
use ascii_table::AsciiTable;
use log::info;

pub fn inspect(config: &Config) -> Result<()> {
    let mut conn = DbConnection::connect(config);

    let users_in_db = conn.get_users()?;
    let mut users = users_in_db
        .iter()
        .map(|u| {
            vec![
                u.name.clone(),
                u.user_createdb.to_string(),
                u.user_super.to_string(),
                u.password.clone(),
            ]
        })
        .collect::<Vec<_>>();

    users.insert(
        0,
        vec![
            "User".to_string(),
            "CreateDB".to_string(),
            "Super".to_string(),
            "Password".to_string(),
        ],
    );
    users.insert(
        1,
        vec![
            "---".to_string(),
            "---".to_string(),
            "---".to_string(),
            "---".to_string(),
        ],
    );

    // Print the table
    let mut table = AsciiTable::default();
    info!(
        "Current users in {}:\n{}",
        config.connection.url,
        table.format(users)
    );

    Ok(())
}
