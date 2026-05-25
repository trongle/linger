use crate::config::TemplatesConfig;
use minijinja::{path_loader, Environment};
use std::{path::Path, sync::Arc};

#[derive(Clone)]
pub struct Templates {
    env: Arc<Environment<'static>>,
}

impl Templates {
    pub fn load(config: &TemplatesConfig) -> anyhow::Result<Self> {
        if !Path::new(&config.path).exists() {
            anyhow::bail!("templates path {:?} does not exist", config.path);
        }
        let mut env = Environment::new();
        env.set_loader(path_loader(&config.path));
        env.add_function("csrf_token", || "");
        env.add_function("is_htmx", || false);
        Ok(Self { env: Arc::new(env) })
    }
    pub fn render(&self, name: &str, ctx: impl serde::Serialize) -> anyhow::Result<String> {
        Ok(self.env.get_template(name)?.render(ctx)?)
    }
    pub fn check(&self) -> anyhow::Result<()> {
        self.env.get_template("layouts/app.html")?;
        self.env.get_template("pages/home/index.html")?;
        Ok(())
    }
}
