use dirdb_core::DirDb;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DirDb::open("./example-state/rust-batch")?;
    let writes = db.set_many(&[
        ("services/auth".into(), json!({"enabled": true})),
        ("services/cache".into(), json!({"ttl": 60})),
    ])?;
    for write in writes {
        println!("version={}", write.version);
    }

    for entry in db.get_many(&["services/auth".into(), "services/cache".into()]) {
        let entry = entry?;
        println!("{}: {}", entry.key, entry.value);
    }
    Ok(())
}
