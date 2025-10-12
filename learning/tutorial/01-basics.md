# 基本文法

Qi言語の基本的な構文とデータ型について学びます。

## 式（Expression）

Qiでは**すべてが式**です。式は値を返します。

```lisp
42              ; => 42
"hello"         ; => "hello"
(+ 1 2)         ; => 3
(println "hi")  ; => nil（副作用として出力）
```

## リテラル（基本的な値）

### 数値

```lisp
; 整数
42
-10
0

; 浮動小数点数
3.14
-2.5
0.0
```

### 文字列

```lisp
"hello"
"こんにちは"
"multi\nline"    ; エスケープシーケンス

; 複数行文字列
"""
This is a
multi-line
string
"""

; 文字列補間（f-string）
(def name "Alice")
f"Hello, {name}!"  ; => "Hello, Alice!"
```

### ブール値と nil

```lisp
true            ; 真
false           ; 偽
nil             ; 値がないことを表す
```

**truthy / falsy:**
- `false`と`nil`が**falsy**
- それ以外はすべて**truthy**（`0`や`""`もtruthy）

## 変数定義

### def - 変数を定義

```lisp
(def x 42)
(def name "Alice")
(def pi 3.14159)

; 使用
(println x)      ; => 42
(println name)   ; => Alice
```

### 再代入はできない

```lisp
(def x 10)
(def x 20)       ; 警告: 再定義
; x は 20 になるが、再代入ではない（新しい束縛）
```

## 関数定義

### defn - 名前付き関数

```lisp
; 基本的な関数
(defn add [a b]
  (+ a b))

(add 3 4)        ; => 7

; ドキュメント付き
(defn greet "Greet a person" [name]
  (str "Hello, " name "!"))

(greet "Alice")  ; => "Hello, Alice!"
```

### 複数の式

関数の本体は複数の式を書けます。最後の式が返り値になります：

```lisp
(defn calculate [x]
  (println "Calculating...")
  (def result (* x x))
  (println f"Result: {result}")
  result)       ; これが返り値

(calculate 5)
; 出力:
; Calculating...
; Result: 25
; => 25
```

### 可変長引数

```lisp
(defn sum [& numbers]
  (reduce + 0 numbers))

(sum 1 2 3 4 5)  ; => 15
```

## 関数呼び出し

### 基本的な呼び出し

```lisp
(function-name arg1 arg2 arg3)

(+ 1 2 3)
(str "Hello" " " "World")
(println "message")
```

### ネストした呼び出し

```lisp
(+ 1 (* 2 3))    ; => 7

; 読みにくい例
(filter even? (map inc (range 10)))

; パイプラインで書く（後述）
(range 10)
|> (map inc)
|> (filter even?)
```

## ローカル変数

### let - ローカルスコープ

```lisp
(let [x 10
      y 20]
  (+ x y))       ; => 30

; x と y はここでは使えない
```

### スコープのネスト

```lisp
(def x 10)       ; グローバル

(let [x 20]      ; ローカル
  (println x)    ; => 20
  (let [x 30]    ; さらにローカル
    (println x)) ; => 30
  (println x))   ; => 20

(println x)      ; => 10
```

## 無名関数（ラムダ）

### fn - 無名関数

```lisp
; 基本
(fn [x] (* x 2))

; 使用
(def double (fn [x] (* x 2)))
(double 5)       ; => 10

; その場で呼び出す
((fn [x] (* x 2)) 5)  ; => 10

; 高階関数に渡す
(map (fn [x] (* x 2)) [1 2 3])
; => [2 4 6]
```

### クロージャ

```lisp
(defn make-adder [n]
  (fn [x] (+ n x)))

(def add5 (make-adder 5))
(add5 10)        ; => 15
(add5 20)        ; => 25
```

## 算術演算

```lisp
; 加算
(+ 1 2 3)        ; => 6

; 減算
(- 10 3)         ; => 7
(- 5)            ; => -5（単項マイナス）

; 乗算
(* 2 3 4)        ; => 24

; 除算
(/ 10 2)         ; => 5
(/ 10 3)         ; => 3（整数除算）
(/ 10.0 3)       ; => 3.333...（浮動小数点）

; 剰余
(% 10 3)         ; => 1

; その他
(inc 5)          ; => 6（インクリメント）
(dec 5)          ; => 4（デクリメント）
(abs -5)         ; => 5（絶対値）
```

## 比較演算

```lisp
; 等価
(= 1 1)          ; => true
(= 1 2)          ; => false
(!= 1 2)         ; => true

; 大小比較
(> 5 3)          ; => true
(< 3 5)          ; => true
(>= 5 5)         ; => true
(<= 3 5)         ; => true

; 複数要素
(= 1 1 1)        ; => true
(= 1 2 1)        ; => false
```

## 論理演算

```lisp
; not
(not true)       ; => false
(not false)      ; => true
(not nil)        ; => true
(not 0)          ; => false（0 はtruthy）

; and（すべてが真）
(and true true)  ; => true
(and true false) ; => false

; or（いずれかが真）
(or true false)  ; => true
(or false false) ; => false

; 短絡評価
(and false (println "not executed"))  ; => false
(or true (println "not executed"))    ; => true
```

## 文字列操作

```lisp
; 連結
(str "Hello" " " "World")  ; => "Hello World"

; 大文字・小文字
(str/upper "hello")        ; => "HELLO"
(str/lower "HELLO")        ; => "hello"

; トリム
(str/trim "  hello  ")     ; => "hello"

; 分割
(str/split "a,b,c" ",")    ; => ["a" "b" "c"]

; 結合
(str/join ", " ["a" "b" "c"])  ; => "a, b, c"

; 部分文字列
(str/slice "hello" 0 2)    ; => "he"

; 置換
(str/replace "hello" "l" "L")  ; => "heLLo"

; 検索
(str/contains? "hello" "ell")  ; => true
(str/starts-with? "hello" "he") ; => true
(str/ends-with? "hello" "lo")   ; => true
```

## 型チェック

```lisp
; 基本的な型
(nil? nil)       ; => true
(integer? 42)    ; => true
(float? 3.14)    ; => true
(string? "hi")   ; => true

; コレクション
(list? '(1 2 3)) ; => true
(vector? [1 2 3]) ; => true
(map? {:a 1})    ; => true

; 関数
(function? add)  ; => true

; 数値チェック
(number? 42)     ; => true
(number? 3.14)   ; => true
(even? 4)        ; => true
(odd? 3)         ; => true
(positive? 5)    ; => true
(negative? -3)   ; => true
(zero? 0)        ; => true
```

## do ブロック

複数の式を順番に実行します：

```lisp
(do
  (println "First")
  (println "Second")
  (+ 1 2))       ; => 3（最後の式が返り値）
```

実際の使用例：

```lisp
(def result
  (do
    (def x 10)
    (def y 20)
    (+ x y)))    ; => 30
```

## コメント

```lisp
; これは行コメント

(+ 1 2)  ; 式の後ろにもコメント

(def x 10)  ; 変数の説明
```

## 実践例

### 例1: FizzBuzz（一部）

```lisp
(defn fizzbuzz [n]
  (if (= (% n 15) 0)
    "FizzBuzz"
    (if (= (% n 3) 0)
      "Fizz"
      (if (= (% n 5) 0)
        "Buzz"
        (str n)))))

(fizzbuzz 15)    ; => "FizzBuzz"
(fizzbuzz 9)     ; => "Fizz"
(fizzbuzz 10)    ; => "Buzz"
(fizzbuzz 7)     ; => "7"
```

### 例2: 階乗

```lisp
(defn factorial [n]
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(factorial 5)    ; => 120
```

### 例3: 挨拶関数

```lisp
(defn greet
  "Personalized greeting"
  [name age]
  (do
    (println f"Hello, {name}!")
    (println f"You are {age} years old.")
    (if (>= age 20)
      (println "You are an adult.")
      (println "You are not an adult yet."))
    nil))

(greet "Alice" 25)
; 出力:
; Hello, Alice!
; You are 25 years old.
; You are an adult.
; => nil
```

## REPL での実験

REPLで試してみましょう：

```lisp
$ qi
qi> (def x 10)
10
qi> (def y 20)
20
qi> (+ x y)
30
qi> (defn double [n] (* n 2))
#<function>
qi> (double 21)
42
qi> (map double [1 2 3 4 5])
[2 4 6 8 10]
```

## まとめ

Qiの基本文法を学びました：

- **式**: すべてが値を返す
- **変数**: `def`で定義
- **関数**: `defn`で定義、`fn`で無名関数
- **演算**: 算術、比較、論理
- **文字列**: 連結、操作、補間
- **スコープ**: `let`でローカル変数

## 次のステップ

次は[リストとコレクション](./02-collections.md)を学びます。データ構造の使い方を理解しましょう。
