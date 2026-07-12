"""Run concurrent async writes. Run with: python examples/python/async_concurrency.py"""

from __future__ import annotations

import asyncio
import tempfile

from dirdb import DirDB


async def main() -> None:
    db = DirDB(tempfile.mkdtemp(prefix="dirdb-async-"))
    concurrency = 8
    semaphore = asyncio.Semaphore(concurrency)

    async def write(index: int) -> int:
        async with semaphore:
            return await db.aset(
                f"users/{index}/settings",
                {"notifications": index % 2 == 0, "theme": "dark"},
            )

    versions = await asyncio.gather(*(write(index) for index in range(24)))
    settings = await asyncio.gather(
        *(db.aget(f"users/{index}/settings") for index in range(24))
    )

    print("written documents:", len(versions))
    print("all first versions:", all(version == 1 for version in versions))
    print("first document:", settings[0])


if __name__ == "__main__":
    asyncio.run(main())
