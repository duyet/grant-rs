# grant.rs

An open-source project that aims to manage Redshift database roles and privileges in GitOps style, written in Rust.

_This project is still in the early stages of development and is not ready for any kind of production use or any alpha/beta testing._

# Usage

Install binary from crates.io

```bash
cargo install grant
```

Using `grant` tool:

```bash
$ grant --help
grant 0.0.1-beta.1
Manage database roles and privileges in GitOps style

USAGE:
    grant <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    apply       Apply changes
    gen         Generate project
    gen-pass    Generate random password
    help        Prints this message or the help of the given subcommand(s)
    inspect     Inspect current database cluster by config file
    validate    Validate target file
```

## Generate project structure

```bash
grant gen --target duyet-cluster

# or
mkdir duyet-cluster && cd $_
grant gen --target .
```

## Apply privilege changes

Dry run:

```bash
export DB_URL="postgres://postgres:postgres@localhost:5439/postgres"
grant apply --dryrun -f ./examples/example.yaml --conn=$DB_URL
```

Apply to cluster:

```bash
grant apply -f ./examples/example.yaml

[2021-11-29T07:44:08Z INFO  grant::apply] Applying configuration:
    ---
    connection:
      type: postgres
      url: "postgres://postgres@localhost:5432/postgres"
    roles:
      - type: database
        name: role_database_level
        databases:
          - db1
          - db2
        grants:
          - CREATE
          - TEMP
      - type: schema
        name: role_schema_level
        grants:
          - CREATE
        schemas:
          - common
          - dwh1
          - dwh2
      - type: table
        name: role_all_schema
        grants:
          - SELECT
          - INSERT
          - UPDATE
        schemas:
          - common
        tables:
          - ALL
    users:
      - name: duyet
        password: "1234567890"
        roles:
          - role_database_level
          - role_all_schema
          - role_schema_level
      - name: duyet2
        password: "1234567890"
        roles:
          - role_database_level
          - role_all_schema
          - role_schema_level

[2021-11-29T07:44:08Z INFO  grant::apply] User duyet password updated
[2021-11-29T07:44:08Z INFO  grant::apply] User duyet2 password updated
[2021-11-29T07:44:08Z INFO  grant::apply] Summary:
    ┌────────────┬───────────────────────────┐
    │ User       │ Action                    │
    │ ---        │ ---                       │
    │ duyet      │ update password           │
    │ duyet2     │ update password           │
    │ postgres   │ no action (not in config) │
    └────────────┴───────────────────────────┘
```

## Generate random password

```bash
$ grant gen-pass

Generated password: q)ItTjN$EXlkF@Tl
```

## Inspect the current cluster

```bash
$ grant inspect -f examples/example.yaml

[2021-11-29T07:46:44Z INFO  grant::inspect] Current users in postgres://postgres@localhost:5432/postgres:
    ┌────────────┬──────────┬───────┬──────────┐
    │ User       │ CreateDB │ Super │ Password │
    │ ---        │ ---      │ ---   │ ---      │
    │ postgres   │ true     │ true  │ ******** │
    │ duyet      │ false    │ false │ ******** │
    └────────────┴──────────┴───────┴──────────┘
```

# Developement

Clone the repo:

```bash
git clone https://github.com/duyet/grant.rs && cd grant.rs
```

Postgres is required for testing, you might need to use the `docker-compose.yaml`:

```bash
docker-compose up -d
```

Make sure you have connection to `postgres://postgres:postgres@localhost:5432/postgres`.

On the MacOS, the easiest way is install [Postgres.app](https://postgresapp.com).

To run the unittest:

```bash
cargo test
```

# TODO

- [ ] Support store encrypted password in Git
- [ ] Support Postgres
- [ ] Visuallization (who can see what?)

# LICENSE

MIT
