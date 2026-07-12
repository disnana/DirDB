use dirdb_core::DirDb;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DirDb::open("example-state/rust/rebuild-index")?;
    db.set("services/auth", &json!({"enabled": true}), None)?;
    db.set("services/payment", &json!({"enabled": false}), None)?;

    let rebuilt = db.rebuild_index()?;
    println!("rebuilt entries: {rebuilt}");
    println!("catalog keys: {:?}", db.list("services")?);

    Ok(())
}
