# エラー処理

**Qiのエラー処理戦略**

Qiは用途に応じて3つのエラー処理方法を提供します：

1. **Result型 (`{:ok/:error}`)** - 回復可能なエラー、Railway Pipeline
2. **try/catch** - 例外のキャッチとリカバリ
3. **defer** - リソース解放の保証（`finally`の代替）

---

## 1. Result型 - Railway Pipeline（推奨パターン）

**用途**: API、ファイルIO、パース等の失敗が予想される処理

### 新仕様：`{:error}`以外は全て成功

**`{:error}`以外は全て成功扱い！`:ok`ラップなし**

```qi
;; シンプル！普通の値を返すだけ
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}  ;; エラーだけ明示的に
    (/ x y)))                     ;; 普通の値 → 成功

;; matchで処理（必要なら）
(match (divide 10 2)
  {:error e} -> (log e)
  result -> result)

;; error?述語で判定（シンプル）
(def result (divide 10 2))
(if (error? result)
  (log "エラーが発生しました")
  (process result))

;; パイプラインでのエラー処理（詳細は02-flow-pipes.mdを参照）
(user-input
 |> validate
 |>? parse-number      ;; 普通の値を返すだけでOK
 |>? (fn [n] (divide 100 n))
 |>? format-result)
;; 成功時 => 結果の値、エラー時 => {:error ...}
```

### Railway Pipelineとの統合

`|>?` 演算子を使うことで、エラーハンドリングをパイプラインに統合できます。

```qi
;; HTTPリクエスト + エラーハンドリング（シンプル！）
("https://api.example.com/users/123"
 |> http/get                      ;; => {:status 200 :body "..."}
 |>? (fn [resp] (get resp :body))  ;; 値を返すだけ！
 |>? json/parse                   ;; => パース結果（値そのまま）
 |>? (fn [data] (get data "user")))  ;; 値を返すだけ！
;; 成功時 => ユーザーデータ、エラー時 => {:error ...}

;; JSONパース + データ変換
("{\"name\":\"Alice\",\"age\":30}"
 |> json/parse                    ;; => パース結果（値そのまま）
 |>? (fn [data] (get data "name"))  ;; 値を返すだけ！
 |>? str/upper)                   ;; 関数を直接渡すだけ！
;; => "ALICE"
```

### 動作ルール

**入力値の処理**:
1. `{:error ...}` → ショートサーキット
2. `{:ok value}` → `value`を取り出して関数に渡す（後方互換性）
3. その他 → そのまま関数に渡す

**出力値の処理**:
1. `{:error ...}` → そのまま返す（エラー伝播）
2. その他 → **そのまま返す**（`:ok`ラップなし！）

### 設計哲学

エラーをデータとして扱い、パイプラインの中で流す。try-catchのネストを避け、データフローが明確になる。`{:error}`以外は全て成功として扱い、Lispの「nil以外は真」と同じ哲学でシンプルに。

---

## 2. try/catch - 例外処理

**用途**: 予期しないエラーのキャッチ、サードパーティコードの呼び出し

### 基本的な使い方

```qi
;; try-catchブロック
(match (try (risky-operation))
  {:error e} -> (handle-error e)
  result -> result)

;; ネスト可能
(match (try
         (let [data (parse-data input)]
           (process data)))
  {:error e} -> {:error (str "Failed: " e)}
  result -> result)
```

### 実用例

```qi
;; ファイル読み込みのエラー処理
(match (try (io/read-file "config.json"))
  {:error e} -> (do
                  (log/error "Failed to read config:" e)
                  {:error e})
  content -> (json/parse content))

;; パイプラインでの使用
(match (try
         (url
          |> http-get
          |> parse
          |> process))
  {:error e} -> []
  data -> data)
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
  {:error e} -> (log (str "Error: " e))
  result -> result)
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

---

## 標準ライブラリの返却形式

Qiの組み込み関数は、エラーの性質に応じて異なる形式を返します。

### Result型を返す関数（値 / `{:error ...}`）

**データフォーマット系** - パースエラーは予期されるため、明示的なハンドリングが必要：

- `json/parse`, `json/stringify`, `json/pretty` - JSON処理
- `yaml/parse`, `yaml/stringify` - YAML処理
- `csv/parse`, `csv/stringify` - CSV処理（予定）

```qi
;; パースエラーを明示的に処理
(match (json/parse user-input)
  {:error msg} -> (show-error-to-user msg)
  data -> (process data))

;; パイプラインでの使用
(user-input
 |> json/parse
 |>? (fn [data] (get data "name"))
 |>? str/upper)
```

### 例外を投げる関数（`Ok(value)` / `Err(message)`）

**I/O・ネットワーク系** - 失敗は例外的な状況として扱う：

- `http/get`, `http/post`, `http/put`, `http/delete` - HTTP操作
- `io/read-file`, `io/write-file` - ファイルI/O
- `io/open`, `io/close` - ファイル操作
- `db/*` - データベース操作（予定）

```qi
;; 失敗時は例外として伝播
(def content (io/read-file "config.json"))

;; tryでキャッチして処理
(match (try (http/get "https://api.example.com/data"))
  {:error e} -> (log-error e)
  response -> (process response))
```

### 設計方針

この区別は以下の理由に基づきます：

1. **データフォーマット系は値/`{:error}`を返す**
   - パースエラーは**予期される失敗**（不正なJSON文字列など）
   - ユーザー入力の検証など、エラーケースが正常なフロー
   - matchやパイプライン（`|>?`）で明示的にハンドリング
   - 成功時は値を直接返す（`:ok`ラップなし）

2. **I/O・ネットワーク系は例外**
   - ファイルが存在しない、ネットワークエラーは**例外的な状況**
   - 正常系のコードを簡潔に保つ
   - 必要に応じてtry/catchでキャッチ

この方針により、ユーザーは「予期されるエラー」と「例外的な状況」を自然に区別できます。

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
           (get resp "body")
           {:error "Failed to fetch"}))
   |>? json/parse
   |>? validate-user))

;; 使用例
(match (fetch-user "123")
  {:error e} -> (log/error "Failed:" e)
  user -> (process-user user))
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
    {:error e} -> (do
                    (log/error "Operation failed:" e)
                    (send-alert e)
                    {:failure e})
    result -> {:success result}))
```
