//! Loading the templates. In development they are re-read when they change,
//! so a template edit needs no recompile; in production they are loaded once
//! at startup.

use std::path::PathBuf;
use std::sync::Arc;

use minijinja::value::Value;
use minijinja::{AutoEscape, Environment, Error, path_loader};
use minijinja_autoreload::AutoReloader;

use super::text::escape;
use super::{form_helpers, functions};
use crate::assets::Assets;
use crate::i18n::Translations;

pub enum Templates {
    Static(Environment<'static>),
    Reloading(AutoReloader),
}

impl Templates {
    pub fn new(dir: PathBuf, translations: Arc<Translations>, assets: Arc<Assets>, reload: bool) -> Self {
        if reload {
            Templates::Reloading(AutoReloader::new(move |notifier| {
                notifier.watch_path(&dir, true);
                Ok(environment(dir.clone(), translations.clone(), assets.clone()))
            }))
        } else {
            Templates::Static(environment(dir, translations, assets))
        }
    }

    pub fn render(&self, name: &str, ctx: Value) -> Result<String, Error> {
        match self {
            Templates::Static(env) => env.get_template(name)?.render(ctx),
            Templates::Reloading(reloader) => reloader.acquire_env()?.get_template(name)?.render(ctx),
        }
    }
}

fn environment(dir: PathBuf, translations: Arc<Translations>, assets: Arc<Assets>) -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(path_loader(dir));
    env.set_trim_blocks(true);
    // Escape like Rails (& < > " '), not "/" too as MiniJinja does by default.
    // Missing values (None) print as nothing, like nil in ERB, not "none".
    env.set_formatter(|out, state, value| {
        if value.is_none() || value.is_undefined() {
            return Ok(());
        }
        if matches!(state.auto_escape(), AutoEscape::Html) && !value.is_safe() && value.as_str().is_some() {
            out.write_str(&escape(value.as_str().unwrap_or_default()))
                .map_err(Error::from)
        } else {
            minijinja::escape_formatter(out, state, value)
        }
    });
    env.set_lstrip_blocks(true);

    env.add_function("t", sapwood_i18n::jinja::function(translations, "locale"));
    env.add_function("path", functions::path);
    env.add_function("asset", move |logical: String| assets.url(&logical));
    form_helpers::register(&mut env);
    env
}
