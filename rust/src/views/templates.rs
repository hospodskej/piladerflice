//! Loading the templates. In development they are re-read when they change,
//! so a template edit needs no recompile; in production they are loaded once
//! at startup.

use std::path::PathBuf;
use std::sync::Arc;

use minijinja::value::{Kwargs, Value};
use minijinja::{AutoEscape, Environment, Error, State, path_loader};
use minijinja_autoreload::AutoReloader;

use super::functions;
use super::text::escape;
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
    env.set_formatter(|out, state, value| {
        if matches!(state.auto_escape(), AutoEscape::Html) && !value.is_safe() && value.as_str().is_some() {
            out.write_str(&escape(value.as_str().unwrap_or_default()))
                .map_err(Error::from)
        } else {
            minijinja::escape_formatter(out, state, value)
        }
    });
    env.set_lstrip_blocks(true);

    env.add_function("t", move |state: &State, key: String, kwargs: Kwargs| {
        functions::translate(&translations, state, &key, kwargs)
    });
    env.add_function("path", functions::path);
    env.add_function("asset", move |logical: String| assets.url(&logical));
    env
}
