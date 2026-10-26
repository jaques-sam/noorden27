//! `~/.config/noorden27/config.json` (or under `$XDG_CONFIG_HOME`): the sheet and the site to deploy to.
//!
//! ```json
//! {"sheet_id": "<google sheet id>", "deploy_target": "<ssh-user>@<ssh-host>", "remote_dir": "sites/<domain>/noorden27"}
//! ```

use std::path::PathBuf;
use std::{env, fs};

use serde_json::Value;

use crate::sheet::Result;

fn path() -> Result<PathBuf> {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?).join(".config"),
    };
    Ok(base.join("noorden27").join("config.json"))
}

/// A non-empty string setting from the config file.
pub fn get(key: &str) -> Result<String> {
    let path = path()?;
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {} ({e}); it needs a \"{key}\" setting", path.display()))?;
    let config: Value = serde_json::from_str(&text).map_err(|e| format!("invalid {}: {e}", path.display()))?;
    match config[key].as_str() {
        Some(value) if !value.is_empty() => Ok(value.to_string()),
        _ => Err(format!("{} has no \"{key}\"", path.display()).into()),
    }
}
