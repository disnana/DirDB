"""Measure async DirDB write and read throughput.

Run after installing the wheel:
    python benchmark/python/async_throughput.py --items 1000 --concurrency 32
"""

from __future__ import annotations

import argparse
import asyncio
import tempfile
from pathlib import Path
from time import perf_counter

from dirdb import DirDB


async def measure(items: int, concurrency: int) -> None:
    state = Path(tempfile.mkdtemp(prefix="dirdb-benchmark-"))
    db = DirDB(str(state))
    semaphore = asyncio.Semaphore(concurrency)

    async def write(index: int) -> None:
        async with semaphore:
            await db.aset(f"entries/{index}", {"index": index, "enabled": True})

    started = perf_counter()
    await asyncio.gather(*(write(index) for index in range(items)))
    write_seconds = perf_counter() - started

    started = perf_counter()
    await asyncio.gather(*(db.aget(f"entries/{index}") for index in range(items)))
    read_seconds = perf_counter() - started

    print(f"items: {items}, concurrency: {concurrency}")
    print(f"writes: {items / write_seconds:,.0f} ops/s ({write_seconds:.3f}s)")
    print(f"reads:  {items / read_seconds:,.0f} ops/s ({read_seconds:.3f}s)")


def main() -> None:
    parser = argparse.ArgumentParser(description="DirDB async throughput benchmark")
    parser.add_argument("--items", type=int, default=1_000)
    parser.add_argument("--concurrency", type=int, default=32)
    args = parser.parse_args()
    asyncio.run(measure(args.items, args.concurrency))


if __name__ == "__main__":
    main()
