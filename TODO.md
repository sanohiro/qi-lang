# TODO - 実装予定機能

このファイルは、将来実装予定の機能や改善点をリストアップしています。

## ✅ 分解機能の拡張（Destructuring Enhancement）- 実装完了

### 実装完了 (2025-01-13)

`...rest`構文やmapの分解が**let、fn/defn、match**のすべてで利用可能になりました。

### 以前の状態

`...rest`構文やmapの分解は**matchパターンマッチング**でのみ利用可能でした。

```qi
;; ✅ matchでは利用可能
(match [1 2 3 4]
  [x ...rest] -> (str "first: " x ", rest: " rest))

(match {:name "Alice" :age 30}
  {:name n :age a} -> (str n " is " a))
```

### 実装内容

以下の機能が実装されました：

#### ✅ 1. letで`...rest`構文

```qi
;; letでrestパターン
(let [[first ...rest] [1 2 3 4]]
  (str "first: " first ", rest: " rest))
;; => "first: 1, rest: (2 3 4)"

(let [[x y ...tail] [10 20 30 40]]
  {:x x :y y :tail tail})
;; => {:x 10, :y 20, :tail (30 40)}
```

#### ✅ 2. letでmapの分解

```qi
;; letでmap分解
(let [{:name n :age a} {:name "Alice" :age 30}]
  (str n " is " a))
;; => "Alice is 30"

;; :as束縛も対応
(let [{:name n :as person} {:name "Bob" :age 25}]
  [n person])
;; => ["Bob" {:name "Bob" :age 25}]
```

#### ✅ 3. fn/defnで`...rest`構文

```qi
;; fnの引数でrestパターン
(defn process-list [[first ...rest]]
  (str "first: " first ", rest: " rest))

(process-list [1 2 3])
;; => "first: 1, rest: (2 3)"
```

#### ✅ 4. fn/defnでmapの分解

```qi
;; fnの引数でmap分解
(defn greet [{:name n :age a}]
  (str n " is " a " years old"))

(greet {:name "Alice" :age 30})
;; => "Alice is 30 years old"
```

### 実装詳細

**変更したファイル**:
- `src/parser.rs`: `parse_defn()`, `parse_defn_private()`, `parse_fn()`に`LBrace`ケースを追加してmap分解に対応
- `SPEC.md`: fn、defn、letのセクションに実装例を追加

**備考**:
- `value.rs`の`FnParam::Map`は既に定義済みでした
- `parser.rs`の`parse_fn_param_vector()`と`parse_fn_param_map()`も既に実装済みでした
- `eval.rs`の`bind_fn_param()`も既に全パターンに対応済みでした
- 今回の実装で追加したのは、fn/defn/defn-のパーサーで`LBrace`トークンを認識してmap分解関数を呼び出す処理のみです

### テスト

`test_destructure.qi`に包括的なテストコードが含まれています。

---

## ✅ HTTPストリーミングレスポンス - 実装完了

### 実装完了 (2025-01-13)

大きなファイル（動画、大きなPDF等）を**メモリ効率的にストリーミング配信**できるようになりました。

### 以前の状態

- 全ファイルをメモリに読み込んでからレスポンス
- 10MBを超えるファイルはエラーになる
- メモリ使用量が大きい

### 実装内容

#### ✅ 1. 非同期ファイルストリーミング

```rust
// create_file_stream_body() 関数で実装
// tokio::fs::File + ReaderStream で8KBチャンクずつ読み込み
async fn create_file_stream_body(file_path: &str) -> Result<BoxBody<Bytes, Infallible>, String> {
    let file = TokioFile::open(file_path).await?;
    let reader_stream = ReaderStream::new(file);
    let stream = reader_stream.map(|result| {
        Ok::<_, Infallible>(Frame::data(Bytes::from(result.unwrap())))
    });
    let body = StreamBody::new(stream);
    Ok(body.boxed())
}
```

#### ✅ 2. `:body-file`キーによるストリーミング指定

```qi
;; ハンドラーでストリーミングレスポンスを返す
(defn my-handler [req]
  {:status 200
   :body-file "/path/to/large-file.mp4"  ; ストリーミング配信
   :headers {:content-type "video/mp4"}})

;; 従来の:bodyキーも引き続き使用可能（メモリに読み込み）
(defn my-handler [req]
  {:status 200
   :body "Hello, World!"  ; メモリに読み込み
   :headers {:content-type "text/plain"}})
```

#### ✅ 3. 静的ファイルサーバーも自動的にストリーミング対応

```qi
;; server/static-fileも内部で:body-fileを使用
(server/serve 8080
  (server/static-file "./public"))
```

### 実装詳細

**変更したファイル**:
- `Cargo.toml`: `tokio-util`, `tokio-stream`を追加、`http-server` featureを更新
- `src/builtins/server.rs`:
  - `create_file_stream_body()` 関数を追加
  - `value_to_response()` を async 化、`:body-file`キー対応
  - `handle_request()` を変更して async `value_to_response()` を呼び出し
  - `error_response()` を `BoxBody` 対応に修正
  - `serve_static_file()` と `native_server_static_file()` を`:body-file`対応に変更
  - `MAX_STATIC_FILE_SIZE` 定数を削除（制限なし）

**技術スタック**:
- `tokio::fs::File` - 非同期ファイルI/O
- `tokio_util::io::ReaderStream` - AsyncRead → Stream 変換
- `tokio_stream::StreamExt` - Stream操作
- `http_body_util::StreamBody` - ストリーミングHTTPボディ
- `http_body_util::combinators::BoxBody` - 型消去されたボディ

**使い分け**:
- **`:body`** - 文字列、JSON、HTML、小さなデータ（通常のレスポンス）
- **`:body-file`** - 大きなファイル（動画、大きなPDF、大きな画像など）

**メリット**:
- ファイルサイズの制限がなくなった
- メモリ使用量が大幅に削減（ファイル全体を読まない）
- 大きなファイル（GB単位）も配信可能

---

## ✅ データベース機能の拡張（Phase 2 & 3） - 実装完了

### 実装完了 (2025-01-13)

データベースの**メタデータAPI**、**機能検出API**、**クエリ情報取得**、および**コネクションプーリング**が実装されました。

### 実装内容

#### ✅ 1. メタデータAPI

```qi
;; テーブル一覧を取得
(def conn (db/connect "sqlite:test.db"))
(db/tables conn)
;; => ["users" "posts" "comments"]

;; カラム情報を取得
(db/columns conn "users")
;; => [{:name "id" :type "INTEGER" :nullable false :default nil :primary_key true}
;;     {:name "name" :type "TEXT" :nullable false :default nil :primary_key false}
;;     {:name "email" :type "TEXT" :nullable true :default nil :primary_key false}]

;; インデックス一覧を取得
(db/indexes conn "users")
;; => [{:name "idx_email" :table "users" :columns ["email"] :unique true}]

;; 外部キー一覧を取得
(db/foreign-keys conn "posts")
;; => [{:name "fk_posts_0" :table "posts" :columns ["user_id"]
;;      :referenced_table "users" :referenced_columns ["id"]}]
```

#### ✅ 2. 機能検出API

```qi
;; データベース機能のサポート確認
(db/supports? conn "transactions")      ;; => true
(db/supports? conn "stored_procedures") ;; => false (SQLite)

;; ドライバー情報を取得
(db/driver-info conn)
;; => {:name "sqlite" :version "0.32.0" :database_version "3.45.0"}
```

#### ✅ 3. ストアドプロシージャ/ファンクション呼び出し

```qi
;; db/call - ストアドプロシージャ/ファンクション呼び出し
(db/call conn "my_function" [arg1 arg2])
;; SQLiteでは未サポート（エラーを返す）
;; PostgreSQL/MySQLでは実装予定
```

#### ✅ 4. クエリ情報取得（Phase 2追加）

```qi
;; db/query-info - クエリのメタデータを取得（実行せずに）
(def conn (db/connect "sqlite:test.db"))
(def info (db/query-info conn "SELECT id, name FROM users WHERE age > ?"))
;; => {:columns [{:name "id" :type "UNKNOWN" :nullable true ...}
;;               {:name "name" :type "UNKNOWN" :nullable true ...}]
;;     :parameter_count 1}

;; カラム情報を取得
(def columns (get info :columns))
(println "カラム数:" (count columns))
(println "パラメータ数:" (get info :parameter_count))
```

#### ✅ 5. コネクションプーリング（Phase 3）

```qi
;; db/create-pool - コネクションプールを作成
(def pool (db/create-pool "sqlite:mydb.db" {} 10))  ; 最大10接続

;; db/pool-acquire - プールから接続を取得
(def conn (db/pool-acquire pool))

;; 接続を使用
(db/exec conn "INSERT INTO users (name) VALUES (?)" ["Alice"])
(def users (db/query conn "SELECT * FROM users" []))

;; db/pool-release - 接続をプールに返却
(db/pool-release pool conn)

;; db/pool-stats - プールの統計情報を取得
(def stats (db/pool-stats pool))
;; => {:available 9, :in_use 1, :max 10}

;; db/pool-close - プール全体をクローズ
(db/pool-close pool)
```

### 実装詳細

**変更したファイル**:
- `src/builtins/db.rs`:
  - DbConnection traitに以下のメソッドを追加:
    - `tables()`, `columns()`, `indexes()`, `foreign_keys()`
    - `call()`, `supports()`, `driver_info()`, `query_info()`
  - 対応するQiビルトイン関数を実装:
    - `native_tables()`, `native_columns()`, `native_indexes()`, `native_foreign_keys()`
    - `native_call()`, `native_supports()`, `native_driver_info()`, `native_query_info()`
  - DbPool構造体を追加（スレッドセーフなコネクションプーリング）:
    - `acquire()` - プールから接続を取得
    - `release()` - 接続をプールに返却
    - `close()` - プール全体をクローズ
    - `stats()` - プールの統計情報を取得
  - プール管理関数を実装:
    - `native_create_pool()`, `native_pool_acquire()`, `native_pool_release()`
    - `native_pool_close()`, `native_pool_stats()`

- `src/builtins/sqlite.rs`:
  - SQLiteドライバーで全メソッドを実装
  - `tables()`: sqlite_masterからテーブル一覧を取得
  - `columns()`: PRAGMA table_info()でカラム情報を取得
  - `indexes()`: PRAGMA index_list()とindex_info()でインデックス情報を取得
  - `foreign_keys()`: PRAGMA foreign_key_list()で外部キー情報を取得
  - `supports()`: SQLiteがサポートする機能を返却
  - `driver_info()`: rusqliteとSQLiteのバージョン情報を返却
  - `call()`: 未サポート（エラーを返す）
  - `query_info()`: プリペアドステートメントからカラム情報とパラメータ数を取得

- `src/builtins/mod.rs`:
  - 12個の新しい関数を登録（Phase 2: 7個 + Phase 3: 5個）

- `src/i18n.rs`:
  - プール関連のエラーメッセージキーを追加:
    - `DbExpectedPool`, `DbPoolNotFound`, `DbInvalidPoolSize`, `DbNeed1To3Args`

**サポートされる機能**:
- SQLiteで確認できる機能:
  - `"transactions"` → true
  - `"prepared_statements"` → true
  - `"blob"` → true
  - `"foreign_keys"` → true
  - `"stored_procedures"` → false
  - `"stored_functions"` → false

**メリット**:
- **Phase 2 (メタデータAPI)**:
  - データベーススキーマの動的な検査が可能
  - マイグレーションツールの実装が容易
  - データベース依存のコードを実行前に検出可能
- **Phase 2 (クエリ情報取得)**:
  - クエリを実行せずにカラム情報を取得可能
  - 動的なクエリビルダーの実装が容易
  - パラメータの数を事前に確認できる
- **Phase 3 (コネクションプーリング)**:
  - 複数の接続を再利用してパフォーマンス向上
  - 並行・並列処理に対応（スレッドセーフ）
  - 最大接続数の制限により、リソース枯渇を防止
  - 統計情報の取得により、プールの状態を監視可能

---

## 🚧 今後の実装予定機能

以下は、SPEC.mdから抽出した未実装機能のリストです。

### 優先度: 高

（現在なし）

---

### 優先度: 中

#### 1. 正規表現の拡張機能（Phase 1実装完了）

**✅ Phase 1 - 実装完了 (2025-01-13)**:
- `str/re-match-groups` - グループキャプチャ（番号付き）
- `str/re-split` - 正規表現による分割

**Phase 2（中優先度）- 未実装**:
- `str/re-match-all` - 全マッチとキャプチャグループの取得
- `str/re-compile` - パターンのプリコンパイル

**Phase 3（低優先度）- 未実装**:
- 名前付きキャプチャグループ (`:named`)
- `str/re-replace-fn` - コールバック置換

**実装済みの機能**:
```qi
;; str/re-match-groups - グループキャプチャ
(str/re-match-groups "(\\d{4})-(\\d{2})-(\\d{2})" "2025-01-13")
;=> {:match "2025-01-13" :groups ["2025" "01" "13"]}

;; str/re-split - 正規表現分割
(str/re-split "\\s+" "hello  world\t\ntest")
;=> ["hello" "world" "test"]

;; 分割数の制限
(str/re-split "\\s+" "a b c d e" 3)
;=> ["a" "b" "c d e"]
```

**関連**: SPEC.md line 5475-5479

---

### 優先度: 低

#### 4. flow DSL - 分岐・合流を含む複雑な流れ

**目的**: 複雑なデータフローを構造化して記述

**現状**:
- 基本的な`flow`は実装済み
- 分岐・合流を含む複雑な流れは未実装

**関連**: SPEC.md line 60, 6396

---

#### 5. その他の標準モジュール

**未実装のモジュール**:
- `http` - HTTPクライアント（基本機能は実装済み、拡張が必要）
- `json` - JSONパース（基本機能は実装済み）

**注**: リアルタイムログ監視は`cmd/stream-lines`と`tail -f`コマンドの組み合わせで実現可能

**関連**: SPEC.md line 6032-6037

---

#### 6. PostgreSQL / MySQL ドライバー

**目的**: 複数のデータベースに対応

**現状**: SQLiteのみ実装済み

**実装内容**:
- PostgreSQLドライバー
- MySQLドライバー
- Feature-gatedモジュール（条件付きコンパイル）

**関連**: SPEC.md line 6856-6863

---

### Phase 6以降（低優先度）

#### 7. 名前空間システム

**目的**: 大規模開発での名前衝突の回避

**現状**: グローバル名前空間のみ

**関連**: SPEC.md line 7208-7211

---

## メモ

- ✅ **既に実装済み**: モジュールシステム（use, defn-, export, module）、match orパターン、分解機能
- 実装の優先度は、HTTPストリーミングなどの実用的な機能を優先
- 標準ライブラリの拡張（HTTP、DB、正規表現）は中優先度
- パフォーマンス最適化やエコシステム機能は低優先度
