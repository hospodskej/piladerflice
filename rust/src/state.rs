//! The state every request shares: settings, translations, assets,
//! templates, the database, and the outside services (exchange rate,
//! delivery distance, e-mail).

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;
use cookie::Key;

use crate::assets::{self, Assets};
use crate::config::Config;
use crate::db::{self, Db};
use crate::i18n::Translations;
use crate::mail::{MailSettings, Mailer};
use crate::money::ExchangeRates;
use crate::shop::delivery::RouteService;
use crate::views::Templates;

pub struct Inner {
    pub config: Config,
    pub translations: Arc<Translations>,
    pub assets: Arc<Assets>,
    pub templates: Templates,
    pub db: Db,
    pub rates: ExchangeRates,
    pub routes: RouteService,
    pub mailer: Mailer,
    pub mail_settings: MailSettings,
    /// Encrypts the session cookie.
    pub cookie_key: Key,
}

/// The outside services; tests swap in fakes.
pub struct Services {
    pub rates: ExchangeRates,
    pub routes: RouteService,
}

/// Shared by every request; cheap to clone.
#[derive(Clone)]
pub struct AppState(Arc<Inner>);

impl Deref for AppState {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl AppState {
    /// Connects to the database and loads translations, assets and templates.
    pub async fn new(config: Config) -> Result<Self> {
        let db = db::connect(&config.database_url).await?;
        db::check_schema(&db).await?;
        let services = Services {
            rates: ExchangeRates::live(),
            routes: RouteService::new(config.ors_api_key.clone()),
        };
        Self::with(config, db, services)
    }

    /// With a given database and services (tests: in-memory, fakes).
    pub fn with(config: Config, db: Db, services: Services) -> Result<Self> {
        crate::views::error_pages::load(&config.public_dir());
        let mailer = Mailer::from_settings(&config.mail)?;
        let mail_settings = config.mail.clone();
        let cookie_key = Key::derive_from(config.secret_key_base.as_bytes());
        let translations = Arc::new(Translations::load(&config.app_root.join("config/locales"))?);
        let assets = Arc::new(Assets::new(
            assets::default_sources(&config.app_root, &config.crate_root),
            config.development,
        )?);
        let templates = Templates::new(
            config.crate_root.join("templates"),
            translations.clone(),
            assets.clone(),
            config.development,
        );
        Ok(Self(Arc::new(Inner {
            config,
            translations,
            assets,
            templates,
            db,
            rates: services.rates,
            routes: services.routes,
            mailer,
            mail_settings,
            cookie_key,
        })))
    }
}
