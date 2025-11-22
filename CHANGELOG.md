# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.11] - 2025-01-23

### Fixed

#### Critical Database Pool Bugs (5 race conditions)
- **DbPool::acquire()**: Fixed race condition where multiple threads could exceed max_connections
  - Reserve slot by incrementing in_use before creating connection
  - Decrement in_use on connection creation failure
- **DbPool::release()**: Fixed race condition causing count inconsistency
  - Decrement in_use before adding to available pool (preserves invariant)
- **DbPool::close()**: Fixed error handling that left pool in inconsistent state
  - Collect all errors, reset in_use even on failure
- **DbPool::stats()**: Fixed inconsistent snapshot (read available and in_use separately)
  - Hold both locks simultaneously for consistent snapshot
- **native_pool_stats()**: Minimized lock hold time
  - Clone pool before releasing POOLS lock (consistent with other pool operations)

#### Database Bugs from Codex Analysis (9 issues)
- **Transaction management**: Fixed commit/rollback removing tx from map before operation completes
- **Metadata functions**: Fixed holding mutex during driver calls (deadlock risk)
- **PostgreSQL/MySQL**: Fixed ignoring transaction isolation level options
  - PostgreSQL: Use `BEGIN TRANSACTION ISOLATION LEVEL ...` single command
  - MySQL: Use `SET TRANSACTION ISOLATION LEVEL ...; START TRANSACTION`
- **ConnectionOptions/QueryOptions**: Fixed ignoring keyword keys (only checked string keys)
- **db/pool-acquire**: Fixed deadlock (holding POOLS mutex during pool.acquire())
- **db/pool-release**: Fixed accepting connections during active transaction
- **db/pool-close**: Fixed not checking for checked-out connections
- **kvs/mset**: Fixed MapKey conversion bug (to_string() returns debug format)
- **db/call**: Added transaction support for stored procedures

## [0.1.10] - 2025-01-22

### Added

#### Binary Data Support (Bytes Type)
- **New Type**: `Bytes` - Immutable binary data type backed by `Arc<[u8]>`
- **Functions (3)**:
  - `bytes` - Create Bytes from integer vector (0-255 range validation)
  - `bytes/to-vec` - Convert Bytes to integer vector
  - `bytes?` - Type predicate for Bytes
- **I/O Integration**:
  - File I/O: `:encoding :binary` option for `io/read-file` and `io/write-file`
  - HTTP client: Automatic UTF-8 detection, returns Bytes for binary responses
  - HTTP server: Binary request/response body support
  - Database: BLOB support for PostgreSQL (BYTEA), MySQL (BLOB), SQLite (BLOB)
  - JSON/YAML: Base64 encoding for serialization
- **Documentation**:
  - Added Bytes section to `docs/spec/06-data-structures.md` (ja/en)
  - Example: `examples/24-binary-data.qi`, `examples/25-database-blob.qi`
  - Updated `std/docs/{ja,en}/bytes.qi`, `http.qi`, `server.qi`

#### Development Tools
- **CI/CD Script**: `scripts/pre-commit-check.sh`
  - Runs all CI checks locally before commit
  - Prevents CI build failures (format, clippy, test, build)
  - Added documentation to `CLAUDE.md`

### Fixed

#### Critical Security & Safety Fixes (21 issues)
- **Binary Data Handling**:
  - Fixed HTTP response body corruption (`.text()` → `.bytes()` with UTF-8 check)
  - Fixed server request body corruption (`from_utf8_lossy` → UTF-8 check)
  - Fixed file streaming silent failures (proper error propagation)
  - Fixed MapKey consistency (`From<&str>` now creates String keys)
- **Memory Safety & Type Safety**:
  - Float→i64 conversion overflow protection
  - Negative index safety improvements
  - PostgreSQL connection resource leaks
  - Channel error handling (producer/consumer independence)
  - HTTP client lazy initialization race conditions
  - Loop protection against infinite recursion
- **Code Quality**:
  - Fixed 11 Clippy warnings (CI/CD compliance)
  - Fixed benchmark lifetime errors
  - Fixed format consistency issues

### Changed
- **Bytes Comparison**: Added proper equality comparison for Bytes type
- **Error Messages**: All error messages are i18n-compliant

## [0.1.9] - 2025-01-23

### Added

#### REPL Enhancements (Phase 4: Metaprogramming & Debugging)

**Metaprogramming Functions (2)**
- `macroexpand` - Expands macros (converts `defn`/`defn-` to `def` + `fn` form)
- `source` - Displays symbol definition source (distinguishes native/user-defined/macro)

**REPL Debug Commands (3)**
- `:test [path]` - Runs test file (all tests if no argument)
- `:trace <function>` - Traces function calls (lists traced functions if no argument)
- `:untrace [function]` - Stops tracing (stops all if no argument)

**Implementation Details**
- `src/builtins/core_state_meta.rs`: Added `macroexpand`, `source`, `expand_defn` (expanded to 10 functions)
- `src/builtins/debug.rs`: Added `TRACED_FUNCTIONS` global state (LazyLock)
- `src/eval/call.rs`: Added trace logging in `apply_func()`
- `src/main.rs`: Implemented `:test`, `:trace`, `:untrace` commands with tab completion

**Documentation Updates**
- `std/docs/{ja,en}/core.qi`: Updated to "State Management & Metaprogramming (10 functions)"
- `docs/{ja,en}/cli.md`: Added test/debug commands and usage examples
- `docs/{ja,en}/tutorial/01-getting-started.md`: Added Phase 3 features

#### REPL Enhancements (Phase 1-3)

**Phase 1: Foundation Improvements**
- History persistence to `~/.qi/history`
- Multi-line editing with automatic parenthesis balance detection (`Validator` trait)
- Colored stack traces and error messages
- Auto table display for `Vector<Map>` results
- `:threads` command for concurrency debugging (Rayon thread pool info and channel status)

**Phase 2: Usability Improvements**
- Syntax highlighting (special forms, operators, strings, numbers, comments)
- Enhanced tab completion (special forms, pipe operators, 40+ common built-in functions)
- Result labels (`$1`, `$2`, `$3`...) to reference previous results
- Automatic execution time display (milliseconds/microseconds)
- Enhanced `:doc` command with:
  - Colored output
  - Return value display
  - Related functions
  - Similar function suggestions (edit distance-based)

**Phase 3: Advanced Features**
- Hot reload with `:watch <file>` and `:unwatch` commands (file monitoring and auto-reload)
- REPL macros:
  - `:macro define <name> <command>` - Define shortcuts
  - `:macro list` - List all macros
  - `:macro delete <name>` - Delete macros
  - `:m <name>` - Execute macros
  - Macros persisted to `~/.qi/macros` (JSON format)
- Profiling:
  - `:profile start/stop` - Control profiling
  - `:profile report` - Display statistics (total, avg, max, min, slowest evaluations)
  - `:profile clear` - Clear data

**New Dependencies**
- `notify` (6.1) - File watching for hot reload
- `comfy-table` (7.1) - Table formatting
- `colored` (2.1) - Terminal colors
- `serde_json` (via `format-json`) - Macro persistence

## [0.1.0] - 2025-01-20

### Added

#### Core Language Features
- Basic Lisp syntax with 8 special forms (`def`, `fn`, `let`, `do`, `if`, `match`, `try`, `defer`)
- Pattern matching with guards and or-patterns
- Multiple pipeline operators (`|>`, `|>?`, `||>`, `~>`, `tap>`)
- Error handling with Result type (`{:ok/:error}`)
- Module system with `module`, `export`, `use`, `load`

#### Concurrency
- 3-layer concurrency architecture (go/chan, pipeline, async/await)
- Thread-safe operations with `Arc<RwLock<_>>`
- Parallel collection functions (`pmap`, `async/pfilter`, `async/preduce`)
- Structured concurrency (`async/with-scope`, `cancel!`)
- Channel operations with timeout and `select!`

#### Standard Library
- **Core**: 90+ functions (map, filter, reduce, etc.)
- **String** (`str/`): 60+ functions for manipulation, encoding, formatting
- **Math** (`math/`): Basic operations, random numbers
- **Stats** (`stats/`): Statistical functions (mean, median, stddev, percentile)
- **List** (`list/`): Advanced list operations
- **Map** (`map/`): Map utilities
- **I/O** (`io/`): File operations with multi-encoding support
- **JSON** (`json/`): Parse and stringify
- **HTTP** (`http/`): Client and server
- **Test** (`test/`): Testing framework

#### CLI Features
- REPL with history
- Script execution
- One-liner execution (`-e`)
- Test runner (`qi test`)
- Code formatter (`qi fmt`)
- Multi-language error messages (English/Japanese via `QI_LANG` env var)

#### Documentation
- Quick reference guide (`docs/spec/QUICK-REFERENCE.md`) - one-page cheat sheet
- Complete function index (`docs/spec/FUNCTION-INDEX.md`) - 200+ functions organized by category
- Usage guidelines for lists vs vectors and parallel pipelines
- Implementation file references in all major chapters
- Documented multilingual support (`QI_LANG` environment variable)

### Changed

#### Railway Pipeline (`|>?`) - Breaking Change
- **New behavior**: `{:error}` is failure, everything else is success (no automatic `:ok` wrapping)
- `try` now returns raw values on success (not `{:ok value}`)
- Simplifies error handling patterns and HTTP/JSON integration
- Added `error?` predicate to check for error values

#### Error Messages
- Added source location information (line number, column number, source code excerpt)

### Performance

- Optimized parser memory usage (reduced String cloning)
- Optimized parallel collection functions for small datasets
- Optimized pattern matching with SmallVec
- Reduced Arc::clone overhead
- Reduced code duplication (~350 lines)

[Unreleased]: https://github.com/sanohiro/qi-lang/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sanohiro/qi-lang/releases/tag/v0.1.0
