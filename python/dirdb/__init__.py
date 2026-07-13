"""Async-first Python API for the filesystem-first DirDB store."""

from __future__ import annotations

import asyncio
from collections.abc import Iterator, MutableMapping
from typing import Any

from ._native import NativeDirDB

__all__ = ["DirDB"]


class DirDB(MutableMapping[str, Any]):
    """A local DirDB store with synchronous and asyncio-friendly operations.

    The ``a*`` methods use a worker thread. Native file and SQLite work releases
    the GIL, so unrelated coroutine work can continue while an operation runs.
    """

    def __init__(self, root: str) -> None:
        self._native = NativeDirDB(root)

    def get(self, key: str, default: Any = None) -> Any:
        try:
            return self._native.get(key)
        except FileNotFoundError:
            return default

    def require(self, key: str) -> Any:
        """Return a value or raise FileNotFoundError when the key is absent."""
        return self._native.get(key)

    def set(self, key: str, value: Any, expected_version: int | None = None) -> int:
        return self._native.set(key, value, expected_version)

    def delete(self, key: str, expected_version: int | None = None) -> None:
        self._native.delete(key, expected_version)

    def exists(self, key: str) -> bool:
        return self._native.exists(key)

    def list(self, prefix: str = "") -> list[str]:
        return self._native.list(prefix)

    def rebuild_index(self) -> int:
        return self._native.rebuild_index()

    def __getitem__(self, key: str) -> Any:
        try:
            return self.require(key)
        except FileNotFoundError as error:
            raise KeyError(key) from error

    def __setitem__(self, key: str, value: Any) -> None:
        self.set(key, value)

    def __delitem__(self, key: str) -> None:
        try:
            self.delete(key)
        except FileNotFoundError as error:
            raise KeyError(key) from error

    def __iter__(self) -> Iterator[str]:
        return iter(self.list())

    def __len__(self) -> int:
        return len(self.list())

    async def aget(self, key: str) -> Any:
        return await asyncio.to_thread(self.get, key)

    async def aset(
        self, key: str, value: Any, expected_version: int | None = None
    ) -> int:
        return await asyncio.to_thread(self.set, key, value, expected_version)

    async def adelete(self, key: str, expected_version: int | None = None) -> None:
        await asyncio.to_thread(self.delete, key, expected_version)

    async def aexists(self, key: str) -> bool:
        return await asyncio.to_thread(self.exists, key)

    async def alist(self, prefix: str = "") -> list[str]:
        return await asyncio.to_thread(self.list, prefix)

    async def arebuild_index(self) -> int:
        return await asyncio.to_thread(self.rebuild_index)
