//! Email delivery abstraction and SMTP implementation for transactional messages.
//!
//! The API service stores only hashed verification codes. This module receives the
//! short-lived plaintext code at the delivery boundary, renders a user-facing
//! message, and sends it through the configured SMTP submission server without
//! exposing credentials in logs or API responses.

use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
};

use crate::config::{MailConfig, MailSmtpTls};

/// Complete data needed to render and send one registration verification email.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationEmail {
    pub to: String,
    pub code: String,
    pub expires_minutes: i64,
    pub from: String,
    pub subject: String,
}

impl VerificationEmail {
    /// Renders the plaintext body used by the current MVP email verification flow.
    fn plain_text_body(&self) -> String {
        format!(
            "你的寻径星野邮箱验证码是：{}\n\n验证码将在 {} 分钟后失效。若不是你本人操作，请忽略这封邮件。",
            self.code, self.expires_minutes
        )
    }
}

/// Sending interface used by authentication code and tests.
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// Sends the already-generated registration verification code to the recipient.
    async fn send_verification_code(&self, email: VerificationEmail) -> anyhow::Result<()>;
}

/// No-op sender for local tests and configurations that explicitly disable email delivery.
#[derive(Debug, Default)]
pub struct NoopEmailSender;

#[async_trait]
impl EmailSender for NoopEmailSender {
    async fn send_verification_code(&self, _email: VerificationEmail) -> anyhow::Result<()> {
        Ok(())
    }
}

/// SMTP-backed sender for production transactional emails.
#[derive(Clone, Debug)]
pub struct SmtpEmailSender {
    config: MailConfig,
}

impl SmtpEmailSender {
    /// Creates a sender from validated mail configuration.
    pub fn from_config(config: &MailConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }

    fn transport(&self) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
        let tls_parameters = TlsParameters::new(self.config.smtp_host.clone())?;
        let tls = match self.config.smtp_tls {
            MailSmtpTls::Implicit => Tls::Wrapper(tls_parameters),
            MailSmtpTls::StartTls => Tls::Required(tls_parameters),
            MailSmtpTls::None => Tls::None,
        };
        Ok(
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.smtp_host)
                .port(self.config.smtp_port)
                .tls(tls)
                .credentials(Credentials::new(
                    self.config.smtp_username.clone(),
                    self.config.smtp_password.clone(),
                ))
                .build(),
        )
    }

    fn message(&self, email: &VerificationEmail) -> anyhow::Result<Message> {
        let from: Mailbox = email.from.parse()?;
        let to: Mailbox = email.to.parse()?;
        Ok(Message::builder()
            .from(from)
            .to(to)
            .subject(&email.subject)
            .singlepart(SinglePart::plain(email.plain_text_body()))?)
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_verification_code(&self, email: VerificationEmail) -> anyhow::Result<()> {
        let message = self.message(&email)?;
        self.transport()?.send(message).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smtp_message_accepts_chinese_sender_and_subject() {
        let config = MailConfig {
            enabled: true,
            smtp_host: "mx1.stellartrail.cn".to_owned(),
            smtp_port: 465,
            smtp_tls: MailSmtpTls::Implicit,
            smtp_username: "sender@example.test".to_owned(),
            smtp_password: "example-mail-password".to_owned(),
            from: "StellarTrail <sender@example.test>".to_owned(),
            verification_subject: "寻径星野邮箱验证码".to_owned(),
        };
        let sender = SmtpEmailSender::from_config(&config).unwrap();
        let message = sender
            .message(&VerificationEmail {
                to: "trail@example.invalid".to_owned(),
                code: "123456".to_owned(),
                expires_minutes: 10,
                from: config.from.clone(),
                subject: config.verification_subject.clone(),
            })
            .unwrap();

        assert!(message.formatted().starts_with(b"From:"));
    }

    #[test]
    fn verification_email_body_contains_code_and_expiry() {
        let email = VerificationEmail {
            to: "trail@example.invalid".to_owned(),
            code: "123456".to_owned(),
            expires_minutes: 10,
            from: "StellarTrail <sender@example.test>".to_owned(),
            subject: "寻径星野邮箱验证码".to_owned(),
        };

        let body = email.plain_text_body();

        assert!(body.contains("123456"));
        assert!(body.contains("10 分钟"));
        assert!(body.contains("若不是你本人操作"));
    }
}
