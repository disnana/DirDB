use dirdb_core::DirDb;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DirDb::open("example-state/rust")?;

    let created = db.set(
        "services/auth/config",
        &json!({
            "enabled": true,
            "providers": ["password", "passkey"]
        }),
        None,
    )?;

    let loaded = db.get("services/auth/config")?;
    println!("saved version: {}", created.version);
    println!("config: {}", loaded.value);
    println!("keys: {:?}", db.list("services")?);

    Ok(())
}
