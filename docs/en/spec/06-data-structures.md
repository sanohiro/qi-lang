# Data Structures

**Qi's Collection Types and Operations**

> **Implementation**: `src/builtins/core_collections.rs`, `src/builtins/list.rs`, `src/builtins/map.rs`, `src/builtins/set.rs`

---

## When to Use Lists vs Vectors

### When to Use Lists

**Lists `(...)`** are suitable for:

- **Recursive data processing** - Natural decomposition with first/rest
- **Lisp-style processing** - Quoted code, S-expressions
- **Functional programming** - Good compatibility with pattern matching
- **Sequential processing** - When processing from the beginning

```qi
;; Recursive processing
(defn sum [lst]
  (if (empty? lst)
    0
    (+ (first lst) (sum (rest lst)))))

(sum (list 1 2 3 4 5))  ;; => 15
```

### When to Use Vectors

**Vectors `[...]`** are suitable for:

- **Random access** - O(1) access to any position with nth function
- **Performance-focused** - Better memory efficiency
- **JSON interoperability** - Natural correspondence with JSON arrays
- **Large datasets** - Efficient memory usage

```qi
;; Random access
(def data [10 20 30 40 50])
(nth data 2)  ;; => 30 (fast)

;; JSON interoperability
(json/stringify {:items [1 2 3]})
;; => "{\"items\":[1,2,3]}"
```

### Practical Guidelines

- **Default to vectors** - Use vectors when in doubt (modern, JSON-compatible)
- **Pipeline processing** - Both work the same way
- **Pattern matching** - Both supported, lists are more idiomatic
- **Performance** - Vectors are faster (though the difference is rarely noticeable)

```qi
;; Both work the same way
[1 2 3] |> (map inc) |> (filter even?)  ;; => [2 4]
(list 1 2 3) |> (map inc) |> (filter even?)  ;; => (2 4)
```

---

## Type Preservation Rules for Lists and Vectors

In Qi, collection operation functions determine return value types according to the following rules:

### Transformation Functions (Process Input)
**Preserve input type**. List input → List return, Vector input → Vector return.

- `map`, `filter`, `reduce` (higher-order functions)
- `sort`, `distinct`, `reverse`
- `take`, `drop`, `rest`, `take-while`, `drop-while`

```qi
;; Vector input → Vector return
(map inc [1 2 3])              ;; => [2 3 4]
(filter even? [1 2 3 4])       ;; => [2 4]
(sort [3 1 4])                 ;; => [1 3 4]

;; List input → List return
(map inc (list 1 2 3))         ;; => (2 3 4)
(filter even? (list 1 2 3 4))  ;; => (2 4)
```

### Concatenation Functions (Combine Multiple Collections)
**Prioritize the first argument's type**.

- `concat` - Returns Vector if first argument is Vector
- `zip` - Returns Vector if first argument is Vector
- `cons` - Preserves the type of the second argument (collection side)

```qi
;; Prioritize first argument's type
(concat [1 2] (list 3 4))      ;; => [1 2 3 4]
(concat (list 1 2) [3 4])      ;; => (1 2 3 4)
(zip [1 2] (list "a" "b"))     ;; => [[1 "a"] [2 "b"]]

;; cons preserves second argument's type
(cons 0 [1 2 3])               ;; => [0 1 2 3]
(cons 0 (list 1 2 3))          ;; => (0 1 2 3)
```

### Construction Functions (Generate New Collections)
**Always return Lists** (Lisp-family language convention).

- `range`, `repeat`, `flatten`
- `keys`, `vals` (map operations)

```qi
(range 5)                      ;; => (0 1 2 3 4)
(repeat 3 "x")                 ;; => ("x" "x" "x")
(flatten [[1 2] [3 4]])        ;; => (1 2 3 4)
```

### Type Conversion Functions
Explicit type conversion as needed.

- `list` - Create List from variadic arguments
- `vector` - Create Vector from variadic arguments
- `to-list` - Convert List/Vector to List
- `to-vector` - Convert List/Vector to Vector

```qi
(list 1 2 3)                   ;; => (1 2 3)
(vector 1 2 3)                 ;; => [1 2 3]
(to-list [1 2 3])              ;; => (1 2 3)
(to-vector (list 1 2 3))       ;; => [1 2 3]
```

---

## Vectors

### Basics

```qi
[1 2 3]           ;; Vector of numbers
["a" "b" "c"]     ;; Vector of strings
[1 "hello" :key]  ;; Mixed types allowed
[]                ;; Empty vector
```

### Access

```qi
;; nth - Get nth element (0-indexed)
(nth [10 20 30] 1)     ;; => 20

;; first - First element
(first [10 20 30])     ;; => 10

;; last - Last element
(last [10 20 30])      ;; => 30

;; rest - All except first element
(rest [10 20 30])      ;; => [20 30]
```

### Adding & Combining

```qi
;; cons - Add element to front (preserves collection type)
(cons 0 [10 20 30])       ;; => [0 10 20 30]

;; conj - Add element to end (Vectors add to end, Lists add to front)
(conj [10 20 30] 40)      ;; => [10 20 30 40]

;; concat - Concatenate lists (prioritizes first argument's type)
(concat [10 20] [30 40])  ;; => [10 20 30 40]
```

### Taking & Skipping

```qi
;; take - Get first n elements (preserves type)
(take 2 [10 20 30 40])    ;; => [10 20]

;; drop - Skip first n elements (preserves type)
(drop 2 [10 20 30 40])    ;; => [30 40]

;; take-while - Take while predicate is true
(take-while (fn [x] (< x 5)) [1 2 3 6 7 4])  ;; => (1 2 3)

;; drop-while - Drop while predicate is true
(drop-while (fn [x] (< x 5)) [1 2 3 6 7 4])  ;; => (6 7 4)

;; list/drop-last - Drop last n elements
(list/drop-last 2 [1 2 3 4 5])  ;; => (1 2 3)

;; list/take-nth - Take every nth element
(list/take-nth 2 [1 2 3 4 5 6])  ;; => (1 3 5)

;; list/split-at - Split at specified position
(list/split-at 2 [1 2 3 4 5])  ;; => [(1 2) (3 4 5)]
```

### Transformation

```qi
;; reverse - Reverse order (preserves type)
(reverse [10 20 30])         ;; => [30 20 10]

;; flatten - Flatten nesting (always returns List)
(flatten [[1 2] [3 [4 5]]])  ;; => (1 2 3 4 5)

;; distinct - Remove duplicates (preserves type)
(distinct [1 2 2 3 3 3])     ;; => [1 2 3]

;; list/dedupe - Remove consecutive duplicates
(list/dedupe [1 1 2 2 3 3])  ;; => (1 2 3)
(list/dedupe [1 2 1 2])      ;; => (1 2 1 2) (non-consecutive remain)

;; sort - Sort ascending (preserves type)
(sort [3 1 4 1 5])           ;; => [1 1 3 4 5]

;; list/interleave - Interleave two lists
(list/interleave [1 2 3] [4 5 6])  ;; => (1 4 2 5 3 6)

;; list/chunk - Split into chunks of specified size
(list/chunk 2 [1 2 3 4 5 6])  ;; => ((1 2) (3 4) (5 6))

;; list/zipmap - Create map from two lists
(list/zipmap [:a :b :c] [1 2 3])  ;; => {:a 1, :b 2, :c 3}
```

### Size & State

```qi
;; len - Return number of elements
(len [10 20 30])      ;; => 3

;; count - Return number of elements (alias for len)
(count [10 20 30])    ;; => 3

;; empty? - Check if empty
(empty? [])           ;; => true
(empty? [1])          ;; => false
```

---

## Lists

### Basics

```qi
'(1 2 3)          ;; Quote required
'()               ;; Empty list

(first '(1 2 3))  ;; => 1
(rest '(1 2 3))   ;; => (2 3)
```

### Creation

```qi
;; list - Create List from variadic arguments
(list 1 2 3)      ;; => (1 2 3)
(list)            ;; => ()

;; range - Generate range of numbers
(range 5)         ;; => (0 1 2 3 4)
(range 2 5)       ;; => (2 3 4)

;; repeat - Repeat same value n times
(repeat 5 0)      ;; => (0 0 0 0 0)
(repeat 3 "a")    ;; => ("a" "a" "a")
(repeat 2 [1 2])  ;; => ([1 2] [1 2])
```

---

## Maps

### Basics

```qi
{:name "Alice" :age 30}    ;; Keywords as keys
{"name" "Bob" "age" 25}    ;; Strings as keys
{}                         ;; Empty map
```

### Valid Map Key Types

Qi maps support only **type-safe keys** (`MapKey` type). The following 4 types can be used as keys:

| Type | Example | Usage |
|---|---|---|
| **Keyword** | `:name`, `:age` | Most common. Keys for structured data |
| **String** | `"name"`, `"email"` | JSON-compatible, external data integration |
| **Symbol** | `'foo`, `'bar` | Macros, metaprogramming |
| **Integer** | `0`, `1`, `42` | Array-like access, indices |

#### Floats Cannot Be Keys

**Reason**: Floating-point numbers have unstable hashing, so they cannot be used as map keys.

```qi
;; ✅ Valid types
{:name "Alice"}           ;; Keyword
{"email" "test@test.com"} ;; String
{42 "value"}              ;; Integer

;; ❌ Error
{3.14 "pi"}               ;; Float - Error: Floats cannot be keys
```

#### Internal Implementation

Internally, map keys are represented as the `MapKey` enum type:

```rust
pub enum MapKey {
    Keyword(Arc<str>),   // :name
    Symbol(Arc<str>),    // 'symbol
    String(String),      // "text"
    Integer(i64),        // 42
}
```

This design provides:
- **Type safety**: Key types are checked at compile time
- **Performance**: Fast comparison via Arc<str> interning
- **Memory efficiency**: Same keywords/symbols share memory

### Access

```qi
;; get - Get value by key
(get {:name "Alice" :age 30} :name)   ;; => "Alice"
(get {:name "Alice"} :age 0)          ;; => 0 (default value)

;; Keywords and strings can be used as functions (concise syntax)
(:name {:name "Alice" :age 30})       ;; => "Alice"
(:age {:name "Alice" :age 30})        ;; => 30
("name" {"name" "Bob" "age" 25})      ;; => "Bob"

;; Use with pipelines
(def response {:status 200 :body "OK"})
(response |> :status)                 ;; => 200

;; Access external data (JSON, etc.)
(def user-data {"name" "Carol" "email" "carol@example.com"})
("email" user-data)                   ;; => "carol@example.com"

;; keys - Get all keys
(keys {:name "Alice" :age 30})        ;; => ("name" "age")

;; vals - Get all values
(vals {:name "Alice" :age 30})        ;; => ("Alice" 30)
```

### Adding & Removing

```qi
;; assoc - Add key and value
(assoc {:name "Alice"} :age 30)           ;; => {:name "Alice" :age 30}

;; dissoc - Remove key
(dissoc {:name "Alice" :age 30} :age)     ;; => {:name "Alice"}
```

### Merging & Selecting

```qi
;; merge - Combine maps
(merge {:a 1} {:b 2})                         ;; => {:a 1 :b 2}
(merge {:a 1} {:a 2})                         ;; => {:a 2} (right takes precedence)

;; map/select-keys - Extract only specified keys
(map/select-keys {:a 1 :b 2 :c 3} [:a :c])   ;; => {:a 1 :c 3}
```

### Nested Operations

```qi
;; update - Update value with function
(update {:name "Alice" :age 30} :age inc)
;; => {:name "Alice" :age 31}

;; update-in - Update nested value
(update-in {:user {:profile {:visits 10}}} [:user :profile :visits] inc)
;; => {:user {:profile {:visits 11}}}

;; get-in - Get nested value
(get-in {:user {:name "Bob"}} [:user :name])    ;; => "Bob"
(get-in {} [:user :name] "guest")               ;; => "guest"

;; map/assoc-in - Set nested value
(map/assoc-in {} [:user :profile :name] "Alice")
;; => {:user {:profile {:name "Alice"}}}
(map/assoc-in {:user {:age 30}} [:user :name] "Bob")
;; => {:user {:age 30, :name "Bob"}}

;; map/dissoc-in - Remove nested key
(map/dissoc-in {:user {:name "Alice" :age 30}} [:user :age])
;; => {:user {:name "Alice"}}
(map/dissoc-in {:a {:b {:c 1}}} [:a :b :c])
;; => {:a {:b {}}}
```

### Bulk Map Transformations

```qi
;; map/map-keys - Transform keys
(map/map-keys str/upper {:name "Alice" :age 30})
;; => {"NAME" "Alice", "AGE" 30}

;; map/map-vals - Transform values
(map/map-vals inc {:a 1 :b 2})
;; => {:a 2, :b 3}

;; map/filter-keys - Filter by keys
(map/filter-keys keyword? {:name "Alice" "age" 30})
;; => {:name "Alice"}

;; map/filter-vals - Filter by values
(map/filter-vals even? {:a 1 :b 2 :c 3})
;; => {:b 2}
```

---

## Sets (Set Operations)

```qi
;; set/union - Union
(set/union [1 2] [2 3])                         ;; => [1 2 3]

;; set/intersect - Intersection
(set/intersect [1 2 3] [2 3 4])                 ;; => [2 3]

;; set/difference - Difference
(set/difference [1 2 3] [2])                    ;; => [1 3]

;; set/symmetric-difference - Symmetric difference
(set/symmetric-difference [1 2 3] [2 3 4])      ;; => [1 4]

;; set/subset? - Subset check
(set/subset? [1 2] [1 2 3])                     ;; => true

;; set/superset? - Superset check
(set/superset? [1 2 3] [1 2])                   ;; => true

;; set/disjoint? - Disjoint check
(set/disjoint? [1 2] [3 4])                     ;; => true
```

---

## Higher-Order Functions

### map - Apply Function to Each Element

```qi
;; Vector input → Vector return
(map inc [1 2 3])                    ;; => [2 3 4]
(map str [1 2 3])                    ;; => ["1" "2" "3"]

;; List input → List return
(map inc (list 1 2 3))               ;; => (2 3 4)

;; Use in pipeline
([1 2 3] |> (map (fn [x] (* x x))))  ;; => [1 4 9]
```

### filter - Extract Elements Matching Condition

```qi
;; Vector input → Vector return
(filter even? [1 2 3 4 5])                      ;; => [2 4]
(filter (fn [x] (> x 10)) [5 15 3 20])          ;; => [15 20]

;; List input → List return
(filter even? (list 1 2 3 4 5))                 ;; => (2 4)

;; Use in pipeline
([1 2 3 4 5] |> (filter odd?))                  ;; => [1 3 5]
```

### reduce - Fold

```qi
(reduce + 0 [1 2 3 4])        ;; => 10
(reduce * 1 [2 3 4])          ;; => 24

;; Use in pipeline
([1 2 3 4 5] |> (reduce + 0)) ;; => 15
```

### each - Apply Function to Each Element (For Side Effects)

Unlike `map`, does not collect return values and returns `nil`. Use for side effects (println, file writing, etc.).

```qi
;; Basic usage
(each println [1 2 3])
;; Output:
;; 1
;; 2
;; 3
;; => nil

;; Works with both Vectors and Lists
(each println (list "a" "b" "c"))
;; Output:
;; a
;; b
;; c
;; => nil

;; With lambda expression
(each (fn [x] (println f"Value: {x}")) [10 20 30])
;; Output:
;; Value: 10
;; Value: 20
;; Value: 30
;; => nil

;; Use in pipeline
(lines
 |> (map str/trim)
 |> (map str/upper)
 |> (each println))
;; Convert each line to uppercase and print

;; Conditional processing with when
(data
 |> (each (fn [item]
            (when (> (len item) 0)
              (println f"Processing: {item}")))))

;; Statistical aggregation
(def count (atom 0))
(data
 |> (each (fn [item]
            (when (valid? item)
              (swap! count inc)))))
```

**Choosing between map and each**:
- `map`: When return value is needed (data transformation)
- `each`: When return value is not needed (side effects only)

```qi
;; map - Returns transformation result
(map inc [1 2 3])  ;; => [2 3 4]

;; each - Side effects only, returns nil
(each println [1 2 3])  ;; => nil (but each element is printed)
```

### find - First Element Matching Condition

```qi
(find (fn [x] (> x 5)) [1 7 3])     ;; => 7
(find even? [1 3 4 5])              ;; => 4
```

### Predicates (Whole Collection Check)

```qi
;; list/every? - Check if all elements satisfy condition
(list/every? (fn [x] (> x 0)) [1 2 3])   ;; => true
(list/every? even? [2 4 6])              ;; => true

;; list/some? - Check if any element satisfies condition
(list/some? (fn [x] (> x 5)) [1 7 3])    ;; => true
(list/some? even? [1 3 5])               ;; => false
```

**Note:** `some?` (1 argument) is a Core predicate function that checks if value is not nil (→ See Basic Syntax).

---

## Sorting & Grouping

### Sorting

```qi
;; sort - Sort ascending (preserves type)
(sort [3 1 4 1 5])                             ;; => [1 1 3 4 5]
(sort ["zebra" "apple" "banana"])              ;; => ["apple" "banana" "zebra"]
(sort (list 3 1 4 1 5))                        ;; => (1 1 3 4 5)

;; list/sort-by - Sort by key
(list/sort-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => ({:name "Bob" :age 25} {:name "Alice" :age 30})
```

### Grouping

```qi
;; list/group-by - Group by key function
(list/group-by even? [1 2 3 4 5 6])
;; => {true [2 4 6], false [1 3 5]}

;; list/partition - Partition into 2 groups by predicate
(list/partition even? [1 2 3 4])
;; => [[2 4] [1 3]]
```

### Frequency & Counting

```qi
;; list/frequencies - Count occurrences
(list/frequencies [1 2 2 3 3 3])
;; => {1 1, 2 2, 3 3}

;; list/count-by - Count by predicate
(list/count-by even? [1 2 3 4])
;; => {true 2, false 2}
```

---

## Practical Examples

### Data Analysis Pipeline

```qi
;; Remove duplicates and sort
([5 2 8 2 9 1 3 8 4]
 |> distinct
 |> sort)
;; => (1 2 3 4 5 8 9)

;; Group and aggregate
(list/group-by (fn [n] (% n 3)) [1 2 3 4 5 6 7 8 9])
;; => {0 (3 6 9), 1 (1 4 7), 2 (2 5 8)}
```

### User Search

```qi
(def users [{:name "Alice" :age 30} {:name "Bob" :age 25}])

;; Find user by name
(find (fn [u] (= (get u :name) "Bob")) users)
;; => {:name "Bob" :age 25}

;; Check if all are adults
(list/every? (fn [u] (>= (get u :age) 20)) users)
;; => true

;; Search via pipeline
(users
 |> (filter (fn [u] (>= (get u :age) 25)))
 |> (map (fn [u] (get u :name))))
;; => ("Alice" "Bob")
```
