# 標準ライブラリ - HTTP

**HTTPクライアントとサーバー**

---

## HTTPクライアント（http/）

Qiでは2種類のHTTPクライアント関数を提供しています：

- **シンプル版**（`http/get`, `http/post`等）: レスポンスボディのみを返す。**2xx以外のステータスコードは例外を投げる**
- **詳細版**（`http/get!`, `http/post!`等）: ステータスコード、ヘッダー、ボディを含む詳細情報を返す。**すべてのステータスコードでMapを返す**（例外を投げない）

### シンプル版 - レスポンスボディのみ取得

多くの場合、レスポンスボディだけが必要です。シンプル版は**文字列**を返します。

**エラーハンドリング**: 2xx以外のステータスコード（404, 500等）は例外を投げます。

```qi
;; http/get - HTTP GETリクエスト（ボディのみ）
(http/get "https://httpbin.org/get")
;; => "{\"args\": {}, \"headers\": {...}}"

;; http/post - HTTP POSTリクエスト
(http/post "https://api.example.com/users" {"name" "Alice" "email" "alice@example.com"})
;; => "{\"id\": 123, \"name\": \"Alice\"}"

;; http/put - HTTP PUTリクエスト
(http/put "https://api.example.com/users/1" {"name" "Alice Updated"})

;; http/delete - HTTP DELETEリクエスト
(http/delete "https://api.example.com/users/1")

;; http/patch - HTTP PATCHリクエスト
(http/patch "https://api.example.com/users/1" {"email" "newemail@example.com"})

;; http/head - HTTP HEADリクエスト
(http/head "https://api.example.com/status")

;; http/options - HTTP OPTIONSリクエスト
(http/options "https://api.example.com")

;; シンプルな使い方 - ボディを直接JSONパース
(def users (http/get "https://api.example.com/users" |> json/parse))

;; エラーハンドリング - 404や500は例外を投げる
(match (try (http/get "https://api.example.com/notfound"))
  {:error e} -> (println "Error:" e)  ;; => "Error: HTTPエラー 404"
  body -> (json/parse body))
```

### 詳細版 - ステータスコード・ヘッダー付き

ステータスコードやヘッダーが必要な場合は、感嘆符（`!`）付きの詳細版を使用します。

**エラーハンドリング**: すべてのステータスコード（2xx, 4xx, 5xx）でMapを返します。例外は投げません。

```qi
;; http/get! - HTTP GETリクエスト（詳細情報）
(http/get! "https://httpbin.org/get")
;; => {:status 200 :headers {"content-type" "application/json" ...} :body "..."}

;; http/post! - HTTP POSTリクエスト（詳細情報）
(http/post! "https://api.example.com/users" {"name" "Alice"})
;; => {:status 201 :headers {...} :body "..."}

;; http/put! - HTTP PUTリクエスト（詳細情報）
(http/put! "https://api.example.com/users/1" {"name" "Alice Updated"})

;; http/delete! - HTTP DELETEリクエスト（詳細情報）
(http/delete! "https://api.example.com/users/1")

;; http/patch! - HTTP PATCHリクエスト（詳細情報）
(http/patch! "https://api.example.com/users/1" {"email" "newemail@example.com"})

;; http/head! - HTTP HEADリクエスト（詳細情報）
(http/head! "https://api.example.com/status")

;; http/options! - HTTP OPTIONSリクエスト（詳細情報）
(http/options! "https://api.example.com")

;; ステータスコードをチェック
(let [res (http/get! "https://api.example.com/users")]
  (if (= 200 (get res :status))
    (json/parse (get res :body))
    (error (str "HTTP error: " (get res :status)))))

;; 404エラーでも例外を投げず、Mapを返す
(let [res (http/get! "https://api.example.com/notfound")]
  (println "Status:" (get res :status))  ;; => "Status: 404"
  (println "Body:" (get res :body)))     ;; エラーメッセージを取得可能

;; ヘッダーを取得
(let [res (http/get! "https://api.example.com/data")]
  (get-in res [:headers "content-type"]))
```

### オプション引数

シンプル版・詳細版の両方で、オプション引数を渡すことができます：

- **`:headers`** - カスタムHTTPヘッダー（Map）
- **`:timeout`** - タイムアウト時間（ミリ秒）
- **`:basic-auth`** - Basic認証（`[username password]`形式のVector）
- **`:bearer-token`** - Bearer Token認証（文字列）

```qi
;; GETリクエストにカスタムヘッダーを追加
(http/get "https://api.example.com/data"
  {:headers {"X-API-Key" "your-api-key"}})

;; POSTリクエストに認証とタイムアウトを設定
(http/post "https://api.example.com/users"
  {"name" "Alice"}
  {:bearer-token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
   :timeout 10000})

;; Basic認証を使用
(http/get! "https://api.example.com/protected"
  {:basic-auth ["username" "password"]})

;; 複数のオプションを組み合わせ
(http/post! "https://api.example.com/data"
  {"key" "value"}
  {:headers {"X-Request-ID" "12345"}
   :timeout 5000})
```

### 詳細設定

```qi
;; http/request - カスタムリクエスト
(http/request {
  :method "POST"
  :url "https://api.example.com/data"
  :headers {"Authorization" "Bearer token123"}
  :body {"data" "value"}
  :timeout 5000
})
```

### 認証

```qi
;; Basic認証
(http/request {
  :url "https://api.example.com/data"
  :basic-auth ["username" "password"]
})

;; Bearer Token認証
(http/request {
  :url "https://api.example.com/data"
  :bearer-token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
})
```

### コンテンツ圧縮

```qi
;; 自動解凍（デフォルトで有効）
(http/get "https://example.com/api")  ;; gzip/deflate/brotli を自動解凍

;; 送信時の圧縮
(http/post "https://example.com/api"
  {"data" "large payload"}
  {:headers {"content-encoding" "gzip"}})  ;; ボディを自動的にgzip圧縮
```

### Railway Pipelineとの統合

```qi
;; シンプル版でパイプライン - ボディが直接返される
("https://api.github.com/users/octocat"
 |> http/get
 |> json/parse
 |> (fn [data] (get data "name")))
;; => "The Octocat"

;; 詳細版でパイプライン - ステータスコードをチェック
("https://api.github.com/users/octocat"
 |> http/get!
 |> (fn [resp]
      (if (= 200 (get resp :status))
        (get resp :body)
        (error (str "HTTP " (get resp :status)))))
 |> json/parse
 |> (fn [data] (get data "name")))
;; => "The Octocat"

;; エラーハンドリング（tryで例外をキャッチ）
(match (try
         ("https://api.github.com/users/octocat"
          |> http/get
          |> json/parse
          |> (fn [data] (get data "name"))))
  {:error e} -> (log/error "Failed:" e)
  name -> name)
;; => "The Octocat" (成功時) または {:error ...} (失敗時)

;; Railway演算子でエラーハンドリング
("https://api.github.com/users/octocat"
 |> (try (http/get _))
 |>? json/parse
 |>? (fn [data] (get data "name")))
;; => "The Octocat" (成功時) または {:error ...} (失敗時)
```

---

## HTTPサーバー（server/）

**Flow-Oriented な Web アプリケーション構築**

### レスポンスヘルパー

```qi
;; server/ok - 200 OKレスポンス
(server/ok "Hello, World!")
;; => {:status 200 :headers {...} :body "Hello, World!"}

;; server/json - JSONレスポンス
(server/json {"message" "hello" "status" "success"})
;; => {:status 200 :headers {"Content-Type" "application/json"} :body "{...}"}

;; server/not-found - 404 Not Foundレスポンス
(server/not-found "Page not found")

;; server/no-content - 204 No Contentレスポンス
(server/no-content)
```

### ルーティング

```qi
;; server/router - ルーター作成
(server/router [["/" {:get hello-handler}]
                ["/api/users" {:get list-users :post create-user}]
                ["/api/users/:id" {:get get-user}]])

;; server/serve - サーバー起動
(comment
  (server/serve app {:port 3000})
  ;; => HTTP server started on http://127.0.0.1:3000
  )

;; server/serve - 詳細設定
(comment
  (server/serve app {:port 8080 :host "0.0.0.0" :timeout 30})
  ;; => HTTP server started on http://0.0.0.0:8080 (timeout: 30s)
  )
```

### ミドルウェア

```qi
;; server/with-logging - リクエスト/レスポンスをログ出力
(def handler (server/with-logging (fn [req] (server/ok "Hello"))))

;; server/with-cors - CORSヘッダーを追加
(def handler (server/with-cors (fn [req] (server/json {...}))))

;; server/with-json-body - リクエストボディを自動的にJSONパース
(def handler (server/with-json-body (fn [req] (get req :json))))

;; server/with-compression - レスポンスボディをgzip圧縮
(def handler (server/with-compression (fn [req] (server/ok "..."))))

;; server/with-basic-auth - Basic認証
(def handler (server/with-basic-auth (fn [req] ...) "user" "pass"))

;; server/with-bearer - Bearer Token抽出
(def handler (server/with-bearer (fn [req] (get req :token))))

;; server/with-no-cache - キャッシュ無効化ヘッダーを追加
(def handler (server/with-no-cache (fn [req] (server/ok "..."))))

;; server/with-cache-control - カスタムCache-Controlヘッダーを追加
(def handler (server/with-cache-control (fn [req] ...) "public, max-age=3600"))
```

### 静的ファイル配信

```qi
;; server/static-file - 単一ファイル配信
(server/static-file "index.html")

;; server/static-dir - ディレクトリ配信
(server/static-dir "public")
```

---

## 実用例

### シンプルなサーバー

```qi
;; ハンドラー（リクエスト -> レスポンス）
(def hello-handler
  (fn [req] (server/ok "Hello, World!")))

;; ルート定義（データ駆動）
(def routes [["/" {:get hello-handler}]])

;; アプリ起動
(def app (server/router routes))
(comment
  (server/serve app {:port 3000}))
```

### JSON API with パスパラメータ

```qi
;; ハンドラー定義
(def list-users
  (fn [req]
    (server/json {"users" [{"id" 1 "name" "Alice"}
                           {"id" 2 "name" "Bob"}]})))

(def get-user
  (fn [req]
    (let [user-id (get-in req [:params "id"])]
      (server/json {"id" user-id "name" "Alice"}))))

(def create-user
  (fn [req]
    (server/json {"status" "created"} {:status 201})))

;; ルート定義（パスパラメータ: /users/:id 形式）
(def routes
  [["/api/users" {:get list-users :post create-user}]
   ["/api/users/:id" {:get get-user}]
   ["/api/users/:user_id/posts/:post_id" {:get get-post}]])

;; アプリ起動
(def app (server/router routes))
(comment
  (server/serve app {:port 8080 :host "0.0.0.0" :timeout 30}))
```

### ミドルウェアの組み合わせ

```qi
;; 複数のミドルウェアを重ねる
(def api-handler
  (-> (fn [req]
        (let [json-data (get req :json)]
          (server/json {"received" json-data})))
      server/with-json-body
      server/with-cors
      server/with-logging
      server/with-compression))

;; または comp を使った関数合成
(def protected-api
  (comp
    server/with-logging
    server/with-cors
    (partial server/with-basic-auth _ "admin" "secret")
    server/with-json-body))

(def routes
  [["/api/data" {:post (protected-api handle-data)}]])
```

### リクエスト/レスポンスオブジェクト

```qi
;; リクエスト構造
{:method :get                       ;; HTTPメソッド（キーワード）
 :path "/api/users/123"             ;; リクエストパス
 :query "page=1&limit=10"           ;; クエリ文字列（生）
 :query-params {"page" "1"          ;; クエリパラメータ（自動パース）
                "limit" "10"}
 :headers {"content-type" "application/json" ...}
 :body "..."                        ;; リクエストボディ（文字列）
 :params {"id" "123"}}              ;; パスパラメータ（マッチした場合のみ）

;; レスポンス構造
{:status 200                        ;; HTTPステータスコード
 :headers {"Content-Type" "text/plain; charset=utf-8" ...}
 :body "Hello, World!"}             ;; レスポンスボディ（文字列、JSON、HTML等）

;; レスポンス構造（ストリーミング配信）
{:status 200
 :headers {"Content-Type" "video/mp4" ...}
 :body-file "/path/to/large-file.mp4"}  ;; ファイルパス（大きなファイル用）
```

---

## 実装済み機能

- ✅ **データ駆動**: ルーティングは検査・変換可能なデータ構造
- ✅ **パイプライン**: ハンドラーは `|>` で流れが明確
- ✅ **合成可能**: すべてが関数で、ミドルウェアも関数
- ✅ **スレッドセーフ**: 並列リクエスト処理に対応
- ✅ **パスパラメータ**: `/users/:id` 形式をサポート（複数パラメータ対応）
- ✅ **クエリパラメータ**: `?page=1&limit=10` を自動パース、配列対応、URLデコード
- ✅ **タイムアウト**: リクエストタイムアウトを設定可能（デフォルト30秒）
- ✅ **ミドルウェア**: ロギング、CORS、JSONボディパース（複数重ね可能）
- ✅ **静的ファイル配信**: HTML、CSS、JS、画像、フォントなどのバイナリファイル対応
- ✅ **ストリーミング配信**: 大きなファイル（動画、PDF等）をメモリ効率的に配信（`:body-file`キー）
- ✅ **コンテンツ圧縮**: gzip/deflate/brotli圧縮をサポート
- ✅ **認証**: Basic Auth、Bearer Token抽出
- ✅ **キャッシュ制御**: Cache-Control、グレースフルシャットダウン
