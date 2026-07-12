# DirDB

**あなたのディレクトリがデータベースになる。**

DirDB（ディアDB／ディレクトリDB）は、ファイルシステムを優先するローカル設定ストアです。`Dir` は Directory を表し、読みは *deer*（鹿）と *dear*（親愛なる、大切な）にも重なります。

`data/` 内のファイルが常に正本です。SQLiteは再構築可能なカタログとリビジョン履歴だけを持ち、Rustで実装したコアを小さなPython APIから使えます。

```python
from dirdb import DirDB

db = DirDB("./state")
version = db.set("services/auth/config", {"enabled": True})
config = db.get("services/auth/config")
```

アプリケーションサーバーでは非同期APIを推奨します。ファイル／SQLite処理はワーカースレッドで行い、Rust拡張は処理中にGILを解放します。

```python
import asyncio
from dirdb import DirDB

async def main() -> None:
    db = DirDB("./state")
    version = await db.aset("services/auth/config", {"enabled": True})
    config = await db.aget("services/auth/config")

asyncio.run(main())
```

## 状態

このリポジトリはv0.1の基盤段階です。JSONドキュメント、原子的な書き込み、楽観的バージョン検査、SQLiteのカタログ／履歴、インデックス再構築、PyO3バインディングを提供します。ファイル監視、キャッシュ方針、復旧計画、ローカルIPC、gRPCは今後のマイルストーンです。

## 保存構造

```text
state/
├── data/                       # 正本のJSONドキュメント
│   └── services/auth/config.json
└── metadata.db                 # 再構築可能なカタログとリビジョン
```

## 開発

```powershell
cargo test -p dirdb-core
uv run maturin develop
```

## インストール

```bash
python -m pip install DirDB-Rust
```

配布物は `uv build` で生成できます。GitHub ActionsはプルリクエストとバージョンタグでLinux、macOS、Windows向けwheelを自動生成・保存します。

### Git for Windows Bash

```bash
# 既定のPython向けwheelを作成する
./scripts/build-wheel.sh

# wheelを作成し、既定のPython環境へインストールする
./scripts/build-and-install.sh

# 仮想環境または任意のPythonを対象にする
PYTHON_BIN=/c/path/to/.venv/Scripts/python.exe ./scripts/build-and-install.sh
```

## サンプル

wheelのインストール後に、async Pythonサンプルを実行できます。

```bash
python examples/python/async_basic.py
```

Rustコアのサンプルは直接実行できます。

```bash
cargo run --manifest-path examples/rust/basic/Cargo.toml
```

[サンプル一覧](examples/README.ja.md)も参照してください。

## テストとベンチマーク

```bash
# 先にDirDBをビルド・インストールし、Pythonテスト依存を入れる
python -m pip install "pytest>=8"
python -m pytest tests/python -q

# pytestへ委譲する簡易コマンド
python -m tests tests/python -q

# async読み書きのスループットを測定する
python benchmark/python/async_throughput.py --items 1000 --concurrency 32

# 辞書形式のドキュメント往復を測定する
python benchmark/python/mapping_roundtrip.py --items 1000
```

[ベンチマークの注意事項](benchmark/README.ja.md)も参照してください。

## CIとリリース

[CI](.github/workflows/ci.yml)は、Rustフォーマット、Clippy、Rustテスト、wheelのビルド／インストールを含むpytest、Pythonコンパイル検査、OS別wheelビルドを実行します。`v*`タグをpushすると、同じ品質ゲートを通過した後にGitHub Releaseを作成し、Trusted PublishingでLinux、macOS、Windows向けwheelとソース配布物を[PyPI](https://pypi.org/project/DirDB-Rust/)へ公開します。

文書: [日本語ガイド](docs/ja/README.md) | [English guides](docs/en/README.md) | [日本語設計書](docs/design.ja.md) | [English design](docs/design.md) | [English README](README.md)

実装タスク: [TODO](TODO.ja.md) | [TODO (English)](TODO.md)
