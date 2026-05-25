use crate::config::Config;

pub fn init(config: &Config) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| config.logging.rust_log.clone().into());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
