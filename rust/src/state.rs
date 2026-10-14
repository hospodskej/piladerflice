//! The state every request shares: settings, translations, assets,
//! templates, the database and the exchange rate.

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Result;

use crate::assets::{self, Assets};
use crate::config::Config;
use crate::db::{self, Db};
use crate::i18n::Translations;
use crate::money::ExchangeRates;
use crate::views::Templates;

pub struct Inner {
    pub config: Config,
    pub translations: Arc<Translations>,
    pub assets: Arc<Assets>,
    pub templates: Templates,
    pub db: Db,
    pub rates: ExchangeRates,
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
        Self::with(config, db, ExchangeRates::live())
    }

    /// For tests: a given database and a fixed exchange rate.
    pub fn with(config: Config, db: Db, rates: ExchangeRates) -> Result<Self> {
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
            rates,
        })))
    }
}
