# 第4章: エラーを優雅に扱う

**所要時間**: 30分

エラー処理は、堅牢なアプリケーションを作るために欠かせません。Qiは**Railway Pipeline**という強力なエラー処理パターンを提供しています。

---

## エラー処理の基本: `try`

`try`は、エラーが発生する可能性のあるコードを安全に実行します。

### 基本的な使い方

```qi
qi> (try (+ 1 2))
; => 3  (成功時は生の値を返す)

qi> (try (/ 1 0))
; => {:error "ゼロ除算エラー"}  (エラー時は{:error}形式)
```

**重要**: Qiの新しい仕様では、`try`は**成功時に生の値を返します**（`{:ok value}`ではありません）。エラーだけが`{:error ...}`形式です。

---

## `error?`述語でエラーをチェック

`error?`を使って、値がエラーかどうかを判定できます。

```qi
qi> (error? {:error "Something went wrong"})
; => true

qi> (error? 42)
; => false

qi> (error? {:ok 10})
; => false
```

### 実用例

```qi
(defn safe-divide [a b]
  (try (/ a b)))

(defn process [a b]
  (let [result (safe-divide a b)]
    (if (error? result)
      (println "Error occurred!")
      (println f"Result: {result}"))))

qi> (process 10 2)
; => "Result: 5"

qi> (process 10 0)
; => "Error occurred!"
```

---

## Railway Pipeline: `|>?`

Railway Pipelineは、**エラーを自動的に伝播**させる強力なパターンです。

### 基本的な考え方

通常のパイプライン(`|>`)では、各ステップが必ず実行されます。しかし、**Railway Pipeline (`|>?`)では、エラーが発生すると、それ以降の処理をスキップ**します。

```
      成功        成功        成功
  ─────────> ─────────> ─────────>
 |          |          |          |
  ─────────> X エラー! ─────────>
      成功        ↓        スキップ
                エラー返却
```

### 例: シンプルなRailway Pipeline

```qi
(defn validate-positive [x]
  (if (> x 0)
    x                          ; 成功: そのまま値を返す
    {:error "Must be positive"}))  ; エラー

(defn double [x]
  (* x 2))

(defn add-ten [x]
  (+ x 10))

; 成功ケース
qi> (10
     |>? validate-positive
     |>? double
     |>? add-ten)
; => 30  (10 -> 10 -> 20 -> 30)

; エラーケース
qi> (-5
     |>? validate-positive  ; ここでエラー
     |>? double             ; スキップ
     |>? add-ten)           ; スキップ
; => {:error "Must be positive"}
```

**ポイント**: `{:error}`以外は全て成功として扱われます！

---

## Railway Pipelineの強み

### 1. エラーが自動で伝播する

```qi
(defn validate-age [age]
  (if (and (>= age 0) (<= age 150))
    age
    {:error "Invalid age"}))

(defn validate-name [name]
  (if (> (len name) 0)
    name
    {:error "Name cannot be empty"}))

(defn create-user [name age]
  ({:name name :age age}
   |>? (fn [u] (if (> (len (get u :name)) 0)
                  u
                  {:error "Name cannot be empty"}))
   |>? (fn [u] (if (and (>= (get u :age) 0) (<= (get u :age) 150))
                  u
                  {:error "Invalid age"}))))

qi> (create-user "Alice" 25)
; => {:name "Alice" :age 25}

qi> (create-user "" 25)
; => {:error "Name cannot be empty"}

qi> (create-user "Bob" 200)
; => {:error "Invalid age"}
```

### 2. 読みやすい

エラー処理がデータフローに自然に組み込まれます：

```qi
; ❌ 従来のエラー処理（読みにくい）
(defn process-data [data]
  (let [result1 (step1 data)]
    (if (error? result1)
      result1
      (let [result2 (step2 result1)]
        (if (error? result2)
          result2
          (step3 result2))))))

; ✅ Railway Pipeline（読みやすい）
(defn process-data [data]
  (data
   |>? step1
   |>? step2
   |>? step3))
```

---

## 実用例1: ファイル処理

```qi
(defn read-file [path]
  (try (io/read-file path)))

(defn parse-json [text]
  (try (json/parse text)))

(defn validate-data [data]
  (if (map? data)
    data
    {:error "Data must be a map"}))

(defn extract-field [data field]
  (if (contains? data field)
    (get data field)
    {:error f"Missing field: {field}"}))

; Railway Pipelineで繋ぐ
(defn load-config [path]
  (path
   |>? read-file
   |>? parse-json
   |>? validate-data
   |>? (fn [d] (extract-field d :database))))

; 使用例
qi> (load-config "config.json")
; => "postgresql://localhost/mydb"  (成功時)
; => {:error "ファイルが見つかりません"}  (ファイルなし)
; => {:error "JSONのパースに失敗しました"}  (不正なJSON)
; => {:error "Missing field: database"}  (フィールドなし)
```

---

## 実用例2: API呼び出し

```qi
(defn fetch-user [id]
  ; 模擬HTTPリクエスト
  (if (> id 0)
    {:status 200 :body f"{{"id":{id},"name":"User{id}"}}"}
    {:status 404}))

(defn check-response [resp]
  (match resp
    {:status 200 :body body} -> body
    {:status 404} -> {:error "User not found"}
    _ -> {:error "Unknown error"}))

(defn parse-user [json-str]
  (try (json/parse json-str)))

(defn extract-name [user]
  (if (contains? user :name)
    (get user :name)
    {:error "Missing name"}))

; Railway Pipelineで繋ぐ
(defn get-user-name [id]
  (id
   |> fetch-user
   |>? check-response
   |>? parse-user
   |>? extract-name))

qi> (get-user-name 1)
; => "User1"

qi> (get-user-name -1)
; => {:error "User not found"}
```

---

## `try`とRailway Pipelineの組み合わせ

`try`とRailway Pipelineを組み合わせると、強力なエラー処理が実現できます。

```qi
(defn safe-parse-int [s]
  (try (string/to-int s)))

(defn validate-range [n]
  (if (and (>= n 0) (<= n 100))
    n
    {:error "Number must be between 0 and 100"}))

(defn double [n]
  (* n 2))

; パイプライン
(defn process-input [input]
  (input
   |>? safe-parse-int
   |>? validate-range
   |>? double))

qi> (process-input "25")
; => 50

qi> (process-input "abc")
; => {:error "パースエラー"}

qi> (process-input "150")
; => {:error "Number must be between 0 and 100"}
```

---

## エラーハンドリングのベストプラクティス

### 1. `{:error}`形式を統一する

```qi
; ✅ 良い例
{:error "User not found"}
{:error "Invalid input"}
{:error "Database connection failed"}

; ❌ 悪い例（統一されていない）
"error"
{:err "..."}
{:failed true}
```

### 2. エラーメッセージは具体的に

```qi
; ✅ 良い例
{:error f"User with ID {id} not found"}
{:error f"Age must be between 0 and 150, got {age}"}

; ❌ 悪い例
{:error "Error"}
{:error "Invalid"}
```

### 3. エラーを早期に返す

```qi
; ✅ 良い例: 早期リターン
(defn process [x]
  (if (< x 0)
    {:error "Negative not allowed"}
    (do-something x)))

; ❌ 悪い例: 深いネスト
(defn process [x]
  (if (>= x 0)
    (do-something x)
    {:error "Negative not allowed"}))
```

---

## 練習問題

### 問題1: 安全な除算関数

ゼロ除算を防ぐ関数を書いてください。

```qi
(defn safe-divide [a b]
  ; ここを埋めてください
  )

; テスト
(safe-divide 10 2)  ; => 5
(safe-divide 10 0)  ; => {:error "Division by zero"}
```

<details>
<summary>解答例</summary>

```qi
(defn safe-divide [a b]
  (if (= b 0)
    {:error "Division by zero"}
    (/ a b)))
```

</details>

### 問題2: 数値バリデーションパイプライン

文字列を受け取り、数値に変換して0〜100の範囲かチェックする関数を書いてください。

```qi
(defn validate-score [input]
  ; ここを埋めてください
  ; ヒント: safe-parse-int と validate-range を使う
  )

; テスト
(validate-score "75")   ; => 75
(validate-score "abc")  ; => {:error "パースエラー"}
(validate-score "150")  ; => {:error "Range error"}
```

<details>
<summary>解答例</summary>

```qi
(defn safe-parse-int [s]
  (try (string/to-int s)))

(defn validate-range [n]
  (if (and (>= n 0) (<= n 100))
    n
    {:error "Range error"}))

(defn validate-score [input]
  (input
   |>? safe-parse-int
   |>? validate-range))
```

</details>

### 問題3: 複数バリデーション

ユーザーデータを受け取り、名前と年齢をバリデーションする関数を書いてください。

```qi
(defn validate-user [user]
  ; ここを埋めてください
  ; 名前: 1文字以上
  ; 年齢: 0〜150の範囲
  )

; テスト
(validate-user {:name "Alice" :age 25})
; => {:name "Alice" :age 25}

(validate-user {:name "" :age 25})
; => {:error "Name cannot be empty"}

(validate-user {:name "Bob" :age 200})
; => {:error "Invalid age"}
```

<details>
<summary>解答例</summary>

```qi
(defn validate-user [user]
  (user
   |>? (fn [u] (if (> (len (get u :name)) 0)
                  u
                  {:error "Name cannot be empty"}))
   |>? (fn [u] (if (and (>= (get u :age) 0) (<= (get u :age) 150))
                  u
                  {:error "Invalid age"}))))
```

</details>

---

## まとめ

この章で学んだこと：

- ✅ `try`でエラーを安全にキャッチ
- ✅ `error?`述語でエラーを判定
- ✅ Railway Pipeline (`|>?`)でエラーを自動伝播
- ✅ エラーハンドリングのベストプラクティス
- ✅ 実用的なエラー処理パターン

---

## 次のステップ

エラー処理をマスターしたら、次は**並行・並列処理**を学びましょう！

➡️ [第5章: 並行・並列処理を簡単に](05-concurrency.md)

並行・並列処理は、Qiの最も強力な機能の一つです。
