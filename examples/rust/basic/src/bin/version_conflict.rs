use dirdb_core::{DirDb, Error};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = DirDb::open("example-state/rust/version-conflict")?;
    let first = db.set("services/payment/config", &json!({"enabled": false}), None)?;
    let second = db.set(
        "services/payment/config",
        &json!({"enabled": true}),
        Some(first.version),
    )?;
    println!("updated to version: {}", second.version);

    match db.set(
        "services/payment/config",
        &json!({"enabled": false}),
        Some(first.version),
    ) {
        Err(Error::VersionConflict { expected, actual, .. }) => {
            println!("stale update rejected: expected {expected}, actual {actual}");
        }
        Ok(_) => unreachable!("a stale update must not succeed"),
        Err(error) => return Err(error.into()),
    }

    Ok(())
}
