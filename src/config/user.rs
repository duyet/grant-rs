use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub name: String,
    // password is optional
    pub password: Option<String>,
    // Need to update password at anytime? by default is false
    pub update_password: Option<bool>,
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

    pub fn to_sql_update(&self) -> String {
        let password = match &self.password {
            Some(p) => format!(" WITH PASSWORD '{}'", p),
            None => "".to_string(),
        };

        format!("ALTER USER {}{};", self.name, password)
    }

    pub fn to_sql_drop(&self) -> String {
        format!("DROP USER IF EXISTS {};", self.name)
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

// Test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_to_sql_create() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        let sql = user.to_sql_create();
        assert_eq!(sql, "CREATE USER test WITH PASSWORD 'test';");
    }

    #[test]
    fn test_user_to_sql_update() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        let sql = user.to_sql_update();
        assert_eq!(sql, "ALTER USER test WITH PASSWORD 'test';");
    }

    #[test]
    fn test_user_to_sql_drop() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        let sql = user.to_sql_drop();
        assert_eq!(sql, "DROP USER IF EXISTS test;");
    }

    #[test]
    fn test_user_validate() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_validate_empty_name() {
        let user = User {
            name: "".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert!(user.validate().is_err());
    }

    #[test]
    fn test_user_validate_empty_password() {
        let user = User {
            name: "test".to_string(),
            password: None,
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_validate_empty_roles() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec![],
        };

        assert!(user.validate().is_ok());
    }

    #[test]
    fn test_user_get_name() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert_eq!(user.get_name(), "test");
    }

    #[test]
    fn test_user_get_password() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert_eq!(user.get_password(), "test");
    }

    #[test]
    fn test_user_get_roles() {
        let user = User {
            name: "test".to_string(),
            password: Some("test".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert_eq!(user.get_roles(), vec!["test".to_string()]);
    }
}
