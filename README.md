# Linger

Linger is a self-renaming Rust web application template built with Axum, Minijinja, SQLx SQLite, Clap, CSRF, auth scaffolding, email outbox/SMTP, and a domain generator.

## Quick start

```bash
cp .env.example .env
cargo run -- db migrate
cargo run -- serve
```

Initialize a cloned template:

```bash
cargo run -- init my-app --yes
```

See `docs/` for details. License: `MIT OR Apache-2.0`.
