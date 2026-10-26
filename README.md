# noorden27

Scandinavia holiday planner: map with all destinations (score, preferences) and one route per couple.

## Data

The Google Sheet is the source of truth; `index.html` reads these tabs live in the browser
(gviz CSV, needs the sheet shared as "anyone with the link"):

- `Bestemmingen`: header on row 2 (`Land`, `Bestemming`, scores, `Totaalscore`, `Voorkeur <koppel>`, ...).
- `Routevoorstel`: one row per stop: `Koppel`, `Dag`, `Datum`, `Volgorde`, `Bestemming` (exact name from `Bestemmingen`, empty for a travel day), `Omschrijving`, ...
- `Coördinaten`: `Bestemming`, `Breedtegraad`, `Lengtegraad`.

When a tab cannot be fetched, the map falls back to the snapshot in `data/*.csv`, embedded at build time.

## Configuration

Everything site-specific lives outside the repo, in `~/.config/noorden27/config.json` (or under `$XDG_CONFIG_HOME`):

```json
{"sheet_id": "<google sheet id>", "deploy_target": "<ssh-user>@<ssh-host>", "remote_dir": "<directory on the host>"}
```

`Routevoorstel` is sensitive: `data/routevoorstel.csv` and `index.html` are gitignored.
When the sheet is not available and there is no `data/routevoorstel.csv`, the tab is left out of the build and the map shows no routes.

## Build and deploy

`cargo run --release -- build` fetches the sheet into `data/*.csv` (a tab that cannot be fetched keeps its existing snapshot) and writes `index.html`.
`cargo run --release -- deploy` uploads only `index.html` into `remote_dir` on `deploy_target`.
