# Gryphon App

Multi-crate Rust simulation + server + Web UI + WASM viewer + Python test utilities.

## Overview

Gryphon is a sandbox for simulating agents moving along sampled paths, exposing:
- Core simulation primitives (core, pathfinder, simulator crates)
- HTTP + WebSocket server (server crate)
- A WASM bridge crate (viewer-wasm) for future visualization logic
- A React/TypeScript frontend (viewer-ui) for interacting with simulation state
- A Python test server (python-test-server) for experimentation / integration tests
- Shared utilities crate (utilities) for cross-cutting concerns (e.g. compression flags, future helpers)
- Protobuf definitions (proto/) compiled via prost build script in core

### Crate Roles

| Crate          | Purpose |
|----------------|---------|
| core           | Generated protobuf types + agent & path abstractions |
| pathfinder     | Simple random path generation (placeholder) |
| simulator      | Basic tick loop and agent management interacting with core types |
| server         | Axum-based HTTP + WebSocket endpoints (/health, /ws) |
| viewer-wasm    | WASM-side scaffolding (init, stub modules) |
| utilities      | Placeholder for shared helpers (e.g. compression flag) |

### Deferred (Future PRs)
- Realistic path geometry (curvature, clothoids, interpolation detail)
- WebSocket broadcast of evolving simulation state
- Control channel over WebSocket for spawning / removing agents
- gzip / compression implementation (feature gated)
- wasm-bindgen + bundling into viewer-ui
- Smooth animation interpolation in the viewer
- Integration tests for /ws behavior
- Architecture diagram & contribution guide

## Quick Start

### Prerequisites
- Rust (stable, with wasm32-unknown-unknown target if building WASM later)
- Node.js >= 18 & npm
- Python 3.11+
- Docker / Docker Compose (optional)

### Build & Test (Rust)
```
cargo test --all
cargo build -p gryphon-server
```

### Frontend
```
cd viewer-ui
npm install
npm run dev   # or: npm run build
```

### Python Test Server
```
cd python-test-server
pip install -e .[dev]
pytest -q
```

### Docker
```
docker build -t gryphon-app .
docker run -p 8080:8080 gryphon-app
curl http://localhost:8080/health   # => ok
```

### Development Scripts
See scripts/ for helpers:
- generate_proto.sh
- build_wasm.sh (stub)
- run_dev.sh (server + UI concurrently, future improvement)
- format_all.sh

## Configuration
Config files in config/: 
- server.toml
- agents.toml (initial agent seeding placeholder)

## Protobuf
Defined in proto/agents.proto and built via build.rs in core using prost.

## Architecture (High Level)
1. Simulator ticks and updates agent positions.
2. (Future) Server broadcasts snapshots over WebSocket.
3. WASM viewer + UI renders agent positions and metadata.
4. Python utilities provide integration test harness / external control (future).

## Testing
- Rust: Unit tests in core (path sampling, repository) and simulator.
- Python: Basic FastAPI endpoint test.
- (Future) WebSocket integration tests.

## Roadmap (Short-Term)
1. Implement WebSocket broadcast loop with periodic AgentBatch dummy frames.
2. Add control channel over WebSocket (spawn/remove agent).
3. wasm-bindgen integration + dynamic import in viewer-ui.
4. gzip compression flag actual implementation in utilities.
5. Add integration test suite (tokio-tungstenite).

## License
MIT (see LICENSE file).

---
Happy hacking! Open issues / PRs for improvements.