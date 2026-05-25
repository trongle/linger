use clap::Args;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub templates: TemplatesConfig,
    pub static_files: StaticConfig,
    pub logging: LoggingConfig,
    pub session: SessionConfig,
    pub auth: AuthConfig,
    pub email: EmailConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub base_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub body_limit_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub auto_migrate: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatesConfig {
    pub path: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticConfig {
    pub path: PathBuf,
    pub mount: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub rust_log: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub cookie_name: String,
    pub secret: String,
    pub secure: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub require_email_verification: bool,
    pub email_verification_ttl_minutes: i64,
    pub password_reset_ttl_minutes: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub from: String,
    pub outbox_dir: PathBuf,
}

#[derive(Debug, Clone, Default, Args)]
pub struct ServeOverrides {
    #[arg(long, env = "HOST")]
    pub host: Option<String>,
    #[arg(long, env = "PORT")]
    pub port: Option<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app: AppConfig {
                name: "Linger".into(),
                base_url: "http://127.0.0.1:3000".into(),
            },
            server: ServerConfig {
                host: "127.0.0.1".into(),
                port: 3000,
                request_timeout_secs: 30,
                body_limit_bytes: 2 * 1024 * 1024,
            },
            database: DatabaseConfig {
                url: "sqlite://tmp/linger.sqlite3".into(),
                auto_migrate: true,
            },
            templates: TemplatesConfig {
                path: "templates".into(),
            },
            static_files: StaticConfig {
                path: "static".into(),
                mount: "/static".into(),
            },
            logging: LoggingConfig {
                rust_log: "linger=debug,tower_http=debug".into(),
            },
            session: SessionConfig {
                cookie_name: "linger_session".into(),
                secret: "change-me-in-env".into(),
                secure: false,
            },
            auth: AuthConfig {
                require_email_verification: true,
                email_verification_ttl_minutes: 60,
                password_reset_ttl_minutes: 15,
            },
            email: EmailConfig {
                enabled: false,
                smtp_host: None,
                smtp_port: 587,
                smtp_username: None,
                smtp_password: None,
                from: "Linger <noreply@example.test>".into(),
                outbox_dir: "tmp/emails".into(),
            },
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Self::load_with_overrides(None)
    }
    pub fn load_with_overrides(overrides: Option<ServeOverrides>) -> anyhow::Result<Self> {
        let mut cfg = Config::default();
        load_file("config/app.toml", &mut cfg.app)?;
        load_file("config/server.toml", &mut cfg.server)?;
        load_file("config/database.toml", &mut cfg.database)?;
        load_file("config/templates.toml", &mut cfg.templates)?;
        load_file("config/static.toml", &mut cfg.static_files)?;
        load_file("config/logging.toml", &mut cfg.logging)?;
        load_file("config/session.toml", &mut cfg.session)?;
        load_file("config/auth.toml", &mut cfg.auth)?;
        load_file("config/email.toml", &mut cfg.email)?;
        let _ = dotenvy::dotenv();
        cfg.apply_env();
        if let Some(o) = overrides {
            if let Some(host) = o.host {
                cfg.server.host = host;
            }
            if let Some(port) = o.port {
                cfg.server.port = port;
            }
        }
        Ok(cfg)
    }
    fn apply_env(&mut self) {
        set(&mut self.app.name, "APP_NAME");
        set(&mut self.app.base_url, "APP_BASE_URL");
        set(&mut self.server.host, "HOST");
        if let Ok(v) = env::var("PORT") {
            if let Ok(p) = v.parse() {
                self.server.port = p;
            }
        }
        set(&mut self.database.url, "DATABASE_URL");
        set(&mut self.session.cookie_name, "SESSION_COOKIE_NAME");
        set(&mut self.session.secret, "SESSION_SECRET");
        if let Ok(v) = env::var("SESSION_SECURE") {
            self.session.secure = matches!(v.as_str(), "1" | "true" | "yes");
        }
        if let Ok(v) = env::var("EMAIL_ENABLED") {
            self.email.enabled = matches!(v.as_str(), "1" | "true" | "yes");
        }
        set(&mut self.email.from, "EMAIL_FROM");
        if let Ok(v) = env::var("SMTP_HOST") {
            self.email.smtp_host = Some(v);
        }
        if let Ok(v) = env::var("SMTP_USERNAME") {
            self.email.smtp_username = Some(v);
        }
        if let Ok(v) = env::var("SMTP_PASSWORD") {
            self.email.smtp_password = Some(v);
        }
        set(&mut self.logging.rust_log, "RUST_LOG");
    }
}

fn set(target: &mut String, key: &str) {
    if let Ok(v) = env::var(key) {
        *target = v;
    }
}
fn load_file<T: for<'de> Deserialize<'de>>(path: &str, target: &mut T) -> anyhow::Result<()> {
    if Path::new(path).exists() {
        *target = toml::from_str(&fs::read_to_string(path)?)?;
    }
    Ok(())
}
