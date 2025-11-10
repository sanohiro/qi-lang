# Error Handling

**Qi's Error Handling Strategy**

Qi provides three error handling methods for different use cases:

1. **Result Type (value / `{:error ...}`)** - Recoverable errors, Railway Pipeline (also supports `{:ok value}` format for validation)
2. **try/catch** - Exception catching and recovery
3. **defer** - Guaranteed resource cleanup (`finally` alternative)

---

## 1. Result Type - Railway Pipeline (Recommended Pattern)

**Use Cases**: API calls, file I/O, parsing, and other operations where failure is expected

### New Specification: Everything Except `{:error}` is Success

**Everything except `{:error}` is success! No `:ok` wrapping**

```qi
;; Simple! Just return regular values
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}  ;; Only errors are explicit
    (/ x y)))                     ;; Regular value → success

;; Process with match (if needed)
(match (divide 10 2)
  {:error e} -> (log e)
  result -> result)

;; Check with error? predicate (simple)
(def result (divide 10 2))
(if (error? result)
  (log "Error occurred")
  (process result))

;; Error handling in pipelines (see 02-flow-pipes.md for details)
(user-input
 |> validate
 |>? parse-number      ;; Just return regular value
 |>? (fn [n] (divide 100 n))
 |>? format-result)
;; On success => result value, On error => {:error ...}
```

### Integration with Railway Pipeline

Using the `|>?` operator integrates error handling into pipelines.

```qi
;; HTTP request + error handling (simple!)
("https://api.example.com/users/123"
 |> http/get                      ;; => "{\"user\": {...}}" (body only)
 |>? json/parse                   ;; => Parse result (value as-is)
 |>? (fn [data] (get data "user")))  ;; Just return the value!
;; On success => user data, On error => {:error ...}

;; JSON parse + data transformation
("{\"name\":\"Alice\",\"age\":30}"
 |> json/parse                    ;; => Parse result (value as-is)
 |>? (fn [data] (get data "name"))  ;; Just return the value!
 |>? str/upper)                   ;; Pass function directly!
;; => "ALICE"
```

### Behavior Rules

**Input processing**:
1. `{:error ...}` → Short-circuit (don't execute next function)
2. Everything else → Pass to next function as-is

**Output processing**:
1. `{:error ...}` → Return as-is (error propagation)
2. Everything else → **Return as-is** (no `:ok` wrapping!)

### Design Philosophy

Treat errors as data and flow them through pipelines. Avoid try-catch nesting, keeping data flow clear. Everything except `{:error}` is treated as success - same philosophy as Lisp's "everything except nil is true," keeping it simple.

---

## 2. try/catch - Exception Handling

**Use Cases**: Catching unexpected errors, calling third-party code

### Basic Usage

```qi
;; try-catch block
(match (try (risky-operation))
  {:error e} -> (handle-error e)
  result -> result)

;; Nesting allowed
(match (try
         (let [data (parse-data input)]
           (process data)))
  {:error e} -> {:error (str "Failed: " e)}
  result -> result)
```

### Practical Examples

```qi
;; File reading error handling
(match (try (io/read-file "config.json"))
  {:error e} -> (do
                  (log/error "Failed to read config:" e)
                  {:error e})
  content -> (json/parse content))

;; Use in pipeline
(match (try
         (url
          |> http-get
          |> parse
          |> process))
  {:error e} -> []
  data -> data)
```

**Note**: Qi doesn't have `finally`. Use `defer` instead (see below).

---

## 3. defer - Guaranteed Resource Cleanup (finally Alternative)

**Use Cases**: Resource management for files, connections, locks, etc.

### Basic Usage

```qi
;; Ensure cleanup with defer
(defn process-file [path]
  (let [f (open-file path)]
    (do
      (defer (close-file f))  ;; Always executed on function exit
      (let [data (io/read-file f)]
        (transform data)))))

;; Practical example: File processing
(defn safe-read [path]
  (let [f (open path)]
    (do
      (defer (close f))  ;; Always executed on function exit
      (read f))))
```

### Execution Order of Multiple defers

Multiple defers execute in stack order (LIFO: Last In, First Out).

```qi
(defn complex-operation []
  (do
    (let [conn (open-connection)]
      (defer (close-connection conn)))
    (let [lock (acquire-lock)]
      (defer (release-lock lock)))
    (let [file (open-file "data.txt")]
      (defer (close-file file)))
    ;; Process...
    ;; Execution order on exit: close-file → release-lock → close-connection
    ))

;; Simple example
(do
  (defer (log "3"))
  (defer (log "2"))
  (defer (log "1"))
  (work))
;; Execution order: work → "1" → "2" → "3"
```

### defer with Errors

Even when errors occur, defer is always executed.

```qi
(defn safe-process []
  (do
    (let [res (allocate-resource)]
      (defer (free-resource res)))
    (if (error-condition?)
      (error "something went wrong")  ;; defer still executes
      (process res))))
```

### Design Philosophy

- Simpler than `finally` - Can be written anywhere in function
- Powerful - Multiple defers can be combined
- Same design as Go's defer
- Lisp-like - No special syntax added

**Why no finally?**: `defer` is more flexible and makes multiple resource management more intuitive. More readable than try-catch-finally nesting.

---

## 4. error - Unrecoverable Errors

**Use Cases**: Fatal errors, precondition violations

### Basic Usage

```qi
;; Throw fatal error with error
(defn critical-init []
  (if (not (io/file-exists? "config.qi"))
    (error "config.qi not found")
    (load-config)))

(defn factorial [n]
  (if (< n 0)
    (error "negative input not allowed")
    (loop [i n acc 1]
      (if (= i 0)
        acc
        (recur (dec i) (* acc i))))))
```

### Catching with try

```qi
;; Catch and process error
(match (try (factorial -5))
  {:error e} -> (log (str "Error: " e))
  result -> result)
```

---

## Choosing Error Handling Methods

### When to Use Result Type

- API calls, HTTP requests
- File I/O, database queries
- JSON/YAML parsing
- User input validation
- **Any operation where failure is expected**

### When to Use try/catch

- Catching unexpected errors
- Calling third-party libraries
- Catching complex processing errors all at once
- **Handling exceptional situations**

---

## Standard Library Return Formats

Qi's built-in functions return different formats based on the nature of the error.

### Functions Returning Result Type (value / `{:error ...}`)

**Data format functions** - Parse errors are expected, requiring explicit handling:

- `json/parse`, `json/stringify`, `json/pretty` - JSON processing
- `yaml/parse`, `yaml/stringify` - YAML processing
- `csv/parse`, `csv/stringify` - CSV processing (planned)

```qi
;; Explicitly handle parse errors
(match (json/parse user-input)
  {:error msg} -> (show-error-to-user msg)
  data -> (process data))

;; Use in pipeline
(user-input
 |> json/parse
 |>? (fn [data] (get data "name"))
 |>? str/upper)
```

### Functions Throwing Exceptions (`Ok(value)` / `Err(message)`)

**I/O & Network functions** - Failure is treated as exceptional:

- `http/get`, `http/post`, `http/put`, `http/delete` - HTTP operations
- `io/read-file`, `io/write-file` - File I/O
- `io/open`, `io/close` - File operations
- `db/*` - Database operations (planned)

```qi
;; Failure propagates as exception
(def content (io/read-file "config.json"))

;; Catch and process with try
(match (try (http/get "https://api.example.com/data"))
  {:error e} -> (log-error e)
  response -> (process response))
```

### Design Policy

This distinction is based on the following reasoning:

1. **Data format functions return value/`{:error}`**
   - Parse errors are **expected failures** (invalid JSON strings, etc.)
   - Error cases are part of normal flow, like user input validation
   - Handle explicitly with match or pipeline (`|>?`)
   - On success, return value directly (no `:ok` wrapping)

2. **I/O & Network functions throw exceptions**
   - File not found, network errors are **exceptional situations**
   - Keep normal path code concise
   - Catch with try/catch as needed

This policy allows users to naturally distinguish between "expected errors" and "exceptional situations".

### When to Use defer

- Closing files
- Closing database connections
- Releasing locks
- Deleting temporary files
- **Whenever resource cleanup is needed**

### When to Use error

- Missing configuration files
- Precondition violations
- Invalid arguments
- **Fatal errors where program cannot continue**

---

## Practical Examples

### API Client

```qi
(defn fetch-user [user-id]
  (user-id
   |> (str "https://api.example.com/users/" _)
   |> http/get
   |>? json/parse
   |>? validate-user))

;; Usage example
(match (fetch-user "123")
  {:error e} -> (log/error "Failed:" e)
  user -> (process-user user))
```

### File Processing with defer

```qi
(defn process-log-file [path]
  (let [f (io/open path :read)]
    (do
      (defer (io/close f))
      (io/read-lines f
       |> (filter (fn [line] (str/contains? line "ERROR")))
       |> (map parse-log-line)
       |> (take 100)))))
```

### Complex Error Handling

```qi
(defn complex-operation [input]
  (match (try
           (input
            |> validate
            |>? parse-data
            |>? transform
            |>? save-to-db))
    {:error e} -> (do
                    (log/error "Operation failed:" e)
                    (send-alert e)
                    {:failure e})
    result -> {:success result}))
```
