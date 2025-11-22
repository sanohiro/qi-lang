# データベース

**データベース（PostgreSQL/MySQL/SQLite）の統一インターフェース**

Qiは、リレーショナルデータベースへの統一的なアクセスを提供します。

---

## 目次

- [概要](#概要)
- [データベース統一インターフェース](#データベース統一インターフェース)
  - [db/connect - 接続](#dbconnect---接続)
  - [db/query - クエリ実行](#dbquery---クエリ実行)
  - [db/exec - コマンド実行](#dbexec---コマンド実行)
- [実用例](#実用例)
- [エラー処理](#エラー処理)
- [パフォーマンス](#パフォーマンス)
- [接続文字列の形式](#接続文字列の形式)
- [実装の詳細](#実装の詳細)

---

## 概要

### 提供機能

**データベース**:
- **統一インターフェース（db/*）**: PostgreSQL/MySQL/SQLite対応
  - 接続管理（`db/connect`）
  - クエリ実行（`db/query`）
  - コマンド実行（`db/exec`）
  - トランザクション（`db/begin`, `db/commit`, `db/rollback`）
  - パラメータ化クエリ対応
  - バックエンド透過的な切り替え（接続URLのみ変更）

### feature flag

```toml
# Cargo.toml
features = ["db-sqlite", "db-postgres", "db-mysql"]
```

デフォルトで有効です。

### 依存クレート

**データベース**:
- **rusqlite** (v0.32) - Pure Rust SQLiteクライアント
- **tokio-postgres** (v0.7) - Pure Rust PostgreSQLクライアント
- **mysql_async** (v0.34) - Pure Rust MySQLクライアント
- **tokio** - 非同期ランタイム

---

## データベース統一インターフェース

### 統一インターフェース設計

データベースは**統一インターフェース**パターンで設計されています。これはGoの`database/sql`パッケージと同じアプローチで、バックエンド（ドライバー）を透過的に扱えます。

```qi
;; 統一インターフェース（バックエンド自動判別）
(def conn (db/connect "postgresql://localhost/mydb"))
(def conn (db/connect "mysql://root:pass@localhost/mydb"))
(def conn (db/connect "sqlite:path/to/db.db"))

;; 以降のコードはバックエンド非依存
(db/query conn "SELECT * FROM users" [])
(db/exec conn "INSERT INTO users (name) VALUES (?)" ["Alice"])

;; トランザクション
(def tx (db/begin conn))
(db/exec tx "UPDATE accounts SET balance = balance - 100 WHERE id = 1" [])
(db/exec tx "UPDATE accounts SET balance = balance + 100 WHERE id = 2" [])
(db/commit tx)
```

**バックエンド切り替え**: 接続URLを変更するだけでPostgreSQL↔MySQL↔SQLite間を移行できます。

**専用関数は公開しない**: `db/pg-*`, `db/my-*`のような専用関数は内部実装用のみです。
統一インターフェースで表現できない機能（PostgreSQLのCOPY、MySQLのLOAD DATA等）が必要になった場合のみ追加します。

---

### db/connect - 接続

**データベースに接続し、接続IDを返します。**

```qi
(db/connect url)
```

#### 引数

- `url`: 文字列（接続URL）
  - PostgreSQL: `"postgresql://user:password@host:port/dbname"`
  - MySQL: `"mysql://user:password@host:port/dbname"`
  - SQLite: `"sqlite:path/to/db.db"`

#### 戻り値

- 接続ID（文字列）
- エラーの場合: `{:error "message"}`

#### 使用例

```qi
;; PostgreSQL接続
(def db-conn (db/connect "postgresql://admin:secret@localhost:5432/myapp"))

;; MySQL接続
(def db-conn (db/connect "mysql://root:pass@localhost:3306/mydb"))

;; SQLite接続
(def db-conn (db/connect "sqlite:/path/to/database.db"))

;; 接続エラー
(def conn (db/connect "invalid-url"))
;; => {:error "Unsupported database URL: invalid-url"}
```

---

### db/query - クエリ実行

**SELECTクエリを実行し、結果行を返します。**

```qi
(db/query conn sql params)
```

#### 引数

- `conn`: 接続ID（`db/connect`の戻り値）
- `sql`: SQL文字列
- `params`: パラメータのベクタ

#### 戻り値

- 結果行のベクタ（各行はマップ）
- エラーの場合: `{:error "message"}`

#### 使用例

```qi
;; 全ユーザー取得
(db/query db-conn "SELECT * FROM users" [])
;; => [{:id 1 :name "Alice" :email "alice@example.com"}
;;     {:id 2 :name "Bob" :email "bob@example.com"}]

;; パラメータ化クエリ
(db/query db-conn "SELECT * FROM users WHERE id = $1" [1])
;; => [{:id 1 :name "Alice" :email "alice@example.com"}]

;; WHERE IN句
(db/query db-conn "SELECT * FROM users WHERE id IN ($1, $2, $3)" [1 2 3])

;; LIMIT/OFFSET（ページング）
(db/query db-conn "SELECT * FROM users LIMIT $1 OFFSET $2" [10 20])
```

---

### db/query-one - クエリ実行（1行のみ）

**SELECTクエリを実行し、最初の1行のみを返します。結果がない場合はnilを返します。**

```qi
(db/query-one conn sql params)
```

#### 引数

- `conn`: 接続ID（`db/connect`の戻り値）またはトランザクションID（`db/begin`の戻り値）
- `sql`: SQL文字列
- `params`: パラメータのベクタ

#### 戻り値

- 1行のマップ（結果がない場合はnil）
- エラーの場合: `{:error "message"}`

#### 使用例

```qi
;; IDでユーザーを1件取得
(db/query-one db-conn "SELECT * FROM users WHERE id = $1" [1])
;; => {:id 1 :name "Alice" :email "alice@example.com"}

;; 結果がない場合
(db/query-one db-conn "SELECT * FROM users WHERE id = $1" [999])
;; => nil

;; トランザクション内での使用
(def tx (db/begin db-conn))
(db/query-one tx "SELECT * FROM users WHERE id = $1" [1])
;; => {:id 1 :name "Alice" :email "alice@example.com"}
(db/commit tx)

;; パイプラインで使用
(db-conn
 |> (db/query-one "SELECT name FROM users WHERE id = $1" [1])
 |> (get "name"))
;; => "Alice"
```

---

### db/exec - コマンド実行

**INSERT/UPDATE/DELETEを実行し、影響を受けた行数を返します。**

```qi
(db/exec conn sql params)
```

#### 引数

- `conn`: 接続ID
- `sql`: SQL文字列
- `params`: パラメータのベクタ

#### 戻り値

- 影響を受けた行数（整数）
- エラーの場合: `{:error "message"}`

#### 使用例

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

;; 複数行INSERT
(db/exec db-conn "INSERT INTO users (name, email) VALUES ($1, $2), ($3, $4)" ["Alice" "alice@example.com" "Bob" "bob@example.com"])
;; => 2
```

---

## 実用例

### ユーザー管理システム

```qi
;; データベース接続（統一インターフェース）
(def db-conn (db/connect "postgresql://admin:secret@localhost:5432/myapp"))

;; ユーザー作成
(defn create-user [name email password-hash]
  (db/exec db-conn
    "INSERT INTO users (name, email, password_hash, created_at)
     VALUES ($1, $2, $3, NOW()) RETURNING id"
    [name email password-hash]))

;; ユーザー検索
(defn find-user-by-email [email]
  (db/query db-conn
    "SELECT id, name, email, created_at FROM users WHERE email = $1"
    [email]))

;; ユーザー更新
(defn update-user [user-id name email]
  (db/exec db-conn
    "UPDATE users SET name = $1, email = $2, updated_at = NOW()
     WHERE id = $3"
    [name email user-id]))

;; ユーザー削除
(defn delete-user [user-id]
  (db/exec db-conn
    "DELETE FROM users WHERE id = $1"
    [user-id]))

;; 使用例
(def result (create-user "Alice" "alice@example.com" "$argon2id$..."))
;; => 1

(find-user-by-email "alice@example.com")
;; => [{:id 1 :name "Alice" :email "alice@example.com" :created_at "..."}]
```

---

### ページネーション

```qi
;; ページングされた結果を取得
(defn get-users-page [page per-page]
  (let [offset (* (- page 1) per-page)]
    (db/query db-conn
      "SELECT id, name, email FROM users
       ORDER BY created_at DESC
       LIMIT $1 OFFSET $2"
      [per-page offset])))

;; 使用例
(get-users-page 1 10)  ;; 1ページ目、10件
;; => [{:id 100 :name "Zara" ...} ...]

(get-users-page 2 10)  ;; 2ページ目、10件
;; => [{:id 90 :name "Yuki" ...} ...]
```

---

### トランザクション（手動）

```qi
;; トランザクション例（手動コミット）
(defn transfer-money [from-id to-id amount]
  (let [conn db-conn]
    ;; BEGIN
    (db/exec conn "BEGIN" [])

    ;; 出金
    (def debit-result
      (db/exec conn
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2"
        [amount from-id]))

    ;; 入金
    (def credit-result
      (db/exec conn
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2"
        [amount to-id]))

    ;; コミットまたはロールバック
    (match [debit-result credit-result]
      [1 1] -> (do
                 (db/exec conn "COMMIT" [])
                 "Transfer successful")
      _ -> (do
             (db/exec conn "ROLLBACK" [])
             {:error "Transfer failed"}))))
```

---

### 集計クエリ

```qi
;; ユーザー数を取得
(defn count-users []
  (db/query db-conn "SELECT COUNT(*) as count FROM users" [])
  |>? (fn [rows] (get (first rows) :count)))

;; グループ化
(defn count-users-by-status []
  (db/query db-conn
    "SELECT status, COUNT(*) as count
     FROM users
     GROUP BY status"
    []))

;; 使用例
(count-users)
;; => 1523

(count-users-by-status)
;; => [{:status "active" :count 1200}
;;     {:status "inactive" :count 323}]
```

---

## エラー処理

### エラーハンドリング

データベース関数は成功時に生データ、失敗時に`{:error "message"}`を返します。

```qi
;; 基本的なエラー処理
(def result (db/query db-conn "SELECT * FROM users" []))
(if (error? result)
  (println "Error:" (get result :error))
  (println "Found" (count result) "users"))

;; パイプラインでのエラー処理（|>?でショートサーキット）
(defn get-user-email [user-id]
  (db/query db-conn "SELECT email FROM users WHERE id = $1" [user-id])
  |>? (fn [rows]
        (if (empty? rows)
          {:error "User not found"}
          (get (first rows) "email"))))

;; matchでのエラー処理
(match (db/query db-conn "SELECT * FROM users" [])
  {:error e} -> (println "Database error:" e)
  rows -> (println "Found" (count rows) "users"))
```

---

### 接続エラー

```qi
;; 不正な接続文字列
(def conn (db/connect "invalid-url"))
;; => {:error "Unsupported database URL: invalid-url"}

;; 接続タイムアウト
(def conn (db/connect "postgresql://localhost:9999/db"))
;; => {:error "Connection failed: connection refused"}
```

---

### クエリエラー

```qi
;; 構文エラー
(db/query db-conn "SELEC * FROM users" [])
;; => {:error "Query error: syntax error at or near \"SELEC\""}

;; テーブルが存在しない
(db/query db-conn "SELECT * FROM nonexistent_table" [])
;; => {:error "Query error: relation \"nonexistent_table\" does not exist"}
```

---

## パフォーマンス

### コネクションプール（未実装）

現在の実装では、各クエリごとに新しい接続を確立します。

将来的には、コネクションプールをサポートする予定です：

```qi
;; 将来の計画
(def pool (db/create-pool "postgresql://..." {:max-connections 10}))
(db/with-connection pool (fn [conn]
  (db/query conn "SELECT * FROM users" [])))
```

---

### パラメータ化クエリ

SQLインジェクション攻撃を防ぐため、常にパラメータ化クエリを使用してください：

```qi
;; ❌ 危険: SQLインジェクションの脆弱性
(def user-input "1 OR 1=1")
(db/query db-conn (str "SELECT * FROM users WHERE id = " user-input) [])

;; ✅ 安全: パラメータ化クエリ
(db/query db-conn "SELECT * FROM users WHERE id = $1" [user-input])
```

---

## 接続文字列の形式

### PostgreSQL

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

**例**:
```qi
;; 基本
"postgresql://localhost/mydb"

;; ユーザー名とパスワード
"postgresql://admin:secret@localhost/mydb"

;; ポート指定
"postgresql://admin:secret@localhost:5433/mydb"

;; SSLモード
"postgresql://admin:secret@localhost/mydb?sslmode=require"
```

---

### MySQL

```
mysql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

**例**:
```qi
;; 基本
"mysql://root@localhost/mydb"

;; ユーザー名とパスワード
"mysql://root:pass@localhost/mydb"

;; ポート指定
"mysql://root:pass@localhost:3307/mydb"
```

---

### SQLite

```
sqlite:path/to/database.db
```

**例**:
```qi
;; 相対パス
"sqlite:mydb.db"

;; 絶対パス
"sqlite:/var/lib/myapp/data.db"

;; インメモリ（将来対応）
"sqlite::memory:"
```

---

### 環境変数からの読み込み

```qi
;; 環境変数から接続文字列を取得（将来の計画）
(def db-conn (db/connect (env/get "DATABASE_URL")))
```

---

## 実装の詳細

### 統一インターフェース設計

データベース（RDBMS）は、**統一インターフェース**パターンで設計されています。
これはGoの`database/sql`パッケージと同じアプローチで、バックエンド（ドライバー）を透過的に扱えます。

```
統一インターフェース（ユーザーが使う）:
- db/connect, db/query, db/exec      ... RDBMS統一インターフェース

内部ドライバー（公開しない）:
- SqliteDriver, PostgresDriver, MysqlDriver
```

---

### ドライバーパターン

```rust
// 統一インターフェーストレイト
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

// バックエンド実装（内部のみ）
pub struct SqliteDriver;
pub struct PostgresDriver;
pub struct MysqlDriver;

impl DbDriver for SqliteDriver { /* ... */ }
impl DbConnection for SqliteConnection { /* ... */ }
```

---

### 非同期処理

内部的には非同期APIを使用していますが、Qiのユーザーには同期的なAPIとして公開されています。

```rust
// Rustでの実装（参考）
let rt = tokio::runtime::Runtime::new()?;
rt.block_on(async {
    let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;
    client.query(query, &params).await
})
```

---

## 高度な機能

### db/call - ストアドプロシージャ/ファンクション呼び出し

```qi
(db/call conn name params)
```

#### 引数

- `conn`: 接続ID（`db/connect`）またはトランザクションID（`db/begin`）
- `name`: プロシージャ/ファンクション名（文字列）
- `params`: パラメータのベクタ（省略可）

#### 戻り値

- 単一の戻り値の場合: その値
- 結果セットの場合: 行のベクタ
- 複数の結果セットの場合: ベクタのベクタ

#### 使用例

```qi
;; PostgreSQL - ストアドファンクション呼び出し
(db/call conn "calculate_total" [100 0.08])
;; => 108.0

;; MySQL - ストアドプロシージャ呼び出し
(db/call conn "get_user_orders" [user-id])
;; => [{:order_id 1 :total 100} {:order_id 2 :total 200}]

;; トランザクション内での使用
(def tx (db/begin conn))
(db/call tx "update_inventory" [product-id -1])
(db/commit tx)
```

#### 注意

- SQLiteはストアドプロシージャをサポートしていません
- PostgreSQLでは関数（SELECT）とプロシージャ（CALL）を自動判別します
- MySQLではCALL文で実行します

---

### db/tables - テーブル一覧取得

```qi
(db/tables conn)
```

#### 引数

- `conn`: 接続ID

#### 戻り値

- テーブル名の文字列ベクタ

#### 使用例

```qi
(db/tables conn)
;; => ["users" "posts" "comments"]
```

---

### db/columns - カラム情報取得

```qi
(db/columns conn table-name)
```

#### 引数

- `conn`: 接続ID
- `table-name`: テーブル名（文字列）

#### 戻り値

- カラム情報のマップのベクタ
  - `:name` - カラム名
  - `:type` - データ型
  - `:nullable` - NULL許可（true/false）
  - `:default` - デフォルト値（ない場合はnil）
  - `:primary_key` - 主キーか（true/false）

#### 使用例

```qi
(db/columns conn "users")
;; => [{:name "id" :type "integer" :nullable false :default nil :primary_key true}
;;     {:name "name" :type "text" :nullable false :default nil :primary_key false}
;;     {:name "email" :type "text" :nullable true :default nil :primary_key false}]
```

---

### db/indexes - インデックス一覧取得

```qi
(db/indexes conn table-name)
```

#### 引数

- `conn`: 接続ID
- `table-name`: テーブル名（文字列）

#### 戻り値

- インデックス情報のマップのベクタ
  - `:name` - インデックス名
  - `:table` - テーブル名
  - `:columns` - カラム名のベクタ
  - `:unique` - ユニークインデックスか（true/false）

#### 使用例

```qi
(db/indexes conn "users")
;; => [{:name "users_email_idx" :table "users" :columns ["email"] :unique true}]
```

---

### db/foreign-keys - 外部キー一覧取得

```qi
(db/foreign-keys conn table-name)
```

#### 引数

- `conn`: 接続ID
- `table-name`: テーブル名（文字列）

#### 戻り値

- 外部キー情報のマップのベクタ
  - `:name` - 外部キー名
  - `:table` - テーブル名
  - `:columns` - カラム名のベクタ
  - `:referenced_table` - 参照先テーブル名
  - `:referenced_columns` - 参照先カラム名のベクタ

#### 使用例

```qi
(db/foreign-keys conn "posts")
;; => [{:name "posts_user_id_fkey"
;;      :table "posts"
;;      :columns ["user_id"]
;;      :referenced_table "users"
;;      :referenced_columns ["id"]}]
```

---

### db/sanitize - 値のサニタイズ

```qi
(db/sanitize conn value)
```

#### 引数

- `conn`: 接続ID
- `value`: サニタイズする文字列

#### 戻り値

- サニタイズされた文字列

#### 使用例

```qi
(db/sanitize conn "O'Reilly")
;; PostgreSQL => "O''Reilly"
;; MySQL => "O\'Reilly"
```

#### 注意

**バインドパラメータを使う方が推奨されます。** この関数は動的SQLを構築する場合のみ使用してください。

---

### db/sanitize-identifier - 識別子のサニタイズ

```qi
(db/sanitize-identifier conn identifier)
```

#### 引数

- `conn`: 接続ID
- `identifier`: サニタイズするテーブル名/カラム名

#### 戻り値

- サニタイズされた識別子

#### 使用例

```qi
(db/sanitize-identifier conn "user name")
;; PostgreSQL => "\"user name\""
;; MySQL => "`user name`"
```

---

### db/escape-like - LIKE句のエスケープ

```qi
(db/escape-like conn pattern)
```

#### 引数

- `conn`: 接続ID
- `pattern`: LIKE句のパターン文字列

#### 戻り値

- エスケープされたパターン文字列

#### 使用例

```qi
(db/escape-like conn "50%_off")
;; => "50\\%\\_off" (PostgreSQL/MySQL)

;; LIKE検索での使用
(def pattern (db/escape-like conn user-input))
(db/query conn "SELECT * FROM products WHERE name LIKE ?" [(str pattern "%")])
```

---

### db/supports? - 機能サポート確認

```qi
(db/supports? conn feature)
```

#### 引数

- `conn`: 接続ID
- `feature`: 機能名（文字列）

#### 戻り値

- サポートされている場合: `true`
- サポートされていない場合: `false`

#### 使用例

```qi
(db/supports? conn "transactions")
;; => true

(db/supports? conn "stored_procedures")
;; PostgreSQL/MySQL => true
;; SQLite => false
```

---

### db/driver-info - ドライバー情報取得

```qi
(db/driver-info conn)
```

#### 引数

- `conn`: 接続ID

#### 戻り値

- ドライバー情報のマップ
  - `:name` - ドライバー名（"PostgreSQL", "MySQL", "SQLite"）
  - `:version` - ドライバーバージョン
  - `:database_version` - データベースバージョン

#### 使用例

```qi
(db/driver-info conn)
;; => {:name "PostgreSQL"
;;     :version "0.19.0"
;;     :database_version "PostgreSQL 15.3"}
```

---

### db/query-info - クエリメタデータ取得

```qi
(db/query-info conn sql)
```

#### 引数

- `conn`: 接続ID
- `sql`: SQL文字列

#### 戻り値

- クエリ情報のマップ
  - `:columns` - カラム情報のベクタ（`db/columns`と同じ形式）
  - `:parameter_count` - パラメータ数

#### 使用例

```qi
(db/query-info conn "SELECT id, name FROM users WHERE age > $1")
;; => {:columns [{:name "id" :type "integer" ...}
;;               {:name "name" :type "text" ...}]
;;     :parameter_count 1}
```

#### 注意

クエリは実行されません。メタデータのみを取得します。

---

## ロードマップ

### 将来的に実装予定の機能

**RDBMS**:
- **コネクションプール**: 接続の再利用で高速化
- **ストリーミングクエリ**: 大量データの効率的な処理
- **専用関数の追加**: 統一IFで表現できない場合のみ
  - PostgreSQL: `COPY`、`LISTEN/NOTIFY`
  - MySQL: `LOAD DATA`
  - SQLite: カスタム関数、仮想テーブル

---

## まとめ

Qiのデータベースライブラリは、統一インターフェースでシンプルかつ安全なアクセスを提供します。

### RDBMS（db/*）

- **db/connect**: PostgreSQL/MySQL/SQLite自動判別
- **db/query**: SELECTクエリ実行
- **db/exec**: INSERT/UPDATE/DELETE実行
- **db/begin/commit/rollback**: トランザクション
- **パラメータ化クエリ**: SQLインジェクション対策
- **バックエンド透過的切り替え**: 接続URLのみ変更

これらの機能を組み合わせることで、データベース駆動のアプリケーションを簡単に構築できます。

---

## 関連ドキュメント

- **[24-stdlib-kvs.md](24-stdlib-kvs.md)** - Key-Value Store統一インターフェース（Redis）
- **[16-stdlib-auth.md](16-stdlib-auth.md)** - 認証機能との統合
- **[08-error-handling.md](08-error-handling.md)** - Result型パターン
- **[11-stdlib-http.md](11-stdlib-http.md)** - WebアプリケーションでのDB使用
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSONデータの保存・読み込み
