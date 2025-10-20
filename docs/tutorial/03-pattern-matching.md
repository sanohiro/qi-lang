# 第3章: パターンマッチングをマスターする

**所要時間**: 25分

パターンマッチングは、データの構造に応じて処理を分岐する強力な機能です。`if`を何重にもネストするより、**はるかに読みやすく保守しやすい**コードが書けます。

---

## `match`式の基本

`match`は、値をパターンと照合して、マッチしたものの結果を返します。

```qi
(match value
  pattern1 -> result1
  pattern2 -> result2
  _ -> default)
```

### 例: シンプルなマッチング

```qi
qi> (match 42
      42 -> "The answer!"
      _ -> "Something else")
; => "The answer!"

qi> (match "hello"
      "hello" -> "Hi!"
      "bye" -> "Goodbye!"
      _ -> "Unknown")
; => "Hi!"
```

**ポイント**: `_`はワイルドカード（何にでもマッチ）です。

---

## リストのパターンマッチング

リストを分解して、最初の要素や残りの要素を取り出せます。

```qi
qi> (match [1 2 3]
      [] -> "Empty"
      [x] -> f"Single: {x}"
      [first second] -> f"Pair: {first}, {second}"
      [a b c] -> f"Three: {a}, {b}, {c}")
; => "Three: 1, 2, 3"
```

### first/rest パターン

```qi
qi> (defn describe-list [lst]
      (match lst
        [] -> "Empty list"
        [x] -> f"Single element: {x}"
        [first & rest] -> f"First: {first}, Rest: {rest}"))

qi> (describe-list [])
; => "Empty list"

qi> (describe-list [10])
; => "Single element: 10"

qi> (describe-list [1 2 3 4 5])
; => "First: 1, Rest: [2 3 4 5]"
```

---

## マップのパターンマッチング

マップの構造を分解して、値を取り出せます。

```qi
qi> (def person {:name "Alice" :age 25 :city "Tokyo"})

qi> (match person
      {:name n :age a} -> f"{n} is {a} years old")
; => "Alice is 25 years old"
```

### 実用例: HTTPレスポンス処理

```qi
(defn handle-response [resp]
  (match resp
    {:status 200 :body body} -> f"Success: {body}"
    {:status 404} -> "Not Found"
    {:status 500 :message msg} -> f"Server Error: {msg}"
    _ -> "Unknown response"))

qi> (handle-response {:status 200 :body "OK"})
; => "Success: OK"

qi> (handle-response {:status 404})
; => "Not Found"

qi> (handle-response {:status 500 :message "Database error"})
; => "Server Error: Database error"
```

---

## ガード条件

`when`を使って、パターンに条件を追加できます。

```qi
(defn classify-number [n]
  (match n
    x when (> x 0) -> "Positive"
    x when (< x 0) -> "Negative"
    _ -> "Zero"))

qi> (classify-number 10)
; => "Positive"

qi> (classify-number -5)
; => "Negative"

qi> (classify-number 0)
; => "Zero"
```

### 複雑なガード

```qi
(defn describe-age [age]
  (match age
    n when (< n 13) -> "Child"
    n when (and (>= n 13) (< n 20)) -> "Teenager"
    n when (and (>= n 20) (< n 65)) -> "Adult"
    _ -> "Senior"))

qi> (describe-age 10)
; => "Child"

qi> (describe-age 15)
; => "Teenager"

qi> (describe-age 30)
; => "Adult"

qi> (describe-age 70)
; => "Senior"
```

---

## orパターン

複数のパターンをまとめて扱えます（`|`で区切る）。

```qi
(defn weekday-or-weekend [day]
  (match day
    "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" -> "Weekday"
    "Saturday" | "Sunday" -> "Weekend"
    _ -> "Unknown"))

qi> (weekday-or-weekend "Monday")
; => "Weekday"

qi> (weekday-or-weekend "Saturday")
; => "Weekend"
```

---

## 実用例1: コマンド処理

```qi
(defn handle-command [cmd]
  (match cmd
    {:type "add" :a a :b b} -> (+ a b)
    {:type "sub" :a a :b b} -> (- a b)
    {:type "mul" :a a :b b} -> (* a b)
    {:type "div" :a a :b b} -> (/ a b)
    _ -> {:error "Unknown command"}))

qi> (handle-command {:type "add" :a 10 :b 20})
; => 30

qi> (handle-command {:type "mul" :a 5 :b 6})
; => 30

qi> (handle-command {:type "unknown"})
; => {:error "Unknown command"}
```

---

## 実用例2: ステートマシン

```qi
(defn process-state [state event]
  (match [state event]
    ["idle" "start"] -> "running"
    ["running" "pause"] -> "paused"
    ["running" "stop"] -> "stopped"
    ["paused" "resume"] -> "running"
    ["paused" "stop"] -> "stopped"
    _ -> state))

qi> (process-state "idle" "start")
; => "running"

qi> (process-state "running" "pause")
; => "paused"

qi> (process-state "paused" "resume")
; => "running"
```

---

## 実用例3: JSONデータのバリデーション

```qi
(defn validate-user [data]
  (match data
    {:name n :email e} when (and (string? n) (string? e)) ->
      {:valid true :user {:name n :email e}}
    _ ->
      {:valid false :error "Invalid user data"}))

qi> (validate-user {:name "Alice" :email "alice@example.com"})
; => {:valid true :user {:name "Alice" :email "alice@example.com"}}

qi> (validate-user {:name "Bob"})
; => {:valid false :error "Invalid user data"}

qi> (validate-user {:name 123 :email "test@test.com"})
; => {:valid false :error "Invalid user data"}
```

---

## 実用例4: 木構造の走査

```qi
; 二分木のノード
; {:value 値 :left 左の子 :right 右の子}

(defn tree-sum [tree]
  (match tree
    nil -> 0
    {:value v :left nil :right nil} -> v
    {:value v :left l :right nil} -> (+ v (tree-sum l))
    {:value v :left nil :right r} -> (+ v (tree-sum r))
    {:value v :left l :right r} -> (+ v (tree-sum l) (tree-sum r))))

(def my-tree
  {:value 10
   :left {:value 5 :left nil :right nil}
   :right {:value 15 :left nil :right nil}})

qi> (tree-sum my-tree)
; => 30  (10 + 5 + 15)
```

---

## matchとifの比較

### ifを使った場合（読みにくい）

```qi
(defn classify-response [resp]
  (if (and (map? resp) (= (get resp :status) 200))
    (get resp :body)
    (if (and (map? resp) (= (get resp :status) 404))
      "Not Found"
      (if (and (map? resp) (= (get resp :status) 500))
        "Server Error"
        "Unknown"))))
```

### matchを使った場合（読みやすい）

```qi
(defn classify-response [resp]
  (match resp
    {:status 200 :body body} -> body
    {:status 404} -> "Not Found"
    {:status 500} -> "Server Error"
    _ -> "Unknown"))
```

---

## パターンマッチングの利点

### 1. 可読性

データ構造が一目瞭然：

```qi
; ✅ 読みやすい
(match user
  {:name n :age a} -> f"{n} is {a}"
  _ -> "Unknown")
```

### 2. 網羅性

すべてのケースを処理しているか確認しやすい：

```qi
(match status
  "pending" -> handle-pending
  "approved" -> handle-approved
  "rejected" -> handle-rejected
  _ -> handle-unknown)  ; 忘れずに！
```

### 3. 保守性

新しいケースを追加するのが簡単：

```qi
(match command
  "start" -> start-process
  "stop" -> stop-process
  "restart" -> restart-process  ; ← 追加が簡単
  _ -> unknown-command)
```

---

## 練習問題

### 問題1: グレード判定

点数を受け取り、グレード（A/B/C/D/F）を返す関数を書いてください。

```qi
(defn get-grade [score]
  ; ここを埋めてください
  ; 90以上: A, 80以上: B, 70以上: C, 60以上: D, それ以下: F
  )

; テスト
(get-grade 95)  ; => "A"
(get-grade 85)  ; => "B"
(get-grade 75)  ; => "C"
(get-grade 65)  ; => "D"
(get-grade 50)  ; => "F"
```

<details>
<summary>解答例</summary>

```qi
(defn get-grade [score]
  (match score
    s when (>= s 90) -> "A"
    s when (>= s 80) -> "B"
    s when (>= s 70) -> "C"
    s when (>= s 60) -> "D"
    _ -> "F"))
```

</details>

### 問題2: リストの最初の2要素を取得

リストを受け取り、最初の2要素を返す関数を書いてください。

```qi
(defn first-two [lst]
  ; ここを埋めてください
  )

; テスト
(first-two [1 2 3 4 5])  ; => [1 2]
(first-two [10])         ; => [10]
(first-two [])           ; => []
```

<details>
<summary>解答例</summary>

```qi
(defn first-two [lst]
  (match lst
    [] -> []
    [x] -> [x]
    [a b & _] -> [a b]))
```

</details>

### 問題3: オプション型の処理

`:some`か`:none`のマップを受け取り、値があれば処理する関数を書いてください。

```qi
(defn process-option [opt]
  ; ここを埋めてください
  ; {:type :some :value v} => v * 2
  ; {:type :none} => 0
  )

; テスト
(process-option {:type :some :value 10})  ; => 20
(process-option {:type :none})            ; => 0
```

<details>
<summary>解答例</summary>

```qi
(defn process-option [opt]
  (match opt
    {:type :some :value v} -> (* v 2)
    {:type :none} -> 0))
```

</details>

---

## まとめ

この章で学んだこと：

- ✅ `match`式の基本
- ✅ リストとマップのパターンマッチング
- ✅ ガード条件（`when`）
- ✅ orパターン（`|`）
- ✅ 実用的なパターンマッチング例

---

## 次のステップ

パターンマッチングでデータ構造を自在に扱えるようになったら、次は**エラー処理**を学びましょう！

➡️ [第4章: エラーを優雅に扱う](04-error-handling.md)

エラー処理とRailway Pipelineは、堅牢なアプリケーションを作るために欠かせません。
