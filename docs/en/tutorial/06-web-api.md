# Chapter 6: Web Applications and APIs

**Time Required**: 40 minutes

Learn how to build HTTP servers and JSON APIs with Qi. You can create **production-ready web applications** with simple, readable code.

---

## Minimal HTTP Server

Let's start by creating the simplest HTTP server.

```qi
(defn handler [req]
  (server/text "Hello, World!"))

(server/serve handler {:port 3000})
; => Server started: http://localhost:3000
```

Access `http://localhost:3000` in your browser to see `Hello, World!`.

---

## Requests and Responses

### Request Structure

The handler receives a request map like this:

```qi
{:method :get
 :path "/users/123"
 :headers {"content-type" "application/json"}
 :body "..."}
```

### Response Types

Qi provides convenient response helpers.

```qi
; Text response
(server/text "Hello")
; => {:status 200 :headers {"Content-Type" "text/plain"} :body "Hello"}

; JSON response
(server/json {"message" "Success" "data" [1 2 3]})
; => {:status 200 :headers {"Content-Type" "application/json"} :body "..."}

; HTML response
(server/html "<h1>Welcome</h1>")
; => {:status 200 :headers {"Content-Type" "text/html"} :body "..."}

; Custom status
(server/response 201 {"message" "Created"})
; => {:status 201 ...}
```

---

## Routing

Branch processing based on the path.

```qi
(defn handler [req]
  (match (get req :path)
    "/" -> (server/text "Home")
    "/about" -> (server/text "About")
    "/api/status" -> (server/json {:status "ok"})
    _ -> (server/response 404 "Not Found")))

(server/serve handler {:port 3000})
```

### Combining Method and Path

```qi
(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["GET" "/"] -> (server/text "Home")
    ["GET" "/users"] -> (server/json {:users []})
    ["POST" "/users"] -> (server/json {:message "User created"})
    ["GET" "/users/123"] -> (server/json {:id 123 :name "Alice"})
    _ -> (server/response 404 "Not Found")))

(server/serve handler {:port 3000})
```

---

## JSON API

### GET: Retrieve Data

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

(server/serve handler {:port 3000})
```

**Testing**:
```bash
curl http://localhost:3000/api/users
# => {"users":[{"id":1,"name":"Alice","age":25},...]}

curl http://localhost:3000/api/users/1
# => {"id":1,"name":"Alice","age":25}

curl http://localhost:3000/api/users/999
# => {"error":"User not found"}
```

### POST: Create Data

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

(server/serve handler {:port 3000})
```

**Testing**:
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","age":25}'
# => {"message":"User created","user":{"name":"Alice","age":25}}
```

---

## Error Handling

Use Railway Pipeline to handle errors gracefully.

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

## Practical Example: CRUD API

Create a complete CRUD (Create, Read, Update, Delete) API.

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

; Routing
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

(server/serve handler {:port 3000})
```

**Testing**:
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

## Middleware

Functions that perform pre-processing and post-processing of requests.

### Logger Middleware

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

(server/serve app {:port 3000})
```

### CORS Middleware

```qi
(defn cors [handler]
  (fn [req]
    (let [resp (handler req)]
      (assoc resp :headers
        (merge (get resp :headers)
               {:access-control-allow-origin "*"
                :access-control-allow-methods "GET, POST, PUT, DELETE"})))))

(def app (cors my-handler))

(server/serve app {:port 3000})
```

### Middleware Composition

```qi
(def app
  (-> my-handler
      logger
      cors))

(server/serve app {:port 3000})
```

### Authentication Middleware (JWT)

Example implementation of JWT (JSON Web Token) authentication middleware.

```qi
;; Extract token from Authorization header
(defn extract-auth-token [request]
  (let [auth-header (get-in request [:headers :authorization])]
    (if (nil? auth-header)
      nil
      (if (string/starts-with? auth-header "Bearer ")
        (string/replace-first auth-header "Bearer " "")
        nil))))

;; Middleware for protected endpoints
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

;; Protected endpoint
(defn handle-profile [request]
  (let [user (get request :user)]
    (server/json {:user user :message "This is a protected resource"})))

;; Routing
(defn handler [req]
  (match [(get req :method) (get req :path)]
    ["POST" "/api/login"] -> (handle-login req)
    ["GET" "/api/profile"] -> ((require-auth handle-profile) req)
    _ -> (server/response 404 "Not Found")))

(server/serve handler {:port 3000})
```

**Authentication flow example**:
```bash
# 1. Login and get token
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
# => {"token":"eyJ0eXAi..."}

# 2. Access protected resource with token
curl http://localhost:3000/api/profile \
  -H "Authorization: Bearer eyJ0eXAi..."
# => {"user":{"user_id":1,"username":"alice"},"message":"This is a protected resource"}

# 3. Access without token (401 error)
curl http://localhost:3000/api/profile
# => {"error":"Missing authorization token"}
```

**Detailed implementation examples**: See `examples/17-jwt-auth.qi` and `examples/19-auth-api.qi`.

---

## Practical Example: Simple Blog API

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

(server/serve (logger (cors handler)) {:port 3000})
```

---

## Practice Problems

### Problem 1: Simple Counter API

Create an access counter API.

```qi
; GET /api/count - Return current count
; POST /api/count/increment - Increment count by 1
; POST /api/count/reset - Reset count to 0
```

<details>
<summary>Solution</summary>

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

(server/serve handler {:port 3000})
```

</details>

### Problem 2: ToDo API

Create a simple ToDo API.

```qi
; GET /api/todos - Get all ToDos
; POST /api/todos - Create new ToDo
; PUT /api/todos/:id/complete - Mark ToDo as complete
```

<details>
<summary>Solution</summary>

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

(server/serve handler {:port 3000})
```

</details>

---

## Summary

What you learned in this chapter:

- ✅ HTTP server basics
- ✅ Routing and request handling
- ✅ JSON API construction
- ✅ CRUD endpoint implementation
- ✅ Middleware patterns
- ✅ JWT authentication middleware
- ✅ Error handling

---

## 🎉 Tutorial Complete!

Congratulations! You've now learned all the main features of Qi.

### What You've Learned
1. ✅ Basic syntax and data types
2. ✅ Pipeline operators
3. ✅ Pattern matching
4. ✅ Error handling (Railway Pipeline)
5. ✅ Concurrency and parallelism
6. ✅ Web applications and APIs

### Next Steps

1. **Start Your Own Project**
   - Small CLI tool
   - Web API
   - Data processing script

2. **Explore the examples/ Directory**
   - Practical code examples
   - Best practices

3. **Deep Dive into Documentation**
   - [Complete Language Specification](../spec/)
   - [Function Index](../../spec/FUNCTION-INDEX.md)

4. **Join the Community**
   - GitHub Issues
   - Discussions

---

Happy coding with Qi! 🚀
