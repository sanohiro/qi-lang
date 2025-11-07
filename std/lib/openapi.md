# OpenAPI Library

REST APIのためのOpenAPI 3.0仕様生成ライブラリ。

## 概要

`std/lib/openapi.qi`は、QiでREST APIを構築する際に、OpenAPI 3.0仕様を自動生成するためのライブラリです。APIエンドポイントを宣言的に定義するだけで、Swagger/OpenAPI対応のドキュメントが自動生成されます。

## インストール

標準ライブラリとして提供されているため、特別なインストールは不要です。

## 基本的な使い方

### 1. ライブラリのインポート

```qi
;; シンプルなインポート（推奨）
(use "openapi" :as openapi)

;; または、フルパス指定
(use "openapi" :as openapi)
```

### 2. APIエンドポイントの定義

`defapi`マクロを使ってAPIエンドポイントを定義します：

```qi
(openapi/defapi :post "/api/users"
  {:summary "ユーザー登録"
   :tags ["users"]
   :requestBody {
     :required true
     :content {"application/json" {
       :schema {
         :type "object"
         :properties {
           "username" {:type "string"}
           "email" {:type "string" :format "email"}}
         :required ["username" "email"]}}}}
   :responses {
     201 {:description "Created"
          :content {"application/json" {
            :schema {:type "object"}}}}
     400 {:description "Bad Request"}}}
  api-create-user

  (let [body (json/parse (get req :body))
        username (get body "username")
        email (get body "email")]
    (if (validate-user username email)
      (server/json {:success true :user {...}} 201)
      (server/json {:error "Invalid input"} 400))))
```

### 3. Swaggerエンドポイントの統合

既存のルーターにSwaggerエンドポイントを追加：

```qi
(def router
  (openapi/with-swagger
    api-router
    {:title "My API"
     :version "1.0.0"
     :description "REST API for my application"}))

(server/serve router {:port 3000})
```

これで`http://localhost:3000/api/swagger.json`にアクセスすると、OpenAPI仕様が取得できます。

## API リファレンス

### `defapi` マクロ

APIエンドポイントを定義し、自動的にレジストリに登録します。

**構文**:
```qi
(defapi method path spec handler-name & body)
```

**引数**:
- `method`: HTTPメソッド (`:get`, `:post`, `:put`, `:delete`, `:patch`)
- `path`: エンドポイントパス (文字列)
- `spec`: OpenAPI仕様のmap
  - `:summary` - エンドポイントの概要
  - `:description` - 詳細説明
  - `:tags` - タグのリスト
  - `:requestBody` - リクエストボディの仕様
  - `:responses` - レスポンスの仕様
- `handler-name`: 定義するハンドラー関数名
- `body`: ハンドラー関数の本体

**例**:
```qi
(defapi :get "/api/users/{id}"
  {:summary "ユーザー情報取得"
   :parameters [{
     :name "id"
     :in "path"
     :required true
     :schema {:type "integer"}}]
   :responses {
     200 {:description "OK"}
     404 {:description "Not Found"}}}
  get-user-handler

  (let [id (get-path-param req "id")]
    (if-let [user (find-user-by-id id)]
      (server/json {:user user})
      (server/json {:error "User not found"} 404))))
```

### `generate` 関数

OpenAPI 3.0仕様を生成します。

**構文**:
```qi
(generate info)
```

**引数**:
- `info`: APIのメタ情報map
  - `:title` - APIタイトル (必須)
  - `:version` - APIバージョン (必須)
  - `:description` - API説明 (オプション)

**戻り値**: OpenAPI 3.0仕様のmap

**例**:
```qi
(def spec (openapi/generate
  {:title "My API"
   :version "1.0.0"
   :description "A sample REST API"}))
```

### `swagger-endpoint` 関数

Swagger JSONを返すハンドラー関数を生成します。

**構文**:
```qi
(swagger-endpoint info)
```

**引数**:
- `info`: APIのメタ情報map

**戻り値**: HTTPハンドラー関数

**例**:
```qi
(def swagger-handler
  (openapi/swagger-endpoint {:title "My API" :version "1.0.0"}))

;; ルーターで使用
(defn router [req]
  (if (= (get req :path) "/api/docs")
    (swagger-handler req)
    (api-router req)))
```

### `with-swagger` 関数

既存のルーターにSwaggerエンドポイントを統合します。

**構文**:
```qi
(with-swagger base-router info)
(with-swagger base-router info swagger-path)
```

**引数**:
- `base-router`: 既存のルーター関数
- `info`: APIのメタ情報map
- `swagger-path`: Swagger JSONのパス (デフォルト: `"/api/swagger.json"`)

**戻り値**: 新しいルーター関数

**例**:
```qi
;; デフォルトパス (/api/swagger.json)
(def router (openapi/with-swagger api-router api-info))

;; カスタムパス
(def router (openapi/with-swagger api-router api-info "/docs/openapi.json"))
```

### `clear-registry` 関数

レジストリをクリアします（主にテスト用）。

**構文**:
```qi
(clear-registry)
```

## 完全な使用例

```qi
;; OpenAPIライブラリをインポート
(use "openapi" :as openapi)

;; ========================================
;; APIエンドポイント定義
;; ========================================

;; ユーザー一覧取得
(openapi/defapi :get "/api/users"
  {:summary "ユーザー一覧取得"
   :tags ["users"]
   :responses {
     200 {:description "OK"
          :content {"application/json" {
            :schema {:type "array"}}}}}}
  api-get-users

  (let [users (db/query "SELECT * FROM users")]
    (server/json {:users users})))

;; ユーザー登録
(openapi/defapi :post "/api/users"
  {:summary "ユーザー登録"
   :tags ["users"]
   :requestBody {
     :required true
     :content {"application/json" {
       :schema {
         :type "object"
         :properties {
           "username" {:type "string"}
           "email" {:type "string"}}
         :required ["username" "email"]}}}}
   :responses {
     201 {:description "Created"}
     400 {:description "Bad Request"}}}
  api-create-user

  (let [body (json/parse (get req :body))]
    (match (create-user body)
      {:error e} -> (server/json {:error e} 400)
      user -> (server/json {:success true :user user} 201))))

;; ユーザー情報取得
(openapi/defapi :get "/api/users/{id}"
  {:summary "ユーザー情報取得"
   :tags ["users"]
   :parameters [{
     :name "id"
     :in "path"
     :required true
     :schema {:type "integer"}}]
   :responses {
     200 {:description "OK"}
     404 {:description "Not Found"}}}
  api-get-user

  (let [id (parse-int (get-path-param req "id"))]
    (if-let [user (find-user id)]
      (server/json {:user user})
      (server/json {:error "User not found"} 404))))

;; ========================================
;; ルーター設定
;; ========================================

;; APIルーター（既存）
(defn api-router [req]
  (match [(get req :method) (get req :path)]
    [:get "/api/users"] -> (api-get-users req)
    [:post "/api/users"] -> (api-create-user req)
    [:get (re-matches #"/api/users/(\d+)" _)] -> (api-get-user req)
    _ -> (server/json {:error "Not Found"} 404)))

;; Swaggerエンドポイントを統合
(def router
  (openapi/with-swagger
    api-router
    {:title "User Management API"
     :version "1.0.0"
     :description "REST API for user management"}))

;; サーバー起動
(server/serve router {:port 3000})

(println "Server started on http://localhost:3000")
(println "Swagger JSON: http://localhost:3000/api/swagger.json")
```

## Swagger UIとの統合

生成されたSwagger JSONをSwagger UIで表示する例：

```qi
;; static/index.html に Swagger UI を配置

(defn router [req]
  (let [path (get req :path)]
    (cond
      (= path "/api/swagger.json")
        ((openapi/swagger-endpoint api-info) req)

      (= path "/docs")
        (server/html (slurp "static/swagger-ui.html"))

      (str/starts-with? path "/api/")
        (api-router req)

      :else
        (server/json {:error "Not Found"} 404))))
```

## OpenAPI仕様の詳細

### リクエストボディの定義

```qi
:requestBody {
  :required true
  :content {
    "application/json" {
      :schema {
        :type "object"
        :properties {
          "name" {:type "string" :minLength 1}
          "age" {:type "integer" :minimum 0}
          "email" {:type "string" :format "email"}}
        :required ["name" "email"]}}}}
```

### レスポンスの定義

```qi
:responses {
  200 {
    :description "成功"
    :content {
      "application/json" {
        :schema {
          :type "object"
          :properties {
            "success" {:type "boolean"}
            "data" {:type "object"}}}}}}
  400 {:description "リクエストが不正"}
  404 {:description "リソースが見つからない"}
  500 {:description "サーバーエラー"}}
```

### パラメータの定義

```qi
:parameters [
  {:name "id"
   :in "path"
   :required true
   :schema {:type "integer"}}

  {:name "limit"
   :in "query"
   :required false
   :schema {:type "integer" :default 10}}

  {:name "Authorization"
   :in "header"
   :required true
   :schema {:type "string"}}]
```

## ベストプラクティス

### 1. タグでエンドポイントをグループ化

```qi
(openapi/defapi :get "/api/users" {:tags ["users"]} ...)
(openapi/defapi :post "/api/users" {:tags ["users"]} ...)
(openapi/defapi :get "/api/posts" {:tags ["posts"]} ...)
```

### 2. 共通のスキーマを定義

```qi
(def user-schema
  {:type "object"
   :properties {
     "id" {:type "integer"}
     "username" {:type "string"}
     "email" {:type "string" :format "email"}}})

(openapi/defapi :get "/api/users"
  {:responses {200 {:content {"application/json" {:schema user-schema}}}}}
  ...)
```

### 3. エラーレスポンスの統一

```qi
(def error-response
  {:type "object"
   :properties {
     "error" {:type "string"}
     "message" {:type "string"}}})

(openapi/defapi :post "/api/users"
  {:responses {
     201 {...}
     400 {:description "Bad Request"
          :content {"application/json" {:schema error-response}}}}}
  ...)
```

## トラブルシューティング

### Q: Swagger JSONが空になる

A: `defapi`でエンドポイントを定義していることを確認してください。通常の`defn`では登録されません。

### Q: パス パラメータが認識されない

A: OpenAPI仕様では`{id}`形式でパスパラメータを定義し、`:parameters`で型を指定する必要があります。

```qi
(openapi/defapi :get "/api/users/{id}"
  {:parameters [{:name "id" :in "path" :schema {:type "integer"}}]}
  ...)
```

### Q: ネストしたスキーマの定義方法は？

A: mapをネストして定義できます：

```qi
:schema {
  :type "object"
  :properties {
    "user" {
      :type "object"
      :properties {
        "name" {:type "string"}
        "address" {
          :type "object"
          :properties {
            "city" {:type "string"}
            "zip" {:type "string"}}}}}}}
```

## 関連資料

- [OpenAPI 3.0 Specification](https://swagger.io/specification/)
- [Swagger UI](https://swagger.io/tools/swagger-ui/)
- [Qi HTTP Server Documentation](../../docs/spec/11-stdlib-http.md)
