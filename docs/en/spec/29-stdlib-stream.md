# Standard Library - Streams (stream/)

**Efficient Data Processing with Lazy Evaluation - Save Memory and Handle Infinite Data Structures**

Qi's streams are lazy-evaluated data structures that compute values only when needed.
They enable memory-efficient processing of large datasets and support infinite data structures.

> **Implementation**: `src/builtins/stream.rs`
>
> **Features**:
> - Lazy evaluation: compute only what's needed
> - Infinite data structures support (infinite lists, sequences, etc.)
> - Perfect integration with pipeline operators (`|>`)
> - Streaming I/O support for files and HTTP

---

## What are Streams?

### Difference from Regular Collections

```qi
;; Regular list: holds all elements in memory
(def nums [1 2 3 4 5 6 7 8 9 10])
(def squares (map (fn [x] (* x x)) nums))
;; => [1 4 9 16 25 36 49 64 81 100] (all computed)

;; Stream: computes only what's needed
(def s (stream/range 1 11))
(def squares-stream (stream/map (fn [x] (* x x)) s))
;; => <Stream> (not yet computed)

;; Get only the first 3 elements
(stream/realize (stream/take 3 squares-stream))
;; => (1 4 9) (only 3 elements computed)
```

### Benefits

1. **Memory Efficiency**: Uses memory only for what's needed
2. **Infinite Data Structures**: Can represent unbounded data
3. **Computation Optimization**: Skips unnecessary calculations
4. **I/O Streaming**: Doesn't load large files entirely into memory

---

## Stream Creation

### stream/stream - Create Stream from Collection

```qi
;; From list
(def s (stream/stream [1 2 3 4 5]))
(stream/realize s)  ;; => (1 2 3 4 5)

;; From vector
(def s (stream/stream [10 20 30]))
(stream/realize s)  ;; => (10 20 30)
```

**Arguments**:
- `coll`: List or vector

**Returns**: Stream

---

### stream/range - Range Stream

```qi
;; 0 to 9
(def s (stream/range 0 10))
(stream/realize s)  ;; => (0 1 2 3 4 5 6 7 8 9)

;; 1 to 5
(def s (stream/range 1 6))
(stream/realize s)  ;; => (1 2 3 4 5)

;; Empty range
(def s (stream/range 5 5))
(stream/realize s)  ;; => ()
```

**Arguments**:
- `start`: Start value (inclusive)
- `end`: End value (exclusive)

**Returns**: Stream

---

### stream/iterate - Infinite Stream by Repeated Function Application

```qi
;; 1, 2, 4, 8, 16, 32, ... (double each time)
(def s (stream/iterate (fn [x] (* x 2)) 1))
(stream/realize (stream/take 6 s))  ;; => (1 2 4 8 16 32)

;; 1, 2, 3, 4, 5, ... (increment by 1)
(def s (stream/iterate inc 1))
(stream/realize (stream/take 5 s))  ;; => (1 2 3 4 5)

;; Fibonacci sequence
(def fib-stream (stream/iterate (fn [[a b]] [b (+ a b)]) [0 1]))
(stream/realize
  (stream/take 10 fib-stream)
  |> (map first))
;; => (0 1 1 2 3 5 8 13 21 34)
```

**Arguments**:
- `f`: Function to apply to each element
- `initial`: Initial value

**Returns**: Stream (infinite)

---

### stream/repeat - Infinite Stream of Same Value

```qi
;; Repeat 42 infinitely
(def s (stream/repeat 42))
(stream/realize (stream/take 5 s))  ;; => (42 42 42 42 42)

;; Repeat string
(def s (stream/repeat "hello"))
(stream/realize (stream/take 3 s))  ;; => ("hello" "hello" "hello")
```

**Arguments**:
- `value`: Value to repeat

**Returns**: Stream (infinite)

---

### stream/cycle - Infinite Stream by Cycling List

```qi
;; Cycle [1 2 3]
(def s (stream/cycle [1 2 3]))
(stream/realize (stream/take 8 s))  ;; => (1 2 3 1 2 3 1 2)

;; Days of week cycle
(def days (stream/cycle ["Mon" "Tue" "Wed" "Thu" "Fri" "Sat" "Sun"]))
(stream/realize (stream/take 10 days))
;; => ("Mon" "Tue" "Wed" "Thu" "Fri" "Sat" "Sun" "Mon" "Tue" "Wed")
```

**Arguments**:
- `coll`: List or vector (must not be empty)

**Returns**: Stream (infinite)

**Error**:
- Raises error if collection is empty

---

## Stream Operations

### stream/map - Apply Function to Each Element

```qi
;; Double each element
(def s (stream/range 1 6))
(def s2 (stream/map (fn [x] (* x 2)) s))
(stream/realize s2)  ;; => (2 4 6 8 10)

;; Get string lengths
(def s (stream/stream ["hello" "world" "qi"]))
(def s2 (stream/map len s))
(stream/realize s2)  ;; => (5 5 2)

;; Works with infinite streams
(def s (stream/iterate inc 1))
(def squares (stream/map (fn [x] (* x x)) s))
(stream/realize (stream/take 5 squares))  ;; => (1 4 9 16 25)
```

**Arguments**:
- `f`: Function to apply to each element
- `stream`: Stream

**Returns**: Stream

---

### stream/filter - Keep Elements Matching Predicate

```qi
;; Even numbers only
(def s (stream/range 1 11))
(def evens (stream/filter (fn [x] (= (% x 2) 0)) s))
(stream/realize evens)  ;; => (2 4 6 8 10)

;; Numbers greater than 10
(def s (stream/range 1 21))
(def large (stream/filter (fn [x] (> x 10)) s))
(stream/realize large)  ;; => (11 12 13 14 15 16 17 18 19 20)

;; Multiples of 3 from infinite stream
(def s (stream/iterate inc 1))
(def multiples (stream/filter (fn [x] (= (% x 3) 0)) s))
(stream/realize (stream/take 5 multiples))  ;; => (3 6 9 12 15)
```

**Arguments**:
- `pred`: Predicate function (keeps elements returning true)
- `stream`: Stream

**Returns**: Stream

---

### stream/take - Take First n Elements

```qi
;; First 5 elements
(def s (stream/range 0 100))
(def s2 (stream/take 5 s))
(stream/realize s2)  ;; => (0 1 2 3 4)

;; Make infinite stream finite
(def s (stream/repeat 42))
(def s2 (stream/take 3 s))
(stream/realize s2)  ;; => (42 42 42)

;; Take 0 elements
(def s (stream/range 1 10))
(def s2 (stream/take 0 s))
(stream/realize s2)  ;; => ()
```

**Arguments**:
- `n`: Number of elements to take (non-negative integer)
- `stream`: Stream

**Returns**: Stream

---

### stream/drop - Skip First n Elements

```qi
;; Skip first 5 elements
(def s (stream/range 0 10))
(def s2 (stream/drop 5 s))
(stream/realize s2)  ;; => (5 6 7 8 9)

;; Skip 0 elements (no-op)
(def s (stream/range 1 4))
(def s2 (stream/drop 0 s))
(stream/realize s2)  ;; => (1 2 3)

;; Skip all elements
(def s (stream/range 1 4))
(def s2 (stream/drop 10 s))
(stream/realize s2)  ;; => ()
```

**Arguments**:
- `n`: Number of elements to skip (non-negative integer)
- `stream`: Stream

**Returns**: Stream

---

### stream/realize - Convert Stream to List

```qi
;; Execute stream and convert to list
(def s (stream/range 1 6))
(stream/realize s)  ;; => (1 2 3 4 5)

;; Execute complex transformation chain
(def s (stream/range 1 11))
(stream/realize
  (s
   |> (stream/map (fn [x] (* x x)))
   |> (stream/filter (fn [x] (> x 20)))))
;; => (25 36 49 64 81 100)
```

**Arguments**:
- `stream`: Stream

**Returns**: List

**Warning**:
- Realizing an infinite stream causes infinite loop
- Always use `take` to make infinite streams finite before `realize`

```qi
;; ❌ Bad: infinite loop
(stream/realize (stream/repeat 42))  ;; Never terminates

;; ✅ Good: make finite with take
(stream/realize (stream/take 5 (stream/repeat 42)))  ;; OK
```

---

## Pipeline Integration

Stream operations can be elegantly composed using Qi's pipeline operator (`|>`).

### Basic Pipelines

```qi
;; From creation to realization
[1 2 3 4 5]
  |> stream/stream
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (> x 10)))
  |> stream/realize
;; => (16 25)

;; Range stream
(stream/range 1 11)
  |> (stream/map (fn [x] (* x 2)))
  |> (stream/filter (fn [x] (> x 10)))
  |> (stream/take 3)
  |> stream/realize
;; => (12 14 16)
```

### Infinite Stream Pipelines

```qi
;; First 10 powers of 2
1
  |> (stream/iterate (fn [x] (* x 2)))
  |> (stream/take 10)
  |> stream/realize
;; => (1 2 4 8 16 32 64 128 256 512)

;; Multiples of 3 less than 100
1
  |> (stream/iterate inc)
  |> (stream/filter (fn [x] (= (% x 3) 0)))
  |> (stream/take-while (fn [x] (< x 100)))
  |> stream/realize
;; => (3 6 9 12 ... 96 99)
```

### Complex Transformation Chains

```qi
;; Square numbers from 1-99 that are multiples of 3
(stream/range 1 100)
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (= (% x 3) 0)))
  |> (stream/take 5)
  |> stream/realize
;; => (9 36 81 144 225)

;; Even Fibonacci numbers
(stream/iterate (fn [[a b]] [b (+ a b)]) [0 1])
  |> (stream/map first)
  |> (stream/filter (fn [x] (= (% x 2) 0)))
  |> (stream/take 8)
  |> stream/realize
;; => (0 2 8 34 144 610 2584 10946)
```

---

## Practical Examples

### Infinite Prime Stream

```qi
;; Prime checker
(defn prime? [n]
  (if (< n 2)
    false
    (not (any? (fn [i] (= (% n i) 0)) (range 2 (+ 1 (sqrt n)))))))

;; Infinite stream of primes
(def primes
  (2
   |> (stream/iterate inc)
   |> (stream/filter prime?)))

;; First 20 primes
(stream/realize (stream/take 20 primes))
;; => (2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71)

;; Primes less than 100
(stream/realize
  (stream/take-while (fn [x] (< x 100)) primes))
```

### Large Data Aggregation

```qi
;; Sum of even numbers from 1 to 1 million (memory efficient)
(stream/range 1 1000001)
  |> (stream/filter (fn [x] (= (% x 2) 0)))
  |> stream/realize
  |> (reduce + 0)
;; => 250000500000

;; With regular list, this would consume memory for 1 million items,
;; but streams compute only what's needed
```

### Data Processing Pipeline

```qi
;; Extract errors from log file (conceptual)
(defn process-logs [data]
  (data
   |> stream/stream
   |> (stream/filter (fn [line] (str/contains? line "ERROR")))
   |> (stream/map parse-log-line)
   |> (stream/take 100)  ; First 100 errors
   |> stream/realize))

;; CSV data transformation
(defn process-csv [rows]
  (rows
   |> stream/stream
   |> (stream/drop 1)  ; Skip header
   |> (stream/map (fn [row] (str/split row ",")))
   |> (stream/filter (fn [cols] (> (len cols) 3)))
   |> (stream/map (fn [cols] {:id (first cols) :name (second cols)}))
   |> (stream/take 1000)
   |> stream/realize))
```

### Infinite Data Generation

```qi
;; Infinite counter
(def counter (stream/iterate inc 0))

;; Infinite timestamp sequence (conceptual)
(def timestamps
  (now)
   |> (stream/iterate (fn [t] (+ t 1000))))  ; Every second

;; Infinite random numbers (conceptual)
(def randoms
  (stream/iterate (fn [_] (math/rand)) 0))
```

---

## Advantages and Disadvantages

### Advantages

1. **Memory Efficiency**
   - Can handle huge datasets
   - Uses memory only for what's needed

2. **Infinite Data Structures**
   - Can represent unbounded data
   - Express mathematical concepts directly

3. **Lazy Computation**
   - Skips unnecessary calculations
   - Early termination possible (take)

4. **Composability**
   - Freely combine map/filter/take
   - Express intuitively with pipelines

### Disadvantages

1. **No Random Access**
   - Cannot access elements by index
   - Sequential processing only

2. **Non-reusable**
   - Streams can only be consumed once
   - Must recreate for multiple uses

3. **Harder to Debug**
   - Lazy evaluation makes it unclear when computation happens
   - Use `tap>` for observation

---

## Performance Guidelines

### When to Use Streams

- ✅ Processing large data (files, logs, databases)
- ✅ Infinite data structures (sequences, generators)
- ✅ Early termination needed (only first n elements)
- ✅ Limited memory

### When to Use Regular Collections

- ❌ Small datasets (tens to hundreds of elements)
- ❌ Random access needed
- ❌ Multiple traversals needed
- ❌ Need to keep all elements

### Performance Comparison

```qi
;; Stream: memory efficient, lazy evaluation
(stream/range 1 1000001)
  |> (stream/map (fn [x] (* x x)))
  |> (stream/take 10)
  |> stream/realize
;; => Only 10 elements computed, rest are never calculated

;; Regular list: computes everything
(range 1 1000001)
  |> (map (fn [x] (* x x)))
  |> (take 10)
;; => Computes all 1 million, then takes 10 (wasteful)
```

---

## Summary

Qi's streams enable lazy evaluation for:

- **Memory Efficiency**: Handle large data
- **Infinite Data Structures**: Express mathematical concepts
- **Pipeline Integration**: Write intuitively with `|>`
- **Early Termination**: Compute only what's needed

Streams are powerful for file I/O, data processing, infinite sequences, and more.
