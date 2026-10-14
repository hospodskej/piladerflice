//! Where e-mails come from and go to. Same environment variables as the
//! Rails app:
//!
//! - `SMTP_ADDRESS` (default smtp.gmail.com), `SMTP_PORT` (587; 465 means
//!   TLS from the start), `SMTP_USERNAME`, `SMTP_PASSWORD`. Without
//!   `SMTP_USERNAME` nothing is sent (the e-mails are only logged).
//! - `MAIL_FROM_ADDRESS`: the sender (objednavky@piladerflice.cz)
//! - `ORDER_NOTIFICATION_EMAIL`, `INQUIRY_NOTIFICATION_EMAIL`: who is told
//!   about new orders and quote requests

#[derive(Clone, Debug)]
pub struct SmtpSettings {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct MailSettings {
    pub smtp: Option<SmtpSettings>,
    pub from: String,
    pub order_notification_to: String,
    pub inquiry_notification_to: String,
}

impl MailSettings {
    pub fn from_env() -> Self {
        let var = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        let smtp = var("SMTP_USERNAME").map(|username| SmtpSettings {
            address: var("SMTP_ADDRESS").unwrap_or_else(|| "smtp.gmail.com".into()),
            port: var("SMTP_PORT").and_then(|port| port.parse().ok()).unwrap_or(587),
            username,
            password: var("SMTP_PASSWORD").unwrap_or_default(),
        });
        let order_to = var("ORDER_NOTIFICATION_EMAIL").unwrap_or_else(|| "pavelpatockaa@gmail.com".into());
        Self {
            smtp,
            from: var("MAIL_FROM_ADDRESS").unwrap_or_else(|| "objednavky@piladerflice.cz".into()),
            inquiry_notification_to: var("INQUIRY_NOTIFICATION_EMAIL").unwrap_or_else(|| order_to.clone()),
            order_notification_to: order_to,
        }
    }

    /// No SMTP: e-mails go to the outbox (tests, development).
    pub fn outbox_only() -> Self {
        Self {
            smtp: None,
            from: "objednavky@piladerflice.cz".into(),
            order_notification_to: "obchod@piladerflice.test".into(),
            inquiry_notification_to: "poptavky@piladerflice.test".into(),
        }
    }
}
