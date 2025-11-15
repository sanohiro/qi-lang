# Standard Library - HTTP

**HTTP Client and Server**

---

## HTTP Client (http/)

Qi provides two types of HTTP client functions:

- **Simple version** (`http/get`, `http/post`, etc.): Returns response body only. **Throws exception for non-2xx status codes**
- **Detailed version** (`http/get!`, `http/post!`, etc.): Returns detailed information including status code, headers, and body. **Returns Map for all status codes** (does not throw)

### Simple Version - Response Body Only

In many cases, only the response body is needed. The simple version returns a **string**.

**Error Handling**: Non-2xx status codes (404, 500, etc.) throw exceptions.

```qi
;; http/get - HTTP GET request (body only)
(http/get "https://httpbin.org/get")
;; => "{\"args\": {}, \"headers\": {...}}"

;; http/post - HTTP POST request
(http/post "https://api.example.com/users" {"name" "Alice" "email" "alice@example.com"})
;; => "{\"id\": 123, \"name\": \"Alice\"}"

;; http/put - HTTP PUT request
(http/put "https://api.example.com/users/1" {"name" "Alice Updated"})

;; http/delete - HTTP DELETE request
(http/delete "https://api.example.com/users/1")

;; http/patch - HTTP PATCH request
(http/patch "https://api.example.com/users/1" {"email" "newemail@example.com"})

;; http/head - HTTP HEAD request
(http/head "https://api.example.com/status")

;; http/options - HTTP OPTIONS request
(http/options "https://api.example.com")

;; Simple usage - directly parse body as JSON
(def users (http/get "https://api.example.com/users" |> json/parse))

;; Error handling - 404 and 500 throw exceptions
(match (try (http/get "https://api.example.com/notfound"))
  {:error e} -> (println "Error:" e)  ;; => "Error: HTTP error 404"
  body -> (json/parse body))
```

### Detailed Version - With Status Code and Headers

When status codes or headers are needed, use the detailed version with an exclamation mark (`!`).

**Error Handling**: Returns Map for all status codes (2xx, 4xx, 5xx). Does not throw exceptions.

```qi
;; http/get! - HTTP GET request (detailed info)
(http/get! "https://httpbin.org/get")
;; => {:status 200 :headers {"content-type" "application/json" ...} :body "..."}

;; http/post! - HTTP POST request (detailed info)
(http/post! "https://api.example.com/users" {"name" "Alice"})
;; => {:status 201 :headers {...} :body "..."}

;; http/put! - HTTP PUT request (detailed info)
(http/put! "https://api.example.com/users/1" {"name" "Alice Updated"})

;; http/delete! - HTTP DELETE request (detailed info)
(http/delete! "https://api.example.com/users/1")

;; http/patch! - HTTP PATCH request (detailed info)
(http/patch! "https://api.example.com/users/1" {"email" "newemail@example.com"})

;; http/head! - HTTP HEAD request (detailed info)
(http/head! "https://api.example.com/status")

;; http/options! - HTTP OPTIONS request (detailed info)
(http/options! "https://api.example.com")

;; Check status code
(let [res (http/get! "https://api.example.com/users")]
  (if (= 200 (:status res))
    (json/parse (:body res))
    (error (str "HTTP error: " (:status res)))))

;; 404 errors also return Map (no exception)
(let [res (http/get! "https://api.example.com/notfound")]
  (println "Status:" (:status res))  ;; => "Status: 404"
  (println "Body:" (:body res)))     ;; Can retrieve error message

;; Get headers
(let [res (http/get! "https://api.example.com/data")]
  (get-in res [:headers "content-type"]))
```

### Optional Parameters

Both simple and detailed versions support optional parameters:

- **`:headers`** - Custom HTTP headers (Map)
- **`:timeout`** - Timeout in milliseconds (Integer)
- **`:basic-auth`** - Basic authentication (Vector in `[username password]` format)
- **`:bearer-token`** - Bearer Token authentication (String)

```qi
;; Add custom headers to GET request
(http/get "https://api.example.com/data"
  {:headers {"X-API-Key" "your-api-key"}})

;; POST request with authentication and timeout
(http/post "https://api.example.com/users"
  {"name" "Alice"}
  {:bearer-token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
   :timeout 10000})

;; Use Basic authentication
(http/get! "https://api.example.com/protected"
  {:basic-auth ["username" "password"]})

;; Combine multiple options
(http/post! "https://api.example.com/data"
  {"key" "value"}
  {:headers {"X-Request-ID" "12345"}
   :timeout 5000})
```

### Advanced Settings

```qi
;; http/request - Custom request
(http/request {
  :method "POST"
  :url "https://api.example.com/data"
  :headers {"Authorization" "Bearer token123"}
  :body {"data" "value"}
  :timeout 5000
})
```

### Authentication

```qi
;; Basic authentication
(http/request {
  :url "https://api.example.com/data"
  :basic-auth ["username" "password"]
})

;; Bearer Token authentication
(http/request {
  :url "https://api.example.com/data"
  :bearer-token "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
})
```

### Content Compression

```qi
;; Automatic decompression (enabled by default)
(http/get "https://example.com/api")  ;; Auto-decompress gzip/deflate/brotli

;; Compression on send
(http/post "https://example.com/api"
  {"data" "large payload"}
  {:headers {"content-encoding" "gzip"}})  ;; Auto-compress body with gzip
```

### Integration with Railway Pipeline

```qi
;; Simple version pipeline - body returned directly
("https://api.github.com/users/octocat"
 |> http/get
 |> json/parse
 |> (fn [data] (get data "name")))
;; => "The Octocat"

;; Detailed version pipeline - check status code
("https://api.github.com/users/octocat"
 |> http/get!
 |> (fn [resp]
      (if (= 200 (:status resp))
        (:body resp)
        (error (str "HTTP " (:status resp)))))
 |> json/parse
 |> (fn [data] (get data "name")))
;; => "The Octocat"

;; Error handling (catch exceptions with try)
(match (try
         ("https://api.github.com/users/octocat"
          |> http/get
          |> json/parse
          |> (fn [data] (get data "name"))))
  {:error e} -> (log/error "Failed:" e)
  name -> name)
;; => "The Octocat" (on success) or {:error ...} (on failure)

;; Error handling with Railway operator
("https://api.github.com/users/octocat"
 |> (try (http/get _))
 |>? json/parse
 |>? (fn [data] (get data "name")))
;; => "The Octocat" (on success) or {:error ...} (on failure)
```

---

## HTTP Server (server/)

**Flow-Oriented Web Application Development**

### Response Helpers

```qi
;; server/ok - 200 OK response
(server/ok "Hello, World!")
;; => {:status 200 :headers {...} :body "Hello, World!"}

;; server/json - JSON response
(server/json {"message" "hello" "status" "success"})
;; => {:status 200 :headers {"Content-Type" "application/json"} :body "{...}"}

;; server/not-found - 404 Not Found response
(server/not-found "Page not found")

;; server/no-content - 204 No Content response
(server/no-content)
```

### Routing

```qi
;; server/router - Create router
(server/router [["/" {:get hello-handler}]
                ["/api/users" {:get list-users :post create-user}]
                ["/api/users/:id" {:get get-user}]])

;; server/serve - Start server (wrapped in comment to prevent actual server startup)
(comment
  (server/serve app {:port 3000})
  ;; => HTTP server started on http://127.0.0.1:3000

  ;; server/serve - Detailed settings
  (server/serve app {:port 8080 :host "0.0.0.0" :timeout 30})
  ;; => HTTP server started on http://0.0.0.0:8080 (timeout: 30s)
  )
```

### Middleware

```qi
;; server/with-logging - Log requests/responses
(def handler (server/with-logging (fn [req] (server/ok "Hello"))))

;; server/with-cors - Add CORS headers
(def handler (server/with-cors (fn [req] (server/json {...}))))

;; server/with-json-body - Auto-parse JSON body
(def handler (server/with-json-body (fn [req] (get req :json))))

;; server/with-compression - Compress response body with gzip
(def handler (server/with-compression (fn [req] (server/ok "..."))))

;; server/with-basic-auth - Basic authentication
(def handler (server/with-basic-auth (fn [req] ...) "user" "pass"))

;; server/with-bearer - Extract Bearer Token
(def handler (server/with-bearer (fn [req] (get req :token))))

;; server/with-no-cache - Add no-cache headers
(def handler (server/with-no-cache (fn [req] (server/ok "..."))))

;; server/with-cache-control - Add custom Cache-Control headers
(def handler (server/with-cache-control (fn [req] ...) "public, max-age=3600"))
```

### Static File Serving

```qi
;; server/static-file - Serve single file
(server/static-file "index.html")

;; server/static-dir - Serve directory
(server/static-dir "public")
```

---

## Practical Examples

### Simple Server

```qi
;; Handler (request -> response)
(def hello-handler
  (fn [req] (server/ok "Hello, World!")))

;; Route definition (data-driven)
(def routes [["/" {:get hello-handler}]])

;; Start app (wrapped in comment)
(def app (server/router routes))
(comment
  (server/serve app {:port 3000}))
```

### JSON API with Path Parameters

```qi
;; Handler definitions
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

;; Route definition (path parameters: /users/:id format)
(def routes
  [["/api/users" {:get list-users :post create-user}]
   ["/api/users/:id" {:get get-user}]
   ["/api/users/:user_id/posts/:post_id" {:get get-post}]])

;; Start app (wrapped in comment)
(def app (server/router routes))
(comment
  (server/serve app {:port 8080 :host "0.0.0.0" :timeout 30}))
```

### Middleware Composition

```qi
;; Stack multiple middleware
(def api-handler
  (-> (fn [req]
        (let [json-data (get req :json)]
          (server/json {"received" json-data})))
      server/with-json-body
      server/with-cors
      server/with-logging
      server/with-compression))

;; Or use comp for function composition
(def protected-api
  (comp
    server/with-logging
    server/with-cors
    (fn [handler] (server/with-basic-auth handler "admin" "secret"))
    server/with-json-body))

(def routes
  [["/api/data" {:post (protected-api handle-data)}]])
```

### Request/Response Objects

```qi
;; Request structure
{:method :get                       ;; HTTP method (keyword)
 :path "/api/users/123"             ;; Request path
 :query "page=1&limit=10"           ;; Query string (raw)
 :query-params {"page" "1"          ;; Query parameters (auto-parsed)
                "limit" "10"}
 :headers {"content-type" "application/json" ...}
 :body "..."                        ;; Request body (string)
 :params {"id" "123"}}              ;; Path parameters (when matched)

;; Response structure
{:status 200                        ;; HTTP status code
 :headers {"Content-Type" "text/plain; charset=utf-8" ...}
 :body "Hello, World!"}             ;; Response body (string, JSON, HTML, etc.)

;; Response structure (streaming)
{:status 200
 :headers {"Content-Type" "video/mp4" ...}
 :body-file "/path/to/large-file.mp4"}  ;; File path (for large files)
```

---

## Implemented Features

- ✅ **Data-driven**: Routing uses inspectable/transformable data structures
- ✅ **Pipeline**: Handlers with clear flow using `|>`
- ✅ **Composable**: Everything is a function, middleware are functions
- ✅ **Thread-safe**: Supports concurrent request processing
- ✅ **Path parameters**: Supports `/users/:id` format (multiple parameters)
- ✅ **Query parameters**: Auto-parse `?page=1&limit=10`, array support, URL decode
- ✅ **Timeout**: Configurable request timeout (default 30s)
- ✅ **Middleware**: Logging, CORS, JSON body parsing (multiple stackable)
- ✅ **Static file serving**: Supports binary files (HTML, CSS, JS, images, fonts)
- ✅ **Streaming**: Memory-efficient serving of large files (video, PDF, etc.) with `:body-file` key
- ✅ **Content compression**: Supports gzip/deflate/brotli
- ✅ **Authentication**: Basic Auth, Bearer Token extraction
- ✅ **Cache control**: Cache-Control, graceful shutdown
