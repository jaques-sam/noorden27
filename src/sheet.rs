//! The Google Sheet: its configuration, the tabs we read and how to fetch them.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde_json::{json, Map, Value};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Tab {
    pub name: &'static str,
    pub file: &'static str,
    pub range: &'static str,
    /// Header that must be present in the first line of the fetched CSV.
    pub must: &'static str,
    /// Optional tabs hold sensitive data: they are never required, and are left
    /// out of the build when their snapshot is missing.
    pub optional: bool,
}

pub const TABS: [Tab; 3] = [
    Tab { name: "Bestemmingen", file: "bestemmingen.csv", range: "A2:Z", must: "Bestemming", optional: false },
    Tab { name: "Routevoorstel", file: "routevoorstel.csv", range: "A1:Z", must: "Koppel", optional: true },
    Tab { name: "Coördinaten", file: "coordinaten.csv", range: "A1:Z", must: "Breedtegraad", optional: false },
];

/// Reads a text file with newlines normalised to `\n`.
pub fn read(path: impl AsRef<Path>) -> Result<String> {
    Ok(fs::read_to_string(path)?.replace("\r\n", "\n"))
}

/// `~/.config/noorden27/config.json` (or under `$XDG_CONFIG_HOME`).
pub fn config_path() -> Result<PathBuf> {
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?).join(".config"),
    };
    Ok(base.join("noorden27").join("config.json"))
}

/// Id of the Google Sheet, from `{"sheet_id": "..."}` in the config file.
fn sheet_id() -> Result<String> {
    let path = config_path()?;
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {} ({e}); create it with {{\"sheet_id\": \"<id>\"}}", path.display()))?;
    let config: Value = serde_json::from_str(&text).map_err(|e| format!("invalid {}: {e}", path.display()))?;
    match config["sheet_id"].as_str() {
        Some(id) if !id.is_empty() => Ok(id.to_string()),
        _ => Err(format!("{} has no \"sheet_id\"", path.display()).into()),
    }
}

fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn url(sheet: &str, tab: &Tab) -> String {
    format!(
        "https://docs.google.com/spreadsheets/d/{sheet}/gviz/tq?tqx=out%3Acsv&sheet={}&headers=1&range={}",
        encode(tab.name),
        encode(tab.range)
    )
}

/// Refreshes `data/*.csv` from the sheet.
pub fn pull(data: &Path) -> Result<()> {
    let sheet = sheet_id()?;
    for tab in &TABS {
        let text = match ureq::get(&url(&sheet, tab)).call().and_then(|r| Ok(r.into_string()?)) {
            Ok(text) => text,
            Err(_) if tab.optional => {
                println!("skip {}: optional, could not be fetched", tab.name);
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        // gviz silently returns the first tab when a tab does not exist
        let header = text.split('\n').next().unwrap_or("");
        if !header.contains(&format!("\"{}\"", tab.must)) {
            println!("skip {}: tab missing or header '{}' not found", tab.name, tab.must);
            continue;
        }
        fs::write(data.join(tab.file), text)?;
        println!("pulled {}", tab.name);
    }
    Ok(())
}

/// The CSV snapshots and live URLs to embed, keyed by tab name.
/// Optional tabs without a snapshot are left out of both.
pub fn embedded(data: &Path) -> Result<(Value, Value)> {
    let sheet = sheet_id()?;
    let mut snap = Map::new();
    let mut live = Map::new();
    for tab in &TABS {
        let file = data.join(tab.file);
        if tab.optional && !file.exists() {
            println!("skip {}: no {}", tab.name, tab.file);
            continue;
        }
        snap.insert(tab.name.into(), read(file)?.into());
        live.insert(tab.name.into(), json!([url(&sheet, tab), tab.must]));
    }
    Ok((Value::Object(snap), Value::Object(live)))
}
