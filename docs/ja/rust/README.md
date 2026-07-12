# Rustガイド

## コアクレートの追加

このリポジトリ内では、パス依存としてコアクレートを追加します。

```toml
[dependencies]
dirdb-core = { path = "../crates/dirdb-core" }
serde_json = "1"
```

## ドキュメントの保存

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

現在のコアAPIは同期です。直接的なローカル利用を対象としており、ネットワーク抽象化をコアへ持ち込みません。Rustのasyncサービスアダプターは、将来のIPCまたはgRPC層に属します。

## 楽観的書き込み

最初の書き込みで返ったバージョンを、次の書き込みの`expected_version`として渡します。同時更新が起きた場合は新しい変更を黙って失わず、`Error::VersionConflict`が返ります。

```rust
let first = db.set("app/config", &json!({"theme": "dark"}), None)?;
let second = db.set("app/config", &json!({"theme": "light"}), Some(first.version))?;
```

[完全なサンプル](../../../examples/rust/basic)は、次で実行できます。

```bash
cargo run --manifest-path examples/rust/basic/Cargo.toml
```

English guide: [../../en/rust/README.md](../../en/rust/README.md)
