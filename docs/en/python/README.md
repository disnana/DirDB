# Python Guide

## Automatic Reload

```python
from dirdb import DirDB

db = DirDB("./state", cache_max_items=10_000, auto_reload=True, debounce_ms=100)
```

The watcher, JSON validation, cache, file I/O, hashing, and SQLite work run in
Rust. Directly edited files become visible to the next `get()` after debounce.
Invalid JSON is atomically replaced with the last valid value. Use
`db.cache_stats()` for hit counters and `db.stat(key)` for reload health.

## Install

Build and install the local wheel from Git for Windows Bash:

```bash
./scripts/build-and-install.sh
```

To install into a virtual environment, set `PYTHON_BIN` to that environment's interpreter:

```bash
PYTHON_BIN=/c/path/to/.venv/Scripts/python.exe ./scripts/build-and-install.sh
```

## Async-First Use

Use the `a*` methods in applications that run an `asyncio` event loop. Each operation runs native storage work in a worker thread, and the Rust extension releases the GIL while it accesses the filesystem and SQLite.

```python
import asyncio
from dirdb import DirDB

async def main() -> None:
    db = DirDB("./state")
    version = await db.aset("app/config", {"theme": "dark"})
    config = await db.aget("app/config")
    print(version, config)

asyncio.run(main())
```

Available async methods are `aget`, `aset`, `adelete`, `aexists`, `alist`, and `arebuild_index`.

## Synchronous Use

Small scripts can use the equivalent synchronous methods: `get`, `set`, `delete`, `exists`, `list`, and `rebuild_index`.

```python
from dirdb import DirDB

db = DirDB("./state")
version = db.set("app/config", {"theme": "dark"})
config = db.get("app/config")
```

`DirDB` also implements Python's mutable mapping protocol. Use this form when configuration documents are naturally handled as a dictionary:

```python
db["app/config"] = {"theme": "dark", "features": ["sync", "async"]}
config = db["app/config"]
del db["app/config"]
```

`get(key, default)` follows standard dictionary behavior. Use `require(key)` when a missing key must raise `FileNotFoundError`.

Python dictionaries and lists cross the Rust boundary as structured JSON-compatible values; the binding does not serialize them to a temporary JSON string first.

## Version Checks

Pass `expected_version` to prevent a stale read from overwriting a newer document. A mismatch raises an error.

```python
current = await db.aset("app/config", {"theme": "dark"})
await db.aset("app/config", {"theme": "light"}, expected_version=current)
```

Run [the complete example](../../../examples/python/async_basic.py) with `python examples/python/async_basic.py`.

## Tests

After building and installing the wheel, run the Python regression suite with:

```bash
python -m pip install "pytest>=8"
python -m pytest tests/python -q
```

The suite includes a timeout-bounded concurrent async read/write test, which detects a regression that causes a storage deadlock.

Japanese guide: [../../ja/python/README.md](../../ja/python/README.md)
