# Basic Syntax

**Qi's fundamental syntax elements**

---

## Comments

```qi
; Single-line comment
;; Typically use ;;

(def x 42)  ; End-of-line comment
```

---

## Data Types

Qi is dynamically typed with the following basic types:

- **Numbers**: Integers, floating-point numbers
- **Strings**: UTF-8 support, f-string support
- **Booleans**: `true`, `false`
- **nil**: Represents absence of value
- **Vectors**: `[1 2 3]`
- **Maps**: `{:key "value"}`
- **Lists**: `'(1 2 3)` (quote required)
- **Functions**: First-class objects
- **Keywords**: `:keyword`

---

## Literals

### Numbers

```qi
42          ;; Integer
3.14        ;; Floating-point
-10         ;; Negative number
1_000_000   ;; Underscore separator (readability)
```

### Strings

```qi
;; Basic
"hello"
"hello\nworld"      ;; Escape sequences
"say \"hello\""     ;; Quote escaping

;; Multi-line strings (Python-style)
"""
This is a
multi-line
string
"""

;; Multi-line SQL or HTML
(def query """
  SELECT name, age
  FROM users
  WHERE age >= 18
  ORDER BY name
""")
```

### f-strings (String Interpolation)

```qi
;; Basic usage
f"Hello, World!"  ;; => "Hello, World!"

;; Variable interpolation
(def name "Alice")
f"Hello, {name}!"  ;; => "Hello, Alice!"

;; Expressions
f"Result: {(+ 1 2)}"  ;; => "Result: 3"

;; List and vector interpolation
f"List: {[1 2 3]}"  ;; => "List: [1 2 3]"

;; Map access
(def user {:name "Bob" :age 30})
f"Name: {(get user :name)}, Age: {(get user :age)}"
;; => "Name: Bob, Age: 30"

;; Escaping
f"Escaped: \{not interpolated\}"  ;; => "Escaped: {not interpolated}"

;; Multi-line f-string
(def name "Alice")
(def age 30)

f"""
Name: {name}
Age: {age}
Status: Active
"""
```

### bool and nil

```qi
true
false
nil

;; Conditional expression behavior
(if nil "yes" "no")     ;; "no" (nil is falsy)
(if false "yes" "no")   ;; "no" (false is falsy)
(if 0 "yes" "no")       ;; "yes" (0 is truthy)
(if "" "yes" "no")      ;; "yes" (empty string is truthy)

;; Explicit comparison
(= x nil)               ;; nil check
(= x false)             ;; false check
```

### Keywords

```qi
:keyword
:name
:type

;; Used as map keys
{:name "Alice" :age 30}

;; Get value from map using get function
(def user {:name "Bob" :age 25})
(get user :name)  ;; => "Bob"
```

#### Internal Implementation: Interning

In Qi, **keywords and symbols are automatically interned**.

**What is interning**:
- A mechanism to store the same string in a single memory location and share it across multiple places
- Implemented using Rust's `Arc<str>`

**Benefits**:
1. **Memory efficiency**: Same keyword uses only one memory location, no matter how many times it's used
2. **Fast comparison**: Just compare pointers instead of string contents
3. **Thread-safe**: Arc allows safe sharing across multiple threads

```qi
;; ✅ Interned (recommended)
(def k1 :name)
(def k2 :name)
;; k1 and k2 point to the same memory location → fast comparison

;; Symbols are also interned
(def s1 'foo)
(def s2 'foo)
;; s1 and s2 point to the same memory location
```

**Technical details**:
```rust
// Internal implementation (Rust)
pub enum Value {
    Keyword(Arc<str>),  // :name
    Symbol(Arc<str>),   // 'symbol
    String(String),     // Regular strings (not interned)
    // ...
}
```

This design makes Qi fast and memory-efficient even when dealing with large numbers of keywords and symbols.

### Vectors

```qi
[]              ;; Empty vector
[1 2 3]         ;; Number vector
["a" "b" "c"]   ;; String vector
[1 "hello" :key]  ;; Mixed types allowed
```

### Maps

```qi
{}                          ;; Empty map
{:name "Alice" :age 30}     ;; Keywords as keys
{"name" "Bob" "age" 25}     ;; Strings as keys
```

### Lists

```qi
'()             ;; Empty list
'(1 2 3)        ;; Quote required
```

---

## Special Forms (9)

### `def` - Global Definition

```qi
(def x 42)
(def greet (fn [name] (str "Hello, " name)))
(def ops [+ - * /])
```

### `defn` - Function Definition (Sugar Syntax)

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

(defn format-kv [[k v]]
  f"{k}={v}")

;; ...rest syntax (get rest of vector elements)
(defn process-list [[first ...rest]]
  (str "first: " first ", rest: " rest))

(process-list [1 2 3 4])  ;; => "first: 1, rest: (2 3 4)"

;; Map destructuring
(defn greet [{:name n :age a}]
  (str n " is " a " years old"))

(greet {:name "Taro" :age 25})  ;; => "Taro is 25 years old"

;; Map destructuring + :as binding
(defn log-user [{:name n :as user}]
  (do
    (println f"Processing: {n}")
    user))

;; defn expands to this
(defn greet [name] body)
;; ↓
(def greet (fn [name] body))
```

### `fn` - Function Definition

```qi
(fn [x] (* x 2))
(fn [x y] (+ x y))
(fn [] (println "no args"))

;; Variadic arguments
(fn [& args] (reduce + 0 args))

;; Vector destructuring
(fn [[x y]] (+ x y))  ;; Takes 2-element vector
(fn [[k v]] f"{k}={v}")  ;; Key-value pair

;; Nested destructuring
(fn [[[a b] c]] (+ a b c))  ;; [[1 2] 3] => 6

;; ...rest syntax
(fn [[first ...rest]]
  (str "first: " first ", rest: " rest))

;; Map destructuring
(fn [{:name n :age a}]
  (str n " is " a " years old"))

;; Map destructuring + :as binding
(fn [{:name n :as user}]
  (do
    (println f"Processing: {n}")
    user))
```

### `let` - Local Binding

```qi
(let [x 10 y 20]
  (+ x y))

;; Nesting allowed
(let [a 1]
  (let [b 2]
    (+ a b)))

;; Vector destructuring
(let [[x y] [10 20]]
  (+ x y))  ;; => 30

(let [[k v] ["name" "Alice"]]
  f"{k}={v}")  ;; => "name=Alice"

;; Nested destructuring
(let [[[a b] c] [[1 2] 3]]
  (+ a b c))  ;; => 6

;; ...rest syntax
(let [[first ...rest] [1 2 3 4]]
  (str "first: " first ", rest: " rest))
;; => "first: 1, rest: (2 3 4)"

(let [[x y ...tail] [10 20 30 40]]
  {:x x :y y :tail tail})
;; => {:x 10, :y 20, :tail (30 40)}

;; Map destructuring
(let [{:name n :age a} {:name "Alice" :age 30}]
  (str n " is " a))
;; => "Alice is 30"

;; :as binding (get both parts and whole)
(let [{:name n :age a :as person} {:name "Bob" :age 25 :role "admin"}]
  [n a person])
;; => ["Bob" 25 {:name "Bob", :age 25, :role "admin"}]
```

### `do` - Sequential Execution

```qi
(do
  (println "first")
  (println "second")
  42)  ;; Returns value of last expression
```

### `if` - Conditional Branching

```qi
;; Basic form
(if test then else)

;; Practical example
(if (> x 10) "big" "small")

;; else can be omitted (returns nil)
(if (valid? data)
  (process data))

;; Nesting
(if (> x 0)
    (if (< x 10) "small positive" "big positive")
    "negative or zero")
```

### `quote` - Quote

```qi
;; Return expression as-is without evaluation
'(1 2 3)        ;; Returns as list
'(+ 1 2)        ;; Not evaluated, remains (+ 1 2)
'symbol         ;; Returns as symbol

;; Without quote causes error (list evaluated as function call)
(1 2 3)         ;; Error: 1 is not a function
```

### `mac` - Macro Definition

Macros are special functions for code generation. Use quasiquote (`` ` ``), unquote (`,`), and unquote-splice (`,@`) to create code templates.

**Qi macros are hygienic**: They only use the scope at macro definition time and cannot automatically reference local variables at the call site. This prevents variable capture and ensures safer, more predictable behavior.

#### Quasiquote / Unquote Basics

```qi
;; quasiquote (`) - Create template
`(+ 1 2)         ;; => (+ 1 2) Returns as list

;; unquote (,) - Evaluate expression in template
(def x 10)
`(+ 1 ,x)        ;; => (+ 1 10) x is evaluated

;; unquote-splice (,@) - Expand list
(def items [1 2 3])
`(list ,@items)  ;; => (list 1 2 3) items expanded
```

#### Unquote in Special Forms

Unquote works correctly inside special forms like fn, let, def:

```qi
(def value 42)

;; Unquote in fn
`(fn [x] ,value)          ;; => (fn [x] 42)
`(fn [y] (+ y ,value))    ;; => (fn [y] (+ y 42))

;; Unquote in let
`(let [x ,value] x)       ;; => (let [x 42] x)
`(let [a ,value b 10] (+ a b))  ;; => (let [a 42 b 10] (+ a b))

;; Unquote in def
`(def myvar ,value)       ;; => (def myvar 42)
```

#### Macro Implementation Examples

```qi
;; when macro - concise if + do
(mac when [test & body]
  `(if ,test
     (do ,@body)
     nil))

(when (> x 0)
  (println "positive")
  (process x))
;; Expands to: (if (> x 0) (do (println "positive") (process x)) nil)

;; unless macro - execute when condition is false
(mac unless [test & body]
  `(if ,test
     nil
     (do ,@body)))

(unless (empty? data)
  (println "has data")
  (process data))

;; debug macro - display expression and result
(mac debug [expr]
  `(let [result ,expr]
     (do
       (println f"Debug: {',expr} = {result}")
       result)))

(debug (+ 1 2))
;; Output: Debug: (+ 1 2) = 3
;; Returns: 3
```

### `loop` / `recur` - Loop

Special forms for tail recursion optimization.

```qi
;; Basic form
(loop [var1 val1 var2 val2 ...]
  body
  (recur new-val1 new-val2 ...))

;; Factorial (5! = 120)
(defn factorial [n]
  (loop [i n acc 1]
    (if (= i 0)
      acc
      (recur (dec i) (* acc i)))))

(factorial 5)  ;; 120

;; Countdown
(defn count-down [n]
  (loop [i n]
    (if (<= i 0)
      "done"
      (do
        (print i)
        (recur (dec i))))))

;; List processing
(defn sum-list [lst]
  (loop [items lst result 0]
    (if (empty? items)
      result
      (recur (rest items) (+ result (first items))))))

(sum-list [1 2 3 4 5])  ;; 15
```

**Implementation notes**:
- `loop` creates new environment and binds variables with initial values
- `recur` handled as special error, caught by `loop` to update variables
- Unlike normal recursion, doesn't consume stack (tail recursion optimization)

#### Detailed Specification and Constraints

**How loop works**:
1. Creates a dedicated new environment (scope) for the loop
2. Evaluates initial values and binds them to variables
3. Evaluates the body
4. Repeats until `recur` is called

**Constraints on recur**:
- **Must be used only in tail position of loop** - All of these are errors:
  ```qi
  ;; ❌ recur in non-tail position (Error)
  (loop [i 10]
    (if (> i 0)
      (+ (recur (dec i)) 1)  ;; Not in tail position
      0))

  ;; ✅ Correct tail position
  (loop [i 10 acc 0]
    (if (> i 0)
      (recur (dec i) (+ acc i))  ;; OK: tail position of if
      acc))
  ```
- **Number of arguments must match the number of loop variables**
  ```qi
  (loop [x 1 y 2]
    (recur x))  ;; Error: requires 2 args but only 1 provided
  ```

**Internal implementation details**:
- Qi implements `recur` as a special error message (sentinel value)
- Arguments to `recur` are evaluated beforehand and stored in thread_local
- `loop` catches this sentinel, updates variables, and re-evaluates
- This design achieves tail recursion without consuming stack space

**Performance characteristics**:
- Normal recursion: O(n) stack consumption → Risk of stack overflow
- loop/recur: O(1) stack consumption → Safe for infinite loops (as long as memory permits)

```qi
;; Normal recursion (consumes stack)
(defn factorial-recursive [n]
  (if (<= n 1)
    1
    (* n (factorial-recursive (dec n)))))

;; loop/recur (no stack consumption)
(defn factorial-loop [n]
  (loop [i n acc 1]
    (if (<= i 1)
      acc
      (recur (dec i) (* acc i)))))

;; Safe even with 1 million iterations
(factorial-loop 1000000)  ;; OK
(factorial-recursive 1000000)  ;; Stack overflow
```

### `when` - Execute Only When Condition is True

Concise notation when if's else clause is unnecessary. Can execute multiple expressions sequentially.

```qi
;; Basic form
(when test
  expr1
  expr2
  ...)

;; Practical example
(when (> x 10)
  (println "Large value")
  (process x))

;; Comparison with if
(if (> x 10)
  (do
    (println "Large value")
    (process x))
  nil)  ;; Same as this

;; Combined with pipeline
(data
 |> (when (valid? data)
      (println "Processing started")
      (process data)))
```

**Return value**:
- If condition is true: value of last expression
- If condition is false: `nil`

### `while` - Loop While Condition is True

Repeatedly executes body while condition expression is true (truthy value).

```qi
;; Basic form
(while test
  body...)

;; Counter example
(def counter (atom 0))
(while (< @counter 5)
  (println f"Count: {@counter}")
  (swap! counter inc))

;; File processing example
(def lines (atom (io/stdin-lines)))
(while (some? (first @lines))
  (println (first @lines))
  (swap! lines rest))
```

**Return value**: Always `nil`

**Caution**: To avoid infinite loops, modify condition in body.

### `until` - Loop Until Condition Becomes True

Repeatedly executes body **until** condition expression becomes true (opposite of `while`).

```qi
;; Basic form
(until test
  body...)

;; Counter example (opposite of while)
(def counter (atom 0))
(until (>= @counter 5)
  (println f"Count: {@counter}")
  (swap! counter inc))

;; Retry example
(def success (atom false))
(until @success
  (println "Retrying...")
  (reset! success (try-operation)))
```

**Return value**: Always `nil`

### `while-some` - Loop Until nil (With Binding)

Evaluates expression and repeats until result is `nil`. Binds result to variable in each iteration.

```qi
;; Basic form
(while-some [binding expr]
  body...)

;; List processing
(def remaining (atom [1 2 3 4 5]))
(while-some [item (first @remaining)]
  (println f"Processing: {item}")
  (swap! remaining rest))

;; File reading (line-by-line)
(while-some [line (io/stdin-line)]
  (line
   |> str/trim
   |> (when (> (len line) 0)
        (process-line line))))

;; Combined with pipeline
(while-some [val (get-next-value)]
  (val
   |> transform
   |> validate
   |> save))
```

**Behavior**:
- Evaluate `expr`
- If result is `nil`, terminate
- If result is not `nil`, bind value to `binding` and execute body
- Continue to next iteration

**Return value**: Always `nil`

### `until-error` - Loop Until Error (With Binding)

Evaluates expression and repeats until result is `{:error ...}`. Binds result to variable in each iteration.

```qi
;; Basic form
(until-error [binding expr]
  body...)

;; Result type integration
(until-error [result (fetch-next)]
  (println f"Fetch success: {result}")
  (process result))

;; HTTP request example
(until-error [response (http/get next-url)]
  (println f"Status: {(get response :status)}")
  (when (= (get response :status) 200)
    (process-response response)))

;; Pagination processing
(def page (atom 1))
(until-error [data (api/fetch-page @page)]
  (data
   |> process-items
   |> save-to-db)
  (swap! page inc))
```

**Behavior**:
- Evaluate `expr`
- If result is `{:error ...}`, return that value and terminate
- If result is not `{:error ...}`, bind value to `binding` and execute body
- Continue to next iteration

**Return value**: Error map `{:error ...}`, or `nil` if loop never executes

**Railway Oriented Programming**:
Powerful error handling when combined with `|>?` pipeline:

```qi
(until-error [result (fetch-data)]
  (result
   |>? validate
   |>? transform
   |>? save))
```

---

## Loop Syntax Usage Guide

Qi has multiple loop constructs. Choose appropriately based on the situation.

### Collection Processing

**When processing entire collections** → Use higher-order functions

```qi
;; ✅ Recommended
(map transform data)
(filter valid? data)
(each println data)

;; ❌ Not recommended (verbose)
(def items (atom data))
(while (some? (first @items))
  (process (first @items))
  (swap! items rest))
```

**Use case**: Data transformation, filtering, applying side effects

### Conditional Loops (Functional Style)

**nil-checking loop** → `while-some`

```qi
;; ✅ Recommended (functional, good pipeline compatibility)
(while-some [line (io/stdin-line)]
  (line
   |> str/trim
   |> process
   |> save))
```

**Error-checking loop** → `until-error`

```qi
;; ✅ Recommended (Result type integration, Railway Oriented Programming)
(until-error [result (fetch-next)]
  (result
   |>? validate
   |>? save))
```

**Use case**: Stream processing, pagination, API calls

### Simple Loops (Side Effect-Based)

**Counter-based loops** → `while` / `until`

```qi
;; ✅ Recommended (simple and intuitive)
(def count (atom 0))
(while (< @count 100)
  (do-something)
  (swap! count inc))

;; Retry logic
(def success (atom false))
(until @success
  (reset! success (try-operation)))
```

**Use case**: Counter processing, retry logic, external resource polling

**Caution**: Always modify condition in body to avoid infinite loops.

### Tail Recursion Optimization

**Recursion without stack consumption** → `loop` / `recur`

```qi
;; ✅ Recommended (large iterations, recursive algorithms)
(defn factorial [n]
  (loop [i n acc 1]
    (if (<= i 1)
      acc
      (recur (dec i) (* acc i)))))
```

**Use case**: Recursive algorithms, heavy iteration processing

### Quick Reference

| Use Case | Syntax | Features |
|------|------|------|
| Collection processing | `map`, `filter`, `each` | Concise, pipeline-friendly |
| Loop until nil | `while-some` | Functional, with binding |
| Loop until error | `until-error` | Result type integration |
| Counter loops | `while`, `until` | Simple, side effect-based |
| Tail recursion | `loop/recur` | No stack consumption |

**Principle**: Choose the most concise syntax with clear intent. When in doubt, start with higher-order functions (`map`, `filter`, `each`).

---

## Namespace

**Lisp-1 (Scheme-style)** - Variables and functions share the same namespace

```qi
(def add (fn [x y] (+ x y)))
(def op add)           ;; Assign function to variable
(op 1 2)               ;; 3
```

---

## Operators

### Arithmetic Operators

Supports both integers and floating-point numbers. Mixing different types results in floating-point.

```qi
(+ 1 2)         ;; 3
(+ 1.5 2.5)     ;; 4.0
(+ 1 2.5)       ;; 3.5 (type promotion)

(- 5 3)         ;; 2
(- 5.0 3.0)     ;; 2.0

(* 4 5)         ;; 20
(* 2.5 3)       ;; 7.5

(/ 10 2)        ;; 5
(/ 10.0 2.0)    ;; 5.0
(/ 10 3.0)      ;; 3.3333...

(% 10 3)        ;; 1 (modulo)
(% 10.5 3)      ;; 1.5 (supports floating-point)
```

### Comparison Operators

Supports both integers and floating-point numbers. Can compare different types.

```qi
(= 1 1)         ;; true
(= 1.0 1.0)     ;; true
(= 1 1.0)       ;; false (different types)

(!= 1 2)        ;; true
(!= 1.0 2.0)    ;; true

(< 1 2)         ;; true
(< 1.5 2.0)     ;; true
(< 1 2.5)       ;; true (integer and floating-point comparison)

(<= 1 1)        ;; true
(<= 1.0 2.0)    ;; true

(> 2 1)         ;; true
(> 2.5 1.5)     ;; true

(>= 2 2)        ;; true
(>= 2.0 2.0)    ;; true
```

### Logical Operators

```qi
(and true false)     ;; false
(or true false)      ;; true
(not true)           ;; false
```

---

## Basic Function Calls

```qi
;; Function application
(f x y z)         ;; Call f with arguments x, y, z

;; Built-in functions
(+ 1 2 3)         ;; 6
(str "hello" " " "world")  ;; "hello world"

;; User-defined functions
(defn square [x] (* x x))
(square 5)        ;; 25

;; Higher-order functions
(map inc [1 2 3])  ;; (2 3 4)
(filter even? [1 2 3 4])  ;; (2 4)
```

---

## Core Predicate Functions

Qi provides many predicate functions (functions ending with `?`) for type checking and conditional testing.

### Type Check Predicates (12)

```qi
;; nil check
(nil? nil)          ;; => true
(nil? 0)            ;; => false
(nil? "")           ;; => false

;; Collection types
(list? '(1 2 3))    ;; => true
(vector? [1 2 3])   ;; => true
(map? {:a 1})       ;; => true

;; Primitive types
(string? "hello")   ;; => true
(integer? 42)       ;; => true
(float? 3.14)       ;; => true
(number? 42)        ;; => true  (integer or float)

;; Special types
(keyword? :test)    ;; => true
(function? inc)     ;; => true
(atom? (atom 0))    ;; => true
(stream? (stream/range 0 10))  ;; => true
```

### Collection Predicates (3)

```qi
(coll? [1 2 3])           ;; => true  (list/vector/map)
(sequential? [1 2 3])     ;; => true  (list or vector)
(empty? [])               ;; => true
(empty? nil)              ;; => true
```

### State Predicates (4)

```qi
;; Not nil check
(some? 0)           ;; => true
(some? "")          ;; => true
(some? nil)         ;; => false

;; Strict boolean check
(true? true)        ;; => true
(true? 1)           ;; => false  (truthy but not true)

(false? false)      ;; => true
(false? nil)        ;; => false  (falsy but not false)

;; Error check
(error? {:error "failed"})  ;; => true
(error? 42)                 ;; => false
(error? nil)                ;; => false
```

### Numeric Predicates (5)

```qi
;; Even/odd
(even? 2)           ;; => true
(odd? 3)            ;; => true

;; Sign check
(positive? 1)       ;; => true
(negative? -1)      ;; => true
(zero? 0)           ;; => true
(zero? 0.0)         ;; => true
```

### Predicate Usage

Predicates are used in scenarios like:

```qi
;; Combined with filter
(filter even? [1 2 3 4 5])        ;; => (2 4)
(filter some? [1 nil 2 nil 3])    ;; => (1 2 3)

;; Conditional branching
(if (nil? x)
  "x is nil"
  "x has some value")

;; match guards
(match data
  {:value v} when (positive? v) -> "positive"
  {:value v} when (zero? v) -> "zero"
  _ -> "other")
```

**Note:** Collection operations `list/some?` and `list/every?` (predicate + collection testing) are separate functions (→ see Data Structures).
