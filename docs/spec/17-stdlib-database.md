# データベース

**PostgreSQLデータベース接続とクエリ実行**

Qiは、PostgreSQLデータベースへの接続とクエリ実行機能を標準ライブラリとして提供します。

---

## 目次

- [概要](#概要)
- [PostgreSQL](#postgresql)
  - [db/pg-query - クエリ実行](#dbpg-query---クエリ実行)
  - [db/pg-exec - コマンド実行](#dbpg-exec---コマンド実行)
- [実用例](#実用例)
- [エラー処理](#エラー処理)

---

## 概要

### 提供機能

- **PostgreSQL**: 非同期PostgreSQL接続
  - クエリ実行（SELECT）
  - コマンド実行（INSERT/UPDATE/DELETE）
  - パラメータ化クエリ対応
  - Result型による統一されたエラー処理

### feature flag

```toml
# Cargo.toml
features = ["db-postgres"]
```

デフォルトで有効です。

### 依存クレート

- **tokio-postgres** (v0.7) - Pure Rust PostgreSQLクライアント
- **tokio** - 非同期ランタイム（同期APIでラップ）

---

## PostgreSQL

### db/pg-query - クエリ実行

**PostgreSQLデータベースに接続してSELECTクエリを実行します。**

```qi
(db/pg-query connection-string query)
(db/pg-query connection-string query params)
```

#### 引数

- `connection-string`: 文字列（PostgreSQL接続文字列）
  - 形式: `"postgresql://user:password@host:port/database"`
- `query`: 文字列（SQLクエリ）
- `params`: ベクタ（オプション、パラメータのリスト）

#### 戻り値

- 成功: `{:ok [行のベクタ]}`
  - 各行はマップ形式（`{:カラム名 値}`）
- 失敗: `{:error "エラーメッセージ"}`

#### 使用例

```qi
;; 基本的なクエリ
(def conn "postgresql://user:pass@localhost/mydb")
(def result (db/pg-query conn "SELECT * FROM users" []))
;; => {:ok [{:id 1 :name "Alice" :email "alice@example.com"}
;;          {:id 2 :name "Bob" :email "bob@example.com"}]}

;; パラメータ化クエリ
(db/pg-query conn "SELECT * FROM users WHERE id = $1" [42])
;; => {:ok [{:id 42 :name "Carol" :email "carol@example.com"}]}

;; 複数パラメータ
(db/pg-query conn
  "SELECT * FROM posts WHERE user_id = $1 AND status = $2"
  [5 "published"])

;; パイプラインでの使用
(conn
 |> (db/pg-query "SELECT name FROM users WHERE active = $1" [true])
 |>? (fn [rows] (map (fn [row] (get row :name)) rows)))
;; => {:ok ["Alice" "Bob" "Carol"]}
```

#### サポートされるデータ型

クエリ結果は以下のQi型に変換されます：

| PostgreSQL型 | Qi型 |
|-------------|------|
| INTEGER, BIGINT | Integer |
| REAL, DOUBLE PRECISION | Float |
| TEXT, VARCHAR | String |
| BOOLEAN | Bool |
| NULL | Nil |
| その他 | String (未サポート型) |

---

### db/pg-exec - コマンド実行

**PostgreSQLデータベースに接続してINSERT/UPDATE/DELETEコマンドを実行します。**

```qi
(db/pg-exec connection-string command)
(db/pg-exec connection-string command params)
```

#### 引数

- `connection-string`: 文字列（PostgreSQL接続文字列）
- `command`: 文字列（SQLコマンド）
- `params`: ベクタ（オプション、パラメータのリスト）

#### 戻り値

- 成功: `{:ok 影響を受けた行数}`
- 失敗: `{:error "エラーメッセージ"}`

#### 使用例

```qi
;; INSERT
(def conn "postgresql://user:pass@localhost/mydb")
(db/pg-exec conn
  "INSERT INTO users (name, email) VALUES ($1, $2)"
  ["Alice" "alice@example.com"])
;; => {:ok 1}

;; UPDATE
(db/pg-exec conn
  "UPDATE users SET email = $1 WHERE id = $2"
  ["newemail@example.com" 42])
;; => {:ok 1}

;; DELETE
(db/pg-exec conn
  "DELETE FROM users WHERE id = $1"
  [999])
;; => {:ok 1}

;; 複数行INSERT
(db/pg-exec conn
  "INSERT INTO logs (user_id, action, created_at)
   VALUES ($1, $2, NOW()), ($3, $4, NOW())"
  [1 "login" 2 "logout"])
;; => {:ok 2}

;; パイプラインでの使用
([{:name "Alice" :email "a@example.com"}
  {:name "Bob" :email "b@example.com"}]
 |> (map (fn [user]
           (db/pg-exec conn
             "INSERT INTO users (name, email) VALUES ($1, $2)"
             [(get user :name) (get user :email)])))
 |> (filter (fn [r] (match r {:ok _} -> true _ -> false)))
 |> count)
;; => 2 (成功した挿入の数)
```

---

## 実用例

### ユーザー管理システム

```qi
;; データベース接続文字列
(def db-conn "postgresql://admin:secret@localhost:5432/myapp")

;; ユーザー作成
(defn create-user [name email password-hash]
  (db/pg-exec db-conn
    "INSERT INTO users (name, email, password_hash, created_at)
     VALUES ($1, $2, $3, NOW()) RETURNING id"
    [name email password-hash]))

;; ユーザー検索
(defn find-user-by-email [email]
  (db/pg-query db-conn
    "SELECT id, name, email, created_at FROM users WHERE email = $1"
    [email]))

;; ユーザー更新
(defn update-user [user-id name email]
  (db/pg-exec db-conn
    "UPDATE users SET name = $1, email = $2, updated_at = NOW()
     WHERE id = $3"
    [name email user-id]))

;; ユーザー削除
(defn delete-user [user-id]
  (db/pg-exec db-conn
    "DELETE FROM users WHERE id = $1"
    [user-id]))

;; 使用例
(def result (create-user "Alice" "alice@example.com" "$argon2id$..."))
;; => {:ok 1}

(find-user-by-email "alice@example.com")
;; => {:ok [{:id 1 :name "Alice" :email "alice@example.com" :created_at "..."}]}
```

### ページネーション

```qi
;; ページングされた結果を取得
(defn get-users-page [page per-page]
  (let [offset (* (- page 1) per-page)]
    (db/pg-query db-conn
      "SELECT id, name, email FROM users
       ORDER BY created_at DESC
       LIMIT $1 OFFSET $2"
      [per-page offset])))

;; 使用例
(get-users-page 1 10)  ;; 1ページ目、10件
;; => {:ok [{:id 100 :name "Zara" ...} ...]}

(get-users-page 2 10)  ;; 2ページ目、10件
;; => {:ok [{:id 90 :name "Yuki" ...} ...]}
```

### トランザクション（手動）

```qi
;; トランザクション例（手動コミット）
(defn transfer-money [from-id to-id amount]
  (let [conn db-conn]
    ;; BEGIN
    (db/pg-exec conn "BEGIN" [])

    ;; 出金
    (def debit-result
      (db/pg-exec conn
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2"
        [amount from-id]))

    ;; 入金
    (def credit-result
      (db/pg-exec conn
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2"
        [amount to-id]))

    ;; コミットまたはロールバック
    (match [debit-result credit-result]
      [{:ok 1} {:ok 1}] -> (do
                             (db/pg-exec conn "COMMIT" [])
                             {:ok "Transfer successful"})
      _ -> (do
             (db/pg-exec conn "ROLLBACK" [])
             {:error "Transfer failed"}))))
```

### 集計クエリ

```qi
;; ユーザー数を取得
(defn count-users []
  (db/pg-query db-conn "SELECT COUNT(*) as count FROM users" [])
  |>? (fn [rows] (get (first rows) :count)))

;; グループ化
(defn count-users-by-status []
  (db/pg-query db-conn
    "SELECT status, COUNT(*) as count
     FROM users
     GROUP BY status"
    []))

;; 使用例
(count-users)
;; => {:ok 1523}

(count-users-by-status)
;; => {:ok [{:status "active" :count 1200}
;;          {:status "inactive" :count 323}]}
```

---

## エラー処理

### Result型パターン

すべてのデータベース関数は`{:ok data}`または`{:error message}`を返します。

```qi
;; パイプラインでのエラー処理
(defn get-user-email [user-id]
  (db-conn
   |> (db/pg-query "SELECT email FROM users WHERE id = $1" [user-id])
   |>? (fn [rows]
         (if (empty? rows)
           {:error "User not found"}
           {:ok (get (first rows) :email)}))))

;; matchでのエラー処理
(match (db/pg-query db-conn "SELECT * FROM users" [])
  {:ok rows} -> (println "Found" (count rows) "users")
  {:error e} -> (println "Database error:" e))
```

### 接続エラー

```qi
;; 不正な接続文字列
(db/pg-query "invalid-connection-string" "SELECT 1" [])
;; => {:error "Connection error: ..."}

;; 接続タイムアウト
(db/pg-query "postgresql://localhost:9999/db" "SELECT 1" [])
;; => {:error "Connection error: connection refused"}
```

### クエリエラー

```qi
;; 構文エラー
(db/pg-query db-conn "SELEC * FROM users" [])
;; => {:error "Query error: syntax error at or near \"SELEC\""}

;; テーブルが存在しない
(db/pg-query db-conn "SELECT * FROM nonexistent_table" [])
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
  (db/pg-query conn "SELECT * FROM users" [])))
```

### パラメータ化クエリ

SQLインジェクション攻撃を防ぐため、常にパラメータ化クエリを使用してください：

```qi
;; ❌ 危険: SQLインジェクションの脆弱性
(def user-input "1 OR 1=1")
(db/pg-query db-conn (str "SELECT * FROM users WHERE id = " user-input) [])

;; ✅ 安全: パラメータ化クエリ
(db/pg-query db-conn "SELECT * FROM users WHERE id = $1" [user-input])
```

---

## 接続文字列の形式

PostgreSQL接続文字列の形式：

```
postgresql://[user[:password]@][host][:port][/dbname][?param1=value1&...]
```

### 例

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

### 環境変数からの読み込み

```qi
;; 環境変数から接続文字列を取得（将来の計画）
(def db-conn (env/get "DATABASE_URL"))
```

---

## 関連ドキュメント

- **[16-stdlib-auth.md](16-stdlib-auth.md)** - 認証機能との統合
- **[08-error-handling.md](08-error-handling.md)** - Result型パターン
- **[11-stdlib-http.md](11-stdlib-http.md)** - WebアプリケーションでのDB使用
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSONデータの保存・読み込み

---

## 実装の詳細

### 使用クレート

- **tokio-postgres** (v0.7) - Pure Rust PostgreSQLクライアント
- **tokio** - 非同期ランタイム

### 非同期処理

内部的にはtokio-postgresの非同期APIを使用していますが、Qiのユーザーには同期的なAPIとして公開されています。

```rust
// Rustでの実装（参考）
let rt = tokio::runtime::Runtime::new()?;
rt.block_on(async {
    let (client, connection) = tokio_postgres::connect(conn_str, NoTls).await?;
    client.query(query, &params).await
})
```

---

## ロードマップ

将来的に実装予定の機能：

- **コネクションプール**: 接続の再利用で高速化
- **トランザクションAPI**: `db/transaction`マクロ
- **プリペアドステートメント**: 繰り返しクエリの最適化
- **MySQL対応**: `db/mysql-query`、`db/mysql-exec`
- **SQLite対応**: 軽量DBとして`db/sqlite-query`、`db/sqlite-exec`

---

## まとめ

Qiのデータベースライブラリは、PostgreSQLへのシンプルで安全なアクセスを提供します。

- **db/pg-query**: SELECTクエリ実行
- **db/pg-exec**: INSERT/UPDATE/DELETE実行
- **Result型**: 統一されたエラー処理
- **パラメータ化クエリ**: SQLインジェクション対策

これらの機能を組み合わせることで、データベース駆動のアプリケーションを簡単に構築できます。
