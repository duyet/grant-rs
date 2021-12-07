//! This crate is an open-source project that aims to manage Redshift database roles and privileges in GitOps style, written in Rust.
//!
//! Github: [https://github.com/duyet/grant.rs](https://github.com/duyet/grant.rs)
//!
//! ## Features
//!
//! ```bash
//! grant 0.0.1-beta.2
//! Manage database roles and privileges in GitOps style
//!
//! USAGE:
//!     grant <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     apply       Apply changes
//!     gen         Generate project
//!     gen-pass    Generate random password
//!     help        Prints this message or the help of the given subcommand(s)
//!     inspect     Inspect current database cluster by config file
//!     validate    Validate target file
//! ```
//!
//! ## Examples
//!
//! Generate a config file:
//!
//! ```bash
//! grant gen --config-file=./my-database.yaml
//! ```
//!
//! Open the config file in your editor and configure the database `roles` and `users`:
//!
//! ```yaml
//! connection:
//!   type: "postgres"
//!   url: "postgres://postgres@localhost:5432/postgres"
//!
//! roles:
//!   - name: role_database_level
//!     type: database
//!     grants:
//!       - CREATE
//!       - TEMP
//!     databases:
//!       - postgres
//!
//!   - name: role_schema_level
//!     type: schema
//!     grants:
//!       - CREATE
//!     databases:
//!       - postgres
//!     schemas:
//!       - public
//!   - name: role_all_schema
//!     type: table
//!     grants:
//!       - SELECT
//!       - INSERT
//!       - UPDATE
//!     databases:
//!       - postgres
//!     schemas:
//!       - public
//!     tables:
//!       - ALL
//!
//! users:
//!   - name: duyet
//!     password: 1234567890
//!     roles:
//!       - role_database_level
//!       - role_all_schema
//!       - role_schema_level
//!   - name: duyet2
//!     password: 1234567890
//!     roles:
//!       - role_database_level
//!       - role_all_schema
//!       - role_schema_level
//! ```
//!
//! Apply users and database privileges to the database:
//!
//! ```bash
//! $ grant apply --config-file=./my-database.yaml
//!
//! [2021-12-07T06:46:48Z INFO  grant::connection] Connected to database: postgres://postgres@localhost:5432/postgres
//! [2021-12-07T06:46:48Z INFO  grant::apply] Revoking: REVOKE CREATE, TEMP ON DATABASE postgres FROM duyet;
//! [2021-12-07T06:46:48Z INFO  grant::apply] Granting: GRANT CREATE, TEMP ON DATABASE postgres TO duyet;
//! [2021-12-07T06:46:48Z INFO  grant::apply] ==> RoleDatabaseLevel { name: "role_database_level", databases: ["postgres"], grants: ["CREATE", "TEMP"] }
//! [2021-12-07T06:46:48Z INFO  grant::apply] Revoking: REVOKE CREATE, TEMP ON DATABASE postgres FROM duyet2;
//! [2021-12-07T06:46:48Z INFO  grant::apply] Granting: GRANT CREATE, TEMP ON DATABASE postgres TO duyet2;
//! ...
//! ```
//!
//! ## License
//!
//! Licensed under the MIT License.
//! See the LICENSE file in the project root for license information.

pub mod apply;
pub mod cli;
pub mod config;
pub mod connection;
pub mod gen;
