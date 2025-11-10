# Concurrency & Parallelism - Qi's Core Strength

**Qi treats concurrency and parallelism as first-class citizens.**

"Making parallelism and concurrency easy is Qi's core strength" - This is the heart of Qi's design philosophy.

> **Implementation**: `src/builtins/concurrency.rs`, `src/builtins/fn.rs`

---

## Design Philosophy

Qi's concurrency and parallelism use a **3-layer architecture**:

```
┌─────────────────────────────────────┐
│  Layer 3: go/await (High-level)     │  ← Ease of use (I/O, API)
│  - go/await, go/then, go/catch      │
├─────────────────────────────────────┤
│  Layer 2: Pipeline (Mid-level)      │  ← Functional style
│  - pmap, go/pipeline, go/fan-out/in │
├─────────────────────────────────────┤
│  Layer 1: go/chan (Low-level base)  │  ← Power and flexibility
│  - go/run, go/chan, go/send!, ...   │
└─────────────────────────────────────┘
```

**All unified under go/ namespace** - Simple and consistent architecture.

---

## Layer 1: go/chan (Foundation)

**Go-style concurrency**

### Channel Creation

```qi
(go/chan)       ;; Unlimited buffer
(go/chan 10)    ;; Buffer size 10
```

### Send/Receive

```qi
;; Channel send and receive
(def ch (go/chan))
(def value 42)

(go/send! ch value)              ;; Send to channel
(go/recv! ch)                    ;; Blocking receive
(go/recv! ch :timeout 1000)      ;; Receive with timeout (milliseconds)
(go/try-recv! ch)                ;; Non-blocking receive
(go/close! ch)                   ;; Close channel
```

### Goroutines

```qi
(go/run (println "async!"))

(def result-ch (go/chan))
(go/run (go/send! result-ch (* 2 3)))
(go/recv! result-ch)  ;; 6
```

### Usage Example: Parallel Computation

```qi
;; Parallel computation with multiple goroutines
(def ch (go/chan))

(go/run (go/send! ch (* 2 3)))
(go/run (go/send! ch (* 4 5)))
(go/run (go/send! ch (* 6 7)))

[(go/recv! ch) (go/recv! ch) (go/recv! ch)]  ;; => [6 20 42]
```

### select! - Waiting on Multiple Channels

```qi
;; Process first data from multiple channels
(def ch1 (go/chan))
(def ch2 (go/chan))

(go/run (go/send! ch1 "from ch1"))
(go/run (go/send! ch2 "from ch2"))

(go/select!
  ch1 (fn [val] (println "Got from ch1:" val))
  ch2 (fn [val] (println "Got from ch2:" val)))
```

### Structured Concurrency

```qi
;; Create scope
(def ctx (go/make-scope))

;; Launch goroutine within scope
(go/scope-go ctx (fn []
  (loop [i 0]
    (if (go/cancelled? ctx)
      (println "cancelled")
      (do
        (println i)
        (sleep 100)
        (recur (inc i)))))))

;; Cancel all goroutines in scope
(go/cancel! ctx)

;; with-scope function (convenient version)
(go/with-scope (fn [ctx]
  (go/scope-go ctx task1)
  (go/scope-go ctx task2)
  ;; Auto-cancel on scope exit
  ))
```

---

## Layer 2: Pipeline (Structured Concurrency)

**Functional-style parallelism**

### Parallel Collection Operations

```qi
;; pmap - parallel map (using rayon)
([1 2 3 4 5] |> (pmap (fn [x] (* x x))))
;; => (1 4 9 16 25)

;; go/pfilter - parallel filter
([1 2 3 4 5 6] |> (go/pfilter (fn [x] (= (% x 2) 0))))
;; => (2 4 6)

;; go/preduce - parallel reduce (fn collection init)
([1 2 3 4 5] |> (fn [data] (go/preduce + data 0)))
;; => 15

;; go/parallel-do - parallel execution of multiple expressions
(go/parallel-do
  (println "Task 1")
  (println "Task 2")
  (println "Task 3"))
```

### Pipeline Processing

```qi
;; pipeline - apply xf transformation to ch with n parallelism
(def ch (go/chan))
(go/pipeline 4 (fn [x] (* x 2)) ch)
```

### Fan-out/Fan-in

```qi
;; fan-out - branch one channel into n channels
(def ch (go/chan))
(def output-chs (go/fan-out ch 3))

;; fan-in - merge multiple channels into one
(def ch1 (go/chan))
(def ch2 (go/chan))
(def ch3 (go/chan))
(def merged (go/fan-in [ch1 ch2 ch3]))
```

---

## Layer 3: go/await (High-level)

**Modern async processing**

### Basic await

```qi
(def p (go/run (fn [] (+ 1 2 3))))
(go/await p)  ;; => 6
```

### Promise Chain

```qi
(-> (go/run (fn [] 10))
    (go/then (fn [x] (* x 2)))
    (go/then (fn [x] (+ x 1)))
    (go/await))  ;; => 21
```

### Promise.all-style

```qi
(def promises [(go/run (fn [] 1)) (go/run (fn [] 2)) (go/run (fn [] 3))])
(go/await (go/all promises))  ;; => [1 2 3]
```

### Promise.race-style

```qi
(def promises [(go/run (fn [] "slow")) (go/run (fn [] "fast"))])
(go/await (go/race promises))  ;; => "fast"
```

### Error Handling

```qi
(go/catch promise (fn [e] (println "Error:" e)))
```

---

## State Management - Atom

**Thread-safe state management**

Qi uses **Atoms** for state management. Atoms provide a mechanism to have state only where needed while maintaining referential transparency.

### Basic Operations

```qi
atom                    ;; Create atom
deref                   ;; Get value
@                       ;; deref shorthand (@counter => (deref counter))
swap!                   ;; Update with function (atomic)
reset!                  ;; Set value directly
```

### Atom Creation and Reference

```qi
;; Counter
(def counter (atom 0))

;; Get value
(deref counter)  ;; 0

;; Update value
(reset! counter 10)
(deref counter)  ;; 10

;; Update with function (uses current value)
(swap! counter inc)
(deref counter)  ;; 11

(swap! counter + 5)
(deref counter)  ;; 16
```

### @ Syntax (deref shorthand)

```qi
;; deref shorthand
(deref counter)  ;; Traditional
@counter         ;; Shorthand

;; Both mean the same thing
(print (deref state))
(print @state)

;; Convenient in pipelines
(def cache (atom {:user-123 {:name "Alice"}}))
(get @cache :user-123)  ;; {:name "Alice"}

;; Also usable as function arguments
(def users (atom [{:name "Alice"} {:name "Bob"}]))
(first @users)  ;; {:name "Alice"}
(map (fn [u] (get u :name)) @users)  ;; ("Alice" "Bob")
```

### Real-world Example 1: Counter

```qi
;; Request counter
(def request-count (atom 0))

(defn handle-request [req]
  (do
    (swap! request-count inc)
    (process req)))

;; Check current count
(deref request-count)  ;; Number of processed requests
```

### Real-world Example 2: Stateful Cache

```qi
;; Cache
(def cache (atom {}))

(defn get-or-fetch [key fetch-fn]
  (let [cached (get (deref cache) key)]
    (if cached
      cached
      (let [value (fetch-fn)]
        (do
          (swap! cache assoc key value)
          value)))))

;; Usage
(get-or-fetch :user-123 (fn [] (fetch-from-db :user-123)))
```

### Real-world Example 3: Connection Management (with defer)

```qi
;; Manage active connections
(def clients (atom #{}))

(defn handle-connection [conn]
  (do
    (swap! clients conj conn)
    (defer (swap! clients dissoc conn))  ;; Guaranteed cleanup
    (process-connection conn)))

;; Active connection count
(len (deref clients))
```

### Real-world Example 4: Complex State Updates

```qi
;; Application state
(def app-state (atom {
  :users {}
  :posts []
  :status "running"
}))

;; Add user
(defn add-user [user]
  (swap! app-state (fn [state]
    (assoc state :users
      (assoc (get state :users) (get user :id) user)))))

;; Add post
(defn add-post [post]
  (swap! app-state (fn [state]
    (assoc state :posts (conj (get state :posts) post)))))

;; Check state
(deref app-state)
```

### Real-world Example 5: Thread-safe Counter

```qi
(def counter (atom 0))

;; Safe increment from multiple goroutines
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))

(sleep 100)  ;; Wait for completion
(deref counter)  ;; => 3
```

### Atom Design Philosophy

1. **Local state**: Use Atoms only where needed instead of global variables
2. **Atomic swap!**: Updates are guaranteed to apply (avoid race conditions)
3. **Coexistence with functional**: Combine pure functions with Atoms
4. **Great with defer**: Powerful for resource management

---

## Implementation Technology Stack

- **crossbeam-channel**: Go-style channel implementation (also provides select! macro)
- **rayon**: Data parallelism (pmap, go/pfilter, go/preduce, etc.)
- **parking_lot**: High-performance RwLock
- **Arc<RwLock<_>>**: Complete thread-safety of Evaluator

---

## Function List

### Layer 1 (go/chan)

- `go/chan`: Create channel
- `go/send!`: Send
- `go/recv!`: Blocking receive
- `go/recv! :timeout`: Receive with timeout
- `go/try-recv!`: Non-blocking receive
- `go/close!`: Close channel
- `go/run`: Launch goroutine
- `go/select!`: Wait on multiple channels
- `go/make-scope`: Create scope
- `go/scope-go`: Goroutine within scope
- `go/cancel!`: Cancel scope
- `go/cancelled?`: Check cancellation
- `go/with-scope`: Auto-manage scope

### Layer 2 (Pipeline)

- `pmap`: Parallel map
- `go/pfilter`: Parallel filter
- `go/preduce`: Parallel reduce
- `go/parallel-do`: Parallel execution of multiple expressions
- `go/pipeline`: Pipeline processing
- `go/pipeline-map`: Pipeline map
- `go/pipeline-filter`: Pipeline filter
- `go/fan-out`: Fan-out
- `go/fan-in`: Fan-in

### Layer 3 (go/await)

- `go/await`: Wait for Promise
- `go/then`: Promise chain
- `go/catch`: Error handling
- `go/all`: Execute multiple Promises in parallel
- `go/race`: Return fastest Promise

### State Management

- `atom`: Create atom
- `deref` (`@`): Get value
- `swap!`: Update with function (atomic)
- `reset!`: Set value directly
