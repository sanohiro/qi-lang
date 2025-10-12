# 関数型プログラミング

高階関数とデータ変換について学びます。

## 高階関数とは

**関数を引数に取る関数**、または**関数を返す関数**です。

```lisp
; 関数を引数に取る
(map inc [1 2 3])    ; => [2 3 4]

; 関数を返す
(defn make-adder [n]
  (fn [x] (+ n x)))
```

## map - 変換

各要素に関数を適用します。

```lisp
(map function collection)

; 例
(map inc [1 2 3 4 5])
; => [2 3 4 5 6]

(map (fn [x] (* x 2)) [1 2 3])
; => [2 4 6]

(map str/upper ["hello" "world"])
; => ["HELLO" "WORLD"]
```

### 複数のコレクション

```lisp
(map + [1 2 3] [10 20 30])
; => [11 22 33]

(map str ["a" "b" "c"] [1 2 3])
; => ["a1" "b2" "c3"]
```

### マップの変換

```lisp
(map (fn [[k v]] [k (* v 2)])
     {:a 1 :b 2 :c 3})
; => ([:a 2] [:b 4] [:c 6])
```

## filter - フィルタリング

条件を満たす要素のみを抽出します。

```lisp
(filter predicate collection)

; 例
(filter even? [1 2 3 4 5 6])
; => [2 4 6]

(filter (fn [x] (> x 0)) [-2 -1 0 1 2])
; => [1 2]

(filter string? [1 "hello" 2 "world" 3])
; => ["hello" "world"]
```

### 実用例

```lisp
(def users
  [{:name "Alice" :age 25}
   {:name "Bob" :age 30}
   {:name "Charlie" :age 35}])

; 30歳以上
(filter (fn [u] (>= (:age u) 30)) users)
; => ({:name "Bob" :age 30} {:name "Charlie" :age 35})
```

## reduce - 畳み込み

コレクションを1つの値に集約します。

```lisp
(reduce function initial collection)

; 例
(reduce + 0 [1 2 3 4 5])
; => 15

(reduce * 1 [1 2 3 4 5])
; => 120
```

### 初期値の省略

```lisp
(reduce + [1 2 3 4 5])
; => 15（最初の要素が初期値）
```

### 実用例

```lisp
; 最大値
(reduce max [3 7 2 9 4])
; => 9

; 文字列連結
(reduce str "" ["Hello" " " "World"])
; => "Hello World"

; カウント
(reduce (fn [acc x] (+ acc 1)) 0 [1 2 3 4 5])
; => 5

; マップの構築
(reduce (fn [acc x] (assoc acc x (* x 2)))
        {}
        [1 2 3])
; => {1 2 2 4 3 6}
```

## map/filter/reduceの組み合わせ

```lisp
; 偶数を2倍して合計
(reduce +
        (map (fn [x] (* x 2))
             (filter even? [1 2 3 4 5 6])))
; => 24（2*2 + 4*2 + 6*2 = 24）
```

## パイプライン演算子

データの流れを直感的に表現します。

### 基本的な |>

```lisp
; ネストした関数呼び出し
(f (g (h x)))

; パイプライン
x |> h |> g |> f

; 実例
[1 2 3 4 5]
|> (map inc)
|> (filter even?)
|> (reduce +)
; => 20
```

### 複数引数

```lisp
; (map inc [1 2 3]) と同じ
[1 2 3] |> (map inc)

; 最後の引数として追加される
[1 2 3] |> (map inc) |> (filter even?)
```

### 実用例

```lisp
; テキスト処理
"hello world"
|> str/upper
|> (str/split " ")
|> (map str/reverse)
|> (str/join " ")
; => "OLLEH DLROW"

; データ処理
users
|> (filter (fn [u] (>= (:age u) 30)))
|> (map :name)
|> (str/join ", ")
; => "Bob, Charlie"
```

## その他の高階関数

### take-while / drop-while

```lisp
; 条件を満たす間取得
(take-while (fn [x] (< x 5)) [1 2 3 4 5 6 7])
; => (1 2 3 4)

; 条件を満たす間スキップ
(drop-while (fn [x] (< x 5)) [1 2 3 4 5 6 7])
; => (5 6 7)
```

### find

```lisp
; 最初にマッチする要素
(find (fn [x] (> x 5)) [1 2 3 6 7 8])
; => 6
```

### every? / some?

```lisp
; すべてが条件を満たす
(every? even? [2 4 6 8])
; => true

; いずれかが条件を満たす
(some? even? [1 3 5 6 7])
; => true
```

### partition / group-by

```lisp
; n個ずつ分割
(list/partition 2 (fn [x] (even? x)) [1 2 3 4 5 6])
; => ((1 3 5) (2 4 6))

; 関数の結果でグループ化
(list/group-by (fn [x] (% x 3)) [1 2 3 4 5 6 7 8 9])
; => {0 [3 6 9] 1 [1 4 7] 2 [2 5 8]}
```

### sort-by

```lisp
(def people
  [{:name "Charlie" :age 35}
   {:name "Alice" :age 25}
   {:name "Bob" :age 30}])

; 年齢でソート
(list/sort-by :age people)
; => ({:name "Alice" :age 25}
;     {:name "Bob" :age 30}
;     {:name "Charlie" :age 35})
```

## 部分適用

### partial

```lisp
; 最初のn個の引数を固定
(def add5 (partial + 5))
(add5 10)        ; => 15

(def multiply-by-2 (partial * 2))
(multiply-by-2 7) ; => 14

; 実用例
(def numbers [1 2 3 4 5])
(map (partial * 2) numbers)
; => [2 4 6 8 10]
```

## 関数合成

### comp

```lisp
; 関数を合成
(def f (comp inc (* 2)))
(f 5)            ; => 11（(5 * 2) + 1）

; 複数の関数
(def process (comp str/upper str/trim))
(process "  hello  ")
; => "HELLO"
```

### パイプラインとの違い

```lisp
; パイプライン（データから始まる）
"  hello  " |> str/trim |> str/upper

; 関数合成（関数を作る）
(def process (comp str/upper str/trim))
(process "  hello  ")
```

## イミュータブルな更新

### update / update-in

```lisp
; マップの値を関数で更新
(update {:a 1 :b 2} :a inc)
; => {:a 2 :b 2}

; ネストしたマップ
(update-in {:user {:score 100}} [:user :score] (fn [s] (* s 2)))
; => {:user {:score 200}}
```

### map/update-keys / map/update-vals

```lisp
; すべてのキーを変換
(map/update-keys {:a 1 :b 2} str/upper)
; => {"A" 1 "B" 2}

; すべての値を変換
(map/update-vals {:a 1 :b 2 :c 3} (fn [v] (* v 2)))
; => {:a 2 :b 4 :c 6}
```

## 遅延評価

Qiの一部の関数は遅延評価されます。

```lisp
; range は遅延シーケンス
(def big-range (range 1000000))  ; すぐに返る

; 必要な部分だけ評価される
(take 5 big-range)
; => (0 1 2 3 4)
```

## 実践例

### 例1: データの集計

```lisp
(def sales
  [{:product "A" :amount 100}
   {:product "B" :amount 200}
   {:product "A" :amount 150}
   {:product "C" :amount 300}
   {:product "B" :amount 250}])

; 商品ごとの売上合計
(def total-by-product
  (list/group-by :product sales
    |> (map/update-vals (fn [items]
                          (reduce + (map :amount items))))))

; => {"A" 250 "B" 450 "C" 300}
```

### 例2: テキスト分析

```lisp
(def text "the quick brown fox jumps over the lazy dog")

; 単語の出現回数
text
|> (str/split " ")
|> list/frequencies
|> (filter (fn [[word count]] (> count 1)))
; => (["the" 2])

; 最も長い単語
text
|> (str/split " ")
|> (reduce (fn [longest word]
             (if (> (str/len word) (str/len longest))
               word
               longest)))
; => "quick" または "brown" または "jumps"
```

### 例3: ユーザーデータの変換

```lisp
(def raw-users
  [{:first "Alice" :last "Smith" :age 25}
   {:first "Bob" :last "Jones" :age 30}
   {:first "Charlie" :last "Brown" :age 35}])

; フルネームを追加
raw-users
|> (map (fn [u]
          (assoc u :fullname
                 (str (:first u) " " (:last u)))))
|> (filter (fn [u] (>= (:age u) 30)))
|> (map (fn [u] (select-keys u [:fullname :age])))
; => ({:fullname "Bob Jones" :age 30}
;     {:fullname "Charlie Brown" :age 35})
```

### 例4: CSV処理

```lisp
(def csv-data
  "name,age,city\nAlice,25,Tokyo\nBob,30,Osaka\nCharlie,35,Tokyo")

; パース
csv-data
|> (str/split "\n")
|> (map (fn [line] (str/split line ",")))
|> (fn [rows]
     (def headers (first rows))
     (def data (rest rows))
     (map (fn [row] (zipmap headers row)) data))
; => ({:name "Alice" :age "25" :city "Tokyo"}
;     {:name "Bob" :age "30" :city "Osaka"}
;     {:name "Charlie" :age "35" :city "Tokyo"})
```

### 例5: 数列の生成と処理

```lisp
; フィボナッチ数列の最初の10個
(loop [a 0 b 1 result []]
  (if (>= (len result) 10)
    result
    (recur b (+ a b) (conj result a))))
; => [0 1 1 2 3 5 8 13 21 34]

; 偶数のフィボナッチ数の合計（100以下）
(loop [a 0 b 1 sum 0]
  (if (> a 100)
    sum
    (recur b (+ a b)
           (if (even? a) (+ sum a) sum))))
; => 44（0 + 2 + 8 + 34）
```

## まとめ

関数型プログラミングの要素：

- **高階関数**: map, filter, reduce
- **パイプライン**: |> でデータフロー
- **部分適用**: partial
- **関数合成**: comp
- **イミュータブル**: update, update-in
- **遅延評価**: range等

これらを組み合わせて、宣言的で読みやすいコードを書けます！

## 次のステップ

次は[並行処理](./05-concurrency.md)を学びます。go、チャネル、並列処理について理解しましょう。
