use crate::config::{Config, Role, User as UserInConfig};
use crate::connection::{DbConnection, User};
use ansi_term::Colour::{Green, Purple, Red};
use anyhow::{anyhow, Result};
use ascii_table::AsciiTable;
use log::error;
use log::info;
use std::path::PathBuf;

/// Read the config from the given path and apply it to the database.
/// If the dryrun flag is set, the changes will not be applied.
pub fn apply(target: &PathBuf, dryrun: bool) -> Result<()> {
    if target.is_dir() {
        return Err(anyhow!("The target is a directory"));
    }

    let config = Config::new(&target)?;

    info!("Applying configuration:\n{}", config);
    let mut conn = DbConnection::new(&config);

    let users_in_db = conn.get_users()?;
    let users_in_config = config.users.clone();

    // Apply users changes (new users, update password)
    apply_users(&mut conn, &users_in_db, &users_in_config, dryrun)?;

    // Apply roles privileges to cluster (database role, schema role, table role)
    apply_privileges(&mut conn, &config, dryrun)?;

    Ok(())
}

/// Apply all config files from the given directory.
pub fn apply_all(target: &PathBuf, dryrun: bool) -> Result<()> {
    // Scan recursively for config files (.yaml for .yml) in target directory
    let mut config_files = Vec::new();
    for entry in std::fs::read_dir(target)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().unwrap();
            if ext == "yaml" || ext == "yml" {
                config_files.push(path);
            }
        }
    }

    // Apply each config file
    for config_file in config_files {
        info!("Applying configuration from {}", config_file.display());
        apply(&config_file, dryrun)?;
    }

    Ok(())
}

/// Apply users from config to database
///
/// Get list users from database and compare with config users
/// If user is in config but not in database, create it
/// If user is in database but not in config, delete it
/// If user is in both, compare passwords and update if needed
///
/// Show the summary as table of users created, updated, deleted
fn apply_users(
    conn: &mut DbConnection,
    users_in_db: &[User],
    users_in_config: &[UserInConfig],
    dryrun: bool,
) -> Result<()> {
    let mut summary = vec![vec!["User".to_string(), "Action".to_string()]];
    summary.push(vec!["---".to_string(), "---".to_string()]);

    // Create or update users in database
    for user in users_in_config {
        let user_in_db = users_in_db.iter().find(|&u| u.name == user.name);
        match user_in_db {
            // User in config and in database
            Some(user_in_db) => {
                // TODO: Update password if needed, currently we can't compare the password

                // Do nothing if user is not changed
                summary.push(vec![
                    user_in_db.name.clone(),
                    "no action (existing)".to_string(),
                ]);
            }

            // User in config but not in database
            None => {
                let sql = user.to_sql_create();

                if dryrun {
                    summary.push(vec![
                        user.name.clone(),
                        format!("would create (dryrun) {}", sql),
                    ]);
                } else {
                    conn.execute(&sql, &[])?;
                    summary.push(vec![user.name.clone(), format!("created {}", sql)]);
                }

                // Update summary
                summary.push(vec![user.name.clone(), "created".to_string()]);
            }
        }
    }

    // TODO: Support delete users in db that are not in config
    for user in users_in_db {
        if !users_in_config.iter().any(|u| u.name == user.name) {
            // Update summary
            summary.push(vec![
                user.name.clone(),
                "no action (not in config)".to_string(),
            ]);
        }
    }

    // Show summary
    print_summary(summary);

    Ok(())
}

/// Render role configuration to SQL and sync with database.
/// If the privileges are not in the database, they will be granted to user.
/// If the privileges are in the database, they will be updated.
/// If the privileges are not in the configuration, they will be revoked from user.
fn apply_privileges(conn: &mut DbConnection, config: &Config, dryrun: bool) -> Result<()> {
    let mut summary = vec![vec![
        "User".to_string(),
        "Role Name".to_string(),
        "Detail".to_string(),
        "Action".to_string(),
        "Status".to_string(),
    ]];
    summary.push(vec![
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
    ]);

    let database_privileges = conn.get_user_database_privileges()?;
    let schema_privileges = conn.get_user_schema_privileges()?;
    let table_privileges = conn.get_user_table_privileges()?;

    // Loop through users in config
    // Get the user Role object by the user.roles[*].name
    // Apply the Role sql privileges to the cluster
    for user in &config.users {
        // Using current privileges on database to revoke nessessary privileges
        let current_user_database_privileges =
            database_privileges.iter().find(|&p| p.name == user.name);
        let current_user_schema_privileges =
            schema_privileges.iter().find(|&p| p.name == user.name);
        let current_user_table_privileges = table_privileges.iter().find(|&p| p.name == user.name);

        // Compare privileges on config and db
        // If privileges on config are not in db, add them
        // If privileges on db are not in config, remove them
        for role_name in user.roles.iter() {
            let role = config.roles.iter().find(|&r| r.find(&role_name)).unwrap();

            // TODO: revoke if privileges on db are not in configuration

            // grant privileges from config to database
            // if the role_name start with `-`, it means the privileges are revoked
            // if the role_name start with `+` or nothing, it means the privileges are granted
            let grant_revoke = if role_name.starts_with("-") {
                "revoke"
            } else {
                "grant"
            };

            let sql = if grant_revoke == "revoke" {
                role.to_sql_revoke(user.name.clone())
            } else {
                role.to_sql_grant(user.name.clone())
            };

            if !dryrun {
                conn.query(&sql, &[]).unwrap_or_else(|e| {
                    error!("{}: {}", Red.paint("Error"), sql);
                    panic!("{}", e);
                });
                info!("{}: {}", Green.paint("Success"), sql);
            } else {
                info!("{}: {}", Purple.paint("Dry-run"), sql);
            }

            let (status, message) = match role {
                Role::Database(role) => {
                    let kind = "database";
                    let status = if current_user_database_privileges.is_none() {
                        "granted"
                    } else {
                        "updated"
                    };
                    let message = format!("{}{:?}", kind, role.databases.clone());

                    (status, message)
                }
                Role::Schema(role) => {
                    let kind = "schema";
                    let status = if current_user_schema_privileges.is_none() {
                        "granted"
                    } else {
                        "updated"
                    };
                    let message = format!("{}{:?}", kind, role.schemas.clone());

                    (status, message)
                }
                Role::Table(role) => {
                    let kind = "table";
                    let status = if current_user_table_privileges.is_none() {
                        "granted"
                    } else {
                        "updated"
                    };
                    let message = format!("{}{:?}", kind, role.tables.clone());

                    (status, message)
                }
            };

            // Update summary
            summary.push(vec![
                user.name.clone(),
                role_name.clone(),
                message.to_string(),
                grant_revoke.to_string(),
                status.to_string(),
            ]);
        }
    }

    // Show summary
    print_summary(summary);

    Ok(())
}

/// Print summary table
/// TODO: Format the table, detect max size to console
fn print_summary(summary: Vec<Vec<String>>) {
    let ascii_table = AsciiTable::default();
    info!("Summary:\n{}", ascii_table.format(summary));
}
