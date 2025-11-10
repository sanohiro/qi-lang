# Chapter 2: Thinking in Pipelines

**Time Required**: 20 minutes

Pipelines are one of Qi's most powerful features. They allow you to **describe data flow intuitively**.

---

## Why Pipelines?

Traditional nested function calls tend to become hard to read.

### Nested Style (Hard to Read)

```qi
(str/upper (str/reverse (str/trim "  hello  ")))
; => "OLLEH"
```

You need to read from inside out, making the data flow unclear.

### Pipeline Style (Easy to Read)

```qi
("  hello  "
 |> str/trim
 |> str/reverse
 |> str/upper)
; => "OLLEH"
```

**Data flows from left to right**, making the processing order immediately clear!

---

## Pipeline Operator `|>`

`|>` passes the left value to the right function.

```qi
qi> (10 |> inc)
; => 11

qi> (5 |> (* 2))
; => 10

qi> (5 |> (fn [x] (* x 2)))
; => 10
```

---

## Practical Example 1: Numeric Processing

```qi
qi> (10
     |> (+ 5)        ; 10 + 5 = 15
     |> (* 2)        ; 15 * 2 = 30
     |> (- 10))      ; 30 - 10 = 20
; => 20
```

**Data flow**:
```
10 → 15 → 30 → 20
```

---

## Practical Example 2: String Processing

```qi
qi> ("hello world"
     |> str/upper        ; "HELLO WORLD"
     |> (str/replace " " "-")  ; "HELLO-WORLD"
     |> (str/concat "!"))      ; "HELLO-WORLD!"
; => "HELLO-WORLD!"
```

---

## Practical Example 3: List Processing

```qi
qi> ([1 2 3 4 5 6 7 8 9 10]
     |> (filter even?)           ; [2 4 6 8 10]
     |> (map (fn [x] (* x x)))   ; [4 16 36 64 100]
     |> (reduce + 0))            ; 220
; => 220
```

**Processing flow**:
1. Extract even numbers: `[2 4 6 8 10]`
2. Square each element: `[4 16 36 64 100]`
3. Sum all: `220`

---

## Pipelines and Debugging: `tap>`

`tap>` executes side effects (like print) without modifying the data. Useful for debugging.

```qi
qi> ([1 2 3 4 5]
     |> (tap println)              ; Debug output
     |> (map (fn [x] (* x 2)))
     |> (tap println)              ; Debug output
     |> (reduce + 0))
; => Output: [1 2 3 4 5]
; => Output: [2 4 6 8 10]
; => 30
```

`tap>` lets you inspect data state in the middle of a pipeline.

---

## Defining Functions with Pipelines

You can write readable functions using pipelines.

### Example: Text Formatting Function

```qi
(defn format-title [text]
  (text
   |> str/trim
   |> str/lower
   |> (str/replace " " "-")
   |> (str/concat "title-")))

qi> (format-title "  Hello World  ")
; => "title-hello-world"
```

### Example: Data Aggregation Function

```qi
(defn sum-of-squares [numbers]
  (numbers
   |> (map (fn [x] (* x x)))
   |> (reduce + 0)))

qi> (sum-of-squares [1 2 3 4 5])
; => 55  (1 + 4 + 9 + 16 + 25)
```

---

## Practical Example 4: CSV Data Processing (Mock)

```qi
(def csv-data
  "Alice,25,Engineer
Bob,30,Designer
Carol,28,Manager")

(defn parse-csv-line [line]
  (let [parts (str/split line ",")]
    {:name (first parts)
     :age (nth parts 1)
     :role (nth parts 2)}))

(csv-data
 |> (str/split "\n")
 |> (map parse-csv-line))
; => [{:name "Alice" :age "25" :role "Engineer"}
;     {:name "Bob" :age "30" :role "Designer"}
;     {:name "Carol" :age "28" :role "Manager"}]
```

---

## Practical Example 5: Statistical Processing

```qi
(defn analyze-scores [scores]
  {:count (len scores)
   :sum (reduce + 0 scores)
   :average (/ (reduce + 0 scores) (len scores))
   :max (reduce max 0 scores)
   :min (reduce min 100 scores)})

(def test-scores [85 92 78 90 88])

(test-scores |> analyze-scores)
; => {:count 5 :sum 433 :average 86.6 :max 92 :min 78}
```

---

## Benefits of Pipelines

### 1. Readable

Data flow is immediately clear:

```qi
; ❌ Hard to read
(reduce + 0 (map (fn [x] (* x x)) (filter even? [1 2 3 4 5 6])))

; ✅ Easy to read
([1 2 3 4 5 6]
 |> (filter even?)
 |> (map (fn [x] (* x x)))
 |> (reduce + 0))
```

### 2. Easy to Debug

Check intermediate results with `tap>`:

```qi
([1 2 3 4 5 6]
 |> (filter even?)
 |> (tap println)  ; ← Check here
 |> (map (fn [x] (* x x)))
 |> (tap println)  ; ← Check here
 |> (reduce + 0))
```

### 3. Easy to Extend

Adding processing steps in the middle is easy:

```qi
(data
 |> step1
 |> step2
 |> new-step  ; ← Easy to add
 |> step3)
```

---

## Pipelines and Function Composition

Pipelines combine multiple functions to create new ones.

```qi
(defn double [x] (* x 2))
(defn add-ten [x] (+ x 10))
(defn square [x] (* x x))

; Compose with pipeline
(defn transform [x]
  (x
   |> double
   |> add-ten
   |> square))

qi> (transform 5)
; => 400  ((5 * 2 + 10) ^ 2 = 20 ^ 2 = 400)
```

---

## Practice Problems

### Problem 1: URL Slug Generation

Write a function that generates a URL slug from a blog title.

```qi
(defn make-slug [title]
  ; Fill this in
  ; Hint: Use str/trim, str/lower, (str/replace " " "-")
  )

; Tests
(make-slug "  Hello World  ")  ; => "hello-world"
(make-slug "Qi Programming Language")  ; => "qi-programming-language"
```

<details>
<summary>Solution</summary>

```qi
(defn make-slug [title]
  (title
   |> str/trim
   |> str/lower
   |> (str/replace " " "-")))
```

</details>

### Problem 2: Sum of Even Squares

Write a function that extracts even numbers from a list, squares each, and sums them all.

```qi
(defn sum-even-squares [numbers]
  ; Fill this in
  )

; Test
(sum-even-squares [1 2 3 4 5 6])  ; => 56  (4 + 16 + 36)
```

<details>
<summary>Solution</summary>

```qi
(defn sum-even-squares [numbers]
  (numbers
   |> (filter even?)
   |> (map (fn [x] (* x x)))
   |> (reduce + 0)))
```

</details>

### Problem 3: Name List Formatting

Write a function that takes a list of names, converts them all to uppercase, and sorts alphabetically.

```qi
(defn format-names [names]
  ; Fill this in
  )

; Test
(format-names ["charlie" "alice" "bob"])
; => ["ALICE" "BOB" "CHARLIE"]
```

<details>
<summary>Solution</summary>

```qi
(defn format-names [names]
  (names
   |> (map str/upper)
   |> sort))
```

</details>

---

## Summary

What you learned in this chapter:

- ✅ How to use the pipeline operator `|>`
- ✅ Thinking about designing data flow
- ✅ Debugging with `tap>`
- ✅ Practical pipeline examples (strings, numbers, lists)
- ✅ Pipelines and function composition

---

## Next Steps

Now that you can freely handle data flow with pipelines, let's learn **pattern matching** next!

➡️ [Chapter 3: Mastering Pattern Matching](03-pattern-matching.md)

Pattern matching is a powerful feature for branching processing based on data structure.
