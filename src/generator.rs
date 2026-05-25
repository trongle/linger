use clap::Args;
use regex::Regex;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Args)]
pub struct DomainArgs {
    pub name: String,
    pub fields: Vec<String>,
    #[arg(long)]
    pub crud: bool,
    #[arg(long)]
    pub no_web: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long, short = 'y')]
    pub yes: bool,
}

pub fn normalize_domain(name: &str) -> String {
    let mut out = String::new();
    let mut prev_was_sep = true;
    let mut prev_was_lower_or_digit = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() && prev_was_lower_or_digit && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_was_sep = false;
            prev_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else if !prev_was_sep && !out.ends_with('_') {
            out.push('_');
            prev_was_sep = true;
            prev_was_lower_or_digit = false;
        }
    }

    let base = out.trim_matches('_').to_string();
    pluralize_snake(&base)
}

fn pluralize_snake(name: &str) -> String {
    if name.ends_with('s') {
        name.to_string()
    } else if let Some(stem) = name.strip_suffix('y') {
        format!("{stem}ies")
    } else if name.ends_with("ch")
        || name.ends_with("sh")
        || name.ends_with('x')
        || name.ends_with('z')
    {
        format!("{name}es")
    } else {
        format!("{name}s")
    }
}

pub fn generate_domain(args: &DomainArgs) -> anyhow::Result<Vec<PathBuf>> {
    if args.crud && args.no_web {
        anyhow::bail!("--crud conflicts with --no-web");
    }
    let name = normalize_domain(&args.name);
    let dir = PathBuf::from("src").join(&name);
    if dir.exists() {
        anyhow::bail!("domain directory already exists: {}", dir.display());
    }

    let mut files = vec![
        dir.join("mod.rs"),
        dir.join("routes.rs"),
        dir.join("handlers.rs"),
        dir.join("repository.rs"),
    ];
    if args.crud {
        files.push(dir.join("model.rs"));
        files.push(migration_path(&name));
    }
    if !args.no_web {
        let page_dir = PathBuf::from("templates/pages").join(&name);
        files.push(page_dir.join("index.html"));
        if args.crud {
            files.push(page_dir.join("show.html"));
            files.push(page_dir.join("new.html"));
            files.push(page_dir.join("edit.html"));
            files.push(page_dir.join("_form.html"));
            files.push(page_dir.join("_row.html"));
        }
    }

    if args.dry_run {
        return Ok(files);
    }

    fs::create_dir_all(&dir)?;
    fs::write(dir.join("mod.rs"), module_source(args.crud))?;
    fs::write(
        dir.join("routes.rs"),
        routes_source(&name, args.crud, args.no_web),
    )?;
    fs::write(dir.join("handlers.rs"), handlers_source(&name, args.crud))?;
    fs::write(
        dir.join("repository.rs"),
        repository_source(&name, args.crud),
    )?;
    if args.crud {
        fs::write(dir.join("model.rs"), model_source(&name, &args.fields))?;
        let migration = migration_path(&name);
        if let Some(parent) = migration.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(migration, migration_source(&name, &args.fields))?;
    }

    if !args.no_web {
        let page_dir = PathBuf::from("templates/pages").join(&name);
        fs::create_dir_all(&page_dir)?;
        fs::write(page_dir.join("index.html"), page_source(&name, "index"))?;
        if args.crud {
            fs::write(page_dir.join("show.html"), page_source(&name, "show"))?;
            fs::write(page_dir.join("new.html"), page_source(&name, "new"))?;
            fs::write(page_dir.join("edit.html"), page_source(&name, "edit"))?;
            fs::write(page_dir.join("_form.html"), format!("<form method=\"post\">\n  <input type=\"hidden\" name=\"csrf_token\" value=\"{{{{ csrf_token }}}}\">\n  <!-- TODO: add {name} fields -->\n  <button type=\"submit\">Save</button>\n</form>\n"))?;
            fs::write(
                page_dir.join("_row.html"),
                format!("<tr><td>{{{{ item.id }}}}</td><td>{name}</td></tr>\n"),
            )?;
        }
    }

    append_unique("src/lib.rs", &format!("pub mod {name};\n"))?;
    append_unique(
        "src/router.rs",
        &format!(
            "\n// TODO: wire generated domain route: .merge(crate::{name}::routes::routes())\n"
        ),
    )?;
    append_unique(
        "src/repositories.rs",
        &format!("\n// TODO: add {name} repository factory when schema is finalized.\n"),
    )?;
    Ok(files)
}

fn migration_path(name: &str) -> PathBuf {
    PathBuf::from("migrations").join(format!(
        "{}_create_{name}.sql",
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    ))
}

fn module_source(crud: bool) -> String {
    let model = if crud { "pub mod model;\n" } else { "" };
    format!("pub mod handlers;\n{model}pub mod repository;\npub mod routes;\n")
}

fn routes_source(name: &str, crud: bool, no_web: bool) -> String {
    if no_web {
        return "use axum::Router;\npub fn routes() -> Router<crate::state::AppState> { Router::new() }\n".into();
    }
    if crud {
        format!("use axum::{{routing::{{get, post}}, Router}};\nuse super::handlers;\n\npub fn routes() -> Router<crate::state::AppState> {{\n    Router::new()\n        .route(\"/{name}\", get(handlers::index).post(handlers::create))\n        .route(\"/{name}/new\", get(handlers::new))\n        .route(\"/{name}/:id\", get(handlers::show).post(handlers::update))\n        .route(\"/{name}/:id/edit\", get(handlers::edit))\n        .route(\"/{name}/:id/delete\", post(handlers::delete))\n}}\n")
    } else {
        format!("use axum::{{routing::get, Router}};\nuse super::handlers;\n\npub fn routes() -> Router<crate::state::AppState> {{\n    Router::new().route(\"/{name}\", get(handlers::index))\n}}\n")
    }
}

fn repository_source(name: &str, crud: bool) -> String {
    let extra = if crud {
        "\n    // TODO: implement list/find/create/update/soft_delete queries.\n"
    } else {
        "\n    // TODO: implement domain queries.\n"
    };
    format!("use sqlx::SqlitePool;\n\n#[derive(Clone)]\npub struct Repository {{\n    pool: SqlitePool,\n}}\n\nimpl Repository {{\n    pub fn new(pool: SqlitePool) -> Self {{\n        Self {{ pool }}\n    }}{extra}}}\n\n// Domain: {name}\n")
}

fn handlers_source(name: &str, crud: bool) -> String {
    if crud {
        format!("use crate::{{error::AppResult, view::View}};\nuse axum::{{extract::Path, response::IntoResponse}};\nuse serde::Deserialize;\nuse serde_json::json;\n\n#[derive(Debug, Deserialize)]\npub struct CreateInput {{\n    pub csrf_token: String,\n    // TODO: add fields\n}}\n\n#[derive(Debug, Deserialize)]\npub struct UpdateInput {{\n    pub csrf_token: String,\n    // TODO: add fields\n}}\n\npub async fn index(view: View) -> AppResult<impl IntoResponse> {{\n    view.render(\"pages/{name}/index.html\", json!({{\"items\": []}}))\n}}\n\npub async fn new(view: View) -> AppResult<impl IntoResponse> {{\n    view.render(\"pages/{name}/new.html\", json!({{}}))\n}}\n\npub async fn show(Path(id): Path<i64>, view: View) -> AppResult<impl IntoResponse> {{\n    view.render(\"pages/{name}/show.html\", json!({{\"id\": id}}))\n}}\n\npub async fn edit(Path(id): Path<i64>, view: View) -> AppResult<impl IntoResponse> {{\n    view.render(\"pages/{name}/edit.html\", json!({{\"id\": id}}))\n}}\n\npub async fn create() -> AppResult<impl IntoResponse> {{\n    Ok(axum::response::Redirect::to(\"/{name}\"))\n}}\n\npub async fn update(Path(_id): Path<i64>) -> AppResult<impl IntoResponse> {{\n    Ok(axum::response::Redirect::to(\"/{name}\"))\n}}\n\npub async fn delete(Path(_id): Path<i64>) -> AppResult<impl IntoResponse> {{\n    Ok(axum::response::Redirect::to(\"/{name}\"))\n}}\n")
    } else {
        format!("use crate::{{error::AppResult, view::View}};\nuse axum::response::IntoResponse;\nuse serde_json::json;\n\npub async fn index(view: View) -> AppResult<impl IntoResponse> {{\n    view.render(\"pages/{name}/index.html\", json!({{}}))\n}}\n")
    }
}

fn model_source(name: &str, fields: &[String]) -> String {
    let mut body = String::from("use chrono::{DateTime, Utc};\nuse serde::{Deserialize, Serialize};\n\n#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]\npub struct Row {\n    pub id: i64,\n");
    for field in fields {
        if let Some((field_name, rust_ty)) = parse_field(field) {
            body.push_str(&format!("    pub {field_name}: {rust_ty},\n"));
        }
    }
    body.push_str("    pub created_at: DateTime<Utc>,\n    pub updated_at: DateTime<Utc>,\n    pub deleted_at: Option<DateTime<Utc>>,\n}\n");
    body.push_str(&format!(
        "\n// Domain model for {name}. Rename Row to a domain-specific type when desired.\n"
    ));
    body
}

fn migration_source(name: &str, fields: &[String]) -> String {
    let mut sql =
        format!("CREATE TABLE IF NOT EXISTS {name} (\n    id INTEGER PRIMARY KEY AUTOINCREMENT,\n");
    for field in fields {
        if let Some((field_name, sql_ty)) = parse_sql_field(field) {
            sql.push_str(&format!("    {field_name} {sql_ty},\n"));
        }
    }
    sql.push_str("    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,\n    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,\n    deleted_at TEXT\n);\n");
    sql
}

fn parse_field(raw: &str) -> Option<(String, String)> {
    let (name, rest) = raw.split_once(':')?;
    let nullable = rest.contains('?');
    let ty = rest.split(':').next().unwrap_or(rest).trim_end_matches('?');
    let rust_ty = match ty {
        "string" | "text" => "String",
        "bool" => "bool",
        "int" => "i32",
        "bigint" => "i64",
        "float" => "f64",
        "datetime" => "DateTime<Utc>",
        _ => "String",
    };
    let rust_ty = if nullable {
        format!("Option<{rust_ty}>")
    } else {
        rust_ty.to_string()
    };
    Some((sanitize_ident(name), rust_ty))
}

fn parse_sql_field(raw: &str) -> Option<(String, String)> {
    let (name, rest) = raw.split_once(':')?;
    let nullable = rest.contains('?');
    let default = rest.split(":default=").nth(1);
    let ty = rest.split(':').next().unwrap_or(rest).trim_end_matches('?');
    let sql_ty = match ty {
        "string" | "text" | "datetime" => "TEXT",
        "bool" | "int" | "bigint" => "INTEGER",
        "float" => "REAL",
        _ => "TEXT",
    };
    let mut col = sql_ty.to_string();
    if !nullable {
        col.push_str(" NOT NULL");
    }
    if let Some(default) = default {
        col.push_str(" DEFAULT ");
        if matches!(ty, "string" | "text" | "datetime") {
            col.push('\'');
            col.push_str(&default.replace('\'', "''"));
            col.push('\'');
        } else {
            col.push_str(default);
        }
    }
    Some((sanitize_ident(name), col))
}

fn sanitize_ident(name: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9_]").unwrap();
    let mut s = re.replace_all(name, "_").to_lowercase();
    if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        s = format!("field_{s}").into();
    }
    s
}

fn page_source(name: &str, action: &str) -> String {
    format!("{{% extends \"layouts/app.html\" %}}\n{{% block title %}}{action} {name}{{% endblock %}}\n{{% block content %}}\n<section class=\"card\">\n  <h1>{action} {name}</h1>\n  <p class=\"muted\">Generated by Linger.</p>\n</section>\n{{% endblock %}}\n")
}

fn append_unique(path: &str, text: &str) -> anyhow::Result<()> {
    let old = fs::read_to_string(path)?;
    if !old.contains(text.trim()) {
        fs::write(path, format!("{old}{text}"))?;
    }
    Ok(())
}
