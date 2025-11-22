# Database

**Unified Interface for Databases (PostgreSQL/MySQL/SQLite)**

Qi provides unified access to relational databases.

---

## Table of Contents

- [Overview](#overview)
- [Unified Database Interface](#unified-database-interface)
  - [db/connect - Connection](#dbconnect---connection)
  - [db/query - Query Execution](#dbquery---query-execution)
  - [db/exec - Command Execution](#dbexec---command-execution)
- [Practical Examples](#practical-examples)
- [Error Handling](#error-handling)
- [Performance](#performance)
- [Connection String Format](#connection-string-format)
- [Implementation Details](#implementation-details)

---

## Overview

### Provided Features

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
features = ["db-sqlite", "db-postgres", "db-mysql"]
```

Enabled by default.

### Dependencies

**Database**:
- **rusqlite** (v0.32) - Pure Rust SQLite client
- **tokio-postgres** (v0.7) - Pure Rust PostgreSQL client
- **mysql_async** (v0.34) - Pure Rust MySQL client
- **tokio** - Async runtime

---

## Unified Database Interface

### Unified Interface Design

Database is designed with **unified interface** pattern. This is the same approach as Go's `database/sql` package, allowing transparent backend handling.

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

---

### db/connect - Connection

**Connects to database and returns connection ID.**

```qi
(db/connect url)
```

#### Arguments

- `url`: String (connection URL)
  - PostgreSQL: `"postgresql://user:password@host:port/dbname"`
  - MySQL: `"mysql://user:password@host:port/dbname"`
  - SQLite: `"sqlite:path/to/db.db"`

#### Return Value

- Connection ID (string)
- Error case: `{:error "message"}`

#### Usage Examples

```qi
;; PostgreSQL connection
(def db-conn (db/connect "postgresql://admin:secret@localhost:5432/myapp"))

;; MySQL connection
(def db-conn (db/connect "mysql://root:pass@localhost:3306/mydb"))

;; SQLite connection
(def db-conn (db/connect "sqlite:/path/to/database.db"))

;; Connection error
(def conn (db/connect "invalid-url"))
;; => {:error "Unsupported database URL: invalid-url"}
```

---

### db/query - Query Execution

**Executes SELECT query and returns rows.**

```qi
(db/query conn sql params)
```

#### Arguments

- `conn`: Connection ID (returned from `db/connect`)
- `sql`: SQL string
- `params`: Vector of parameters

#### Return Value

- Vector of result rows (each row is a map)
- Error case: `{:error "message"}`

#### Usage Examples

```qi
;; Get all users
(db/query db-conn "SELECT * FROM users" [])
;; => [{:id 1 :name "Alice" :email "alice@example.com"}
;;     {:id 2 :name "Bob" :email "bob@example.com"}]

;; Parameterized query
(db/query db-conn "SELECT * FROM users WHERE id = $1" [1])
;; => [{:id 1 :name "Alice" :email "alice@example.com"}]

;; WHERE IN clause
(db/query db-conn "SELECT * FROM users WHERE id IN ($1, $2, $3)" [1 2 3])

;; LIMIT/OFFSET (pagination)
(db/query db-conn "SELECT * FROM users LIMIT $1 OFFSET $2" [10 20])
```

---

### db/exec - Command Execution

**Executes INSERT/UPDATE/DELETE and returns affected rows.**

```qi
(db/exec conn sql params)
```

#### Arguments

- `conn`: Connection ID
- `sql`: SQL string
- `params`: Vector of parameters

#### Return Value

- Number of affected rows (integer)
- Error case: `{:error "message"}`

#### Usage Examples

```qi
;; INSERT
(db/exec db-conn "INSERT INTO users (name, email) VALUES ($1, $2)" ["Alice" "alice@example.com"])
;; => 1

;; UPDATE
(db/exec db-conn "UPDATE users SET name = $1 WHERE id = $2" ["Bob" 1])
;; => 1

;; DELETE
(db/exec db-conn "DELETE FROM users WHERE id = $1" [1])
;; => 1

;; Multiple row INSERT
(db/exec db-conn "INSERT INTO users (name, email) VALUES ($1, $2), ($3, $4)" ["Alice" "alice@example.com" "Bob" "bob@example.com"])
;; => 2
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

---

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

---

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

---

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

---

### Connection Errors

```qi
;; Invalid connection string
(def conn (db/connect "invalid-url"))
;; => {:error "Unsupported database URL: invalid-url"}

;; Connection timeout
(def conn (db/connect "postgresql://localhost:9999/db"))
;; => {:error "Connection failed: connection refused"}
```

---

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

---

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

### PostgreSQL

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

**Examples**:
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

---

### MySQL

```
mysql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

**Examples**:
```qi
;; Basic
"mysql://root@localhost/mydb"

;; Username and password
"mysql://root:pass@localhost/mydb"

;; Port specified
"mysql://root:pass@localhost:3307/mydb"
```

---

### SQLite

```
sqlite:path/to/database.db
```

**Examples**:
```qi
;; Relative path
"sqlite:mydb.db"

;; Absolute path
"sqlite:/var/lib/myapp/data.db"

;; In-memory (future support)
"sqlite::memory:"
```

---

### Loading from Environment Variables

```qi
;; Load connection string from environment variable (future plan)
(def db-conn (db/connect (env/get "DATABASE_URL")))
```

---

## Implementation Details

### Unified Interface Design

Database (RDBMS) is designed with **unified interface** pattern.
This is the same approach as Go's `database/sql` package, allowing transparent backend handling.

```
Unified Interface (user-facing):
- db/connect, db/query, db/exec      ... RDBMS unified interface

Internal Drivers (not public):
- SqliteDriver, PostgresDriver, MysqlDriver
```

---

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

---

## Advanced Features

### db/call - Call Stored Procedures/Functions

```qi
(db/call conn name params)
```

#### Arguments

- `conn`: Connection ID (`db/connect`) or transaction ID (`db/begin`)
- `name`: Procedure/function name (string)
- `params`: Parameter vector (optional)

#### Returns

- For single return value: The value
- For result set: Vector of rows
- For multiple result sets: Vector of vectors

#### Examples

```qi
;; PostgreSQL - Call stored function
(db/call conn "calculate_total" [100 0.08])
;; => 108.0

;; MySQL - Call stored procedure
(db/call conn "get_user_orders" [user-id])
;; => [{:order_id 1 :total 100} {:order_id 2 :total 200}]

;; Use within transaction
(def tx (db/begin conn))
(db/call tx "update_inventory" [product-id -1])
(db/commit tx)
```

#### Notes

- SQLite does not support stored procedures
- PostgreSQL automatically detects function (SELECT) vs procedure (CALL)
- MySQL executes via CALL statement

---

### db/tables - Get Table List

```qi
(db/tables conn)
```

#### Arguments

- `conn`: Connection ID

#### Returns

- Vector of table names (strings)

#### Examples

```qi
(db/tables conn)
;; => ["users" "posts" "comments"]
```

---

### db/columns - Get Column Information

```qi
(db/columns conn table-name)
```

#### Arguments

- `conn`: Connection ID
- `table-name`: Table name (string)

#### Returns

- Vector of column information maps
  - `:name` - Column name
  - `:type` - Data type
  - `:nullable` - NULL allowed (true/false)
  - `:default` - Default value (nil if none)
  - `:primary_key` - Primary key (true/false)

#### Examples

```qi
(db/columns conn "users")
;; => [{:name "id" :type "integer" :nullable false :default nil :primary_key true}
;;     {:name "name" :type "text" :nullable false :default nil :primary_key false}
;;     {:name "email" :type "text" :nullable true :default nil :primary_key false}]
```

---

### db/indexes - Get Index List

```qi
(db/indexes conn table-name)
```

#### Arguments

- `conn`: Connection ID
- `table-name`: Table name (string)

#### Returns

- Vector of index information maps
  - `:name` - Index name
  - `:table` - Table name
  - `:columns` - Vector of column names
  - `:unique` - Unique index (true/false)

#### Examples

```qi
(db/indexes conn "users")
;; => [{:name "users_email_idx" :table "users" :columns ["email"] :unique true}]
```

---

### db/foreign-keys - Get Foreign Key List

```qi
(db/foreign-keys conn table-name)
```

#### Arguments

- `conn`: Connection ID
- `table-name`: Table name (string)

#### Returns

- Vector of foreign key information maps
  - `:name` - Foreign key name
  - `:table` - Table name
  - `:columns` - Vector of column names
  - `:referenced_table` - Referenced table name
  - `:referenced_columns` - Vector of referenced column names

#### Examples

```qi
(db/foreign-keys conn "posts")
;; => [{:name "posts_user_id_fkey"
;;      :table "posts"
;;      :columns ["user_id"]
;;      :referenced_table "users"
;;      :referenced_columns ["id"]}]
```

---

### db/sanitize - Sanitize Values

```qi
(db/sanitize conn value)
```

#### Arguments

- `conn`: Connection ID
- `value`: String to sanitize

#### Returns

- Sanitized string

#### Examples

```qi
(db/sanitize conn "O'Reilly")
;; PostgreSQL => "O''Reilly"
;; MySQL => "O\'Reilly"
```

#### Notes

**Using bind parameters is recommended.** Use this function only when building dynamic SQL.

---

### db/sanitize-identifier - Sanitize Identifiers

```qi
(db/sanitize-identifier conn identifier)
```

#### Arguments

- `conn`: Connection ID
- `identifier`: Table/column name to sanitize

#### Returns

- Sanitized identifier

#### Examples

```qi
(db/sanitize-identifier conn "user name")
;; PostgreSQL => "\"user name\""
;; MySQL => "`user name`"
```

---

### db/escape-like - Escape LIKE Patterns

```qi
(db/escape-like conn pattern)
```

#### Arguments

- `conn`: Connection ID
- `pattern`: LIKE pattern string

#### Returns

- Escaped pattern string

#### Examples

```qi
(db/escape-like conn "50%_off")
;; => "50\\%\\_off" (PostgreSQL/MySQL)

;; Usage in LIKE search
(def pattern (db/escape-like conn user-input))
(db/query conn "SELECT * FROM products WHERE name LIKE ?" [(str pattern "%")])
```

---

### db/supports? - Check Feature Support

```qi
(db/supports? conn feature)
```

#### Arguments

- `conn`: Connection ID
- `feature`: Feature name (string)

#### Returns

- If supported: `true`
- If not supported: `false`

#### Examples

```qi
(db/supports? conn "transactions")
;; => true

(db/supports? conn "stored_procedures")
;; PostgreSQL/MySQL => true
;; SQLite => false
```

---

### db/driver-info - Get Driver Information

```qi
(db/driver-info conn)
```

#### Arguments

- `conn`: Connection ID

#### Returns

- Driver information map
  - `:name` - Driver name ("PostgreSQL", "MySQL", "SQLite")
  - `:version` - Driver version
  - `:database_version` - Database version

#### Examples

```qi
(db/driver-info conn)
;; => {:name "PostgreSQL"
;;     :version "0.19.0"
;;     :database_version "PostgreSQL 15.3"}
```

---

### db/query-info - Get Query Metadata

```qi
(db/query-info conn sql)
```

#### Arguments

- `conn`: Connection ID
- `sql`: SQL string

#### Returns

- Query information map
  - `:columns` - Vector of column information (same format as `db/columns`)
  - `:parameter_count` - Parameter count

#### Examples

```qi
(db/query-info conn "SELECT id, name FROM users WHERE age > $1")
;; => {:columns [{:name "id" :type "integer" ...}
;;               {:name "name" :type "text" ...}]
;;     :parameter_count 1}
```

#### Notes

The query is not executed. Only metadata is retrieved.

---

## Roadmap

### Future Features

**RDBMS**:
- **Connection pooling**: Reuse connections for performance
- **Streaming queries**: Efficient processing of large data
- **Specific functions**: Only when unified interface cannot express
  - PostgreSQL: `COPY`, `LISTEN/NOTIFY`
  - MySQL: `LOAD DATA`
  - SQLite: Custom functions, virtual tables

---

## Summary

Qi's database library provides simple and safe access through unified interface.

### RDBMS (db/*)

- **db/connect**: PostgreSQL/MySQL/SQLite auto-detection
- **db/query**: Execute SELECT queries
- **db/exec**: Execute INSERT/UPDATE/DELETE
- **db/begin/commit/rollback**: Transactions
- **Parameterized queries**: SQL injection protection
- **Backend-agnostic switching**: Change only connection URL

By combining these features, you can easily build database-driven applications.

---

## Related Documentation

- **[24-stdlib-kvs.md](24-stdlib-kvs.md)** - Key-Value Store Unified Interface (Redis)
- **[16-stdlib-auth.md](16-stdlib-auth.md)** - Authentication integration
- **[08-error-handling.md](08-error-handling.md)** - Error handling patterns
- **[11-stdlib-http.md](11-stdlib-http.md)** - Using DB in web applications
- **[12-stdlib-json.md](12-stdlib-json.md)** - Storing/loading JSON data
