from __future__ import annotations

import asyncio
import json
import time

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


def test_python_mapping_operations(tmp_path) -> None:
    db = DirDB(str(tmp_path / "state"))
    db["app/config"] = {"theme": "dark", "features": ["sync", "async"]}

    assert db["app/config"] == {"theme": "dark", "features": ["sync", "async"]}
    assert db.get("missing") is None
    assert db.get("missing", {"fallback": True}) == {"fallback": True}
    assert "app/config" in db
    assert list(db) == ["app/config"]
    assert len(db) == 1

    del db["app/config"]
    assert "app/config" not in db
    with pytest.raises(KeyError):
        _ = db["app/config"]


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


def test_concurrent_async_operations_complete_without_deadlock(tmp_path) -> None:
    async def scenario() -> None:
        db = DirDB(str(tmp_path / "state"))
        item_count = 32

        versions = await asyncio.wait_for(
            asyncio.gather(
                *(
                    db.aset(f"bulk/{index}", {"index": index})
                    for index in range(item_count)
                )
            ),
            timeout=5,
        )
        assert versions == [1] * item_count

        values = await asyncio.wait_for(
            asyncio.gather(*(db.aget(f"bulk/{index}") for index in range(item_count))),
            timeout=5,
        )
        assert values == [{"index": index} for index in range(item_count)]

    asyncio.run(scenario())


def test_cache_reports_hits_and_respects_limit(tmp_path) -> None:
    db = DirDB(str(tmp_path / "state"), cache_max_items=1, auto_reload=False)
    db.set("one", {"value": 1})
    db.set("two", {"value": 2})

    assert db.get("two") == {"value": 2}
    stats = db.cache_stats()
    assert stats["hits"] >= 1
    assert stats["entries"] == 1


def test_external_edits_reload_and_invalid_json_is_repaired(tmp_path) -> None:
    root = tmp_path / "state"
    db = DirDB(str(root), debounce_ms=20)
    db.set("app/config", {"value": 1})
    path = root / "data" / "app" / "config.json"

    path.write_text(json.dumps({"value": 2}), encoding="utf-8")
    wait_until(lambda: db.get("app/config") == {"value": 2})
    version = db.stat("app/config")["current_version"]

    path.write_text("{", encoding="utf-8")
    wait_until(lambda: is_valid_json(path))
    assert db.get("app/config") == {"value": 2}
    assert db.stat("app/config") == {
        "file_valid": True,
        "current_version": version,
        "last_reload_error": None,
    }


def wait_until(predicate, timeout: float = 3.0) -> None:
    deadline = time.monotonic() + timeout
    while not predicate():
        if time.monotonic() >= deadline:
            raise AssertionError("condition timed out")
        time.sleep(0.02)


def is_valid_json(path) -> bool:
    try:
        json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    return True
