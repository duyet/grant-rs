use crate::config::Config;
use crate::connection::DbConnection;
use anyhow::Result;
use ascii_table::AsciiTable;
use indoc::indoc;
use log::info;
use term_size;

pub fn inspect(config: &Config) -> Result<()> {
    let mut conn = DbConnection::new(config);

    let users_in_db = conn.get_users()?;
    let mut users = users_in_db
        .iter()
        .map(|u| {
            vec![
                u.name.clone(),
                u.user_super.to_string(),
                get_user_database_privileges(&mut conn, &u.name).unwrap(),
                get_user_schema_privileges(&mut conn, &u.name).unwrap(),
                get_user_table_privileges(&mut conn, &u.name).unwrap(),
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
    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

    // Print the table in max size
    let mut table = AsciiTable::default();
    table.max_width = term_width;

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
fn get_user_database_privileges(conn: &mut DbConnection, user: &str) -> Result<String> {
    let privileges = conn
        .get_user_database_privileges()?
        .iter()
        .filter(|p| p.name == user.to_string()) // is current user
        .filter(|p| p.has_create || p.has_temp) // has at least create or temp
        .filter(|p| p.database_name == conn.get_current_database().unwrap()) // is current database
        .map(|p| p.perm_to_string(true))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(privileges)
}

/// Get current user schema privileges
fn get_user_schema_privileges(conn: &mut DbConnection, user: &str) -> Result<String> {
    let privileges = conn
        .get_user_schema_privileges()?
        .iter()
        .filter(|p| p.name == user.to_string())
        .filter(|p| p.has_create || p.has_usage)
        .map(|p| p.perm_to_string(true))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(privileges)
}

/// Get current user schema.table privileges
fn get_user_table_privileges(conn: &mut DbConnection, user: &str) -> Result<String> {
    let privileges = conn
        .get_user_table_privileges()?
        .iter()
        .filter(|p| p.name == user.to_string()) // is current user
        .filter(|p| {
            p.has_select || p.has_insert || p.has_update || p.has_delete || p.has_references
        }) // has at least create or select
        .map(|p| p.perm_to_string(true))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(privileges)
}
