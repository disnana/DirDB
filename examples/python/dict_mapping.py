"""Use DirDB like a dictionary. Run with: python examples/python/dict_mapping.py"""

from __future__ import annotations

import tempfile

from dirdb import DirDB


def main() -> None:
    db = DirDB(tempfile.mkdtemp(prefix="dirdb-dict-"))

    db["ui/preferences"] = {
        "theme": "dark",
        "locale": "ja-JP",
        "shortcuts": ["ctrl+k", "ctrl+p"],
    }

    print("preferences:", db["ui/preferences"])
    print("missing with default:", db.get("ui/missing", {"theme": "system"}))
    print("all keys:", list(db))

    del db["ui/preferences"]
    print("exists after delete:", "ui/preferences" in db)


if __name__ == "__main__":
    main()
