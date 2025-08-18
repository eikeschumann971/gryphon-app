Plan: I'll explain why the loader belongs in an adapter, recommend exact filenames/paths, show a small domain port trait and a matching adapter sketch, and list the concrete steps I can take to move and wire the code.

Checklist

Explain DDD reasoning for adapter vs domain. Done below.
Recommend file paths & names for the adapter and the domain port. Done below.
Provide a minimal trait (port) that the domain depends on and a thin adapter implementation sketch. Done below.
Offer to perform the refactor (move loader.rs, add trait, wire DI). Ask whether to proceed. Done at end.
Why this code should be an adapter

Domain layer (path_planning) should contain pure business rules and data types, not I/O or filesystem concerns.
Code that touches the filesystem, environment variables, parsing formats (GeoJSON, bincode) is an infrastructure concern: it is an adapter that the domain calls through a port (an interface).
Keeping I/O in adapters preserves testability, keeps the domain language-focused, and follows Dependency Inversion: domain depends on an interface (port), concrete adapter lives in infrastructure.
Recommended organization (concrete)

Domain port (trait): src/domains/path_planning/ports.rs (or src/domains/path_planning/api.rs)
Export the trait in mod.rs (pub use).
Trait name: PathPlanningDataSource or MapDataProvider.
Adapter (concrete implementation): src/adapters/outbound/path_planning_data.rs (or src/adapters/outbound/path_planning_loader.rs)
Contains the filesystem logic currently in loader.rs.
Implements the domain trait.
Resources folder (already created): resources/path_planning/... (keep it as-is).
Minimal port trait (concept)

Methods should be domain-centric and not expose filesystem details:
load_geojson(name: &str) -> Result<String, LoaderError>
load_serialized_graph(name: &str) -> Result<Vec<u8>, LoaderError>
optionally: list_available_maps() -> Result<Vec<String>, LoaderError>
Sketch (Rust-like, minimal)

Trait (in src/domains/path_planning/ports.rs):
pub trait PathPlanningDataSource: Send + Sync { fn load_geojson(&self, name: &str) -> Result<String, crate::common::DomainError>; fn load_graph_bytes(&self, name: &str) -> Result<Vec<u8>, crate::common::DomainError>; }
Adapter (in src/adapters/outbound/path_planning_data.rs):
pub struct FilesystemDataSource { base_dir: PathBuf }
impl PathPlanningDataSource for FilesystemDataSource { /* implement using your resolve_data_dir/load_geojson logic */ }
Wiring / DI

Pass the trait object (Arc<dyn PathPlanningDataSource>) into the domain service (e.g., into a PathFinder service constructor in application/ or into the domain aggregate factory).
In tests, provide an in-memory/mock implementation.
Practical benefits

Domain tests stay pure and fast: you can inject a mock datasource.
Adapter code can use crates for parsing/serialization (serde_json, bincode) without polluting domain crate dependencies.
Deployment/packaging logic (Docker COPY, mount points) stays in infra docs and adapter config.
Concrete next steps I can do for you

Move loader.rs -> src/adapters/outbound/path_planning_data.rs and remove loader exposure from domain mod.
Add src/domains/path_planning/ports.rs with PathPlanningDataSource trait and export it from mod.rs.
Implement FilesystemDataSource in the adapter file, implementing the trait, reusing the existing resolve/load logic.
Update any internal callers (if the domain currently calls load_geojson directly) to accept the trait via constructor/parameter.
Add a tiny integration test showing injection of FilesystemDataSource and a unit test using a mock.
Which steps should I perform now? I can:

A) implement the full refactor (move file, add trait, adapter impl, wire exports), or
B) only move the file and rename it to the adapters path (leave the domain references), or
C) suggest precise patches for you to review.
Pick A/B/C and I’ll apply the changes.