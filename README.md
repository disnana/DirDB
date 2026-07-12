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

The async API is the preferred path for application servers. It runs filesystem and SQLite work in a worker thread while the Rust extension releases the GIL.

```python
import asyncio
from dirdb import DirDB

async def main() -> None:
    db = DirDB("./state")
    version = await db.aset("services/auth/config", {"enabled": True})
    config = await db.aget("services/auth/config")

asyncio.run(main())
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

Build distributable artifacts with `uv build`. GitHub Actions builds and uploads wheels for Linux, macOS, and Windows on pull requests and version tags.

### Git for Windows Bash

```bash
# Build a wheel for the default Python.
./scripts/build-wheel.sh

# Build and install the newest wheel into the default Python.
./scripts/build-and-install.sh

# Target a virtual environment or a specific Python installation.
PYTHON_BIN=/c/path/to/.venv/Scripts/python.exe ./scripts/build-and-install.sh
```

## Examples

Run the async Python example after installing the wheel:

```bash
python examples/python/async_basic.py
```

Run the Rust core example directly:

```bash
cargo run --manifest-path examples/rust/basic/Cargo.toml
```

See [all examples](examples/README.md).

## Tests and Benchmarks

```bash
# Build/install DirDB first, then install the Python test dependency.
python -m pip install "pytest>=8"
python -m pytest tests/python -q

# Convenience alias; this delegates to pytest.
python -m tests tests/python -q

# Measure async read/write throughput.
python benchmark/python/async_throughput.py --items 1000 --concurrency 32
```

See [benchmark notes](benchmark/README.md). CI runs Rust tests, builds a wheel, installs it, and then runs the Python pytest suite.

Documentation: [English guides](docs/en/README.md) | [Japanese guides](docs/ja/README.md) | [English design](docs/design.md) | [日本語設計書](docs/design.ja.md) | [日本語README](README.ja.md)
