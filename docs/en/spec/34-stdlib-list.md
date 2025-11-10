# Standard Library - List Extended Operations (list/)

**18 List Extension Functions**

Provides advanced operations for lists and vectors. Basic operations (map, filter, reduce, etc.) are included in the Core module.

> **Implementation**: `src/builtins/list.rs`
>
> **Related Documentation**: [06-data-structures.md](06-data-structures.md) - Basic list operations

---

## Overview

The `list/` module provides advanced functions that complement the Core module's basic list operations (map, filter, reduce, etc.).

**Difference from Core**:
- **Core**: Basic operations (map, filter, reduce, take, drop, etc.) - Global namespace
- **list/**: Extended operations (grouping, search, transformation, predicates, etc.) - Requires `list/` prefix

---

## Conditional Take/Drop

### take-while - Take elements while predicate is true

```qi
;; Basic usage
(take-while (fn [x] (< x 5)) [1 2 3 6 7 4])
;; => (1 2 3)

;; Take until blank line
(def lines ["Line 1" "Line 2" "" "Line 3"])
(take-while (fn [s] (not (str/blank? s))) lines)
;; => ("Line 1" "Line 2")

;; Using in pipeline
([1 2 3 6 7 4] |> (take-while (fn [x] (< x 5))))
;; => (1 2 3)
```

### drop-while - Skip elements while predicate is true

```qi
;; Basic usage
(drop-while (fn [x] (< x 5)) [1 2 3 6 7 4])
;; => (6 7 4)

;; Skip header lines
(def lines ["# Header" "---" "Content 1" "Content 2"])
(drop-while (fn [s] (str/starts-with? s "#")) lines)
;; => ("---" "Content 1" "Content 2")

;; Using in pipeline
([1 2 3 6 7 4] |> (drop-while (fn [x] (< x 5))))
;; => (6 7 4)
```

---

## Split/Join

### list/split-at - Split list at specified position

```qi
;; Basic usage
(list/split-at 2 [1 2 3 4 5])
;; => [(1 2) (3 4 5)]

;; Split into first 3 and rest
(list/split-at 3 ["a" "b" "c" "d" "e"])
;; => [("a" "b" "c") ("d" "e")]

;; Using in pipeline
([1 2 3 4 5] |> (list/split-at 2))
;; => [(1 2) (3 4 5)]
```

### list/chunk - Split into fixed-size chunks

```qi
;; Basic usage
(list/chunk 2 [1 2 3 4 5 6])
;; => ((1 2) (3 4) (5 6))

;; Split into groups of 3
(list/chunk 3 [1 2 3 4 5 6 7 8])
;; => ((1 2 3) (4 5 6) (7 8))

;; Using in pipeline
([1 2 3 4 5 6] |> (list/chunk 2))
;; => ((1 2) (3 4) (5 6))
```

### list/interleave - Interleave two lists

```qi
;; Basic usage
(list/interleave [1 2 3] [4 5 6])
;; => (1 4 2 5 3 6)

;; Stops at shorter list
(list/interleave [1 2] [4 5 6 7])
;; => (1 4 2 5)

;; Alternate keys and values
(list/interleave [:a :b :c] [1 2 3])
;; => (:a 1 :b 2 :c 3)
```

### list/zipmap - Create map from two lists

```qi
;; Basic usage
(list/zipmap [:a :b :c] [1 2 3])
;; => {:a 1, :b 2, :c 3}

;; Combine keys and values into map
(def keys [:name :age :email])
(def values ["Alice" 30 "alice@example.com"])
(list/zipmap keys values)
;; => {:name "Alice", :age 30, :email "alice@example.com"}
```

---

## Selection

### list/take-nth - Take every nth element

```qi
;; Basic usage
(list/take-nth 2 [1 2 3 4 5 6])
;; => (1 3 5)

;; Take every 3rd element
(list/take-nth 3 [0 1 2 3 4 5 6 7 8 9])
;; => (0 3 6 9)

;; Using in pipeline
([1 2 3 4 5 6] |> (list/take-nth 2))
;; => (1 3 5)
```

### list/drop-last - Drop last n elements

```qi
;; Basic usage
(list/drop-last 2 [1 2 3 4 5])
;; => (1 2 3)

;; Drop last element
(list/drop-last 1 [1 2 3])
;; => (1 2)

;; Using in pipeline
([1 2 3 4 5] |> (list/drop-last 2))
;; => (1 2 3)
```

---

## Search/Find

### find - Find first element matching predicate

```qi
;; Basic usage
(find (fn [x] (> x 5)) [1 7 3])
;; => 7

;; Find even number
(find even? [1 3 4 5])
;; => 4

;; Returns nil if not found
(find (fn [x] (> x 10)) [1 2 3])
;; => nil

;; Using in pipeline
([1 7 3] |> (find (fn [x] (> x 5))))
;; => 7
```

### list/find-index - Find index of first matching element

```qi
;; Basic usage
(list/find-index (fn [x] (> x 5)) [1 7 3])
;; => 1

;; Find index of even number
(list/find-index even? [1 3 4 5])
;; => 2

;; Returns nil if not found
(list/find-index (fn [x] (> x 10)) [1 2 3])
;; => nil
```

---

## Predicates (Collection-wide)

### list/every? - Check if all elements satisfy predicate

```qi
;; Basic usage
(list/every? (fn [x] (> x 0)) [1 2 3])
;; => true

;; Check if all even
(list/every? even? [2 4 6])
;; => true

;; Returns false if any element fails
(list/every? even? [2 4 5])
;; => false

;; Using in pipeline
([1 2 3] |> (list/every? (fn [x] (> x 0))))
;; => true
```

### list/some? - Check if any element satisfies predicate

```qi
;; Basic usage
(list/some? (fn [x] (> x 5)) [1 7 3])
;; => true

;; Check if any even
(list/some? even? [1 3 5])
;; => false

;; Using in pipeline
([1 7 3] |> (list/some? (fn [x] (> x 5))))
;; => true
```

**Note**: `some?` (1 argument) is a Core predicate function that checks if a value is not nil. `list/some?` takes 2 arguments and checks collection elements.

---

## Sort/Aggregate

### list/sort-by - Sort by key function

```qi
;; Basic usage
(list/sort-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => ({:name "Bob" :age 25} {:name "Alice" :age 30})

;; Sort by string length
(list/sort-by len ["zzz" "a" "bb"])
;; => ("a" "bb" "zzz")

;; Using in pipeline
([{:name "Bob" :age 25} {:name "Alice" :age 30}]
 |> (list/sort-by (fn [u] (get u :age))))
```

### list/max-by - Get element with maximum key value

```qi
;; Basic usage
(list/max-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => {:name "Alice" :age 30}

;; Get longest string
(list/max-by len ["a" "bbb" "cc"])
;; => "bbb"

;; Returns nil for empty list
(list/max-by identity [])
;; => nil
```

### list/min-by - Get element with minimum key value

```qi
;; Basic usage
(list/min-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => {:name "Bob" :age 25}

;; Get shortest string
(list/min-by len ["aaa" "b" "cc"])
;; => "b"

;; Returns nil for empty list
(list/min-by identity [])
;; => nil
```

### list/sum-by - Sum by key function

```qi
;; Basic usage
(list/sum-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => 55

;; Sum of string lengths
(list/sum-by len ["a" "bb" "ccc"])
;; => 6

;; Using in pipeline
([{:name "Bob" :age 25} {:name "Alice" :age 30}]
 |> (list/sum-by (fn [u] (get u :age))))
;; => 55
```

---

## Grouping/Frequency

### list/frequencies - Count occurrences

```qi
;; Basic usage
(list/frequencies [1 2 2 3 3 3])
;; => {"1" 1, "2" 2, "3" 3}

;; String frequencies
(list/frequencies ["a" "b" "a" "c" "b" "a"])
;; => {"a" 3, "b" 2, "c" 1}

;; Using in pipeline
([1 2 2 3 3 3] |> list/frequencies)
;; => {"1" 1, "2" 2, "3" 3}
```

### list/partition-by - Group consecutive elements by predicate

```qi
;; Basic usage
(list/partition-by even? [1 1 2 2 3 3])
;; => ((1 1) (2 2) (3 3))

;; Group consecutive identical values
(list/partition-by identity [1 1 2 2 2 3 3])
;; => ((1 1) (2 2 2) (3 3))

;; Group by string length
(list/partition-by len ["a" "b" "cc" "dd" "eee"])
;; => (("a" "b") ("cc" "dd") ("eee"))
```

---

## Transformation

### keep - Map and filter out nils

```qi
;; Basic usage
(keep (fn [x] (when (even? x) (* x 2))) [1 2 3 4])
;; => (4 8)

;; Elements returning nil are filtered out
(keep (fn [x] (when (> x 2) x)) [1 2 3 4])
;; => (3 4)

;; Using in pipeline
([1 2 3 4] |> (keep (fn [x] (when (even? x) (* x 2)))))
;; => (4 8)
```

### list/dedupe - Remove consecutive duplicates

```qi
;; Basic usage
(list/dedupe [1 1 2 2 3 3])
;; => (1 2 3)

;; Non-consecutive duplicates remain
(list/dedupe [1 2 1 2])
;; => (1 2 1 2)

;; Using in pipeline
([1 1 2 2 3 3] |> list/dedupe)
;; => (1 2 3)
```

**Difference from distinct**:
- `distinct`: Removes all duplicates (`[1 2 1 2]` → `[1 2]`)
- `list/dedupe`: Removes only consecutive duplicates (`[1 2 1 2]` → `(1 2 1 2)`)

---

## Practical Examples

### Data Analysis Pipeline

```qi
;; Filter even numbers and sum
([1 2 3 4 5 6]
 |> (filter even?)
 |> (reduce + 0))
;; => 12

;; Group and count
(def data [1 1 2 2 2 3 3])
(data
 |> list/frequencies)
;; => {"1" 2, "2" 3, "3" 2}
```

### User Search and Aggregation

```qi
(def users [
  {:name "Alice" :age 30 :dept "Sales"}
  {:name "Bob" :age 25 :dept "Dev"}
  {:name "Carol" :age 35 :dept "Sales"}
])

;; Get oldest user
(users |> (list/max-by (fn [u] (get u :age))))
;; => {:name "Carol" :age 35 :dept "Sales"}

;; Sum of ages
(users |> (list/sum-by (fn [u] (get u :age))))
;; => 90

;; Find Sales department user
(users |> (find (fn [u] (= (get u :dept) "Sales"))))
;; => {:name "Alice" :age 30 :dept "Sales"}
```

### Data Transformation

```qi
;; Process CSV data in pairs
(def csv-row ["Name" "Alice" "Age" "30" "City" "Tokyo"])
(csv-row |> (list/chunk 2))
;; => (("Name" "Alice") ("Age" "30") ("City" "Tokyo"))

;; Create map from keys and values
(list/zipmap [:name :age :city] ["Alice" 30 "Tokyo"])
;; => {:name "Alice", :age 30, :city "Tokyo"}

;; Sample every 100th element
(def large-data (range 1000))
(large-data |> (list/take-nth 100))
;; => (0 100 200 300 400 500 600 700 800 900)
```

### Log Analysis

```qi
;; Process log files
(def logs [
  "INFO: Starting..."
  "INFO: Connected"
  "ERROR: Connection lost"
  "INFO: Retrying..."
])

;; Find ERROR log
(logs |> (find (fn [s] (str/starts-with? s "ERROR:"))))
;; => "ERROR: Connection lost"

;; Check if all are INFO
(logs |> (list/every? (fn [s] (str/starts-with? s "INFO:"))))
;; => false

;; Check if any ERROR exists
(logs |> (list/some? (fn [s] (str/starts-with? s "ERROR:"))))
;; => true
```

### Batch Processing

```qi
;; Process large data in batches of 100
(def data (range 1000))

(data
 |> (list/chunk 100)
 |> (each (fn [batch]
            (println f"Processing batch of {(len batch)} items...")
            ;; Batch processing logic
            )))
```

---

## Function List

### Conditional Take/Drop
- `take-while` - Take elements while predicate is true
- `drop-while` - Skip elements while predicate is true
- `list/drop-last` - Drop last n elements

### Split/Join
- `list/split-at` - Split at specified position
- `list/chunk` - Split into fixed-size chunks
- `list/interleave` - Interleave two lists
- `list/zipmap` - Create map from two lists

### Selection
- `list/take-nth` - Take every nth element

### Search/Find
- `find` - Find first matching element
- `list/find-index` - Find index of first matching element

### Predicates
- `list/every?` - Check if all elements satisfy predicate
- `list/some?` - Check if any element satisfies predicate

### Sort/Aggregate
- `list/sort-by` - Sort by key function
- `list/max-by` - Get element with maximum key value
- `list/min-by` - Get element with minimum key value
- `list/sum-by` - Sum by key function

### Grouping/Frequency
- `list/frequencies` - Count occurrences
- `list/partition-by` - Group consecutive elements by predicate

### Transformation
- `keep` - Map and filter out nils
- `list/dedupe` - Remove consecutive duplicates
