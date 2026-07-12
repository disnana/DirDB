"""Measure dictionary-style DirDB round trips.

Run after installing the wheel:
    python benchmark/python/mapping_roundtrip.py --items 1000
"""

from __future__ import annotations

import argparse
import tempfile
from time import perf_counter

from dirdb import DirDB


def main() -> None:
    parser = argparse.ArgumentParser(description="DirDB dictionary API benchmark")
    parser.add_argument("--items", type=int, default=1_000)
    args = parser.parse_args()

    db = DirDB(tempfile.mkdtemp(prefix="dirdb-mapping-benchmark-"))
    document = {
        "enabled": True,
        "theme": "dark",
        "limits": {"requests_per_minute": 120, "burst": 20},
        "features": ["sync", "async", "watch"],
    }

    started = perf_counter()
    for index in range(args.items):
        db[f"entries/{index}"] = document
    write_seconds = perf_counter() - started

    started = perf_counter()
    for index in range(args.items):
        assert db[f"entries/{index}"] == document
    read_seconds = perf_counter() - started

    print(f"items: {args.items}")
    print(f"dict writes: {args.items / write_seconds:,.0f} ops/s ({write_seconds:.3f}s)")
    print(f"dict reads:  {args.items / read_seconds:,.0f} ops/s ({read_seconds:.3f}s)")


if __name__ == "__main__":
    main()
