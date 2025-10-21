# Qi言語ロードマップ

**未実装機能と将来の計画**

このドキュメントでは、Qi言語の未実装機能と将来の計画をまとめています。実装済み機能については `docs/spec/` ディレクトリを参照してください。

---

## 📋 優先度高（次期実装予定）

### APIサーバー・アプリケーション開発機能

#### 1. HTTPサーバー拡張 🔥

**WebSocket対応**:
```qi
;; WebSocketサーバー
(def ws-handler
  (fn [conn]
    (ws/on-message conn (fn [msg] (ws/send conn (process msg))))
    (ws/on-close conn (fn [] (log "client disconnected")))))

(server/serve ws-handler {:port 3000 :ws true})
```

#### 2. テストフレームワーク ✅ **実装済み**

**基本機能は実装完了。`qi test`コマンドで実行可能。**

```qi
;; tests/core_test.qi
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))))

(test/run "exception test" (fn []
  (test/assert-throws (fn [] (/ 10 0)))))
```

```bash
$ qi test
running 2 test files

テスト結果:
===========
  ✓ addition
  ✓ exception test

2 テスト, 2 成功, 0 失敗

finished in 0.05s
```

**実装済み機能:**
- ✅ `test/run` - テスト実行
- ✅ `test/assert-eq` - 等価性アサーション
- ✅ `test/assert` - 真偽値アサーション
- ✅ `test/assert-not` - 偽値アサーション
- ✅ `test/assert-throws` - 例外アサーション
- ✅ `qi test` コマンド - tests/ディレクトリの自動検出・実行
- ✅ Rust風のシンプルな出力形式

**今後の拡張予定（優先度低）:**
- カバレッジ計測 (`test/coverage`)
- `deftest`マクロ（糖衣構文）
- タグによるフィルタリング
- watch モード

#### 3. データベース接続 🎯

**PostgreSQL/MySQL対応** (SQLiteは実装済み):

```qi
;; PostgreSQL接続
(def conn (db/connect "postgresql://user:pass@localhost/mydb"))

;; コネクションプール
(def pool (db/pool "postgresql://..." {:max-connections 10}))
(db/with-connection pool
  (fn [conn]
    (db/query conn "SELECT * FROM users WHERE age > ?" [18])))

;; トランザクション管理
(db/transaction conn
  (fn [tx]
    (db/exec tx "INSERT INTO users (name) VALUES (?)" ["Alice"])
    (db/exec tx "UPDATE stats SET count = count + 1")
    ;; エラー時は自動ロールバック
    ))

;; ORM機能（オプション）
(db/defmodel User
  {:table "users"
   :fields [:id :name :email :created_at]})

(User/find-by-email "alice@example.com")
(User/create {:name "Bob" :email "bob@example.com"})
```

#### 4. 認証・認可 🎯

```qi
;; JWT生成・検証
(def token (jwt/sign {:user-id 123} "secret-key"))
(jwt/verify token "secret-key")
;; => {:ok {:user-id 123}}

;; セッション管理
(def session (session/create {:user-id 123}))
(session/get session :user-id)
;; => 123

;; OAuth2対応
(def oauth-config
  {:provider :google
   :client-id "..."
   :client-secret "..."
   :redirect-uri "http://localhost:3000/callback"})

(oauth/authorize oauth-config)
(oauth/callback oauth-config code)
;; => {:access_token "..." :refresh_token "..."}

;; パスワードハッシュ
(def hash (password/hash "my-password" :bcrypt))
(password/verify "my-password" hash)
;; => true
```

#### 5. ファイル監視 📁

```qi
;; ファイル・ディレクトリ監視
(def watcher
  (fs/watch "src"
    {:on-create (fn [path] (log f"Created: {path}"))
     :on-modify (fn [path] (log f"Modified: {path}"))
     :on-delete (fn [path] (log f"Deleted: {path}"))
     :on-rename (fn [old new] (log f"Renamed: {old} -> {new}"))}))

;; ホットリロード機能
(def server
  (server/serve app
    {:port 3000
     :hot-reload true
     :watch-dir "src"}))
;; ファイル変更時に自動的にリロード
```

#### 6. ログ高度機能 📊

```qi
;; ログ出力先指定
(log/configure
  {:level :info
   :outputs [{:type :file :path "app.log"}
             {:type :stdout :format :json}
             {:type :syslog :host "localhost"}]})

;; ログローテーション
(log/configure
  {:outputs [{:type :file
              :path "app.log"
              :rotation :daily  ;; or :size
              :max-files 7
              :compress true}]})

;; 非同期ログ出力（パフォーマンス向上）
(log/configure {:async true :buffer-size 1000})
```

#### 7. メトリクス・モニタリング 📈

```qi
;; カウンター
(def requests-counter (metrics/counter "http_requests_total"))
(metrics/inc requests-counter)

;; ゲージ
(def memory-gauge (metrics/gauge "memory_usage_bytes"))
(metrics/set memory-gauge 1024000)

;; ヒストグラム
(def duration-histogram (metrics/histogram "http_request_duration_seconds"))
(metrics/observe duration-histogram 0.125)

;; Prometheus形式エクスポート
(server/serve (metrics/handler) {:port 9090})
;; => http://localhost:9090/metrics

;; APM連携
(metrics/configure {:apm {:provider :datadog :api-key "..."}})
```

---

## 📌 優先度中（将来検討）

### パイプライン拡張

#### flow DSL - 分岐・合流を含む複雑な流れ

```qi
;; 複雑なデータフローを構造化
(flow data
  -> parse
  -> (branch
       [valid? -> process]
       [invalid? -> log-error])
  -> merge
  -> save)

;; 実用例: データ処理パイプライン
(flow raw-data
  -> clean
  -> (split
       [:numeric -> (branch
                      [outlier? -> remove-outlier]
                      [normal? -> normalize])]
       [:categorical -> encode])
  -> merge
  -> model/predict)
```

### パターンマッチング拡張

#### => 変換パターン - マッチ時にデータを変換

```qi
;; 束縛と同時に変換関数を適用（パイプライン的）
(match data
  {:price p => parse-float} -> (calc-tax p)
  {:name n => lower} -> (log n)
  {:created-at t => parse-date} -> (format t))

;; 複数の変換をつなげる
(match input
  {:raw r => trim => lower => (split " ")} -> (process-words r))

;; 実用例: APIレスポンス処理
(match (http/get "/api/user")
  {:body b => json/parse} -> (extract-user b)
  {:status s => str} when (= s "404") -> nil
  _ -> (error "unexpected response"))
```

### 正規表現（regex）拡張

**Phase 2以降の機能**:

```qi
;; 名前付きキャプチャ
(regex/matches "(?P<year>\\d{4})-(?P<month>\\d{2})-(?P<day>\\d{2})" "2024-01-15")
;; => {:ok {:year "2024" :month "01" :day "15"}}

;; 複数マッチの詳細情報
(regex/find-all "\\d+" "abc123def456ghi")
;; => [{:match "123" :start 3 :end 6}
;;     {:match "456" :start 9 :end 12}]
```

### 時刻処理拡張

**Phase 4: タイムゾーン対応**:

```qi
;; タイムゾーン変換
(time/to-timezone (time/now) "America/New_York")
;; => "2024-01-15T09:30:00-05:00"

;; タイムゾーン情報付き日時
(time/parse "2024-01-15T14:30:00+09:00")
;; => {:ok #inst "2024-01-15T05:30:00Z"}
```

---

## 📍 優先度低（長期計画）

### JITコンパイル

**現在の実行速度**: 中速〜高速（インタープリタ方式）

**将来の計画**:
- JITコンパイラ導入による高速化
- ホットパス最適化
- インライン展開

### 名前空間システム（Phase 6以降）

**現状**: グローバル名前空間のみ

**将来検討**:
```qi
;; 案1: Clojure風
(ns myapp.core)
(def map {...})  ;; myapp.core/map

(myapp.core/map ...)  ;; 自分のmap
(core/map ...)        ;; 組み込みmap

;; 案2: モジュールシステム拡張
(module myapp
  (def map {...}))

(myapp/map ...)
```

**優先度**: 低（設計思想「シンプル」に反するため、必要になったら検討）

---

## ✅ 完了したフェーズ

### フェーズ1: コア強化

- ✅ ネスト操作: `update`, `update-in`, `get-in`, `assoc-in`, `dissoc-in`
- ✅ 関数型基礎: `identity`, `constantly`, `comp`, `apply`, `partial`
- ✅ 集合演算: `union`, `intersect`, `difference`
- ✅ 数値基本: `pow`, `sqrt`, `round`, `floor`, `ceil`, `clamp`, `rand`, `rand-int`

### フェーズ2: 分析・集約

- ✅ `list/sort-by`, `frequencies`, `list/count-by`
- ✅ `list/chunk`, `take-while`, `drop-while`
- ✅ `println`, `read-lines`, `file-exists?`

### フェーズ3: 高度機能

- ✅ `list/max-by`, `list/min-by`, `list/sum-by`
- ✅ `complement`, `juxt`

### フェーズ4: 並行・並列処理

- ✅ 完全スレッドセーフ化（`Arc<RwLock<_>>`）
- ✅ `pmap`の完全並列化（rayon）
- ✅ Layer 1: `go`/`chan`実装
- ✅ Layer 2: `pipeline`実装
- ✅ Layer 3: `async`/`await`実装

### フェーズ4.5: Web開発機能

- ✅ Railway Pipeline (`|>?`)
- ✅ JSON/HTTP完全実装
- ✅ デバッグ関数（`inspect`, `time`）
- ✅ コレクション拡張（`find`, `every?`, `some?`, `zipmap`等）

### フェーズ5: 並行・並列処理の完成

- ✅ 並列コレクション完成（`go/pfilter`, `go/preduce`）
- ✅ `go/select!`とタイムアウト（`go/recv! :timeout`, `go/select!`）
- ✅ Structured Concurrency（`go/make-scope`, `go/scope-go`, `go/cancel!`, `go/cancelled?`, `go/with-scope`）
- ✅ `go/parallel-do`（複数式の並列実行）

### フェーズ5.5: アプリケーション開発機能

- ✅ ZIP圧縮・解凍モジュール（`zip/create`, `zip/extract`, `zip/list`, `zip/add`, `zip/gzip`, `zip/gunzip`）
- ✅ コマンドライン引数パースモジュール（`args/all`, `args/get`, `args/parse`, `args/count`）

### フェーズ6: 統計・データ分析

- ✅ 基本統計関数（`stats/mean`, `stats/median`, `stats/mode`）
- ✅ 分散・標準偏差（`stats/variance`, `stats/stddev`）
- ✅ パーセンタイル（`stats/percentile`）

**実装済み機能例**:
```qi
(stats/mean [1 2 3 4 5])        ;; => 3.0
(stats/median [1 2 3 4 5])      ;; => 3.0
(stats/stddev [1 2 3 4 5])      ;; => 1.414...
(stats/percentile [1 2 3 4 5] 95)  ;; => 4.8
```

---

## 📚 関連ドキュメント

- **[docs/spec/](docs/spec/)** - 実装済み機能の完全仕様
- **[README.md](README.md)** - プロジェクト概要
- **[CLAUDE.md](CLAUDE.md)** - 開発者向けガイド
- **[docs/style-guide.md](docs/style-guide.md)** - コーディングスタイルガイド

---

## 📝 ドキュメント更新履歴

- 2025-10-21: フェーズ6（統計・データ分析）を完了フェーズに移動
- 2025-01-XX: 初版作成（SPEC.mdから未実装機能を抽出）
