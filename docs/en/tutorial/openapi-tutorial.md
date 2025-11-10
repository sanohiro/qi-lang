# OpenAPI Library Tutorial

This tutorial teaches you how to build REST APIs with automatic OpenAPI specification generation using Qi's `std/lib/openapi.qi` library.

## Prerequisites

- Qi installed
- Basic knowledge of HTTP servers
- Basic knowledge of REST APIs

## Step 1: Create a Simple API Server

First, create a normal API server without using the OpenAPI library.

```qi
;; simple-api.qi

;; Get users list
(defn api-get-users [req]
  (server/json {:users [{:id 1 :name "Alice"} {:id 2 :name "Bob"}]}))

;; Create user
(defn api-create-user [req]
  (let [body (json/parse (get req :body))]
    (server/json {:success true :user body} 201)))

;; Router
(defn router [req]
  (match [(get req :method) (get req :path)]
    [:get "/api/users"] -> (api-get-users req)
    [:post "/api/users"] -> (api-create-user req)
    _ -> (server/json {:error "Not Found"} 404)))

;; Start server
(server/serve router {:port 3000})
(println "Server started on http://localhost:3000")
```

This works, but there's no API specification documentation.

## Step 2: Introduce the OpenAPI Library

Use the OpenAPI library to add documentation to the same API.

```qi
;; openapi-demo.qi

;; Import OpenAPI library (simple import)
(use "openapi" :as openapi)

;; ========================================
;; API Endpoint Definitions (OpenAPI-enabled)
;; ========================================

;; Get users list
(openapi/defapi :get "/api/users"
  {:summary "Get users list"
   :tags ["users"]
   :responses {
     200 {:description "Success"
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

;; Create user
(openapi/defapi :post "/api/users"
  {:summary "Create a new user"
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
     201 {:description "Created successfully"}
     400 {:description "Invalid request"}}}
  api-create-user

  (let [body (json/parse (get req :body))]
    (server/json {:success true :user body} 201)))

;; ========================================
;; Router Configuration
;; ========================================

;; API router
(defn api-router [req]
  (match [(get req :method) (get req :path)]
    [:get "/api/users"] -> (api-get-users req)
    [:post "/api/users"] -> (api-create-user req)
    _ -> (server/json {:error "Not Found"} 404)))

;; Integrate Swagger endpoint
(def router
  (openapi/with-swagger
    api-router
    {:title "User Management API"
     :version "1.0.0"
     :description "REST API for user management"}))

;; Start server
(server/serve router {:port 3000})
(println "Server started on http://localhost:3000")
(println "Swagger JSON: http://localhost:3000/api/swagger.json")
```

## Step 3: Verify Functionality

Start the server:

```bash
QI_LANG=ja qi openapi-demo.qi
```

Get Swagger JSON:

```bash
curl http://localhost:3000/api/swagger.json | jq .
```

Example output:

```json
{
  "openapi": "3.0.0",
  "info": {
    "title": "User Management API",
    "version": "1.0.0",
    "description": "REST API for user management"
  },
  "paths": {
    "/api/users": {
      "get": {
        "summary": "Get users list",
        "tags": ["users"],
        "responses": {
          "200": {
            "description": "Success",
            "content": {
              "application/json": {
                "schema": {...}
              }
            }
          }
        }
      },
      "post": {
        "summary": "Create a new user",
        "tags": ["users"],
        "requestBody": {...},
        "responses": {...}
      }
    }
  }
}
```

## Step 4: Add Path Parameters

Add an endpoint to get individual user information.

```qi
;; Get user information
(openapi/defapi :get "/api/users/{id}"
  {:summary "Get specific user information"
   :tags ["users"]
   :parameters [{
     :name "id"
     :in "path"
     :required true
     :description "User ID"
     :schema {:type "integer"}}]
   :responses {
     200 {:description "Success"
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
     404 {:description "User not found"}}}
  api-get-user

  (let [id-str (last (str/split (get req :path) "/"))
        id (to-int id-str)]
    (if (= id 1)
      (server/json {:user {:id 1 :name "Alice" :email "alice@example.com"}})
      (server/json {:error "User not found"} 404))))

;; Update router
(defn api-router [req]
  (let [path (get req :path)]
    (match [(get req :method) path]
      [:get "/api/users"] -> (api-get-users req)
      [:post "/api/users"] -> (api-create-user req)
      [:get (re-matches #"/api/users/\d+" _)] -> (api-get-user req)
      _ -> (server/json {:error "Not Found"} 404))))
```

Test:

```bash
curl http://localhost:3000/api/users/1 | jq .
```

## Step 5: Add Query Parameters

Add pagination to the user list endpoint.

```qi
(openapi/defapi :get "/api/users"
  {:summary "Get users list (with pagination)"
   :tags ["users"]
   :parameters [
     {:name "limit"
      :in "query"
      :required false
      :description "Number of items to retrieve"
      :schema {:type "integer" :default 10 :minimum 1 :maximum 100}}
     {:name "offset"
      :in "query"
      :required false
      :description "Offset"
      :schema {:type "integer" :default 0 :minimum 0}}]
   :responses {
     200 {:description "Success"}}}
  api-get-users

  (let [query-params (parse-query-string (get req :query))
        limit (or (to-int (get query-params "limit")) 10)
        offset (or (to-int (get query-params "offset")) 0)]
    (server/json
      {:users [{:id 1 :name "Alice"} {:id 2 :name "Bob"}]
       :limit limit
       :offset offset})))
```

Test:

```bash
curl "http://localhost:3000/api/users?limit=5&offset=10" | jq .
```

## Step 6: Add Authentication Headers

Add an endpoint that requires authentication.

```qi
(openapi/defapi :delete "/api/users/{id}"
  {:summary "Delete user"
   :tags ["users"]
   :parameters [
     {:name "id"
      :in "path"
      :required true
      :schema {:type "integer"}}
     {:name "Authorization"
      :in "header"
      :required true
      :description "Bearer token"
      :schema {:type "string"}}]
   :responses {
     204 {:description "Deleted successfully"}
     401 {:description "Authentication required"}
     404 {:description "User not found"}}}
  api-delete-user

  (let [auth (get (get req :headers) "authorization")]
    (if (and auth (str/starts-with? auth "Bearer "))
      (server/json {} 204)
      (server/json {:error "Unauthorized"} 401))))
```

Test:

```bash
# Without authentication (error)
curl -X DELETE http://localhost:3000/api/users/1

# With authentication (success)
curl -X DELETE http://localhost:3000/api/users/1 \
  -H "Authorization: Bearer token123"
```

## Step 7: Schema Reusability

Define common schemas and reuse them.

```qi
;; Common schema definitions
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

;; Use schemas
(openapi/defapi :get "/api/users/{id}"
  {:summary "Get user information"
   :responses {
     200 {:description "Success"
          :content {"application/json" {:schema user-schema}}}
     404 {:description "Not Found"
          :content {"application/json" {:schema error-schema}}}}}
  api-get-user
  ...)
```

## Step 8: Integrate Swagger UI

Use Swagger UI to display API documentation in a browser.

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

Add to router:

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

Open `http://localhost:3000/docs` in your browser to see Swagger UI.

## Next Steps

- [OpenAPI Library Reference](../../std/lib/openapi.md)
- [HTTP Server Documentation](../spec/11-stdlib-http.md)
- [Database Integration](../spec/17-stdlib-database.md)

## Summary

In this tutorial, you learned:

1. Basic usage of the OpenAPI library
2. Endpoint definition with the `defapi` macro
3. Defining path parameters and query parameters
4. Adding authentication headers
5. Schema reusability
6. Integration with Swagger UI

Now you can build REST APIs that automatically generate OpenAPI specifications.
