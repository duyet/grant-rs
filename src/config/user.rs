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
    /// Escape single quotes in a string for SQL safety
    fn escape_sql_string(s: &str) -> String {
        s.replace("'", "''")
    }

    /// Validate and format username for SQL (must be valid identifier)
    fn format_username(name: &str) -> Result<String> {
        if name.is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }

        // Check if username contains only valid characters (alphanumeric, underscore, no spaces)
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(anyhow!("Username '{}' contains invalid characters. Only alphanumeric and underscore allowed.", name));
        }

        // Ensure username doesn't start with a number
        if name.chars().next().unwrap().is_numeric() {
            return Err(anyhow!("Username '{}' cannot start with a number", name));
        }

        Ok(name.to_string())
    }

    pub fn to_sql_create(&self) -> Result<String> {
        let username = Self::format_username(&self.name)?;
        let password = match &self.password {
            Some(p) => format!(" WITH PASSWORD '{}'", Self::escape_sql_string(p)),
            None => "".to_string(),
        };

        Ok(format!("CREATE USER {}{};", username, password))
    }

    pub fn to_sql_update(&self) -> Result<String> {
        let username = Self::format_username(&self.name)?;
        let password = match &self.password {
            Some(p) => format!(" WITH PASSWORD '{}'", Self::escape_sql_string(p)),
            None => "".to_string(),
        };

        Ok(format!("ALTER USER {}{};", username, password))
    }

    pub fn to_sql_drop(&self) -> Result<String> {
        let username = Self::format_username(&self.name)?;
        Ok(format!("DROP USER IF EXISTS {};", username))
    }

    pub fn validate(&self) -> Result<()> {
        // Use the format_username validation
        Self::format_username(&self.name)?;
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

        let sql = user.to_sql_create().unwrap();
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

        let sql = user.to_sql_update().unwrap();
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

        let sql = user.to_sql_drop().unwrap();
        assert_eq!(sql, "DROP USER IF EXISTS test;");
    }

    #[test]
    fn test_sql_injection_prevention_password() {
        let user = User {
            name: "test".to_string(),
            password: Some("'; DROP TABLE users; --".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        let sql = user.to_sql_create().unwrap();
        assert_eq!(sql, "CREATE USER test WITH PASSWORD '''; DROP TABLE users; --';");
    }

    #[test]
    fn test_invalid_username_with_spaces() {
        let user = User {
            name: "test user".to_string(),
            password: Some("password".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert!(user.to_sql_create().is_err());
        assert!(user.validate().is_err());
    }

    #[test]
    fn test_invalid_username_starting_with_number() {
        let user = User {
            name: "1test".to_string(),
            password: Some("password".to_string()),
            update_password: Some(true),
            roles: vec!["test".to_string()],
        };

        assert!(user.to_sql_create().is_err());
        assert!(user.validate().is_err());
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
