# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
