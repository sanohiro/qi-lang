# Functions

**Functions as First-Class Objects**

In Qi, functions are first-class objects that can be assigned to variables, passed as arguments, and returned as values.

---

## Function Definition

### fn - Anonymous Functions

```qi
;; Basic function
(fn [x] (* x 2))

;; Multiple arguments
(fn [x y] (+ x y))

;; No arguments
(fn [] (println "no args"))

;; Variadic arguments
(fn [& args] (reduce + 0 args))
```

### defn - Named Functions

```qi
;; Basic form
(defn greet [name]
  (str "Hello, " name))

;; Variadic arguments
(defn sum [& nums]
  (reduce + 0 nums))

;; Vector destructuring
(defn add-pair [[x y]]
  (+ x y))

;; Map destructuring
(defn greet-user [{:name n :age a}]
  (str n " is " a " years old"))
```

---

## Closures

Functions can capture variables from their defining scope (closures).

```qi
;; Counter function generator
(defn make-counter []
  (let [count (atom 0)]
    (fn []
      (swap! count inc)
      (deref count))))

(def counter (make-counter))
(counter)  ;; => 1
(counter)  ;; => 2
(counter)  ;; => 3

;; Closure-based partial application
(defn make-adder [n]
  (fn [x] (+ x n)))

(def add5 (make-adder 5))
(add5 10)  ;; => 15
```

---

## Higher-Order Functions

### identity - Return Argument As-Is

```qi
(identity 42)                         ;; => 42

;; Filter out nil and false
(filter identity [1 false nil 2])     ;; => (1 2)
```

### constantly - Generate Function That Always Returns Same Value

```qi
(def always-42 (constantly 42))
(always-42 "anything")                ;; => 42
(always-42 1 2 3)                     ;; => 42

;; Practical example: Default value generation
(map (constantly 0) [1 2 3])          ;; => (0 0 0)
```

### apply - Apply Function with List as Arguments

```qi
(apply + [1 2 3])                     ;; => 6
(apply max [5 2 8 3])                 ;; => 8

;; Practical example: Variadic argument expansion
(defn sum-all [& nums]
  (apply + nums))

(sum-all 1 2 3 4 5)                   ;; => 15
```

### comp - Function Composition (Apply Right-to-Left)

```qi
;; Function composition
(def process (comp inc (* 2)))
(process 5)                           ;; => 11  ((5 * 2) + 1)

;; Compose multiple functions
(def transform (comp str/upper str/trim))
(transform "  hello  ")               ;; => "HELLO"

;; Comparison with pipelines
;; comp: (comp f g h) applies h → g → f
;; |>:   x |> h |> g |> f is the same processing
```

### partial - Partial Application

```qi
;; Create new function with partial application
(def add5 (partial + 5))
(add5 10)                             ;; => 15

;; Partial application with multiple arguments
(def greet-hello (partial str "Hello, "))
(greet-hello "Alice")                 ;; => "Hello, Alice"

;; Practical example: Generate filter condition
(def greater-than-10 (partial < 10))
(filter greater-than-10 [5 15 3 20])  ;; => (15 20)
```

---

## fn/ Module (Advanced Higher-Order Functions)

### fn/complement - Negate Predicate

```qi
;; Generate negated predicate function
(def odd? (fn/complement even?))
(odd? 3)                              ;; => true
(odd? 4)                              ;; => false

;; Practical example: Invert filter
(filter (fn/complement nil?) [1 nil 2 nil 3])  ;; => (1 2 3)
```

### fn/juxt - Apply Multiple Functions in Parallel

```qi
;; Apply multiple functions in parallel and return results in vector
((fn/juxt inc dec) 5)                 ;; => [6 4]

;; Practical example: Multi-faceted data analysis
(def analyze (fn/juxt min max sum len))
(analyze [1 2 3 4 5])                 ;; => [1 5 15 5]

;; Extract user data
(def extract-info (fn/juxt :name :age :email))
(extract-info {:name "Alice" :age 30 :email "alice@example.com"})
;; => ["Alice" 30 "alice@example.com"]
```

### fn/tap> - Processing with Side Effects

```qi
;; Return value as-is while executing side effect (logging, etc.)
(def log-and-pass (fn/tap> println))
(log-and-pass 42)  ;; Prints 42 and returns 42

;; Debugging in pipelines
(10
  |> (fn/tap> (fn [x] (println "Input:" x)))
  |> (* _ 2)
  |> (fn/tap> (fn [x] (println "Doubled:" x)))
  |> (+ _ 5))
;; Output:
;; Input: 10
;; Doubled: 20
;; => 25

;; Counter implementation
(def counter (atom 0))
(def count-and-pass
  (fn/tap> (fn [_] (reset! counter (+ (deref counter) 1)))))

(map count-and-pass [1 2 3 4 5])
;; => (1 2 3 4 5)
(deref counter)  ;; => 5
```

---

## Practical Examples

### Data Transformation Pipeline

```qi
;; Process data with function composition
(def process-text
  (comp
    str/upper
    str/trim
    (partial str/replace _ "!" ".")))

(process-text "  hello world!  ")     ;; => "HELLO WORLD."
```

### Filter Combination

```qi
;; Filter with multiple conditions
(def valid-user?
  (fn [user]
    (and
      ((complement nil?) (:name user))
      (> (:age user) 18))))

(filter valid-user?
  [{:name "Alice" :age 30}
   {:name nil :age 20}
   {:name "Bob" :age 15}])
;; => ({:name "Alice" :age 30})
```

### Debugging with Higher-Order Functions

```qi
;; Use tap to observe data flow
(defn debug [label]
  (fn [x]
    (println label x)
    x))

([1 2 3]
 |> (map inc)
 |> ((debug "after map:"))
 |> sum)
;; Output: after map: (2 3 4)
;; => 9
```

### Currying-Style Function Definition

```qi
;; Apply multiple arguments in stages
(defn make-multiplier [n]
  (fn [x] (* x n)))

(def double (make-multiplier 2))
(def triple (make-multiplier 3))

(double 5)  ;; => 10
(triple 5)  ;; => 15

;; Use in pipeline
([1 2 3] |> (map double))  ;; => (2 4 6)
```
