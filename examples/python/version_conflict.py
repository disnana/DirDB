"""Protect a document with optimistic version checks."""

from __future__ import annotations

import tempfile

from dirdb import DirDB


def main() -> None:
    db = DirDB(tempfile.mkdtemp(prefix="dirdb-version-"))
    first = db.set("services/payment/config", {"enabled": False})
    second = db.set(
        "services/payment/config",
        {"enabled": True},
        expected_version=first,
    )
    print("updated to version:", second)

    try:
        db.set("services/payment/config", {"enabled": False}, expected_version=first)
    except RuntimeError as error:
        print("stale update rejected:", error)


if __name__ == "__main__":
    main()
