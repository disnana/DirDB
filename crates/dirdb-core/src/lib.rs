//! DirDB core: files are authoritative; memory and SQLite are rebuildable helpers.

use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    num::NonZeroUsize,
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc, Weak,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use lru::LruCache;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
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
    #[error("file watcher error: {0}")]
    Watch(String),
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

#[derive(Clone, Debug)]
pub struct Options {
    pub cache_max_items: usize,
    pub auto_reload: bool,
    pub debounce: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            cache_max_items: 10_000,
            auto_reload: true,
            debounce: Duration::from_millis(100),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryStatus {
    pub file_valid: bool,
    pub current_version: u64,
    pub last_reload_error: Option<String>,
}

struct Inner {
    root: PathBuf,
    data_dir: PathBuf,
    connection: Mutex<Connection>,
    cache: Mutex<LruCache<String, Entry>>,
    reload_errors: Mutex<HashMap<String, String>>,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

/// A local store. Its `data/` tree is authoritative and can rebuild `metadata.db`.
#[derive(Clone)]
pub struct DirDb {
    inner: Arc<Inner>,
}

impl DirDb {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_options(root, Options::default())
    }

    pub fn open_with_options(root: impl AsRef<Path>, options: Options) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)?;
        let connection = Connection::open(root.join("metadata.db"))?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
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
        let capacity = NonZeroUsize::new(options.cache_max_items.max(1)).expect("non-zero cache");
        let inner = Arc::new(Inner {
            root,
            data_dir,
            connection: Mutex::new(connection),
            cache: Mutex::new(LruCache::new(capacity)),
            reload_errors: Mutex::new(HashMap::new()),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        });
        if options.auto_reload {
            start_watcher(Arc::downgrade(&inner), options.debounce)?;
        }
        Ok(Self { inner })
    }

    pub fn root(&self) -> &Path {
        &self.inner.root
    }

    pub fn get(&self, key: &str) -> Result<Entry> {
        let key = normalize_key(key)?;
        if let Some(entry) = self.inner.cache.lock().get(&key).cloned() {
            self.inner.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(entry);
        }
        self.inner.cache_misses.fetch_add(1, Ordering::Relaxed);
        let entry = load_entry(&self.inner, &key)?;
        self.inner.cache.lock().put(key, entry.clone());
        Ok(entry)
    }

    pub fn exists(&self, key: &str) -> Result<bool> {
        let key = normalize_key(key)?;
        Ok(file_path(&self.inner, &key).is_file())
    }

    pub fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix = normalize_prefix(prefix)?;
        let base = self.inner.data_dir.join(&prefix);
        if !base.exists() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        collect_keys(&self.inner.data_dir, &base, &mut entries)?;
        entries.sort();
        Ok(entries)
    }

    /// Writes a JSON document atomically, then records metadata and an immutable revision.
    pub fn set(&self, key: &str, value: &Value, expected_version: Option<u64>) -> Result<Entry> {
        let key = normalize_key(key)?;
        let connection = self.inner.connection.lock();
        let actual = current_version_from_connection(&connection, &key)?;
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
        atomic_write(&file_path(&self.inner, &key), &bytes)?;
        let version = actual.unwrap_or(0) + 1;
        let hash = digest(&bytes);
        record_revision(&connection, &key, version, &hash, &bytes)?;
        let entry = Entry {
            key: key.clone(),
            value: value.clone(),
            version,
            hash,
        };
        self.inner.cache.lock().put(key.clone(), entry.clone());
        self.inner.reload_errors.lock().remove(&key);
        Ok(entry)
    }

    pub fn delete(&self, key: &str, expected_version: Option<u64>) -> Result<()> {
        let key = normalize_key(key)?;
        let connection = self.inner.connection.lock();
        let actual = current_version_from_connection(&connection, &key)?
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
        fs::remove_file(file_path(&self.inner, &key))?;
        connection.execute("DELETE FROM entries WHERE path=?1", params![key])?;
        self.inner.cache.lock().pop(&key);
        self.inner.reload_errors.lock().remove(&key);
        Ok(())
    }

    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.inner.cache_hits.load(Ordering::Relaxed),
            misses: self.inner.cache_misses.load(Ordering::Relaxed),
            entries: self.inner.cache.lock().len(),
        }
    }

    pub fn stat(&self, key: &str) -> Result<EntryStatus> {
        let key = normalize_key(key)?;
        let current_version = self.current_version(&key)?.unwrap_or(0);
        let error = self.inner.reload_errors.lock().get(&key).cloned();
        Ok(EntryStatus {
            file_valid: error.is_none() && file_path(&self.inner, &key).is_file(),
            current_version,
            last_reload_error: error,
        })
    }

    /// Recreates catalog metadata from authoritative files and clears stale cache state.
    pub fn rebuild_index(&self) -> Result<usize> {
        let keys = self.list("")?;
        let connection = self.inner.connection.lock();
        let transaction = connection.unchecked_transaction()?;
        transaction.execute("DELETE FROM entries", [])?;
        for key in &keys {
            let bytes = fs::read(file_path(&self.inner, key))?;
            let _: Value = serde_json::from_slice(&bytes)?;
            transaction.execute(
                "INSERT INTO entries(path, version, content_hash, modified_at) VALUES(?1, 1, ?2, ?3)",
                params![key, digest(&bytes), now_unix()],
            )?;
        }
        transaction.commit()?;
        self.inner.cache.lock().clear();
        self.inner.reload_errors.lock().clear();
        Ok(keys.len())
    }

    fn current_version(&self, key: &str) -> Result<Option<u64>> {
        current_version_from_connection(&self.inner.connection.lock(), key)
    }
}

fn load_entry(inner: &Inner, key: &str) -> Result<Entry> {
    let bytes = match fs::read(file_path(inner, key)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::NotFound(key.to_owned()))
        }
        Err(error) => return Err(error.into()),
    };
    let value = serde_json::from_slice(&bytes)?;
    let hash = digest(&bytes);
    let version = current_version_from_connection(&inner.connection.lock(), key)?.unwrap_or(0);
    Ok(Entry {
        key: key.to_owned(),
        value,
        version,
        hash,
    })
}

fn refresh_external(inner: &Inner, key: &str) {
    let path = file_path(inner, key);
    if !path.is_file() {
        inner.cache.lock().pop(key);
        inner.reload_errors.lock().remove(key);
        let _ = inner
            .connection
            .lock()
            .execute("DELETE FROM entries WHERE path=?1", params![key]);
        return;
    }
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            inner
                .reload_errors
                .lock()
                .insert(key.to_owned(), error.to_string());
            return;
        }
    };
    let value: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            if restore_last_valid(inner, key).is_ok() {
                inner.reload_errors.lock().remove(key);
            } else {
                inner
                    .reload_errors
                    .lock()
                    .insert(key.to_owned(), error.to_string());
            }
            return;
        }
    };
    let hash = digest(&bytes);
    let connection = inner.connection.lock();
    let known = current_metadata_from_connection(&connection, key)
        .ok()
        .flatten();
    if known
        .as_ref()
        .is_some_and(|(_, known_hash)| known_hash == &hash)
    {
        inner.reload_errors.lock().remove(key);
        return;
    }
    let version = known.map_or(1, |(version, _)| version + 1);
    if let Err(error) = record_revision(&connection, key, version, &hash, &bytes) {
        inner
            .reload_errors
            .lock()
            .insert(key.to_owned(), error.to_string());
        return;
    }
    inner.cache.lock().put(
        key.to_owned(),
        Entry {
            key: key.to_owned(),
            value,
            version,
            hash,
        },
    );
    inner.reload_errors.lock().remove(key);
}

fn restore_last_valid(inner: &Inner, key: &str) -> Result<()> {
    let cached = inner.cache.lock().peek(key).cloned();
    let bytes = if let Some(entry) = cached {
        serde_json::to_vec_pretty(&entry.value)?
    } else {
        inner
            .connection
            .lock()
            .query_row(
                "SELECT content FROM revisions WHERE path=?1 ORDER BY version DESC LIMIT 1",
                params![key],
                |row| row.get(0),
            )
            .optional()?
            .ok_or_else(|| Error::NotFound(key.to_owned()))?
    };
    let _: Value = serde_json::from_slice(&bytes)?;
    atomic_write(&file_path(inner, key), &bytes)
}

fn start_watcher(inner: Weak<Inner>, debounce: Duration) -> Result<()> {
    let data_dir = inner
        .upgrade()
        .expect("new store is alive")
        .data_dir
        .clone();
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    thread::Builder::new()
        .name("dirdb-watcher".into())
        .spawn(move || watcher_loop(inner, data_dir, debounce, ready_tx))?;
    ready_rx
        .recv()
        .map_err(|error| Error::Watch(error.to_string()))?
}

fn watcher_loop(
    inner: Weak<Inner>,
    data_dir: PathBuf,
    debounce: Duration,
    ready: mpsc::SyncSender<Result<()>>,
) {
    let (event_tx, event_rx) = mpsc::channel();
    let watcher: notify::Result<RecommendedWatcher> = notify::recommended_watcher(move |event| {
        let _ = event_tx.send(event);
    });
    let mut watcher = match watcher {
        Ok(watcher) => watcher,
        Err(error) => {
            let _ = ready.send(Err(Error::Watch(error.to_string())));
            return;
        }
    };
    if let Err(error) = watcher.watch(&data_dir, RecursiveMode::Recursive) {
        let _ = ready.send(Err(Error::Watch(error.to_string())));
        return;
    }
    let _ = ready.send(Ok(()));
    let mut dirty = HashSet::new();
    let mut last_event = Instant::now();
    loop {
        match event_rx.recv_timeout(debounce) {
            Ok(Ok(event)) => {
                for path in event.paths {
                    if let Some(key) = key_from_path(&data_dir, &path) {
                        dirty.insert(key);
                        last_event = Instant::now();
                    }
                }
            }
            Ok(Err(_)) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        if !dirty.is_empty() && last_event.elapsed() >= debounce {
            let Some(inner) = inner.upgrade() else { break };
            for key in dirty.drain() {
                refresh_external(&inner, &key);
            }
        }
        if inner.strong_count() == 0 {
            break;
        }
    }
}

fn record_revision(
    connection: &Connection,
    key: &str,
    version: u64,
    hash: &str,
    bytes: &[u8],
) -> Result<()> {
    let transaction = connection.unchecked_transaction()?;
    let now = now_unix();
    transaction.execute(
        "INSERT INTO entries(path, version, content_hash, modified_at) VALUES(?1, ?2, ?3, ?4)
         ON CONFLICT(path) DO UPDATE SET version=excluded.version, content_hash=excluded.content_hash, modified_at=excluded.modified_at",
        params![key, version, hash, now],
    )?;
    transaction.execute(
        "INSERT OR IGNORE INTO revisions(path, version, content, content_hash, created_at) VALUES(?1, ?2, ?3, ?4, ?5)",
        params![key, version, bytes, hash, now],
    )?;
    transaction.commit()?;
    Ok(())
}

fn current_version_from_connection(connection: &Connection, key: &str) -> Result<Option<u64>> {
    Ok(connection
        .query_row(
            "SELECT version FROM entries WHERE path=?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?)
}

fn current_metadata_from_connection(
    connection: &Connection,
    key: &str,
) -> Result<Option<(u64, String)>> {
    Ok(connection
        .query_row(
            "SELECT version, content_hash FROM entries WHERE path=?1",
            params![key],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?)
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

fn key_from_path(data_dir: &Path, path: &Path) -> Option<String> {
    if path.extension()? != "json" {
        return None;
    }
    path.strip_prefix(data_dir).ok().map(|relative| {
        relative
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/")
    })
}

fn collect_keys(data_dir: &Path, directory: &Path, output: &mut Vec<String>) -> Result<()> {
    for item in fs::read_dir(directory)? {
        let path = item?.path();
        if path.is_dir() {
            collect_keys(data_dir, &path, output)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let relative = path.strip_prefix(data_dir).expect("inside data directory");
            output.push(
                relative
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

fn file_path(inner: &Inner, key: &str) -> PathBuf {
    inner.data_dir.join(format!("{key}.json"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().expect("data file has a parent");
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().expect("file name").to_string_lossy(),
        Uuid::new_v4()
    ));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok::<_, std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
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

    #[test]
    fn repeated_reads_hit_bounded_cache() {
        let directory = tempfile::tempdir().unwrap();
        let db = DirDb::open_with_options(
            directory.path(),
            Options {
                cache_max_items: 1,
                auto_reload: false,
                ..Options::default()
            },
        )
        .unwrap();
        db.set("one", &serde_json::json!({"value": 1}), None)
            .unwrap();
        db.set("two", &serde_json::json!({"value": 2}), None)
            .unwrap();
        db.get("two").unwrap();
        assert_eq!(db.cache_stats().entries, 1);
        assert!(db.cache_stats().hits >= 1);
    }

    #[test]
    fn watcher_refreshes_valid_files_and_keeps_last_valid_value() {
        let directory = tempfile::tempdir().unwrap();
        let db = DirDb::open_with_options(
            directory.path(),
            Options {
                debounce: Duration::from_millis(20),
                ..Options::default()
            },
        )
        .unwrap();
        db.set("app/config", &serde_json::json!({"value": 1}), None)
            .unwrap();
        let path = directory.path().join("data/app/config.json");
        fs::write(&path, br#"{"value":2}"#).unwrap();
        wait_until(|| db.get("app/config").unwrap().value["value"] == 2);
        let version = db.get("app/config").unwrap().version;
        fs::write(&path, b"{").unwrap();
        wait_until(|| serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).is_ok());
        assert!(db.stat("app/config").unwrap().file_valid);
        assert_eq!(db.get("app/config").unwrap().value["value"], 2);
        assert_eq!(db.get("app/config").unwrap().version, version);
    }

    fn wait_until(mut predicate: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !predicate() {
            assert!(Instant::now() < deadline, "condition timed out");
            thread::sleep(Duration::from_millis(20));
        }
    }
}
