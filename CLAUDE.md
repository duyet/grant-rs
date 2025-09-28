# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

grant-rs is a CLI tool written in Rust for managing PostgreSQL/Redshift database roles and privileges in GitOps style. It allows declarative configuration of database permissions through YAML files.

## Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Run tests (requires PostgreSQL running)
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Install locally
cargo install --path .
```

### Database Setup for Testing
```bash
# Start PostgreSQL via Docker
docker-compose up -d

# Connect to test database (required for tests)
# Connection: postgres://postgres:postgres@localhost:5432/postgres
```

### Running the CLI Tool
```bash
# Generate sample configuration
cargo run -- gen --target ./cluster

# Validate configuration
cargo run -- validate -f examples/example.yaml

# Apply configuration to database
cargo run -- apply -f examples/example.yaml

# Inspect current database state
cargo run -- inspect -f examples/example.yaml

# Generate random password
cargo run -- gen-pass --user username
```

## Architecture

### Core Modules
- **config/**: Configuration parsing and validation
  - `connection.rs` - Database connection configuration
  - `role_*.rs` - Role definitions (database, schema, table levels)
  - `user.rs` - User management and password handling
- **apply.rs** - Main logic for applying privilege changes to database
- **inspect.rs** - Database introspection and current state analysis
- **validate.rs** - Configuration file validation
- **gen.rs** - Configuration generation utilities
- **connection.rs** - Database connection management
- **cli.rs** - Command-line interface definitions

### Key Design Patterns
- Configuration-driven: All database changes defined in YAML files
- Three privilege levels: DATABASE, SCHEMA, TABLE with inheritance
- GitOps approach: Version-controlled privilege management
- Idempotent operations: Safe to run multiple times
- Support for include/exclude patterns in table grants (+ and - prefixes)

### Dependencies
- `structopt` for CLI parsing
- `postgres` for database connectivity
- `serde`/`serde_yaml` for configuration parsing
- `anyhow` for error handling
- `log`/`env_logger` for logging

## Testing Strategy

Tests require a running PostgreSQL instance. The project includes:
- Unit tests for configuration parsing
- Integration tests for database operations
- Example configurations in `examples/` directory
- Development database via `docker-compose.yaml`

## Configuration Format

The tool uses YAML configuration with three main sections:
- `connection`: Database connection details (supports environment variables)
- `roles`: Privilege definitions at database/schema/table levels
- `users`: User accounts and role assignments

Supports password encryption (MD5) and environment variable substitution in connection strings.