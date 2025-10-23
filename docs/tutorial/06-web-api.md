# 第6章: WebアプリケーションとAPI

**所要時間**: 40分

QiでHTTPサーバーとJSON APIを構築する方法を学びます。シンプルで読みやすいコードで、**本格的なWebアプリケーション**が作れます。

---

## 最小のHTTPサーバー

まずは、最もシンプルなHTTPサーバーを作ってみましょう。

```qi
(defn handler [req]
  (server/text "Hello, World!"))

(server/serve 3000 handler)
; => サーバーが起動: http://localhost:3000
```

ブラウザで`http://localhost:3000`にアクセスすると、`Hello, World!`が表示されます。

---

## リクエストとレスポンス

### リクエストの構造

ハンドラーには、以下のようなリクエストマップが渡されます：

```qi
{:method "GET"
 :path "/users/123"
 :headers {:content-type "application/json"}
 :body "..."}
```

### レスポンスの種類

Qiは、便利なレスポンスヘルパーを提供しています。

```qi
; テキストレスポンス
(server/text "Hello")
; => {:status 200 :headers {:content-type "text/plain"} :body "Hello"}

; JSONレスポンス
(server/json {:message "Success" :data [1 2 3]})
; => {:status 200 :headers {:content-type "application/json"} :body "..."}

; HTMLレスポンス
(server/html "<h1>Welcome</h1>")
; => {:status 200 :headers {:content-type "text/html"} :body "..."}

; カスタムステータス
(server/response 201 {:message "Created"})
; => {:status 201 ...}
```

---

## ルーティング

パスに応じて処理を分岐します。

```qi
(defn handler [req]
  (match (get req :path)
    "/" -> (server/text "Home")
    "/about" -> (server/text "About")
    "/api/status" -> (server/json {:status "ok"})
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

### メソッドとパスの組み合わせ

```qi
(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/"] -> (server/text "Home")
    ["GET" "/users"] -> (server/json {:users []})
    ["POST" "/users"] -> (server/json {:message "User created"})
    ["GET" "/users/123"] -> (server/json {:id 123 :name "Alice"})
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

---

## JSON API

### GET: データ取得

```qi
(def users
  [{:id 1 :name "Alice" :age 25}
   {:id 2 :name "Bob" :age 30}
   {:id 3 :name "Carol" :age 28}])

(defn get-users [req]
  (server/json {:users users}))

(defn get-user [req id]
  (let [user (first (filter (fn [u] (= (get u :id) id)) users))]
    (if (nil? user)
      (server/response 404 {:error "User not found"})
      (server/json user))))

(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/users"] -> (get-users req)
    ["GET" path] when (str/starts-with? path "/api/users/") ->
      (let [id-str (str/replace path "/api/users/" "")
            id (string/to-int id-str)]
        (get-user req id))
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

**テスト**:
```bash
curl http://localhost:3000/api/users
# => {"users":[{"id":1,"name":"Alice","age":25},...]}

curl http://localhost:3000/api/users/1
# => {"id":1,"name":"Alice","age":25}

curl http://localhost:3000/api/users/999
# => {"error":"User not found"}
```

### POST: データ作成

```qi
(def users (atom []))

(defn create-user [req]
  (let [body (get req :body)
        user (json/parse body)]
    (if (error? user)
      (server/response 400 {:error "Invalid JSON"})
      (do
        (swap! users conj user)
        (server/response 201 {:message "User created" :user user})))))

(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/users"] -> (server/json {:users @users})
    ["POST" "/api/users"] -> (create-user req)
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

**テスト**:
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","age":25}'
# => {"message":"User created","user":{"name":"Alice","age":25}}
```

---

## エラーハンドリング

Railway Pipelineを使って、エラーを優雅に処理します。

```qi
(defn parse-body [req]
  (let [body (get req :body)]
    (if (nil? body)
      {:error "No body"}
      (json/parse body))))

(defn validate-user [user]
  (if (error? user)
    user
    (match user
      {:name n :age a} when (and (string? n) (number? a)) -> user
      _ -> {:error "Invalid user data"})))

(defn save-user [user]
  (if (error? user)
    user
    (do
      (swap! users conj user)
      user)))

(defn create-user [req]
  (let [result (req
                |> parse-body
                |>? validate-user
                |>? save-user)]
    (if (error? result)
      (server/response 400 result)
      (server/response 201 {:message "User created" :user result}))))
```

---

## 実用例: CRUD API

完全なCRUD（Create, Read, Update, Delete）APIを作成します。

```qi
(def users (atom {}))
(def next-id (atom 1))

; Create
(defn create-user [req]
  (req
   |> (get :body)
   |> json/parse
   |>? (fn [user]
         (let [id @next-id
               new-user (assoc user :id id)]
           (do
             (swap! next-id inc)
             (swap! users assoc id new-user)
             new-user)))
   |> (fn [result]
        (if (error? result)
          (server/response 400 result)
          (server/response 201 result)))))

; Read (all)
(defn list-users [req]
  (server/json {:users (vals @users)}))

; Read (one)
(defn get-user [req id]
  (let [user (get @users id)]
    (if (nil? user)
      (server/response 404 {:error "User not found"})
      (server/json user))))

; Update
(defn update-user [req id]
  (let [user (get @users id)]
    (if (nil? user)
      (server/response 404 {:error "User not found"})
      (req
       |> (get :body)
       |> json/parse
       |>? (fn [updates]
             (let [updated (merge user updates)]
               (do
                 (swap! users assoc id updated)
                 updated)))
       |> (fn [result]
            (if (error? result)
              (server/response 400 result)
              (server/json result)))))))

; Delete
(defn delete-user [req id]
  (let [user (get @users id)]
    (if (nil? user)
      (server/response 404 {:error "User not found"})
      (do
        (swap! users dissoc id)
        (server/response 204 nil)))))

; ルーティング
(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/users"] -> (list-users req)
    ["POST" "/api/users"] -> (create-user req)
    ["GET" path] when (str/starts-with? path "/api/users/") ->
      (let [id (string/to-int (str/replace path "/api/users/" ""))]
        (get-user req id))
    ["PUT" path] when (str/starts-with? path "/api/users/") ->
      (let [id (string/to-int (str/replace path "/api/users/" ""))]
        (update-user req id))
    ["DELETE" path] when (str/starts-with? path "/api/users/") ->
      (let [id (string/to-int (str/replace path "/api/users/" ""))]
        (delete-user req id))
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

**テスト**:
```bash
# Create
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","age":25}'

# Read all
curl http://localhost:3000/api/users

# Read one
curl http://localhost:3000/api/users/1

# Update
curl -X PUT http://localhost:3000/api/users/1 \
  -H "Content-Type: application/json" \
  -d '{"age":26}'

# Delete
curl -X DELETE http://localhost:3000/api/users/1
```

---

## ミドルウェア

リクエストの前処理・後処理を行う関数です。

### ログミドルウェア

```qi
(defn logger [handler]
  (fn [req]
    (do
      (println f"{(get req :method)} {(get req :path)}")
      (let [resp (handler req)]
        (do
          (println f"  -> {(get resp :status)}")
          resp)))))

(defn my-handler [req]
  (server/text "Hello"))

(def app (logger my-handler))

(server/serve 3000 app)
```

### CORSミドルウェア

```qi
(defn cors [handler]
  (fn [req]
    (let [resp (handler req)]
      (assoc resp :headers
        (merge (get resp :headers)
               {:access-control-allow-origin "*"
                :access-control-allow-methods "GET, POST, PUT, DELETE"})))))

(def app (cors my-handler))

(server/serve 3000 app)
```

### ミドルウェアの合成

```qi
(def app
  (-> my-handler
      logger
      cors))

(server/serve 3000 app)
```

### 認証ミドルウェア（JWT）

JWT（JSON Web Token）を使った認証ミドルウェアの実装例です。

```qi
;; Authorizationヘッダーからトークンを抽出
(defn extract-auth-token [request]
  (let [auth-header (get-in request [:headers :authorization])]
    (if (nil? auth-header)
      nil
      (if (string/starts-with? auth-header "Bearer ")
        (string/replace-first auth-header "Bearer " "")
        nil))))

;; 認証が必要なエンドポイント用ミドルウェア
(defn require-auth [handler]
  (fn [request]
    (let [token (extract-auth-token request)]
      (if (nil? token)
        {:status 401
         :headers {:content-type "application/json"}
         :body (json/stringify {:error "Missing authorization token"})}
        (match (jwt/verify token "my-secret-key")
          {:error _} -> {:status 401
                        :headers {:content-type "application/json"}
                        :body (json/stringify {:error "Invalid token"})}
          payload -> (handler (assoc request :user payload)))))))

;; 保護されたエンドポイント
(defn handle-profile [request]
  (let [user (get request :user)]
    (server/json {:user user :message "This is a protected resource"})))

;; ルーティング
(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["POST" "/api/login"] -> (handle-login req)
    ["GET" "/api/profile"] -> ((require-auth handle-profile) req)
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

**認証フローの例**:
```bash
# 1. ログインしてトークンを取得
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
# => {"token":"eyJ0eXAi..."}

# 2. トークンを使って保護されたリソースにアクセス
curl http://localhost:3000/api/profile \
  -H "Authorization: Bearer eyJ0eXAi..."
# => {"user":{"user_id":1,"username":"alice"},"message":"This is a protected resource"}

# 3. トークンなしでアクセス（401エラー）
curl http://localhost:3000/api/profile
# => {"error":"Missing authorization token"}
```

**詳細な実装例**: `examples/17-jwt-auth.qi` および `examples/19-auth-api.qi` を参照してください。

---

## 実用例: シンプルなブログAPI

```qi
(def posts (atom {}))
(def next-id (atom 1))

(defn create-post [req]
  (req
   |> (get :body)
   |> json/parse
   |>? (fn [post]
         (let [id @next-id
               new-post (assoc post :id id :created-at (now))]
           (do
             (swap! next-id inc)
             (swap! posts assoc id new-post)
             new-post)))
   |> (fn [result]
        (if (error? result)
          (server/response 400 result)
          (server/response 201 result)))))

(defn list-posts [req]
  (server/json {:posts (vals @posts)}))

(defn get-post [req id]
  (let [post (get @posts id)]
    (if (nil? post)
      (server/response 404 {:error "Post not found"})
      (server/json post))))

(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/posts"] -> (list-posts req)
    ["POST" "/api/posts"] -> (create-post req)
    ["GET" path] when (str/starts-with? path "/api/posts/") ->
      (let [id (string/to-int (str/replace path "/api/posts/" ""))]
        (get-post req id))
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 (logger (cors handler)))
```

---

## 練習問題

### 問題1: シンプルなカウンターAPI

アクセスカウンターAPIを作ってください。

```qi
; GET /api/count - 現在のカウントを返す
; POST /api/count/increment - カウントを1増やす
; POST /api/count/reset - カウントを0にリセット
```

<details>
<summary>解答例</summary>

```qi
(def count (atom 0))

(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/count"] -> (server/json {:count @count})
    ["POST" "/api/count/increment"] ->
      (do
        (swap! count inc)
        (server/json {:count @count}))
    ["POST" "/api/count/reset"] ->
      (do
        (reset! count 0)
        (server/json {:count @count}))
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

</details>

### 問題2: ToDo API

シンプルなToDo APIを作ってください。

```qi
; GET /api/todos - 全てのToDoを取得
; POST /api/todos - 新しいToDoを作成
; PUT /api/todos/:id/complete - ToDoを完了にする
```

<details>
<summary>解答例</summary>

```qi
(def todos (atom {}))
(def next-id (atom 1))

(defn create-todo [req]
  (let [body (json/parse (get req :body))
        id @next-id
        todo {:id id :title (get body :title) :completed false}]
    (do
      (swap! next-id inc)
      (swap! todos assoc id todo)
      (server/response 201 todo))))

(defn complete-todo [req id]
  (let [todo (get @todos id)]
    (if (nil? todo)
      (server/response 404 {:error "Todo not found"})
      (let [updated (assoc todo :completed true)]
        (do
          (swap! todos assoc id updated)
          (server/json updated))))))

(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/api/todos"] -> (server/json {:todos (vals @todos)})
    ["POST" "/api/todos"] -> (create-todo req)
    ["PUT" path] when (str/starts-with? path "/api/todos/") ->
      (let [parts (str/split path "/")
            id (string/to-int (nth parts 3))]
        (complete-todo req id))
    _ -> (server/response 404 "Not Found")))

(server/serve 3000 handler)
```

</details>

---

## まとめ

この章で学んだこと：

- ✅ HTTPサーバーの基本
- ✅ ルーティングとリクエスト処理
- ✅ JSON APIの構築
- ✅ CRUDエンドポイントの実装
- ✅ ミドルウェアパターン
- ✅ JWT認証ミドルウェア
- ✅ エラーハンドリング

---

## 🎉 チュートリアル完了！

お疲れさまでした！これでQiの主要機能を全て学びました。

### 学んだこと
1. ✅ 基本構文とデータ型
2. ✅ パイプライン演算子
3. ✅ パターンマッチング
4. ✅ エラー処理（Railway Pipeline）
5. ✅ 並行・並列処理
6. ✅ WebアプリケーションとAPI

### 次のステップ

1. **自分のプロジェクトを始める**
   - 小さなCLIツール
   - Web API
   - データ処理スクリプト

2. **examples/ディレクトリを見る**
   - 実践的なコード例
   - ベストプラクティス

3. **ドキュメントを深く学ぶ**
   - [完全な言語仕様](../spec/)
   - [関数索引](../spec/FUNCTION-INDEX.md)

4. **コミュニティに参加する**
   - GitHub Issues
   - Discussions

---

Happy coding with Qi! 🚀
