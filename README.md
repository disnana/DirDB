# DirDB

**Your directory is the database.**

DirDB (pronounced **Deer DB** / **Directory DB**) is a filesystem-first local configuration store. `Dir` represents a directory; its reading also evokes *deer* and *dear*: a database close to your files.

The files in `data/` are always the source of truth. SQLite holds rebuildable metadata and revision history, while callers get a small Python API backed by Rust.

```python
from dirdb import DirDB

db = DirDB("./state")
version = db.set("services/auth/config", {"enabled": True})
config = db.get("services/auth/config")
```

## Status

This repository is at the v0.1 foundation stage. It currently provides JSON documents, atomic writes, optimistic version checks, SQLite catalog/history, index rebuilding, and a PyO3 binding. File watching, cache policy, recovery plans, local IPC, and gRPC are deliberately future milestones.

## Layout

```text
state/
├── data/                       # authoritative JSON documents
│   └── services/auth/config.json
└── metadata.db                 # rebuildable catalog and revisions
```

## Development

```powershell
cargo test -p dirdb-core
uv run maturin develop
```

Documentation: [English design](docs/design.md) | [日本語設計書](docs/design.ja.md) | [日本語README](README.ja.md)
