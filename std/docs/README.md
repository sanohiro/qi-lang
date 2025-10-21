# Qi Standard Library Documentation

This directory contains comprehensive documentation for all Qi standard library functions.

## Structure

```
std/docs/
├── ja/          # Japanese documentation
│   ├── core.qi
│   ├── string.qi
│   ├── async.qi
│   ├── http.qi
│   ├── io.qi
│   ├── time.qi
│   ├── data.qi
│   ├── math.qi
│   ├── stats.qi
│   └── ...
└── en/          # English documentation
    ├── core.qi
    └── ...
```

## Documentation Format

Each `.qi` file contains documentation in the following format:

```qi
(def __doc__function-name
  {:desc "Function description"
   :params [{:name "param1" :type "type" :desc "Parameter description"}
            {:name "param2" :type "type" :desc "Parameter description"}]
   :returns {:type "return-type" :desc "Return value description"}
   :examples ["(function-name arg1 arg2) ;=> result"
              "(function-name other-args) ;=> other-result"]
   :feature "optional-feature-name"  ; Only if feature-gated
   })
```

## Available Modules

### Core Functions (87 functions)
- **Arithmetic**: `+`, `-`, `*`, `/`, `%`, `abs`, `min`, `max`, `inc`, `dec`, `sum`
- **Comparison**: `=`, `!=`, `<`, `>`, `<=`, `>=`
- **Collections**: `first`, `rest`, `last`, `nth`, `len`, `count`, `cons`, `conj`, `concat`, `flatten`, `range`, `repeat`, `reverse`, `take`, `drop`, `sort`, `distinct`, `zip`
- **Maps**: `get`, `keys`, `vals`, `assoc`, `dissoc`, `merge`, `get-in`
- **Predicates**: `nil?`, `list?`, `vector?`, `map?`, `string?`, `integer?`, `float?`, `number?`, `keyword?`, `function?`, `atom?`, `coll?`, `sequential?`, `empty?`, `some?`, `true?`, `false?`, `error?`, `even?`, `odd?`, `positive?`, `negative?`, `zero?`
- **Higher-Order**: `identity`, `constantly`, `partial`
- **String**: `str`, `split`, `join`
- **I/O**: `print`, `println`, `not`, `error`
- **State**: `atom`, `deref`, `reset!`, `uvar`, `variable`, `macro?`
- **Utility**: `to-int`, `to-float`, `to-string`, `now`, `timestamp`, `sleep`

### String Functions (72 functions)
- **Basic Operations**: `str/split`, `str/upper`, `str/lower`, `str/trim`, `str/contains?`, `str/starts-with?`, `str/ends-with?`, `str/index-of`, `str/last-index-of`, `str/slice`, `str/replace`, `str/repeat`, etc.
- **Predicates**: `str/digit?`, `str/alpha?`, `str/alnum?`, `str/space?`, `str/lower?`, `str/upper?`
- **Regex**: `str/match?`, `str/find`, `str/find-all`, `str/replace-re`
- **Encoding** (requires `string-encoding` feature): `str/to-base64`, `str/from-base64`, `str/url-encode`, `str/url-decode`, `str/html-escape`, `str/html-unescape`
- **Crypto** (requires `string-crypto` feature): `str/sha256`, `str/md5`, `str/uuid`
- **Templates**: `str/template`, `str/format`

### Async/Concurrency Functions (13 functions - `go/*`)
- **Channels**: `go/chan`, `go/send!`, `go/recv!`, `go/try-recv!`, `go/close!`
- **Promises**: `go/await`, `go/all`, `go/race`
- **Patterns**: `go/fan-out`, `go/fan-in`
- **Cancellation**: `go/make-scope`, `go/cancel!`, `go/cancelled?`

### HTTP Client Functions (11 functions)
- **HTTP Methods**: `http/get`, `http/post`, `http/put`, `http/delete`, `http/patch`, `http/head`, `http/options`, `http/request`
- **Async**: `http/get-async`, `http/post-async`
- **Streaming**: `http/get-stream`, `http/post-stream`, `http/request-stream`

### File I/O Functions (19 functions)
- **Read/Write**: `io/read-file`, `io/write-file`, `io/append-file`, `io/read-lines`
- **Directory**: `io/list-dir`, `io/create-dir`, `io/delete-dir`
- **File Operations**: `io/file-exists?`, `io/delete-file`, `io/copy-file`, `io/move-file`, `io/file-info`, `io/is-file?`, `io/is-dir?`
- **Streaming**: `io/file-stream`, `io/write-stream`
- **Stdin**: `io/stdin-line`, `io/stdin-lines`
- **Temp** (requires `temp-files` feature): `io/temp-file`, `io/temp-dir`

### Time Functions (20 functions)
- **Formatting**: `time/now-iso`, `time/from-unix`, `time/to-unix`, `time/format`, `time/parse`, `time/today`
- **Arithmetic**: `time/add-days`, `time/add-hours`, `time/add-minutes`, `time/sub-days`, `time/sub-hours`, `time/sub-minutes`
- **Comparison**: `time/diff-days`, `time/diff-hours`, `time/diff-minutes`, `time/before?`, `time/after?`, `time/between?`
- **Extraction**: `time/year`, `time/month`, `time/day`, `time/hour`, `time/minute`, `time/second`, `time/weekday`

### Data Format Functions (11 functions)
- **JSON**: `json/parse`, `json/stringify`, `json/pretty`
- **YAML**: `yaml/parse`, `yaml/stringify`, `yaml/pretty`
- **CSV**: `csv/parse`, `csv/stringify`, `csv/read-file`, `csv/write-file`, `csv/read-stream`

### Math Functions (10 functions)
- **Basic**: `math/pow`, `math/sqrt`, `math/round`, `math/floor`, `math/ceil`, `math/clamp`
- **Random** (requires `std-math` feature): `math/rand`, `math/rand-int`, `math/random-range`, `math/shuffle`

### Statistics Functions (6 functions)
- `stats/mean`, `stats/median`, `stats/mode`, `stats/variance`, `stats/stddev`, `stats/percentile`

### Other Modules

Additional modules documented (or to be documented):
- **Server**: `server/*` (14 functions) - HTTP server and middleware
- **Path**: `path/*` (8 functions) - Path manipulation
- **Environment**: `env/*` (4 functions) - Environment variables
- **Arguments**: `args/*` (4 functions) - Command-line arguments
- **Command Execution**: `cmd/*` (10 functions) - Process execution
- **Testing**: `test/*` (5 functions) - Unit testing
- **Logging**: `log/*` (6 functions) - Logging utilities
- **Profiling**: `profile/*` (4 functions) - Performance profiling
- **Streams**: `stream/*` (7 functions) - Lazy streams
- **Database**: `db/*` (19 functions) - Database operations
- **Data Structures**: `ds/*` (12 functions) - Queue and Stack
- **Compression**: `zip/*` (6 functions) - ZIP and gzip
- **Markdown**: `markdown/*` (9 functions) - Markdown generation
- **Higher-Order**: `fn/*` (3 functions) - Advanced function utilities
- **List/Map/Set**: Extended collection operations

## Feature Gates

Some functions require specific Cargo features to be enabled:

- **string-encoding**: Base64, URL encoding, HTML escape
- **string-crypto**: SHA256, MD5, UUID
- **std-math**: Random number generation
- **http-client**: HTTP client functions
- **http-server**: HTTP server functions
- **temp-files**: Temporary file/directory creation
- **db-sqlite**: SQLite database support
- **util-zip**: ZIP compression/extraction

## Usage

To view documentation for a specific module, open the corresponding `.qi` file in `ja/` (Japanese) or `en/` (English).

## Contributing

When adding new standard library functions:
1. Update the corresponding `.qi` documentation file
2. Follow the established format
3. Include practical examples
4. Document any feature requirements
5. Update this README

## Total Function Count

- **Core**: 87 functions
- **String**: 72 functions
- **Async**: 13 functions
- **HTTP**: 11 functions
- **I/O**: 19 functions
- **Time**: 20 functions
- **Data**: 11 functions
- **Math**: 10 functions
- **Stats**: 6 functions
- **Others**: ~100+ functions

**Grand Total**: ~350+ standard library functions

## Language Philosophy

Qi's standard library follows these principles:
1. **Flow-Oriented**: Natural data transformation with `|>`, `|>?`, `||>`, `~>`
2. **Concurrency-First**: Built-in support for channels, promises, and parallel execution
3. **Practical**: Real-world utilities (HTTP, JSON, CSV, Database, etc.)
4. **Consistent**: Uniform naming (`namespace/function`) and error handling
5. **Feature-Gated**: Optional dependencies for minimal builds
