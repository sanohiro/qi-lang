# KVS (Key-Value Store)

**Redis Unified Interface - Caching, Session Management, Queues**

Qi provides unified access to key-value stores. Currently supports Redis with future support planned for Memcached and other backends.

---

## Table of Contents

- [Overview](#overview)
- [Unified Interface Design](#unified-interface-design)
- [kvs/connect - Connection](#kvsconnect---connection)
- [Basic Operations](#basic-operations)
- [Numeric Operations](#numeric-operations)
- [List Operations](#list-operations)
- [Hash Operations](#hash-operations)
- [Set Operations](#set-operations)
- [Batch Operations](#batch-operations)
- [Practical Examples](#practical-examples)
- [Error Handling](#error-handling)
- [Performance](#performance)
- [Implementation Details](#implementation-details)

---

## Overview

### Provided Features

**Supported Backends**:
- **Redis**: Caching, session management, queues, Pub/Sub
  - Unified interface (`kvs/*`)
  - Basic operations (get/set/del)
  - Numeric operations (incr/decr)
  - Data structures (lists, hashes, sets)
  - Batch operations (mget/mset)
  - Expiration (expire/ttl)

**Benefits of Unified Interface**:
- Backend auto-detection (URL parsing)
- Backend-agnostic switching (only connection URL changes)
- Simple and consistent API
- Future extensibility (Memcached, in-memory KVS, etc.)

### Feature Flag

```toml
# Cargo.toml
features = ["kvs-redis"]
```

Enabled by default.

### Dependencies

- **redis** (v0.27) - Pure Rust Redis client
- **tokio** - Async runtime
- **dashmap** - Connection pool (thread-safe HashMap)

---

## Unified Interface Design

Like `db/connect` for databases, KVS backends are handled transparently.

```qi
;; Backend auto-detected from URL
(def kvs (kvs/connect "redis://localhost:6379"))

;; Code is backend-agnostic
(kvs/set kvs "key" "value")
(kvs/get kvs "key")

;; To change backend, only update connection URL
;; (def kvs (kvs/connect "memcached://localhost:11211"))  ;; Future support
```

**Design Principles**:
- **Unified Interface**: Only `kvs/*` functions are public
- **Private Specific Functions**: `kvs/redis-*` for internal drivers only
- **Extensibility**: Add specific functions only when unified interface cannot express functionality (Redis Pub/Sub, etc.)

---

## kvs/connect - Connection

**Connects to KVS and returns a connection object.**

```qi
(kvs/connect url)
```

### Arguments

- `url`: String (connection URL)
  - Redis: `"redis://localhost:6379"`
  - Redis (with auth): `"redis://:password@localhost:6379"`
  - Memcached: `"memcached://localhost:11211"` (future support)

### Return Value

- Connection ID (string) - Format: `"KvsConnection:kvs:1"`
- Error case: `{:error "message"}`

### Usage Examples

```qi
;; Redis connection
(def kvs (kvs/connect "redis://localhost:6379"))
;; => "KvsConnection:kvs:1"

;; Redis with authentication
(def kvs (kvs/connect "redis://:my-secret-password@localhost:6379"))

;; Connection error
(def kvs (kvs/connect "invalid-url"))
;; => {:error "Unsupported KVS URL: invalid-url"}
```

### Connection Management

- **Automatic connection pool**: Multiple connections to same URL are shared internally
- **Automatic reconnection**: Automatically retries on connection failure
- **Thread-safe**: Safe concurrent access from multiple threads

---

## Basic Operations

### kvs/set - Set Value

Set a value for a key.

```qi
(kvs/set conn key value)
```

**Arguments**:
- `conn`: Connection ID (returned from `kvs/connect`)
- `key`: Key name (string)
- `value`: Value (string, integer, float, boolean)

**Return Value**:
- `"OK"` (success)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/set kvs "user:1" "Alice")
;; => "OK"

(kvs/set kvs "counter" 42)
;; => "OK"

(kvs/set kvs "ratio" 3.14)
;; => "OK"

(kvs/set kvs "active" true)
;; => "OK"
```

---

### kvs/get - Get Value

Get the value of a key.

```qi
(kvs/get conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)

**Return Value**:
- Value (string) - if key exists
- `nil` - if key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/get kvs "user:1")
;; => "Alice"

(kvs/get kvs "nonexistent")
;; => nil

;; Error handling
(def value (kvs/get kvs "user:1"))
(if (nil? value)
  (println "Key not found")
  (println "Value:" value))
```

---

### kvs/del - Delete Key

Delete a key (function name is `kvs/delete`).

```qi
(kvs/del conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)

**Return Value**:
- Number of keys deleted (integer) - usually `1`, `0` if key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/del kvs "user:1")
;; => 1  ;; Deleted successfully

(kvs/del kvs "nonexistent")
;; => 0  ;; Nothing to delete
```

---

### kvs/exists - Check Existence

Check if a key exists (function name is `kvs/exists?`).

```qi
(kvs/exists conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)

**Return Value**:
- `true` - key exists
- `false` - key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/set kvs "user:1" "Alice")
(kvs/exists kvs "user:1")
;; => true

(kvs/exists kvs "nonexistent")
;; => false

;; Conditional logic
(if (kvs/exists kvs "cache:data")
  (kvs/get kvs "cache:data")
  (do
    (def data (fetch-from-db))
    (kvs/set kvs "cache:data" data)
    data))
```

---

### kvs/keys - Pattern Match

Get list of keys matching a pattern.

```qi
(kvs/keys conn pattern)
```

**Arguments**:
- `conn`: Connection ID
- `pattern`: Pattern string
  - `*` - any string
  - `?` - single character
  - Examples: `"user:*"`, `"cache:2025-*"`, `"session:???"`

**Return Value**:
- Vector of key names
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/set kvs "user:1" "Alice")
(kvs/set kvs "user:2" "Bob")
(kvs/set kvs "user:3" "Charlie")

(kvs/keys kvs "user:*")
;; => ["user:1" "user:2" "user:3"]

(kvs/keys kvs "user:?")
;; => ["user:1" "user:2" "user:3"]

(kvs/keys kvs "cache:*")
;; => []  ;; No matches
```

**Warning**: In production with many keys, `kvs/keys` is a blocking operation that can impact performance. Consider using SCAN commands (future implementation) instead.

---

### kvs/expire - Set Expiration

Set expiration time for a key (in seconds).

```qi
(kvs/expire conn key seconds)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)
- `seconds`: Expiration time (integer, seconds)

**Return Value**:
- `true` - expiration set successfully
- `false` - key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
;; Session with 30-minute expiration
(kvs/set kvs "session:abc123" "user-data")
(kvs/expire kvs "session:abc123" 1800)  ;; 30 minutes = 1800 seconds
;; => true

;; 1-hour expiration
(kvs/expire kvs "cache:data" 3600)
;; => true

;; Non-existent key
(kvs/expire kvs "nonexistent" 60)
;; => false
```

---

### kvs/ttl - Get Remaining Time

Get remaining expiration time for a key (in seconds).

```qi
(kvs/ttl conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)

**Return Value**:
- Remaining seconds (integer) - if expiration is set
- `-1` - key exists but no expiration
- `-2` - key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/set kvs "session:abc" "data")
(kvs/expire kvs "session:abc" 3600)

(kvs/ttl kvs "session:abc")
;; => 3599  ;; About 1 hour remaining

;; No expiration
(kvs/set kvs "permanent" "data")
(kvs/ttl kvs "permanent")
;; => -1

;; Non-existent key
(kvs/ttl kvs "nonexistent")
;; => -2
```

---

## Numeric Operations

### kvs/incr - Increment

Increment a key's value by 1. Starts from 0 if key doesn't exist.

```qi
(kvs/incr conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)

**Return Value**:
- Value after increment (integer)
- `{:error "message"}` (failure)

**Examples**:
```qi
;; First time (starts from 0)
(kvs/incr kvs "page-views")
;; => 1

(kvs/incr kvs "page-views")
;; => 2

(kvs/incr kvs "page-views")
;; => 3

;; Increment from existing value
(kvs/set kvs "counter" 100)
(kvs/incr kvs "counter")
;; => 101
```

**Use Cases**: Page view counters, user ID generators, rate limiting, etc.

---

### kvs/decr - Decrement

Decrement a key's value by 1. Starts from 0 if key doesn't exist.

```qi
(kvs/decr conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Key name (string)

**Return Value**:
- Value after decrement (integer)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/set kvs "stock" 10)
(kvs/decr kvs "stock")
;; => 9

(kvs/decr kvs "stock")
;; => 8

;; First time (starts from 0)
(kvs/decr kvs "countdown")
;; => -1
```

**Use Cases**: Inventory management, countdown timers, etc.

---

## List Operations

Redis lists act as double-ended queues (deque). They can be used for both FIFO (queue) and LIFO (stack) operations.

### kvs/lpush - Add to Left

Add element to the left (head) of a list.

```qi
(kvs/lpush conn key value)
```

**Arguments**:
- `conn`: Connection ID
- `key`: List key name (string)
- `value`: Value to add (string, integer, float, boolean)

**Return Value**:
- List length (integer)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/lpush kvs "mylist" "first")
;; => 1

(kvs/lpush kvs "mylist" "second")
;; => 2

;; Result: ["second" "first"]
```

---

### kvs/rpush - Add to Right

Add element to the right (tail) of a list.

```qi
(kvs/rpush conn key value)
```

**Arguments**:
- `conn`: Connection ID
- `key`: List key name (string)
- `value`: Value to add (string, integer, float, boolean)

**Return Value**:
- List length (integer)
- `{:error "message"}` (failure)

**Example (Queue - FIFO)**:
```qi
;; Add tasks
(kvs/rpush kvs "tasks" "task1")
;; => 1
(kvs/rpush kvs "tasks" "task2")
;; => 2
(kvs/rpush kvs "tasks" "task3")
;; => 3

;; Get tasks (from head)
(kvs/lpop kvs "tasks")  ;; => "task1"
(kvs/lpop kvs "tasks")  ;; => "task2"
(kvs/lpop kvs "tasks")  ;; => "task3"
```

**Example (Stack - LIFO)**:
```qi
;; Push elements
(kvs/lpush kvs "stack" "item1")
(kvs/lpush kvs "stack" "item2")
(kvs/lpush kvs "stack" "item3")

;; Pop elements (most recent first)
(kvs/lpop kvs "stack")  ;; => "item3"
(kvs/lpop kvs "stack")  ;; => "item2"
(kvs/lpop kvs "stack")  ;; => "item1"
```

---

### kvs/lpop - Get from Left

Get and remove element from the left (head) of a list.

```qi
(kvs/lpop conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: List key name (string)

**Return Value**:
- Retrieved value (string)
- `nil` - list is empty or key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/rpush kvs "queue" "task1")
(kvs/rpush kvs "queue" "task2")

(kvs/lpop kvs "queue")
;; => "task1"

(kvs/lpop kvs "queue")
;; => "task2"

(kvs/lpop kvs "queue")
;; => nil  ;; Empty
```

---

### kvs/rpop - Get from Right

Get and remove element from the right (tail) of a list.

```qi
(kvs/rpop conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: List key name (string)

**Return Value**:
- Retrieved value (string)
- `nil` - list is empty or key doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/lpush kvs "stack" "a")
(kvs/lpush kvs "stack" "b")
(kvs/lpush kvs "stack" "c")

(kvs/rpop kvs "stack")
;; => "a"  ;; First element added
```

---

### kvs/lrange - Get Range

Get elements in specified range from a list.

```qi
(kvs/lrange conn key start stop)
```

**Arguments**:
- `conn`: Connection ID
- `key`: List key name (string)
- `start`: Start index (integer, 0-based)
- `stop`: End index (integer, -1 for end)

**Return Value**:
- Vector of elements
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/rpush kvs "mylist" "a")
(kvs/rpush kvs "mylist" "b")
(kvs/rpush kvs "mylist" "c")
(kvs/rpush kvs "mylist" "d")
(kvs/rpush kvs "mylist" "e")

;; Get all elements
(kvs/lrange kvs "mylist" 0 -1)
;; => ["a" "b" "c" "d" "e"]

;; First 3 elements
(kvs/lrange kvs "mylist" 0 2)
;; => ["a" "b" "c"]

;; Elements 2 to 4
(kvs/lrange kvs "mylist" 1 3)
;; => ["b" "c" "d"]

;; Last 2 elements
(kvs/lrange kvs "mylist" -2 -1)
;; => ["d" "e"]
```

**Use Cases**: Pagination, history display, latest N items, etc.

---

## Hash Operations

Redis hashes are maps with field-value pairs. Suitable for storing structured data like user information, configuration values, etc.

### kvs/hset - Set Field

Set a field value in a hash.

```qi
(kvs/hset conn key field value)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Hash key name (string)
- `field`: Field name (string)
- `value`: Value (string, integer, float, boolean)

**Return Value**:
- `true` - created new field
- `false` - updated existing field
- `{:error "message"}` (failure)

**Examples**:
```qi
;; Store user information
(kvs/hset kvs "user:1" "name" "Alice")
;; => true  ;; New field

(kvs/hset kvs "user:1" "email" "alice@example.com")
;; => true

(kvs/hset kvs "user:1" "age" 30)
;; => true

;; Update existing field
(kvs/hset kvs "user:1" "name" "Alice Smith")
;; => false  ;; Updated
```

---

### kvs/hget - Get Field

Get a field value from a hash.

```qi
(kvs/hget conn key field)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Hash key name (string)
- `field`: Field name (string)

**Return Value**:
- Value (string)
- `nil` - field doesn't exist
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/hset kvs "user:1" "name" "Alice")
(kvs/hset kvs "user:1" "email" "alice@example.com")

(kvs/hget kvs "user:1" "name")
;; => "Alice"

(kvs/hget kvs "user:1" "email")
;; => "alice@example.com"

(kvs/hget kvs "user:1" "nonexistent")
;; => nil
```

---

### kvs/hgetall - Get All Hash

Get all fields and values from a hash.

```qi
(kvs/hgetall conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Hash key name (string)

**Return Value**:
- Map (field-value pairs)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/hset kvs "user:1" "name" "Alice")
(kvs/hset kvs "user:1" "email" "alice@example.com")
(kvs/hset kvs "user:1" "age" "30")

(kvs/hgetall kvs "user:1")
;; => {"name" "Alice" "email" "alice@example.com" "age" "30"}

;; Map operations
(def user (kvs/hgetall kvs "user:1"))
(get user "name")
;; => "Alice"
```

**Warning**: For hashes with many fields, this can impact performance. Consider using `kvs/hget` to retrieve only needed fields.

---

## Set Operations

Redis sets are collections of unique strings. Suitable for tags, unique value management, etc.

### kvs/sadd - Add Member

Add a member to a set.

```qi
(kvs/sadd conn key member)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Set key name (string)
- `member`: Member (string, integer, float, boolean)

**Return Value**:
- Number of members added (integer) - usually `1`, `0` if already exists
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/sadd kvs "tags" "redis")
;; => 1  ;; New member

(kvs/sadd kvs "tags" "cache")
;; => 1

(kvs/sadd kvs "tags" "nosql")
;; => 1

;; Existing member
(kvs/sadd kvs "tags" "redis")
;; => 0  ;; Not added (duplicate)
```

---

### kvs/smembers - Get All Members

Get all members from a set.

```qi
(kvs/smembers conn key)
```

**Arguments**:
- `conn`: Connection ID
- `key`: Set key name (string)

**Return Value**:
- Vector of members (order is undefined)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/sadd kvs "tags" "redis")
(kvs/sadd kvs "tags" "cache")
(kvs/sadd kvs "tags" "nosql")

(kvs/smembers kvs "tags")
;; => ["redis" "cache" "nosql"]  ;; Order is undefined

;; Empty set
(kvs/smembers kvs "nonexistent")
;; => []
```

**Use Cases**: Tag lists, user permission lists, unique visitors, etc.

---

## Batch Operations

### kvs/mget - Get Multiple Keys

Get values of multiple keys at once.

```qi
(kvs/mget conn keys)
```

**Arguments**:
- `conn`: Connection ID
- `keys`: Vector of key names

**Return Value**:
- Vector of values (nil for non-existent keys)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/set kvs "key1" "value1")
(kvs/set kvs "key2" "value2")
(kvs/set kvs "key3" "value3")

(kvs/mget kvs ["key1" "key2" "key3"])
;; => ["value1" "value2" "value3"]

(kvs/mget kvs ["key1" "nonexistent" "key3"])
;; => ["value1" nil "value3"]

;; Empty vector
(kvs/mget kvs [])
;; => []
```

**Performance Benefits**:
- Get multiple keys in one network round trip
- Faster than loops

```qi
;; ❌ Inefficient (3 network round trips)
(def v1 (kvs/get kvs "key1"))
(def v2 (kvs/get kvs "key2"))
(def v3 (kvs/get kvs "key3"))

;; ✅ Efficient (1 network round trip)
(def [v1 v2 v3] (kvs/mget kvs ["key1" "key2" "key3"]))
```

---

### kvs/mset - Set Multiple Keys

Set multiple key-value pairs at once.

```qi
(kvs/mset conn pairs)
```

**Arguments**:
- `conn`: Connection ID
- `pairs`: Map of keys and values

**Return Value**:
- `"OK"` (success)
- `{:error "message"}` (failure)

**Examples**:
```qi
(kvs/mset kvs {"key1" "value1" "key2" "value2" "key3" "value3"})
;; => "OK"

;; Set many keys at once
(kvs/mset kvs {
  "user:1" "Alice"
  "user:2" "Bob"
  "user:3" "Charlie"
  "cache:1" "data1"
  "cache:2" "data2"
})
;; => "OK"
```

**Performance Benefits**:
- Set multiple keys in one network round trip
- Faster than loops
- Executed atomically (all succeed or all fail)

```qi
;; ❌ Inefficient (3 network round trips)
(kvs/set kvs "key1" "value1")
(kvs/set kvs "key2" "value2")
(kvs/set kvs "key3" "value3")

;; ✅ Efficient (1 network round trip)
(kvs/mset kvs {"key1" "value1" "key2" "value2" "key3" "value3"})
```

---

## Practical Examples

### Session Cache

Use KVS for web application session management.

```qi
(def kvs (kvs/connect "redis://localhost:6379"))

;; Create session
(defn create-session [user-id]
  (def session-id (str "session:" user-id))
  (def session-data (json/stringify {
    :user_id user-id
    :created_at (now)
    :ip_address "192.168.1.1"
  }))
  (kvs/set kvs session-id session-data)
  (kvs/expire kvs session-id 1800)  ;; 30 minute expiration
  session-id)

;; Get session
(defn get-session [session-id]
  (kvs/get kvs session-id)
  |> (fn [data]
       (if (nil? data)
         {:error "Session not found or expired"}
         (json/parse data))))

;; Delete session (logout)
(defn destroy-session [session-id]
  (kvs/del kvs session-id))

;; Usage example
(def sid (create-session 42))
;; => "session:42"

(get-session sid)
;; => {:user_id 42 :created_at "2025-01-22T10:30:00Z" :ip_address "192.168.1.1"}

;; After 30 minutes
(get-session sid)
;; => {:error "Session not found or expired"}
```

---

### Page View Counter

Track web page access counts.

```qi
;; Track page view
(defn track-page-view [page-url]
  (kvs/incr kvs (str "page-views:" page-url)))

;; Get page views
(defn get-page-views [page-url]
  (def count-str (kvs/get kvs (str "page-views:" page-url)))
  (if (nil? count-str)
    0
    (parse-int count-str)))

;; Popular pages ranking
(defn get-popular-pages []
  (def keys (kvs/keys kvs "page-views:*"))
  (def counts (kvs/mget kvs keys))
  (map (fn [key count]
         {:url (str/replace key "page-views:" "")
          :views (parse-int count)})
       keys counts)
  |> (sort-by (fn [item] (get item :views)))
  |> reverse
  |> (take 10))

;; Usage example
(track-page-view "/home")     ;; => 1
(track-page-view "/home")     ;; => 2
(track-page-view "/about")    ;; => 1
(track-page-view "/home")     ;; => 3

(get-page-views "/home")      ;; => 3
(get-page-views "/about")     ;; => 1
(get-page-views "/contact")   ;; => 0

(get-popular-pages)
;; => [{:url "/home" :views 3} {:url "/about" :views 1}]
```

---

### Task Queue (Job Queue)

Manage background tasks with a queue.

```qi
;; Add task
(defn enqueue-task [task-type task-data]
  (def task (json/stringify {
    :type task-type
    :data task-data
    :created_at (now)
  }))
  (kvs/rpush kvs "task-queue" task))

;; Get task
(defn dequeue-task []
  (kvs/lpop kvs "task-queue")
  |> (fn [data]
       (if (nil? data)
         nil
         (json/parse data))))

;; Task worker
(defn process-tasks []
  (loop []
    (def task (dequeue-task))
    (if (nil? task)
      (do
        (println "No tasks, waiting...")
        (sleep 1000)  ;; Wait 1 second
        (recur))
      (do
        (println "Processing task:" task)
        (match (get task :type)
          "send-email" -> (send-email (get task :data))
          "generate-report" -> (generate-report (get task :data))
          _ -> (println "Unknown task type"))
        (recur)))))

;; Usage example
(enqueue-task "send-email" {:to "user@example.com" :subject "Hello"})
(enqueue-task "generate-report" {:user_id 42})

(dequeue-task)
;; => {:type "send-email" :data {:to "user@example.com" :subject "Hello"} :created_at "..."}

(dequeue-task)
;; => {:type "generate-report" :data {:user_id 42} :created_at "..."}

(dequeue-task)
;; => nil  ;; Queue is empty
```

---

### Rate Limiting

Implement API rate limiting.

```qi
;; Rate limit check (max 10 requests per minute)
(defn check-rate-limit [user-id]
  (def key (str "rate-limit:" user-id))
  (def count (kvs/incr kvs key))

  ;; Set 1-minute expiration on first access
  (if (= count 1)
    (kvs/expire kvs key 60))

  ;; Limit if exceeds 10
  (if (> count 10)
    {:allowed false :remaining 0}
    {:allowed true :remaining (- 10 count)}))

;; Usage example
(check-rate-limit 42)
;; => {:allowed true :remaining 9}

;; After 10 requests
(check-rate-limit 42)
;; => {:allowed true :remaining 0}

;; 11th request
(check-rate-limit 42)
;; => {:allowed false :remaining 0}

;; After 1 minute (TTL expires)
(check-rate-limit 42)
;; => {:allowed true :remaining 9}
```

---

### Cache (Database Query Results)

Cache database query results for performance.

```qi
;; Get user with cache
(defn get-user-with-cache [user-id]
  (def cache-key (str "cache:user:" user-id))

  ;; Check cache
  (def cached (kvs/get kvs cache-key))
  (if (not (nil? cached))
    (do
      (println "Cache hit!")
      (json/parse cached))
    (do
      (println "Cache miss, fetching from DB...")
      ;; Fetch from DB
      (def user (db/query db-conn "SELECT * FROM users WHERE id = $1" [user-id])
                 |> first)
      ;; Save to cache (5 minutes)
      (kvs/set kvs cache-key (json/stringify user))
      (kvs/expire kvs cache-key 300)
      user)))

;; Invalidate cache
(defn invalidate-user-cache [user-id]
  (kvs/del kvs (str "cache:user:" user-id)))

;; Usage example
(get-user-with-cache 1)
;; => Cache miss, fetching from DB...
;; => {:id 1 :name "Alice" ...}

(get-user-with-cache 1)
;; => Cache hit!
;; => {:id 1 :name "Alice" ...}

;; Delete cache on user update
(update-user 1 "Alice Smith")
(invalidate-user-cache 1)
```

---

## Error Handling

### Error Handling

KVS functions return raw data on success and `{:error "message"}` on failure.

```qi
;; Basic error handling
(def result (kvs/get kvs "user:1"))
(if (error? result)
  (println "Error:" (get result :error))
  (println "Value:" result))

;; Pipeline error handling (short-circuit with |>?)
(defn get-cached-data [key]
  (kvs/get kvs key)
  |>? (fn [data]
        (if (nil? data)
          {:error "Cache miss"}
          (json/parse data))))

;; match error handling
(match (kvs/get kvs "user:1")
  {:error e} -> (println "KVS error:" e)
  nil -> (println "Key not found")
  value -> (println "Value:" value))
```

---

### Connection Errors

```qi
;; Invalid connection string
(def kvs (kvs/connect "invalid-url"))
;; => {:error "Unsupported KVS URL: invalid-url"}

;; Connection timeout
(def kvs (kvs/connect "redis://localhost:9999"))
;; => {:error "Connection error: ..."}
```

---

### Operation Errors

```qi
;; Type error (non-string key)
(kvs/get kvs 123)
;; => Error: kvs/get (key) expects strings

;; Invalid argument count
(kvs/set kvs "key")
;; => Error: kvs/set expects 3 arguments
```

---

## Performance

### Connection Pool

Qi's kvs implementation automatically pools connections. Multiple `kvs/connect` calls to the same URL share the same connection internally.

```qi
;; These share the same connection internally
(def kvs1 (kvs/connect "redis://localhost:6379"))
(def kvs2 (kvs/connect "redis://localhost:6379"))
```

---

### Automatic Reconnection

Automatically retries reconnection when connection is lost.

```rust
// Rust implementation (reference)
async fn execute_with_retry<T, F, Fut>(url: &str, operation: F) -> redis::RedisResult<T>
where
    F: Fn(MultiplexedConnection) -> Fut,
    Fut: std::future::Future<Output = redis::RedisResult<T>>,
{
    // First attempt
    let conn = get_or_create_connection(url).await?;
    let result = operation(conn).await;

    // On error, reconnect and retry
    if let Err(ref e) = result {
        if is_connection_error(e) {
            if let Ok(new_conn) = reconnect(url).await {
                return operation(new_conn).await;
            }
        }
    }

    result
}
```

---

### Batch Operations Recommended

For multiple keys, use `mget`/`mset`.

```qi
;; ❌ Inefficient (N network round trips)
(def keys ["key1" "key2" "key3" "key4" "key5"])
(map (fn [k] (kvs/get kvs k)) keys)

;; ✅ Efficient (1 network round trip)
(kvs/mget kvs ["key1" "key2" "key3" "key4" "key5"])
```

---

## Implementation Details

### Unified Interface Design

KVS is designed with **unified interface** pattern. This is the same approach as Go's `database/sql` package, allowing transparent backend handling.

```
Unified Interface (user-facing):
- kvs/connect, kvs/get, kvs/set, kvs/del, ...

Internal Drivers (not public):
- RedisDriver, MemcachedDriver (future)
```

**Design Principles**:
- **Backend auto-detection**: Determine connection from URL (`redis://`, `memcached://`, etc.)
- **Private specific functions**: `kvs/redis-*` for internal drivers only
- **Extensibility**: Add specific functions only when unified interface cannot express functionality (Redis Pub/Sub, Lua scripting, etc.)

---

### Driver Pattern

```rust
// KVS driver trait (unified interface)
pub trait KvsDriver: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn set(&self, key: &str, value: &str) -> Result<String, String>;
    fn delete(&self, key: &str) -> Result<i64, String>;
    fn exists(&self, key: &str) -> Result<bool, String>;
    fn keys(&self, pattern: &str) -> Result<Vec<String>, String>;
    fn expire(&self, key: &str, seconds: i64) -> Result<bool, String>;
    fn ttl(&self, key: &str) -> Result<i64, String>;
    // ... other methods
}

// Redis driver (internal implementation)
#[cfg(feature = "kvs-redis")]
struct RedisDriver {
    url: String,
}

impl KvsDriver for RedisDriver {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        crate::builtins::redis::native_redis_get(&[
            Value::String(self.url.clone()),
            Value::String(key.to_string()),
        ])
        // ... error conversion
    }
    // ... other method implementations
}
```

---

### Async Processing

Internally uses async APIs, but exposed as sync API to Qi users.

```rust
// Rust implementation (reference)
use tokio::runtime::Runtime;

static TOKIO_RT: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create tokio runtime for Redis"));

pub fn native_redis_get(args: &[Value]) -> Result<Value, String> {
    // ... argument checking

    // Execute async code synchronously
    TOKIO_RT.block_on(async {
        let result = execute_with_retry(url, |mut conn| async move {
            conn.get(key).await
        }).await;

        match result {
            Ok(Some(value)) => Ok(Value::String(value)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Ok(Value::error(format!("Get error: {}", e))),
        }
    })
}
```

---

### Connection Management

```rust
use dashmap::DashMap;
use redis::aio::MultiplexedConnection;
use std::sync::LazyLock;

// Redis connection pool (URL → Connection mapping)
static REDIS_POOL: LazyLock<DashMap<String, MultiplexedConnection>> =
    LazyLock::new(DashMap::new);

// Get or create connection
async fn get_or_create_connection(url: &str) -> Result<MultiplexedConnection, String> {
    // Get existing connection
    if let Some(conn) = REDIS_POOL.get(url) {
        return Ok(conn.clone());
    }

    // Create new connection
    let client = Client::open(url)?;
    let conn = client.get_multiplexed_async_connection().await?;

    // Save to pool
    REDIS_POOL.insert(url.to_string(), conn.clone());

    Ok(conn)
}
```

---

## Roadmap

### Future Features

**Backend Extensions**:
- **Memcached support**: `kvs/connect "memcached://localhost:11211"`
- **In-memory KVS**: `kvs/connect ":memory:"` (Pure Rust, no dependencies, for testing)

**Specific Functions** (only when unified interface cannot express):
- **Redis Pub/Sub**: `kvs/subscribe`, `kvs/publish`
- **Redis Lua scripting**: `kvs/eval`, `kvs/evalsha`
- **Redis Sorted Sets**: `kvs/zadd`, `kvs/zrange`
- **Redis Streams**: `kvs/xadd`, `kvs/xread`
- **Redis HyperLogLog**: `kvs/pfadd`, `kvs/pfcount`

**Performance Optimizations**:
- **Pipelining**: Execute multiple commands in one network round trip
- **Transactions**: `kvs/multi`, `kvs/exec`
- **Streaming SCAN**: `kvs/scan` (for large key sets)

---

## Summary

Qi's key-value store library provides simple and safe access through unified interface.

### Main Features

- **kvs/connect**: Redis auto-connect (Memcached, etc. in future)
- **Basic operations**: get/set/del/exists/keys/expire/ttl
- **Numeric operations**: incr/decr
- **Lists**: lpush/rpush/lpop/rpop/lrange (queues, stacks)
- **Hashes**: hset/hget/hgetall (structured data)
- **Sets**: sadd/smembers (unique value collections)
- **Batch operations**: mget/mset (batch processing)
- **Automatic reconnection**: Auto-retry on connection failure
- **Connection pooling**: Efficient connection management
- **Backend-agnostic switching**: Change only connection URL

By combining these features, you can easily implement caching, session management, task queues, rate limiting, etc.

---

## Related Documentation

- **[17-stdlib-database.md](17-stdlib-database.md)** - Database unified interface
- **[11-stdlib-http.md](11-stdlib-http.md)** - Using KVS in web applications
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSON data serialization
- **[08-error-handling.md](08-error-handling.md)** - Error handling patterns
