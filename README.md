# Qi - A Lisp that flows

**[日本語](README.ja.md)** | English

<p align="center">
  <img src="./assets/logo/qi-logo-full-512.png" alt="Qi Logo" width="400">
</p>

**A simple Lisp-based programming language for designing data flow. Strong support for pipelines, pattern matching, and concurrency.**

## ⚠️ Development Status

**This project is under active development (Pre-1.0)**

- Breaking changes occur frequently
- APIs and interfaces may change without notice
- Not recommended for production use
- Many features remain untested

Current development stage: **Alpha / Experimental**

---

## Features

- **Pipelines**: Express data flow intuitively with `|>` `|>?` `||>` `~>`
- **Pattern Matching**: Unified branching and transformation with powerful `match` expressions
- **Concurrency & Parallelism**: Goroutine-style concurrency with channels and parallel pipelines
- **Web Development**: JSON/HTTP support with Railway Pipeline for error handling
- **Authentication & Authorization**: JWT authentication, Argon2 password hashing, auth middleware
- **Databases**: PostgreSQL/MySQL/SQLite support (unified interface)
- **KVS**: Redis support (unified interface, Memcached/InMemory support planned)
- **Debugging**: Trace, breakpoints, stack traces (VSCode debugger support)
- **F-strings**: String interpolation and multi-line strings (`"""..."""`)
- **i18n**: English/Japanese error messages (`QI_LANG=ja`)


## Hello World

```qi
(defn greet [name]
  f"Hello, {name}!")

(println (greet "World"))
;; => Hello, World!
```

## Pipeline Examples

### Basic Pipeline
```qi
;; Filter and transform numbers
([1 2 3 4 5 6 7 8 9 10]
 |> (filter (fn [x] (> x 5)))
 |> (map (fn [x] (* x 2)))
 |> (reduce + 0))
;; => 90

;; String processing
("hello world"
 |> str/upper
 |> str/reverse)
;; => "DLROW OLLEH"
```

### Railway Pipeline - Error Handling
```qi
;; Everything except {:error} is treated as success (no :ok wrapping!)
(defn validate-positive [x]
  (if (> x 0)
    x                          ;; Plain value → success
    {:error "Must be positive"}))

(defn double [x]
  (* x 2))                     ;; Plain value → success

(defn format-result [x]
  f"Result: {x}")              ;; Plain value → success

;; Success case - values flow through
(10
 |>? validate-positive
 |>? double
 |>? format-result)
;; => "Result: 20"

;; Error case - errors propagate automatically
(-5
 |>? validate-positive
 |>? double                    ;; Not executed (short-circuit)
 |>? format-result)            ;; Not executed
;; => {:error "Must be positive"}
```

### Parallel Pipeline
```qi
;; ||> executes multiple operations in parallel
([1 2 3 4 5]
 ||> (fn [x] (* x 2))
 ||> (fn [x] (+ x 10))
 ||> (fn [x] (* x x)))
;; => [144, 196, 256, 324, 400]
```

## Quick Start

### Installation

```bash
# If Rust is installed
cargo install --path .

# Or
cargo build --release
```

### Upgrade

```bash
# Upgrade to the latest version
qi --upgrade
```

Qi can automatically upgrade itself to the latest release from GitHub. No need to manually build or install.

### Create a Project

```bash
# Basic project
qi new my-project
cd my-project
qi main.qi

# HTTP server project
qi new myapi --template http-server
cd myapi
qi main.qi
```

### Available Templates

```bash
qi template list
qi template info http-server
```

### REPL (Interactive Environment)

The Qi REPL provides a powerful interactive development environment with advanced features:

```bash
qi

# Inside REPL - with syntax highlighting and result labels
qi:1> (+ 1 2 3)
$1 => 6

qi:2> ([1 2 3 4 5] |> (map (fn [x] (* x 2))))
$2 => [2 4 6 8 10]

# Reference previous results
qi:3> (+ $1 $2)
$3 => 36

# Auto-display execution time for slow operations
qi:4> (range 1000000 |> (reduce + 0))
$4 => 499999500000
(125ms)
```

**REPL Features:**
- 🎨 **Syntax highlighting** - Color-coded keywords, operators, strings, numbers, and comments
- 📝 **Tab completion** - Functions, variables, REPL commands, special forms, and pipe operators
- 📚 **Enhanced documentation** - `:doc` shows parameters, return values, examples, and related functions
- 🔄 **Hot reload** - `:watch` monitors files and auto-reloads on changes
- ⚡ **Macros** - Define shortcuts for frequently used commands
- 📊 **Profiling** - Measure and analyze code performance
- 🧵 **Concurrency debugging** - Inspect thread pools and channels
- 📋 **Result history** - Access previous results with `$1`, `$2`, etc.
- 📊 **Auto table display** - Vector of maps automatically rendered as tables
- ⏱️ **Execution timing** - Shows evaluation time for operations

### Other Commands

```bash
# Execute script file
qi script.qi

# One-liner execution
qi -e '(+ 1 2 3)'

# Process piped input (automatically stored in stdin variable)
cat data.csv | qi -e '(stdin |> (map str/trim) |> (filter (fn [x] (> (len x) 0))))'
ls -l | qi -e '(count stdin)'

# Show help
qi --help
```

### Initialization File (.qi/init.qi)

During REPL and one-liner execution, initialization files are automatically loaded in the following order:

```bash
# 1. User global settings (priority)
~/.qi/init.qi

# 2. Project local settings
./.qi/init.qi
```

You can preload commonly used libraries or define convenience functions in initialization files:

```qi
;; Example ~/.qi/init.qi
;; Preload table processing library
(use "std/lib/table" :as table)

;; Debug function
(defn dbg [x]
  (do (println (str "DEBUG: " x))
      x))
```

## Editor Extensions

### Visual Studio Code

Official VSCode extension is available:

- **Repository**: [qi-vscode](https://github.com/sanohiro/qi-vscode)
- **Features**:
  - Syntax highlighting
  - Code snippets
  - Bracket matching

See the [qi-vscode repository](https://github.com/sanohiro/qi-vscode) for installation instructions and details.

## Testing

### Unit Tests (Fast)

```bash
# Normal tests (without Docker)
cargo test

# Test specific modules
cargo test parser
cargo test eval
```

### Integration Tests (Auto Docker)

Integration tests for PostgreSQL, MySQL, and Redis automatically start and clean up Docker containers using testcontainers.

**Prerequisites**: Docker must be installed.

```bash
# Run integration tests (PostgreSQL + MySQL + Redis)
cargo test --features integration-tests

# Individual execution
cargo test --features integration-tests --test integration_postgres
cargo test --features integration-tests --test integration_mysql
cargo test --features integration-tests --test integration_redis
```

**Behavior**:
- Containers automatically start at test startup (ports auto-assigned)
- Containers automatically removed after tests
- Images remain for faster subsequent test runs

## Links

- **GitHub Repository**: [qi-lang](https://github.com/sanohiro/qi-lang)
- **VSCode Extension**: [qi-vscode](https://github.com/sanohiro/qi-vscode)

## Documentation

### Getting Started
- **[Lisp Basics](docs/en/tutorial/00-lisp-basics.md)** 📚 For Lisp beginners - How to read parentheses (5 min)
- **[Tutorial](docs/en/tutorial/01-getting-started.md)** ⭐ For beginners - Getting started with Qi
- **[CLI Reference](docs/en/cli.md)** - How to use the `qi` command
- **[Project Management](docs/en/project.md)** - qi.toml, templates, customization

### Language Reference
- **[Language Specification](docs/en/spec/)** - Complete Qi specification and reference
  - [Pipeline Operators](docs/en/spec/02-flow-pipes.md) - `|>`, `|>?`, `||>`, `~>`
  - [Concurrency & Parallelism](docs/en/spec/03-concurrency.md) - `go`, `chan`
  - [Pattern Matching](docs/en/spec/04-match.md) - `match` expressions
  - [Error Handling](docs/en/spec/08-error-handling.md) - `try`, `defer`
- **[Standard Library](docs/en/spec/10-stdlib-string.md)** - 60+ built-in functions

## License

Dual-licensed under MIT OR Apache-2.0. Choose whichever you prefer.

- [LICENSE-MIT](LICENSE-MIT) - MIT License
- [LICENSE-APACHE](LICENSE-APACHE) - Apache License 2.0

See each license file for details.
