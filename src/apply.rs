use crate::config::{Config, Role, User as UserInConfig};
use crate::connection::{DbConnection, User};
use ansi_term::Colour::{Green, Purple, Red};
use anyhow::{anyhow, Result};
use ascii_table::AsciiTable;
use log::{error, info};
use std::path::Path;

/// Read the config from the given path and apply it to the database.
/// If the dryrun flag is set, the changes will not be applied.
pub fn apply(target: &Path, dryrun: bool) -> Result<()> {
    let target = target.to_path_buf();

    if target.is_dir() {
        return Err(anyhow!(
            "directory is not supported yet ({})",
            target.display()
        ));
    }

    let config = Config::new(&target)?;

    info!("Applying configuration:\n{}", config);
    let mut conn = DbConnection::new(&config)?;

    let users_in_db = conn.get_users()?;
    let users_in_config = config.users.clone();

    // Apply users changes (new users, update password)
    create_or_update_users(&mut conn, &users_in_db, &users_in_config, dryrun)?;

    // Apply roles privileges to cluster (database role, schema role, table role)
    create_or_update_privileges(&mut conn, &config, dryrun)?;

    Ok(())
}

/// Apply all config files from the given directory.
pub fn apply_all(target: &Path, dryrun: bool) -> Result<()> {
    let target = target.to_path_buf();

    // Scan recursively for config files (.yaml for .yml) in target directory
    let mut config_files = Vec::new();
    for entry in std::fs::read_dir(target)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yaml" || ext == "yml" {
                    config_files.push(path);
                }
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
fn create_or_update_users(
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
                // Update password if `update_password` is set to true
                if user.update_password.unwrap_or(false) {
                    let sql = user.to_sql_update()?;

                    if dryrun {
                        info!("{}: {}", Purple.paint("Dry-run"), Purple.paint(&sql));
                        summary.push(vec![
                            user.name.to_string(),
                            Green.paint("would update password").to_string(),
                        ]);
                    } else {
                        conn.execute(&sql, &[])?;
                        info!("{}: {}", Green.paint("Success"), Purple.paint(&sql));
                        summary.push(vec![user.name.clone(), "password updated".to_string()]);
                    }
                } else {
                    // Do nothing if user is not changed
                    summary.push(vec![
                        user_in_db.name.clone(),
                        "no action (already exists)".to_string(),
                    ]);
                }
            }

            // User in config but not in database
            None => {
                let sql = user.to_sql_create()?;

                if dryrun {
                    info!("{}: {}", Purple.paint("Dry-run"), &sql);
                    summary.push(vec![
                        user.name.clone(),
                        format!("would create (dryrun) {}", sql),
                    ]);
                } else {
                    conn.execute(&sql, &[])?;
                    info!("{}: {}", Green.paint("Success"), &sql);
                    summary.push(vec![user.name.clone(), format!("created {}", sql)]);
                }
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
fn create_or_update_privileges(
    conn: &mut DbConnection,
    config: &Config,
    dryrun: bool,
) -> Result<()> {
    let mut summary = vec![vec![
        "User".to_string(),
        "Role Name".to_string(),
        "Detail".to_string(),
        "Status".to_string(),
    ]];
    summary.push(vec![
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
    ]);

    // Loop through users in config
    // Get the user Role object by the user.roles[*].name
    // Apply the Role sql privileges to the cluster
    for user in &config.users {
        // Compare privileges on config and db
        // If privileges on config are not in db, add them
        // If privileges on db are not in config, remove them
        for role_name in user.roles.iter() {
            let role = config.roles.iter()
                .find(|&r| r.find(role_name))
                .ok_or_else(|| anyhow!("Role '{}' not found for user '{}'", role_name, user.name))?;

            // TODO: revoke if privileges on db are not in configuration

            let sql = role.to_sql(&user.name);

            let mut status = if dryrun {
                "dry-run".to_string()
            } else {
                "updated".to_string()
            };

            if !dryrun {
                let nrows = conn.execute(&sql, &[]).unwrap_or_else(|e| {
                    error!("{}: {}", Red.paint("Error"), sql);
                    error!("  -> {}: {}", Red.paint("Error details"), e);
                    status = "error".to_string();

                    -1
                });

                if nrows > -1 {
                    info!(
                        "{}: {} {}",
                        Green.paint("Success"),
                        Purple.paint(sql),
                        format!("(updated {} row(s))", nrows)
                    );
                }
            } else {
                info!("{}: {}", Purple.paint("Dry-run"), sql);
            }

            let detail = match role {
                Role::Database(role) => format!("database{:?}", role.databases.clone()),
                Role::Schema(role) => format!("schema{:?}", role.schemas.clone()),
                Role::Table(role) => format!("table{:?}", role.tables.clone()),
            };

            // Update summary
            summary.push(vec![
                user.name.clone(),
                role_name.clone(),
                detail.to_string(),
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
