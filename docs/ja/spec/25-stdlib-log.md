# 標準ライブラリ - ログ出力（log/）

**構造化ログと複数フォーマット対応**

すべての関数は `log/` モジュールに属します。

---

## 概要

`log/` モジュールは、以下の機能を提供します：

- **4段階のログレベル** - DEBUG, INFO, WARN, ERROR
- **構造化ログ** - コンテキスト情報をマップで追加
- **複数フォーマット** - テキスト形式とJSON形式
- **タイムスタンプ自動付与** - ISO8601形式（UTC）
- **レベルフィルタリング** - 設定レベル未満のログは出力されない

すべてのログは標準エラー出力（stderr）に出力されます。

---

## ログレベル

### 4段階のレベル

```qi
;; DEBUG - デバッグ情報（開発時のみ）
(log/debug "処理開始")

;; INFO - 一般情報（通常の動作ログ）
(log/info "サーバー起動")

;; WARN - 警告（問題の可能性）
(log/warn "接続タイムアウト")

;; ERROR - エラー（重大な問題）
(log/error "データベース接続失敗")
```

### レベルの優先度

```
DEBUG (0) < INFO (1) < WARN (2) < ERROR (3)
```

設定したレベル未満のログは出力されません。

**デフォルト**: `INFO`（INFOレベル以上のみ出力）

---

## ログ出力

### log/debug

DEBUGレベルのログを出力します。開発時のデバッグ情報に使用します。

```qi
(log/debug message)
(log/debug message context-map)
```

**引数:**
- `message` (string) - ログメッセージ
- `context-map` (map, optional) - コンテキスト情報

**戻り値:** nil

**例:**

```qi
;; シンプルなメッセージ
(log/debug "関数開始")
;; [2025-01-15T10:30:45.123+0000] DEBUG 関数開始

;; コンテキスト付き
(log/debug "変数の値" {:x 10 :y 20})
;; [2025-01-15T10:30:45.456+0000] DEBUG 変数の値 | x=10 y=20

;; パイプラインでの使用
(defn process-data [data]
  (log/debug "データ処理開始" {:count (len data)})
  (data
   |> (map transform)
   |> (filter valid?)))
```

---

### log/info

INFOレベルのログを出力します。通常の動作ログに使用します。

```qi
(log/info message)
(log/info message context-map)
```

**引数:**
- `message` (string) - ログメッセージ
- `context-map` (map, optional) - コンテキスト情報

**戻り値:** nil

**例:**

```qi
;; サーバー起動
(log/info "HTTPサーバー起動" {:port 8080 :host "localhost"})
;; [2025-01-15T10:30:45.789+0000] INFO HTTPサーバー起動 | port=8080 host=localhost

;; リクエスト処理
(defn handle-request [req]
  (log/info "リクエスト受信" {:method (get req :method) :path (get req :path)})
  (process req))

;; バッチ処理の進捗
(doseq [item items]
  (log/info "アイテム処理" {:id (get item :id) :status "processing"})
  (process-item item))
```

---

### log/warn

WARNレベルのログを出力します。問題の可能性がある状況に使用します。

```qi
(log/warn message)
(log/warn message context-map)
```

**引数:**
- `message` (string) - ログメッセージ
- `context-map` (map, optional) - コンテキスト情報

**戻り値:** nil

**例:**

```qi
;; タイムアウト警告
(log/warn "接続タイムアウト" {:timeout-ms 5000 :retry-count 3})
;; [2025-01-15T10:30:46.012+0000] WARN 接続タイムアウト | timeout-ms=5000 retry-count=3

;; リトライ可能なエラー
(defn fetch-with-retry [url max-retries]
  (loop [retries 0]
    (try
      (http/get url)
      (fn [err]
        (if (< retries max-retries)
            (do
              (log/warn "リクエスト失敗、リトライ中" {:url url :retry retries :error err})
              (recur (+ retries 1)))
            (log/error "最大リトライ回数に到達" {:url url :error err}))))))

;; 非推奨の機能使用
(defn old-api [data]
  (log/warn "非推奨のAPIが使用されました" {:function "old-api"})
  (process-legacy data))
```

---

### log/error

ERRORレベルのログを出力します。重大な問題に使用します。

```qi
(log/error message)
(log/error message context-map)
```

**引数:**
- `message` (string) - ログメッセージ
- `context-map` (map, optional) - コンテキスト情報

**戻り値:** nil

**例:**

```qi
;; データベース接続エラー
(log/error "データベース接続失敗" {:error "connection refused" :host "localhost"})
;; [2025-01-15T10:30:46.345+0000] ERROR データベース接続失敗 | error=connection refused host=localhost

;; エラーハンドリング
(try
  (db/connect db-config)
  (fn [err]
    (log/error "DB接続エラー" {:config db-config :error (str err)})
    (exit 1)))

;; バリデーションエラー
(defn validate-user [user]
  (if (not (get user :email))
      (do
        (log/error "バリデーションエラー" {:field "email" :user-id (get user :id)})
        false)
      true))
```

---

## 設定

### log/set-level

ログレベルを設定します。設定したレベル未満のログは出力されません。

```qi
(log/set-level level-string)
```

**引数:**
- `level-string` (string) - ログレベル（`"debug"`, `"info"`, `"warn"`, `"error"`）

**戻り値:** nil

**例:**

```qi
;; 開発環境: すべてのログを出力
(log/set-level "debug")
(log/debug "これは表示される")
(log/info "これも表示される")

;; 本番環境: INFOレベル以上のみ
(log/set-level "info")
(log/debug "これは表示されない")
(log/info "これは表示される")

;; エラーのみ
(log/set-level "error")
(log/warn "これは表示されない")
(log/error "これは表示される")

;; 環境変数から設定
(log/set-level (env/get "LOG_LEVEL" "info"))

;; 警告レベル（"warning" も受け付ける）
(log/set-level "warning")
```

---

### log/set-format

ログフォーマットを設定します。

```qi
(log/set-format format-string)
```

**引数:**
- `format-string` (string) - フォーマット（`"text"`, `"plain"`, `"json"`）

**戻り値:** nil

**フォーマット:**

#### テキスト形式（デフォルト）

```qi
(log/set-format "text")
(log/info "サーバー起動" {:port 8080})
;; [2025-01-15T10:30:45.123+0000] INFO サーバー起動 | port=8080
```

形式: `[タイムスタンプ] レベル メッセージ | key1=value1 key2=value2`

#### JSON形式

```qi
(log/set-format "json")
(log/info "サーバー起動" {:port 8080})
;; {"timestamp":"2025-01-15T10:30:45.123+0000","level":"INFO","message":"サーバー起動","port":"8080"}
```

**用途:**
- **text**: 人間が読みやすい（開発環境、デバッグ）
- **json**: ログ集約ツール向け（Elasticsearch, Splunk等）

**例:**

```qi
;; 環境に応じてフォーマット変更
(let [env (env/get "ENV" "development")]
  (if (= env "production")
      (log/set-format "json")
      (log/set-format "text")))

;; JSON形式で構造化ログ
(log/set-format "json")
(log/info "リクエスト処理完了"
  {:request-id "req-123"
   :user-id 456
   :duration-ms 234
   :status 200})
;; {"timestamp":"2025-01-15T10:30:45.567+0000","level":"INFO","message":"リクエスト処理完了","request-id":"req-123","user-id":"456","duration-ms":"234","status":"200"}
```

---

## 実用例

### アプリケーション起動時の設定

```qi
;; 起動時にログ設定を初期化
(defn init-logging []
  (let [level (env/get "LOG_LEVEL" "info")
        format (env/get "LOG_FORMAT" "text")]
    (log/set-level level)
    (log/set-format format)
    (log/info "ログ設定完了" {:level level :format format})))

(init-logging)
```

---

### HTTPサーバーのログ

```qi
;; リクエスト/レスポンスのログ
(defn log-request [req]
  (log/info "リクエスト受信"
    {:method (get req :method)
     :path (get req :path)
     :user-agent (get-in req [:headers "User-Agent"])}))

(defn log-response [req res duration-ms]
  (log/info "レスポンス送信"
    {:method (get req :method)
     :path (get req :path)
     :status (get res :status)
     :duration-ms duration-ms}))

;; ミドルウェア
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

### エラー追跡

```qi
;; エラー詳細をログに記録
(defn process-with-error-tracking [data]
  (try
    (do
      (log/debug "処理開始" {:data-size (len data)})
      (let [result (process data)]
        (log/info "処理成功" {:result-size (len result)})
        result))
    (fn [err]
      (log/error "処理失敗"
        {:error (str err)
         :data-size (len data)
         :stack-trace (get err :stack)})
      (throw err))))
```

---

### バッチ処理の進捗ログ

```qi
;; 大量データ処理の進捗表示
(defn process-batch [items]
  (let [total (len items)]
    (log/info "バッチ処理開始" {:total total})
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
                           (log/warn "アイテム処理失敗"
                             {:index i :item-id (get item :id) :error (str err)})
                           :error))]
            (when (= (mod i 100) 0)
              (log/info "進捗"
                {:processed i :total total :percent (/ (* i 100) total)}))
            (recur
              (+ i 1)
              (if (= result :ok) (+ processed 1) processed)
              (if (= result :error) (+ failed 1) failed)))
          (log/info "バッチ処理完了"
            {:total total :processed processed :failed failed})))))
```

---

### デバッグログ

```qi
;; 開発時のデバッグ
(defn complex-calculation [x y z]
  (log/debug "計算開始" {:x x :y y :z z})

  (let [step1 (+ x y)]
    (log/debug "ステップ1完了" {:step1 step1})

    (let [step2 (* step1 z)]
      (log/debug "ステップ2完了" {:step2 step2})

      (let [result (/ step2 2)]
        (log/debug "計算完了" {:result result})
        result))))

;; 本番環境では DEBUG ログは出力されない
(log/set-level "info")
(complex-calculation 10 20 5)  ;; デバッグログは表示されない
```

---

### 構造化ログの活用

```qi
;; JSONフォーマットで統一したログ構造
(log/set-format "json")

;; ユーザーアクション追跡
(defn track-user-action [user-id action details]
  (log/info "ユーザーアクション"
    {:user-id user-id
     :action action
     :timestamp (time/now-iso)
     :details (json/stringify details)}))

(track-user-action 123 "login" {:ip "192.168.1.1" :device "mobile"})
;; {"timestamp":"2025-01-15T10:30:45.890+0000","level":"INFO","message":"ユーザーアクション","user-id":"123","action":"login","details":"{\"ip\":\"192.168.1.1\",\"device\":\"mobile\"}"}

;; ログ集約ツール（Elasticsearch等）で検索・分析が容易
```

---

### 条件付きログ

```qi
;; 特定条件でのみログ出力
(defn process-with-conditional-log [data]
  (when (> (len data) 1000)
    (log/warn "大量データ処理" {:size (len data)}))

  (let [result (process data)]
    (when (empty? result)
      (log/warn "処理結果が空" {:input-size (len data)}))
    result))

;; デバッグモード時のみ詳細ログ
(def debug-mode (= (env/get "DEBUG") "true"))

(defn debug-log [msg ctx]
  (when debug-mode
    (log/debug msg ctx)))

(debug-log "詳細情報" {:var1 x :var2 y})
```

---

## ログレベルのベストプラクティス

### DEBUGレベル

**用途**: 開発時のデバッグ情報

```qi
;; 変数の値
(log/debug "変数の値" {:x x :y y})

;; 関数の開始/終了
(log/debug "関数開始" {:function "process-data" :args args})
(log/debug "関数終了" {:function "process-data" :result result})

;; 内部状態
(log/debug "ループ状態" {:iteration i :current-value val})
```

---

### INFOレベル

**用途**: 通常の動作ログ、重要なイベント

```qi
;; サーバー起動/停止
(log/info "サーバー起動" {:port 8080})
(log/info "サーバー停止")

;; リクエスト処理
(log/info "リクエスト処理完了" {:path "/api/users" :status 200})

;; バッチ処理の開始/完了
(log/info "バッチ処理開始" {:job-id "batch-001"})
(log/info "バッチ処理完了" {:job-id "batch-001" :processed 1000})
```

---

### WARNレベル

**用途**: 問題の可能性、非推奨の使用

```qi
;; リトライ可能なエラー
(log/warn "接続失敗、リトライ中" {:retry-count 2 :max-retries 5})

;; パフォーマンス問題
(log/warn "処理時間が長い" {:duration-ms 5000 :threshold-ms 1000})

;; 非推奨の機能
(log/warn "非推奨APIの使用" {:api "old-api" :alternative "new-api"})

;; データの問題
(log/warn "不正なデータをスキップ" {:line 123 :reason "invalid format"})
```

---

### ERRORレベル

**用途**: 重大な問題、処理失敗

```qi
;; 接続エラー
(log/error "データベース接続失敗" {:error err :host db-host})

;; バリデーションエラー
(log/error "データ検証失敗" {:field "email" :value invalid-email})

;; システムエラー
(log/error "ファイル書き込み失敗" {:path file-path :error err})

;; 予期しないエラー
(log/error "予期しないエラー" {:error err :context ctx :stack-trace stack})
```

---

## エラーハンドリング

### エラー時のログ

```qi
;; try/catch と組み合わせ
(try
  (risky-operation)
  (fn [err]
    (log/error "操作失敗" {:operation "risky-operation" :error (str err)})
    (default-value)))

;; パイプラインでのエラー
(data
 |> (map (fn [x]
           (try
             (process x)
             (fn [err]
               (log/warn "アイテム処理スキップ" {:item x :error (str err)})
               nil))))
 |> (filter some?))
```

---

## パフォーマンス考慮事項

### ログレベルによるフィルタリング

```qi
;; ❌ 悪い例: 常に文字列を構築
(log/debug (str "大きなデータ: " (json/stringify large-data)))
;; DEBUG レベルでなくても文字列構築のコストがかかる

;; ✅ 良い例: 必要な場合のみ構築
(when (>= (log/current-level) :debug)  ;; 仮想の関数
  (log/debug "大きなデータ" {:data large-data}))
```

---

### 本番環境での設定

```qi
;; 本番環境
(log/set-level "info")    ;; DEBUG ログは出力しない
(log/set-format "json")   ;; 構造化ログで集約・分析

;; 開発環境
(log/set-level "debug")   ;; すべてのログを出力
(log/set-format "text")   ;; 人間が読みやすい形式
```

---

## 関数一覧

| 関数 | 説明 | 用途 |
|------|------|------|
| `log/debug` | DEBUGレベルログ出力 | デバッグ情報 |
| `log/info` | INFOレベルログ出力 | 通常の動作ログ |
| `log/warn` | WARNレベルログ出力 | 警告 |
| `log/error` | ERRORレベルログ出力 | エラー |
| `log/set-level` | ログレベル設定 | フィルタリング制御 |
| `log/set-format` | ログフォーマット設定 | 出力形式制御 |

---

## ログフォーマット詳細

### テキスト形式

```
[2025-01-15T10:30:45.123+0000] INFO サーバー起動 | port=8080 host=localhost
```

**構成要素:**
- `[...]` - タイムスタンプ（ISO8601形式、UTC）
- `INFO` - ログレベル
- `サーバー起動` - メッセージ
- `| key=value ...` - コンテキスト（オプション）

---

### JSON形式

```json
{"timestamp":"2025-01-15T10:30:45.123+0000","level":"INFO","message":"サーバー起動","port":"8080","host":"localhost"}
```

**フィールド:**
- `timestamp` - タイムスタンプ（ISO8601形式、UTC）
- `level` - ログレベル
- `message` - メッセージ
- その他 - コンテキストマップのキー・値

---

## 関連項目

- [エラー処理](08-error-handling.md) - try/catchによるエラーハンドリング
- [環境変数](23-stdlib-env.md) - 設定の管理
- [HTTPサーバー](11-stdlib-http.md) - リクエストログ
- [デバッグ](20-stdlib-debug.md) - デバッグ機能
