# Pythonガイド

## インストール

Git for Windows Bashからローカルwheelを作成・インストールします。

```bash
./scripts/build-and-install.sh
```

仮想環境へ入れる場合は、`PYTHON_BIN`にそのインタープリタを指定します。

```bash
PYTHON_BIN=/c/path/to/.venv/Scripts/python.exe ./scripts/build-and-install.sh
```

## Async-Firstで使う

`asyncio`イベントループ上で動くアプリケーションでは、`a*`メソッドを使います。各操作はワーカースレッドでネイティブのストレージ処理を実行し、Rust拡張はファイルシステムとSQLiteへのアクセス中にGILを解放します。

```python
import asyncio
from dirdb import DirDB

async def main() -> None:
    db = DirDB("./state")
    version = await db.aset("app/config", {"theme": "dark"})
    config = await db.aget("app/config")
    print(version, config)

asyncio.run(main())
```

利用できる非同期メソッドは、`aget`、`aset`、`adelete`、`aexists`、`alist`、`arebuild_index`です。

## 同期で使う

小さなスクリプトでは、対応する同期メソッド、`get`、`set`、`delete`、`exists`、`list`、`rebuild_index`を使えます。

```python
from dirdb import DirDB

db = DirDB("./state")
version = db.set("app/config", {"theme": "dark"})
config = db.get("app/config")
```

## バージョン検査

古い読み込み結果で新しいドキュメントを上書きしないために、`expected_version`を渡します。現在のバージョンと違う場合はエラーになります。

```python
current = await db.aset("app/config", {"theme": "dark"})
await db.aset("app/config", {"theme": "light"}, expected_version=current)
```

[完全なサンプル](../../../examples/python/async_basic.py)は、`python examples/python/async_basic.py`で実行できます。

English guide: [../../en/python/README.md](../../en/python/README.md)
