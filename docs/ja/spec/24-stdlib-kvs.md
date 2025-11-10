# KVS（Key-Value Store）

**Redis統一インターフェース - キャッシュ、セッション管理、キュー**

Qiは、Key-Value Storeへの統一的なアクセスを提供します。現在はRedisをサポートし、将来的にMemcached等のバックエンドにも対応予定です。

---

## 目次

- [概要](#概要)
- [統一インターフェース設計](#統一インターフェース設計)
- [kvs/connect - 接続](#kvsconnect---接続)
- [基本操作](#基本操作)
- [数値操作](#数値操作)
- [リスト操作](#リスト操作)
- [ハッシュ操作](#ハッシュ操作)
- [セット操作](#セット操作)
- [複数操作（バッチ操作）](#複数操作バッチ操作)
- [実用例](#実用例)
- [エラー処理](#エラー処理)
- [パフォーマンス](#パフォーマンス)
- [実装の詳細](#実装の詳細)

---

## 概要

### 提供機能

**対応バックエンド**:
- **Redis**: キャッシュ、セッション管理、キュー、Pub/Sub
  - 統一インターフェース（`kvs/*`）
  - 基本操作（get/set/del）
  - 数値操作（incr/decr）
  - データ構造（リスト、ハッシュ、セット）
  - 複数操作（mget/mset）
  - 有効期限設定（expire/ttl）

**統一インターフェースの利点**:
- バックエンド自動判別（URL解析）
- バックエンド透過的な切り替え（接続URLのみ変更）
- シンプルで一貫したAPI
- 将来的な拡張性（Memcached、インメモリKVS等）

### feature flag

```toml
# Cargo.toml
features = ["kvs-redis"]
```

デフォルトで有効です。

### 依存クレート

- **redis** (v0.27) - Pure Rust Redisクライアント
- **tokio** - 非同期ランタイム
- **dashmap** - 接続プール（スレッドセーフなHashMap）

---

## 統一インターフェース設計

データベースの`db/connect`と同じパターンで、KVSバックエンドを透過的に扱えます。

```qi
;; バックエンドはURLから自動判別
(def kvs (kvs/connect "redis://localhost:6379"))

;; 以降のコードはバックエンド非依存
(kvs/set kvs "key" "value")
(kvs/get kvs "key")

;; バックエンドを変更する場合も、接続URLだけ変えればOK
;; (def kvs (kvs/connect "memcached://localhost:11211"))  ;; 将来対応
```

**設計原則**:
- **統一インターフェース**: `kvs/*`関数のみ公開
- **専用関数は非公開**: `kvs/redis-*`は内部ドライバー用
- **拡張性**: 統一IFで表現できない機能（Redis Pub/Sub等）が必要になった場合のみ専用関数を追加

---

## kvs/connect - 接続

**KVSに接続し、接続オブジェクトを返します。**

```qi
(kvs/connect url)
```

### 引数

- `url`: 文字列（接続URL）
  - Redis: `"redis://localhost:6379"`
  - Redis（認証付き）: `"redis://:password@localhost:6379"`
  - Memcached: `"memcached://localhost:11211"` （将来対応）

### 戻り値

- 接続ID（文字列） - 形式: `"KvsConnection:kvs:1"`
- エラーの場合: `{:error "message"}`

### 使用例

```qi
;; Redis接続
(def kvs (kvs/connect "redis://localhost:6379"))
;; => "KvsConnection:kvs:1"

;; 認証付きRedis
(def kvs (kvs/connect "redis://:my-secret-password@localhost:6379"))

;; 接続エラーの場合
(def kvs (kvs/connect "invalid-url"))
;; => {:error "Unsupported KVS URL: invalid-url"}
```

### 接続管理

- **自動接続プール**: 同じURLへの複数接続は内部で共有
- **自動再接続**: 接続断時は自動で再接続を試行
- **スレッドセーフ**: 複数のスレッドから同時アクセス可能

---

## 基本操作

### kvs/set - 値の設定

キーに値を設定します。

```qi
(kvs/set conn key value)
```

**引数**:
- `conn`: 接続ID（`kvs/connect`の戻り値）
- `key`: キー名（文字列）
- `value`: 値（文字列、整数、浮動小数点数、真偽値）

**戻り値**:
- `"OK"` (成功時)
- `{:error "message"}` (失敗時)

**例**:
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

### kvs/get - 値の取得

キーの値を取得します。

```qi
(kvs/get conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）

**戻り値**:
- 値（文字列） - キーが存在する場合
- `nil` - キーが存在しない場合
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/get kvs "user:1")
;; => "Alice"

(kvs/get kvs "nonexistent")
;; => nil

;; エラー処理
(def value (kvs/get kvs "user:1"))
(if (nil? value)
  (println "Key not found")
  (println "Value:" value))
```

---

### kvs/del - キーの削除

キーを削除します（関数名は`kvs/delete`）。

```qi
(kvs/del conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）

**戻り値**:
- 削除されたキー数（整数） - 通常は`1`、キーが存在しない場合は`0`
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/del kvs "user:1")
;; => 1  ;; 削除成功

(kvs/del kvs "nonexistent")
;; => 0  ;; 削除対象なし
```

---

### kvs/exists - 存在確認

キーが存在するかチェックします（関数名は`kvs/exists?`）。

```qi
(kvs/exists conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）

**戻り値**:
- `true` - キーが存在する
- `false` - キーが存在しない
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/set kvs "user:1" "Alice")
(kvs/exists kvs "user:1")
;; => true

(kvs/exists kvs "nonexistent")
;; => false

;; 条件分岐
(if (kvs/exists kvs "cache:data")
  (kvs/get kvs "cache:data")
  (do
    (def data (fetch-from-db))
    (kvs/set kvs "cache:data" data)
    data))
```

---

### kvs/keys - パターンマッチ

パターンにマッチするキー一覧を取得します。

```qi
(kvs/keys conn pattern)
```

**引数**:
- `conn`: 接続ID
- `pattern`: パターン文字列
  - `*` - 任意の文字列
  - `?` - 1文字
  - 例: `"user:*"`, `"cache:2025-*"`, `"session:???"`

**戻り値**:
- キー名のベクタ
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/set kvs "user:1" "Alice")
(kvs/set kvs "user:2" "Bob")
(kvs/set kvs "user:3" "Charlie")

(kvs/keys kvs "user:*")
;; => ["user:1" "user:2" "user:3"]

(kvs/keys kvs "user:?")
;; => ["user:1" "user:2" "user:3"]

(kvs/keys kvs "cache:*")
;; => []  ;; マッチなし
```

**注意**: 本番環境で大量のキーがある場合、`kvs/keys`はブロッキング操作のためパフォーマンスに影響します。代わりにSCAN系コマンドの使用を検討してください（将来実装予定）。

---

### kvs/expire - 有効期限設定

キーに有効期限を設定します（秒単位）。

```qi
(kvs/expire conn key seconds)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）
- `seconds`: 有効期限（整数、秒）

**戻り値**:
- `true` - 有効期限設定成功
- `false` - キーが存在しない
- `{:error "message"}` (失敗時)

**例**:
```qi
;; セッションに30分の有効期限
(kvs/set kvs "session:abc123" "user-data")
(kvs/expire kvs "session:abc123" 1800)  ;; 30分 = 1800秒
;; => true

;; 1時間の有効期限
(kvs/expire kvs "cache:data" 3600)
;; => true

;; 存在しないキー
(kvs/expire kvs "nonexistent" 60)
;; => false
```

---

### kvs/ttl - 残り有効期限取得

キーの残り有効期限を取得します（秒単位）。

```qi
(kvs/ttl conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）

**戻り値**:
- 残り秒数（整数） - 有効期限が設定されている場合
- `-1` - キーは存在するが、有効期限が設定されていない
- `-2` - キーが存在しない
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/set kvs "session:abc" "data")
(kvs/expire kvs "session:abc" 3600)

(kvs/ttl kvs "session:abc")
;; => 3599  ;; 残り約1時間

;; 有効期限なし
(kvs/set kvs "permanent" "data")
(kvs/ttl kvs "permanent")
;; => -1

;; 存在しないキー
(kvs/ttl kvs "nonexistent")
;; => -2
```

---

## 数値操作

### kvs/incr - インクリメント

キーの値を1増やします。キーが存在しない場合は0から開始します。

```qi
(kvs/incr conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）

**戻り値**:
- インクリメント後の値（整数）
- `{:error "message"}` (失敗時)

**例**:
```qi
;; 初回（0から開始）
(kvs/incr kvs "page-views")
;; => 1

(kvs/incr kvs "page-views")
;; => 2

(kvs/incr kvs "page-views")
;; => 3

;; 既存の値から増加
(kvs/set kvs "counter" 100)
(kvs/incr kvs "counter")
;; => 101
```

**用途**: ページビューカウンター、ユーザーIDジェネレーター、レート制限等

---

### kvs/decr - デクリメント

キーの値を1減らします。キーが存在しない場合は0から開始します。

```qi
(kvs/decr conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: キー名（文字列）

**戻り値**:
- デクリメント後の値（整数）
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/set kvs "stock" 10)
(kvs/decr kvs "stock")
;; => 9

(kvs/decr kvs "stock")
;; => 8

;; 初回（0から開始）
(kvs/decr kvs "countdown")
;; => -1
```

**用途**: 在庫管理、ダウンカウンター等

---

## リスト操作

Redisのリストは、両端キュー（deque）として動作します。FIFO（キュー）、LIFO（スタック）両方の用途で使用できます。

### kvs/lpush - 左端に要素追加

リストの左端（先頭）に要素を追加します。

```qi
(kvs/lpush conn key value)
```

**引数**:
- `conn`: 接続ID
- `key`: リストのキー名（文字列）
- `value`: 追加する値（文字列、整数、浮動小数点数、真偽値）

**戻り値**:
- リストの長さ（整数）
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/lpush kvs "mylist" "first")
;; => 1

(kvs/lpush kvs "mylist" "second")
;; => 2

;; 結果: ["second" "first"]
```

---

### kvs/rpush - 右端に要素追加

リストの右端（末尾）に要素を追加します。

```qi
(kvs/rpush conn key value)
```

**引数**:
- `conn`: 接続ID
- `key`: リストのキー名（文字列）
- `value`: 追加する値（文字列、整数、浮動小数点数、真偽値）

**戻り値**:
- リストの長さ（整数）
- `{:error "message"}` (失敗時)

**例（キュー - FIFO）**:
```qi
;; タスクを追加
(kvs/rpush kvs "tasks" "task1")
;; => 1
(kvs/rpush kvs "tasks" "task2")
;; => 2
(kvs/rpush kvs "tasks" "task3")
;; => 3

;; タスクを取得（先頭から）
(kvs/lpop kvs "tasks")  ;; => "task1"
(kvs/lpop kvs "tasks")  ;; => "task2"
(kvs/lpop kvs "tasks")  ;; => "task3"
```

**例（スタック - LIFO）**:
```qi
;; 要素を積む
(kvs/lpush kvs "stack" "item1")
(kvs/lpush kvs "stack" "item2")
(kvs/lpush kvs "stack" "item3")

;; 要素を取る（最後に追加したものから）
(kvs/lpop kvs "stack")  ;; => "item3"
(kvs/lpop kvs "stack")  ;; => "item2"
(kvs/lpop kvs "stack")  ;; => "item1"
```

---

### kvs/lpop - 左端から要素取得

リストの左端（先頭）から要素を取得し、削除します。

```qi
(kvs/lpop conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: リストのキー名（文字列）

**戻り値**:
- 取得した値（文字列）
- `nil` - リストが空、またはキーが存在しない
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/rpush kvs "queue" "task1")
(kvs/rpush kvs "queue" "task2")

(kvs/lpop kvs "queue")
;; => "task1"

(kvs/lpop kvs "queue")
;; => "task2"

(kvs/lpop kvs "queue")
;; => nil  ;; 空
```

---

### kvs/rpop - 右端から要素取得

リストの右端（末尾）から要素を取得し、削除します。

```qi
(kvs/rpop conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: リストのキー名（文字列）

**戻り値**:
- 取得した値（文字列）
- `nil` - リストが空、またはキーが存在しない
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/lpush kvs "stack" "a")
(kvs/lpush kvs "stack" "b")
(kvs/lpush kvs "stack" "c")

(kvs/rpop kvs "stack")
;; => "a"  ;; 最初に追加した要素
```

---

### kvs/lrange - 範囲取得

リストの指定範囲の要素を取得します。

```qi
(kvs/lrange conn key start stop)
```

**引数**:
- `conn`: 接続ID
- `key`: リストのキー名（文字列）
- `start`: 開始インデックス（整数、0始まり）
- `stop`: 終了インデックス（整数、-1で末尾まで）

**戻り値**:
- 要素のベクタ
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/rpush kvs "mylist" "a")
(kvs/rpush kvs "mylist" "b")
(kvs/rpush kvs "mylist" "c")
(kvs/rpush kvs "mylist" "d")
(kvs/rpush kvs "mylist" "e")

;; 全要素取得
(kvs/lrange kvs "mylist" 0 -1)
;; => ["a" "b" "c" "d" "e"]

;; 最初の3要素
(kvs/lrange kvs "mylist" 0 2)
;; => ["a" "b" "c"]

;; 2番目から4番目
(kvs/lrange kvs "mylist" 1 3)
;; => ["b" "c" "d"]

;; 最後の2要素
(kvs/lrange kvs "mylist" -2 -1)
;; => ["d" "e"]
```

**用途**: ページング、履歴表示、最新N件取得等

---

## ハッシュ操作

Redisのハッシュは、フィールド-値のペアを持つマップです。ユーザー情報、設定値等の構造化データの保存に適しています。

### kvs/hset - フィールド設定

ハッシュのフィールドに値を設定します。

```qi
(kvs/hset conn key field value)
```

**引数**:
- `conn`: 接続ID
- `key`: ハッシュのキー名（文字列）
- `field`: フィールド名（文字列）
- `value`: 値（文字列、整数、浮動小数点数、真偽値）

**戻り値**:
- `true` - 新規フィールドを作成
- `false` - 既存フィールドを更新
- `{:error "message"}` (失敗時)

**例**:
```qi
;; ユーザー情報を保存
(kvs/hset kvs "user:1" "name" "Alice")
;; => true  ;; 新規作成

(kvs/hset kvs "user:1" "email" "alice@example.com")
;; => true

(kvs/hset kvs "user:1" "age" 30)
;; => true

;; 既存フィールドを更新
(kvs/hset kvs "user:1" "name" "Alice Smith")
;; => false  ;; 更新
```

---

### kvs/hget - フィールド取得

ハッシュのフィールドから値を取得します。

```qi
(kvs/hget conn key field)
```

**引数**:
- `conn`: 接続ID
- `key`: ハッシュのキー名（文字列）
- `field`: フィールド名（文字列）

**戻り値**:
- 値（文字列）
- `nil` - フィールドが存在しない
- `{:error "message"}` (失敗時)

**例**:
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

### kvs/hgetall - ハッシュ全体取得

ハッシュの全フィールドと値を取得します。

```qi
(kvs/hgetall conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: ハッシュのキー名（文字列）

**戻り値**:
- マップ（フィールド-値のペア）
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/hset kvs "user:1" "name" "Alice")
(kvs/hset kvs "user:1" "email" "alice@example.com")
(kvs/hset kvs "user:1" "age" "30")

(kvs/hgetall kvs "user:1")
;; => {"name" "Alice" "email" "alice@example.com" "age" "30"}

;; マップ操作
(def user (kvs/hgetall kvs "user:1"))
(get user "name")
;; => "Alice"
```

**注意**: フィールド数が多い場合はパフォーマンスに影響します。代わりに`kvs/hget`で必要なフィールドのみ取得することを検討してください。

---

## セット操作

Redisのセットは、重複のない文字列の集合です。タグ、ユニークな値の管理等に適しています。

### kvs/sadd - メンバー追加

セットにメンバーを追加します。

```qi
(kvs/sadd conn key member)
```

**引数**:
- `conn`: 接続ID
- `key`: セットのキー名（文字列）
- `member`: メンバー（文字列、整数、浮動小数点数、真偽値）

**戻り値**:
- 追加されたメンバー数（整数） - 通常は`1`、既存の場合は`0`
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/sadd kvs "tags" "redis")
;; => 1  ;; 新規追加

(kvs/sadd kvs "tags" "cache")
;; => 1

(kvs/sadd kvs "tags" "nosql")
;; => 1

;; 既存のメンバー
(kvs/sadd kvs "tags" "redis")
;; => 0  ;; 追加されない（重複）
```

---

### kvs/smembers - 全メンバー取得

セットの全メンバーを取得します。

```qi
(kvs/smembers conn key)
```

**引数**:
- `conn`: 接続ID
- `key`: セットのキー名（文字列）

**戻り値**:
- メンバーのベクタ（順序は不定）
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/sadd kvs "tags" "redis")
(kvs/sadd kvs "tags" "cache")
(kvs/sadd kvs "tags" "nosql")

(kvs/smembers kvs "tags")
;; => ["redis" "cache" "nosql"]  ;; 順序は不定

;; 空のセット
(kvs/smembers kvs "nonexistent")
;; => []
```

**用途**: タグ一覧、ユーザーの権限リスト、ユニークな訪問者等

---

## 複数操作（バッチ操作）

### kvs/mget - 複数キー一括取得

複数のキーの値を一括で取得します。

```qi
(kvs/mget conn keys)
```

**引数**:
- `conn`: 接続ID
- `keys`: キー名のベクタ

**戻り値**:
- 値のベクタ（存在しないキーは`nil`）
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/set kvs "key1" "value1")
(kvs/set kvs "key2" "value2")
(kvs/set kvs "key3" "value3")

(kvs/mget kvs ["key1" "key2" "key3"])
;; => ["value1" "value2" "value3"]

(kvs/mget kvs ["key1" "nonexistent" "key3"])
;; => ["value1" nil "value3"]

;; 空のベクタ
(kvs/mget kvs [])
;; => []
```

**パフォーマンスメリット**:
- 複数のキーを1回のネットワーク往復で取得
- ループ処理より高速

```qi
;; ❌ 非効率（3回のネットワーク往復）
(def v1 (kvs/get kvs "key1"))
(def v2 (kvs/get kvs "key2"))
(def v3 (kvs/get kvs "key3"))

;; ✅ 効率的（1回のネットワーク往復）
(def [v1 v2 v3] (kvs/mget kvs ["key1" "key2" "key3"]))
```

---

### kvs/mset - 複数キー一括設定

複数のキーと値を一括で設定します。

```qi
(kvs/mset conn pairs)
```

**引数**:
- `conn`: 接続ID
- `pairs`: キーと値のマップ

**戻り値**:
- `"OK"` (成功時)
- `{:error "message"}` (失敗時)

**例**:
```qi
(kvs/mset kvs {"key1" "value1" "key2" "value2" "key3" "value3"})
;; => "OK"

;; 大量のキーを一度に設定
(kvs/mset kvs {
  "user:1" "Alice"
  "user:2" "Bob"
  "user:3" "Charlie"
  "cache:1" "data1"
  "cache:2" "data2"
})
;; => "OK"
```

**パフォーマンスメリット**:
- 複数のキーを1回のネットワーク往復で設定
- ループ処理より高速
- アトミックに実行される（全て成功するか、全て失敗する）

```qi
;; ❌ 非効率（3回のネットワーク往復）
(kvs/set kvs "key1" "value1")
(kvs/set kvs "key2" "value2")
(kvs/set kvs "key3" "value3")

;; ✅ 効率的（1回のネットワーク往復）
(kvs/mset kvs {"key1" "value1" "key2" "value2" "key3" "value3"})
```

---

## 実用例

### セッションキャッシュ

Webアプリケーションのセッション管理にKVSを使用します。

```qi
(def kvs (kvs/connect "redis://localhost:6379"))

;; セッション作成
(defn create-session [user-id]
  (def session-id (str "session:" user-id))
  (def session-data (json/stringify {
    :user_id user-id
    :created_at (now)
    :ip_address "192.168.1.1"
  }))
  (kvs/set kvs session-id session-data)
  (kvs/expire kvs session-id 1800)  ;; 30分の有効期限
  session-id)

;; セッション取得
(defn get-session [session-id]
  (kvs/get kvs session-id)
  |> (fn [data]
       (if (nil? data)
         {:error "Session not found or expired"}
         (json/parse data))))

;; セッション削除（ログアウト）
(defn destroy-session [session-id]
  (kvs/del kvs session-id))

;; 使用例
(def sid (create-session 42))
;; => "session:42"

(get-session sid)
;; => {:user_id 42 :created_at "2025-01-22T10:30:00Z" :ip_address "192.168.1.1"}

;; 30分後
(get-session sid)
;; => {:error "Session not found or expired"}
```

---

### ページビューカウンター

Webページのアクセス数を記録します。

```qi
;; ページビュー記録
(defn track-page-view [page-url]
  (kvs/incr kvs (str "page-views:" page-url)))

;; ページビュー取得
(defn get-page-views [page-url]
  (def count-str (kvs/get kvs (str "page-views:" page-url)))
  (if (nil? count-str)
    0
    (parse-int count-str)))

;; 人気ページランキング
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

;; 使用例
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

### タスクキュー（ジョブキュー）

バックグラウンドタスクをキューで管理します。

```qi
;; タスク追加
(defn enqueue-task [task-type task-data]
  (def task (json/stringify {
    :type task-type
    :data task-data
    :created_at (now)
  }))
  (kvs/rpush kvs "task-queue" task))

;; タスク取得
(defn dequeue-task []
  (kvs/lpop kvs "task-queue")
  |> (fn [data]
       (if (nil? data)
         nil
         (json/parse data))))

;; タスクワーカー
(defn process-tasks []
  (loop []
    (def task (dequeue-task))
    (if (nil? task)
      (do
        (println "No tasks, waiting...")
        (sleep 1000)  ;; 1秒待機
        (recur))
      (do
        (println "Processing task:" task)
        (match (get task :type)
          "send-email" -> (send-email (get task :data))
          "generate-report" -> (generate-report (get task :data))
          _ -> (println "Unknown task type"))
        (recur)))))

;; 使用例
(enqueue-task "send-email" {:to "user@example.com" :subject "Hello"})
(enqueue-task "generate-report" {:user_id 42})

(dequeue-task)
;; => {:type "send-email" :data {:to "user@example.com" :subject "Hello"} :created_at "..."}

(dequeue-task)
;; => {:type "generate-report" :data {:user_id 42} :created_at "..."}

(dequeue-task)
;; => nil  ;; キューが空
```

---

### レート制限（Rate Limiting）

APIのレート制限を実装します。

```qi
;; レート制限チェック（1分間に10リクエストまで）
(defn check-rate-limit [user-id]
  (def key (str "rate-limit:" user-id))
  (def count (kvs/incr kvs key))

  ;; 初回の場合、1分間の有効期限を設定
  (if (= count 1)
    (kvs/expire kvs key 60))

  ;; 10を超えたら制限
  (if (> count 10)
    {:allowed false :remaining 0}
    {:allowed true :remaining (- 10 count)}))

;; 使用例
(check-rate-limit 42)
;; => {:allowed true :remaining 9}

;; 10回リクエスト後
(check-rate-limit 42)
;; => {:allowed true :remaining 0}

;; 11回目
(check-rate-limit 42)
;; => {:allowed false :remaining 0}

;; 1分後（TTL切れ）
(check-rate-limit 42)
;; => {:allowed true :remaining 9}
```

---

### キャッシュ（データベースクエリ結果）

データベースクエリ結果をキャッシュして高速化します。

```qi
;; キャッシュ付きデータ取得
(defn get-user-with-cache [user-id]
  (def cache-key (str "cache:user:" user-id))

  ;; キャッシュを確認
  (def cached (kvs/get kvs cache-key))
  (if (not (nil? cached))
    (do
      (println "Cache hit!")
      (json/parse cached))
    (do
      (println "Cache miss, fetching from DB...")
      ;; DBから取得
      (def user (db/query db-conn "SELECT * FROM users WHERE id = $1" [user-id])
                 |> first)
      ;; キャッシュに保存（5分間）
      (kvs/set kvs cache-key (json/stringify user))
      (kvs/expire kvs cache-key 300)
      user)))

;; キャッシュ無効化
(defn invalidate-user-cache [user-id]
  (kvs/del kvs (str "cache:user:" user-id)))

;; 使用例
(get-user-with-cache 1)
;; => Cache miss, fetching from DB...
;; => {:id 1 :name "Alice" ...}

(get-user-with-cache 1)
;; => Cache hit!
;; => {:id 1 :name "Alice" ...}

;; ユーザー更新時にキャッシュを削除
(update-user 1 "Alice Smith")
(invalidate-user-cache 1)
```

---

## エラー処理

### エラーハンドリング

KVS関数は成功時に生データ、失敗時に`{:error "message"}`を返します。

```qi
;; 基本的なエラー処理
(def result (kvs/get kvs "user:1"))
(if (error? result)
  (println "Error:" (get result :error))
  (println "Value:" result))

;; パイプラインでのエラー処理（|>?でショートサーキット）
(defn get-cached-data [key]
  (kvs/get kvs key)
  |>? (fn [data]
        (if (nil? data)
          {:error "Cache miss"}
          (json/parse data))))

;; matchでのエラー処理
(match (kvs/get kvs "user:1")
  {:error e} -> (println "KVS error:" e)
  nil -> (println "Key not found")
  value -> (println "Value:" value))
```

---

### 接続エラー

```qi
;; 不正な接続文字列
(def kvs (kvs/connect "invalid-url"))
;; => {:error "Unsupported KVS URL: invalid-url"}

;; 接続タイムアウト
(def kvs (kvs/connect "redis://localhost:9999"))
;; => {:error "Connection error: ..."}
```

---

### 操作エラー

```qi
;; 型エラー（文字列以外のキー）
(kvs/get kvs 123)
;; => Error: kvs/get (key) expects strings

;; 不正な引数数
(kvs/set kvs "key")
;; => Error: kvs/set expects 3 arguments
```

---

## パフォーマンス

### 接続プール

Qiのkvs実装は自動的に接続をプールします。同じURLへの複数の`kvs/connect`呼び出しは、内部で同じ接続を共有します。

```qi
;; これらは内部で同じ接続を共有
(def kvs1 (kvs/connect "redis://localhost:6379"))
(def kvs2 (kvs/connect "redis://localhost:6379"))
```

---

### 自動再接続

接続が切断された場合、自動的に再接続を試行します。

```rust
// Rust実装（参考）
async fn execute_with_retry<T, F, Fut>(url: &str, operation: F) -> redis::RedisResult<T>
where
    F: Fn(MultiplexedConnection) -> Fut,
    Fut: std::future::Future<Output = redis::RedisResult<T>>,
{
    // 最初の試行
    let conn = get_or_create_connection(url).await?;
    let result = operation(conn).await;

    // エラーの場合、再接続して再試行
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

### バッチ操作の推奨

複数のキーを扱う場合は、`mget`/`mset`を使用してください。

```qi
;; ❌ 非効率（N回のネットワーク往復）
(def keys ["key1" "key2" "key3" "key4" "key5"])
(map (fn [k] (kvs/get kvs k)) keys)

;; ✅ 効率的（1回のネットワーク往復）
(kvs/mget kvs ["key1" "key2" "key3" "key4" "key5"])
```

---

## 実装の詳細

### 統一インターフェース設計

KVSは**統一インターフェース**パターンで設計されています。これはGoの`database/sql`パッケージと同じアプローチで、バックエンド（ドライバー）を透過的に扱えます。

```
統一インターフェース（ユーザーが使う）:
- kvs/connect, kvs/get, kvs/set, kvs/del, ...

内部ドライバー（公開しない）:
- RedisDriver, MemcachedDriver（将来）
```

**設計原則**:
- **バックエンド自動判別**: URLから接続先を判定（`redis://`, `memcached://`等）
- **専用関数は非公開**: `kvs/redis-*`は内部ドライバー用のみ
- **拡張性**: 統一IFで表現できない機能（Redis Pub/Sub、Lua scripting等）が必要になった場合のみ専用関数を追加

---

### ドライバーパターン

```rust
// KVSドライバートレイト（統一インターフェース）
pub trait KvsDriver: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn set(&self, key: &str, value: &str) -> Result<String, String>;
    fn delete(&self, key: &str) -> Result<i64, String>;
    fn exists(&self, key: &str) -> Result<bool, String>;
    fn keys(&self, pattern: &str) -> Result<Vec<String>, String>;
    fn expire(&self, key: &str, seconds: i64) -> Result<bool, String>;
    fn ttl(&self, key: &str) -> Result<i64, String>;
    // ... 他のメソッド
}

// Redisドライバー（内部実装）
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
        // ... エラー変換
    }
    // ... 他のメソッド実装
}
```

---

### 非同期処理

内部的には非同期APIを使用していますが、Qiのユーザーには同期的なAPIとして公開されています。

```rust
// Rustでの実装（参考）
use tokio::runtime::Runtime;

static TOKIO_RT: LazyLock<Runtime> =
    LazyLock::new(|| Runtime::new().expect("Failed to create tokio runtime for Redis"));

pub fn native_redis_get(args: &[Value]) -> Result<Value, String> {
    // ... 引数チェック

    // 非同期処理を同期的に実行
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

### 接続管理

```rust
use dashmap::DashMap;
use redis::aio::MultiplexedConnection;
use std::sync::LazyLock;

// Redis接続プール（URL → Connection のマッピング）
static REDIS_POOL: LazyLock<DashMap<String, MultiplexedConnection>> =
    LazyLock::new(DashMap::new);

// 接続を取得または新規作成
async fn get_or_create_connection(url: &str) -> Result<MultiplexedConnection, String> {
    // 既存の接続を取得
    if let Some(conn) = REDIS_POOL.get(url) {
        return Ok(conn.clone());
    }

    // 新規接続を作成
    let client = Client::open(url)?;
    let conn = client.get_multiplexed_async_connection().await?;

    // プールに保存
    REDIS_POOL.insert(url.to_string(), conn.clone());

    Ok(conn)
}
```

---

## ロードマップ

### 将来的に実装予定の機能

**バックエンド拡張**:
- **Memcached対応**: `kvs/connect "memcached://localhost:11211"`
- **インメモリKVS**: `kvs/connect ":memory:"` （Pure Rust、依存なし、テスト用）

**専用関数の追加**（統一IFで表現できない場合のみ）:
- **Redis Pub/Sub**: `kvs/subscribe`, `kvs/publish`
- **Redis Lua scripting**: `kvs/eval`, `kvs/evalsha`
- **Redis Sorted Sets**: `kvs/zadd`, `kvs/zrange`
- **Redis Streams**: `kvs/xadd`, `kvs/xread`
- **Redis HyperLogLog**: `kvs/pfadd`, `kvs/pfcount`

**パフォーマンス最適化**:
- **パイプライン**: 複数コマンドを1回のネットワーク往復で実行
- **トランザクション**: `kvs/multi`, `kvs/exec`
- **ストリーミングSCAN**: `kvs/scan`（大量キー対応）

---

## まとめ

Qiのkey-value storeライブラリは、統一インターフェースでシンプルかつ安全なアクセスを提供します。

### 主な機能

- **kvs/connect**: Redis自動接続（将来Memcached等も）
- **基本操作**: get/set/del/exists/keys/expire/ttl
- **数値操作**: incr/decr
- **リスト**: lpush/rpush/lpop/rpop/lrange（キュー、スタック）
- **ハッシュ**: hset/hget/hgetall（構造化データ）
- **セット**: sadd/smembers（ユニークな値の集合）
- **複数操作**: mget/mset（バッチ処理）
- **自動再接続**: 接続断時の自動リトライ
- **接続プール**: 効率的な接続管理
- **バックエンド透過的切り替え**: 接続URLのみ変更

これらの機能を組み合わせることで、キャッシュ、セッション管理、タスクキュー、レート制限等の実装が簡単にできます。

---

## 関連ドキュメント

- **[17-stdlib-database.md](17-stdlib-database.md)** - データベース統一インターフェース
- **[11-stdlib-http.md](11-stdlib-http.md)** - WebアプリケーションでのKVS使用
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSONデータのシリアライズ
- **[08-error-handling.md](08-error-handling.md)** - エラー処理パターン
