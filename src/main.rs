//! Builds `index.html`: the map reads the Google Sheet live in the browser and
//! falls back to the CSV snapshots in `data/` (embedded at build time).

mod cli;
mod sheet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run()
}
