# DirDB サンプル

| 言語 | サンプル | 実行 |
| --- | --- | --- |
| Python | [async_basic.py](python/async_basic.py) | `python examples/python/async_basic.py` |
| Rust | [basic](rust/basic) | `cargo run --manifest-path examples/rust/basic/Cargo.toml` |

Pythonサンプルはasync-firstの公開APIを使います。Rustサンプルは`dirdb-core`を直接使います。どちらも破棄可能なデータを`example-state/`以下へ保存します。

English guide: [README.md](README.md)
