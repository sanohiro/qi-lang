# KVS統一インターフェース設計

## 概要

データベース統一インターフェース（`db/*`）と同様に、キーバリューストア（KVS）の統一インターフェースを提供します。

## 設計方針

### データベースとの類似性

```
データベース:
- db/connect, db/query, db/exec      ... SQLite統一インターフェース
- db/pg-query, db/pg-exec             ... PostgreSQL専用
- db/my-query, db/my-exec             ... MySQL専用

KVS:
- kvs/connect, kvs/get, kvs/set, ... ... 統一インターフェース（✅実装済み）
- kvs/redis-*                         ... Redis専用（統一IFで十分、現在不要）
- kvs/memcached-*                     ... Memcached専用（将来）
- kvs/dynamodb-*                      ... DynamoDB専用（将来）
```

**注**: 現在のRedis実装は統一インターフェースで全機能をカバーしているため、
Redis専用関数（`kvs/redis-*`）は公開していません。将来、統一インターフェースで
表現できないRedis固有機能（Pub/Sub、Lua scripting等）が必要になった場合のみ追加します。

## Phase 1: 統一インターフェース実装（✅完了）

### 接続 (1関数)

```qi
;; KVS接続（バックエンド自動判別）
(def kvs (kvs/connect "redis://localhost:6379"))
;; => "KvsConnection:1"
```

### 基本操作 (7関数)

```qi
;; 値の設定
(kvs/set kvs "key" "value")
;; => "OK" or {:error "message"}

;; 値の取得
(kvs/get kvs "key")
;; => "value" or nil

;; 削除
(kvs/delete kvs "key")
;; => 1 (削除数) or {:error "message"}

;; 存在確認
(kvs/exists? kvs "key")
;; => true or false

;; キー一覧（パターンマッチ）
(kvs/keys kvs "user:*")
;; => ["user:1" "user:2" ...]

;; 有効期限設定（秒）
(kvs/expire kvs "key" 3600)
;; => true or false

;; 残り時間取得
(kvs/ttl kvs "key")
;; => 3600 or -1 (期限なし) or -2 (存在しない)
```

### 数値操作 (2関数)

```qi
;; インクリメント
(kvs/incr kvs "counter")
;; => 1

;; デクリメント
(kvs/decr kvs "counter")
;; => 0
```

### リスト操作 (4関数)

```qi
;; リスト左端に追加
(kvs/lpush kvs "queue" "item1")
;; => 1 (リスト長)

;; リスト右端に追加
(kvs/rpush kvs "queue" "item2")
;; => 2

;; リスト左端から取得
(kvs/lpop kvs "queue")
;; => "item1"

;; リスト右端から取得
(kvs/rpop kvs "queue")
;; => "item2"
```

### ハッシュ操作 (3関数)

```qi
;; ハッシュフィールド設定
(kvs/hset kvs "user:1" "name" "Alice")
;; => true

;; ハッシュフィールド取得
(kvs/hget kvs "user:1" "name")
;; => "Alice"

;; ハッシュ全体取得
(kvs/hgetall kvs "user:1")
;; => {:name "Alice" :age "30"}
```

### セット操作 (2関数)

```qi
;; セットに追加
(kvs/sadd kvs "tags" "redis")
;; => 1 (追加数)

;; セット全メンバー取得
(kvs/smembers kvs "tags")
;; => ["redis" "cache" "kvs"]
```

## Phase 2: バックエンド拡張（将来）

将来的に、他のKVSバックエンドも同じ統一インターフェースで利用可能にする：

```qi
;; メモリベースKVS
(def kvs (kvs/connect ":memory:"))
(kvs/set kvs "key" "value")
(kvs/get kvs "key")

;; ファイルベースKVS
(def kvs (kvs/connect "file:///tmp/kvs.db"))

;; Memcached
(def kvs (kvs/connect "memcached://localhost:11211"))
(kvs/set kvs "key" "value")
```

## Phase 3: Redis固有機能（必要な場合のみ）

現在の統一インターフェースで表現できないRedis固有機能が必要になった場合のみ、
専用関数を追加する：

- **Pub/Sub**: `kvs/redis-publish`, `kvs/redis-subscribe`
- **Lua Scripting**: `kvs/redis-eval`
- **トランザクション**: `kvs/redis-multi`, `kvs/redis-exec`
- **Sorted Sets**: `kvs/redis-zadd`, `kvs/redis-zrange`

**注**: 現時点ではこれらは実装していない（基本的なKVS操作のみで十分）

## Feature Flags

```toml
[features]
kvs-redis = ["dep:redis", "dep:tokio"]
# kvs-memcached = ["dep:memcache"]  # 将来
# kvs-inmemory = []  # 将来（Pure Rust、依存なし）
```

## エラーハンドリング

データベースと同じパターン：
- 成功時: 生データ（文字列、数値、真偽値、nil）
- 失敗時: `{:error "message"}`

## 依存クレート

- **redis**: Pure Rust実装、async対応
- **memcache**: Pure Rust実装（将来）
- **rusoto_dynamodb**: AWS SDK（将来、C依存）
