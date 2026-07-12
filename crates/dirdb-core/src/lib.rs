//! DirDB core: files are the source of truth; SQLite is rebuildable metadata.

use std::{
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid key: {0}")]
    InvalidKey(String),
    #[error("entry does not exist: {0}")]
    NotFound(String),
    #[error("version conflict for {path}: expected {expected}, actual {actual}")]
    VersionConflict {
        path: String,
        expected: u64,
        actual: u64,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub value: Value,
    pub version: u64,
    pub hash: String,
}

/// A local store. Its `data/` tree is authoritative and can rebuild `metadata.db`.
#[derive(Clone)]
pub struct DirDb {
    root: PathBuf,
    data_dir: PathBuf,
    connection: Arc<Mutex<Connection>>,
}

impl DirDb {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)?;
        let connection = Connection::open(root.join("metadata.db"))?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS entries (
                path TEXT PRIMARY KEY, version INTEGER NOT NULL,
                content_hash TEXT NOT NULL, modified_at INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS revisions (
                path TEXT NOT NULL, version INTEGER NOT NULL, content BLOB NOT NULL,
                content_hash TEXT NOT NULL, created_at INTEGER NOT NULL,
                PRIMARY KEY(path, version)
             );",
        )?;
        Ok(Self {
            root,
            data_dir,
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn get(&self, key: &str) -> Result<Entry> {
        let key = normalize_key(key)?;
        let bytes = match fs::read(self.file_path(&key)) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::NotFound(key))
            }
            Err(error) => return Err(error.into()),
        };
        let value = serde_json::from_slice(&bytes)?;
        let hash = digest(&bytes);
        let version = self.current_version(&key)?.unwrap_or(0);
        Ok(Entry {
            key,
            value,
            version,
            hash,
        })
    }

    pub fn exists(&self, key: &str) -> Result<bool> {
        let key = normalize_key(key)?;
        Ok(self.file_path(&key).is_file())
    }

    pub fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix = normalize_prefix(prefix)?;
        let base = self.data_dir.join(&prefix);
        if !base.exists() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        collect_keys(&self.data_dir, &base, &mut entries)?;
        entries.sort();
        Ok(entries)
    }

    /// Writes a JSON document atomically, then records its metadata and immutable revision.
    pub fn set(&self, key: &str, value: &Value, expected_version: Option<u64>) -> Result<Entry> {
        let key = normalize_key(key)?;
        let _connection = self.connection.lock();
        let actual = current_version_from_connection(&_connection, &key)?;
        if let Some(expected) = expected_version {
            if actual.unwrap_or(0) != expected {
                return Err(Error::VersionConflict {
                    path: key,
                    expected,
                    actual: actual.unwrap_or(0),
                });
            }
        }
        let bytes = serde_json::to_vec_pretty(value)?;
        let path = self.file_path(&key);
        atomic_write(&path, &bytes)?;
        let version = actual.unwrap_or(0) + 1;
        let hash = digest(&bytes);
        let now = now_unix();
        _connection.execute(
            "INSERT INTO entries(path, version, content_hash, modified_at) VALUES(?1, ?2, ?3, ?4)
             ON CONFLICT(path) DO UPDATE SET version=excluded.version, content_hash=excluded.content_hash, modified_at=excluded.modified_at",
            params![key, version, hash, now],
        )?;
        _connection.execute(
            "INSERT INTO revisions(path, version, content, content_hash, created_at) VALUES(?1, ?2, ?3, ?4, ?5)",
            params![key, version, bytes, hash, now],
        )?;
        Ok(Entry {
            key,
            value: value.clone(),
            version,
            hash,
        })
    }

    pub fn delete(&self, key: &str, expected_version: Option<u64>) -> Result<()> {
        let key = normalize_key(key)?;
        let _connection = self.connection.lock();
        let actual = current_version_from_connection(&_connection, &key)?
            .ok_or_else(|| Error::NotFound(key.clone()))?;
        if let Some(expected) = expected_version {
            if actual != expected {
                return Err(Error::VersionConflict {
                    path: key,
                    expected,
                    actual,
                });
            }
        }
        let path = self.file_path(&key);
        fs::remove_file(path)?;
        _connection.execute("DELETE FROM entries WHERE path=?1", params![key])?;
        Ok(())
    }

    /// Recreates catalog metadata and revision zero from the authoritative files.
    pub fn rebuild_index(&self) -> Result<usize> {
        let keys = self.list("")?;
        let mut connection = self.connection.lock();
        let transaction = connection.unchecked_transaction()?;
        transaction.execute("DELETE FROM entries", [])?;
        for key in &keys {
            let bytes = fs::read(self.file_path(key))?;
            transaction.execute(
                "INSERT INTO entries(path, version, content_hash, modified_at) VALUES(?1, 1, ?2, ?3)",
                params![key, digest(&bytes), now_unix()],
            )?;
        }
        transaction.commit()?;
        Ok(keys.len())
    }

    fn current_version(&self, key: &str) -> Result<Option<u64>> {
        let connection = self.connection.lock();
        current_version_from_connection(&connection, key)
    }
    fn file_path(&self, key: &str) -> PathBuf {
        self.data_dir.join(format!("{key}.json"))
    }
}

fn current_version_from_connection(connection: &Connection, key: &str) -> Result<Option<u64>> {
    Ok(connection
        .query_row(
            "SELECT version FROM entries WHERE path=?1",
            params![key],
            |row| row.get(0),
        )
        .ok())
}

fn normalize_key(key: &str) -> Result<String> {
    let key = key.trim_matches('/');
    if key.is_empty() {
        return Err(Error::InvalidKey("key must not be empty".into()));
    }
    let path = Path::new(key);
    if path.components().any(|part| {
        matches!(
            part,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(Error::InvalidKey(key.into()));
    }
    Ok(key.replace('\\', "/"))
}
fn normalize_prefix(prefix: &str) -> Result<String> {
    if prefix.is_empty() {
        return Ok(String::new());
    }
    normalize_key(prefix)
}
fn collect_keys(data_dir: &Path, directory: &Path, output: &mut Vec<String>) -> Result<()> {
    for item in fs::read_dir(directory)? {
        let item = item?;
        let path = item.path();
        if path.is_dir() {
            collect_keys(data_dir, &path, output)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let relative = path.strip_prefix(data_dir).expect("inside data directory");
            let mut key = relative
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");
            if key.ends_with(".json") {
                key.truncate(key.len() - 5);
            }
            output.push(key);
        }
    }
    Ok(())
}
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().expect("data file has a parent");
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap().to_string_lossy(),
        Uuid::new_v4()
    ));
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    fs::rename(&temporary, path)?;
    Ok(())
}
fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stores_documents_as_files_and_versions_them() {
        let directory = tempfile::tempdir().unwrap();
        let db = DirDb::open(directory.path()).unwrap();
        let first = db
            .set("app/theme", &serde_json::json!({"theme": "dark"}), None)
            .unwrap();
        assert_eq!(first.version, 1);
        assert!(directory.path().join("data/app/theme.json").is_file());
        let second = db
            .set("app/theme", &serde_json::json!({"theme": "light"}), Some(1))
            .unwrap();
        assert_eq!(second.version, 2);
        assert!(matches!(
            db.set("app/theme", &serde_json::json!({}), Some(1)),
            Err(Error::VersionConflict { .. })
        ));
    }
}
