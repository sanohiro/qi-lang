# 第2章: パイプラインで考える

**所要時間**: 20分

パイプラインは、Qiの最も強力な機能の一つです。**データの流れを直感的に記述**できます。

---

## なぜパイプラインか？

従来のネストした関数呼び出しは、読みにくくなりがちです。

### ネストした書き方（読みにくい）

```qi
(str/upper (str/reverse (str/trim "  hello  ")))
; => "OLLEH"
```

内側から外側へ読む必要があり、データの流れが分かりにくいです。

### パイプラインで書く（読みやすい）

```qi
("  hello  "
 |> str/trim
 |> str/reverse
 |> str/upper)
; => "OLLEH"
```

**データが左から右に流れる**ので、処理の順序が一目瞭然です！

---

## パイプライン演算子 `|>`

`|>`は、左の値を右の関数に渡します。

```qi
qi> (10 |> inc)
; => 11

qi> (5 |> (* 2))
; => 10

qi> (5 |> (fn [x] (* x 2)))
; => 10
```

---

## 実用例1: 数値処理

```qi
qi> (10
     |> (+ 5)        ; 10 + 5 = 15
     |> (* 2)        ; 15 * 2 = 30
     |> (- 10))      ; 30 - 10 = 20
; => 20
```

**データの流れ**:
```
10 → 15 → 30 → 20
```

---

## 実用例2: 文字列処理

```qi
qi> ("hello world"
     |> str/upper        ; "HELLO WORLD"
     |> (str/replace " " "-")  ; "HELLO-WORLD"
     |> (str/concat "!"))      ; "HELLO-WORLD!"
; => "HELLO-WORLD!"
```

---

## 実用例3: リスト処理

```qi
qi> ([1 2 3 4 5 6 7 8 9 10]
     |> (filter even?)           ; [2 4 6 8 10]
     |> (map (fn [x] (* x x)))   ; [4 16 36 64 100]
     |> (reduce + 0))            ; 220
; => 220
```

**処理の流れ**:
1. 偶数だけを抽出: `[2 4 6 8 10]`
2. 各要素を2乗: `[4 16 36 64 100]`
3. 全て足し算: `220`

---

## パイプラインとデバッグ: `tap>`

`tap>`は、データを変更せずに副作用（printなど）を実行します。デバッグに便利です。

```qi
qi> ([1 2 3 4 5]
     |> (tap println)              ; デバッグ出力
     |> (map (fn [x] (* x 2)))
     |> (tap println)              ; デバッグ出力
     |> (reduce + 0))
; => 出力: [1 2 3 4 5]
; => 出力: [2 4 6 8 10]
; => 30
```

`tap>`を使うと、パイプラインの途中でデータの状態を確認できます。

---

## パイプラインで関数を定義する

パイプラインを使って、読みやすい関数を書けます。

### 例: テキスト整形関数

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

### 例: データ集計関数

```qi
(defn sum-of-squares [numbers]
  (numbers
   |> (map (fn [x] (* x x)))
   |> (reduce + 0)))

qi> (sum-of-squares [1 2 3 4 5])
; => 55  (1 + 4 + 9 + 16 + 25)
```

---

## 実用例4: CSVデータの処理（模擬）

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

## 実用例5: 統計処理

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

## パイプラインの利点

### 1. 読みやすい

データの流れが一目瞭然：

```qi
; ❌ 読みにくい
(reduce + 0 (map (fn [x] (* x x)) (filter even? [1 2 3 4 5 6])))

; ✅ 読みやすい
([1 2 3 4 5 6]
 |> (filter even?)
 |> (map (fn [x] (* x x)))
 |> (reduce + 0))
```

### 2. デバッグしやすい

`tap>`で途中経過を確認：

```qi
([1 2 3 4 5 6]
 |> (filter even?)
 |> (tap println)  ; ← ここで確認
 |> (map (fn [x] (* x x)))
 |> (tap println)  ; ← ここで確認
 |> (reduce + 0))
```

### 3. 拡張しやすい

パイプラインの途中に処理を追加するのが簡単：

```qi
(data
 |> step1
 |> step2
 |> new-step  ; ← 簡単に追加
 |> step3)
```

---

## パイプラインと関数合成

パイプラインは、複数の関数を組み合わせて新しい関数を作ります。

```qi
(defn double [x] (* x 2))
(defn add-ten [x] (+ x 10))
(defn square [x] (* x x))

; パイプラインで合成
(defn transform [x]
  (x
   |> double
   |> add-ten
   |> square))

qi> (transform 5)
; => 400  ((5 * 2 + 10) ^ 2 = 20 ^ 2 = 400)
```

---

## 練習問題

### 問題1: URLスラッグ生成

ブログのタイトルからURL用のスラッグを生成する関数を書いてください。

```qi
(defn make-slug [title]
  ; ここを埋めてください
  ; ヒント: str/trim, str/lower, (str/replace " " "-") を使う
  )

; テスト
(make-slug "  Hello World  ")  ; => "hello-world"
(make-slug "Qi Programming Language")  ; => "qi-programming-language"
```

<details>
<summary>解答例</summary>

```qi
(defn make-slug [title]
  (title
   |> str/trim
   |> str/lower
   |> (str/replace " " "-")))
```

</details>

### 問題2: 偶数の2乗の合計

リストから偶数だけを取り出し、それぞれを2乗して、全て足し算する関数を書いてください。

```qi
(defn sum-even-squares [numbers]
  ; ここを埋めてください
  )

; テスト
(sum-even-squares [1 2 3 4 5 6])  ; => 56  (4 + 16 + 36)
```

<details>
<summary>解答例</summary>

```qi
(defn sum-even-squares [numbers]
  (numbers
   |> (filter even?)
   |> (map (fn [x] (* x x)))
   |> (reduce + 0)))
```

</details>

### 問題3: 名前リストの整形

名前のリストを受け取り、すべて大文字にして、アルファベット順にソートする関数を書いてください。

```qi
(defn format-names [names]
  ; ここを埋めてください
  )

; テスト
(format-names ["charlie" "alice" "bob"])
; => ["ALICE" "BOB" "CHARLIE"]
```

<details>
<summary>解答例</summary>

```qi
(defn format-names [names]
  (names
   |> (map str/upper)
   |> sort))
```

</details>

---

## まとめ

この章で学んだこと：

- ✅ パイプライン演算子 `|>` の使い方
- ✅ データの流れを設計する考え方
- ✅ `tap>`でデバッグする方法
- ✅ 実用的なパイプライン例（文字列、数値、リスト）
- ✅ パイプラインと関数合成

---

## 次のステップ

パイプラインでデータの流れを自由に扱えるようになったら、次は**パターンマッチング**を学びましょう！

➡️ [第3章: パターンマッチングをマスターする](03-pattern-matching.md)

パターンマッチングは、データの構造に応じて処理を分岐する強力な機能です。
