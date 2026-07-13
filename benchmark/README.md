# Benchmarks

The Python benchmarks measure end-to-end performance, including Python scheduling, the Rust binding, structured value conversion, filesystem I/O, and SQLite metadata updates.

### Async Throughput

```bash
python benchmark/python/async_throughput.py --items 1000 --concurrency 32
```

This reports individual async operations and native batch operations separately.
Batch writes retain one `fsync` per authoritative file; batch reads primarily
measure the single Python/Rust boundary crossing and warm Rust cache.

### Dictionary Round Trips

This uses `db["path"] = value` and `db["path"]` with nested dictionaries and lists. It measures the structured Python-to-Rust conversion path.

```bash
python benchmark/python/mapping_roundtrip.py --items 1000 --read-rounds 10
```

The result separates the first pass from repeated warm-cache reads and prints
Rust cache hit/miss counters.

Run it on the same machine and storage class when comparing changes. Treat the result as a regression signal rather than a universal performance claim.

Japanese guide: [README.ja.md](README.ja.md)
