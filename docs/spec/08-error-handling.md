# エラー処理

**Qiのエラー処理戦略**

Qiは用途に応じて3つのエラー処理方法を提供します：

1. **Result型 (`{:ok/:error}`)** - 回復可能なエラー、Railway Pipeline
2. **try/catch** - 例外のキャッチとリカバリ
3. **defer** - リソース解放の保証（`finally`の代替）

---

## 1. Result型 - Railway Pipeline（推奨パターン）

**用途**: API、ファイルIO、パース等の失敗が予想される処理

### 基本的な使い方

```qi
;; Result型を返す関数
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}
    {:ok (/ x y)}))

;; matchで処理
(match (divide 10 2)
  {:ok result} -> result
  {:error e} -> (log e))

;; パイプラインでのエラー処理（詳細は02-flow-pipes.mdを参照）
(user-input
 |> validate
 |>? parse-number
 |>? (fn [n] (divide 100 n))
 |>? format-result)
;; エラーは自動的に伝播
```

### Railway Pipelineとの統合

`|>?` 演算子を使うことで、エラーハンドリングをパイプラインに統合できます。

```qi
;; HTTPリクエスト + エラーハンドリング
("https://api.example.com/users/123"
 |> http/get                      ;; => {:ok {:status 200 :body "..."}}
 |>? (fn [resp] (get resp "body"))
 |>? json/parse
 |>? (fn [data] {:ok (get data "user")}))
;; エラー時は自動的に伝播

;; JSONパース + データ変換
("{\"name\":\"Alice\",\"age\":30}"
 |> json/parse                    ;; => {:ok {...}}
 |>? (fn [data] {:ok (get data "name")})
 |>? (fn [name] {:ok (str/upper name)}))
;; => {:ok "ALICE"}
```

### 設計哲学

エラーをデータとして扱い、パイプラインの中で流す。try-catchのネストを避け、データフローが明確になる。

---

## 2. try/catch - 例外処理

**用途**: 予期しないエラーのキャッチ、サードパーティコードの呼び出し

### 基本的な使い方

```qi
;; try-catchブロック
(match (try (risky-operation))
  {:ok result} -> result
  {:error e} -> (handle-error e))

;; ネスト可能
(match (try
         (let [data (parse-data input)]
           (process data)))
  {:ok result} -> result
  {:error e} -> {:error (str "Failed: " e)})
```

### 実用例

```qi
;; ファイル読み込みのエラー処理
(match (try (io/read-file "config.json"))
  {:ok content} -> (json/parse content)
  {:error e} -> (do
                  (log/error "Failed to read config:" e)
                  {:error e}))

;; パイプラインでの使用
(match (try
         (url
          |> http-get
          |> parse
          |> process))
  {:ok data} -> data
  {:error e} -> [])
```

**注意**: Qiには`finally`がありません。代わりに`defer`を使います（下記参照）。

---

## 3. defer - リソース解放の保証（finallyの代替）

**用途**: ファイル、接続、ロックなどのリソース管理

### 基本的な使い方

```qi
;; deferで確実にクリーンアップ
(defn process-file [path]
  (let [f (open-file path)]
    (do
      (defer (close-file f))  ;; 関数終了時に必ず実行
      (let [data (io/read-file f)]
        (transform data)))))

;; 実用例: ファイル処理
(defn safe-read [path]
  (let [f (open path)]
    (do
      (defer (close f))  ;; 関数終了時に必ず実行
      (read f))))
```

### 複数のdeferの実行順序

複数のdeferはスタック的に実行されます（後入れ先出し: LIFO）。

```qi
(defn complex-operation []
  (do
    (let [conn (open-connection)]
      (defer (close-connection conn)))
    (let [lock (acquire-lock)]
      (defer (release-lock lock)))
    (let [file (open-file "data.txt")]
      (defer (close-file file)))
    ;; 処理...
    ;; 終了時の実行順序: close-file → release-lock → close-connection
    ))

;; シンプルな例
(do
  (defer (log "3"))
  (defer (log "2"))
  (defer (log "1"))
  (work))
;; 実行順: work → "1" → "2" → "3"
```

### エラー時のdefer

エラーが発生しても、deferは必ず実行されます。

```qi
(defn safe-process []
  (do
    (let [res (allocate-resource)]
      (defer (free-resource res)))
    (if (error-condition?)
      (error "something went wrong")  ;; deferは実行される
      (process res))))
```

### 設計哲学

- `finally`よりシンプル - 関数のどこにでも書ける
- 強力 - 複数のdeferを組み合わせられる
- Go言語のdeferと同じ設計
- Lisp的 - 特殊な構文を増やさない

**なぜfinallyがないのか**: `defer`の方が柔軟で、複数のリソース管理が直感的。try-catch-finallyのネストより読みやすい。

---

## 4. error - 回復不能なエラー

**用途**: 致命的なエラー、前提条件の違反

### 基本的な使い方

```qi
;; 致命的エラーはerrorで投げる
(defn critical-init []
  (if (not (io/file-exists? "config.qi"))
    (error "config.qi not found")
    (load-config)))

(defn factorial [n]
  (if (< n 0)
    (error "negative input not allowed")
    (loop [i n acc 1]
      (if (= i 0)
        acc
        (recur (dec i) (* acc i))))))
```

### tryでキャッチ

```qi
;; errorをキャッチして処理
(match (try (factorial -5))
  {:ok result} -> result
  {:error e} -> (log (str "Error: " e)))
```

---

## エラー処理の使い分け

### Result型を使うべきケース

- API呼び出し、HTTPリクエスト
- ファイルI/O、データベースクエリ
- JSON/YAMLパース
- ユーザー入力の検証
- **失敗が予想される処理全般**

### try/catchを使うべきケース

- 予期しないエラーのキャッチ
- サードパーティライブラリの呼び出し
- 複雑な処理のまとめてキャッチ
- **例外的な状況の処理**

### deferを使うべきケース

- ファイルのクローズ
- データベース接続のクローズ
- ロックの解放
- 一時ファイルの削除
- **リソースの解放が必要な場合**

### errorを使うべきケース

- 設定ファイルの不在
- 前提条件の違反
- 不正な引数
- **プログラムが継続できない致命的なエラー**

---

## 実用例

### APIクライアント

```qi
(defn fetch-user [user-id]
  (user-id
   |> (str "https://api.example.com/users/" _)
   |> http/get
   |>? (fn [resp]
         (if (= (get resp "status") 200)
           {:ok (get resp "body")}
           {:error "Failed to fetch"}))
   |>? json/parse
   |>? validate-user))

;; 使用例
(match (fetch-user "123")
  {:ok user} -> (process-user user)
  {:error e} -> (log/error "Failed:" e))
```

### ファイル処理with defer

```qi
(defn process-log-file [path]
  (let [f (io/open path :read)]
    (do
      (defer (io/close f))
      (io/read-lines f
       |> (filter (fn [line] (str/contains? line "ERROR")))
       |> (map parse-log-line)
       |> (take 100)))))
```

### 複雑なエラー処理

```qi
(defn complex-operation [input]
  (match (try
           (input
            |> validate
            |>? parse-data
            |>? transform
            |>? save-to-db))
    {:ok result} -> {:success result}
    {:error e} -> (do
                    (log/error "Operation failed:" e)
                    (send-alert e)
                    {:failure e})))
```
