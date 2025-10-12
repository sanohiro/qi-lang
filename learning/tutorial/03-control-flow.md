# 制御構造

条件分岐、パターンマッチ、ループについて学びます。

## if - 条件分岐

### 基本的なif

```lisp
(if condition
  then-expr
  else-expr)

; 例
(if (> x 10)
  "big"
  "small")
```

### elseの省略

```lisp
(if (> x 0)
  (println "Positive"))
; => nil（条件が偽の場合）
```

### ネストしたif

```lisp
(if (> score 90)
  "A"
  (if (> score 80)
    "B"
    (if (> score 70)
      "C"
      "F")))
```

### 実践例

```lisp
(defn abs [x]
  (if (< x 0)
    (- x)
    x))

(abs -5)             ; => 5
(abs 3)              ; => 3
```

## match - パターンマッチ

強力なパターンマッチングです。

### 基本的なmatch

```lisp
(match value
  pattern1 -> result1
  pattern2 -> result2
  _ -> default)

; 例
(match x
  1 -> "one"
  2 -> "two"
  3 -> "three"
  _ -> "other")
```

### リテラルパターン

```lisp
(match status
  :ok -> "Success"
  :error -> "Failed"
  :pending -> "Waiting"
  _ -> "Unknown")
```

### ワイルドカード

```lisp
(match x
  0 -> "zero"
  _ -> "non-zero")  ; すべてにマッチ
```

### Orパターン

```lisp
(match x
  1 | 2 | 3 -> "small"
  4 | 5 | 6 -> "medium"
  _ -> "large")
```

### ベクタパターン

```lisp
(match vec
  [] -> "empty"
  [x] -> f"single: {x}"
  [x y] -> f"pair: {x}, {y}"
  [x y ...rest] -> f"many: first={x}, second={y}, rest={rest}")

(match [1 2 3 4 5]
  [x y ...rest] -> rest)
; => [3 4 5]
```

### マップパターン

```lisp
(match person
  {:name n :age a} -> f"{n} is {a} years old"
  _ -> "Unknown person")

; 例
(def alice {:name "Alice" :age 25})
(match alice
  {:name n :age a} -> f"{n} is {a} years old")
; => "Alice is 25 years old"
```

### 変換パターン

```lisp
; キーの値を変換
(match {:x 10}
  {:x x => (* x 2)} -> x)
; => 20
```

### ガード条件

```lisp
(match x
  n when (> n 0) -> "positive"
  n when (< n 0) -> "negative"
  _ -> "zero")

; 例
(match 5
  n when (> n 10) -> "big"
  n when (> n 0) -> "small"
  _ -> "zero or negative")
; => "small"
```

### :asパターン

```lisp
; パターン全体を変数に束縛
(match {:name "Alice" :age 25}
  {:name n :age a} :as person -> person)
; => {:name "Alice" :age 25}
```

### 実践例

```lisp
; HTTPレスポンスの処理
(defn handle-response [response]
  (match response
    {:status 200 :body b} -> (println f"Success: {b}")
    {:status 404} -> (println "Not found")
    {:status s} when (>= s 500) -> (println "Server error")
    _ -> (println "Unknown response")))

; 再帰的なリスト処理
(defn sum [lst]
  (match lst
    [] -> 0
    [x ...rest] -> (+ x (sum rest))))

(sum [1 2 3 4 5])    ; => 15
```

## loop / recur - 末尾再帰

効率的なループを実現します。

### 基本的なloop

```lisp
(loop [bindings]
  body
  (recur new-values))

; 例
(loop [i 0
       sum 0]
  (if (>= i 10)
    sum
    (recur (+ i 1) (+ sum i))))
; => 45（0+1+2+...+9）
```

### 階乗

```lisp
(defn factorial [n]
  (loop [i n
         acc 1]
    (if (<= i 1)
      acc
      (recur (- i 1) (* acc i)))))

(factorial 5)        ; => 120
```

### FizzBuzz

```lisp
(loop [n 1]
  (if (> n 100)
    nil
    (do
      (println
        (if (= (% n 15) 0)
          "FizzBuzz"
          (if (= (% n 3) 0)
            "Fizz"
            (if (= (% n 5) 0)
              "Buzz"
              n))))
      (recur (+ n 1)))))
```

### フィボナッチ数列

```lisp
(defn fib [n]
  (loop [a 0
         b 1
         i 0]
    (if (= i n)
      a
      (recur b (+ a b) (+ i 1)))))

(fib 10)             ; => 55
```

## do - 複数の式

複数の式を順番に実行します。

```lisp
(do
  (println "First")
  (println "Second")
  (+ 1 2))           ; 最後の式が返り値
```

実用例：

```lisp
(defn process-data [data]
  (do
    (println "Processing data...")
    (def cleaned (clean-data data))
    (println "Data cleaned")
    (def transformed (transform cleaned))
    (println "Data transformed")
    transformed))
```

## try - エラーハンドリング

エラーを捕捉します。

```lisp
(try expr)
; => {:ok result} または {:error message}

; 例
(try (/ 1 0))
; => {:error "Division by zero"}

(try (+ 1 2))
; => {:ok 3}
```

### matchと組み合わせ

```lisp
(match (try (/ 10 x))
  {:ok result} -> (println f"Result: {result}")
  {:error msg} -> (println f"Error: {msg}"))
```

### 実践例

```lisp
(defn safe-divide [a b]
  (match (try (/ a b))
    {:ok result} -> result
    {:error _} -> nil))

(safe-divide 10 2)   ; => 5
(safe-divide 10 0)   ; => nil
```

## defer - 遅延実行

スコープを抜ける時に実行されます（LIFO順）。

```lisp
(do
  (defer (println "cleanup"))
  (defer (println "close"))
  (println "main")
  42)

; 出力順:
; main
; close
; cleanup
; => 42
```

### ファイル処理

```lisp
(defn process-file [path]
  (do
    (def file (io/open path))
    (defer (io/close file))  ; 必ず実行される
    (process-content (io/read file))))
```

## and / or - 論理演算

短絡評価されます。

### and

```lisp
(and true true)      ; => true
(and true false)     ; => false
(and false (println "Not executed"))  ; => false
```

### or

```lisp
(or true false)      ; => true
(or false false)     ; => false
(or true (println "Not executed"))    ; => true
```

### 実用例

```lisp
; デフォルト値
(def name (or user-input "Default Name"))

; 複数条件
(if (and (> age 18) (< age 65))
  "Working age"
  "Other")
```

## 実践例

### 例1: グレード計算

```lisp
(defn calculate-grade [score]
  (match score
    s when (>= s 90) -> "A"
    s when (>= s 80) -> "B"
    s when (>= s 70) -> "C"
    s when (>= s 60) -> "D"
    _ -> "F"))

(calculate-grade 85)  ; => "B"
```

### 例2: リストの最大値

```lisp
(defn max-in-list [lst]
  (match lst
    [] -> nil
    [x] -> x
    [x ...rest] ->
      (def max-rest (max-in-list rest))
      (if (> x max-rest) x max-rest)))

(max-in-list [3 7 2 9 4])  ; => 9
```

### 例3: 状態機械

```lisp
(defn state-machine [state event]
  (match [state event]
    [:idle :start] -> :running
    [:running :pause] -> :paused
    [:paused :resume] -> :running
    [:running :stop] -> :idle
    [s _] -> s))  ; 無効な遷移は現在の状態を維持

(state-machine :idle :start)      ; => :running
(state-machine :running :pause)   ; => :paused
```

### 例4: JSONレスポンスの処理

```lisp
(defn handle-api-response [response]
  (match (try (json/parse response))
    {:ok {:status "success" :data data}} ->
      (process-data data)

    {:ok {:status "error" :message msg}} ->
      (println f"API error: {msg}")

    {:error err} ->
      (println f"Parse error: {err}")))
```

### 例5: カウントダウン

```lisp
(defn countdown [n]
  (loop [i n]
    (if (<= i 0)
      (println "Done!")
      (do
        (println i)
        (recur (- i 1))))))

(countdown 5)
; 出力:
; 5
; 4
; 3
; 2
; 1
; Done!
```

## まとめ

制御構造：

- **if**: 条件分岐（then/else）
- **match**: パターンマッチ
  - リテラル、ベクタ、マップ
  - Orパターン、ガード、:as
- **loop/recur**: 効率的なループ
- **do**: 複数の式を順番に実行
- **try**: エラーハンドリング
- **defer**: 遅延実行
- **and/or**: 短絡評価

matchが最も強力で柔軟！

## 次のステップ

次は[関数型プログラミング](./04-functional.md)を学びます。map、filter、reduceなどの高階関数を理解しましょう。
