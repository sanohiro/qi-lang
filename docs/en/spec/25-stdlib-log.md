# Standard Library - Logging (log/)

**Structured Logging with Multiple Formats**

All functions belong to the `log/` module.

---

## Overview

The `log/` module provides:

- **4 log levels** - DEBUG, INFO, WARN, ERROR
- **Structured logging** - Add context information with maps
- **Multiple formats** - Text and JSON formats
- **Automatic timestamps** - ISO8601 format (UTC)
- **Level filtering** - Logs below the configured level are not output

All logs are output to standard error (stderr).

---

## Log Levels

### 4 Levels

```qi
;; DEBUG - Debug information (development only)
(log/debug "Processing started")

;; INFO - General information (normal operation logs)
(log/info "Server started")

;; WARN - Warnings (potential issues)
(log/warn "Connection timeout")

;; ERROR - Errors (critical issues)
(log/error "Database connection failed")
```

### Level Priority

```
DEBUG (0) < INFO (1) < WARN (2) < ERROR (3)
```

Logs below the configured level are not output.

**Default**: `INFO` (only INFO and above are output)

---

## Log Output

### log/debug

Output DEBUG level log. Used for debug information during development.

```qi
(log/debug message)
(log/debug message context-map)
```

**Arguments:**
- `message` (string) - Log message
- `context-map` (map, optional) - Context information

**Returns:** nil

**Examples:**

```qi
;; Simple message
(log/debug "Function started")
;; [2025-01-15T10:30:45.123+0000] DEBUG Function started

;; With context
(log/debug "Variable values" {:x 10 :y 20})
;; [2025-01-15T10:30:45.456+0000] DEBUG Variable values | x=10 y=20

;; In a pipeline
(defn process-data [data]
  (log/debug "Data processing started" {:count (len data)})
  (data
   |> (map transform)
   |> (filter valid?)))
```

---

### log/info

Output INFO level log. Used for normal operation logs.

```qi
(log/info message)
(log/info message context-map)
```

**Arguments:**
- `message` (string) - Log message
- `context-map` (map, optional) - Context information

**Returns:** nil

**Examples:**

```qi
;; Server startup
(log/info "HTTP server started" {:port 8080 :host "localhost"})
;; [2025-01-15T10:30:45.789+0000] INFO HTTP server started | port=8080 host=localhost

;; Request processing
(defn handle-request [req]
  (log/info "Request received" {:method (get req :method) :path (get req :path)})
  (process req))

;; Batch processing progress
(doseq [item items]
  (log/info "Processing item" {:id (get item :id) :status "processing"})
  (process-item item))
```

---

### log/warn

Output WARN level log. Used for potential issues.

```qi
(log/warn message)
(log/warn message context-map)
```

**Arguments:**
- `message` (string) - Log message
- `context-map` (map, optional) - Context information

**Returns:** nil

**Examples:**

```qi
;; Timeout warning
(log/warn "Connection timeout" {:timeout-ms 5000 :retry-count 3})
;; [2025-01-15T10:30:46.012+0000] WARN Connection timeout | timeout-ms=5000 retry-count=3

;; Retryable errors
(defn fetch-with-retry [url max-retries]
  (loop [retries 0]
    (try
      (http/get url)
      (fn [err]
        (if (< retries max-retries)
            (do
              (log/warn "Request failed, retrying" {:url url :retry retries :error err})
              (recur (+ retries 1)))
            (log/error "Max retries reached" {:url url :error err}))))))

;; Deprecated feature usage
(defn old-api [data]
  (log/warn "Deprecated API used" {:function "old-api"})
  (process-legacy data))
```

---

### log/error

Output ERROR level log. Used for critical issues.

```qi
(log/error message)
(log/error message context-map)
```

**Arguments:**
- `message` (string) - Log message
- `context-map` (map, optional) - Context information

**Returns:** nil

**Examples:**

```qi
;; Database connection error
(log/error "Database connection failed" {:error "connection refused" :host "localhost"})
;; [2025-01-15T10:30:46.345+0000] ERROR Database connection failed | error=connection refused host=localhost

;; Error handling
(try
  (db/connect db-config)
  (fn [err]
    (log/error "DB connection error" {:config db-config :error (str err)})
    (exit 1)))

;; Validation error
(defn validate-user [user]
  (if (not (get user :email))
      (do
        (log/error "Validation error" {:field "email" :user-id (get user :id)})
        false)
      true))
```

---

## Configuration

### log/set-level

Set the log level. Logs below this level will not be output.

```qi
(log/set-level level-string)
```

**Arguments:**
- `level-string` (string) - Log level (`"debug"`, `"info"`, `"warn"`, `"error"`)

**Returns:** nil

**Examples:**

```qi
;; Development: output all logs
(log/set-level "debug")
(log/debug "This will be shown")
(log/info "This will also be shown")

;; Production: INFO and above only
(log/set-level "info")
(log/debug "This will NOT be shown")
(log/info "This will be shown")

;; Errors only
(log/set-level "error")
(log/warn "This will NOT be shown")
(log/error "This will be shown")

;; Configure from environment variable
(log/set-level (env/get "LOG_LEVEL" "info"))

;; Warning level ("warning" is also accepted)
(log/set-level "warning")
```

---

### log/set-format

Set the log format.

```qi
(log/set-format format-string)
```

**Arguments:**
- `format-string` (string) - Format (`"text"`, `"plain"`, `"json"`)

**Returns:** nil

**Formats:**

#### Text format (default)

```qi
(log/set-format "text")
(log/info "Server started" {:port 8080})
;; [2025-01-15T10:30:45.123+0000] INFO Server started | port=8080
```

Format: `[timestamp] level message | key1=value1 key2=value2`

#### JSON format

```qi
(log/set-format "json")
(log/info "Server started" {:port 8080})
;; {"timestamp":"2025-01-15T10:30:45.123+0000","level":"INFO","message":"Server started","port":"8080"}
```

**Use cases:**
- **text**: Human-readable (development, debugging)
- **json**: For log aggregation tools (Elasticsearch, Splunk, etc.)

**Examples:**

```qi
;; Change format based on environment
(let [env (env/get "ENV" "development")]
  (if (= env "production")
      (log/set-format "json")
      (log/set-format "text")))

;; Structured logging with JSON
(log/set-format "json")
(log/info "Request completed"
  {:request-id "req-123"
   :user-id 456
   :duration-ms 234
   :status 200})
;; {"timestamp":"2025-01-15T10:30:45.567+0000","level":"INFO","message":"Request completed","request-id":"req-123","user-id":"456","duration-ms":"234","status":"200"}
```

---

## Practical Examples

### Application Startup Configuration

```qi
;; Initialize logging at startup
(defn init-logging []
  (let [level (env/get "LOG_LEVEL" "info")
        format (env/get "LOG_FORMAT" "text")]
    (log/set-level level)
    (log/set-format format)
    (log/info "Logging configured" {:level level :format format})))

(init-logging)
```

---

### HTTP Server Logging

```qi
;; Request/response logging
(defn log-request [req]
  (log/info "Request received"
    {:method (get req :method)
     :path (get req :path)
     :user-agent (get-in req [:headers "User-Agent"])}))

(defn log-response [req res duration-ms]
  (log/info "Response sent"
    {:method (get req :method)
     :path (get req :path)
     :status (get res :status)
     :duration-ms duration-ms}))

;; Middleware
(defn logging-middleware [handler]
  (fn [req]
    (let [start (time/now-ms)]
      (log-request req)
      (let [res (handler req)
            duration (- (time/now-ms) start)]
        (log-response req res duration)
        res))))
```

---

### Error Tracking

```qi
;; Record error details in log
(defn process-with-error-tracking [data]
  (try
    (do
      (log/debug "Processing started" {:data-size (len data)})
      (let [result (process data)]
        (log/info "Processing succeeded" {:result-size (len result)})
        result))
    (fn [err]
      (log/error "Processing failed"
        {:error (str err)
         :data-size (len data)
         :stack-trace (get err :stack)})
      (throw err))))
```

---

### Batch Processing Progress

```qi
;; Progress logging for large data processing
(defn process-batch [items]
  (let [total (len items)]
    (log/info "Batch processing started" {:total total})
    (loop [i 0
           processed 0
           failed 0]
      (if (< i total)
          (let [item (nth items i)
                result (try
                         (do
                           (process-item item)
                           :ok)
                         (fn [err]
                           (log/warn "Item processing failed"
                             {:index i :item-id (get item :id) :error (str err)})
                           :error))]
            (when (= (mod i 100) 0)
              (log/info "Progress"
                {:processed i :total total :percent (/ (* i 100) total)}))
            (recur
              (+ i 1)
              (if (= result :ok) (+ processed 1) processed)
              (if (= result :error) (+ failed 1) failed)))
          (log/info "Batch processing completed"
            {:total total :processed processed :failed failed})))))
```

---

### Debug Logging

```qi
;; Debugging during development
(defn complex-calculation [x y z]
  (log/debug "Calculation started" {:x x :y y :z z})

  (let [step1 (+ x y)]
    (log/debug "Step 1 completed" {:step1 step1})

    (let [step2 (* step1 z)]
      (log/debug "Step 2 completed" {:step2 step2})

      (let [result (/ step2 2)]
        (log/debug "Calculation completed" {:result result})
        result))))

;; In production, DEBUG logs are not output
(log/set-level "info")
(complex-calculation 10 20 5)  ;; Debug logs won't be shown
```

---

### Structured Logging

```qi
;; Unified log structure with JSON format
(log/set-format "json")

;; Track user actions
(defn track-user-action [user-id action details]
  (log/info "User action"
    {:user-id user-id
     :action action
     :timestamp (time/now-iso)
     :details (json/stringify details)}))

(track-user-action 123 "login" {:ip "192.168.1.1" :device "mobile"})
;; {"timestamp":"2025-01-15T10:30:45.890+0000","level":"INFO","message":"User action","user-id":"123","action":"login","details":"{\"ip\":\"192.168.1.1\",\"device\":\"mobile\"}"}

;; Easy to search and analyze with log aggregation tools (Elasticsearch, etc.)
```

---

### Conditional Logging

```qi
;; Log only under certain conditions
(defn process-with-conditional-log [data]
  (when (> (len data) 1000)
    (log/warn "Processing large data" {:size (len data)}))

  (let [result (process data)]
    (when (empty? result)
      (log/warn "Processing result is empty" {:input-size (len data)}))
    result))

;; Detailed logs only in debug mode
(def debug-mode (= (env/get "DEBUG") "true"))

(defn debug-log [msg ctx]
  (when debug-mode
    (log/debug msg ctx)))

(debug-log "Detailed info" {:var1 x :var2 y})
```

---

## Log Level Best Practices

### DEBUG Level

**Use case**: Debug information during development

```qi
;; Variable values
(log/debug "Variable values" {:x x :y y})

;; Function entry/exit
(log/debug "Function started" {:function "process-data" :args args})
(log/debug "Function completed" {:function "process-data" :result result})

;; Internal state
(log/debug "Loop state" {:iteration i :current-value val})
```

---

### INFO Level

**Use case**: Normal operation logs, important events

```qi
;; Server start/stop
(log/info "Server started" {:port 8080})
(log/info "Server stopped")

;; Request processing
(log/info "Request completed" {:path "/api/users" :status 200})

;; Batch processing start/completion
(log/info "Batch processing started" {:job-id "batch-001"})
(log/info "Batch processing completed" {:job-id "batch-001" :processed 1000})
```

---

### WARN Level

**Use case**: Potential issues, deprecated usage

```qi
;; Retryable errors
(log/warn "Connection failed, retrying" {:retry-count 2 :max-retries 5})

;; Performance issues
(log/warn "Long processing time" {:duration-ms 5000 :threshold-ms 1000})

;; Deprecated features
(log/warn "Deprecated API usage" {:api "old-api" :alternative "new-api"})

;; Data issues
(log/warn "Skipping invalid data" {:line 123 :reason "invalid format"})
```

---

### ERROR Level

**Use case**: Critical issues, processing failures

```qi
;; Connection errors
(log/error "Database connection failed" {:error err :host db-host})

;; Validation errors
(log/error "Data validation failed" {:field "email" :value invalid-email})

;; System errors
(log/error "File write failed" {:path file-path :error err})

;; Unexpected errors
(log/error "Unexpected error" {:error err :context ctx :stack-trace stack})
```

---

## Error Handling

### Logging on Errors

```qi
;; Combined with try/catch
(try
  (risky-operation)
  (fn [err]
    (log/error "Operation failed" {:operation "risky-operation" :error (str err)})
    (default-value)))

;; Errors in pipelines
(data
 |> (map (fn [x]
           (try
             (process x)
             (fn [err]
               (log/warn "Skipping item" {:item x :error (str err)})
               nil))))
 |> (filter some?))
```

---

## Performance Considerations

### Filtering by Log Level

```qi
;; ❌ Bad: always construct string
(log/debug (str "Large data: " (json/stringify large-data)))
;; String construction cost even when DEBUG level is not active

;; ✅ Good: construct only when needed
(when (>= (log/current-level) :debug)  ;; hypothetical function
  (log/debug "Large data" {:data large-data}))
```

---

### Production Environment Settings

```qi
;; Production environment
(log/set-level "info")    ;; Don't output DEBUG logs
(log/set-format "json")   ;; Structured logs for aggregation/analysis

;; Development environment
(log/set-level "debug")   ;; Output all logs
(log/set-format "text")   ;; Human-readable format
```

---

## Function Reference

| Function | Description | Use Case |
|----------|-------------|----------|
| `log/debug` | Output DEBUG level log | Debug information |
| `log/info` | Output INFO level log | Normal operation logs |
| `log/warn` | Output WARN level log | Warnings |
| `log/error` | Output ERROR level log | Errors |
| `log/set-level` | Set log level | Control filtering |
| `log/set-format` | Set log format | Control output format |

---

## Log Format Details

### Text Format

```
[2025-01-15T10:30:45.123+0000] INFO Server started | port=8080 host=localhost
```

**Components:**
- `[...]` - Timestamp (ISO8601 format, UTC)
- `INFO` - Log level
- `Server started` - Message
- `| key=value ...` - Context (optional)

---

### JSON Format

```json
{"timestamp":"2025-01-15T10:30:45.123+0000","level":"INFO","message":"Server started","port":"8080","host":"localhost"}
```

**Fields:**
- `timestamp` - Timestamp (ISO8601 format, UTC)
- `level` - Log level
- `message` - Message
- Others - Keys and values from context map

---

## See Also

- [Error Handling](08-error-handling.md) - Error handling with try/catch
- [Environment Variables](23-stdlib-env.md) - Configuration management
- [HTTP Server](11-stdlib-http.md) - Request logging
- [Debugging](20-stdlib-debug.md) - Debug features
