# Lisp Language Basics

Qi is a **Lisp-family language**. This tutorial explains how to read parentheses and basic concepts for those new to Lisp-family languages.

## Table of Contents

- [What is a Lisp-family Language](#what-is-a-lisp-family-language)
- [How to Read Parentheses](#how-to-read-parentheses)
- [Function Calls](#function-calls)
- [Comparison with Other Languages](#comparison-with-other-languages)
- [Nested Expressions](#nested-expressions)
- [Why This Syntax](#why-this-syntax)
- [Next Steps](#next-steps)

---

## What is a Lisp-family Language

Lisp (List Processing) is a historic programming language family created in 1958.

**Characteristics:**
- **Expressions enclosed in parentheses**: Everything is a list (expression enclosed in parentheses)
- **Prefix notation**: Operators come first
- **Simple syntax**: Less to memorize
- **Code is data**: Programs themselves are data structures

**Representative Lisp-family languages:**
- Common Lisp
- Scheme
- Clojure (Lisp on JVM)
- Racket
- And **Qi**!

---

## How to Read Parentheses

### Basic Rule

```qi
(function-name arg1 arg2 arg3)
```

The **first element** inside the parentheses is the **function name**, and the rest are **arguments**.

### Example: Addition

```qi
(+ 1 2 3)
;; => 6
```

This means "add 1, 2, and 3".

**Comparison with other languages:**

| Language | Syntax |
|----------|--------|
| JavaScript | `1 + 2 + 3` |
| Python | `1 + 2 + 3` |
| **Qi (Lisp)** | `(+ 1 2 3)` |

---

## Function Calls

### Simple Examples

```qi
;; Multiplication
(* 2 3)
;; => 6

;; Division
(/ 10 2)
;; => 5

;; Print string
(println "Hello, World!")
;; => Hello, World!
```

### Multiple Arguments

In Lisp, many functions can take **multiple arguments**.

```qi
;; Add multiple numbers
(+ 1 2 3 4 5)
;; => 15

;; Multiply multiple numbers
(* 2 3 4)
;; => 24

;; Find maximum value
(max 10 5 8 20 3)
;; => 20
```

---

## Comparison with Other Languages

### Example 1: Addition

**JavaScript:**
```javascript
1 + 2 + 3
```

**Python:**
```python
1 + 2 + 3
```

**Qi:**
```qi
(+ 1 2 3)
```

### Example 2: Function Calls

**JavaScript:**
```javascript
Math.max(10, 5, 8)
```

**Python:**
```python
max(10, 5, 8)
```

**Qi:**
```qi
(max 10 5 8)
```

In Qi, **no commas are needed**!

---

## Nested Expressions

You can put parentheses inside parentheses (nesting).

### Example: Nested Calculations

```qi
(+ 1 (* 2 3))
;; => 7
```

**How to read:**
1. Read from the inside: `(* 2 3)` → `6`
2. Calculate the outside: `(+ 1 6)` → `7`

**Comparison with other languages:**

| Language | Syntax |
|----------|--------|
| JavaScript/Python | `1 + (2 * 3)` |
| **Qi (Lisp)** | `(+ 1 (* 2 3))` |

### A More Complex Example

```qi
(* (+ 1 2) (+ 3 4))
;; => 21
```

**How to read:**
1. `(+ 1 2)` → `3`
2. `(+ 3 4)` → `7`
3. `(* 3 7)` → `21`

**Comparison with other languages:**

| Language | Syntax |
|----------|--------|
| JavaScript/Python | `(1 + 2) * (3 + 4)` |
| **Qi (Lisp)** | `(* (+ 1 2) (+ 3 4))` |

---

## Why This Syntax

### Benefits

#### 1. **Consistency**
Everything follows the same pattern: `(function args...)`

```qi
(+ 1 2)           ;; Operator
(println "Hi")    ;; Function
(if true 1 2)     ;; Conditional
(def x 10)        ;; Variable definition
```

All written the same way!

#### 2. **Extensible**
Adding new operators doesn't change the syntax.

```qi
;; Existing operator
(+ 1 2 3)

;; Custom operator (hypothetical)
(my-operator 1 2 3)
```

#### 3. **Parentheses eliminate ambiguity**
No need to memorize operator precedence.

**Other languages (ambiguous):**
```
1 + 2 * 3  ;; 7? 9? Which one?
```

**Qi (clear):**
```qi
(+ 1 (* 2 3))  ;; 7
(* (+ 1 2) 3)  ;; 9
```

#### 4. **Powerful macros**
Since code is data, it's easy to write code that generates code (for advanced users).

---

## Practice: Try in the REPL

Let's launch the REPL and try it out:

```bash
qi
```

### Basic Calculations

```qi
qi:1> (+ 1 2)
3

qi:2> (* 5 6)
30

qi:3> (+ 1 (* 2 3))
7
```

### Define Variables

```qi
qi:4> (def x 10)
10

qi:5> (def y 5)
5

qi:6> (+ x y)
15

qi:7> (* x y)
50
```

### Define Functions

```qi
qi:8> (defn square [n] (* n n))
Function(square)

qi:9> (square 5)
25

qi:10> (square 10)
100
```

### Nested Calculations

```qi
qi:11> (square (+ 2 3))
25

qi:12> (+ (square 3) (square 4))
25
```

---

## Frequently Asked Questions

### Q: Aren't there too many parentheses?

**A:** It might seem like a lot at first, but you'll get used to it quickly! With your editor's parenthesis matching feature, it's no problem.

Also, Qi has **pipeline operators** that can reduce parentheses:

```qi
;; Many parentheses
(reduce + 0 (filter (fn [x] (> x 5)) (map (fn [x] (* x 2)) [1 2 3 4 5 6 7 8 9 10])))

;; Clean with pipeline
([1 2 3 4 5 6 7 8 9 10]
 |> (map (fn [x] (* x 2)))
 |> (filter (fn [x] (> x 5)))
 |> (reduce + 0))
```

### Q: Why is `+` at the front?

**A:** This is called **prefix notation**. Benefits include:

1. **Naturally write multiple arguments**: `(+ 1 2 3 4 5)`
2. **Clear precedence**: Enclosed in parentheses
3. **Consistency**: Functions and operators follow the same rule

### Q: Where are the commas?

**A:** In Lisp, **commas are unnecessary**. Spaces are sufficient.

```qi
;; ✓ OK
(max 10 5 8)

;; ✗ Don't write commas
(max 10, 5, 8)  ;; Not an error, but commas are interpreted as variable names
```

---

## Summary

You've learned the basics of Lisp-family languages:

- ✅ Enclose expressions in parentheses: `(function args...)`
- ✅ Prefix notation: Operators come first
- ✅ Nesting: Parentheses inside parentheses
- ✅ Consistency: Everything follows the same pattern
- ✅ No commas needed: Separate with spaces

Once you get used to it, this consistency becomes comfortable!

---

## Next Steps

Now that you understand the basics of Lisp-family languages, let's actually use Qi:

➡️ **[Tutorial: Getting Started with Qi](01-getting-started.md)**

---

## Reference: Other Lisp-family Languages

If you're interested, try other Lisp-family languages:

- **Clojure** - Modern Lisp on the JVM, popular for web development
- **Racket** - Lisp designed for education, excellent documentation
- **Common Lisp** - Classic Lisp, powerful but steep learning curve
- **Scheme** - Simple and beautiful Lisp, academically popular

Qi is influenced by these Lisps while evolving uniquely with a focus on **pipelines** and **concurrency**.

---

Happy Lisping with Qi! 🚀
