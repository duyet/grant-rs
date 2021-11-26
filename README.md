# grant.rs

Manage Redshift database roles and privileges in GitOps style.

# Usage

Install binary from crates.io

```bash
cargo install grant
```

Using `grant` tool:

```bash
$ grant --help

Manage database roles and privileges in GitOps style

USAGE:
    grant <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    apply    Apply changes
    gen      Generate project
    help     Prints this message or the help of the given subcommand(s)
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

# [2021-11-15T03:37:38Z INFO  grant::apply] Try to apply definition from "./examples/example.yaml", dryrun=false, conn=None
# [2021-11-15T03:37:38Z INFO  grant::apply] SQL = GRANT CREATE, TEMP ON DATABASE db1, db2 TO duyet;
# [2021-11-15T03:37:38Z INFO  grant::apply] SQL = GRANT CREATE, USAGE ON SCHEMA  TO duyet;
# [2021-11-15T03:37:38Z INFO  grant::apply] SQL = GRANT CREATE, USAGE ON SCHEMA common, dwh1, dwh2 TO duyet;
# [2021-11-15T03:37:38Z INFO  grant::apply] SQL = GRANT CREATE, TEMP ON DATABASE db1, db2 TO duyet2;
# [2021-11-15T03:37:38Z INFO  grant::apply] SQL = GRANT CREATE, USAGE ON SCHEMA  TO duyet2;
# [2021-11-15T03:37:38Z INFO  grant::apply] SQL = GRANT CREATE, USAGE ON SCHEMA common, dwh1, dwh2 TO duyet2;
```

## Generate random password

```bash
$ grant gen-pass

Generated password: q)ItTjN$EXlkF@Tl
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
