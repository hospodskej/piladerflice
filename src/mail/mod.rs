//! E-mails: the order confirmation for the customer, and notifications of
//! new orders and quote requests for the business.
//!
//! - this file: [`Email`] and the [`Mailer`] that sends it
//! - [`settings`]: SMTP server and addresses, from the environment
//! - [`orders`], [`inquiries`]: each e-mail's data and plain text; the
//!   HTML is in `ui::mail`

pub mod inquiries;
pub mod orders;
pub mod settings;

use std::sync::{Arc, Mutex};

use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

pub use settings::MailSettings;

use crate::AppState;
use crate::locale::Locale;
use crate::ui::{Ui, render};

/// An e-mail's HTML document, rendered in `locale`.
fn render_html<F, V>(state: &AppState, locale: Locale, body: F) -> String
where
    F: FnOnce() -> V + Send + 'static,
    V: leptos::prelude::IntoView + 'static,
{
    use leptos::prelude::*;
    let ui = Ui::with(state, locale, String::new(), String::new());
    render::document(
        ui,
        move || view! { <crate::ui::mail::MailLayout>{body()}</crate::ui::mail::MailLayout> },
    )
}

/// An e-mail ready to send: HTML with a plain text alternative.
#[derive(Clone, Debug)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
}

/// Sends e-mails through an SMTP server, or, without one (development,
/// tests), keeps them in an outbox and logs them.
pub enum Mailer {
    Smtp {
        // Boxed: the transport is much bigger than the outbox.
        transport: Box<AsyncSmtpTransport<Tokio1Executor>>,
        from: Mailbox,
    },
    Outbox(Arc<Mutex<Vec<Email>>>),
}

impl Mailer {
    pub fn from_settings(settings: &MailSettings) -> anyhow::Result<Mailer> {
        let Some(smtp) = &settings.smtp else {
            return Ok(Mailer::Outbox(Arc::default()));
        };
        let builder = if smtp.port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp.address)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.address)?
        };
        let transport = builder
            .port(smtp.port)
            .credentials(Credentials::new(smtp.username.clone(), smtp.password.clone()))
            .build();
        Ok(Mailer::Smtp {
            transport: Box::new(transport),
            from: settings.from.parse()?,
        })
    }

    /// Sends in the background: the visitor doesn't wait for the mail
    /// server, and a failure is logged (the order is saved either way).
    pub fn send(&self, email: Email) {
        match self {
            Mailer::Smtp { transport, from } => {
                let message = match build(from.clone(), &email) {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::error!("e-mail \"{}\" to {} not built: {error:#}", email.subject, email.to);
                        return;
                    }
                };
                let transport = (**transport).clone();
                tokio::spawn(async move {
                    if let Err(error) = transport.send(message).await {
                        tracing::error!("e-mail \"{}\" to {} not sent: {error}", email.subject, email.to);
                    }
                });
            }
            Mailer::Outbox(outbox) => {
                tracing::info!(
                    "e-mail (not sent, no SMTP configured) to {}: {}",
                    email.to,
                    email.subject
                );
                outbox.lock().expect("outbox").push(email);
            }
        }
    }

    /// The e-mails kept so far (no SMTP); empty when sending for real.
    pub fn outbox(&self) -> Vec<Email> {
        match self {
            Mailer::Outbox(outbox) => outbox.lock().expect("outbox").clone(),
            Mailer::Smtp { .. } => Vec::new(),
        }
    }
}

fn build(from: Mailbox, email: &Email) -> anyhow::Result<Message> {
    Ok(Message::builder()
        .from(from)
        .to(email.to.parse()?)
        .subject(&email.subject)
        .multipart(MultiPart::alternative_plain_html(
            email.text.clone(),
            email.html.clone(),
        ))?)
}
