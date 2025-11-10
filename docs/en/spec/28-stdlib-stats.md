# Standard Library: Statistical Functions

**Statistical Computation & Data Analysis**

The `stats/` module provides functions for statistical calculations. It enables easy computation of basic statistical measures such as mean, median, variance, and standard deviation.

This module is compiled with the `std-stats` feature.

---

## Overview

The statistics module provides the following functions:

- **Measures of Central Tendency**: mean, median, mode
- **Measures of Dispersion**: variance, stddev (standard deviation)
- **Measures of Position**: percentile

All functions accept lists or vectors as arguments and support numeric data (integers and floating-point numbers).

---

## Measures of Central Tendency

### stats/mean - Arithmetic Mean

Calculates the arithmetic mean (average) of the data.

```qi
;; Basic usage
(stats/mean [1 2 3 4 5])              ;; => 3.0

;; Mixed integers and floats
(stats/mean [1 2.5 3])                ;; => 2.166666...

;; Works with lists too
(stats/mean '(10 20 30))              ;; => 20.0

;; Pipeline usage
([1 2 3 4 5] |> stats/mean)           ;; => 3.0
```

**Arguments**:
- Collection (list or vector): Collection containing only numbers

**Returns**:
- Float: Arithmetic mean

**Errors**:
- Empty collection
- Non-numeric elements in collection

---

### stats/median - Median

Returns the middle value when the data is sorted in ascending order. For an even number of elements, returns the average of the two middle values.

```qi
;; Odd number of elements
(stats/median [1 2 3 4 5])            ;; => 3.0

;; Even number of elements (average of middle two)
(stats/median [1 2 3 4])              ;; => 2.5

;; Unsorted data (automatically sorted)
(stats/median [5 1 3 2 4])            ;; => 3.0

;; Floating-point numbers
(stats/median [1.5 2.0 2.5 3.0])      ;; => 2.25
```

**Arguments**:
- Collection (list or vector): Collection containing only numbers

**Returns**:
- Float: Median value

**Errors**:
- Empty collection
- Non-numeric elements in collection

---

### stats/mode - Mode

Returns the most frequently occurring value.

```qi
;; Basic usage
(stats/mode [1 2 2 3 3 3])            ;; => 3

;; Integers
(stats/mode [1 1 1 2 2 3])            ;; => 1

;; Floating-point numbers
(stats/mode [1.5 1.5 2.0 2.0 2.0])    ;; => 2.0

;; Pipeline usage
([1 2 2 3 3 3] |> stats/mode)         ;; => 3
```

**Arguments**:
- Collection (list or vector): Collection containing only numbers

**Returns**:
- Number: Most frequent value (preserves original data type)

**Errors**:
- Empty collection
- Non-numeric elements in collection

**Note**:
- If multiple values have the same maximum frequency, one of them is returned (which one is undefined)

---

## Measures of Dispersion

### stats/variance - Variance

Calculates the variance (population variance) of the data. Variance is a measure of data spread.

```qi
;; Basic usage
(stats/variance [1 2 3 4 5])          ;; => 2.0

;; Floating-point numbers
(stats/variance [1.0 2.0 3.0])        ;; => 0.666666...

;; Pipeline usage
([1 2 3 4 5] |> stats/variance)       ;; => 2.0
```

**Formula**:
```
variance = Σ(xi - mean)² / n
```

**Arguments**:
- Collection (list or vector): Collection containing only numbers

**Returns**:
- Float: Variance

**Errors**:
- Empty collection
- Non-numeric elements in collection

---

### stats/stddev - Standard Deviation

Calculates the standard deviation of the data. Standard deviation is the square root of variance and represents data spread in the original units.

```qi
;; Basic usage
(stats/stddev [1 2 3 4 5])            ;; => 1.414213... (√2)

;; Floating-point numbers
(stats/stddev [2 4 6 8])              ;; => 2.236067... (√5)

;; Pipeline usage
([1 2 3 4 5] |> stats/stddev)         ;; => 1.414213...
```

**Formula**:
```
stddev = √variance
```

**Arguments**:
- Collection (list or vector): Collection containing only numbers

**Returns**:
- Float: Standard deviation

**Errors**:
- Empty collection
- Non-numeric elements in collection

---

## Measures of Position

### stats/percentile - Percentile

Calculates the value at the specified percentile position using linear interpolation.

```qi
;; 50th percentile (same as median)
(stats/percentile [1 2 3 4 5] 50)     ;; => 3.0

;; 95th percentile
(stats/percentile [1 2 3 4 5] 95)     ;; => 4.8

;; 25th percentile (first quartile)
(stats/percentile [1 2 3 4 5] 25)     ;; => 2.0

;; 75th percentile (third quartile)
(stats/percentile [1 2 3 4 5] 75)     ;; => 4.0

;; Float percentile value
(stats/percentile [1 2 3 4 5] 50.5)   ;; => 3.02

;; Pipeline usage
([1 2 3 4 5] |> (stats/percentile _ 95))  ;; => 4.8
```

**Arguments**:
- Collection (list or vector): Collection containing only numbers
- Percentile value (integer or float): Range 0-100

**Returns**:
- Float: Value at the specified percentile position

**Errors**:
- Empty collection
- Non-numeric elements in collection
- Percentile value outside 0-100 range

**Note**:
- Uses linear interpolation, so values not present in the actual data may be returned

---

## Pipeline Usage

Statistical functions can be naturally combined with Qi's pipeline operators.

```qi
;; Data processing pipeline
(def data [10 20 30 40 50])

(data
 |> stats/mean
 |> (math/round _))
;; => 30

;; Calculate multiple statistics
(defn summary-stats [data]
  {:mean (stats/mean data)
   :median (stats/median data)
   :stddev (stats/stddev data)
   :min (apply min data)
   :max (apply max data)})

(summary-stats [1 2 3 4 5])
;; => {:mean 3.0, :median 3.0, :stddev 1.414..., :min 1, :max 5}

;; Filter and compute statistics
([1 2 3 4 5 6 7 8 9 10]
 |> (filter (fn [x] (> x 5)))
 |> stats/mean)
;; => 8.0
```

---

## Practical Examples

### Data Analysis Pipeline

```qi
;; Analyze test scores
(def test-scores [85 90 78 92 88 76 95 89 84 91])

(defn analyze-scores [scores]
  (let [sorted (sort scores)
        n (len scores)]
    {:count n
     :mean (stats/mean scores)
     :median (stats/median scores)
     :stddev (stats/stddev scores)
     :min (first sorted)
     :max (last sorted)
     :q1 (stats/percentile scores 25)
     :q3 (stats/percentile scores 75)
     :p95 (stats/percentile scores 95)}))

(analyze-scores test-scores)
;; => {:count 10
;;     :mean 86.8
;;     :median 88.5
;;     :stddev 5.68...
;;     :min 76
;;     :max 95
;;     :q1 83.25
;;     :q3 91.25
;;     :p95 94.2}
```

### Outlier Detection

```qi
;; Outlier detection using standard deviation (3-sigma method)
(defn detect-outliers [data threshold]
  (let [m (stats/mean data)
        sd (stats/stddev data)
        lower (- m (* threshold sd))
        upper (+ m (* threshold sd))]
    (filter (fn [x] (or (< x lower) (> x upper))) data)))

(def data [10 12 11 13 100 12 11 10 12])
(detect-outliers data 3)
;; => [100]  (100 is an outlier)
```

### Normalization (z-score)

```qi
;; Transform data to z-scores (mean=0, stddev=1)
(defn z-score [data]
  (let [m (stats/mean data)
        sd (stats/stddev data)]
    (map (fn [x] (/ (- x m) sd)) data)))

(z-score [1 2 3 4 5])
;; => [-1.414... -0.707... 0.0 0.707... 1.414...]
```

### Percentile Ranking

```qi
;; Calculate percentile rank of a score
(defn percentile-rank [data value]
  (let [sorted (sort data)
        n (len sorted)
        below (len (filter (fn [x] (< x value)) sorted))
        equal (len (filter (fn [x] (= x value)) sorted))]
    (* 100.0 (/ (+ below (/ equal 2.0)) n))))

(def scores [70 80 85 90 95])
(percentile-rank scores 85)
;; => 50.0  (85 is at the 50th percentile)
```

### Group Statistics

```qi
;; Calculate statistics by category
(def data
  [{:category "A" :value 10}
   {:category "A" :value 20}
   {:category "B" :value 30}
   {:category "B" :value 40}
   {:category "B" :value 50}])

(defn group-stats [data key-fn value-fn]
  (let [grouped (group-by key-fn data)]
    (map-vals
      (fn [items]
        (let [values (map value-fn items)]
          {:mean (stats/mean values)
           :median (stats/median values)
           :count (len values)}))
      grouped)))

(group-stats data
  (fn [x] (get x :category))
  (fn [x] (get x :value)))
;; => {"A" {:mean 15.0, :median 15.0, :count 2}
;;     "B" {:mean 40.0, :median 40.0, :count 3}}
```

### Moving Average

```qi
;; Simple moving average
(defn moving-average [data window-size]
  (let [windows (partition window-size 1 data)]
    (map stats/mean windows)))

(moving-average [1 2 3 4 5 6 7 8 9 10] 3)
;; => [2.0 3.0 4.0 5.0 6.0 7.0 8.0 9.0]
```

### Covariance Calculation

```qi
;; Calculate covariance between two datasets
(defn covariance [x-data y-data]
  (let [n (len x-data)
        x-mean (stats/mean x-data)
        y-mean (stats/mean y-data)
        products (map (fn [x y] (* (- x x-mean) (- y y-mean)))
                      x-data y-data)]
    (/ (reduce + 0 products) n)))

(def x [1 2 3 4 5])
(def y [2 4 6 8 10])
(covariance x y)
;; => 4.0
```

---

## Error Handling

Statistical functions return errors in the following cases:

```qi
;; Empty collection
(stats/mean [])
;; => Error: stats/mean: collection must not be empty

;; Non-numeric elements
(stats/mean [1 2 "three" 4])
;; => Error: stats/mean: all elements must be numbers

;; Invalid percentile value
(stats/percentile [1 2 3] 150)
;; => Error: stats/percentile: invalid percentile (must be 0-100)

;; Error handling with try/ok
(try
  (stats/mean [])
  (fn [result]
    (match result
      [:ok value] (println f"Mean: {value}")
      [:error msg] (println f"Error: {msg}"))))
```

---

## Function Reference

| Function | Description | Arguments | Returns |
|----------|-------------|-----------|---------|
| `stats/mean` | Arithmetic mean | Collection | Float |
| `stats/median` | Median | Collection | Float |
| `stats/mode` | Mode | Collection | Number |
| `stats/variance` | Variance | Collection | Float |
| `stats/stddev` | Standard deviation | Collection | Float |
| `stats/percentile` | Percentile | Collection, Percentile(0-100) | Float |

---

## Integration with Math Functions

Statistical functions can be combined with the `math/` module:

```qi
;; Coefficient of Variation (CV)
(defn cv [data]
  (let [m (stats/mean data)
        sd (stats/stddev data)]
    (* 100.0 (/ sd m))))

(cv [10 12 11 13 12])
;; => 10.5...  (CV: 10.5%)

;; Range of standardized data
(defn standardized-range [data]
  (let [z-scores (z-score data)]
    {:min (apply min z-scores)
     :max (apply max z-scores)
     :range (- (apply max z-scores) (apply min z-scores))}))
```

---

## Performance Considerations

- **Sorting**: `median` and `percentile` sort the data, resulting in O(n log n) time complexity
- **Multiple Calculations**: When computing multiple statistics on the same data, it's more efficient to calculate them together
- **Large Datasets**: For very large datasets (1 million+ elements), be mindful of memory usage

```qi
;; Efficient multiple statistics calculation
(defn efficient-stats [data]
  ;; Calculate mean once and reuse for variance and stddev
  (let [m (stats/mean data)
        v (stats/variance data)
        sd (math/sqrt v)]  ;; Calculate from variance
    {:mean m
     :variance v
     :stddev sd}))
```

---

## Future Extensions

The following features may be added in future versions:

- Sample variance (unbiased variance) support
- Skewness and kurtosis
- Correlation coefficient
- Basic regression analysis functions
- Histogram generation
- Box plot data calculation

---

## See Also

- [Math Functions](15-stdlib-math.md) - Numeric operations and random number generation
- [Collection Operations](05-syntax-basics.md) - filter, map, reduce
- [Error Handling](08-error-handling.md) - Error handling with try/ok and match
