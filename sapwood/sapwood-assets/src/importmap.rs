//! The JavaScript import map: which URL each module name loads from, so
//! `import "controllers/hello"` works in the browser without a bundler.

use serde_json::json;

use crate::Assets;

/// Module names and the files they load. Build it once; render it with the
/// current URLs on each page ([`Importmap::json`]).
///
/// ```
/// let importmap = sapwood_assets::Importmap::new()
///     .pin("application", "application.js")
///     .pin("@hotwired/turbo", "turbo.min.js")
///     .pin_all_from("controllers");
/// ```
#[derive(Debug, Clone, Default)]
pub struct Importmap {
    pins: Vec<Pin>,
}

#[derive(Debug, Clone)]
enum Pin {
    One { module: String, file: String },
    Folder(String),
}

impl Importmap {
    pub fn new() -> Self {
        Self::default()
    }

    /// `module` loads the file `file` (an asset name).
    pub fn pin(mut self, module: impl Into<String>, file: impl Into<String>) -> Self {
        self.pins.push(Pin::One {
            module: module.into(),
            file: file.into(),
        });
        self
    }

    /// Every `.js` file in the folder `folder`, as a module of the same name:
    /// `controllers/hello.js` is `"controllers/hello"`, and an `index.js` is
    /// the folder itself (`"controllers"`).
    pub fn pin_all_from(mut self, folder: impl Into<String>) -> Self {
        self.pins
            .push(Pin::Folder(folder.into().trim_end_matches('/').to_string()));
        self
    }

    /// (module, URL) pairs, in the order pinned (folders sorted by name).
    pub fn urls(&self, assets: &Assets) -> Vec<(String, String)> {
        let names = assets.names();
        let mut urls = Vec::new();
        for pin in &self.pins {
            match pin {
                Pin::One { module, file } => urls.push((module.clone(), assets.url(file))),
                Pin::Folder(folder) => {
                    let prefix = format!("{folder}/");
                    for name in names.iter().filter(|name| name.starts_with(&prefix)) {
                        let Some(module) = name.strip_suffix(".js") else {
                            continue;
                        };
                        let module = module.strip_suffix("/index").unwrap_or(module);
                        urls.push((module.to_string(), assets.url(name)));
                    }
                }
            }
        }
        urls
    }

    /// The import map for `<script type="importmap">`.
    pub fn json(&self, assets: &Assets) -> String {
        let imports: serde_json::Map<String, serde_json::Value> = self
            .urls(assets)
            .into_iter()
            .map(|(module, url)| (module, json!(url)))
            .collect();
        serde_json::to_string_pretty(&json!({ "imports": imports })).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::tests::Dir;

    #[test]
    fn pins_files_and_folders() {
        let dir = Dir::new(&[
            ("application.js", "1"),
            ("controllers/index.js", "2"),
            ("controllers/hello.js", "3"),
            ("controllers/notes.txt", "x"),
            ("other/x.js", "4"),
        ]);
        let assets = Assets::builder().dir(&dir.0).build().unwrap();
        let importmap = Importmap::new()
            .pin("app", "application.js")
            .pin_all_from("controllers/");
        let modules: Vec<String> = importmap.urls(&assets).into_iter().map(|(module, _)| module).collect();
        assert_eq!(modules, ["app", "controllers/hello", "controllers"]);

        let json: serde_json::Value = serde_json::from_str(&importmap.json(&assets)).unwrap();
        assert_eq!(json["imports"]["app"], assets.url("application.js"));
    }
}
