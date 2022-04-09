use crate::config::Config;
use crate::connection::{DbConnection, UserDatabaseRole, UserSchemaRole, UserTableRole};
use anyhow::Result;
use ascii_table::AsciiTable;
use indoc::indoc;
use log::info;

pub fn inspect(config: &Config) -> Result<()> {
    let mut conn = DbConnection::new(config);

    let users_in_db = conn.get_users()?;
    let user_database_privileges = conn
        .get_user_database_privileges()
        .unwrap()
        .into_iter()
        .filter(|p| p.database_name == conn.get_current_database().unwrap())
        .collect::<Vec<_>>();
    let user_schema_privileges = conn.get_user_schema_privileges()?;
    let user_table_privileges = conn.get_user_table_privileges()?;

    let mut users = users_in_db
        .iter()
        .map(|u| {
            vec![
                u.name.clone(),
                u.user_super.to_string(),
                get_user_database_privileges(&user_database_privileges, &u.name).unwrap(),
                get_user_schema_privileges(&user_schema_privileges, &u.name).unwrap(),
                get_user_table_privileges(&user_table_privileges, &u.name).unwrap(),
            ]
        })
        .collect::<Vec<_>>();

    users.insert(
        0,
        vec![
            "User".to_string(),
            "Super".to_string(),
            "Current Database".to_string(),
            "Schemas".to_string(),
            "Tables".to_string(),
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

    // Get the terminal with
    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120) - 5;

    // Print the table in max size
    let mut table = AsciiTable::default();
    table.set_max_width(term_width);

    info!(
        "Current users in {}:\n{}",
        config.connection.url,
        table.format(users)
    );

    info!(indoc! { r#"
        == Legend ==

        Database:
            A = ALL Privileges
            C = CREATE
            T = TEMP

        Schema:
            A = ALL Privileges
            C = CREATE
            U = USAGE

        Table:
            A = ALL Privileges
            S = SELECT
            U = UPDATE
            I = INSERT
            D = DELETE
            R = REFERENCES
    "#});

    Ok(())
}

/// Get current user database privileges
fn get_user_database_privileges(privileges: &[UserDatabaseRole], user: &str) -> Result<String> {
    let privileges = privileges
        .iter()
        .filter(|p| p.name == *user) // is current user
        .filter(|p| p.has_create || p.has_temp) // has at least create or temp
        .map(|p| p.perm_to_string(true))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(privileges)
}

/// Get current user schema privileges
fn get_user_schema_privileges(privileges: &[UserSchemaRole], user: &str) -> Result<String> {
    let privileges = privileges
        .iter()
        .filter(|p| p.name == *user)
        .filter(|p| p.has_create || p.has_usage)
        .map(|p| p.perm_to_string(true))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(privileges)
}

/// Get current user schema.table privileges
fn get_user_table_privileges(privileges: &[UserTableRole], user: &str) -> Result<String> {
    let privileges = privileges
        .iter()
        .filter(|p| p.name == *user) // is current user
        .filter(|p| {
            p.has_select || p.has_insert || p.has_update || p.has_delete || p.has_references
        }) // has at least create or select
        .map(|p| p.perm_to_string(true))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(privileges)
}
