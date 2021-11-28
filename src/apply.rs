use crate::config::{Config, Role, User as UserInConfig};
use crate::connection::{DbConnection, User};
use anyhow::{bail, Result};
use ascii_table::AsciiTable;
use log::info;

pub fn apply(config: &Config, dryrun: bool) -> Result<()> {
    info!("Applying configuration: {}", config);
    let mut conn = DbConnection::connect(config);

    let users_in_db = conn.get_users()?;
    let users_in_config = config.users.clone();
    apply_users(&mut conn, &users_in_db, &users_in_config, dryrun)?;

    //     for user in config.users.iter() {
    //         let user_name = user.name.clone();
    //
    //         // Lookup roles for user, error if not found in config::Role
    //         let user_roles: Vec<_> = user
    //             .roles
    //             .iter()
    //             .map(|role| {
    //                 lookup_role(config, role.to_string())
    //                     .with_context(|| format!("Role {} not found", role))
    //             })
    //             .collect::<Result<Vec<_>>>()?;
    //
    //         // Get user's current permissions on database using config.connection.url
    //         let user_permissions = connection::get_user_permissions(config, &user_name)?;
    //
    //         // Loop through roles and generate grant statement for current user
    //         for role in user_roles {
    //             let grant_statement = role.to_sql_grant(user_name.clone());
    //
    //             if dryrun {
    //                 info!("Dry run: {}", grant_statement);
    //             } else {
    //                 info!("Granting: {}", grant_statement);
    //             }
    //         }
    //     }
    //
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
        let mut user_in_db = users_in_db.iter().find(|&u| u.name == user.name);
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

    // Delete users in db that are not in config
    for user in users_in_db {
        if !users_in_config.iter().any(|u| u.name == user.name) {
            // Update summary
            summary.push(vec![user.name.clone(), "no action (not in config)".to_string()]);
        }
    }

    // Show summary
    let ascii_table = AsciiTable::default();
    info!("Summary:\n{}", ascii_table.format(summary));

    Ok(())
}
