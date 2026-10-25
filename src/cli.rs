//! Command line: `noorden27 build [--pull]` and `noorden27 deploy`.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, fs};

use crate::sheet::{self, Result};

const USAGE: &str = "usage: noorden27 <build [--pull] | deploy>";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("build") => build(args.iter().any(|a| a == "--pull")),
        Some("deploy") => deploy(),
        _ => Err(USAGE.into()),
    }
}

/// Writes `index.html`; `pull_first` refreshes `data/*.csv` from the sheet first.
fn build(pull_first: bool) -> Result<()> {
    let root = root();
    let data = root.join("data");
    if pull_first {
        sheet::pull(&data)?;
    }
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
    let target = env::var("DEPLOY_TARGET").map_err(|_| "set DEPLOY_TARGET to the ssh user@host")?;
    let remote_dir = env::var("REMOTE_DIR").unwrap_or_else(|_| "sites/kr8ium.com/noorden27".into());
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
