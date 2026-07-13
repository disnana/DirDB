"""Observe Rust-native reload and automatic repair of invalid JSON."""

from __future__ import annotations

import json
import tempfile
import time
from pathlib import Path

from dirdb import DirDB


root = Path(tempfile.mkdtemp(prefix="dirdb-auto-reload-"))
db = DirDB(str(root), debounce_ms=50)
db["services/auth"] = {"enabled": True, "timeout": 10}

file = root / "data" / "services" / "auth.json"
file.write_text(json.dumps({"enabled": True, "timeout": 30}), encoding="utf-8")
time.sleep(0.2)
print("reloaded:", db["services/auth"])

file.write_text("{", encoding="utf-8")
time.sleep(0.2)
print("last valid:", db["services/auth"])
print("repaired file:", json.loads(file.read_text(encoding="utf-8")))
print("cache:", db.cache_stats())
