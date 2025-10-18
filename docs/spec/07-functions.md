# 関数

**第一級オブジェクトとしての関数**

Qiでは関数は第一級オブジェクトであり、変数に代入したり、引数として渡したり、返り値として返すことができます。

---

## 関数の定義

### fn - 無名関数

```qi
;; 基本的な関数
(fn [x] (* x 2))

;; 複数引数
(fn [x y] (+ x y))

;; 引数なし
(fn [] (println "no args"))

;; 可変長引数
(fn [& args] (reduce + 0 args))
```

### defn - 名前付き関数

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

;; mapの分解
(defn greet-user [{:name n :age a}]
  (str n "さんは" a "歳です"))
```

---

## クロージャ

関数は定義時のスコープの変数を捕捉できます（クロージャ）。

```qi
;; カウンター関数の生成
(defn make-counter []
  (let [count (atom 0)]
    (fn []
      (swap! count inc)
      (deref count))))

(def counter (make-counter))
(counter)  ;; => 1
(counter)  ;; => 2
(counter)  ;; => 3

;; 部分適用のようなクロージャ
(defn make-adder [n]
  (fn [x] (+ x n)))

(def add5 (make-adder 5))
(add5 10)  ;; => 15
```

---

## 高階関数

### identity - 引数をそのまま返す

```qi
(identity 42)                         ;; => 42

;; フィルタでnilとfalseを除去
(filter identity [1 false nil 2])     ;; => (1 2)
```

### constantly - 常に同じ値を返す関数を生成

```qi
(def always-42 (constantly 42))
(always-42 "anything")                ;; => 42
(always-42 1 2 3)                     ;; => 42

;; 実用例: デフォルト値生成
(map (constantly 0) [1 2 3])          ;; => (0 0 0)
```

### apply - リストを引数として関数に適用

```qi
(apply + [1 2 3])                     ;; => 6
(apply max [5 2 8 3])                 ;; => 8

;; 実用例: 可変長引数の展開
(defn sum-all [& nums]
  (apply + nums))

(sum-all 1 2 3 4 5)                   ;; => 15
```

### comp - 関数を合成（右から左に適用）

```qi
;; 関数合成
(def process (comp inc (* 2)))
(process 5)                           ;; => 11  ((5 * 2) + 1)

;; 複数の関数を合成
(def transform (comp str/upper str/trim))
(transform "  hello  ")               ;; => "HELLO"

;; パイプラインとの比較
;; comp: (comp f g h) は h → g → f の順
;; |>:   x |> h |> g |> f は同じ処理
```

### partial - 部分適用

```qi
;; 部分適用で新しい関数を作成
(def add5 (partial + 5))
(add5 10)                             ;; => 15

;; 複数引数の部分適用
(def greet-hello (partial str "Hello, "))
(greet-hello "Alice")                 ;; => "Hello, Alice"

;; 実用例: フィルタ条件の生成
(def greater-than-10 (partial < 10))
(filter greater-than-10 [5 15 3 20])  ;; => (15 20)
```

---

## fn/ モジュール（高度な高階関数）

### fn/complement - 述語の否定

```qi
;; 述語を否定した関数を生成
(def odd? (fn/complement even?))
(odd? 3)                              ;; => true
(odd? 4)                              ;; => false

;; 実用例: フィルタの反転
(filter (fn/complement nil?) [1 nil 2 nil 3])  ;; => (1 2 3)
```

### fn/juxt - 複数関数を並列適用

```qi
;; 複数の関数を並列に適用して結果をベクタで返す
((fn/juxt inc dec) 5)                 ;; => [6 4]

;; 実用例: データの多角的分析
(def analyze (fn/juxt min max sum len))
(analyze [1 2 3 4 5])                 ;; => [1 5 15 5]

;; ユーザーデータの抽出
(def extract-info (fn/juxt :name :age :email))
(extract-info {:name "Alice" :age 30 :email "alice@example.com"})
;; => ["Alice" 30 "alice@example.com"]
```

### fn/tap> - 副作用を伴う処理

```qi
;; 値をそのまま返しつつ、副作用（ロギングなど）を実行
(def log-and-pass (fn/tap> println))
(log-and-pass 42)  ;; 42を出力して、42を返す

;; パイプラインでのデバッグ
(10
  |> (fn/tap> (fn [x] (println "入力:" x)))
  |> (* _ 2)
  |> (fn/tap> (fn [x] (println "2倍:" x)))
  |> (+ _ 5))
;; 出力:
;; 入力: 10
;; 2倍: 20
;; => 25

;; カウンターの実装
(def counter (atom 0))
(def count-and-pass
  (fn/tap> (fn [_] (reset! counter (+ (deref counter) 1)))))

(map count-and-pass [1 2 3 4 5])
;; => (1 2 3 4 5)
(deref counter)  ;; => 5
```

---

## 実用例

### データ変換パイプライン

```qi
;; 関数合成でデータ処理
(def process-text
  (comp
    str/upper
    str/trim
    (partial str/replace _ "!" ".")))

(process-text "  hello world!  ")     ;; => "HELLO WORLD."
```

### フィルタの組み合わせ

```qi
;; 複数の条件でフィルタ
(def valid-user?
  (fn [user]
    (and
      ((complement nil?) (:name user))
      (> (:age user) 18))))

(filter valid-user?
  [{:name "Alice" :age 30}
   {:name nil :age 20}
   {:name "Bob" :age 15}])
;; => ({:name "Alice" :age 30})
```

### 高階関数でのデバッグ

```qi
;; タップを使ってデータの流れを観察
(defn debug [label]
  (fn [x]
    (println label x)
    x))

([1 2 3]
 |> (map inc)
 |> ((debug "after map:"))
 |> sum)
;; 出力: after map: (2 3 4)
;; => 9
```

### カリー化風の関数定義

```qi
;; 複数の引数を段階的に適用
(defn make-multiplier [n]
  (fn [x] (* x n)))

(def double (make-multiplier 2))
(def triple (make-multiplier 3))

(double 5)  ;; => 10
(triple 5)  ;; => 15

;; パイプラインで使う
([1 2 3] |> (map double))  ;; => (2 4 6)
```
