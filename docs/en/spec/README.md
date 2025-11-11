# Qi Language Specification

**Complete language specification and reference for Qi**

This directory contains the specification for implemented features only of the Qi language.

---

## 📚 Table of Contents

**⚡ Quick Start**: [Quick Reference](QUICK-REFERENCE.md) - Learn Qi basics in one page

### Core Features (★ Highlights)

- **[02-flow-pipes.md](02-flow-pipes.md)** - Pipeline Operators and Data Flow ⭐
  - `|>`, `||>`, `|>?`, `tap>`, `~>` operators
  - stream (lazy evaluation)
  - Designing data flow

- **[03-concurrency.md](03-concurrency.md)** - Concurrency and Parallelism ⭐
  - go/chan (goroutine-style)
  - async/await, pmap, pipeline
  - Atom (thread-safe state management)

- **[04-match.md](04-match.md)** - Pattern Matching ⭐
  - Data structure destructuring
  - Guard conditions, or patterns
  - Railway Oriented Programming

### Basics

- **[01-overview.md](01-overview.md)** - Overview of Qi
  - Language philosophy (Flow-Oriented Programming)
  - Design principles
  - Core design

- **[05-syntax-basics.md](05-syntax-basics.md)** - Basic Syntax
  - Data types, literals, comments
  - Special forms (def, fn, let, do, if, match, loop/recur, when, while, until, while-some, until-error)
  - Operators

- **[06-data-structures.md](06-data-structures.md)** - Data Structures
  - Vectors, lists, maps, sets
  - Higher-order functions (map, filter, reduce, each)
  - Sorting, grouping

- **[07-functions.md](07-functions.md)** - Functions
  - Function definition (fn, defn)
  - Closures
  - Higher-order functions (comp, partial, apply, identity)

- **[08-error-handling.md](08-error-handling.md)** - Error Handling
  - Result type (value / `{:error ...}`) - also supports `{:ok value}` format for validation
  - try/catch
  - defer (resource management)

- **[09-modules.md](09-modules.md)** - Module System
  - module declaration, export
  - use, load
  - Namespace management

### Standard Library

- **[10-stdlib-string.md](10-stdlib-string.md)** - String Operations (60+ functions)
  - Search, conversion, case conversion, encoding, validation
- **[11-stdlib-http.md](11-stdlib-http.md)** - HTTP Client/Server
  - Client (GET/POST/PUT/DELETE), Server (routing, middleware)
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSON/YAML Processing
  - Parse, stringify, Result type integration
- **[13-stdlib-io.md](13-stdlib-io.md)** - File I/O (encoding support)
  - File read/write, multilingual encoding (Shift_JIS, GBK, Big5, etc.)
- **[14-stdlib-test.md](14-stdlib-test.md)** - Testing Framework ⭐
  - test/run, assertions (assert-eq, assert, assert-not, assert-throws)
  - qi test command (auto-detection, simple output)
- **[15-stdlib-math.md](15-stdlib-math.md)** - Math Functions
  - Power & root (pow, sqrt), rounding (round, floor, ceil), clamping (clamp)
  - Random generation (rand, rand-int, random-range, shuffle)
- **[16-stdlib-auth.md](16-stdlib-auth.md)** - Authentication & Authorization ⭐
  - JWT (JSON Web Token) generation, verification, decoding
  - Password hashing (Argon2)
- **[17-stdlib-database.md](17-stdlib-database.md)** - Database ⭐
  - PostgreSQL connection (query execution, command execution)
  - Parameterized queries, Result type integration
- **[18-stdlib-websocket.md](18-stdlib-websocket.md)** - WebSocket Communication ⭐
  - WebSocket client (connect, send, receive, close)
  - TLS/SSL support (wss://), message types (text, binary, close, error)
- **[19-stdlib-validation.md](19-stdlib-validation.md)** - Data Validation ⭐
  - Schema-based validation (type checking, required fields, string length, numeric range, pattern matching)
  - Nested map validation, Result type integration
- **[20-stdlib-debug.md](20-stdlib-debug.md)** - Debug Features ⭐
  - Tracing (debug/trace), breakpoints (debug/break)
  - Stack trace retrieval (debug/stack), debugger info (debug/info)
- **[21-stdlib-table.md](21-stdlib-table.md)** - Table Processing ⭐
  - SQL/awk-like tabular data operations
  - Grouping (table/group-by), deduplication (table/distinct-table)
- **[22-stdlib-csv.md](22-stdlib-csv.md)** - CSV Processing
  - CSV parsing/generation (csv/parse, csv/stringify)
  - File I/O (csv/read-file, csv/write-file), RFC 4180 compliant
- **[23-stdlib-env.md](23-stdlib-env.md)** - Environment Variables
  - Get/set environment variables (env/get, env/set)
  - Get all environment variables (env/all)
  - Load .env files (env/load-dotenv)
- **[24-stdlib-kvs.md](24-stdlib-kvs.md)** - Key-Value Store ⭐
  - Unified interface (Redis/Memcached etc.)
  - Basic ops, numeric ops, list, hash, set, batch operations (22 functions)
- **[25-stdlib-log.md](25-stdlib-log.md)** - Logging
  - Log levels (DEBUG, INFO, WARN, ERROR)
  - Formats (text, JSON), level setting (log/set-level)
- **[26-stdlib-markdown.md](26-stdlib-markdown.md)** - Markdown Processing
  - Markdown generation (markdown/header, markdown/list, markdown/table, etc.)
  - Markdown parsing (markdown/parse), static site generation
- **[27-stdlib-path.md](27-stdlib-path.md)** - Path Operations
  - Path joining (path/join), filename extraction (path/basename)
  - Cross-platform support (Windows/Unix)
- **[28-stdlib-stats.md](28-stdlib-stats.md)** - Statistical Functions ⭐
  - Measures of central tendency (mean, median, mode)
  - Measures of dispersion (variance, stddev)
  - Measures of position (percentile)
- **[29-stdlib-stream.md](29-stdlib-stream.md)** - Stream Processing ⭐
  - Lazy evaluation (stream, stream/map, stream/filter)
  - Infinite sequences (stream/range, stream/iterate)
  - Memory-efficient large data processing
- **[30-stdlib-temp.md](30-stdlib-temp.md)** - Temporary Files
  - Temporary file creation (temp/temp-file)
  - Temporary directory creation (temp/temp-dir), automatic cleanup
- **[31-stdlib-time.md](31-stdlib-time.md)** - Date/Time Processing ⭐
  - ISO 8601 format (time/now-iso, time/format, time/parse)
  - Date/time arithmetic (time/add-days, time/diff-days), UTC/timezone
- **[32-stdlib-zip.md](32-stdlib-zip.md)** - ZIP Compression/Decompression ⭐
  - ZIP creation (zip/create), extraction (zip/extract), listing (zip/list)
  - gzip compression (zip/gzip), decompression (zip/gunzip)
  - Backup, log rotation, distribution archives
- **[33-stdlib-cmd.md](33-stdlib-cmd.md)** - External Command Execution ⭐
  - Command execution (cmd/exec, cmd/sh)
  - Pipelines (cmd/pipe), stream processing (cmd/stream-lines)
  - Security considerations (command injection prevention)
- **[34-stdlib-list.md](34-stdlib-list.md)** - List Extensions
  - Advanced list operations (list/take-while, list/chunk, list/find)
  - Frequency counting (list/frequencies), rotation (list/rotate)
- **[35-stdlib-map.md](35-stdlib-map.md)** - Map Extensions
  - Nested map operations (map/assoc-in, map/dissoc-in)
  - Key/value transformation (map/update-keys, map/update-vals)
- **[36-stdlib-set.md](36-stdlib-set.md)** - Set Operations
  - Mathematical set operations (union, intersect, difference, symmetric-difference)
  - Subset checking (subset?, superset?, disjoint?)

---

## 🎯 Qi Features

### 1. Flow-Oriented Programming

**"Data flows, programs design the flow"**

```qi
(data
 |> parse
 |> (filter valid?)
 |> (map transform)
 |> save)
```

### 2. Easy Concurrency & Parallelism

**Qi's Essence - Thread-safe and Natural Parallelization**

```qi
;; Parallel pipeline
(urls ||> http/get ||> json/parse)

;; goroutine-style concurrency
(def result (data ~> transform ~> process))
(recv! result)
```

### 3. Pattern Matching

**Branch and Transform Data Flow**

```qi
;; HTTP response pattern matching (try catches errors)
(match (try (http/get! url))  ;; Detailed version to get status code
  {:error e} -> (log-error e)
  {:status 200 :body body} -> (process-body body)
  {:status 404} -> nil
  {:status _} -> (error "Unexpected status"))
```

---

## 📖 How to Read This Documentation

### For Beginners

1. [01-overview.md](01-overview.md) - What is Qi?
2. [05-syntax-basics.md](05-syntax-basics.md) - Learn basic syntax
3. [06-data-structures.md](06-data-structures.md) - How to handle data
4. [02-flow-pipes.md](02-flow-pipes.md) - Try using pipelines
5. [10-stdlib-string.md](10-stdlib-string.md) - Learn string operations

### For Intermediate Users

1. [04-match.md](04-match.md) - Utilize pattern matching
2. [07-functions.md](07-functions.md) - Functional programming
3. [08-error-handling.md](08-error-handling.md) - Error handling strategies
4. [03-concurrency.md](03-concurrency.md) - Utilize concurrent processing
5. [11-stdlib-http.md](11-stdlib-http.md) - Build HTTP client/server
6. [13-stdlib-io.md](13-stdlib-io.md) - File I/O and encoding

### For Advanced Users

1. [03-concurrency.md](03-concurrency.md) - 3-tier concurrency architecture
2. [09-modules.md](09-modules.md) - Module design
3. [02-flow-pipes.md](02-flow-pipes.md) - stream (lazy evaluation)
4. [12-stdlib-json.md](12-stdlib-json.md) - JSON/YAML pipeline processing

---

## 🔍 Function & Operator Index

### Special Forms (14)

- `def`, `defn`, `defn-` - Definition → [05-syntax-basics.md](05-syntax-basics.md)
- `fn` - Function definition → [05-syntax-basics.md](05-syntax-basics.md), [07-functions.md](07-functions.md)
- `let` - Local binding → [05-syntax-basics.md](05-syntax-basics.md)
- `if`, `do` - Control structures → [05-syntax-basics.md](05-syntax-basics.md)
- `when` - Execute only when condition is true → [05-syntax-basics.md](05-syntax-basics.md)
- `while` - Loop while condition is true → [05-syntax-basics.md](05-syntax-basics.md)
- `until` - Loop until condition is true → [05-syntax-basics.md](05-syntax-basics.md)
- `while-some` - Loop until nil (with binding) → [05-syntax-basics.md](05-syntax-basics.md)
- `until-error` - Loop until error (with binding) → [05-syntax-basics.md](05-syntax-basics.md)
- `loop`, `recur` - Loop → [05-syntax-basics.md](05-syntax-basics.md)
- `match` - Pattern matching → [04-match.md](04-match.md)
- `try`, `defer` - Error handling → [08-error-handling.md](08-error-handling.md)
- `mac` - Macro → [05-syntax-basics.md](05-syntax-basics.md)
- `module`, `export`, `use` - Modules → [09-modules.md](09-modules.md)

### Pipeline Operators (5) ⭐

- `|>` - Sequential pipe → [02-flow-pipes.md](02-flow-pipes.md)
- `|>?` - Railway Pipeline (error handling) → [02-flow-pipes.md](02-flow-pipes.md), [08-error-handling.md](08-error-handling.md)
- `||>` - Parallel pipe → [02-flow-pipes.md](02-flow-pipes.md)
- `~>` - Async pipe → [02-flow-pipes.md](02-flow-pipes.md), [03-concurrency.md](03-concurrency.md)
- `tap>` - Side effect tap → [02-flow-pipes.md](02-flow-pipes.md)

### Core Functions (commonly used)

**Numeric Operations**:
- `+`, `-`, `*`, `/`, `%` - Arithmetic operations → [05-syntax-basics.md](05-syntax-basics.md)
- `abs`, `min`, `max`, `inc`, `dec`, `sum` - Numeric functions → [06-data-structures.md](06-data-structures.md)
- `=`, `<`, `>`, `<=`, `>=` - Comparison operations → [05-syntax-basics.md](05-syntax-basics.md)

**Collections**:
- `first`, `rest`, `last`, `nth` - Access → [06-data-structures.md](06-data-structures.md)
- `cons`, `conj`, `concat` - Concatenation → [06-data-structures.md](06-data-structures.md)
- `take`, `drop`, `filter`, `map`, `reduce`, `each` - Transformation → [06-data-structures.md](06-data-structures.md)
- `sort`, `reverse`, `distinct` - Sorting & deduplication → [06-data-structures.md](06-data-structures.md)

**Strings**:
- `str`, `split`, `join` - Basic operations → [05-syntax-basics.md](05-syntax-basics.md)
- 60+ string functions → [10-stdlib-string.md](10-stdlib-string.md)

**Predicates (23)**:
- `nil?`, `some?`, `empty?` - nil/existence check → [05-syntax-basics.md](05-syntax-basics.md)
- `number?`, `string?`, `list?`, `vector?`, `map?` - Type checking → [05-syntax-basics.md](05-syntax-basics.md)
- `even?`, `odd?`, `positive?`, `negative?`, `zero?` - Numeric predicates → [05-syntax-basics.md](05-syntax-basics.md)
- `error?` - Error checking → [05-syntax-basics.md](05-syntax-basics.md), [08-error-handling.md](08-error-handling.md)

**I/O**:
- `print`, `println` - Output → [05-syntax-basics.md](05-syntax-basics.md)
- File I/O → [13-stdlib-io.md](13-stdlib-io.md)

**Concurrency** ⭐:
- `go/chan`, `go/send!`, `go/recv!` - goroutine-style → [03-concurrency.md](03-concurrency.md)
- `pmap`, `pfilter`, `preduce` - Parallel map/filter/reduce → [03-concurrency.md](03-concurrency.md)
- `atom`, `swap!`, `reset!`, `deref` - Thread-safe state management → [03-concurrency.md](03-concurrency.md)

### Standard Library Functions

- **HTTP**: `http/get`, `http/post`, `server/serve` → [11-stdlib-http.md](11-stdlib-http.md)
- **WebSocket**: `ws/connect`, `ws/send`, `ws/receive`, `ws/close` → [18-stdlib-websocket.md](18-stdlib-websocket.md)
- **JSON/YAML**: `json/parse`, `json/stringify`, `yaml/parse` → [12-stdlib-json.md](12-stdlib-json.md)
- **Math**: `math/pow`, `math/sqrt`, `math/round`, `math/rand` → [15-stdlib-math.md](15-stdlib-math.md)
- **Stats**: `stats/mean`, `stats/median`, `stats/stddev`, `stats/percentile` → [28-stdlib-stats.md](28-stdlib-stats.md)
- **Test**: `test/assert-eq`, `test/run` → [14-stdlib-test.md](14-stdlib-test.md)
- **String**: `string/upper`, `string/lower`, `string/trim`, plus 60+ → [10-stdlib-string.md](10-stdlib-string.md)
- **Auth**: `jwt/sign`, `jwt/verify`, `password/hash`, `password/verify` → [16-stdlib-auth.md](16-stdlib-auth.md)
- **Database**: `db/connect`, `db/query`, `db/exec` (PostgreSQL/MySQL/SQLite) → [17-stdlib-database.md](17-stdlib-database.md)
- **Debug**: `debug/trace`, `debug/break`, `debug/stack`, `debug/info` → [20-stdlib-debug.md](20-stdlib-debug.md)

**📑 Complete Function Index**: [FUNCTION-INDEX.md](../../spec/FUNCTION-INDEX.md) - Detailed reference for all functions (generated by `./scripts/list_qi_functions.sh`)

---

## 🚀 Unimplemented Features

For unimplemented features and roadmap, please refer to `ROADMAP.md` in the project root.

---

## 📝 Documentation Policy

Documentation in this directory:

- **Only implemented features** - All code examples work
- **No Phase markers** - All implemented, Phase markers removed
- **Practical examples focus** - Not just concepts, but actual working code examples
- **Flow-Oriented** - Explanations aligned with Qi's philosophy

---

## 🌍 Multilingual Support

Qi supports **multilingual error messages**.

### Usage

You can specify the language using the `QI_LANG` environment variable:

```bash
# Display error messages in Japanese
QI_LANG=ja qi script.qi

# Display error messages in English (default)
QI_LANG=en qi script.qi
```

### Example

```bash
# Japanese error
$ QI_LANG=ja qi -e '(+ 1 "abc")'
エラー: 数値演算には数値のみを使用できます

# English error
$ QI_LANG=en qi -e '(+ 1 "abc")'
Error: Numeric operations require numbers only
```

Currently supported languages:
- **Japanese** (`ja`) - Default (for Japanese developers)
- **English** (`en`) - International support

**Implementation**: Messages are centrally managed in `src/i18n.rs`.

---

## 🔗 Related Documentation

- **[SPEC.md.archive](../../SPEC.md.archive)** - Original unified specification (archive)
- **[ROADMAP.md](../../ROADMAP.md)** - Unimplemented features and roadmap
- **[style-guide.md](../style-guide.md)** - Coding style guide
- **[README.md](../../README.md)** - Overall project description

---

## 📜 License

This documentation is part of the Qi language project and follows the same license.
