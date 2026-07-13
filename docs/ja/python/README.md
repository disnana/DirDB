# Pythonガイド

## 自動再読み込み

```python
from dirdb import DirDB

db = DirDB(
    "./state",
    cache_max_items=10_000,
    auto_reload=True,
    debounce_ms=100,
    verify_interval_seconds=60,
)
```

監視、JSON検証、キャッシュ、ファイルI/O、ハッシュ、SQLite処理はRust側で動きます。直接編集したファイルはdebounce後の次回`get()`から見えます。壊れたJSONは最後の正常値へ原子的に書き戻されます。ヒット数は`db.cache_stats()`、再読込状態は`db.stat(key)`で確認できます。

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

利用できる非同期メソッドには、`aget`、`aset`、`aget_many`、`aset_many`、`adelete`、`aexists`、`alist`、`astat`、`arebuild_index`があります。

## 同期で使う

小さなスクリプトでは、対応する同期メソッド、`get`、`set`、`delete`、`exists`、`list`、`rebuild_index`を使えます。

```python
from dirdb import DirDB

db = DirDB("./state")
version = db.set("app/config", {"theme": "dark"})
config = db.get("app/config")
```

`DirDB`はPythonの可変マッピングプロトコルも実装します。設定ドキュメントを辞書として自然に扱いたい場合は、次の形式を使えます。

```python
db["app/config"] = {"theme": "dark", "features": ["sync", "async"]}
config = db["app/config"]
del db["app/config"]
```

`get(key, default)`は標準の辞書と同じ挙動です。キーがない場合に`FileNotFoundError`を必ず受け取りたいときは`require(key)`を使います。

Pythonの辞書とリストは構造化されたJSON互換値としてRust境界を通過します。一時的なJSON文字列へのシリアライズは行いません。

## バージョン検査

古い読み込み結果で新しいドキュメントを上書きしないために、`expected_version`を渡します。現在のバージョンと違う場合はエラーになります。

```python
current = await db.aset("app/config", {"theme": "dark"})
await db.aset("app/config", {"theme": "light"}, expected_version=current)
```

[完全なサンプル](../../../examples/python/async_basic.py)は、`python examples/python/async_basic.py`で実行できます。

## テスト

wheelをビルド・インストールした後、Pythonの回帰テストは次で実行します。

```bash
python -m pip install "pytest>=8"
python -m pytest tests/python -q
```

テストには、タイムアウトを設定した並行async読み書きテストが含まれます。ストレージ処理のデッドロックが再発した場合は失敗します。

English guide: [../../en/python/README.md](../../en/python/README.md)
