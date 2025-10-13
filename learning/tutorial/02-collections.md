# リストとコレクション

Qiのデータ構造について学びます。

## リスト

### 作成

```lisp
; list関数
(list 1 2 3 4 5)     ; => (1 2 3 4 5)

; クォート
'(1 2 3 4 5)         ; => (1 2 3 4 5)

; 空リスト
'()                  ; => ()
```

### 基本操作

```lisp
; 先頭要素
(first '(1 2 3))     ; => 1

; 残り
(rest '(1 2 3))      ; => (2 3)

; 末尾要素
(last '(1 2 3))      ; => 3

; n番目の要素（0始まり）
(nth '(1 2 3 4) 2)   ; => 3

; 長さ
(len '(1 2 3))       ; => 3
(count '(1 2 3))     ; => 3

; 空かチェック
(empty? '())         ; => true
(empty? '(1))        ; => false
```

### 要素の追加

```lisp
; 先頭に追加
(cons 0 '(1 2 3))    ; => (0 1 2 3)

; 末尾に追加
(conj '(1 2 3) 4)    ; => (1 2 3 4)

; 連結
(concat '(1 2) '(3 4) '(5 6))
; => (1 2 3 4 5 6)
```

## ベクタ

ベクタはランダムアクセスが速いコレクションです。

### 作成

```lisp
; ブラケット記法
[1 2 3 4 5]

; vec関数
(vec '(1 2 3))       ; => [1 2 3]

; 空ベクタ
[]
```

### 基本操作

```lisp
; リストと同じ操作が使える
(first [1 2 3])      ; => 1
(rest [1 2 3])       ; => (2 3)  ; リストになる
(last [1 2 3])       ; => 3
(nth [1 2 3] 1)      ; => 2

; conjは末尾に追加
(conj [1 2 3] 4)     ; => [1 2 3 4]
```

### リストvsベクタ

| 操作 | リスト | ベクタ |
|---|---|---|
| 先頭アクセス | O(1) | O(1) |
| ランダムアクセス | O(n) | O(1) |
| 先頭に追加 | O(1) | O(n) |
| 末尾に追加 | O(n) | O(1) |

## マップ

キーと値のペアを格納します。

### 作成

```lisp
; ブレース記法
{:name "Alice" :age 30}

; 空マップ
{}
```

### アクセス

```lisp
(def person {:name "Alice" :age 30})

; get関数
(get person :name)   ; => "Alice"
(get person :city)   ; => nil

; デフォルト値
(get person :city "Unknown")  ; => "Unknown"

; キーワードを関数として使う
(:name person)       ; => "Alice"
(:city person)       ; => nil
```

### 更新

```lisp
; 追加・更新
(assoc {:a 1} :b 2)  ; => {:a 1 :b 2}
(assoc {:a 1} :a 10) ; => {:a 10}

; 削除
(dissoc {:a 1 :b 2} :a)  ; => {:b 2}

; マージ
(merge {:a 1} {:b 2} {:c 3})
; => {:a 1 :b 2 :c 3}
```

### キーと値

```lisp
(def m {:a 1 :b 2 :c 3})

; キーの一覧
(keys m)             ; => [:a :b :c]

; 値の一覧
(vals m)             ; => [1 2 3]
```

### ネストしたマップ

```lisp
(def data
  {:user {:name "Alice"
          :address {:city "Tokyo"
                    :zip "100-0001"}}})

; get-in - ネストしたアクセス
(get-in data [:user :name])            ; => "Alice"
(get-in data [:user :address :city])   ; => "Tokyo"

; assoc-in - ネストした更新
(assoc-in data [:user :age] 30)
; => {:user {:name "Alice" :age 30 :address {...}}}

; update-in - ネストした更新（関数適用）
(update-in data [:user :address :zip] str/upper)
```

## セット

重複を許さないコレクションです。

### 作成

```lisp
; セット
#{1 2 3 4 5}

; 重複は自動で削除される
#{1 2 2 3 3 3}       ; => #{1 2 3}
```

### 操作

```lisp
; 要素の確認
(contains? #{1 2 3} 2)     ; => true
(contains? #{1 2 3} 5)     ; => false

; 追加
(conj #{1 2 3} 4)          ; => #{1 2 3 4}

; 削除
(disj #{1 2 3} 2)          ; => #{1 3}

; 集合演算
(set/union #{1 2} #{2 3})        ; => #{1 2 3}
(set/intersect #{1 2 3} #{2 3 4}) ; => #{2 3}
(set/difference #{1 2 3} #{2})    ; => #{1 3}
```

## Range

連続した数値のシーケンスを生成します。

```lisp
; 0から9まで
(range 10)           ; => (0 1 2 3 4 5 6 7 8 9)

; 1から10まで
(range 1 11)         ; => (1 2 3 4 5 6 7 8 9 10)

; ステップ指定
(range 0 10 2)       ; => (0 2 4 6 8)

; 負の数
(range 10 0 -1)      ; => (10 9 8 7 6 5 4 3 2 1)
```

## 基本的なコレクション操作

### take / drop

```lisp
; 最初のn個
(take 3 [1 2 3 4 5])  ; => (1 2 3)

; 最初のn個をスキップ
(drop 2 [1 2 3 4 5])  ; => (3 4 5)

; 末尾のn個を削除
(list/drop-last 2 [1 2 3 4 5])  ; => (1 2 3)
```

### reverse

```lisp
(reverse [1 2 3 4 5])  ; => (5 4 3 2 1)
```

### sort

```lisp
; 昇順
(sort [3 1 4 1 5 9])  ; => (1 1 3 4 5 9)

; 降順
(reverse (sort [3 1 4 1 5]))  ; => (5 4 3 1 1)
```

### distinct

```lisp
; 重複を削除
(distinct [1 2 2 3 3 3])  ; => (1 2 3)
```

### flatten

```lisp
; ネストを平坦化
(flatten [[1 2] [3 4] [5 6]])
; => (1 2 3 4 5 6)

(flatten [1 [2 [3 [4 5]]]])
; => (1 2 3 4 5)
```

### zip

```lisp
; 2つのコレクションを結合
(zip [1 2 3] ["a" "b" "c"])
; => ([1 "a"] [2 "b"] [3 "c"])
```

### interleave

```lisp
; 交互に要素を取る
(list/interleave [1 2 3] [:a :b :c])
; => (1 :a 2 :b 3 :c)
```

### list/partition

```lisp
; n個ずつグループ化
(list/chunk 2 [1 2 3 4 5 6])
; => ((1 2) (3 4) (5 6))

(list/chunk 3 [1 2 3 4 5 6 7 8])
; => ((1 2 3) (4 5 6) (7 8))
```

## シーケンス処理

すべてのコレクションは**シーケンス**として扱えます。

```lisp
; リスト
(first '(1 2 3))     ; => 1

; ベクタ
(first [1 2 3])      ; => 1

; マップ（キー・バリューのペア）
(first {:a 1 :b 2})  ; => [:a 1]

; セット
(first #{1 2 3})     ; => 1（順序は不定）

; Range
(first (range 10))   ; => 0

; 文字列
(first "hello")      ; => \h
```

## 実践例

### 例1: リストの合計

```lisp
(defn sum [numbers]
  (if (empty? numbers)
    0
    (+ (first numbers) (sum (rest numbers)))))

(sum [1 2 3 4 5])    ; => 15
```

### 例2: リストの反転

```lisp
(defn my-reverse [coll]
  (if (empty? coll)
    '()
    (concat (my-reverse (rest coll)) (list (first coll)))))

(my-reverse [1 2 3 4 5])  ; => (5 4 3 2 1)
```

### 例3: ユーザー管理

```lisp
(def users
  [{:id 1 :name "Alice" :age 25}
   {:id 2 :name "Bob" :age 30}
   {:id 3 :name "Charlie" :age 35}])

; IDで検索
(defn find-user [id users]
  (first (filter (fn [u] (= (:id u) id)) users)))

(find-user 2 users)
; => {:id 2 :name "Bob" :age 30}

; 年齢でフィルタ
(filter (fn [u] (>= (:age u) 30)) users)
; => ({:id 2 :name "Bob" :age 30}
;     {:id 3 :name "Charlie" :age 35})
```

### 例4: 単語の出現回数

```lisp
(def text "the quick brown fox jumps over the lazy dog")

(def words (str/split text " "))
; => ["the" "quick" "brown" "fox" "jumps" "over" "the" "lazy" "dog"]

; 出現回数をカウント
(list/frequencies words)
; => {"the" 2 "quick" 1 "brown" 1 "fox" 1 ...}
```

### 例5: データの整形

```lisp
(def raw-data
  [{:name "Alice" :score 85}
   {:name "Bob" :score 92}
   {:name "Charlie" :score 78}])

; スコアの降順でソート
(def sorted-data
  (reverse (list/sort-by :score raw-data)))

; 名前のリストを抽出
(map :name sorted-data)
; => ("Bob" "Alice" "Charlie")
```

## イミュータブル（不変性）

Qiのデータ構造は**イミュータブル**です。元のデータは変更されません：

```lisp
(def original [1 2 3])

(def modified (conj original 4))
; => [1 2 3 4]

(println original)    ; => [1 2 3]（変わらない）
(println modified)    ; => [1 2 3 4]
```

**利点:**
- 予期しない変更がない
- 並行処理が安全
- 履歴を保持できる

## まとめ

Qiのコレクション：

- **リスト**: 先頭アクセスが速い
- **ベクタ**: ランダムアクセスが速い
- **マップ**: キー・バリューのペア
- **セット**: 重複なし
- **Range**: 連続した数値

基本操作：
- `first`, `rest`, `last`, `nth`
- `conj`, `cons`, `concat`
- `take`, `drop`, `reverse`, `sort`
- `get`, `assoc`, `dissoc`, `merge`

すべてイミュータブル！

## 次のステップ

次は[制御構造](./03-control-flow.md)を学びます。if、match、loopなどの使い方を理解しましょう。
