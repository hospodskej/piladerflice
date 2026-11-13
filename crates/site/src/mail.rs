//! E-mails: order confirmations, notifications to the sawmill, password
//! resets. Sent over SMTP (SMTP_ADDRESS, SMTP_PORT, SMTP_USERNAME,
//! SMTP_PASSWORD; port 465 is TLS, anything else STARTTLS). Without
//! SMTP_ADDRESS (development) a message is written to the log instead.

use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

/// One e-mail: plain text, and HTML when there is any.
#[derive(Clone, Debug)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    pub text: String,
    pub html: Option<String>,
}

/// Who the site's e-mails come from.
fn from_address() -> String {
    std::env::var("MAIL_FROM_ADDRESS").unwrap_or_else(|_| "objednavky@piladerflice.cz".into())
}

/// Where new orders and inquiries are announced (ORDER_NOTIFICATION_EMAIL;
/// INQUIRY_NOTIFICATION_EMAIL for inquiries if set).
pub fn notification_address(inquiry: bool) -> Option<String> {
    let inquiry_address = inquiry.then(|| std::env::var("INQUIRY_NOTIFICATION_EMAIL").ok()).flatten();
    inquiry_address.or_else(|| std::env::var("ORDER_NOTIFICATION_EMAIL").ok()).filter(|a| !a.trim().is_empty())
}

fn transport() -> Option<AsyncSmtpTransport<Tokio1Executor>> {
    let host = std::env::var("SMTP_ADDRESS").ok().filter(|h| !h.is_empty())?;
    let port: u16 = std::env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(587);
    let builder = if port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
    };
    let mut builder = builder.ok()?.port(port);
    if let (Ok(user), Ok(password)) = (std::env::var("SMTP_USERNAME"), std::env::var("SMTP_PASSWORD")) {
        builder = builder.credentials(Credentials::new(user, password));
    }
    Some(builder.build())
}

/// Sends the e-mail. A failure is logged, not raised: the order or
/// inquiry is saved already and must not fail because of the mail server.
pub async fn send(mail: Mail) {
    let Some(transport) = transport() else {
        tracing::info!(to = %mail.to, subject = %mail.subject, "e-mail (no SMTP configured):\n{}", mail.text);
        return;
    };
    let message = (|| -> Result<Message, Box<dyn std::error::Error>> {
        let builder = Message::builder()
            .from(from_address().parse::<Mailbox>()?)
            .to(mail.to.parse::<Mailbox>()?)
            .subject(&mail.subject);
        let text = SinglePart::builder().header(ContentType::TEXT_PLAIN).body(mail.text.clone());
        Ok(match &mail.html {
            Some(html) => builder.multipart(
                MultiPart::alternative().singlepart(text).singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(html.clone())),
            )?,
            None => builder.singlepart(text)?,
        })
    })()
    .map_err(|e| e.to_string());
    match message {
        Ok(message) => {
            if let Err(e) = transport.send(message).await {
                tracing::error!(to = %mail.to, subject = %mail.subject, "e-mail not sent: {e}");
            }
        }
        Err(e) => tracing::error!(to = %mail.to, subject = %mail.subject, "e-mail not built: {e}"),
    }
}
