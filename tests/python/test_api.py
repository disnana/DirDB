from __future__ import annotations

import asyncio

import pytest

from dirdb import DirDB


def test_sync_document_lifecycle(tmp_path) -> None:
    db = DirDB(str(tmp_path / "state"))

    version = db.set("app/config", {"theme": "dark"})

    assert version == 1
    assert db.get("app/config") == {"theme": "dark"}
    assert db.exists("app/config") is True
    assert db.list("app") == ["app/config"]

    db.delete("app/config", expected_version=version)
    assert db.exists("app/config") is False


def test_async_document_lifecycle(tmp_path) -> None:
    async def scenario() -> None:
        db = DirDB(str(tmp_path / "state"))
        version = await db.aset("services/auth", {"enabled": True})

        assert version == 1
        assert await db.aget("services/auth") == {"enabled": True}
        assert await db.alist("services") == ["services/auth"]

        await db.adelete("services/auth", expected_version=version)
        assert await db.aexists("services/auth") is False

    asyncio.run(scenario())


def test_expected_version_rejects_stale_write(tmp_path) -> None:
    db = DirDB(str(tmp_path / "state"))
    version = db.set("app/config", {"theme": "dark"})
    db.set("app/config", {"theme": "light"}, expected_version=version)

    with pytest.raises(RuntimeError, match="version conflict"):
        db.set("app/config", {"theme": "system"}, expected_version=version)
