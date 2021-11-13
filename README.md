# grant.rs

Manage database roles and privileges in GitOps style.

# Usage

Install binary from crates.io

```bash
cargo install grant
```

Using `grant` tool:

```bash
grant --help
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
grant apply --dryrun -f ./duyet-cluster/roles.yaml
```

Apply to cluster:

```bash
grant apply -f ./duyet-cluster/roles.yaml --conn postgres://localhost:5432
```

# Developement

Clone the repo:

```bash
git clone https://github.com/duyet/grant.rs && cd grant.rs 
cargo test
cargo build
```

