# 基本構文

**Qiの基本的な構文要素**

---

## コメント

```qi
; 1行コメント
;; 一般的には ;; を使うことが多い

(def x 42)  ; 行末コメント
```

---

## データ型

Qiは動的型付け言語で、以下の基本型を持ちます：

- **数値**: 整数、浮動小数点数
- **文字列**: UTF-8対応、f-string対応
- **真偽値**: `true`, `false`
- **nil**: 値の不在を表す
- **ベクター**: `[1 2 3]`
- **マップ**: `{:key "value"}`
- **リスト**: `'(1 2 3)` (クオート必須)
- **関数**: 第一級オブジェクト
- **キーワード**: `:keyword`

---

## リテラル

### 数値

```qi
42          ;; 整数
3.14        ;; 浮動小数点数
-10         ;; 負の数
1_000_000   ;; アンダースコア区切り（可読性向上）
```

### 文字列

```qi
;; 基本
"hello"
"hello\nworld"      ;; エスケープシーケンス
"say \"hello\""     ;; クオートのエスケープ

;; 複数行文字列（Python風）
"""
This is a
multi-line
string
"""

;; 複数行でSQLやHTMLを記述
(def query """
  SELECT name, age
  FROM users
  WHERE age >= 18
  ORDER BY name
""")
```

### f-string（文字列補間）

```qi
;; 基本的な使い方
f"Hello, World!"  ;; => "Hello, World!"

;; 変数の補間
(def name "Alice")
f"Hello, {name}!"  ;; => "Hello, Alice!"

;; 式も使える
f"Result: {(+ 1 2)}"  ;; => "Result: 3"

;; リストやベクタの補間
f"List: {[1 2 3]}"  ;; => "List: [1 2 3]"

;; マップアクセス
(def user {:name "Bob" :age 30})
f"Name: {(:name user)}, Age: {(:age user)}"
;; => "Name: Bob, Age: 30"

;; エスケープ
f"Escaped: \{not interpolated\}"  ;; => "Escaped: {not interpolated}"

;; 複数行f-string
(def name "Alice")
(def age 30)

f"""
Name: {name}
Age: {age}
Status: Active
"""
```

### bool と nil

```qi
true
false
nil

;; 条件式での扱い
(if nil "yes" "no")     ;; "no" (nilはfalsy)
(if false "yes" "no")   ;; "no" (falseはfalsy)
(if 0 "yes" "no")       ;; "yes" (0はtruthy)
(if "" "yes" "no")      ;; "yes" (空文字もtruthy)

;; 明示的な比較
(= x nil)               ;; nilチェック
(= x false)             ;; falseチェック
```

### キーワード

```qi
:keyword
:name
:type

;; マップのキーとして使用
{:name "Alice" :age 30}

;; 関数として使える（マップから値を取得）
(def user {:name "Bob" :age 25})
(:name user)  ;; => "Bob"
```

### ベクター

```qi
[]              ;; 空のベクター
[1 2 3]         ;; 数値のベクター
["a" "b" "c"]   ;; 文字列のベクター
[1 "hello" :key]  ;; 混在も可能
```

### マップ

```qi
{}                          ;; 空のマップ
{:name "Alice" :age 30}     ;; キーワードをキーにする
{"name" "Bob" "age" 25}     ;; 文字列をキーにする
```

### リスト

```qi
'()             ;; 空のリスト
'(1 2 3)        ;; クオート必須
```

---

## 特殊形式（9つ）

### `def` - グローバル定義

```qi
(def x 42)
(def greet (fn [name] (str "Hello, " name)))
(def ops [+ - * /])
```

### `defn` - 関数定義（糖衣構文）

```qi
;; 基本形式
(defn greet [name]
  (str "Hello, " name))

;; 可変長引数
(defn sum [& nums]
  (reduce + 0 nums))

;; ベクタの分解（Destructuring）
(defn add-pair [[x y]]
  (+ x y))

(defn format-kv [[k v]]
  f"{k}={v}")

;; ...rest構文（ベクタの残りの要素を取得）
(defn process-list [[first ...rest]]
  (str "first: " first ", rest: " rest))

(process-list [1 2 3 4])  ;; => "first: 1, rest: (2 3 4)"

;; mapの分解
(defn greet [{:name n :age a}]
  (str n "さんは" a "歳です"))

(greet {:name "太郎" :age 25})  ;; => "太郎さんは25歳です"

;; map分解 + :as束縛
(defn log-user [{:name n :as user}]
  (do
    (println f"Processing: {n}")
    user))

;; defnは以下のように展開される
(defn greet [name] body)
;; ↓
(def greet (fn [name] body))
```

### `fn` - 関数定義

```qi
(fn [x] (* x 2))
(fn [x y] (+ x y))
(fn [] (println "no args"))

;; 可変長引数
(fn [& args] (reduce + 0 args))

;; ベクタの分解（Destructuring）
(fn [[x y]] (+ x y))  ;; 2要素のベクタを受け取る
(fn [[k v]] f"{k}={v}")  ;; キーと値のペア

;; ネストした分解
(fn [[[a b] c]] (+ a b c))  ;; [[1 2] 3] => 6

;; ...rest構文
(fn [[first ...rest]]
  (str "first: " first ", rest: " rest))

;; mapの分解
(fn [{:name n :age a}]
  (str n " is " a " years old"))

;; map分解 + :as束縛
(fn [{:name n :as user}]
  (do
    (println f"Processing: {n}")
    user))
```

### `let` - ローカル束縛

```qi
(let [x 10 y 20]
  (+ x y))

;; ネスト可能
(let [a 1]
  (let [b 2]
    (+ a b)))

;; ベクタの分解（Destructuring）
(let [[x y] [10 20]]
  (+ x y))  ;; => 30

(let [[k v] ["name" "Alice"]]
  f"{k}={v}")  ;; => "name=Alice"

;; ネストした分解
(let [[[a b] c] [[1 2] 3]]
  (+ a b c))  ;; => 6

;; ...rest構文
(let [[first ...rest] [1 2 3 4]]
  (str "first: " first ", rest: " rest))
;; => "first: 1, rest: (2 3 4)"

(let [[x y ...tail] [10 20 30 40]]
  {:x x :y y :tail tail})
;; => {:x 10, :y 20, :tail (30 40)}

;; mapの分解
(let [{:name n :age a} {:name "Alice" :age 30}]
  (str n " is " a))
;; => "Alice is 30"

;; :as束縛（部分と全体を同時に取得）
(let [{:name n :age a :as person} {:name "Bob" :age 25 :role "admin"}]
  [n a person])
;; => ["Bob" 25 {:name "Bob", :age 25, :role "admin"}]
```

### `do` - 順次実行

```qi
(do
  (println "first")
  (println "second")
  42)  ;; 最後の式の値を返す
```

### `if` - 条件分岐

```qi
;; 基本形
(if test then else)

;; 実用例
(if (> x 10) "big" "small")

;; else省略可能（省略時はnil）
(if (valid? data)
  (process data))

;; ネスト
(if (> x 0)
    (if (< x 10) "small positive" "big positive")
    "negative or zero")
```

### `quote` - クオート

```qi
;; 式を評価せずにそのまま返す
'(1 2 3)        ;; リストとして返す
'(+ 1 2)        ;; 評価されず (+ 1 2) のまま
'symbol         ;; シンボルとして返す

;; quoteなしだとエラー（リストは関数呼び出しとして評価される）
(1 2 3)         ;; エラー: 1は関数ではない
```

### `mac` - マクロ定義

マクロはコード生成を行う特殊な関数です。quasiquote (`` ` ``)、unquote (`,`)、unquote-splice (`,@`) を使ってコードテンプレートを作成します。

#### Quasiquote / Unquote の基本

```qi
;; quasiquote (`) - テンプレート作成
`(+ 1 2)         ;; => (+ 1 2) リストとして返す

;; unquote (,) - テンプレート内で式を評価
(def x 10)
`(+ 1 ,x)        ;; => (+ 1 10) xが評価される

;; unquote-splice (,@) - リストを展開
(def items [1 2 3])
`(list ,@items)  ;; => (list 1 2 3) itemsが展開される
```

#### 特殊形式内でのunquote

fn、let、defなどの特殊形式内でもunquoteは正しく動作します：

```qi
(def value 42)

;; fn内でのunquote
`(fn [x] ,value)          ;; => (fn [x] 42)
`(fn [y] (+ y ,value))    ;; => (fn [y] (+ y 42))

;; let内でのunquote
`(let [x ,value] x)       ;; => (let [x 42] x)
`(let [a ,value b 10] (+ a b))  ;; => (let [a 42 b 10] (+ a b))

;; def内でのunquote
`(def myvar ,value)       ;; => (def myvar 42)
```

#### マクロの実装例

```qi
;; whenマクロ - if + doの簡潔版
(mac when [test & body]
  `(if ,test
     (do ,@body)
     nil))

(when (> x 0)
  (println "positive")
  (process x))
;; 展開結果: (if (> x 0) (do (println "positive") (process x)) nil)

;; unlessマクロ - 条件が偽のときに実行
(mac unless [test & body]
  `(if ,test
     nil
     (do ,@body)))

(unless (empty? data)
  (println "has data")
  (process data))

;; debugマクロ - 式と結果を表示
(mac debug [expr]
  `(let [result ,expr]
     (do
       (println f"Debug: {',expr} = {result}")
       result)))

(debug (+ 1 2))
;; 出力: Debug: (+ 1 2) = 3
;; 返り値: 3
```

### `loop` / `recur` - ループ

末尾再帰最適化を実現するための特殊形式です。

```qi
;; 基本形
(loop [var1 val1 var2 val2 ...]
  body
  (recur new-val1 new-val2 ...))

;; 階乗（5! = 120）
(defn factorial [n]
  (loop [i n acc 1]
    (if (= i 0)
      acc
      (recur (dec i) (* acc i)))))

(factorial 5)  ;; 120

;; カウントダウン
(defn count-down [n]
  (loop [i n]
    (if (<= i 0)
      "done"
      (do
        (print i)
        (recur (dec i))))))

;; リスト処理
(defn sum-list [lst]
  (loop [items lst result 0]
    (if (empty? items)
      result
      (recur (rest items) (+ result (first items))))))

(sum-list [1 2 3 4 5])  ;; 15
```

**実装のポイント**:
- `loop`は新しい環境を作成し、変数を初期値で束縛
- `recur`は特別なエラーとして扱い、`loop`でキャッチして変数を更新
- 通常の再帰と異なり、スタックを消費しない（末尾再帰最適化）

---

## 名前空間

**Lisp-1（Scheme派）** - 変数と関数は同じ名前空間

```qi
(def add (fn [x y] (+ x y)))
(def op add)           ;; 関数を変数に代入
(op 1 2)               ;; 3
```

---

## 演算子

### 算術演算子

整数と浮動小数点数の両方をサポートします。異なる型を混在させると、結果は浮動小数点数になります。

```qi
(+ 1 2)         ;; 3
(+ 1.5 2.5)     ;; 4.0
(+ 1 2.5)       ;; 3.5 （型が昇格）

(- 5 3)         ;; 2
(- 5.0 3.0)     ;; 2.0

(* 4 5)         ;; 20
(* 2.5 3)       ;; 7.5

(/ 10 2)        ;; 5
(/ 10.0 2.0)    ;; 5.0
(/ 10 3.0)      ;; 3.3333...

(% 10 3)        ;; 1（剰余）
(% 10.5 3)      ;; 1.5（浮動小数にも対応）
```

### 比較演算子

整数と浮動小数点数の両方をサポートします。異なる型を比較することも可能です。

```qi
(= 1 1)         ;; true
(= 1.0 1.0)     ;; true
(= 1 1.0)       ;; false （型が異なる）

(!= 1 2)        ;; true
(!= 1.0 2.0)    ;; true

(< 1 2)         ;; true
(< 1.5 2.0)     ;; true
(< 1 2.5)       ;; true （整数と浮動小数点数の比較）

(<= 1 1)        ;; true
(<= 1.0 2.0)    ;; true

(> 2 1)         ;; true
(> 2.5 1.5)     ;; true

(>= 2 2)        ;; true
(>= 2.0 2.0)    ;; true
```

### 論理演算子

```qi
(and true false)     ;; false
(or true false)      ;; true
(not true)           ;; false
```

---

## 基本的な関数呼び出し

```qi
;; 関数適用
(f x y z)         ;; f を引数 x, y, z で呼び出し

;; 組み込み関数
(+ 1 2 3)         ;; 6
(str "hello" " " "world")  ;; "hello world"

;; ユーザー定義関数
(defn square [x] (* x x))
(square 5)        ;; 25

;; 高階関数
(map inc [1 2 3])  ;; (2 3 4)
(filter even? [1 2 3 4])  ;; (2 4)
```

---

## Core述語関数

Qiは型チェックや条件判定のための述語関数（`?`で終わる関数）を多数提供しています。

### 型チェック述語（11個）

```qi
;; nil判定
(nil? nil)          ;; => true
(nil? 0)            ;; => false
(nil? "")           ;; => false

;; コレクション型
(list? '(1 2 3))    ;; => true
(vector? [1 2 3])   ;; => true
(map? {:a 1})       ;; => true

;; プリミティブ型
(string? "hello")   ;; => true
(integer? 42)       ;; => true
(float? 3.14)       ;; => true
(number? 42)        ;; => true  (integerまたはfloat)

;; 特殊型
(keyword? :test)    ;; => true
(function? inc)     ;; => true
(atom? (atom 0))    ;; => true
```

### コレクション述語（3個）

```qi
(coll? [1 2 3])           ;; => true  (list/vector/map)
(sequential? [1 2 3])     ;; => true  (listまたはvector)
(empty? [])               ;; => true
(empty? nil)              ;; => true
```

### 状態述語（4個）

```qi
;; nilでない判定
(some? 0)           ;; => true
(some? "")          ;; => true
(some? nil)         ;; => false

;; 厳密な真偽値チェック
(true? true)        ;; => true
(true? 1)           ;; => false  (truthyだがtrueではない)

(false? false)      ;; => true
(false? nil)        ;; => false  (falsyだがfalseではない)

;; エラー判定
(error? {:error "failed"})  ;; => true
(error? 42)                 ;; => false
(error? nil)                ;; => false
```

### 数値述語（5個）

```qi
;; 偶数・奇数
(even? 2)           ;; => true
(odd? 3)            ;; => true

;; 符号判定
(positive? 1)       ;; => true
(negative? -1)      ;; => true
(zero? 0)           ;; => true
(zero? 0.0)         ;; => true
```

### 述語の用途

述語は以下のような場面で活用されます：

```qi
;; filterとの組み合わせ
(filter even? [1 2 3 4 5])        ;; => (2 4)
(filter some? [1 nil 2 nil 3])    ;; => (1 2 3)

;; 条件分岐
(if (nil? x)
  "xはnil"
  "xは何らかの値")

;; match文のガード
(match data
  {:value v} when (positive? v) -> "正の値"
  {:value v} when (zero? v) -> "ゼロ"
  _ -> "その他")
```

**注:** コレクション操作の`list/some?`と`list/every?`（述語+コレクションで判定）は別の関数です（→ データ構造参照）。
