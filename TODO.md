# DirDB TODO

This list tracks implementation work. The [design document](docs/design.md) explains the architecture and scope.

## v0.1 Foundation

- [x] Filesystem-first JSON documents
- [x] Atomic document writes
- [x] SQLite catalog and immutable revision records
- [x] Optimistic version checks
- [x] Rust core and Python bindings
- [x] Async Python API and dictionary-style access
- [x] Python and Rust examples
- [x] Rust/Python tests, benchmarks, wheel builds, and release CI

## v0.2 Local Reliability

- [ ] Process-safe path-level file locks
- [x] Configurable bounded memory cache with automatic invalidation
- [x] Native file watching, debounce, reload, and invalid-edit self-repair
- [ ] File watch API with event coalescing
- [ ] CLI for `get`, `set`, `list`, `inspect`, and `rebuild-index`
- [ ] Metadata consistency audit and repair command
- [ ] Platform-specific atomic replacement behavior and retry policy

## v0.3 Recovery

- [ ] Logical snapshots with retention policy
- [ ] Maintenance mode that blocks normal writes
- [ ] `plan_restore()` with dry-run output
- [ ] `apply_restore()` through verified staging files
- [ ] `merge` and explicit `mirror` recovery modes
- [ ] Recovery and interrupted-write fault-injection tests

## Python API

- [ ] Typed exception classes for not found, version conflict, path, and storage errors
- [ ] `AsyncDirDB` convenience facade or documented lifecycle policy
- [ ] Async batch APIs after core batch semantics exist
- [ ] Type stubs and API reference generation

## Performance and Quality

- [ ] Criterion benchmarks for the Rust core
- [ ] Benchmark baselines stored per release
- [ ] Cache, large-document, and concurrent-write benchmark scenarios
- [ ] Windows, macOS, and Linux wheel-install integration tests
- [ ] Property and crash-recovery tests for path and write behavior

## Separate Server Project

- [ ] Local IPC adapter: Unix socket and Windows Named Pipe
- [ ] gRPC/HTTP2 service with TLS and authentication
- [ ] `BatchGet`, `BatchSet`, compare-and-set, and streaming `Watch`
- [ ] Change sequence log: path, operation, version, content hash
- [ ] `not_modified` responses for matching client versions/hashes
- [ ] Inline only small values; fetch full documents on demand
- [ ] Snapshot bootstrap and log catch-up for replicas

## Release Operations

- [ ] Confirm PyPI Trusted Publisher settings against the `ci.yml` workflow and environment
- [ ] Add changelog and version-tag validation
- [ ] Decide release versioning policy and supported Python/Rust versions
