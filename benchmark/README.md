# Benchmarks

The Python benchmarks measure end-to-end performance, including Python scheduling, the Rust binding, structured value conversion, filesystem I/O, and SQLite metadata updates.

### Async Throughput

```bash
python benchmark/python/async_throughput.py --items 1000 --concurrency 32
```

### Dictionary Round Trips

This uses `db["path"] = value` and `db["path"]` with nested dictionaries and lists. It measures the structured Python-to-Rust conversion path.

```bash
python benchmark/python/mapping_roundtrip.py --items 1000
```

Run it on the same machine and storage class when comparing changes. Treat the result as a regression signal rather than a universal performance claim.

Japanese guide: [README.ja.md](README.ja.md)
