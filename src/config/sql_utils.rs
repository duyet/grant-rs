//! SQL utility functions for safe query construction.
//!
//! This module provides functions to safely escape and quote SQL identifiers
//! and string literals to prevent SQL injection vulnerabilities.

/// Escape and quote a PostgreSQL identifier to prevent SQL injection.
///
/// PostgreSQL identifiers (table names, column names, role names, etc.) must be
/// quoted with double quotes and any internal double quotes must be escaped by
/// doubling them.
///
/// # Examples
///
/// ```
/// use grant::config::sql_utils::escape_identifier;
///
/// assert_eq!(escape_identifier("users"), "\"users\"");
/// assert_eq!(escape_identifier("my\"table"), "\"my\"\"table\"");
/// assert_eq!(escape_identifier("role'name"), "\"role'name\"");
/// ```
///
/// # Security
///
/// This function prevents SQL injection by ensuring that user-provided
/// identifiers cannot break out of their quoted context.
pub fn escape_identifier(ident: &str) -> String {
    // PostgreSQL identifiers are quoted with double quotes
    // Escape double quotes by doubling them
    format!("\"{}\"", ident.replace("\"", "\"\""))
}

/// Escape a string literal for use in SQL queries.
///
/// PostgreSQL string literals are quoted with single quotes and any internal
/// single quotes must be escaped by doubling them.
///
/// # Examples
///
/// ```
/// use grant::config::sql_utils::escape_sql_string;
///
/// assert_eq!(escape_sql_string("password"), "password");
/// assert_eq!(escape_sql_string("pass'word"), "pass''word");
/// assert_eq!(escape_sql_string("it's"), "it''s");
/// ```
///
/// # Security
///
/// This function prevents SQL injection in string literals by escaping
/// single quotes that could terminate the string.
pub fn escape_sql_string(s: &str) -> String {
    s.replace("'", "''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_identifier_simple() {
        assert_eq!(escape_identifier("users"), "\"users\"");
        assert_eq!(escape_identifier("my_table"), "\"my_table\"");
    }

    #[test]
    fn test_escape_identifier_with_quotes() {
        assert_eq!(escape_identifier("my\"table"), "\"my\"\"table\"");
        assert_eq!(
            escape_identifier("\"already\"quoted\""),
            "\"\"\"already\"\"quoted\"\"\""
        );
    }

    #[test]
    fn test_escape_identifier_with_single_quotes() {
        // Single quotes don't need escaping in identifiers
        assert_eq!(escape_identifier("role'name"), "\"role'name\"");
    }

    #[test]
    fn test_escape_sql_string_simple() {
        assert_eq!(escape_sql_string("password"), "password");
        assert_eq!(escape_sql_string("simple"), "simple");
    }

    #[test]
    fn test_escape_sql_string_with_quotes() {
        assert_eq!(escape_sql_string("pass'word"), "pass''word");
        assert_eq!(escape_sql_string("it's"), "it''s");
        assert_eq!(escape_sql_string("'quoted'"), "''quoted''");
    }

    #[test]
    fn test_escape_sql_string_with_double_quotes() {
        // Double quotes don't need escaping in string literals
        assert_eq!(escape_sql_string("my\"value"), "my\"value");
    }
}
