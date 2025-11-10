# Pattern Matching

**Control structure for branching data flow**

Qi's pattern matching isn't just conditional branching - it routes processing while decomposing, transforming, and validating data structures.

---

## Basic Patterns

### Value Matching

```qi
(match x
  0 -> "zero"
  1 -> "one"
  n -> (str "other: " n))
```

### nil/bool Distinction

```qi
(match result
  nil -> "not found"
  false -> "explicitly false"
  true -> "success"
  v -> (str "value: " v))
```

### Map Matching

```qi
(match data
  {:type "user" :name n} -> (greet n)
  {:type "admin"} -> "admin"
  _ -> "unknown")
```

### List Matching

```qi
(match lst
  [] -> "empty"
  [x] -> x
  [x ...rest] -> (str "first: " x))
```

### Guard Conditions

```qi
(match x
  n when (> n 0) -> "positive"
  n when (< n 0) -> "negative"
  _ -> "zero")
```

---

## Extended Patterns

### 1. `:as` Binding - Use Both Parts and Whole

You can bind the whole matched pattern to a variable.

```qi
;; Basic :as usage
(match data
  {:user {:name n :age a} :as u} -> (do
    (log u)           ;; Log the whole
    (process n a)))   ;; Process parts

;; Works with nested structures
(match response
  {:body {:user u :posts ps} :as body} -> (cache body)
  {:error e :as err} -> (log err))

;; Deeply nested :as
(match {:type "person" :data {:name "Alice" :age 30}}
  {:type t :data {:name n :age a :as user-data} :as record} ->
    (do
      (println f"Type: {t}")           ;; "Type: person"
      (println f"Name: {n}, Age: {a}") ;; "Name: Alice, Age: 30"
      (println f"User data: {user-data}") ;; {:name "Alice", :age 30}
      (println f"Full record: {record}"))) ;; Full map

;; Combining vectors and maps
(match [1 {:x 10 :y 20}]
  [a {:x b :as inner}] -> [a b inner])
;; => [1 10 {:x 10 :y 20}]

;; Also works in function parameters
(defn process [{:name n :age a :as user}]
  (do
    (println f"Processing: {n}")
    user))  ;; Return whole

;; Complex nesting example
(defn handle-request [{:headers h :body {:user u :data d :as body} :as req}]
  (do
    (log req)      ;; Log full request
    (cache body)   ;; Cache body only
    (process-user u d)))
```

### 2. `or` Pattern - Same Processing for Multiple Patterns

You can match multiple values for the same processing (using `|` notation).

```qi
;; Match multiple values
(match status
  200 | 201 | 204 -> "success"
  400 | 401 | 403 -> "client error"
  500 | 502 | 503 -> "server error"
  _ -> "unknown")

;; Works with strings
(match day
  "月" | "火" | "水" | "木" | "金" -> "平日"
  "土" | "日" -> "週末")

;; Works with keywords
(match result
  :ok | :success -> (handle-ok)
  :error | :fail -> (handle-error))
```

### 3. Nesting + Guard - Structural Conditional Branching

You can combine deep nesting with guard conditions.

```qi
;; Readable even with deep nesting
(match request
  {:user {:age a :country c}} when (and (>= a 18) (= c "JP")) -> (allow)
  {:user {:age a}} when (< a 18) -> (error "age restriction")
  _ -> (deny))

;; Flow-style reading: Decompose data structure → Validate with guard → Process
```

### 4. Wildcard `_` - Extract Only What Matters

Extract only needed parts, ignoring the rest with `_`.

```qi
;; Use only some fields
(match data
  {:user {:name _ :age a :city c}} -> (process-location a c)
  {:error _} -> "error occurred")

;; Skip parts of list
(match coords
  [_ y _] -> y  ;; Extract only y coordinate
  _ -> 0)
```

### 5. Array Multiple Binding

You can bind multiple elements simultaneously.

```qi
;; Bind multiple elements at once
(match data
  [{:id id1} {:id id2}] -> (compare id1 id2)
  [first ...rest] -> (process-batch first rest))

;; Combine with pipelines
(match (coords |> (take 2))
  [x y] -> (distance x y)
  _ -> 0)
```

---

## match Design Philosophy

1. **Branch data flow**: match isn't just if-else, it decomposes data structures to create flows
2. **Readability first**: Patterns readable top-to-bottom, conditions at a glance
3. **Progressive disclosure**: Start with basic patterns, use extensions as needed

---

## Combining with try

`try` catches exceptions and returns `{:error e}` on error. Success values are returned as-is.

```qi
;; Values as-is on success, {:error e} on error
(match (try (risky-operation))
  {:error e} -> (log e)
  result -> result)

;; Combine with pipelines
(match (try
         (url
          |> http-get
          |> parse
          |> process))
  {:error e} -> []
  data -> data)
```

---

## Real-world Examples

### Example 1: HTTP Response Handling

```qi
;; http/get! may throw exceptions, so catch with try
(match (try (http/get! url))  ;; Detailed version to get status code
  {:error e} -> (log-error e)
  {:status 200 :body body} -> (process-body body)
  {:status 404} -> nil
  {:status s} -> (error (str "Unexpected status: " s)))
```

### Example 2: Data Validation

```qi
(match user
  {:name n :age a :email e} when (and (> a 0) (str/contains? e "@")) -> (save-user user)
  {:name _ :age a} when (<= a 0) -> (error "Invalid age")
  {:name _ :email e} when (not (str/contains? e "@")) -> (error "Invalid email")
  _ -> (error "Missing required fields"))
```

### Example 3: List Processing

```qi
(defn process-list [lst]
  (match lst
    [] -> "empty"
    [x] -> (str "single: " x)
    [x y] -> (str "pair: " x ", " y)
    [x y ...rest] -> (str "multiple: " x ", " y ", and " (len rest) " more")))
```
