# Chapter 4: Handling Errors Gracefully

**Time Required**: 30 minutes

Error handling is essential for building robust applications. Qi provides a powerful error handling pattern called **Railway Pipeline**.

---

## Error Handling Basics: `try`

`try` safely executes code that might produce errors.

### Basic Usage

```qi
qi> (try (+ 1 2))
; => 3  (On success, returns the raw value)

qi> (try (/ 1 0))
; => {:error "Division by zero"}  (On error, returns {:error} format)
```

**Important**: In Qi's new specification, `try` **returns the raw value on success** (not `{:ok value}`). Only errors are in `{:error ...}` format.

---

## Checking for Errors with `error?` Predicate

Use `error?` to determine if a value is an error.

```qi
qi> (error? {:error "Something went wrong"})
; => true

qi> (error? 42)
; => false

qi> (error? {:ok 10})
; => false
```

### Practical Example

```qi
(defn safe-divide [a b]
  (try (/ a b)))

(defn process [a b]
  (let [result (safe-divide a b)]
    (if (error? result)
      (println "Error occurred!")
      (println f"Result: {result}"))))

qi> (process 10 2)
; => "Result: 5"

qi> (process 10 0)
; => "Error occurred!"
```

---

## Railway Pipeline: `|>?`

Railway Pipeline is a powerful pattern that **automatically propagates errors**.

### Basic Concept

With normal pipelines (`|>`), each step always executes. However, **with Railway Pipeline (`|>?`), when an error occurs, all subsequent processing is skipped**.

```
      Success     Success     Success
  ─────────> ─────────> ─────────>
 |          |          |          |
  ─────────> X Error!  ─────────>
      Success     ↓        Skipped
                Error returned
```

### Example: Simple Railway Pipeline

```qi
(defn validate-positive [x]
  (if (> x 0)
    x                          ; Success: return value as-is
    {:error "Must be positive"}))  ; Error

(defn double [x]
  (* x 2))

(defn add-ten [x]
  (+ x 10))

; Success case
qi> (10
     |>? validate-positive
     |>? double
     |>? add-ten)
; => 30  (10 -> 10 -> 20 -> 30)

; Error case
qi> (-5
     |>? validate-positive  ; Error here
     |>? double             ; Skipped
     |>? add-ten)           ; Skipped
; => {:error "Must be positive"}
```

**Key point**: Everything except `{:error}` is treated as success!

---

## Strengths of Railway Pipeline

### 1. Errors Propagate Automatically

```qi
(defn validate-age [age]
  (if (and (>= age 0) (<= age 150))
    age
    {:error "Invalid age"}))

(defn validate-name [name]
  (if (> (len name) 0)
    name
    {:error "Name cannot be empty"}))

(defn create-user [name age]
  ({:name name :age age}
   |>? (fn [u] (if (> (len (get u :name)) 0)
                  u
                  {:error "Name cannot be empty"}))
   |>? (fn [u] (if (and (>= (get u :age) 0) (<= (get u :age) 150))
                  u
                  {:error "Invalid age"}))))

qi> (create-user "Alice" 25)
; => {:name "Alice" :age 25}

qi> (create-user "" 25)
; => {:error "Name cannot be empty"}

qi> (create-user "Bob" 200)
; => {:error "Invalid age"}
```

### 2. Readable

Error handling is naturally integrated into data flow:

```qi
; ❌ Traditional error handling (hard to read)
(defn process-data [data]
  (let [result1 (step1 data)]
    (if (error? result1)
      result1
      (let [result2 (step2 result1)]
        (if (error? result2)
          result2
          (step3 result2))))))

; ✅ Railway Pipeline (easy to read)
(defn process-data [data]
  (data
   |>? step1
   |>? step2
   |>? step3))
```

---

## Practical Example 1: File Processing

```qi
(defn read-file [path]
  (try (io/read-file path)))

(defn parse-json [text]
  (try (json/parse text)))

(defn validate-data [data]
  (if (map? data)
    data
    {:error "Data must be a map"}))

(defn extract-field [data field]
  (if (contains? data field)
    (get data field)
    {:error f"Missing field: {field}"}))

; Connect with Railway Pipeline
(defn load-config [path]
  (path
   |>? read-file
   |>? parse-json
   |>? validate-data
   |>? (fn [d] (extract-field d :database))))

; Usage example
qi> (load-config "config.json")
; => "postgresql://localhost/mydb"  (On success)
; => {:error "File not found"}  (No file)
; => {:error "JSON parse failed"}  (Invalid JSON)
; => {:error "Missing field: database"}  (Missing field)
```

---

## Practical Example 2: API Call

```qi
(defn fetch-user [id]
  ; Mock HTTP request
  (if (> id 0)
    {:status 200 :body f"{{"id":{id},"name":"User{id}"}}"}
    {:status 404}))

(defn check-response [resp]
  (match resp
    {:status 200 :body body} -> body
    {:status 404} -> {:error "User not found"}
    _ -> {:error "Unknown error"}))

(defn parse-user [json-str]
  (try (json/parse json-str)))

(defn extract-name [user]
  (if (contains? user :name)
    (get user :name)
    {:error "Missing name"}))

; Connect with Railway Pipeline
(defn get-user-name [id]
  (id
   |> fetch-user
   |>? check-response
   |>? parse-user
   |>? extract-name))

qi> (get-user-name 1)
; => "User1"

qi> (get-user-name -1)
; => {:error "User not found"}
```

---

## Combining `try` and Railway Pipeline

Combining `try` and Railway Pipeline enables powerful error handling.

```qi
(defn safe-parse-int [s]
  (try (string/to-int s)))

(defn validate-range [n]
  (if (and (>= n 0) (<= n 100))
    n
    {:error "Number must be between 0 and 100"}))

(defn double [n]
  (* n 2))

; Pipeline
(defn process-input [input]
  (input
   |>? safe-parse-int
   |>? validate-range
   |>? double))

qi> (process-input "25")
; => 50

qi> (process-input "abc")
; => {:error "Parse error"}

qi> (process-input "150")
; => {:error "Number must be between 0 and 100"}
```

---

## Error Handling Best Practices

### 1. Standardize `{:error}` Format

```qi
; ✅ Good example
{:error "User not found"}
{:error "Invalid input"}
{:error "Database connection failed"}

; ❌ Bad example (not standardized)
"error"
{:err "..."}
{:failed true}
```

### 2. Make Error Messages Specific

```qi
; ✅ Good example
{:error f"User with ID {id} not found"}
{:error f"Age must be between 0 and 150, got {age}"}

; ❌ Bad example
{:error "Error"}
{:error "Invalid"}
```

### 3. Return Errors Early

```qi
; ✅ Good example: Early return
(defn process [x]
  (if (< x 0)
    {:error "Negative not allowed"}
    (do-something x)))

; ❌ Bad example: Deep nesting
(defn process [x]
  (if (>= x 0)
    (do-something x)
    {:error "Negative not allowed"}))
```

---

## Practice Problems

### Problem 1: Safe Division Function

Write a function that prevents division by zero.

```qi
(defn safe-divide [a b]
  ; Fill this in
  )

; Tests
(safe-divide 10 2)  ; => 5
(safe-divide 10 0)  ; => {:error "Division by zero"}
```

<details>
<summary>Solution</summary>

```qi
(defn safe-divide [a b]
  (if (= b 0)
    {:error "Division by zero"}
    (/ a b)))
```

</details>

### Problem 2: Numeric Validation Pipeline

Write a function that takes a string, converts it to a number, and checks if it's in the 0-100 range.

```qi
(defn validate-score [input]
  ; Fill this in
  ; Hint: Use safe-parse-int and validate-range
  )

; Tests
(validate-score "75")   ; => 75
(validate-score "abc")  ; => {:error "Parse error"}
(validate-score "150")  ; => {:error "Range error"}
```

<details>
<summary>Solution</summary>

```qi
(defn safe-parse-int [s]
  (try (string/to-int s)))

(defn validate-range [n]
  (if (and (>= n 0) (<= n 100))
    n
    {:error "Range error"}))

(defn validate-score [input]
  (input
   |>? safe-parse-int
   |>? validate-range))
```

</details>

### Problem 3: Multiple Validations

Write a function that takes user data and validates name and age.

```qi
(defn validate-user [user]
  ; Fill this in
  ; Name: At least 1 character
  ; Age: 0-150 range
  )

; Tests
(validate-user {:name "Alice" :age 25})
; => {:name "Alice" :age 25}

(validate-user {:name "" :age 25})
; => {:error "Name cannot be empty"}

(validate-user {:name "Bob" :age 200})
; => {:error "Invalid age"}
```

<details>
<summary>Solution</summary>

```qi
(defn validate-user [user]
  (user
   |>? (fn [u] (if (> (len (get u :name)) 0)
                  u
                  {:error "Name cannot be empty"}))
   |>? (fn [u] (if (and (>= (get u :age) 0) (<= (get u :age) 150))
                  u
                  {:error "Invalid age"}))))
```

</details>

---

## Summary

What you learned in this chapter:

- ✅ Safely catching errors with `try`
- ✅ Checking for errors with the `error?` predicate
- ✅ Automatic error propagation with Railway Pipeline (`|>?`)
- ✅ Error handling best practices
- ✅ Practical error handling patterns

---

## Next Steps

Now that you've mastered error handling, let's learn **concurrency and parallelism** next!

➡️ [Chapter 5: Easy Concurrency and Parallelism](05-concurrency.md)

Concurrency and parallelism are among Qi's most powerful features.
