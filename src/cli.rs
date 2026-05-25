use crate::{
    config::{Config, ServeOverrides},
    generator::{self, DomainArgs},
};
use clap::{Args, Parser, Subcommand};
use rand::RngCore;
use std::{fs, path::Path};

#[derive(Parser)]
#[command(
    name = "linger",
    version,
    about = "Self-renaming Rust web app template"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init(InitArgs),
    Serve(ServeOverrides),
    Db {
        #[command(subcommand)]
        command: DbCommand,
    },
    Generate {
        #[command(subcommand)]
        command: GenerateCommand,
    },
    Check {
        #[command(subcommand)]
        command: CheckCommand,
    },
    Test {
        #[command(subcommand)]
        command: TestCommand,
    },
}
#[derive(Subcommand)]
enum DbCommand {
    Migrate,
}
#[derive(Subcommand)]
enum GenerateCommand {
    Domain(DomainArgs),
}
#[derive(Subcommand)]
enum CheckCommand {
    Config,
    Templates,
}
#[derive(Subcommand)]
enum TestCommand {
    Email { to: String },
}
#[derive(Args)]
struct InitArgs {
    name: Option<String>,
    #[arg(long)]
    force: bool,
    #[arg(long, short = 'y')]
    yes: bool,
}

pub async fn run() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Init(args) => init(args).await,
        Command::Serve(overrides) => {
            crate::server::serve(Config::load_with_overrides(Some(overrides))?).await
        }
        Command::Db {
            command: DbCommand::Migrate,
        } => {
            let cfg = Config::load()?;
            let pool = crate::db::connect(&cfg.database).await?;
            crate::db::migrate(&pool).await?;
            println!("migrations complete");
            Ok(())
        }
        Command::Generate {
            command: GenerateCommand::Domain(args),
        } => {
            let files = generator::generate_domain(&args)?;
            for f in files {
                println!("{}", f.display());
            }
            Ok(())
        }
        Command::Check {
            command: CheckCommand::Config,
        } => {
            let cfg = Config::load()?;
            println!(
                "config ok: {} on {}:{}",
                cfg.app.name, cfg.server.host, cfg.server.port
            );
            Ok(())
        }
        Command::Check {
            command: CheckCommand::Templates,
        } => {
            let t = crate::templates::Templates::load(&Config::load()?.templates)?;
            t.check()?;
            println!("templates ok");
            Ok(())
        }
        Command::Test {
            command: TestCommand::Email { to },
        } => {
            let state = crate::state::AppState::new(Config::load()?).await?;
            crate::email::Email::new(state)
                .send_verify(&to, "test-token")
                .await?;
            println!("email rendered/sent to {to}");
            Ok(())
        }
    }
}

async fn init(args: InitArgs) -> anyhow::Result<()> {
    let marker = Path::new(".linger/init.json");
    if marker.exists() && !args.force {
        anyhow::bail!("already initialized; use --force");
    }
    let name = args.name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "linger".into())
    });
    let pkg = normalize_package(&name);
    let app_name = title(&pkg);
    replace_in_file(
        "Cargo.toml",
        "name = \"linger\"",
        &format!("name = \"{pkg}\""),
    )?;
    replace_in_file("README.md", "Linger", &app_name).ok();
    replace_in_file(
        "config/app.toml",
        "name = \"Linger\"",
        &format!("name = \"{app_name}\""),
    )?;
    replace_in_file(
        "config/session.toml",
        "cookie_name = \"linger_session\"",
        &format!("cookie_name = \"{}_session\"", pkg.replace('-', "_")),
    )?;
    replace_in_file(
        "config/database.toml",
        "sqlite://tmp/linger.sqlite3",
        &format!("sqlite://tmp/{pkg}.sqlite3"),
    )?;
    if !Path::new(".env").exists() {
        fs::copy(".env.example", ".env")?;
    }
    let mut env = fs::read_to_string(".env")?;
    env = env.replace(
        "SESSION_SECRET=change-me",
        &format!("SESSION_SECRET={}", random_secret()),
    );
    fs::write(".env", env)?;
    fs::create_dir_all(".linger")?;
    fs::write(marker, format!("{{\"name\":\"{pkg}\"}}\n"))?;
    let _ = std::process::Command::new("cargo").arg("fmt").status();
    let _ = std::process::Command::new("cargo").arg("check").status();
    println!("initialized {pkg}");
    Ok(())
}
fn normalize_package(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
fn title(s: &str) -> String {
    s.split(['-', '_'])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut ch = p.chars();
            match ch.next() {
                Some(c) => c.to_uppercase().collect::<String>() + ch.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn replace_in_file(path: &str, from: &str, to: &str) -> anyhow::Result<()> {
    let s = fs::read_to_string(path)?;
    fs::write(path, s.replace(from, to))?;
    Ok(())
}
fn random_secret() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}
