# 標準ライブラリ - JSON/YAML

**JSON・YAML処理**

---

## JSON処理（json/）

### 基本操作

```qi
;; json/parse - JSON文字列をパース
(json/parse "{\"name\":\"Alice\",\"age\":30}")
;; => {"name" "Alice" "age" 30}

;; json/stringify - 値をJSON化（コンパクト）
(json/stringify {"name" "Bob" "age" 25})
;; => "{\"name\":\"Bob\",\"age\":25}"

;; json/pretty - 値を整形JSON化
(json/pretty {"name" "Bob" "age" 25})
;; => "{\n  \"name\": \"Bob\",\n  \"age\": 25\n}"
```

### Result型による安全なパース

すべてのJSON関数は成功時に値をそのまま返し、失敗時に `{:error message}` を返します。

```qi
;; 正常なパース
(match (json/parse "{\"valid\": true}")
  {:error e} -> (log e)
  data -> data)
;; => {"valid" true}

;; エラーハンドリング
(match (json/parse "{invalid json}")
  {:error e} -> (println "Parse error:" e)
  data -> data)
;; => "Parse error: ..."
```

### パイプラインでの使用

```qi
;; APIレスポンスをパース→変換→保存（HTTPは例外を投げるのでtryでキャッチ）
(match (try
         ("https://api.example.com/users/123"
          |> http/get
          |> (fn [resp] (get resp "body"))
          |>? json/parse
          |>? (fn [data] (assoc data "processed" true))
          |>? json/pretty
          |>? (fn [json] (io/write-file "output.json" json))))
  {:error e} -> (log/error "Failed:" e)
  result -> result)
```

---

## YAML処理（yaml/）

**Pure Rust実装 - serde_yaml使用**

### 基本操作

```qi
;; yaml/parse - YAML文字列をパース
(yaml/parse "name: Alice\nage: 30\ntags:\n  - dev\n  - ops")
;; => {"name" "Alice" "age" 30 "tags" ["dev" "ops"]}

;; yaml/stringify - 値をYAML化
(yaml/stringify {"name" "Bob" "age" 25 "tags" ["backend" "devops"]})
;; => "name: Bob\nage: 25\ntags:\n- backend\n- devops\n"

;; yaml/pretty - 値を整形YAML化（yaml/stringifyと同じ）
(yaml/pretty {"server" {"host" "localhost" "port" 8080}})
;; => "server:\n  host: localhost\n  port: 8080\n"
```

### 設定ファイル処理

```qi
;; 設定ファイルをパースしてポート番号を取得（I/Oは例外を投げるのでtryでキャッチ）
(match (try
         ("config.yaml"
          |> io/read-file
          |>? yaml/parse
          |>? (fn [conf] (get-in conf ["server" "port"]))))
  {:error e} -> (log/error "Failed:" e)
  port -> port)
;; => 8080

;; データ変換パイプライン（JSON → YAML）
(match (try
         ("data.json"
          |> io/read-file
          |>? json/parse
          |>? yaml/stringify
          |>? (fn [yaml] (io/write-file "data.yaml" yaml))))
  {:error e} -> (log/error "Failed:" e)
  result -> result)
```

### YAMLの特徴

- 設定ファイルに最適（JSON/TOMLより読みやすい）
- インデント自動整形
- JSON互換（YAMLはJSONのスーパーセット）
- エラーハンドリング: 成功時は値をそのまま返し、失敗時に `{:error "..."}`

---

## 実用例

### API データの取得と保存

```qi
(defn fetch-and-save [url output-file]
  (match (try
           (url
            |> http/get
            |> (fn [resp] (get resp "body"))
            |>? json/parse
            |>? json/pretty
            |>? (fn [json-str] (io/write-file output-file json-str))))
    {:error e} -> (log/error "Failed:" e)
    result -> result))

(fetch-and-save "https://api.github.com/users/octocat" "user.json")
```

### 設定ファイルの読み込み

```qi
(defn load-config [path]
  (match (try
           (path
            |> io/read-file
            |>? yaml/parse
            |>? (fn [config]
                  ;; バリデーション
                  (if (get config "version")
                    config
                    {:error "Missing version field"}))))
    {:error e} -> {:error e}
    config -> config))

(match (load-config "config.yaml")
  {:error e} -> (println "Error:" e)
  config -> (println "Config loaded:" config))
```

### データ変換

```qi
;; JSON → YAML変換
(defn json-to-yaml [input-file output-file]
  (match (try
           (input-file
            |> io/read-file
            |>? json/parse
            |>? yaml/stringify
            |>? (fn [yaml-str] (io/write-file output-file yaml-str))))
    {:error e} -> (log/error "Failed:" e)
    result -> result))

(json-to-yaml "data.json" "data.yaml")

;; YAML → JSON変換
(defn yaml-to-json [input-file output-file]
  (match (try
           (input-file
            |> io/read-file
            |>? yaml/parse
            |>? json/stringify
            |>? (fn [json-str] (io/write-file output-file json-str))))
    {:error e} -> (log/error "Failed:" e)
    result -> result))

(yaml-to-json "config.yaml" "config.json")
```

### バッチ処理

```qi
;; 複数のJSONファイルを並列パース
(def files ["data1.json" "data2.json" "data3.json"])

(files
 ||> (fn [f] (try (io/read-file f)))
 ||> (fn [content]
       (match content
         {:error e} -> {:error e}
         c -> (json/parse c)))
 |> (filter (fn [result] (not (error? result)))))
```

### エラーハンドリングパターン

```qi
;; パイプラインでのエラー処理
(defn process-json [json-str]
  (match (json/parse json-str)
    {:error e} -> (do
                    (log/error "Parse failed:" e)
                    {:error e})
    data -> (do
              (println "Parsed successfully")
              (assoc data "timestamp" (now)))))

;; 複数のパース試行
(defn try-parse-formats [input-str]
  (match (json/parse input-str)
    {:error _} -> (yaml/parse input-str)
    data -> data))
```

---

## 型マッピング

### Qi → JSON/YAML

| Qi型 | JSON | YAML |
|------|------|------|
| `nil` | `null` | `null` |
| `true/false` | `true/false` | `true/false` |
| 整数・浮動小数点数 | 数値 | 数値 |
| 文字列 | 文字列 | 文字列 |
| ベクター・リスト | 配列 | リスト |
| マップ | オブジェクト | マップ |
| キーワード | 文字列 | 文字列 |

### JSON/YAML → Qi

| JSON/YAML | Qi型 |
|-----------|------|
| `null` | `nil` |
| `true/false` | `true/false` |
| 数値 | 整数 or 浮動小数点数 |
| 文字列 | 文字列 |
| 配列 | ベクター |
| オブジェクト/マップ | マップ |
