# パターンマッチング

**データの流れを分岐させる制御構造**

Qiのパターンマッチは、単なる条件分岐ではなく、データ構造を分解・変換・検証しながら処理を振り分けます。

---

## 基本パターン

### 値のマッチ

```qi
(match x
  0 -> "zero"
  1 -> "one"
  n -> (str "other: " n))
```

### nil/boolの区別

```qi
(match result
  nil -> "not found"
  false -> "explicitly false"
  true -> "success"
  v -> (str "value: " v))
```

### マップのマッチ

```qi
(def data {:type "user" :name "Alice"})
(match data
  {:type "user" :name n} -> (str "Hello, " n)
  {:type "admin"} -> "admin"
  _ -> "unknown")
;; => "Hello, Alice"
```

### リストのマッチ

```qi
(match lst
  [] -> "empty"
  [x] -> x
  [x ...rest] -> (str "first: " x))
```

### ガード条件

```qi
(match x
  n when (> n 0) -> "positive"
  n when (< n 0) -> "negative"
  _ -> "zero")
```

---

## 拡張パターン

### 1. `:as` 束縛 - 部分と全体の両方を使う

パターンマッチした全体を変数に束縛できます。

```qi
;; 基本的な:as使用
(def data {:user {:name "Alice" :age 30}})
(match data
  {:user {:name n :age a} :as u} -> (do
    (println u)       ;; 全体を表示
    (str n " is " a " years old")))   ;; 部分を使用
;; => "Alice is 30 years old"

;; ネストした構造でも使える
(def response {:body {:user "Bob" :posts 10}})
(match response
  {:body {:user u :posts ps} :as body} -> (str "User: " u ", Posts: " ps ", Body: " body)
  {:error e :as err} -> (str "Error: " err))
;; => "User: Bob, Posts: 10, Body: {:body {:user Bob, :posts 10}}"

;; 深くネストされた:as
(match {:type "person" :data {:name "Alice" :age 30}}
  {:type t :data {:name n :age a :as user-data} :as record} ->
    (do
      (println f"Type: {t}")           ;; "Type: person"
      (println f"Name: {n}, Age: {a}") ;; "Name: Alice, Age: 30"
      (println f"User data: {user-data}") ;; {:name "Alice", :age 30}
      (println f"Full record: {record}"))) ;; 全体のマップ

;; ベクターとマップの組み合わせ
(match [1 {:x 10 :y 20}]
  [a {:x b :as inner}] -> [a b inner])
;; => [1 10 {:x 10 :y 20}]

;; 関数パラメータでも使える
(defn process [{:name n :age a :as user}]
  (do
    (println f"Processing: {n}")
    user))  ;; 全体を返す

(process {:name "Charlie" :age 25})
;; 出力: Processing: Charlie
;; 戻り値: {:name "Charlie", :age 25}
```

### 2. `or` パターン - 複数パターンで同じ処理

複数の値にマッチさせて同じ処理を実行できます（`|` 記法を使用）。

```qi
;; 複数の値にマッチ
(match status
  200 | 201 | 204 -> "success"
  400 | 401 | 403 -> "client error"
  500 | 502 | 503 -> "server error"
  _ -> "unknown")

;; 文字列にも使える
(match day
  "月" | "火" | "水" | "木" | "金" -> "平日"
  "土" | "日" -> "週末")

;; キーワードにも使える
(match result
  :ok | :success -> (handle-ok)
  :error | :fail -> (handle-error))
```

### 3. ネスト + ガード - 構造的な条件分岐

深いネストとガード条件を組み合わせることができます。

```qi
;; 深いネストでも読みやすい
(def request {:user {:age 20 :country "JP"}})
(match request
  {:user {:age a :country c}} when (and (>= a 18) (= c "JP")) -> "allowed"
  {:user {:age a}} when (< a 18) -> {:error "age restriction"}
  _ -> "denied")
;; => "allowed"
```

### 4. ワイルドカード `_` - 関心のある部分だけ抽出

必要な部分だけを抽出し、不要な部分は `_` で無視できます。

```qi
;; 一部のフィールドだけ使う
(def data {:user {:name "Alice" :age 30 :city "Tokyo"}})
(match data
  {:user {:name _ :age a :city c}} -> (str "Age: " a ", City: " c)
  {:error _} -> "error occurred")
;; => "Age: 30, City: Tokyo"

;; リストの一部をスキップ
(def coords [10 20 30])
(match coords
  [_ y _] -> y  ;; y座標だけ取り出す
  _ -> 0)
;; => 20
```

### 5. 配列の複数束縛

複数要素を同時に束縛できます。

```qi
;; 複数要素を同時に束縛
(def data [{:id 1} {:id 2}])
(match data
  [{:id id1} {:id id2}] -> (str "ID1: " id1 ", ID2: " id2)
  [first ...rest] -> (str "First: " first ", Rest: " rest))
;; => "ID1: 1, ID2: 2"

;; パイプラインと組み合わせ
(def coords [10 20 30 40])
(match (coords |> (take 2))
  [x y] -> (+ x y)
  _ -> 0)
;; => 30
```

---

## matchの設計哲学

1. **データの流れを分岐させる**: matchは単なるif-elseではなく、データ構造を分解して流れを作る
2. **読みやすさ優先**: パターンが上から下に読める、条件が一目で分かる
3. **段階的開示**: 基本パターンから始めて、必要に応じて拡張機能を使う

---

## tryとの組み合わせ

`try` は例外をキャッチし、エラー時に `{:error e}` を返します。成功時は値をそのまま返します。

```qi
;; 成功時は値そのまま、エラー時は {:error e}
(defn risky-operation []
  (if (> (rand) 0.5)
    42
    (panic "Failed")))

(match (try (risky-operation))
  {:error e} -> (str "Error: " e)
  result -> result)
;; => 42 または "Error: Failed"

;; パイプラインと組み合わせ
(def json-str "{\"value\": 100}")
(match (try
         (json-str
          |> json/parse
          |> (fn [data] (get data "value"))
          |> (fn [v] (* v 2))))
  {:error e} -> nil
  data -> data)
;; => 200
```

---

## 実用例

### 例1: HTTPレスポンスのハンドリング

```qi
;; (comment) でラップして実行不可能な例を示す
(comment
  ;; http/get!は例外を投げる可能性があるのでtryでキャッチ
  (match (try (http/get! url))  ;; 詳細版でステータスコードを取得
    {:error e} -> (str "Error: " e)
    {:status 200 :body body} -> body
    {:status 404} -> nil
    {:status s} -> {:error (str "Unexpected status: " s)}))
```

### 例2: データバリデーション

```qi
(def user {:name "Alice" :age 30 :email "alice@example.com"})
(match user
  {:name n :age a :email e} when (and (> a 0) (str/contains? e "@")) -> "Valid user"
  {:name _ :age a} when (<= a 0) -> {:error "Invalid age"}
  {:name _ :email e} when (not (str/contains? e "@")) -> {:error "Invalid email"}
  _ -> {:error "Missing required fields"})
;; => "Valid user"
```

### 例3: リスト処理

```qi
(defn process-list [lst]
  (match lst
    [] -> "empty"
    [x] -> (str "single: " x)
    [x y] -> (str "pair: " x ", " y)
    [x y ...rest] -> (str "multiple: " x ", " y ", and " (len rest) " more")))
```
