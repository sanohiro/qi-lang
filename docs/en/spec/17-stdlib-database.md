# Database & KVS

**Unified Interface for Databases (PostgreSQL/MySQL/SQLite) and KVS (Redis)**

Qi provides unified access to relational databases and key-value stores.

---

## Table of Contents

- [Overview](#overview)
- [KVS (Key-Value Store)](#kvs-key-value-store)
  - [kvs/connect - Connection](#kvsconnect---connection)
  - [Basic Operations](#basic-operations)
  - [Practical Examples](#kvs-practical-examples)
- [Unified Database Interface](#unified-database-interface)
  - [db/connect - Connection](#dbconnect---connection)
  - [db/query - Query Execution](#dbquery---query-execution)
  - [db/exec - Command Execution](#dbexec---command-execution)
- [Practical Examples](#practical-examples)
- [Error Handling](#error-handling)

---

## Overview

### Provided Features

**KVS (Key-Value Store)**:
- **Redis**: Caching, session management, queues
  - Unified interface (`kvs/*`)
  - Basic operations, numeric operations, lists, hashes, sets
  - Backend auto-detection (URL parsing)

**Database**:
- **Unified Interface (db/\*)**: PostgreSQL/MySQL/SQLite support
  - Connection management (`db/connect`)
  - Query execution (`db/query`)
  - Command execution (`db/exec`)
  - Transactions (`db/begin`, `db/commit`, `db/rollback`)
  - Parameterized queries
  - Backend-agnostic switching (only connection URL changes)

### Feature Flags

```toml
# Cargo.toml
features = ["kvs-redis", "db-sqlite", "db-postgres", "db-mysql"]
```

Enabled by default.

### Dependencies

**KVS**:
- **redis** (v0.27) - Pure Rust Redis client
- **tokio** - Async runtime

**Database**:
- **rusqlite** (v0.32) - Pure Rust SQLite client
- **tokio-postgres** (v0.7) - Pure Rust PostgreSQL client
- **mysql_async** (v0.34) - Pure Rust MySQL client
- **tokio** - Async runtime

---

## KVS (Key-Value Store)

### Unified Interface Design

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

---

### kvs/connect - Connection

**Connects to KVS and returns a connection object.**

```qi
(kvs/connect url)
```

#### Arguments

- `url`: String (connection URL)
  - Redis: `"redis://localhost:6379"`
  - Memcached: `"memcached://localhost:11211"` (future support)

#### Return Value

- Connection ID (string)

#### Usage Examples

```qi
;; Redis connection
(def kvs (kvs/connect "redis://localhost:6379"))

;; Redis with authentication
(def kvs (kvs/connect "redis://:password@localhost:6379"))
```

---

### Basic Operations

#### kvs/set - Set Value

```qi
(kvs/set conn key value)
```

**Example**:
```qi
(kvs/set kvs "user:1" "Alice")
;; => "OK"

(kvs/set kvs "counter" 42)
;; => "OK"
```

#### kvs/get - Get Value

```qi
(kvs/get conn key)
```

**Example**:
```qi
(kvs/get kvs "user:1")
;; => "Alice"

(kvs/get kvs "nonexistent")
;; => nil
```

#### kvs/delete - Delete Key

```qi
(kvs/delete conn key)
```

**Example**:
```qi
(kvs/delete kvs "user:1")
;; => 1  ;; Number of keys deleted
```

#### kvs/exists? - Check Existence

```qi
(kvs/exists? conn key)
```

**Example**:
```qi
(kvs/exists? kvs "user:1")
;; => true
```

#### kvs/keys - Pattern Match

```qi
(kvs/keys conn pattern)
```

**Example**:
```qi
(kvs/keys kvs "user:*")
;; => ["user:1" "user:2" "user:3"]
```

#### kvs/expire - Set Expiration

```qi
(kvs/expire conn key seconds)
```

**Example**:
```qi
(kvs/expire kvs "session:abc" 3600)  ;; 1 hour
;; => true
```

#### kvs/ttl - Get Remaining Time

```qi
(kvs/ttl conn key)
```

**Example**:
```qi
(kvs/ttl kvs "session:abc")
;; => 3599  ;; -1: no expiration, -2: doesn't exist
```

---

### Numeric Operations

#### kvs/incr - Increment

```qi
(kvs/incr conn key)
```

**Example**:
```qi
(kvs/set kvs "page-views" 0)
(kvs/incr kvs "page-views")  ;; => 1
(kvs/incr kvs "page-views")  ;; => 2
```

#### kvs/decr - Decrement

```qi
(kvs/decr conn key)
```

---

### List Operations

#### kvs/lpush / kvs/rpush - Add Element

```qi
(kvs/lpush conn key value)  ;; Add to left (head)
(kvs/rpush conn key value)  ;; Add to right (tail)
```

**Example (Queue - FIFO)**:
```qi
(kvs/rpush kvs "tasks" "task1")
(kvs/rpush kvs "tasks" "task2")
(kvs/lpop kvs "tasks")  ;; => "task1"
(kvs/lpop kvs "tasks")  ;; => "task2"
```

**Example (Stack - LIFO)**:
```qi
(kvs/lpush kvs "stack" "item1")
(kvs/lpush kvs "stack" "item2")
(kvs/lpop kvs "stack")  ;; => "item2"
```

#### kvs/lpop / kvs/rpop - Get and Remove Element

```qi
(kvs/lpop conn key)  ;; Get from left
(kvs/rpop conn key)  ;; Get from right
```

#### kvs/lrange - Get Range

```qi
(kvs/lrange conn key start stop)
```

**Arguments**:
- `start`: Start index (0-based)
- `stop`: End index (-1 for end)

**Example**:
```qi
(kvs/rpush kvs "mylist" "a")
(kvs/rpush kvs "mylist" "b")
(kvs/rpush kvs "mylist" "c")

(kvs/lrange kvs "mylist" 0 -1)  ;; => ["a" "b" "c"] ;; All elements
(kvs/lrange kvs "mylist" 0 1)   ;; => ["a" "b"]    ;; First 2 elements
(kvs/lrange kvs "tasks" 0 9)    ;; => [...]        ;; First 10 elements
```

---

### Hash Operations

#### kvs/hset - Set Field

```qi
(kvs/hset conn key field value)
```

**Example**:
```qi
(kvs/hset kvs "user:1" "name" "Alice")
(kvs/hset kvs "user:1" "email" "alice@example.com")
```

#### kvs/hget - Get Field

```qi
(kvs/hget conn key field)
```

**Example**:
```qi
(kvs/hget kvs "user:1" "name")
;; => "Alice"
```

#### kvs/hgetall - Get All Hash

```qi
(kvs/hgetall conn key)
```

**Example**:
```qi
(kvs/hgetall kvs "user:1")
;; => {:name "Alice" :email "alice@example.com"}
```

---

### Set Operations

#### kvs/sadd - Add Member

```qi
(kvs/sadd conn key member)
```

**Example**:
```qi
(kvs/sadd kvs "tags" "redis")
(kvs/sadd kvs "tags" "cache")
```

#### kvs/smembers - Get All Members

```qi
(kvs/smembers conn key)
```

**Example**:
```qi
(kvs/smembers kvs "tags")
;; => ["redis" "cache" "nosql"]
```

---

### Batch Operations

#### kvs/mget - Get Multiple Keys

```qi
(kvs/mget conn keys)
```

**Arguments**:
- `keys`: Vector of key names

**Return Value**:
- Vector of values (nil for non-existent keys)

**Example**:
```qi
(kvs/set kvs "key1" "value1")
(kvs/set kvs "key2" "value2")
(kvs/set kvs "key3" "value3")

(kvs/mget kvs ["key1" "key2" "key3"])
;; => ["value1" "value2" "value3"]

(kvs/mget kvs ["key1" "nonexistent" "key3"])
;; => ["value1" nil "value3"]
```

#### kvs/mset - Set Multiple Keys

```qi
(kvs/mset conn pairs)
```

**Arguments**:
- `pairs`: Map of keys and values

**Return Value**:
- `"OK"` (on success)

**Example**:
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
```

**Performance Benefits**:
- Multiple keys in one network round trip
- Faster than loops
- Executed atomically

```qi
;; ❌ Inefficient (3 network round trips)
(kvs/set kvs "key1" "value1")
(kvs/set kvs "key2" "value2")
(kvs/set kvs "key3" "value3")

;; ✅ Efficient (1 network round trip)
(kvs/mset kvs {"key1" "value1" "key2" "value2" "key3" "value3"})
```

---

### KVS Practical Examples

#### Session Cache

```qi
(def kvs (kvs/connect "redis://localhost:6379"))

(defn create-session [user-id]
  (def session-id (str "session:" user-id))
  (def session-data (json/stringify {:user_id user-id :created_at (now)}))
  (kvs/set kvs session-id session-data)
  (kvs/expire kvs session-id 1800)  ;; 30 minutes
  session-id)

(defn get-session [session-id]
  (kvs/get kvs session-id)
  |> (fn [data]
       (if (nil? data)
         {:error "Session not found"}
         (json/parse data))))

;; Usage example
(def sid (create-session 42))
(get-session sid)
;; => {:user_id 42 :created_at "2025-01-22T..."}
```

#### Counter (Page Views)

```qi
(defn track-page-view [page-url]
  (kvs/incr kvs (str "page-views:" page-url)))

(defn get-page-views [page-url]
  (kvs/get kvs (str "page-views:" page-url)))

;; Usage example
(track-page-view "/home")  ;; => 1
(track-page-view "/home")  ;; => 2
(get-page-views "/home")   ;; => "2"
```

#### Task Queue

```qi
(defn enqueue-task [task-data]
  (kvs/rpush kvs "task-queue" (json/stringify task-data)))

(defn dequeue-task []
  (kvs/lpop kvs "task-queue")
  |> (fn [data]
       (if (nil? data)
         nil
         (json/parse data))))

;; Usage example
(enqueue-task {:type "send-email" :to "user@example.com"})
(dequeue-task)
;; => {:type "send-email" :to "user@example.com"}
```

---

## Unified Database Interface

### db/connect - Connection

**Connects to database and returns connection ID.**

```qi
(db/connect url)
```

Backend auto-detected from URL:
- PostgreSQL: `"postgresql://..."`
- MySQL: `"mysql://..."`
- SQLite: `"sqlite:..."`

**Example**:
```qi
(def db-conn (db/connect "postgresql://admin:secret@localhost:5432/myapp"))
(def db-conn (db/connect "mysql://root:pass@localhost/mydb"))
(def db-conn (db/connect "sqlite:path/to/db.db"))
```

### db/query - Query Execution

**Executes SELECT query and returns rows.**

```qi
(db/query conn sql params)
```

**Example**:
```qi
(db/query db-conn "SELECT * FROM users" [])
;; => [{:id 1 :name "Alice"} {:id 2 :name "Bob"}]

(db/query db-conn "SELECT * FROM users WHERE id = $1" [1])
;; => [{:id 1 :name "Alice"}]
```

### db/exec - Command Execution

**Executes INSERT/UPDATE/DELETE and returns affected rows.**

```qi
(db/exec conn sql params)
```

**Example**:
```qi
(db/exec db-conn "INSERT INTO users (name, email) VALUES ($1, $2)" ["Alice" "alice@example.com"])
;; => 1

(db/exec db-conn "UPDATE users SET name = $1 WHERE id = $2" ["Bob" 1])
;; => 1
```

---

## Practical Examples

### User Management System

```qi
;; Database connection (unified interface)
(def db-conn (db/connect "postgresql://admin:secret@localhost:5432/myapp"))

;; Create user
(defn create-user [name email password-hash]
  (db/exec db-conn
    "INSERT INTO users (name, email, password_hash, created_at)
     VALUES ($1, $2, $3, NOW()) RETURNING id"
    [name email password-hash]))

;; Find user
(defn find-user-by-email [email]
  (db/query db-conn
    "SELECT id, name, email, created_at FROM users WHERE email = $1"
    [email]))

;; Update user
(defn update-user [user-id name email]
  (db/exec db-conn
    "UPDATE users SET name = $1, email = $2, updated_at = NOW()
     WHERE id = $3"
    [name email user-id]))

;; Delete user
(defn delete-user [user-id]
  (db/exec db-conn
    "DELETE FROM users WHERE id = $1"
    [user-id]))

;; Usage example
(def result (create-user "Alice" "alice@example.com" "$argon2id$..."))
;; => 1

(find-user-by-email "alice@example.com")
;; => [{:id 1 :name "Alice" :email "alice@example.com" :created_at "..."}]
```

### Pagination

```qi
;; Get paginated results
(defn get-users-page [page per-page]
  (let [offset (* (- page 1) per-page)]
    (db/query db-conn
      "SELECT id, name, email FROM users
       ORDER BY created_at DESC
       LIMIT $1 OFFSET $2"
      [per-page offset])))

;; Usage example
(get-users-page 1 10)  ;; Page 1, 10 items
;; => [{:id 100 :name "Zara" ...} ...]

(get-users-page 2 10)  ;; Page 2, 10 items
;; => [{:id 90 :name "Yuki" ...} ...]
```

### Transactions (Manual)

```qi
;; Transaction example (manual commit)
(defn transfer-money [from-id to-id amount]
  (let [conn db-conn]
    ;; BEGIN
    (db/exec conn "BEGIN" [])

    ;; Debit
    (def debit-result
      (db/exec conn
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2"
        [amount from-id]))

    ;; Credit
    (def credit-result
      (db/exec conn
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2"
        [amount to-id]))

    ;; Commit or rollback
    (match [debit-result credit-result]
      [1 1] -> (do
                 (db/exec conn "COMMIT" [])
                 "Transfer successful")
      _ -> (do
             (db/exec conn "ROLLBACK" [])
             {:error "Transfer failed"}))))
```

### Aggregation Queries

```qi
;; Count users
(defn count-users []
  (db/query db-conn "SELECT COUNT(*) as count FROM users" [])
  |>? (fn [rows] (get (first rows) :count)))

;; Group by
(defn count-users-by-status []
  (db/query db-conn
    "SELECT status, COUNT(*) as count
     FROM users
     GROUP BY status"
    []))

;; Usage example
(count-users)
;; => 1523

(count-users-by-status)
;; => [{:status "active" :count 1200}
;;     {:status "inactive" :count 323}]
```

---

## Error Handling

### Error Handling

Database functions return raw data on success and `{:error "message"}` on failure.

```qi
;; Basic error handling
(def result (db/query db-conn "SELECT * FROM users" []))
(if (error? result)
  (println "Error:" (get result :error))
  (println "Found" (count result) "users"))

;; Pipeline error handling (short-circuit with |>?)
(defn get-user-email [user-id]
  (db/query db-conn "SELECT email FROM users WHERE id = $1" [user-id])
  |>? (fn [rows]
        (if (empty? rows)
          {:error "User not found"}
          (get (first rows) "email"))))

;; match error handling
(match (db/query db-conn "SELECT * FROM users" [])
  {:error e} -> (println "Database error:" e)
  rows -> (println "Found" (count rows) "users"))
```

### Connection Errors

```qi
;; Invalid connection string
(def conn (db/connect "invalid-url"))
;; => {:error "Unsupported database URL: invalid-url"}

;; Connection timeout
(def conn (db/connect "postgresql://localhost:9999/db"))
;; => {:error "Connection failed: connection refused"}
```

### Query Errors

```qi
;; Syntax error
(db/query db-conn "SELEC * FROM users" [])
;; => {:error "Query error: syntax error at or near \"SELEC\""}

;; Table doesn't exist
(db/query db-conn "SELECT * FROM nonexistent_table" [])
;; => {:error "Query error: relation \"nonexistent_table\" does not exist"}
```

---

## Performance

### Connection Pooling (Not Implemented)

Current implementation establishes a new connection per query.

Future plan to support connection pooling:

```qi
;; Future plan
(def pool (db/create-pool "postgresql://..." {:max-connections 10}))
(db/with-connection pool (fn [conn]
  (db/query conn "SELECT * FROM users" [])))
```

### Parameterized Queries

Always use parameterized queries to prevent SQL injection attacks:

```qi
;; ❌ Dangerous: SQL injection vulnerability
(def user-input "1 OR 1=1")
(db/query db-conn (str "SELECT * FROM users WHERE id = " user-input) [])

;; ✅ Safe: Parameterized query
(db/query db-conn "SELECT * FROM users WHERE id = $1" [user-input])
```

---

## Connection String Format

PostgreSQL connection string format:

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

### Examples

```qi
;; Basic
"postgresql://localhost/mydb"

;; Username and password
"postgresql://admin:secret@localhost/mydb"

;; Port specified
"postgresql://admin:secret@localhost:5433/mydb"

;; SSL mode
"postgresql://admin:secret@localhost/mydb?sslmode=require"
```

### Loading from Environment Variables

```qi
;; Load connection string from environment variable (future plan)
(def db-conn (env/get "DATABASE_URL"))
```

---

## Related Documentation

- **[16-stdlib-auth.md](16-stdlib-auth.md)** - Authentication integration
- **[08-error-handling.md](08-error-handling.md)** - Result type patterns
- **[11-stdlib-http.md](11-stdlib-http.md)** - Using DB in web applications
- **[12-stdlib-json.md](12-stdlib-json.md)** - Storing/loading JSON data

---

## Implementation Details

### Unified Interface Design

#### Design Principles

Database (RDBMS) and KVS (Key-Value Store) are designed with **unified interface** pattern.
This is the same approach as Go's `database/sql` package, allowing transparent backend handling.

```
Unified Interface (user-facing):
- db/connect, db/query, db/exec      ... RDBMS unified interface
- kvs/connect, kvs/get, kvs/set      ... KVS unified interface

Internal Drivers (not public):
- SqliteDriver, PostgresDriver, MysqlDriver
- RedisDriver, MemcachedDriver (future)
```

#### RDBMS Design

```qi
;; Unified interface (backend auto-detection)
(def conn (db/connect "postgresql://localhost/mydb"))
(def conn (db/connect "mysql://root:pass@localhost/mydb"))
(def conn (db/connect "sqlite:path/to/db.db"))

;; Code is backend-agnostic
(db/query conn "SELECT * FROM users" [])
(db/exec conn "INSERT INTO users (name) VALUES (?)" ["Alice"])

;; Transactions
(def tx (db/begin conn))
(db/exec tx "UPDATE accounts SET balance = balance - 100 WHERE id = 1" [])
(db/exec tx "UPDATE accounts SET balance = balance + 100 WHERE id = 2" [])
(db/commit tx)
```

**Backend Switching**: Change only the connection URL to migrate between PostgreSQL↔MySQL↔SQLite.

**No Public-Specific Functions**: Functions like `db/pg-*`, `db/my-*` are for internal use only.
Add only when unified interface cannot express functionality (PostgreSQL COPY, MySQL LOAD DATA, etc.).

#### KVS Design

```qi
;; Unified interface (backend auto-detection)
(def kvs (kvs/connect "redis://localhost:6379"))

;; Code is backend-agnostic
(kvs/set kvs "key" "value")
(kvs/get kvs "key")
(kvs/delete kvs "key")

;; Data structure operations
(kvs/hset kvs "user:1" "name" "Alice")  ;; Hash
(kvs/lpush kvs "queue" "task1")         ;; List
(kvs/sadd kvs "tags" "redis")           ;; Set
```

**Backend Switching**: Change only the connection URL to switch Redis↔Memcached, etc. (future).

**No Public-Specific Functions**: Functions like `kvs/redis-*` are for internal use only.
Add only when unified interface cannot express functionality (Redis Pub/Sub, Lua scripting, etc.).

### Crates Used

**RDBMS**:
- **rusqlite** (v0.32) - Pure Rust SQLite client
- **tokio-postgres** (v0.7) - Pure Rust PostgreSQL client
- **mysql_async** (v0.34) - Pure Rust MySQL client
- **tokio** - Async runtime

**KVS**:
- **redis** (v0.27) - Pure Rust Redis client
- **tokio** - Async runtime

### Async Processing

Internally uses async APIs, but exposed as sync API to Qi users.

```rust
// Rust implementation (reference)
let rt = tokio::runtime::Runtime::new()?;
rt.block_on(async {
    let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;
    client.query(query, &params).await
})
```

### Driver Pattern

```rust
// Unified interface trait
pub trait DbDriver: Send + Sync {
    fn connect(&self, url: &str, opts: &ConnectionOptions)
        -> DbResult<Arc<dyn DbConnection>>;
    fn name(&self) -> &str;
}

pub trait DbConnection: Send + Sync {
    fn query(&self, sql: &str, params: &[Value], opts: &QueryOptions)
        -> DbResult<Rows>;
    fn exec(&self, sql: &str, params: &[Value], opts: &QueryOptions)
        -> DbResult<i64>;
    fn begin(&self, opts: &TransactionOptions)
        -> DbResult<Arc<dyn DbTransaction>>;
    // ...
}

// Backend implementations (internal only)
pub struct SqliteDriver;
pub struct PostgresDriver;
pub struct MysqlDriver;

impl DbDriver for SqliteDriver { /* ... */ }
impl DbConnection for SqliteConnection { /* ... */ }
```

---

## Roadmap

### RDBMS

Future features to implement:

- **Connection pooling**: Reuse connections for performance
- **Streaming queries**: Efficient processing of large data
- **Specific functions**: Only when unified interface cannot express
  - PostgreSQL: `COPY`, `LISTEN/NOTIFY`
  - MySQL: `LOAD DATA`
  - SQLite: Custom functions, virtual tables

### KVS

Future features to implement:

- **Memcached support**: `kvs/connect "memcached://..."`
- **In-memory KVS**: `kvs/connect ":memory:"` (Pure Rust, no dependencies)
- **Specific functions**: Only when unified interface cannot express
  - Redis: Pub/Sub, Lua scripting, Sorted Sets, Streams

---

## Summary

Qi's database & KVS library provides simple and safe access through unified interface.

### RDBMS (db/*)

- **db/connect**: PostgreSQL/MySQL/SQLite auto-detection
- **db/query**: Execute SELECT queries
- **db/exec**: Execute INSERT/UPDATE/DELETE
- **db/begin/commit/rollback**: Transactions
- **Parameterized queries**: SQL injection protection
- **Backend-agnostic switching**: Change only connection URL

### KVS (kvs/*)

- **kvs/connect**: Redis support (Memcached, etc. in future)
- **kvs/get/set/delete**: Basic operations
- **kvs/hget/hset**: Hash operations
- **kvs/lpush/rpush**: List operations
- **kvs/sadd/smembers**: Set operations
- **Backend-agnostic switching**: Change only connection URL

By combining these features, you can easily build database and cache-driven applications.
