use crate::config::{Config, Role};
use anyhow::{Result, *};
use log::info;

pub fn apply(config: &Config, dryrun: bool) -> Result<()> {
    info!("Applying configuration: {}", config);

    for user in config.users.iter() {
        let user_name = user.name.clone();

        // Lookup roles for user, error if not found in config::Role
        let user_roles: Vec<_> = user
            .roles
            .iter()
            .map(|role| {
                lookup_role(config, role.to_string())
                    .with_context(|| format!("Role {} not found", role))
            })
            .collect::<Result<Vec<_>>>()?;

        // Loop through roles and generate grant statement for current user
        for role in user_roles {
            let grant_statement = role.to_sql_grant(user_name.clone());

            if dryrun {
                info!("Dry run: {}", grant_statement);
            } else {
                info!("Granting: {}", grant_statement);
            }
        }
    }

    Ok(())
}

// Lookup role from config::Config by role_name
// Return role::Role variant if found or None if not found
fn lookup_role(config: &Config, name: String) -> Result<Role> {
    for role in config.roles.iter() {
        match role {
            Role::Database(role) => {
                if role.name == name {
                    return Ok(Role::Database(role.clone()));
                }
            }
            Role::Schema(role) => {
                if role.name == name {
                    return Ok(Role::Schema(role.clone()));
                }
            }
            Role::Table(role) => {
                if role.name == name {
                    return Ok(Role::Table(role.clone()));
                }
            }
            _ => anyhow::bail!("Unsupported role type"),
        }
    }

    anyhow::bail!("Role {} not found", name);
}
