# OpenAPIライブラリチュートリアル

このチュートリアルでは、Qiの`std/lib/openapi.qi`ライブラリを使って、OpenAPI仕様を自動生成するREST APIを構築します。

## 前提条件

- Qiがインストールされていること
- HTTPサーバーの基本的な知識
- REST APIの基本的な知識

## ステップ1: シンプルなAPIサーバーの作成

まず、OpenAPIライブラリを使わない通常のAPIサーバーを作成します。

```qi
;; simple-api.qi

;; ユーザー一覧取得
(defn api-get-users [req]
  (server/json {:users [{:id 1 :name "Alice"} {:id 2 :name "Bob"}]}))

;; ユーザー登録
(defn api-create-user [req]
  (let [body (json/parse (get req :body))]
    (server/json {:success true :user body} 201)))

;; ルーター
(defn router [req]
  (match [(get req :method) (get req :path)]
    [:get "/api/users"] -> (api-get-users req)
    [:post "/api/users"] -> (api-create-user req)
    _ -> (server/json {:error "Not Found"} 404)))

;; サーバー起動
(server/serve router {:port 3000})
(println "Server started on http://localhost:3000")
```

これは動作しますが、API仕様のドキュメントがありません。

## ステップ2: OpenAPIライブラリの導入

OpenAPIライブラリを使って、同じAPIにドキュメントを追加します。

```qi
;; openapi-demo.qi

;; OpenAPIライブラリをインポート（シンプルなインポート）
(use "openapi" :as openapi)

;; ========================================
;; APIエンドポイント定義（OpenAPI対応）
;; ========================================

;; ユーザー一覧取得
(openapi/defapi :get "/api/users"
  {:summary "ユーザー一覧を取得"
   :tags ["users"]
   :responses {
     200 {:description "成功"
          :content {"application/json" {
            :schema {
              :type "object"
              :properties {
                "users" {
                  :type "array"
                  :items {
                    :type "object"
                    :properties {
                      "id" {:type "integer"}
                      "name" {:type "string"}}}}}}}}}}}
  api-get-users

  (server/json {:users [{:id 1 :name "Alice"} {:id 2 :name "Bob"}]}))

;; ユーザー登録
(openapi/defapi :post "/api/users"
  {:summary "新しいユーザーを登録"
   :tags ["users"]
   :requestBody {
     :required true
     :content {"application/json" {
       :schema {
         :type "object"
         :properties {
           "name" {:type "string"}
           "email" {:type "string" :format "email"}}
         :required ["name" "email"]}}}}
   :responses {
     201 {:description "作成成功"}
     400 {:description "リクエストが不正"}}}
  api-create-user

  (let [body (json/parse (get req :body))]
    (server/json {:success true :user body} 201)))

;; ========================================
;; ルーター設定
;; ========================================

;; APIルーター
(defn api-router [req]
  (match [(get req :method) (get req :path)]
    [:get "/api/users"] -> (api-get-users req)
    [:post "/api/users"] -> (api-create-user req)
    _ -> (server/json {:error "Not Found"} 404)))

;; Swaggerエンドポイントを統合
(def router
  (openapi/with-swagger
    api-router
    {:title "User Management API"
     :version "1.0.0"
     :description "ユーザー管理のためのREST API"}))

;; サーバー起動
(server/serve router {:port 3000})
(println "Server started on http://localhost:3000")
(println "Swagger JSON: http://localhost:3000/api/swagger.json")
```

## ステップ3: 動作確認

サーバーを起動します：

```bash
QI_LANG=ja qi openapi-demo.qi
```

Swagger JSON を取得：

```bash
curl http://localhost:3000/api/swagger.json | jq .
```

出力例：

```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "User Management API",
    "version": "1.0.0",
    "description": "ユーザー管理のためのREST API"
  },
  "paths": {
    "/api/users": {
      "get": {
        "summary": "ユーザー一覧を取得",
        "tags": ["users"],
        "responses": {
          "200": {
            "description": "成功",
            "content": {
              "application/json": {
                "schema": {...}
              }
            }
          }
        }
      },
      "post": {
        "summary": "新しいユーザーを登録",
        "tags": ["users"],
        "requestBody": {...},
        "responses": {...}
      }
    }
  }
}
```

## ステップ4: パスパラメータの追加

個別ユーザー取得エンドポイントを追加します。

```qi
;; ユーザー情報取得
(openapi/defapi :get "/api/users/{id}"
  {:summary "特定のユーザー情報を取得"
   :tags ["users"]
   :parameters [{
     :name "id"
     :in "path"
     :required true
     :description "ユーザーID"
     :schema {:type "integer"}}]
   :responses {
     200 {:description "成功"
          :content {"application/json" {
            :schema {
              :type "object"
              :properties {
                "user" {
                  :type "object"
                  :properties {
                    "id" {:type "integer"}
                    "name" {:type "string"}
                    "email" {:type "string"}}}}}}}}
     404 {:description "ユーザーが見つからない"}}}
  api-get-user

  (let [id-str (last (str/split (get req :path) "/"))
        id (to-int id-str)]
    (if (= id 1)
      (server/json {:user {:id 1 :name "Alice" :email "alice@example.com"}})
      (server/json {:error "User not found"} 404))))

;; ルーターを更新
(defn api-router [req]
  (let [path (get req :path)]
    (match [(get req :method) path]
      [:get "/api/users"] -> (api-get-users req)
      [:post "/api/users"] -> (api-create-user req)
      [:get (re-matches #"/api/users/\d+" _)] -> (api-get-user req)
      _ -> (server/json {:error "Not Found"} 404))))
```

テスト：

```bash
curl http://localhost:3000/api/users/1 | jq .
```

## ステップ5: クエリパラメータの追加

ユーザー一覧取得にページネーションを追加します。

```qi
(openapi/defapi :get "/api/users"
  {:summary "ユーザー一覧を取得（ページネーション対応）"
   :tags ["users"]
   :parameters [
     {:name "limit"
      :in "query"
      :required false
      :description "取得件数"
      :schema {:type "integer" :default 10 :minimum 1 :maximum 100}}
     {:name "offset"
      :in "query"
      :required false
      :description "オフセット"
      :schema {:type "integer" :default 0 :minimum 0}}]
   :responses {
     200 {:description "成功"}}}
  api-get-users

  (let [query-params (parse-query-string (get req :query))
        limit (or (to-int (get query-params "limit")) 10)
        offset (or (to-int (get query-params "offset")) 0)]
    (server/json
      {:users [{:id 1 :name "Alice"} {:id 2 :name "Bob"}]
       :limit limit
       :offset offset})))
```

テスト：

```bash
curl "http://localhost:3000/api/users?limit=5&offset=10" | jq .
```

## ステップ6: 認証ヘッダーの追加

認証が必要なエンドポイントを追加します。

```qi
(openapi/defapi :delete "/api/users/{id}"
  {:summary "ユーザーを削除"
   :tags ["users"]
   :parameters [
     {:name "id"
      :in "path"
      :required true
      :schema {:type "integer"}}
     {:name "Authorization"
      :in "header"
      :required true
      :description "Bearer トークン"
      :schema {:type "string"}}]
   :responses {
     204 {:description "削除成功"}
     401 {:description "認証が必要"}
     404 {:description "ユーザーが見つからない"}}}
  api-delete-user

  (let [auth (get (get req :headers) "authorization")]
    (if (and auth (str/starts-with? auth "Bearer "))
      (server/json {} 204)
      (server/json {:error "Unauthorized"} 401))))
```

テスト：

```bash
# 認証なし（エラー）
curl -X DELETE http://localhost:3000/api/users/1

# 認証あり（成功）
curl -X DELETE http://localhost:3000/api/users/1 \
  -H "Authorization: Bearer token123"
```

## ステップ7: スキーマの再利用

共通のスキーマを定義して再利用します。

```qi
;; 共通スキーマ定義
(def user-schema
  {:type "object"
   :properties {
     "id" {:type "integer"}
     "name" {:type "string"}
     "email" {:type "string" :format "email"}
     "created_at" {:type "string" :format "date-time"}}})

(def error-schema
  {:type "object"
   :properties {
     "error" {:type "string"}
     "message" {:type "string"}}})

;; スキーマを使用
(openapi/defapi :get "/api/users/{id}"
  {:summary "ユーザー情報取得"
   :responses {
     200 {:description "成功"
          :content {"application/json" {:schema user-schema}}}
     404 {:description "Not Found"
          :content {"application/json" {:schema error-schema}}}}}
  api-get-user
  ...)
```

## ステップ8: Swagger UIの統合

Swagger UIを使って、ブラウザでAPIドキュメントを表示します。

```html
<!-- static/swagger-ui.html -->
<!DOCTYPE html>
<html>
<head>
  <title>API Documentation</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/api/swagger.json',
      dom_id: '#swagger-ui',
    })
  </script>
</body>
</html>
```

ルーターに追加：

```qi
(defn router [req]
  (let [path (get req :path)]
    (cond
      ;; Swagger JSON
      (= path "/api/swagger.json")
        ((openapi/swagger-endpoint api-info) req)

      ;; Swagger UI
      (= path "/docs")
        {:status 200
         :headers {"Content-Type" "text/html"}
         :body (slurp "static/swagger-ui.html")}

      ;; API endpoints
      (str/starts-with? path "/api/")
        (api-router req)

      :else
        (server/json {:error "Not Found"} 404))))
```

ブラウザで`http://localhost:3000/docs`を開くと、Swagger UIが表示されます。

## 次のステップ

- [OpenAPI Library Reference](../../../std/lib/openapi.md)
- [HTTP Server Documentation](../spec/11-stdlib-http.md)
- [Database Integration](../spec/17-stdlib-database.md)

## まとめ

このチュートリアルでは：

1. OpenAPIライブラリの基本的な使い方
2. `defapi`マクロでのエンドポイント定義
3. パスパラメータとクエリパラメータの定義
4. 認証ヘッダーの追加
5. スキーマの再利用
6. Swagger UIとの統合

を学びました。これで、OpenAPI仕様を自動生成するREST APIを構築できるようになりました。
