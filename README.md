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

DirDB is at the v0.2 local-reliability stage. It provides a bounded Rust cache,
native file watching with a periodic integrity check, automatic repair of
invalid external JSON edits, and sync/async batch APIs. Recovery plans, local
IPC, and gRPC remain future milestones.

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

## Install

```bash
python -m pip install DirDB-Rust
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

When launched in an interactive Git Bash window, each script keeps the window open and reports success or failure until Enter is pressed. Set `NO_PAUSE=1` for automation.

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

# Measure dictionary-style document round trips and warm-cache reads.
python benchmark/python/mapping_roundtrip.py --items 1000 --read-rounds 10
```

See [benchmark notes](benchmark/README.md).

## CI and Releases

[CI](https://github.com/disnana/DirDB/actions/workflows/ci.yml) runs Rust formatting, Clippy, Rust tests, Ruff checks/formatting, wheel build/install tests with pytest, Python compilation checks, and platform wheel builds. A successful push to `main` automatically checks the package version against PyPI; if it is new, CI creates a GitHub Release and publishes the Linux, macOS, and Windows wheels plus the source distribution to [PyPI](https://pypi.org/project/DirDB-Rust/) through Trusted Publishing.

Documentation: [English guides](https://github.com/disnana/DirDB/tree/main/docs/en) | [Japanese guides](https://github.com/disnana/DirDB/tree/main/docs/ja) | [English design](https://github.com/disnana/DirDB/blob/main/docs/design.md) | [日本語設計書](https://github.com/disnana/DirDB/blob/main/docs/design.ja.md) | [日本語README](https://github.com/disnana/DirDB/blob/main/README.ja.md)

Implementation tracker: [TODO](https://github.com/disnana/DirDB/blob/main/TODO.md) | [TODO (Japanese)](https://github.com/disnana/DirDB/blob/main/TODO.ja.md)
