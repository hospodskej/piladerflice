//! The JavaScript import map: which URL each module name loads from. Same
//! pins as `config/importmap.rb` in Rails.

use serde_json::json;

use super::Assets;

impl Assets {
    /// The import map (same pins as `config/importmap.rb`): module name -> URL.
    pub fn importmap(&self) -> Vec<(String, String)> {
        let mut pins = vec![
            ("application".to_string(), self.url("application.js")),
            ("@hotwired/turbo-rails".to_string(), self.url("turbo.min.js")),
            ("@hotwired/stimulus".to_string(), self.url("stimulus.min.js")),
            (
                "@hotwired/stimulus-loading".to_string(),
                self.url("stimulus-loading.js"),
            ),
        ];
        let index = self.index.read().unwrap();
        for folder in ["controllers", "custom"] {
            let prefix = format!("{folder}/");
            for (logical, asset) in index.by_logical.range(prefix.clone()..) {
                if !logical.starts_with(&prefix) {
                    break;
                }
                let Some(module) = logical.strip_suffix(".js") else {
                    continue;
                };
                // controllers/index.js is imported as "controllers".
                let name = module.strip_suffix("/index").unwrap_or(module).to_string();
                pins.push((name, format!("/assets/{}", asset.digested)));
            }
        }
        pins
    }

    pub fn importmap_json(&self) -> String {
        let imports: serde_json::Map<String, serde_json::Value> = self
            .importmap()
            .into_iter()
            .map(|(name, url)| (name, json!(url)))
            .collect();
        serde_json::to_string_pretty(&json!({ "imports": imports })).unwrap_or_default()
    }
}
