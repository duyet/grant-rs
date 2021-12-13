# grant.rs [![crates.io](https://img.shields.io/crates/v/grant.svg)](https://crates.io/crates/grant) [![Build & Test](https://github.com/duyet/grant.rs/actions/workflows/build-test.yaml/badge.svg)](https://github.com/duyet/grant.rs/actions/workflows/build-test.yaml)

An open-source project that aims to manage Postgres/Redshift database roles and privileges in GitOps style, written in Rust.

[**Home**](https://github.com/duyet/grant.rs) | [**Documentation**](https://docs.rs/grant)

_This project is still in the early stages of development and is not ready for any kind of production use or any alpha/beta testing._

| Level      | Supported | Description                                                                                                                                                                                                                                            |
| ---------- | :-------: | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `DATABASE` |     ✓     | Support grant `CREATE`\| `TEMP` \| `ALL` on database(s) to user                                                                                                                                                                                        |
| `SCHEMA`   |     ✓     | Support grant `CREATE` \| `USAGE` \| `ALL` on schema(s) to user                                                                                                                                                                                        |
| `TABLE`    |     ✓     | Support grant `SELECT` \| `INSERT` \| `UPDATE` \| `DELETE` \| `DROP` \| `REFERENCES` \| `ALL` on tables(s) or `ALL` tables in schema(s) to user. <br> Supported excluding table(s) by adding `-` before the table name (e.g. `tables: [ALL, -table]`). |
| `FUNCTION` |           | Not supported yet                                                                                                                                                                                                                                      |

<!-- edit in https://www.tablesgenerator.com/markdown_tables -->

# Installation

Install via Homebrew:

```bash
brew tap duyet/tap
brew install grant
```

Install binary from crates.io via Cargo:

```bash
cargo install grant
```

# Usage

Using `grant` tool:

```bash
$ grant --help

grant 0.0.1-beta.3
Manage database roles and privileges in GitOps style

USAGE:
    grant <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    apply       Apply a configuration to a redshift by file name. Yaml format are accepted
    gen         Generate sample configuration file
    gen-pass    Generate random password
    help        Prints this message or the help of the given subcommand(s)
    inspect     Inspect current database cluster with connection info from configuration file
    validate    Validate a configuration file or a target directory that contains configuration files
```

## Generate project structure

```bash
grant gen --target ./cluster

Creating path: "./cluster"
Generated: "./cluster/config.yml"
```

## Apply privilege changes

Content of `./examples/example.yaml`:

```yaml
connection:
  type: "postgres"
  # support environment variables, e.g. postgres://${HOSTNAME}:5432
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
      - ALL # include all table
      - +public_table # can add `+` to mark included tables (public.public_table)
      - -secret_table # add `-` to exclude this table (public.secret_table)
      - -schema2.table # exclude schema2.table

users:
  - name: duyet
    password: 1234567890 # password in plaintext
    roles:
      - role_database_level
      - role_all_schema
      - role_schema_level
  - name: duyet2
    password: md58243e8f5dfb84bbd851de920e28f596f # support md5 style: grant gen-pass -u duyet2
    roles:
      - role_database_level
      - role_all_schema
      - role_schema_level
```

Apply this config to cluster:

```bash
grant apply -f ./examples/example.yaml

[2021-12-06T14:37:03Z INFO  grant::connection] Connected to database: postgres://postgres@localhost:5432/postgres
[2021-12-06T14:37:03Z INFO  grant::apply] Summary:
    ┌────────────┬────────────────────────────┐
    │ User       │ Action                     │
    │ ---        │ ---                        │
    │ duyet      │ no action (already exists) │
    │ duyet2     │ no action (already exists) │
    └────────────┴────────────────────────────┘
[2021-12-12T13:48:22Z INFO  grant::apply] Success: GRANT CREATE, TEMP ON DATABASE postgres TO duyet;
[2021-12-12T13:48:22Z INFO  grant::apply] Success: GRANT CREATE ON SCHEMA public TO duyet;
[2021-12-12T13:48:22Z INFO  grant::apply] Success: GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO duyet;
[2021-12-12T13:48:22Z INFO  grant::apply] Success: GRANT CREATE, TEMP ON DATABASE postgres TO duyet2;
[2021-12-12T13:48:22Z INFO  grant::apply] Success: GRANT CREATE ON SCHEMA public TO duyet2;
[2021-12-12T13:48:22Z INFO  grant::apply] Success: GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO duyet2;
[2021-12-12T13:48:22Z INFO  grant::apply] Summary:
    ┌────────┬─────────────────────┬──────────────────────┬─────────┐
    │ User   │ Role Name           │ Detail               │ Status  │
    │ ---    │ ---                 │ ---                  │ ---     │
    │ duyet  │ role_database_level │ database["postgres"] │ updated │
    │ duyet  │ role_schema_level   │ schema["public"]     │ updated │
    │ duyet  │ role_table_level    │ table["ALL"]         │ updated │
    │ duyet2 │ role_database_level │ database["postgres"] │ updated │
    │ duyet2 │ role_schema_level   │ schema["public"]     │ updated │
    │ duyet2 │ role_table_level    │ table["ALL"]         │ updated │
    └────────┴─────────────────────┴──────────────────────┴─────────┘
```

## Generate random password

```bash
$ grant gen-pass

Generated password: q)ItTjN$EXlkF@Tl
```

```bash
$ grant gen-pass --user duyet

Generated password: o^b3aD1L$xLm%#~U
Generated MD5 (user: duyet): md58243e8f5dfb84bbd851de920e28f596f
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

- [x] Support reading connection info from environment variables
- [ ] Support store encrypted password in Git
- [x] Support Postgres and Redshift
- [ ] Support change password
- [ ] Visuallization (who can see what?)
- [ ] Apply show more detail about diff changes
- [ ] Inspect show more detail about user privileges
- [ ] Command to rotate passwords

# LICENSE

MIT
