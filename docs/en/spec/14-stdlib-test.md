# Standard Library - Test Framework

**Simple and Easy-to-Use Test Framework**

---

## Overview

Qi's test framework is designed with a focus on **simplicity** and **ease of learning**.

**Features:**
- Zero-config (no configuration files required)
- Auto-detection (explores `tests/` directory)
- Simple output in Rust/Go style
- No special syntax (plain function calls)

---

## Test Execution Functions

### test/run

Executes a test and records the result.

**Syntax:**
```qi
(test/run test-name test-fn)
```

**Arguments:**
- `test-name` (string): Name of the test
- `test-fn` (function): Function to execute the test (no arguments)

**Return Value:**
- `true`: Test passed
- `false`: Test failed (error occurred)

**Example:**
```qi
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))))

(test/run "list operations" (fn []
  (test/assert-eq 3 (len [1 2 3]))
  (test/assert-eq 1 (first [1 2 3]))))
```

### test/run-all

Displays all recorded test results.

**Syntax:**
```qi
(test/run-all)
```

**Return Value:**
- Number of successful tests (integer)
- Error if any test failed

**Output Example:**
```
Test results:
===========
  ✓ addition
  ✓ list operations
  ✗ division by zero
    Expected: {:error ...}
    Actual: 42

3 tests, 2 passed, 1 failed
```

### test/clear

Clears test results.

**Syntax:**
```qi
(test/clear)
```

**Return Value:**
- `nil`

**Usage Example:**
```qi
(test/clear)  ;; Reset test results
;; Run new tests
```

---

## Assertion Functions

### test/assert-eq

Asserts that two values are equal.

**Syntax:**
```qi
(test/assert-eq expected actual)
```

**Arguments:**
- `expected`: Expected value
- `actual`: Actual value

**Success Condition:**
- `expected` and `actual` are equal (compared with `=`)

**Example:**
```qi
(test/run "equality tests" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq "HELLO" (str/upper "hello"))
  (test/assert-eq [1 2 3] (map inc [0 1 2]))))
```

### test/assert

Asserts that a value is truthy.

**Syntax:**
```qi
(test/assert value)
```

**Arguments:**
- `value`: Value to check

**Success Condition:**
- `value` is truthy (anything other than `nil` and `false`)

**Example:**
```qi
(test/run "truthy tests" (fn []
  (test/assert (> 5 3))
  (test/assert (even? 4))
  (test/assert (some? [1 2 3]))          ;; Vector is not nil
  (test/assert (list/some? even? [2 4 6]))))
```

### test/assert-not

Asserts that a value is falsy.

**Syntax:**
```qi
(test/assert-not value)
```

**Arguments:**
- `value`: Value to check

**Success Condition:**
- `value` is falsy (`nil` or `false`)

**Example:**
```qi
(test/run "falsy tests" (fn []
  (test/assert-not (< 5 3))
  (test/assert-not (odd? 4))
  (test/assert-not nil)))
```

### test/assert-throws

Asserts that a function throws an exception.

**Syntax:**
```qi
(test/assert-throws test-fn)
```

**Arguments:**
- `test-fn` (function): Function to execute (no arguments)

**Success Condition:**
- An error occurs when executing `test-fn`

**Example:**
```qi
(test/run "exception tests" (fn []
  (test/assert-throws (fn [] (/ 10 0)))
  (test/assert-throws (fn [] (first [])))
  (test/assert-throws (fn [] (get {} :missing)))))
```

---

## CLI Command

### qi test

Executes test files.

**Usage:**
```bash
# Run all *.qi files in tests/ directory
qi test

# Run specific test file
qi test tests/core_test.qi

# Run multiple files
qi test tests/core_test.qi tests/pipeline_test.qi
```

**Test File Placement:**
```
project/
  tests/
    core_test.qi
    pipeline_test.qi
    http_test.qi
```

**Output:**
```
running 3 test files

Test results:
===========
  ✓ addition
  ✓ subtraction
  ✓ multiplication
  ✓ list operations
  ✓ pipeline test

5 tests, 5 passed, 0 failed

finished in 0.08s
```

---

## Practical Examples

### Basic Tests

```qi
;; tests/core_test.qi
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))
  (test/assert-eq -1 (+ 1 -2))))

(test/run "subtraction" (fn []
  (test/assert-eq 1 (- 3 2))
  (test/assert-eq -5 (- 0 5))))

(test/run "multiplication" (fn []
  (test/assert-eq 6 (* 2 3))
  (test/assert-eq 0 (* 5 0))))
```

### Pipeline Tests

```qi
;; tests/pipeline_test.qi
(test/run "basic pipeline" (fn []
  (test/assert-eq (list 2 3 4) ([1 2 3] |> (map inc)))))

(test/run "pipeline with multiple steps" (fn []
  (test/assert-eq 9 ([1 2 3] |> (map inc) |> sum))))

(test/run "filter and map" (fn []
  (test/assert-eq (list 3 5)
    ([1 2 3 4 5 6] |> (filter even?) |> (map inc) |> (take 2)))))
```

### String Operation Tests

```qi
;; tests/string_test.qi
(test/run "string upper/lower" (fn []
  (test/assert-eq "HELLO" (str/upper "hello"))
  (test/assert-eq "world" (str/lower "WORLD"))))

(test/run "string trimming" (fn []
  (test/assert-eq "hello" (str/trim "  hello  "))
  (test/assert-eq "test" (str/trim-start "  test"))
  (test/assert-eq "test" (str/trim-end "test  "))))

(test/run "string splitting" (fn []
  (test/assert-eq (list "a" "b" "c") (str/split "a,b,c" ","))
  (test/assert-eq (list "hello" "world") (str/split "hello world" " "))))
```

### Error Handling Tests

```qi
;; tests/error_test.qi
(test/run "division by zero" (fn []
  (test/assert-throws (fn [] (/ 10 0)))))

(test/run "empty list operations" (fn []
  (test/assert-throws (fn [] (first [])))
  (test/assert-throws (fn [] (last [])))))

(test/run "map key not found" (fn []
  (test/assert-throws (fn [] (get {} :missing)))))
```

### HTTP Tests

```qi
;; tests/http_test.qi
(test/run "http/get success" (fn []
  (def result (http/get! "https://httpbin.org/get"))  ;; Detailed version to get status
  (test/assert (map? result))
  (test/assert-eq 200 (get result :status))))

(test/run "railway pipeline with http" (fn []
  (def result
    (match (try
             ("https://httpbin.org/get"
              |> http/get  ;; Simple version (body only)
              |>? json/parse))
      {:error e} -> {:error e}
      data -> data))

  (match result
    {:error e} -> (test/assert false)  ;; Should not reach here
    data -> (test/assert (map? data)))))
```

### Data Transformation Tests

```qi
;; tests/transform_test.qi
(test/run "json round-trip" (fn []
  (def original {"name" "Alice" "age" 30})
  (def result
    (original
     |>? json/stringify
     |>? json/parse))

  (match result
    {:error e} -> (test/assert false)
    data -> (test/assert-eq original data))))

(test/run "map transformations" (fn []
  (def users [{:name "Alice" :age 30} {:name "Bob" :age 25}])
  (def result (map (fn [u] (update u :age inc)) users))

  (test/assert-eq 31 (get (first result) :age))
  (test/assert-eq 26 (get (last result) :age))))
```

---

## Best Practices

### File Organization

```
project/
  src/
    core.qi
    utils.qi
  tests/
    core_test.qi      # Tests for src/core.qi
    utils_test.qi     # Tests for src/utils.qi
    integration_test.qi  # Integration tests
```

### Test Naming

```qi
;; ✅ Good: Clear what is being tested
(test/run "addition with positive numbers" (fn [] ...))
(test/run "string upper converts to uppercase" (fn [] ...))
(test/run "http/get returns 200 for valid URL" (fn [] ...))

;; ❌ Bad: Unclear what is being tested
(test/run "test1" (fn [] ...))
(test/run "check" (fn [] ...))
```

### Test Granularity

```qi
;; ✅ Good: One test for one feature
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))))

(test/run "subtraction" (fn []
  (test/assert-eq 1 (- 3 2))))

;; ❌ Bad: One test for multiple unrelated features
(test/run "math" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq "HELLO" (str/upper "hello"))
  (test/assert-eq [1 2] (take 2 [1 2 3]))))
```

### Error Messages

```qi
;; When assertion fails, a clear message is displayed
(test/run "example" (fn []
  (test/assert-eq 5 (+ 2 2))))

;; Output:
;;   ✗ example
;;     Assertion failed:
;;   Expected: 5
;;   Actual: 4
```

---

## Execution Flow

1. **Test File Detection**: `qi test` searches for `*.qi` files in the `tests/` directory
2. **Test Execution**: Each file is loaded and executed sequentially
3. **Result Collection**: Each call to `test/run` records the result
4. **Report Display**: `test/run-all` displays all results
5. **Exit Code**:
   - All passed: exit code 0
   - Any failure: exit code 1

---

## Limitations

**Currently Unimplemented:**
- Coverage measurement
- Test tagging/filtering
- `--watch` mode (auto re-run on file changes)
- Setup/teardown (`before`/`after`)
- Parallel test execution

These features **may be considered in future versions**, but current functionality is sufficient for basic testing.

---

## Design Philosophy

Qi's test framework prioritizes **simplicity**:

1. **Zero-config**: No configuration files required. Just place files in `tests/`
2. **No special syntax**: Write tests with normal function calls
3. **Minimal API**: Only 5 functions to remember
4. **Clear output**: Simple and readable output in Rust/Go style

**Complex features come later**. Being able to write basic tests is what matters most.
