# Benchmarks

The Python benchmark measures end-to-end async write and read throughput, including Python scheduling, the Rust binding, JSON serialization, filesystem I/O, and SQLite metadata updates.

```bash
python benchmark/python/async_throughput.py --items 1000 --concurrency 32
```

Run it on the same machine and storage class when comparing changes. Treat the result as a regression signal rather than a universal performance claim.

Japanese guide: [README.ja.md](README.ja.md)
