# noorden27

Scandinavia holiday planner: map with all destinations (score, preferences) and one route per couple.
Published at <https://www.kr8ium.com/noorden27/>.

## Data

The Google Sheet is the source of truth; `index.html` reads these tabs live in the browser
(gviz CSV, needs the sheet shared as "anyone with the link"):

- `Bestemmingen`: header on row 2 (`Land`, `Bestemming`, scores, `Totaalscore`, `Voorkeur <koppel>`, ...).
- `Routevoorstel`: one row per stop: `Koppel`, `Dag`, `Datum`, `Volgorde`, `Bestemming` (exact name from `Bestemmingen`, empty for a travel day), `Omschrijving`, ...
- `Coördinaten`: `Bestemming`, `Breedtegraad`, `Lengtegraad`.

When a tab cannot be fetched, the map falls back to the snapshot in `data/*.csv`, embedded at build time.

## Configuration

The sheet id lives outside the repo, in `~/.config/noorden27/config.json` (or under `$XDG_CONFIG_HOME`):

```json
{"sheet_id": "<google sheet id>"}
```

`Routevoorstel` is optional and sensitive: `data/routevoorstel.csv` and `index.html` are gitignored.
Without `data/routevoorstel.csv` the tab is left out of the build and the map shows no routes.

## Build and deploy

`cargo run --release -- build --pull` refreshes `data/*.csv` from the sheet and writes `index.html` (omit `--pull` to build from the existing snapshots).
`DEPLOY_TARGET=<ssh-user>@<ssh-host> cargo run --release -- deploy` uploads only `index.html` into `sites/kr8ium.com/noorden27` (same host as `rust-slides`).
