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
(match data
  {:type "user" :name n} -> (greet n)
  {:type "admin"} -> "admin"
  _ -> "unknown")
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
(match data
  {:user {:name n :age a} :as u} -> (do
    (log u)           ;; 全体をログ
    (process n a)))   ;; 部分を処理

;; ネストした構造でも使える
(match response
  {:body {:user u :posts ps} :as body} -> (cache body)
  {:error e :as err} -> (log err))

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

;; 複雑なネスト例
(defn handle-request [{:headers h :body {:user u :data d :as body} :as req}]
  (do
    (log req)      ;; リクエスト全体
    (cache body)   ;; bodyだけキャッシュ
    (process-user u d)))
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
(match request
  {:user {:age a :country c}} when (and (>= a 18) (= c "JP")) -> (allow)
  {:user {:age a}} when (< a 18) -> (error "age restriction")
  _ -> (deny))

;; Flow的な読み方: データ構造を分解 → ガードで検証 → 処理
```

### 4. ワイルドカード `_` - 関心のある部分だけ抽出

必要な部分だけを抽出し、不要な部分は `_` で無視できます。

```qi
;; 一部のフィールドだけ使う
(match data
  {:user {:name _ :age a :city c}} -> (process-location a c)
  {:error _} -> "error occurred")

;; リストの一部をスキップ
(match coords
  [_ y _] -> y  ;; y座標だけ取り出す
  _ -> 0)
```

### 5. 配列の複数束縛

複数要素を同時に束縛できます。

```qi
;; 複数要素を同時に束縛
(match data
  [{:id id1} {:id id2}] -> (compare id1 id2)
  [first ...rest] -> (process-batch first rest))

;; パイプラインと組み合わせ
(match (coords |> (take 2))
  [x y] -> (distance x y)
  _ -> 0)
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
(match (try (risky-operation))
  {:error e} -> (log e)
  result -> result)

;; パイプラインと組み合わせ
(match (try
         (url
          |> http-get
          |> parse
          |> process))
  {:error e} -> []
  data -> data)
```

---

## 実用例

### 例1: HTTPレスポンスのハンドリング

```qi
;; http/get!は例外を投げる可能性があるのでtryでキャッチ
(match (try (http/get! url))  ;; 詳細版でステータスコードを取得
  {:error e} -> (log-error e)
  {:status 200 :body body} -> (process-body body)
  {:status 404} -> nil
  {:status s} -> (error (str "Unexpected status: " s)))
```

### 例2: データバリデーション

```qi
(match user
  {:name n :age a :email e} when (and (> a 0) (str/contains? e "@")) -> (save-user user)
  {:name _ :age a} when (<= a 0) -> (error "Invalid age")
  {:name _ :email e} when (not (str/contains? e "@")) -> (error "Invalid email")
  _ -> (error "Missing required fields"))
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
