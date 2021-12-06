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

Content of `./examples/example.yaml`:

```yaml
connection:
  type: "postgres"
  url: "postgres://postgres@localhost:5432/postgres"

roles:
  - name: role_database_level
    type: database
    grants:
      - CREATE
      - TEMP
    databases:
      - postgres

  - name: role_schema_level
    type: schema
    grants:
      - CREATE
    databases:
      - postgres
    schemas:
      - public
  - name: role_all_schema
    type: table
    grants:
      - SELECT
      - INSERT
      - UPDATE
    databases:
      - postgres
    schemas:
      - public
    tables:
      - ALL

users:
  - name: duyet
    password: 1234567890
    roles:
      - role_database_level
      - role_all_schema
      - role_schema_level
  - name: duyet2
    password: 1234567890
    roles:
      - role_database_level
      - role_all_schema
      - role_schema_level
```

Apply this config to cluster:

```bash
grant apply --dryrun -f ./examples/example.yaml

[2021-12-06T14:37:03Z INFO  grant::connection] Connected to database: postgres://postgres@localhost:5432/postgres
[2021-12-06T14:37:03Z INFO  grant::apply] Summary:
    ┌────────────┬───────────────────────────┐
    │ User       │ Action                    │
    │ ---        │ ---                       │
    │ duyet      │ update password           │
    │ duyet2     │ update password           │
    └────────────┴───────────────────────────┘

[2021-12-06T14:37:03Z INFO  grant::apply] Summary:
    ┌────────┬───────────────────────────────────────────────────────────┬─────────┐
    │ User   │ Database Privilege                                        │ Action  │
    │ ---    │ ---                                                       │ ---     │
    │ duyet  │ privileges `role_database_level` for database: ["postgre+ │ updated │
    │ duyet2 │ privileges `role_database_level` for database: ["postgre+ │ updated │
    └────────┴───────────────────────────────────────────────────────────┴─────────┘

[2021-12-06T14:37:03Z INFO  grant::apply] Summary:
    ┌────────┬───────────────────────────────────────────────────────┬─────────┐
    │ User   │ Schema Privileges                                     │ Action  │
    │ ---    │ ---                                                   │ ---     │
    │ duyet  │ privileges `role_schema_level` for schema: ["public"] │ updated │
    │ duyet2 │ privileges `role_schema_level` for schema: ["public"] │ updated │
    └────────┴───────────────────────────────────────────────────────┴─────────┘

[2021-12-06T14:37:03Z INFO  grant::apply] Summary:
    ┌────────┬─────────────────────────────────────────────────┬─────────┐
    │ User   │ Table Privileges                                │ Action  │
    │ ---    │ ---                                             │ ---     │
    │ duyet  │ privileges `role_all_schema` for table: ["ALL"] │ updated │
    │ duyet2 │ privileges `role_all_schema` for table: ["ALL"] │ updated │
    └────────┴─────────────────────────────────────────────────┴─────────┘
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
- [x] Support Postgres
- [ ] Visuallization (who can see what?)

# LICENSE

MIT
