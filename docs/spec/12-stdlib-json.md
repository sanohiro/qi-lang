# 標準ライブラリ - JSON/YAML

**JSON・YAML処理**

---

## JSON処理（json/）

### 基本操作

```qi
;; json/parse - JSON文字列をパース
(json/parse "{\"name\":\"Alice\",\"age\":30}")
;; => {:ok {"name" "Alice" "age" 30}}

;; json/stringify - 値をJSON化（コンパクト）
(json/stringify {"name" "Bob" "age" 25})
;; => {:ok "{\"name\":\"Bob\",\"age\":25}"}

;; json/pretty - 値を整形JSON化
(json/pretty {"name" "Bob" "age" 25})
;; => {:ok "{\n  \"name\": \"Bob\",\n  \"age\": 25\n}"}
```

### Result型による安全なパース

すべてのJSON関数は `{:ok value}` または `{:error message}` を返します。

```qi
;; 正常なパース
(match (json/parse "{\"valid\": true}")
  {:ok data} -> data
  {:error e} -> (log e))
;; => {"valid" true}

;; エラーハンドリング
(match (json/parse "{invalid json}")
  {:ok data} -> data
  {:error e} -> (println "Parse error:" e))
;; => "Parse error: ..."
```

### パイプラインでの使用

```qi
;; APIレスポンスをパース→変換→保存
("https://api.example.com/users/123"
 |> http/get
 |>? (fn [resp] {:ok (get resp "body")})
 |>? json/parse
 |>? (fn [data] {:ok (assoc data "processed" true)})
 |>? json/pretty
 |>? (fn [json] {:ok (io/write-file "output.json" json)}))
```

---

## YAML処理（yaml/）

**Pure Rust実装 - serde_yaml使用**

### 基本操作

```qi
;; yaml/parse - YAML文字列をパース
(yaml/parse "name: Alice\nage: 30\ntags:\n  - dev\n  - ops")
;; => {:ok {"name" "Alice" "age" 30 "tags" ["dev" "ops"]}}

;; yaml/stringify - 値をYAML化
(yaml/stringify {"name" "Bob" "age" 25 "tags" ["backend" "devops"]})
;; => {:ok "name: Bob\nage: 25\ntags:\n- backend\n- devops\n"}

;; yaml/pretty - 値を整形YAML化（yaml/stringifyと同じ）
(yaml/pretty {"server" {"host" "localhost" "port" 8080}})
;; => {:ok "server:\n  host: localhost\n  port: 8080\n"}
```

### 設定ファイル処理

```qi
;; 設定ファイルをパースしてポート番号を取得
("config.yaml"
 |> io/read-file
 |> yaml/parse
 |>? (fn [conf] {:ok (get-in conf ["server" "port"])}))
;; => {:ok 8080}

;; データ変換パイプライン（JSON → YAML）
("data.json"
 |> io/read-file
 |> json/parse
 |>? (fn [data] (yaml/stringify (get data "ok")))
 |>? (fn [yaml] {:ok (io/write-file "data.yaml" yaml)}))
```

### YAMLの特徴

- 設定ファイルに最適（JSON/TOMLより読みやすい）
- インデント自動整形
- JSON互換（YAMLはJSONのスーパーセット）
- エラーハンドリング: `{:ok ...}` または `{:error "..."}`

---

## 実用例

### API データの取得と保存

```qi
(defn fetch-and-save [url output-file]
  (url
   |> http/get
   |>? (fn [resp] {:ok (get resp "body")})
   |>? json/parse
   |>? json/pretty
   |>? (fn [json-str] {:ok (io/write-file output-file json-str)})))

(fetch-and-save "https://api.github.com/users/octocat" "user.json")
```

### 設定ファイルの読み込み

```qi
(defn load-config [path]
  (path
   |> io/read-file
   |> yaml/parse
   |>? (fn [config]
         ;; バリデーション
         (if (get config "version")
           {:ok config}
           {:error "Missing version field"}))))

(match (load-config "config.yaml")
  {:ok config} -> (println "Config loaded:" config)
  {:error e} -> (println "Error:" e))
```

### データ変換

```qi
;; JSON → YAML変換
(defn json-to-yaml [input-file output-file]
  (input-file
   |> io/read-file
   |> json/parse
   |>? yaml/stringify
   |>? (fn [yaml-str] {:ok (io/write-file output-file yaml-str)})))

(json-to-yaml "data.json" "data.yaml")

;; YAML → JSON変換
(defn yaml-to-json [input-file output-file]
  (input-file
   |> io/read-file
   |> yaml/parse
   |>? json/stringify
   |>? (fn [json-str] {:ok (io/write-file output-file json-str)})))

(yaml-to-json "config.yaml" "config.json")
```

### バッチ処理

```qi
;; 複数のJSONファイルを並列パース
(def files ["data1.json" "data2.json" "data3.json"])

(files
 ||> io/read-file
 ||> json/parse
 |> (map (fn [result]
           (match result
             {:ok data} -> data
             {:error e} -> nil)))
 |> (filter some?))
```

### エラーハンドリングパターン

```qi
;; パイプラインでのエラー処理
(defn process-json [json-str]
  (match (json/parse json-str)
    {:ok data} -> (do
                    (println "Parsed successfully")
                    {:ok (assoc data "timestamp" (now))})
    {:error e} -> (do
                    (log/error "Parse failed:" e)
                    {:error e})))

;; 複数のパース試行
(defn try-parse-formats [input-str]
  (match (json/parse input-str)
    {:ok data} -> {:ok data}
    {:error _} -> (yaml/parse input-str)))
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
