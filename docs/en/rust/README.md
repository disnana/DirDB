# Rust Guide

## Add the Core Crate

Within this repository, depend on the core crate by path:

```toml
[dependencies]
dirdb-core = { path = "../crates/dirdb-core" }
serde_json = "1"
```

## Store a Document

```rust
use dirdb_core::DirDb;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DirDb::open("./state")?;
    let saved = db.set("app/config", &json!({"theme": "dark"}), None)?;
    let config = db.get("app/config")?;
    println!("version: {}, value: {}", saved.version, config.value);
    Ok(())
}
```

The current core API is synchronous. It is intended for direct local use and releases no network abstraction into the core. Rust async service adapters belong to a later IPC or gRPC layer.

## Optimistic Writes

Use a returned version as `expected_version` on the next write. Concurrent modifications produce `Error::VersionConflict` instead of silently losing the newer change.

```rust
let first = db.set("app/config", &json!({"theme": "dark"}), None)?;
let second = db.set("app/config", &json!({"theme": "light"}), Some(first.version))?;
```

Run [the complete example](../../../examples/rust/basic) with:

```bash
cargo run --manifest-path examples/rust/basic/Cargo.toml
```

Japanese guide: [../../ja/rust/README.md](../../ja/rust/README.md)
