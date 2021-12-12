use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    // password is optional
    pub password: Option<String>,
    pub roles: Vec<String>,
}

impl User {
    pub fn to_sql_create(&self) -> String {
        let password = match &self.password {
            Some(p) => format!(" WITH PASSWORD '{}'", p),
            None => "".to_string(),
        };

        format!("CREATE USER {}{};", self.name, password)
    }

    pub fn to_sql_drop(&self) -> String {
        let sql = format!("DROP USER IF EXISTS {};", self.name);
        sql
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("user name is empty"));
        }

        Ok(())
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_password(&self) -> String {
        match &self.password {
            Some(p) => p.clone(),
            None => "".to_string(),
        }
    }

    pub fn get_roles(&self) -> Vec<String> {
        self.roles.clone()
    }
}
