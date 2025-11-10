# Chapter 3: Mastering Pattern Matching

**Time Required**: 25 minutes

Pattern matching is a powerful feature for branching processing based on data structure. It produces **much more readable and maintainable** code than nesting multiple `if` statements.

---

## `match` Expression Basics

`match` matches a value against patterns and returns the result of the matched pattern.

```qi
(match value
  pattern1 -> result1
  pattern2 -> result2
  _ -> default)
```

### Example: Simple Matching

```qi
qi> (match 42
      42 -> "The answer!"
      _ -> "Something else")
; => "The answer!"

qi> (match "hello"
      "hello" -> "Hi!"
      "bye" -> "Goodbye!"
      _ -> "Unknown")
; => "Hi!"
```

**Key point**: `_` is a wildcard (matches anything).

---

## List Pattern Matching

You can destructure lists to extract the first element or remaining elements.

```qi
qi> (match [1 2 3]
      [] -> "Empty"
      [x] -> f"Single: {x}"
      [first second] -> f"Pair: {first}, {second}"
      [a b c] -> f"Three: {a}, {b}, {c}")
; => "Three: 1, 2, 3"
```

### first/rest Pattern

```qi
qi> (defn describe-list [lst]
      (match lst
        [] -> "Empty list"
        [x] -> f"Single element: {x}"
        [first & rest] -> f"First: {first}, Rest: {rest}"))

qi> (describe-list [])
; => "Empty list"

qi> (describe-list [10])
; => "Single element: 10"

qi> (describe-list [1 2 3 4 5])
; => "First: 1, Rest: [2 3 4 5]"
```

---

## Map Pattern Matching

You can destructure maps to extract values.

```qi
qi> (def person {:name "Alice" :age 25 :city "Tokyo"})

qi> (match person
      {:name n :age a} -> f"{n} is {a} years old")
; => "Alice is 25 years old"
```

### Practical Example: HTTP Response Processing

```qi
(defn handle-response [resp]
  (match resp
    {:status 200 :body body} -> f"Success: {body}"
    {:status 404} -> "Not Found"
    {:status 500 :message msg} -> f"Server Error: {msg}"
    _ -> "Unknown response"))

qi> (handle-response {:status 200 :body "OK"})
; => "Success: OK"

qi> (handle-response {:status 404})
; => "Not Found"

qi> (handle-response {:status 500 :message "Database error"})
; => "Server Error: Database error"
```

---

## Guard Conditions

You can add conditions to patterns using `when`.

```qi
(defn classify-number [n]
  (match n
    x when (> x 0) -> "Positive"
    x when (< x 0) -> "Negative"
    _ -> "Zero"))

qi> (classify-number 10)
; => "Positive"

qi> (classify-number -5)
; => "Negative"

qi> (classify-number 0)
; => "Zero"
```

### Complex Guards

```qi
(defn describe-age [age]
  (match age
    n when (< n 13) -> "Child"
    n when (and (>= n 13) (< n 20)) -> "Teenager"
    n when (and (>= n 20) (< n 65)) -> "Adult"
    _ -> "Senior"))

qi> (describe-age 10)
; => "Child"

qi> (describe-age 15)
; => "Teenager"

qi> (describe-age 30)
; => "Adult"

qi> (describe-age 70)
; => "Senior"
```

---

## or Patterns

Multiple patterns can be grouped together (separated by `|`).

```qi
(defn weekday-or-weekend [day]
  (match day
    "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" -> "Weekday"
    "Saturday" | "Sunday" -> "Weekend"
    _ -> "Unknown"))

qi> (weekday-or-weekend "Monday")
; => "Weekday"

qi> (weekday-or-weekend "Saturday")
; => "Weekend"
```

---

## Practical Example 1: Command Processing

```qi
(defn handle-command [cmd]
  (match cmd
    {:type "add" :a a :b b} -> (+ a b)
    {:type "sub" :a a :b b} -> (- a b)
    {:type "mul" :a a :b b} -> (* a b)
    {:type "div" :a a :b b} -> (/ a b)
    _ -> {:error "Unknown command"}))

qi> (handle-command {:type "add" :a 10 :b 20})
; => 30

qi> (handle-command {:type "mul" :a 5 :b 6})
; => 30

qi> (handle-command {:type "unknown"})
; => {:error "Unknown command"}
```

---

## Practical Example 2: State Machine

```qi
(defn process-state [state event]
  (match [state event]
    ["idle" "start"] -> "running"
    ["running" "pause"] -> "paused"
    ["running" "stop"] -> "stopped"
    ["paused" "resume"] -> "running"
    ["paused" "stop"] -> "stopped"
    _ -> state))

qi> (process-state "idle" "start")
; => "running"

qi> (process-state "running" "pause")
; => "paused"

qi> (process-state "paused" "resume")
; => "running"
```

---

## Practical Example 3: JSON Data Validation

```qi
(defn validate-user [data]
  (match data
    {:name n :email e} when (and (string? n) (string? e)) ->
      {:valid true :user {:name n :email e}}
    _ ->
      {:valid false :error "Invalid user data"}))

qi> (validate-user {:name "Alice" :email "alice@example.com"})
; => {:valid true :user {:name "Alice" :email "alice@example.com"}}

qi> (validate-user {:name "Bob"})
; => {:valid false :error "Invalid user data"}

qi> (validate-user {:name 123 :email "test@test.com"})
; => {:valid false :error "Invalid user data"}
```

---

## Practical Example 4: Tree Traversal

```qi
; Binary tree node
; {:value value :left left-child :right right-child}

(defn tree-sum [tree]
  (match tree
    nil -> 0
    {:value v :left nil :right nil} -> v
    {:value v :left l :right nil} -> (+ v (tree-sum l))
    {:value v :left nil :right r} -> (+ v (tree-sum r))
    {:value v :left l :right r} -> (+ v (tree-sum l) (tree-sum r))))

(def my-tree
  {:value 10
   :left {:value 5 :left nil :right nil}
   :right {:value 15 :left nil :right nil}})

qi> (tree-sum my-tree)
; => 30  (10 + 5 + 15)
```

---

## Comparing match and if

### Using if (Hard to Read)

```qi
(defn classify-response [resp]
  (if (and (map? resp) (= (get resp :status) 200))
    (get resp :body)
    (if (and (map? resp) (= (get resp :status) 404))
      "Not Found"
      (if (and (map? resp) (= (get resp :status) 500))
        "Server Error"
        "Unknown"))))
```

### Using match (Easy to Read)

```qi
(defn classify-response [resp]
  (match resp
    {:status 200 :body body} -> body
    {:status 404} -> "Not Found"
    {:status 500} -> "Server Error"
    _ -> "Unknown"))
```

---

## Benefits of Pattern Matching

### 1. Readability

Data structure is immediately clear:

```qi
; ✅ Easy to read
(match user
  {:name n :age a} -> f"{n} is {a}"
  _ -> "Unknown")
```

### 2. Exhaustiveness

Easy to verify that all cases are handled:

```qi
(match status
  "pending" -> handle-pending
  "approved" -> handle-approved
  "rejected" -> handle-rejected
  _ -> handle-unknown)  ; Don't forget!
```

### 3. Maintainability

Adding new cases is easy:

```qi
(match command
  "start" -> start-process
  "stop" -> stop-process
  "restart" -> restart-process  ; ← Easy to add
  _ -> unknown-command)
```

---

## Practice Problems

### Problem 1: Grade Classification

Write a function that takes a score and returns a grade (A/B/C/D/F).

```qi
(defn get-grade [score]
  ; Fill this in
  ; 90+: A, 80+: B, 70+: C, 60+: D, below: F
  )

; Tests
(get-grade 95)  ; => "A"
(get-grade 85)  ; => "B"
(get-grade 75)  ; => "C"
(get-grade 65)  ; => "D"
(get-grade 50)  ; => "F"
```

<details>
<summary>Solution</summary>

```qi
(defn get-grade [score]
  (match score
    s when (>= s 90) -> "A"
    s when (>= s 80) -> "B"
    s when (>= s 70) -> "C"
    s when (>= s 60) -> "D"
    _ -> "F"))
```

</details>

### Problem 2: Get First Two Elements of List

Write a function that takes a list and returns the first two elements.

```qi
(defn first-two [lst]
  ; Fill this in
  )

; Tests
(first-two [1 2 3 4 5])  ; => [1 2]
(first-two [10])         ; => [10]
(first-two [])           ; => []
```

<details>
<summary>Solution</summary>

```qi
(defn first-two [lst]
  (match lst
    [] -> []
    [x] -> [x]
    [a b & _] -> [a b]))
```

</details>

### Problem 3: Option Type Processing

Write a function that takes a map with `:some` or `:none` and processes it if a value exists.

```qi
(defn process-option [opt]
  ; Fill this in
  ; {:type :some :value v} => v * 2
  ; {:type :none} => 0
  )

; Tests
(process-option {:type :some :value 10})  ; => 20
(process-option {:type :none})            ; => 0
```

<details>
<summary>Solution</summary>

```qi
(defn process-option [opt]
  (match opt
    {:type :some :value v} -> (* v 2)
    {:type :none} -> 0))
```

</details>

---

## Summary

What you learned in this chapter:

- ✅ `match` expression basics
- ✅ List and map pattern matching
- ✅ Guard conditions (`when`)
- ✅ or patterns (`|`)
- ✅ Practical pattern matching examples

---

## Next Steps

Now that you can freely handle data structures with pattern matching, let's learn **error handling** next!

➡️ [Chapter 4: Handling Errors Gracefully](04-error-handling.md)

Error handling and Railway Pipeline are essential for building robust applications.
