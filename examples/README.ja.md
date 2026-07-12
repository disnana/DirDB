# DirDB サンプル

| 言語 | サンプル | 内容 | 実行 |
| --- | --- | --- | --- |
| Python | [async_basic.py](python/async_basic.py) | 非同期ドキュメント読み書き | `python examples/python/async_basic.py` |
| Python | [dict_mapping.py](python/dict_mapping.py) | `dict`形式のアクセスとデフォルト値 | `python examples/python/dict_mapping.py` |
| Python | [async_concurrency.py](python/async_concurrency.py) | 上限付きの並行async書き込み | `python examples/python/async_concurrency.py` |
| Python | [version_conflict.py](python/version_conflict.py) | 楽観的バージョン検査 | `python examples/python/version_conflict.py` |
| Rust | [basic](rust/basic/src/main.rs) | コアへの直接読み書き | `cargo run --manifest-path examples/rust/basic/Cargo.toml` |
| Rust | [version_conflict](rust/basic/src/bin/version_conflict.rs) | 楽観的バージョン検査 | `cargo run --manifest-path examples/rust/basic/Cargo.toml --bin version_conflict` |
| Rust | [rebuild_index](rust/basic/src/bin/rebuild_index.rs) | ファイルからSQLiteカタログを再構築 | `cargo run --manifest-path examples/rust/basic/Cargo.toml --bin rebuild_index` |

Pythonサンプルはasync-firstの公開APIを使います。Rustサンプルは`dirdb-core`を直接使います。どちらも破棄可能なデータを`example-state/`以下へ保存します。

English guide: [README.md](README.md)
