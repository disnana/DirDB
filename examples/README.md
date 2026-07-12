# DirDB Examples

| Language | Example | Focus | Run |
| --- | --- | --- | --- |
| Python | [async_basic.py](python/async_basic.py) | Async document read/write | `python examples/python/async_basic.py` |
| Python | [dict_mapping.py](python/dict_mapping.py) | `dict`-style access and defaults | `python examples/python/dict_mapping.py` |
| Python | [async_concurrency.py](python/async_concurrency.py) | Bounded concurrent async writes | `python examples/python/async_concurrency.py` |
| Python | [version_conflict.py](python/version_conflict.py) | Optimistic version checks | `python examples/python/version_conflict.py` |
| Rust | [basic](rust/basic/src/main.rs) | Direct core read/write | `cargo run --manifest-path examples/rust/basic/Cargo.toml` |
| Rust | [version_conflict](rust/basic/src/bin/version_conflict.rs) | Optimistic version checks | `cargo run --manifest-path examples/rust/basic/Cargo.toml --bin version_conflict` |
| Rust | [rebuild_index](rust/basic/src/bin/rebuild_index.rs) | Rebuild SQLite catalog from files | `cargo run --manifest-path examples/rust/basic/Cargo.toml --bin rebuild_index` |

The Python example uses the async-first public API. The Rust example uses `dirdb-core` directly. Both write disposable data beneath `example-state/`.

Japanese guide: [README.ja.md](README.ja.md)
