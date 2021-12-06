use crate::config::{Config, Role, User as UserInConfig};
use crate::connection::{DbConnection, User};
use anyhow::Result;
use ascii_table::AsciiTable;
use log::error;
use log::info;

pub fn apply(config: &Config, dryrun: bool) -> Result<()> {
    info!("Applying configuration:\n{}", config);
    let mut conn = DbConnection::new(config);

    let users_in_db = conn.get_users()?;
    let users_in_config = config.users.clone();

    // TODO: Refactor these functions
    apply_users(&mut conn, &users_in_db, &users_in_config, dryrun)?;
    apply_database_privileges(&mut conn, &config, dryrun)?;
    apply_schema_privileges(&mut conn, &config, dryrun)?;
    apply_table_privileges(&mut conn, &config, dryrun)?;

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
            Some(user_in_db) => {
                if user_in_db.password != user.password {
                    // TODO: fixme just clone the user and change the password, but this is not working
                    let user_to_update = User {
                        name: user_in_db.name.clone(),
                        user_createdb: false,
                        user_super: false,
                        password: user.password.clone(),
                    };

                    if !dryrun {
                        conn.update_user_password(&user_to_update);
                        info!("User {} password updated", user_to_update.name);
                    } else {
                        info!("User {} password would be updated", user_to_update.name);
                    }

                    // Update summary
                    summary.push(vec![user.name.clone(), "update password".to_string()]);
                } else {
                    info!("User {} already exists", user.name);

                    // Update summary
                    summary.push(vec![user.name.clone(), "unchanged".to_string()]);
                }
            }
            None => {
                let new_user = User::new(user.name.clone(), false, false, user.password.clone());

                if !dryrun {
                    conn.create_user(&new_user);
                    info!("User {} created", new_user.name);
                } else {
                    info!("User {} would be created", new_user.name);
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

/// Apply database privileges from config to database
pub fn apply_database_privileges(
    conn: &mut DbConnection,
    config: &Config,
    dryrun: bool,
) -> Result<()> {
    let mut summary = vec![vec![
        "User".to_string(),
        "Database Privilege".to_string(),
        "Action".to_string(),
    ]];
    summary.push(vec![
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
    ]);

    let user_privileges_on_db = conn.get_user_database_privileges()?;

    // Loop through users in config
    // Get the user Role object by the user.roles[*].name
    // Apply the Role sql privileges to the cluster
    for user in &config.users {
        // Get roles for user from config
        let user_roles_in_config: Vec<_> = user
            .roles
            .iter()
            .map(|role_name| {
                config
                    .roles
                    .iter()
                    .find(|&r| r.get_name() == role_name.to_string())
                    .unwrap()
            })
            .collect();

        let privileges_on_db = user_privileges_on_db.iter().find(|&p| p.name == user.name);

        // Compare privileges on config and db
        // If privileges on config are not in db, add them
        // If privileges on db are not in config, remove them
        for role in user_roles_in_config {
            match role {
                Role::Database(role) => {
                    println!("==> {:?}", role);
                    // revoke
                    if !privileges_on_db.is_none() {
                        let sql = role.to_sql_revoke(user.name.clone());
                        if !dryrun {
                            conn.query(&sql, &[]).unwrap_or_else(|e| {
                                error!("failed to run sql:\n{}", sql);
                                panic!("{}", e);
                            });
                            info!("Revoking: {}", sql);
                        } else {
                            info!("Would revoke: {}", sql);
                        }
                    }

                    // grant
                    let sql = role.to_sql_grant(user.name.clone());
                    if !dryrun {
                        conn.query(&sql, &[]).unwrap_or_else(|e| {
                            error!("failed to run sql:\n{}", sql);
                            panic!("{}", e);
                        });
                        info!("Granting: {}", sql);
                    } else {
                        info!("Granting: {}", sql);
                    }

                    let action = if privileges_on_db.is_none() {
                        "granted"
                    } else {
                        "updated"
                    };

                    // Update summary
                    summary.push(vec![
                        user.name.clone(),
                        format!(
                            "privileges `{}` for database: {:?}",
                            role.name.clone(),
                            role.databases.clone()
                        ),
                        action.to_string(),
                    ]);
                }
                _ => {}
            }
        }
    }

    // Show summary
    print_summary(summary);

    Ok(())
}

/// Apply schema privileges from config to database
pub fn apply_schema_privileges(
    conn: &mut DbConnection,
    config: &Config,
    dryrun: bool,
) -> Result<()> {
    let mut summary = vec![vec![
        "User".to_string(),
        "Schema Privileges".to_string(),
        "Action".to_string(),
    ]];
    summary.push(vec![
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
    ]);

    let user_privileges_on_db = conn.get_user_schema_privileges()?;

    // Loop through users in config
    // Get the user Role object by the user.roles[*].name
    // Apply the Role sql privileges to the cluster
    for user in &config.users {
        // Get roles for user from config
        let user_roles_in_config: Vec<_> = user
            .roles
            .iter()
            .map(|role_name| {
                config
                    .roles
                    .iter()
                    .find(|&r| r.get_name() == role_name.to_string())
                    .unwrap()
            })
            .collect();

        let privileges_on_db = user_privileges_on_db.iter().find(|&p| p.name == user.name);

        // Compare privileges on config and db
        // If privileges on config are not in db, add them
        // If privileges on db are not in config, remove them
        for role in user_roles_in_config {
            match role {
                Role::Schema(role) => {
                    println!("==> {:?}", role);
                    // revoke
                    if !privileges_on_db.is_none() {
                        let sql = role.to_sql_revoke(user.name.clone());
                        if !dryrun {
                            conn.query(&sql, &[]).unwrap_or_else(|e| {
                                error!("failed to run sql:\n{}", sql);
                                panic!("{}", e);
                            });
                            info!("Revoking: {}", sql);
                        } else {
                            info!("Would revoke: {}", sql);
                        }
                    }

                    // grant
                    let sql = role.to_sql_grant(user.name.clone());
                    if !dryrun {
                        conn.query(&sql, &[]).unwrap_or_else(|e| {
                            error!("failed to run sql:\n{}", sql);
                            panic!("{}", e);
                        });
                        info!("Granting: {}", sql);
                    } else {
                        info!("Granting: {}", sql);
                    }

                    let action = if privileges_on_db.is_none() {
                        "granted"
                    } else {
                        "updated"
                    };

                    // Update Summary
                    summary.push(vec![
                        user.name.clone(),
                        format!(
                            "privileges `{}` for schema: {:?}",
                            role.name.clone(),
                            role.schemas.clone()
                        ),
                        action.to_string(),
                    ]);
                }
                _ => {}
            }
        }
    }

    // Show Summary
    print_summary(summary);

    Ok(())
}

/// Apply table privileges from config to database
pub fn apply_table_privileges(
    conn: &mut DbConnection,
    config: &Config,
    dryrun: bool,
) -> Result<()> {
    let mut summary = vec![vec![
        "User".to_string(),
        "Table Privileges".to_string(),
        "Action".to_string(),
    ]];
    summary.push(vec![
        "---".to_string(),
        "---".to_string(),
        "---".to_string(),
    ]);

    let user_privileges_on_db = conn.get_user_table_privileges()?;

    // Loop through users in config
    // Get the user Role object by the user.roles[*].name
    // Apply the Role sql privileges to the cluster
    for user in &config.users {
        // Get roles for user from config
        let user_roles_in_config: Vec<_> = user
            .roles
            .iter()
            .map(|role_name| {
                config
                    .roles
                    .iter()
                    .find(|&r| r.get_name() == role_name.to_string())
                    .unwrap()
            })
            .collect();

        let privileges_on_db = user_privileges_on_db.iter().find(|&p| p.name == user.name);

        // Compare privileges on config and db
        // If privileges on config are not in db, add them
        // If privileges on db are not in config, remove them
        for role in user_roles_in_config {
            match role {
                Role::Table(role) => {
                    println!("==> {:?}", role);

                    // revoke
                    if !privileges_on_db.is_none() {
                        let sql = role.to_sql_revoke(user.name.clone());
                        if !dryrun {
                            conn.query(&sql, &[]).unwrap_or_else(|e| {
                                error!("failed to run sql:\n{}", sql);
                                panic!("{}", e);
                            });
                            info!("Revoking: {}", sql);
                        } else {
                            info!("Would revoke: {}", sql);
                        }
                    }

                    // grant
                    let sql = role.to_sql_grant(user.name.clone());
                    if !dryrun {
                        conn.query(&sql, &[]).unwrap_or_else(|e| {
                            error!("failed to run sql:\n{}", sql);
                            panic!("{}", e);
                        });
                        info!("Granting: {}", sql);
                    } else {
                        info!("Granting: {}", sql);
                    }

                    let action = if privileges_on_db.is_none() {
                        "granted"
                    } else {
                        "updated"
                    };

                    // Update summary
                    summary.push(vec![
                        user.name.clone(),
                        format!(
                            "privileges `{}` for table: {:?}",
                            role.name.clone(),
                            role.tables.clone()
                        ),
                        action.to_string(),
                    ]);
                }
                _ => {}
            }
        }
    }

    // Show Summary
    print_summary(summary);

    Ok(())
}

/// Print summary table
/// TODO: Format the table, detect max size to console
fn print_summary(summary: Vec<Vec<String>>) {
    let ascii_table = AsciiTable::default();
    info!("Summary:\n{}", ascii_table.format(summary));
}
