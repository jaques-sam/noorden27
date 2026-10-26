//! Command line: `noorden27 build` and `noorden27 deploy`.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, fs};

use crate::config;
use crate::sheet::{self, Result};

const USAGE: &str = "usage: noorden27 <build | deploy>";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => build(),
        Some("deploy") => deploy(),
        _ => Err(USAGE.into()),
    }
}

/// Refreshes `data/*.csv` from the sheet (falling back to the existing snapshots
/// when it is not available) and writes `index.html`.
fn build() -> Result<()> {
    let root = root();
    let data = root.join("data");
    sheet::pull(&data)?;
    let (snap, live) = sheet::embedded(&data)?;

    let vendor = root.join("vendor");
    let replacements = [
        ("@LCSS@", sheet::read(vendor.join("leaflet.css"))?),
        ("@LJS@", sheet::read(vendor.join("leaflet.js"))?),
        ("@GEO@", sheet::read(vendor.join("nordic.json"))?),
        ("@SNAP@", snap.to_string()),
        ("@LIVE@", live.to_string()),
        ("@BUILT@", chrono::Local::now().format("%d/%m/%Y").to_string()),
    ];
    let mut html = sheet::read(root.join("template.html"))?;
    for (key, value) in replacements {
        html = html.replace(key, &value);
    }
    fs::write(root.join("index.html"), html)?;
    println!("index.html written");
    Ok(())
}

/// Uploads only the listed files, and only inside the remote `noorden27/` directory.
fn deploy() -> Result<()> {
    let target = config::get("deploy_target")?;
    let remote_dir = config::get("remote_dir")?;
    const FILES: [&str; 1] = ["index.html"];

    let mut tar = Command::new("tar")
        .arg("czf")
        .arg("-")
        .args(FILES)
        .current_dir(root())
        .stdout(Stdio::piped())
        .spawn()?;
    let ssh = Command::new("ssh")
        .arg(&target)
        .arg(format!("mkdir -p '{remote_dir}' && tar xzf - -C '{remote_dir}'"))
        .stdin(tar.stdout.take().ok_or("no tar stdout")?)
        .status()?;
    if !tar.wait()?.success() || !ssh.success() {
        return Err("deploy failed".into());
    }
    println!("Deployed to {remote_dir}");
    Ok(())
}
