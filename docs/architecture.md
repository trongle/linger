# architecture

Linger documentation for architecture.

- Config is loaded from functional TOML files, then `.env`, environment variables, then CLI overrides.
- Runtime templates live under `templates/`; static assets under `static/`.
- Auth, email, CSRF and generators are intentionally small foundations with TODOs for production hardening.
