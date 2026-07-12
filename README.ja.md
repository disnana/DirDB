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

文書: [日本語設計書](docs/design.ja.md) | [English design](docs/design.md) | [English README](README.md)
