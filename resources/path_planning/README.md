This folder holds runtime data for the path_planning domain.

Structure:
- geojson/: human-editable GeoJSON files that serve as source-of-truth map geometry.
- graphs/: serialized graph artifacts (binary) generated from the GeoJSON for fast runtime loading.
- manifest.toml: metadata mapping sources to generated graphs.

Runtime behavior (recommended):
- Loader prefers pre-serialized binaries in `graphs/` when available and valid.
- If a binary is missing or out-of-date, build the graph from the GeoJSON and write a binary cache.

Configuration:
- PATH_PLANNING_DATA_DIR environment variable overrides the default `resources/path_planning` location.

Generation:
Add a small tool or use the domain loader to convert GeoJSON -> Petgraph and write `.graph.bin` using `bincode` + `serde`.
