"""Write and read configuration documents with one native call per batch."""

from __future__ import annotations

import asyncio

from dirdb import DirDB


async def main() -> None:
    db = DirDB("./example-state/batch")
    versions = await db.aset_many(
        {
            "services/auth": {"enabled": True, "timeout": 10},
            "services/cache": {"enabled": True, "ttl": 60},
        }
    )
    values = await db.aget_many(["services/auth", "services/cache"])
    print("versions:", versions)
    print("values:", values)


asyncio.run(main())
