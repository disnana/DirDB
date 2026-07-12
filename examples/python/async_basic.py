"""Run with: python examples/python/async_basic.py"""

from __future__ import annotations

import asyncio
from pathlib import Path

from dirdb import DirDB


async def main() -> None:
    db = DirDB(str(Path("example-state/python")))

    version = await db.aset(
        "services/auth/config",
        {"enabled": True, "providers": ["password", "passkey"]},
    )
    config = await db.aget("services/auth/config")

    print(f"saved version: {version}")
    print(f"config: {config}")
    print(f"keys: {await db.alist('services')}")


if __name__ == "__main__":
    asyncio.run(main())
