# Pipeline Extensions - Flow DSL

**A language for designing flow**

Qi extends pipeline operators to **intuitively express data flow**.

> **Implementation**: `src/builtins/flow.rs`, `src/builtins/util.rs`, `src/builtins/stream.rs`

---

## Pipeline Operator System

| Operator | Meaning | Use Case |
|----------|---------|----------|
| `|>` | Sequential pipe | Basic data transformation |
| `\|>?` | Railway pipe | Error handling, Result chaining |
| `||>` | Parallel pipe | Auto-pmap, list parallelization |
| `tap>` | Side-effect tap | Debugging, logging, monitoring |
| `~>` | Async pipe | go/chan integration, async I/O |

---

## `|>` Basic Pipeline

**Flow data left-to-right**

```qi
;; Basic
(data |> parse |> transform |> save)

;; Avoid nesting
(data
 |> (filter valid?)
 |> (map transform)
 |> (reduce merge {}))

;; Functions with arguments
(10 |> (+ 5) |> (* 2))  ;; (+ 10 5) |> (* 2) => 30

;; _ placeholder: Insert value at any position
(42 |> (+ 10 _ 3))  ;; (+ 10 42 3) => 55
("world" |> (str "Hello, " _))  ;; (str "Hello, " "world") => "Hello, world"

;; Real-world example: URL construction
(def params [["user" "john"] ["id" "123"]])
(def base-url "https://api.example.com")
(params
 |> (map (fn [[k v]] f"{k}={v}"))
 |> (join "&" _)
 |> (str base-url "?" _))
```

---

## `||>` Parallel Pipeline

**Auto-expands to pmap**

```qi
;; Parallel processing
(urls ||> http/get ||> json/parse)
;; ↓ Expands to
(urls |> (pmap http/get) |> (pmap json/parse))

;; Basic usage
([1 2 3 4 5] ||> inc)  ;; (2 3 4 5 6)

;; CPU-intensive processing
(images ||> resize ||> compress ||> save)

;; Data analysis
(files
 ||> load-csv
 ||> analyze
 |> merge-results)  ;; Final merge is sequential

;; Complex pipeline
(data
 ||> (fn [x] (* x 2))
 |> (filter (fn [n] (> n 50)))
 |> sum)
```

**Implementation**:
- lexer: Recognizes `||>` as `Token::ParallelPipe`
- parser: `x ||> f` → expands to `(pmap f x)`

### Performance Guidelines ⚡

**When to parallelize**:
- CPU-intensive operations (image processing, compression, computation)
- I/O-bound operations (HTTP requests, file reading)
- **Large number of elements** (guideline: 100+ elements)

**When NOT to parallelize**:
- Lightweight operations (< 1ms per element)
- **Small number of elements** (guideline: < 10 elements)
- Memory constraints

```qi
;; ✅ Good: CPU-intensive + large dataset
(large-images ||> resize ||> compress)

;; ❌ Bad: Lightweight operation + small data (parallelization overhead slows it down)
([1 2 3] ||> inc)  ;; Use |> instead

;; 💡 Guideline for choosing
(if (> (len data) 100)
  (data ||> heavy-process)  ;; Parallelize
  (data |> (map heavy-process)))  ;; Sequential
```

---

## `|>?` Railway Pipeline

**Build error handling into the flow** - Railway Oriented Programming

### New Specification: Everything except `:error` is success

**Everything except `{:error}` is treated as success! No `:ok` wrapping**

```qi
;; Simple! Values flow through as-is
(10
 |>? (fn [x] (* x 2))     ;; 20 → passes to next
 |>? (fn [x] (+ x 5)))    ;; 25 → passes to next
;; => 25

;; Only use {:error} explicitly when you want to return an error
(10
 |>? (fn [x] (if (> x 0) (* x 2) {:error "negative"}))
 |>? (fn [x] (+ x 5)))
;; => 25

(-5
 |>? (fn [x] (if (> x 0) (* x 2) {:error "negative"}))
 |>? (fn [x] (+ x 5)))    ;; Not executed (short-circuit)
;; => {:error "negative"}
```

### Behavior Rules

**Input processing**:
1. `{:error ...}` → Short-circuit (don't execute next function)
2. Everything else → Pass to next function as-is

**Output processing**:
1. `{:error ...}` → Return as-is (error propagation)
2. Everything else → **Return as-is** (no `:ok` wrapping!)

### Real-world Examples

```qi
;; HTTP request + data transformation (Simple!)
("https://api.example.com/users/123"
 |> http/get                 ;; => "{\"user\": {...}}" (body only)
 |>? json/parse              ;; => Parse result (value as-is)
 |>? (fn [data] (get data "user")))  ;; Just return the value!
;; => User data (value as-is)

;; Conditional error
(defn validate-age [age]
  (if (>= age 18)
    age                       ;; Plain value → success
    {:error "Must be 18+"}))  ;; Only errors are explicit

(20 |>? validate-age |>? (fn [x] (* x 2)))  ;; => 40
(15 |>? validate-age |>? (fn [x] (* x 2)))  ;; => {:error "Must be 18+"}
```

**Usage distinction**:
- `|>`: Normal data transformation (no errors)
- `|>?`: Error-prone operations (API, file I/O, parsing)

**Design Philosophy**:
Express error handling as part of the flow. Avoid try-catch nesting, keeping data flow clear. Perfect integration with JSON and HTTP for web development. Everything except `{:error}` is success - same philosophy as Lisp's "everything except nil is true," keeping it simple.

---

## `tap>` Side-effect Tap

**Observe without stopping the flow** (Unix `tee` equivalent)

```qi
;; Debugging
(data
 |> clean
 |> (tap print)
 |> analyze
 |> (tap log)
 |> save)

;; Logging
(requests
 |> (tap log-request)
 |> process
 |> (tap log-response))

;; Concise usage
([1 2 3]
 |> (map inc)
 |> (tap print)
 |> sum)
```

**Implementation**:
- Implemented as `tap` function
- Used as `|> (tap f)` in pipelines
- Executes function and returns original value

---

## `~>` Async Pipeline

**Integration with concurrency - goroutine-style async execution**

The `~>` operator automatically executes pipelines in goroutines and returns results via channels.

```qi
;; Basic async pipeline
(def result (data ~> transform ~> process))  ; Returns channel immediately
(go/recv! result)  ; Receive result

;; Multiple async operations
(def r1 (10 ~> inc ~> (fn [x] (* x 2))))
(def r2 (20 ~> (fn [x] (* x 2)) ~> inc))
(println (go/recv! r1) (go/recv! r2))  ; Execute concurrently

;; Also usable in go blocks
(go/run
  (go/send! output-chan (data ~> transform)))
```

---

## `stream` Lazy Evaluation

**Efficient processing of large data - Lazy evaluation and infinite data structures**

Streams are lazy-evaluated data structures that don't compute values until needed.
They enable memory-efficient handling of infinite data structures and large datasets.

### Stream Creation

```qi
;; Create stream from collection
(stream/stream [1 2 3 4 5])

;; Range stream
(stream/range 0 10)  ;; 0 to 9

;; Infinite stream: repeat same value
(stream/repeat 42)  ;; 42, 42, 42, ...

;; Infinite stream: cycle through list
(stream/cycle [1 2 3])  ;; 1, 2, 3, 1, 2, 3, ...

;; Infinite stream: iterate function application
(stream/iterate (fn [x] (* x 2)) 1)  ;; 1, 2, 4, 8, 16, 32, ...
```

### Stream Transformations

```qi
;; map: Apply function to each element
(def s (stream/range 1 6))
(def s2 (stream/map (fn [x] (* x 2)) s))
(stream/realize s2)  ;; (2 4 6 8 10)

;; filter: Only elements matching condition
(def s (stream/range 1 11))
(def s2 (stream/filter (fn [x] (= (% x 2) 0)) s))
(stream/realize s2)  ;; (2 4 6 8 10)

;; take: Get first n elements (make infinite streams finite)
(def s (stream/repeat 42))
(def s2 (stream/take 5 s))
(stream/realize s2)  ;; (42 42 42 42 42)

;; drop: Skip first n elements
(def s (stream/range 0 10))
(def s2 (stream/drop 5 s))
(stream/realize s2)  ;; (5 6 7 8 9)
```

### Stream Execution

```qi
;; realize: Convert stream to list (compute all elements)
(stream/realize (stream/stream [1 2 3]))  ;; (1 2 3)

;; ⚠️ Warning: Realizing infinite streams causes infinite loops
;; (stream/realize (stream/repeat 42))  ;; NG: Never terminates

;; Correct usage: Make finite with take before realizing
(stream/realize (stream/take 5 (stream/repeat 42)))  ;; OK
```

### Pipeline Integration

```qi
;; Works with existing |> pipeline operator
[1 2 3 4 5]
  |> stream/stream
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (> x 10)))
  |> stream/realize
;; (16 25)

;; Infinite stream processing
1
  |> (stream/iterate (fn [x] (* x 2)))
  |> (stream/take 10)
  |> stream/realize
;; (1 2 4 8 16 32 64 128 256 512)

;; Complex transformation chain
(stream/range 1 100)
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (= (% x 3) 0)))
  |> (stream/take 5)
  |> stream/realize
;; (9 36 81 144 225)
```

### Real-world Examples

```qi
;; Infinite stream of primes (concept - requires prime? function)
(comment
  (defn prime? [n]
    (and (> n 1)
         (not (some (fn [i] (= (% n i) 0)) (range 2 (+ (int (math/sqrt n)) 1))))))

  (def primes
    (2
     |> (stream/iterate inc)
     |> (stream/filter prime?)))

  (stream/realize (stream/take 10 primes)))  ;; First 10 primes

;; Fibonacci sequence
(def fib-stream
  (stream/iterate (fn [[a b]] [b (+ a b)]) [0 1]))

(stream/realize (stream/take 10 fib-stream)
  |> (map first))  ;; (0 1 1 2 3 5 8 13 21 34)

;; Data processing pipeline (concept - requires parse and valid? functions)
(comment
  (defn parse [x] (json/parse x))
  (defn valid? [x] (get x "valid"))

  (defn process-data [data]
    (data
     |> stream/stream
     |> (stream/map parse)
     |> (stream/filter valid?)
     |> (stream/take 1000)
     |> stream/realize)))
```

### I/O Streams

**Lazy file and HTTP data loading - Text & binary support**

#### Text Mode (line-based)

```qi
;; stream/file: Lazy line-by-line file reading (concept example)
(comment
  (defn error-line? [line] (str/contains? line "ERROR"))
  (defn parse [line] (str/split line " "))

  (stream/file "large.log")
    |> (stream/filter error-line?)
    |> (stream/map parse)
    |> (stream/take 100)
    |> stream/realize)

;; http/get-stream: Read HTTP response line-by-line
(http/get-stream "https://api.example.com/data")
  |> (stream/take 10)
  |> stream/realize

;; http/post-stream: Streaming POST request reception
(http/post-stream "https://api.example.com/upload" {:data "value"})
  |> (stream/take 10)
  |> stream/realize

;; http/request-stream: Streaming with detailed configuration (concept example)
(comment
  (defn important? [line] (str/contains? line "IMPORTANT"))

  (http/request-stream {
    :method "GET"
    :url "https://api.example.com/stream"
  })
    |> (stream/filter important?)
    |> stream/realize)
```

#### Binary Mode (byte chunks)

```qi
;; stream/file :bytes - Read file in 4KB chunks
(stream/file "image.png" :bytes)
  |> (stream/take 10)
  |> stream/realize
;; => List of Vector of Integers (bytes)

;; http/get-stream :bytes - HTTP binary download
(http/get-stream "https://example.com/file.bin" :bytes)
  |> (stream/map process-chunk)
  |> stream/realize

;; Byte processing example
(def bytes (first (stream/realize (stream/take 1 (stream/file "data.bin" :bytes)))))
(def sum (reduce + bytes))  ; Sum of bytes
(println sum)

;; Image download
(http/get-stream "https://example.com/logo.png" :bytes)
  |> stream/realize
  |> flatten
  |> (write-bytes "logo.png")  ; write-bytes to be implemented
```

**Mode Comparison**:

| Mode | Use Case | Return Value | Example |
|------|----------|--------------|---------|
| Text (default) | Logs, CSV, JSON | String (per line) | `(stream/file "data.txt")` |
| Binary (`:bytes`) | Images, video, binary | Vector of Integers (4KB chunks) | `(stream/file "image.png" :bytes)` |

```qi
;; CSV file processing
(stream/file "data.csv")
  |> (stream/drop 1)  ; Skip header
  |> (stream/map (fn [line] (split line ",")))
  |> (stream/filter (fn [cols] (> (len cols) 2)))
  |> (stream/take 1000)
  |> stream/realize

;; Fetch and parse JSON from HTTP
(http/get-stream "https://jsonplaceholder.typicode.com/todos/1")
  |> stream/realize
  |> (join "\n")
  |> json/parse
```

**Real-world Example: Log File Analysis**

```qi
;; Process large log files memory-efficiently (concept example)
(comment
  (defn parse-log-line [line]
    (let [parts (str/split line " ")]
      {:timestamp (first parts)
       :level (get parts 1)
       :message (join " " (drop 2 parts))}))

  (defn analyze-logs [file]
    (stream/file file
     |> (stream/filter (fn [line] (str/contains? line "ERROR")))
     |> (stream/map parse-log-line)
     |> (stream/take 100)  ; First 100 errors
     |> stream/realize))

  ;; Get results
  (def errors (analyze-logs "/var/log/app.log"))
  (println (str "Found " (len errors) " errors")))
```

---

## Pipeline Culture

**Unix Philosophy × Functional × Lisp**

You can build complex processing by defining small pipes and combining them.

```qi
;; Define small pipes (concept example)
(comment
  (defn remove-punctuation [text]
    (str/replace text "[^a-zA-Z0-9\\s@.]" ""))

  (defn email? [s]
    (str/contains? s "@"))

  (def clean-text
    (fn [text]
      (text |> str/trim |> str/lower |> remove-punctuation)))

  (def extract-emails
    (fn [text]
      (text |> (str/split "\\s+") |> (filter email?))))

  (def dedupe
    (fn [coll]
      (coll |> sort |> distinct)))

  ;; Combine them
  (def document "Contact: john@example.com, JANE@EXAMPLE.COM!! Support: support@example.com")
  (document
   |> clean-text
   |> extract-emails
   |> dedupe
   |> (join ", ")))
```
