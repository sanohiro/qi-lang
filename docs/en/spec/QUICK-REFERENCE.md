# Qi Language Quick Reference

**Learn Qi Basics in One Page**

---

## 📌 Basic Syntax

### Data Types

```qi
42                ;; Integer
3.14              ;; Float
"hello"           ;; String
f"Hello, {name}"  ;; f-string (string interpolation)
true / false      ;; Boolean
nil               ;; nil
:keyword          ;; Keyword
[1 2 3]           ;; Vector
'(1 2 3)          ;; List (quote required)
{:name "Alice"}   ;; Map
```

### Definitions

```qi
(def x 42)                          ;; Variable definition
(defn greet [name] (str "Hello, " name))  ;; Function definition
(let [x 10 y 20] (+ x y))          ;; Local binding
```

### Control Structures

```qi
(if (> x 10) "big" "small")        ;; if
(do (println "1") (println "2"))   ;; Sequential execution
(loop [i 0] (if (< i 10) (recur (inc i)) i))  ;; Loop
```

---

## ⚡ Pipeline Operators (★Key Feature)

```qi
;; |> - Sequential pipeline
(data |> parse |> transform |> save)

;; |>? - Railway Pipeline (error handling)
(input |>? validate |>? parse |>? process)
;; Short-circuits on {:error ...}, otherwise success

;; ||> - Parallel pipeline (automatically uses pmap)
([1 2 3 4] ||> heavy-process)  ;; Parallel execution

;; ~> - Async pipeline (goroutine-style)
(def result (data ~> transform))
(go/recv! result)

;; tap> - Side-effect tap (for debugging)
(data |> parse |> (tap print) |> save)
```

---

## 🔀 Pattern Matching (★Key Feature)

```qi
(match value
  {:ok data} -> (process data)
  {:error e} -> (log e)
  _ -> "default")

;; Guard conditions
(match x
  n when (> n 0) -> "positive"
  n when (< n 0) -> "negative"
  _ -> "zero")

;; Vector destructuring
(match [1 2 3]
  [a b c] -> (+ a b c))  ;; => 6
```

---

## 🚀 Concurrency and Parallelism (★Key Feature)

### Goroutine-style

```qi
;; Channel creation
(def ch (go/chan))

;; Send/receive
(go/send! ch 42)
(def val (go/recv! ch))  ;; => 42

;; Execute in goroutine
(go/run (println "async!"))
```

### Parallel map/filter/reduce

```qi
(pmap (fn [x] (* x 2)) [1 2 3 4])     ;; Parallel map
(pfilter even? [1 2 3 4])              ;; Parallel filter
(preduce + [1 2 3 4] 0)                ;; Parallel reduce (fn collection init)
```

### Atom (thread-safe state management)

```qi
(def counter (atom 0))
(swap! counter inc)        ;; => 1
(reset! counter 0)         ;; => 0
(deref counter)            ;; => 0 or @counter
```

---

## 📦 Collection Operations

### Access

```qi
(first [1 2 3])            ;; => 1
(rest [1 2 3])             ;; => (2 3)
(last [1 2 3])             ;; => 3
(nth [10 20 30] 1)         ;; => 20
```

### Transformation

```qi
(map inc [1 2 3])          ;; => [2 3 4]
(filter even? [1 2 3 4])   ;; => [2 4]
(reduce + 0 [1 2 3])       ;; => 6
(take 2 [1 2 3 4])         ;; => [1 2]
(drop 2 [1 2 3 4])         ;; => [3 4]
```

### Concatenation & Sorting

```qi
(concat [1 2] [3 4])       ;; => [1 2 3 4]
(cons 0 [1 2 3])           ;; => [0 1 2 3]
(sort [3 1 4])             ;; => [1 3 4]
(reverse [1 2 3])          ;; => [3 2 1]
(distinct [1 2 2 3])       ;; => [1 2 3]
```

---

## 🔍 Predicate Functions

```qi
;; Type checking
(nil? x) (number? x) (string? x) (list? x) (vector? x) (map? x)

;; State checking
(some? x)      ;; Not nil
(empty? coll)  ;; Empty collection
(error? x)     ;; {:error ...} format

;; Numeric predicates
(even? 2) (odd? 3) (positive? 1) (negative? -1) (zero? 0)
```

---

## ⚠️ Error Handling

### Railway Pipeline (Recommended)

```qi
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}
    (/ x y)))

(10 |>? (fn [x] (divide 100 x)))  ;; => 10
(0 |>? (fn [x] (divide 100 x)))   ;; => {:error "division by zero"}

;; Check with error? predicate
(if (error? result)
  (log "Error")
  (process result))
```

### try/catch

```qi
(match (try (risky-operation))
  {:error e} -> (log e)
  result -> result)
```

### defer (Resource management)

```qi
(defn process-file [path]
  (let [f (open path)]
    (do
      (defer (close f))  ;; Always executed when function exits
      (read f))))
```

---

## 🌐 HTTP & JSON

### HTTP Client

```qi
;; Simple version (body only)
(def resp (http/get "https://api.example.com/data"))
(def data (json/parse resp))

;; Detailed version (status, headers, body)
(def resp (http/get! "https://api.example.com/data"))
(def data (json/parse (get resp :body)))
```

### HTTP Server

```qi
(defn handler [req]
  (server/json {:message "Hello, World!"}))

(comment
  (server/serve handler {:port 3000}))
```

### JSON

```qi
(json/parse "{\"name\":\"Alice\"}")  ;; => {:name "Alice"}
(json/stringify {:name "Bob"})       ;; => "{\"name\":\"Bob\"}"
```

---

## 📁 File I/O

```qi
(io/read-file "data.txt")                ;; Read file
(io/write-file "output.txt" "content")   ;; Write file
(io/read-lines "data.txt")               ;; Read by lines
```

---

## 🧮 Math Functions

```qi
(math/pow 2 3)      ;; => 8
(math/sqrt 16)      ;; => 4.0
(math/round 3.14)   ;; => 3.0
(math/rand)         ;; Random [0.0, 1.0)
(math/rand-int 10)  ;; Random integer [0, 10)
```

---

## 🧪 Testing

```qi
(test/assert-eq (+ 1 2) 3)
(test/assert (> 5 3))
(test/assert-throws (fn [] (error "test")))

;; Run tests
(test/run)
```

---

## 💡 Tips

### When to Use Lists vs Vectors

- **Vectors `[...]`**: Default (JSON-compatible, fast)
- **Lists `'(...)`**: Recursive processing, Lisp-style operations

### Parallelization Guidelines

- **Use when**: CPU-intensive, I/O-heavy, 100+ elements
- **Don't use when**: Lightweight operations, fewer than 10 elements

### Error Handling Selection

- **Railway Pipeline (`|>?`)**: API calls, file I/O, parsing
- **try/catch**: Unexpected errors, third-party libraries

---

## 📚 Detailed Documentation

For complete reference, see [docs/spec/](.) directory.

- [02-flow-pipes.md](02-flow-pipes.md) - Pipeline operators
- [03-concurrency.md](03-concurrency.md) - Concurrency & parallelism
- [04-match.md](04-match.md) - Pattern matching
- [06-data-structures.md](06-data-structures.md) - Data structures
- [08-error-handling.md](08-error-handling.md) - Error handling
