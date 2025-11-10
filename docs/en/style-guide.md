# Qi Style Guide

A style guide that summarizes formatting rules and best practices for Qi code.

The `qi fmt` command automatically formats code based on this guide.

---

## Table of Contents

1. [Formatting Rules](#1-formatting-rules)
2. [Naming Conventions](#2-naming-conventions)
3. [Best Practices](#3-best-practices)
4. [Anti-patterns](#4-anti-patterns)
5. [Formatter Configuration](#5-formatter-configuration)

---

## 1. Formatting Rules

### 1.1 Indentation

Use **2 spaces**. Do not use tabs.

```qi
;; ✅ Good example
(defn factorial [n]
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

;; ❌ Bad example (4 spaces)
(defn factorial [n]
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
```

### 1.2 Line Length

**Maximum 100 characters** is recommended.

```qi
;; ✅ Within 100 characters
(some-function arg1 arg2 arg3 arg4 arg5)

;; ✅ Break lines when exceeding
(some-very-long-function-name-that-would-exceed-the-limit
  arg1
  arg2
  arg3)
```

### 1.3 Pipelines

Pipeline operators should be **placed at the beginning** of lines, with each step on a new line.

```qi
;; ✅ Good example
(data
  |> validate
  |> transform
  |> (filter active?)
  |> (map format)
  |> save)

;; ✅ Single line is OK for short pipelines
("hello" |> str/upper |> str/reverse)

;; ❌ Bad example (operator at the end)
(data |>
  validate |>
  transform |>
  save)
```

#### Parallel Pipeline (`||>`)

```qi
(data
  ||> process1
  ||> process2
  ||> process3
  |> collect-results)
```

#### Railway Pipeline (`|>?`)

```qi
(request
  |>? validate-input
  |>? check-permissions
  |>? execute-action
  |>? format-response)
```

### 1.4 Special Forms

#### `def` / `defn`

```qi
;; ✅ def
(def pi 3.14159)

(def config
  {:host "localhost"
   :port 8080})

;; ✅ defn
(defn greet [name]
  (println f"Hello, {name}!"))

(defn complex-function [arg1 arg2 arg3]
  (let [result (process arg1 arg2)]
    (if (valid? result)
      (transform result arg3)
      nil)))
```

#### `let`

Clearly separate bindings and body.

```qi
;; ✅ Good example
(let [x 10
      y 20
      sum (+ x y)]
  (* sum 2))

;; ✅ Complex bindings
(let [user (get-user id)
      profile (get-profile user)
      settings (get-settings user)]
  (render-page user profile settings))
```

#### `if`

Align condition, then-clause, and else-clause.

```qi
;; ✅ Simple case
(if (> x 10)
  "big"
  "small")

;; ✅ Complex case
(if (and (valid? user)
         (active? user)
         (authorized? user))
  (process-request user)
  (reject-request user))
```

#### `do`

Align each expression at the same indentation level.

```qi
;; ✅ Good example
(do
  (log/info "Starting process")
  (initialize-resources)
  (execute-main-task)
  (cleanup-resources)
  (log/info "Process completed"))
```

#### `match`

Place `->` with **spaces on both sides** between patterns and actions.

```qi
;; ✅ Good example
(match value
  nil -> "empty"
  0 -> "zero"
  n when (> n 0) -> "positive"
  _ -> "negative")

;; ✅ Complex patterns
(match [x y]
  [0 0] -> "origin"
  [0 _] -> "y-axis"
  [_ 0] -> "x-axis"
  [x y] -> f"point ({x}, {y})")

;; ✅ Or patterns
(match status
  :pending | :processing -> "in-progress"
  :completed | :success -> "done"
  :failed | :error -> "error")

;; ✅ Long actions
(match status
  :pending -> (do
                (log/info "Processing request")
                (process-request req))
  :completed -> (notify-user user)
  :failed -> (retry-with-backoff req))
```

#### `try` / `defer`

```qi
;; ✅ try
(try
  (risky-operation)
  (another-operation)
  (catch e
    (log/error f"Failed: {e}")
    nil))

;; ✅ defer
(do
  (defer (cleanup-resources))
  (do-main-work))
```

### 1.5 Function Calls

#### Short Arguments

```qi
(+ 1 2 3)
(str/upper "hello")
(map inc [1 2 3])
```

#### Long Arguments

```qi
;; ✅ Break each argument to new line
(create-user
  "alice"
  "alice@example.com"
  30
  "123 Main St")

;; ✅ Same for function definitions
(defn process-user [username
                    email
                    age
                    address]
  ...)
```

### 1.6 Data Structures

#### Vectors `[]`

```qi
;; ✅ Short case
[1 2 3 4 5]

;; ✅ Long case
[first-element
 second-element
 third-element
 fourth-element]

;; ✅ Nested
[[1 2 3]
 [4 5 6]
 [7 8 9]]
```

#### Maps `{}`

Align keys and values vertically.

```qi
;; ✅ Basic form
{:name "Alice"
 :age 30
 :email "alice@example.com"}

;; ✅ Single line is OK for short maps
{:x 10 :y 20}

;; ✅ Nested
{:user {:name "Alice"
        :age 30
        :email "alice@example.com"}
 :status :active
 :created-at "2025-01-13"}
```

#### Lists `()`

```qi
'(1 2 3 4 5)

'(first-item
  second-item
  third-item)
```

### 1.7 Strings

#### Regular Strings

```qi
"Hello, World!"

;; ✅ Use multiline strings for long text
"""
This is a very long string that spans multiple lines.
You can write it naturally without worrying about line breaks.
"""
```

#### F-strings

Keep interpolation parts as-is.

```qi
;; ✅ Single line
(println f"Hello, {name}! You are {age} years old.")

;; ✅ Multiline
(println f"""
  Dear {name},

  Your account balance is {balance}.
  Thank you for using our service.
  """)
```

> `qi fmt` respects the input form of string literals without reconstructing line breaks, quotes, or escape representations.

### 1.8 Comments

#### Line Comments

```qi
;; ✅ Top-level: two semicolons
;; This is a top-level comment
;; explaining the following code.

(def x 10)

;; ✅ Inline: one semicolon (with 2 spaces before)
(def x 10)  ; This is an inline comment
```

> Comments are not deleted or merged. Feel free to write necessary explanations.

#### Section Separators

```qi
;; ========================================
;; Data Processing Functions
;; ========================================

(defn process-data [data]
  ...)

(defn transform-data [data]
  ...)


;; ========================================
;; Helper Functions
;; ========================================

(defn helper-1 []
  ...)
```

### 1.9 Blank Lines

#### Between Top-level Definitions: **Recommended 1 line (0-2 lines allowed)**

Typically separate with 1 blank line, but can be adjusted within the range of 0-2 lines for readability.

```qi
(def x 10)

(def y 20)

(def config
  {:host "localhost"
   :port 8080})
```

#### Before `def` / `defn` / `defn-`: **Always 1 blank line (except comments)**

Always insert at least 1 blank line before top-level definition forms (`def`, `defn`, `defn-`) to clearly separate blocks. When placing an explanatory comment immediately before, you can write the definition right after the comment.

```qi
(def cache (atom {}))

(defn clear-cache []
  ...)

;; kick entry point
(defn main []
  ...)

;; internal helper
;; It's OK to write defn- right after comment
(defn- build-index [entries]
  ...)
```

#### Section Separators: **2 lines**

```qi
(defn helper-1 [] ...)
(defn helper-2 [] ...)


;; ========================================
;; Public API
;; ========================================

(defn public-api-1 [] ...)
(defn public-api-2 [] ...)
```

### 1.10 Module System

#### `use` Declarations

```qi
;; ✅ Short case
(use str :as s)
(use list :only [map filter reduce])

;; ✅ Long case
(use http
  :only [get post put delete
         request with-headers
         json xml])

;; ✅ Multiple use declarations (group at file beginning)
(use str :as s)
(use list :as l)
(use io :only [read-file write-file])
(use http :only [get post])
```

#### `export` Declarations

```qi
;; ✅ Short case
(export [func1 func2 func3])

;; ✅ Long case
(export
  [public-api-1
   public-api-2
   public-api-3
   helper-function
   utility-function])
```

### 1.11 Formatter Policy

The Qi formatter aims to "preserve the author's intended text representation without changing the meaning of the code."

- Comments and whitespace are not deleted or merged, with minimal adjustments within configured ranges
- String literals are not re-escaped or normalized
- Top-level definition separators are normalized based on `blank-lines-between-defs` in `.qi-format.edn`
- At least 1 blank line is ensured before `def`/`defn`/`defn-`, allowing definitions immediately after comments
- For cases not defined in the guide, preserve existing layout as much as possible, with rules to be added in the future

---

## 2. Naming Conventions

### 2.1 Function Names

Use **kebab-case**.

```qi
;; ✅ Good examples
(defn get-user [id] ...)
(defn process-payment [amount] ...)
(defn calculate-total-price [items] ...)

;; ❌ Bad examples
(defn getUser [id] ...)          ; camelCase
(defn process_payment [amount] ...) ; snake_case
```

### 2.2 Predicate Functions

Add `?` at the end.

```qi
(defn active? [user] ...)
(defn valid-email? [email] ...)
(defn empty? [coll] ...)
```

### 2.3 Destructive Operations

Add `!` at the end.

```qi
(defn send! [chan value] ...)
(defn reset! [atom value] ...)
(defn swap! [atom f] ...)
```

### 2.4 Variable Names

Use **kebab-case**.

```qi
(def max-connections 100)
(def api-base-url "https://api.example.com")
(let [user-id 123
      user-name "alice"]
  ...)
```

### 2.5 Constants

Use **kebab-case**, same as regular variables.

```qi
(def pi 3.14159)
(def max-retry-count 3)
(def default-timeout 30000)
```

### 2.6 Keywords

Use **kebab-case**.

```qi
{:user-id 123
 :user-name "alice"
 :created-at "2025-01-13"}
```

### 2.7 Private Functions

Use `defn-`.

```qi
(defn- internal-helper [x]
  "Used only within module"
  (* x 2))

(defn public-api [x]
  "Exposed to external"
  (internal-helper x))
```

---

## 3. Best Practices

### 3.1 Leverage Pipelines

```qi
;; ✅ Pipeline
(data
  |> validate
  |> transform
  |> save)

;; ❌ Nested function calls
(save (transform (validate data)))
```

### 3.2 Use Pattern Matching with match

```qi
;; ✅ match
(match value
  nil -> "empty"
  [x] -> f"single: {x}"
  [x ...rest] -> f"multiple")

;; ❌ Complex if-else
(if (nil? value)
  "empty"
  (if (= (count value) 1)
    f"single: {(first value)}"
    "multiple"))
```

### 3.3 Leverage Destructuring

```qi
;; ✅ let destructuring
(let [{:name n :age a} user]
  (println f"{n} is {a} years old"))

;; ✅ Function argument destructuring
(defn greet [{:name n}]
  (println f"Hello, {n}!"))

;; ✅ match destructuring
(match coords
  [x y] -> (+ x y))
```

### 3.4 Organize Namespaces with use

```qi
;; ✅ Use aliases
(use str :as s)
(s/upper "hello")

;; ✅ Import only necessary functions
(use list :only [map filter reduce])
```

### 3.5 Leverage Early Returns

```qi
;; ✅ Guard clauses
(defn process [data]
  (if (nil? data)
    nil
    (do
      (validate data)
      (transform data))))

;; ✅ Early return with match
(match status
  :invalid -> nil
  :pending -> (process-pending)
  :completed -> result)
```

### 3.6 Leverage F-strings

```qi
;; ✅ f-string
(println f"User {name} has {count} items")

;; ❌ str concatenation
(println (str "User " name " has " count " items"))
```

---

## 4. Anti-patterns

### 4.1 Deep Nesting

```qi
;; ❌ Deep nesting
(if condition1
  (if condition2
    (if condition3
      (do-something)
      default)
    default)
  default)

;; ✅ Pipeline or match
(value
  |>? validate1
  |>? validate2
  |>? validate3
  |>? do-something)
```

### 4.2 Overly Long Functions

```qi
;; ❌ Functions over 100 lines
(defn huge-function [...]
  ;; 100+ lines
  )

;; ✅ Split into smaller functions
(defn- step1 [x] ...)
(defn- step2 [x] ...)
(defn main-function [x]
  (x |> step1 |> step2))
```

### 4.3 Abusing Global State

```qi
;; ❌ Direct modification of global variables
(def counter 0)
(defn increment [] (set! counter (+ counter 1)))

;; ✅ Use atoms
(def counter (atom 0))
(defn increment [] (swap! counter inc))
```

### 4.4 Meaningless Variable Names

```qi
;; ❌ Bad examples
(defn f [x y z] ...)
(let [a 1 b 2] ...)

;; ✅ Good examples
(defn calculate-total [price quantity tax] ...)
(let [user-id 1 user-name "alice"] ...)
```

---

## 5. Formatter Configuration

### 5.1 Default Configuration

```edn
;; .qi-format.edn
{:indent-width 2
 :max-line-length 100
 :pipeline-newline true
 :pipeline-operator-position :leading
 :match-arrow-spacing :both
 :align-map-values true
 :sort-use-declarations false
 :blank-lines-between-defs 1
 :blank-lines-between-sections 2}
```

### 5.2 Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `indent-width` | `2` | Indentation width (number of spaces) |
| `max-line-length` | `100` | Maximum line length (characters) |
| `pipeline-newline` | `true` | Whether to break pipelines into multiple lines |
| `pipeline-operator-position` | `:leading` | Pipeline operator position (`:leading` or `:trailing`) |
| `match-arrow-spacing` | `:both` | Space around match arrows (`:both`, `:before`, `:after`, `:none`) |
| `align-map-values` | `true` | Whether to align map values |
| `sort-use-declarations` | `false` | Whether to sort use declarations |
| `blank-lines-between-defs` | `1` | Normalize blank lines between top-level definitions within 0-2 range (ensure at least 1 line before `def`/`defn`/`defn-`) |
| `blank-lines-between-sections` | `2` | Number of blank lines between sections |

Current style guide values are the defaults, but you can override them in `.qi-format.edn` according to your team's policy.

Place `.qi-format.edn` in the directory containing forms or in the current directory root. Write an EDN map like this:

```clojure
{:indent-width 2
 :blank-lines-between-defs 1
 :max-line-length 100}
```

Non-numeric values or unsupported keys are ignored, and if a parser error occurs, it will fall back to default values.

### 5.3 Usage

```bash
# Format file (overwrite)
qi fmt src/main.qi

# Format file (output to stdout)
qi fmt --check src/main.qi

# Recursively format directory
qi fmt src/

# Format from stdin
cat src/main.qi | qi fmt --stdin
```

### 5.4 Formatter Checklist

`qi fmt` is implemented to satisfy the following points (behavior changes based on configuration values, see `.qi-format.edn`):

- String literal line breaks, quotes, and escape representations respect the input and are not re-encoded
- Comments are not deleted or merged, and their position is maintained within bounds that don't break meaning
- Normalize the number of blank lines between top-level definitions to 0-2 according to `blank-lines-between-defs`
- Ensure at least 1 blank line before `def`/`defn`/`defn-`, allowing immediate preceding comments
- For layouts not defined in this guide, preserve the existing structure as much as possible

---

## References

This style guide references the following:

- [Clojure Style Guide](https://github.com/bbatsov/clojure-style-guide)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- [Elixir Style Guide](https://github.com/christopheradams/elixir_style_guide)

---

## License

MIT
