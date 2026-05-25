use crate::{error::AppResult, state::AppState};
use lettre::{Message, SmtpTransport, Transport};
use serde_json::json;
use std::{fs, path::PathBuf};

#[derive(Clone)]
pub struct Email {
    state: AppState,
}
impl Email {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    pub async fn send_verify(&self, to: &str, token: &str) -> AppResult<()> {
        self.send_template(to, "Verify your email", "emails/verify_email", json!({"token": token, "url": format!("{}/auth/email/verify/{}", self.state.config().app.base_url, token)})).await
    }
    pub async fn send_reset(&self, to: &str, token: &str) -> AppResult<()> {
        self.send_template(to, "Reset your password", "emails/reset_password", json!({"token": token, "url": format!("{}/auth/password/reset/{}", self.state.config().app.base_url, token)})).await
    }
    async fn send_template(
        &self,
        to: &str,
        subject: &str,
        stem: &str,
        ctx: serde_json::Value,
    ) -> AppResult<()> {
        let html = self
            .state
            .templates()
            .render(&format!("{stem}.html"), &ctx)?;
        let text = self
            .state
            .templates()
            .render(&format!("{stem}.txt"), &ctx)?;
        if !self.state.config().email.enabled {
            return self.write_outbox(to, subject, &text, &html);
        }
        let cfg = &self.state.config().email;
        let msg = Message::builder()
            .from(
                cfg.from
                    .parse()
                    .map_err(|e| crate::AppError::BadRequest(format!("bad from: {e}")))?,
            )
            .to(to
                .parse()
                .map_err(|e| crate::AppError::BadRequest(format!("bad to: {e}")))?)
            .subject(subject)
            .multipart(lettre::message::MultiPart::alternative_plain_html(
                text, html,
            ))
            .map_err(|e| crate::AppError::BadRequest(e.to_string()))?;
        if let Some(host) = &cfg.smtp_host {
            SmtpTransport::relay(host)
                .map_err(|e| crate::AppError::BadRequest(e.to_string()))?
                .port(cfg.smtp_port)
                .build()
                .send(&msg)
                .map_err(|e| crate::AppError::BadRequest(e.to_string()))?;
        }
        Ok(())
    }
    fn write_outbox(&self, to: &str, subject: &str, text: &str, html: &str) -> AppResult<()> {
        let dir: PathBuf = self.state.config().email.outbox_dir.clone();
        fs::create_dir_all(&dir)?;
        fs::write(
            dir.join(format!("{}.eml", uuid::Uuid::new_v4())),
            format!("To: {to}\nSubject: {subject}\n\n{text}\n\n--- html ---\n{html}"),
        )?;
        Ok(())
    }
}
