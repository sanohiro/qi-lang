# 標準ライブラリ - CSV処理（csv/）

**RFC 4180準拠のCSV処理ライブラリ**

すべての関数は `csv/` モジュールに属します。

---

## パース

### csv/parse

CSV文字列をパースして、リストのリスト形式（`[[string]]`）に変換します。

```qi
;; 基本的な使い方
(def csv-text "name,age,city
Alice,30,Tokyo
Bob,25,Osaka")

(csv/parse csv-text)
;=> [["name" "age" "city"]
;    ["Alice" "30" "Tokyo"]
;    ["Bob" "25" "Osaka"]]

;; カスタム区切り文字（TSV）
(def tsv-text "name\tage\tcity
Alice\t30\tTokyo")

(csv/parse tsv-text :delimiter "\t")
;=> [["name" "age" "city"]
;    ["Alice" "30" "Tokyo"]]

;; クォートされたフィールド（RFC 4180準拠）
(def quoted-csv "name,description
\"Alice, Bob\",\"She said \"\"hello\"\"\"")

(csv/parse quoted-csv)
;=> [["name" "description"]
;    ["Alice, Bob" "She said \"hello\""]]
```

**引数**:
- `text` - CSV形式の文字列
- `:delimiter` - オプション。区切り文字（デフォルト: `","`）。単一文字のみ

**戻り値**: リストのリスト（`[[string]]`）

**RFC 4180準拠機能**:
- ダブルクォートでのフィールド囲み
- クォート内の `""` によるエスケープ
- CRLF / LF 改行の両対応
- クォート内の改行、カンマ、ダブルクォートの保持

---

## シリアライズ

### csv/stringify

リストのリスト（またはベクターのベクター）をCSV文字列に変換します。

```qi
;; 基本的な使い方
(def data [
  ["name" "age" "city"]
  ["Alice" 30 "Tokyo"]
  ["Bob" 25 "Osaka"]])

(csv/stringify data)
;=> "name,age,city
;    Alice,30,Tokyo
;    Bob,25,Osaka"

;; 様々な型に対応
(def mixed-data [
  ["string" "number" "float" "bool" "nil"]
  ["text" 123 3.14 true nil]])

(csv/stringify mixed-data)
;=> "string,number,float,bool,nil
;    text,123,3.14,true,"

;; 特殊文字を含むフィールド（自動クォート）
(def special-data [
  ["name" "description"]
  ["Alice, Bob" "She said \"hello\""]])

(csv/stringify special-data)
;=> "name,description
;    \"Alice, Bob\",\"She said \"\"hello\"\"\""
```

**引数**:
- `data` - リストのリスト、またはベクターのベクター

**戻り値**: CSV形式の文字列

**対応する型**:
- `String` - そのまま出力
- `Integer` - 文字列に変換
- `Float` - 文字列に変換
- `Bool` - `"true"` / `"false"`
- `Nil` - 空文字列

**自動クォート**: 以下を含むフィールドは自動的にダブルクォートで囲まれます
- カンマ (`,`)
- ダブルクォート (`"`)
- 改行 (`\n`, `\r`)

---

## ファイル読み込み

### csv/read-file

CSVファイルを読み込んでパースします。

```qi
;; ファイルから読み込み
(def data (csv/read-file "users.csv"))

;; ヘッダーとデータを分離
(def headers (first data))
(def rows (rest data))

(println f"カラム: {(join \", \" headers)}")
(println f"行数: {(len rows)}")
```

**引数**:
- `path` - CSVファイルのパス（文字列）

**戻り値**: リストのリスト（`[[string]]`）

**エラー**: ファイルが存在しない、または読み込めない場合はエラー

---

### csv/read-stream

大きなCSVファイルをストリームとして読み込みます。メモリ効率的に1行ずつ処理できます。

```qi
;; 大きなファイルをストリームで処理
(def stream (csv/read-stream "large-data.csv"))

;; ヘッダーをスキップして処理
(def header (stream/next stream))
(println f"カラム: {(join \", \" header)}")

;; 行ごとに処理
(stream/for-each stream
  (fn [row]
    (println f"処理中: {(first row)}")))

;; パイプラインで処理
(csv/read-stream "sales.csv")
 |> (stream/drop 1)  ;; ヘッダースキップ
 |> (stream/filter (fn [row] (> (parse-int (nth row 2)) 1000)))
 |> (stream/take 10)
 |> stream/to-list
```

**引数**:
- `path` - CSVファイルのパス（文字列）

**戻り値**: ストリーム（各要素はリスト）

**使用例**:
- 大容量CSVファイルの処理（数十MB〜数GB）
- メモリを節約しながらの行ごと処理
- パイプラインによるフィルタリング・変換

---

## ファイル書き込み

### csv/write-file

データをCSV形式でファイルに書き込みます。パイプライン対応。

```qi
;; 直接書き込み
(def data [
  ["name" "age" "city"]
  ["Alice" 30 "Tokyo"]
  ["Bob" 25 "Osaka"]])

(csv/write-file data "output.csv")

;; パイプラインで使用
(data
 |> (map (fn [row] (map str/upper row)))  ;; 全て大文字化
 |> (csv/write-file _ "output-upper.csv"))

;; データ変換後に保存
(csv/read-file "input.csv")
 |> (filter (fn [row] (!= (first row) "name")))  ;; ヘッダー除外
 |> (filter (fn [row] (> (parse-int (nth row 1)) 25)))  ;; age > 25
 |> (csv/write-file _ "filtered.csv")
```

**引数**:
- `data` - リストのリスト、またはベクターのベクター
- `path` - 出力先ファイルパス（文字列）

**戻り値**: `nil`

**機能**: `csv/stringify` + `io/write-file` の便利関数

---

## 実用例

### CSVファイルの読み込みと変換

```qi
;; CSVを読み込んで、マップのリストに変換
(defn csv->maps [csv-data]
  (let [headers (first csv-data)
        rows (rest csv-data)]
    (map (fn [row]
           (zipmap headers row))
         rows)))

(def users-csv (csv/read-file "users.csv"))
(def users (csv->maps users-csv))
;=> [{:name "Alice" :age "30" :city "Tokyo"}
;    {:name "Bob" :age "25" :city "Osaka"}]

;; フィルタリング
(def tokyo-users
  (filter (fn [u] (= (get u :city) "Tokyo"))
          users))
```

### データクリーニングパイプライン

```qi
;; CSVデータのクリーニング
(csv/read-file "raw-data.csv")
 |> (map (fn [row]
           (map str/trim row)))  ;; 余分な空白除去
 |> (filter (fn [row]
              (not (str/blank? (first row)))))  ;; 空行除外
 |> (fn [data]
      (let [headers (first data)
            rows (rest data)]
        (cons headers
              (distinct rows))))  ;; 重複行除去
 |> (csv/write-file _ "cleaned-data.csv")
```

### 大容量ファイルの集計

```qi
;; 100万行のCSVファイルから条件に合う行をカウント
(defn count-high-sales [filepath threshold]
  (let [stream (csv/read-stream filepath)
        _ (stream/next stream)]  ;; ヘッダースキップ
    (stream
     |> (stream/filter (fn [row]
                         (> (parse-float (nth row 2)) threshold)))
     |> stream/count)))

(println f"高額取引: {(count-high-sales \"sales.csv\" 10000)} 件")
```

### TSV形式の処理

```qi
;; タブ区切り形式（TSV）
(def tsv-text (io/read-file "data.tsv"))
(def data (csv/parse tsv-text :delimiter "\t"))

;; CSVに変換して保存
(csv/write-file data "data.csv")
```

### データの結合とエクスポート

```qi
;; 複数のCSVファイルを結合
(defn merge-csv-files [files output]
  (let [headers (-> files
                    first
                    csv/read-file
                    first)
        all-rows (files
                  |> (mapcat (fn [f]
                               (-> f
                                   csv/read-file
                                   rest))))]  ;; 各ファイルのヘッダー除外
    (cons headers all-rows)
     |> (csv/write-file _ output)))

(merge-csv-files ["jan.csv" "feb.csv" "mar.csv"] "q1-sales.csv")
```

### データベースへのインポート

```qi
;; CSVデータをデータベースに一括挿入
(defn import-csv-to-db [csv-file table-name conn]
  (let [data (csv/read-file csv-file)
        headers (first data)
        rows (rest data)]
    (doseq [row rows]
      (let [values (zipmap (map keyword headers) row)
            sql (str/format "INSERT INTO {} VALUES ({})"
                           table-name
                           (str/join ", " (repeat (len row) "?")))]
        (db/execute conn sql (vals values))))))

;; ストリームを使ったメモリ効率的なインポート
(defn stream-import-csv-to-db [csv-file table-name conn]
  (let [stream (csv/read-stream csv-file)
        headers (stream/next stream)]  ;; ヘッダー取得
    (stream/for-each stream
      (fn [row]
        (let [values (zipmap (map keyword headers) row)]
          (db/insert conn table-name values))))))
```

---

## エラー処理

```qi
;; ファイル読み込みのエラーハンドリング
(try
  (csv/read-file "data.csv")
  (catch e
    (println f"CSVの読み込みエラー: {e}")
    []))

;; パースエラーのハンドリング
(try
  (csv/parse "invalid\ncsv\ndata")
  (catch e
    (println f"パースエラー: {e}")
    []))

;; 書き込みエラーのハンドリング
(try
  (csv/write-file data "/invalid/path/file.csv")
  (catch e
    (println f"書き込みエラー: {e}")
    nil))
```

---

## パフォーマンスガイド

### 小〜中サイズのファイル（< 10MB）

```qi
;; csv/read-file を使用（シンプル）
(def data (csv/read-file "data.csv"))
(doseq [row data]
  (process-row row))
```

### 大サイズのファイル（> 10MB）

```qi
;; csv/read-stream を使用（メモリ効率的）
(def stream (csv/read-stream "large-data.csv"))
(stream/for-each stream process-row)
```

### 超大サイズのファイル（> 1GB）

```qi
;; ストリーム + パイプライン（並列処理可能）
(csv/read-stream "huge-data.csv")
 |> (stream/drop 1)  ;; ヘッダースキップ
 |> (stream/map parse-row)
 |> (stream/filter valid-row?)
 |> (stream/for-each process-row)
```

---

## 関数一覧

| 関数 | 説明 | 用途 |
|------|------|------|
| `csv/parse` | CSV文字列をパース | テキストからのパース |
| `csv/stringify` | データをCSV文字列に変換 | CSV文字列の生成 |
| `csv/read-file` | CSVファイルを読み込み | 小〜中サイズファイル |
| `csv/read-stream` | CSVファイルをストリーム読み込み | 大サイズファイル |
| `csv/write-file` | データをCSVファイルに書き込み | ファイル出力 |

---

## 仕様準拠

このライブラリは **RFC 4180** に準拠しています:

- ✅ CRLF / LF 改行のサポート
- ✅ ダブルクォートによるフィールド囲み
- ✅ `""` によるダブルクォートのエスケープ
- ✅ クォート内の改行・カンマの保持
- ✅ カスタム区切り文字（TSV等）

---

## 参考

- **RFC 4180**: Common Format and MIME Type for CSV Files
- **関連モジュール**:
  - `io/` - ファイル入出力
  - `str/` - 文字列操作
  - `stream/` - ストリーム処理
  - `table/` - テーブル処理（グループ化・集計）
