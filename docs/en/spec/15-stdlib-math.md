# Standard Library: Math Functions

**Numeric Operations and Mathematical Functions**

---

## Basic Operations

Qi provides basic arithmetic operators (`+`, `-`, `*`, `/`, `%`).
All operators support both integers and floats, and return a float if at least one operand is a float.

```qi
(+ 1 2 3)        ;; => 6
(+ 1.5 2 3)      ;; => 6.5 (contains float)
(- 10 3)         ;; => 7
(* 2 3 4)        ;; => 24
(/ 10 2)         ;; => 5
(% 10 3)         ;; => 1 (remainder)
(% 10.5 3)       ;; => 1.5 (supports floats)
```

### Convenient Numeric Functions

```qi
;; min - Minimum value
(min 1 2 3)          ;; => 1
(min 1.5 2 3)        ;; => 1.5 (supports floats)

;; max - Maximum value
(max 1 2 3)          ;; => 3
(max 1.5 2 3)        ;; => 3 (supports floats)

;; sum - Sum (total of collection elements)
(sum [1 2 3])        ;; => 6
(sum [1.5 2 3])      ;; => 6.5 (supports floats)

;; inc - Increment (add 1)
(inc 5)              ;; => 6
(inc 5.5)            ;; => 6.5 (supports floats)

;; dec - Decrement (subtract 1)
(dec 5)              ;; => 4
(dec 5.5)            ;; => 4.5 (supports floats)

;; abs - Absolute value
(abs -5)             ;; => 5
(abs -5.5)           ;; => 5.5
```

---

## Mathematical Functions

### Power and Square Root

```qi
;; math/pow - Power
(math/pow 2 3)      ;; => 8 (2^3)
(math/pow 10 2)     ;; => 100
(math/pow 2 -1)     ;; => 0.5

;; math/sqrt - Square root
(math/sqrt 4)       ;; => 2.0
(math/sqrt 9)       ;; => 3.0
(math/sqrt 2)       ;; => 1.4142135623730951
```

### Rounding

```qi
;; math/round - Round to nearest integer
(math/round 3.4)    ;; => 3
(math/round 3.5)    ;; => 4
(math/round -3.5)   ;; => -4

;; math/floor - Round down (toward negative infinity)
(math/floor 3.9)    ;; => 3
(math/floor -3.1)   ;; => -4

;; math/ceil - Round up (toward positive infinity)
(math/ceil 3.1)     ;; => 4
(math/ceil -3.9)    ;; => -3
```

### Range Clamping

```qi
;; math/clamp - Clamp value to range
(math/clamp 5 0 10)     ;; => 5 (within range)
(math/clamp -5 0 10)    ;; => 0 (minimum value)
(math/clamp 15 0 10)    ;; => 10 (maximum value)
(math/clamp -7 -10 -5)  ;; => -7 (negative range)
```

---

## Random Number Generation (Feature: std-math)

Random number functions are available when the `std-math` feature is enabled.

```qi
;; math/rand - Random number between 0.0 and 1.0
(math/rand)              ;; => 0.7234...

;; math/rand-int - Random integer (0 to n-1)
(math/rand-int 10)       ;; => 0-9 (integer)

;; math/random-range - Random integer in range
(math/random-range 10 20)  ;; => 10-20 (integer)

;; math/shuffle - Shuffle list
(math/shuffle [1 2 3 4 5])  ;; => [3 1 5 2 4] (random)
```

---

## Pipeline Usage

Math functions can be combined with pipeline operators.

```qi
;; Range clamping and rounding
(16.5 |> math/sqrt |> math/round)
;; => 4

;; Sequential calculations
(2 |> (math/pow _ 3) |> (math/clamp _ 0 10))
;; => 8

;; Data processing
(map (fn [x] (x |> math/sqrt |> math/round)) [4 9 16 25])
;; => (2 3 4 5)
```

---

## Usage Examples

### Statistical Calculations

```qi
;; Root Mean Square (RMS)
(defn rms [numbers]
  (let [squares (map (fn [x] (math/pow x 2)) numbers)
        sum (reduce + 0 squares)
        mean (/ sum (len numbers))]
    (math/sqrt mean)))

(rms [1 2 3 4 5])  ;; => 3.316...
```

### Range Clamping

```qi
;; Clamp score to 0-100
(defn normalize-score [score]
  (math/clamp (math/round score) 0 100))

(map normalize-score [-10 45.7 99.2 150])
;; => (0 46 99 100)
```

### Random Sampling

```qi
;; Get random samples
(defn random-sample [n coll]
  (take n (math/shuffle coll)))

(random-sample 3 [1 2 3 4 5 6 7 8 9 10])
;; => (7 2 9) etc.
```

---

## Function Reference

### Basic Operations and Numeric Functions

| Function | Description | Example |
|----------|-------------|---------|
| `+`, `-`, `*`, `/`, `%` | Basic operations (int/float support) | `(+ 1.5 2 3)` → `6.5` |
| `min` | Minimum value (int/float support) | `(min 1.5 2 3)` → `1.5` |
| `max` | Maximum value (int/float support) | `(max 1.5 2 3)` → `3` |
| `sum` | Sum (int/float support) | `(sum [1.5 2 3])` → `6.5` |
| `inc` | Increment (int/float support) | `(inc 5.5)` → `6.5` |
| `dec` | Decrement (int/float support) | `(dec 5.5)` → `4.5` |
| `abs` | Absolute value | `(abs -5.5)` → `5.5` |

### Mathematical Functions

| Function | Description | Example |
|----------|-------------|---------|
| `math/pow` | Power | `(math/pow 2 3)` → `8` |
| `math/sqrt` | Square root | `(math/sqrt 16)` → `4.0` |
| `math/round` | Round to nearest integer | `(math/round 3.5)` → `4` |
| `math/floor` | Round down | `(math/floor 3.9)` → `3` |
| `math/ceil` | Round up | `(math/ceil 3.1)` → `4` |
| `math/clamp` | Clamp to range | `(math/clamp 15 0 10)` → `10` |

### Random Number Functions (Feature: std-math)

| Function | Description | Example |
|----------|-------------|---------|
| `math/rand` | Random (0.0-1.0) | `(math/rand)` → `0.7234...` |
| `math/rand-int` | Random integer | `(math/rand-int 10)` → `0-9` |
| `math/random-range` | Random in range | `(math/random-range 10 20)` → `10-20` |
| `math/shuffle` | Shuffle | `(math/shuffle [1 2 3])` → `[3 1 2]` |

---

## Notes

- **Integer and Float**: `math/pow` may return an integer. `math/sqrt` always returns a float.
- **Feature Gates**: Random functions (`rand`, `rand-int`, `random-range`, `shuffle`) require the `std-math` feature.
- **Range Clamping**: `math/clamp` argument order is `(value, min, max)`.
