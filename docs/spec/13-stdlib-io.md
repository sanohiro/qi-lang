# 標準ライブラリ - ファイルI/O

**ファイル操作とエンコーディング対応**

---

## 基本I/O

### 出力

```qi
;; print - 標準出力（改行なし）
(print "Hello")                     ;; Hello（改行なし）

;; println - 標準出力（改行あり）
(println "Hello")                   ;; Hello\n
```

### ファイル読み込み

```qi
;; io/read-file - ファイル全体を読み込み
(io/read-file "data.txt")           ;; => "file content..."

;; io/read-lines - 行ごとに読み込み（メモリ効率的）
(io/read-lines "data.txt")          ;; => ["line1" "line2" "line3"]
```

### ファイル書き込み

```qi
;; io/write-file - ファイルに書き込み（上書き）
(io/write-file "Hello, Qi!" "/tmp/test.txt")

;; io/append-file - ファイルに追記
(io/append-file "\nSecond line" "/tmp/test.txt")
```

### ファイル確認

```qi
;; io/file-exists? - ファイルの存在確認
(io/file-exists? "/tmp/test.txt")   ;; => true
```

---

## エンコーディング対応

### Unicode

```qi
;; :utf-8 (デフォルト、BOM自動除去)
(io/read-file "data.txt")

;; :utf-8-bom (BOM付きUTF-8、Excel対応)
(io/write-file data "excel.csv" :encoding :utf-8-bom)

;; :utf-16le (UTF-16LE、BOM付き、Excel多言語対応)
(io/write-file data "multilang_excel.csv" :encoding :utf-16le)

;; :utf-16be (UTF-16BE、BOM付き)
(io/write-file data "data.txt" :encoding :utf-16be)
```

### 日本語

```qi
;; :sjis / :shift-jis (Shift_JIS/Windows-31J、日本Windows/Excel)
(io/read-file "legacy.csv" :encoding :sjis)
(io/write-file data "for_excel.csv" :encoding :sjis)

;; :euc-jp (EUC-JP、Unix系)
(io/read-file "unix_text.txt" :encoding :euc-jp)

;; :iso-2022-jp (JIS、メール)
(io/read-file "mail.txt" :encoding :iso-2022-jp)
```

### 中国語

```qi
;; :gbk (GBK、中国本土・シンガポール、簡体字Windows/Excel)
(io/write-file data "china_excel.csv" :encoding :gbk)

;; :gb18030 (GB18030、中国国家規格、GBK上位互換)
(io/write-file data "china_official.txt" :encoding :gb18030)

;; :big5 (Big5、台湾・香港、繁体字Windows/Excel)
(io/write-file data "taiwan_excel.csv" :encoding :big5)
```

### 韓国語

```qi
;; :euc-kr (EUC-KR、韓国Windows/Excel)
(io/write-file data "korea_excel.csv" :encoding :euc-kr)
```

### 欧州

```qi
;; :windows-1252 / :cp1252 / :latin1 (西欧、米国Windows/Excel)
(io/write-file data "europe_excel.csv" :encoding :windows-1252)

;; :windows-1251 / :cp1251 (ロシア・キリル文字圏Windows/Excel)
(io/write-file data "russia_excel.csv" :encoding :windows-1251)
```

### 自動検出

```qi
;; :auto (BOM検出 → UTF-8 → 各地域エンコーディングを順次試行)
(io/read-file "unknown.txt" :encoding :auto)
```

---

## 書き込みオプション

### ファイル存在時の動作

```qi
;; :if-exists オプション
(io/write-file data "out.txt" :if-exists :error)      ;; 存在したらエラー
(io/write-file data "out.txt" :if-exists :skip)       ;; 存在したらスキップ
(io/write-file data "out.txt" :if-exists :append)     ;; 追記
(io/write-file data "out.txt" :if-exists :overwrite)  ;; 上書き（デフォルト）
```

### ディレクトリ自動作成

```qi
;; :create-dirs オプション
(io/write-file data "path/to/out.txt" :create-dirs true)

;; 複数オプション組み合わせ
(io/write-file data "backup/data.csv"
               :encoding :sjis
               :if-exists :error
               :create-dirs true)
```

---

## ファイルシステム操作

### ディレクトリ一覧

```qi
;; io/list-dir - ディレクトリ一覧取得
(io/list-dir ".")                                ;; カレントディレクトリ
(io/list-dir "logs" :pattern "*.log")            ;; ログファイルのみ
(io/list-dir "src" :pattern "*.rs" :recursive true)  ;; 再帰的に検索
```

### ディレクトリ操作

```qi
;; io/create-dir - ディレクトリ作成（親も自動作成）
(io/create-dir "data/backup")

;; io/delete-dir - ディレクトリ削除
(io/delete-dir "temp")                           ;; 空ディレクトリ削除
(io/delete-dir "old_data" :recursive true)       ;; 中身ごと削除
```

### ファイル操作

```qi
;; io/copy-file - ファイルコピー
(io/copy-file "data.txt" "data_backup.txt")

;; io/move-file - ファイル移動・名前変更
(io/move-file "old.txt" "new.txt")

;; io/delete-file - ファイル削除
(io/delete-file "temp.txt")
```

### メタデータ取得

```qi
;; io/file-info - ファイル情報取得
(def info (io/file-info "data.txt"))
(get info "size")                                ;; ファイルサイズ
(get info "modified")                            ;; 更新日時（UNIXタイムスタンプ）
(get info "is-dir")                              ;; ディレクトリか
(get info "is-file")                             ;; ファイルか

;; 判定関数
(io/is-file? "data.txt")                         ;; true
(io/is-dir? "data")                              ;; true
(io/file-exists? "config.json")                  ;; true/false
```

---

## 一時ファイル・ディレクトリ

### 自動削除（推奨）

```qi
;; io/temp-file - 一時ファイル作成（プログラム終了時に自動削除）
(let [tmp (io/temp-file)]
  (io/write-file "temporary data" tmp)
  (process-file tmp))
;; プログラム終了時にtmpは自動的に削除される

;; io/temp-dir - 一時ディレクトリ作成（自動削除）
(let [tmpdir (io/temp-dir)]
  (io/write-file "data1" (path/join tmpdir "file1.txt"))
  (io/write-file "data2" (path/join tmpdir "file2.txt"))
  (process-directory tmpdir))
;; プログラム終了時にtmpdirと中身は自動的に削除される
```

### 手動削除（keep版）

```qi
;; io/temp-file-keep - 永続的な一時ファイルを作成
(let [tmp (io/temp-file-keep)]
  (io/write-file "persistent data" tmp)
  (println f"Created: {tmp}")
  tmp)
;; => "/tmp/.tmpXXXXXX" （削除されない、手動で削除が必要）

;; io/temp-dir-keep - 永続的な一時ディレクトリを作成
(let [tmpdir (io/temp-dir-keep)]
  (io/create-dir (path/join tmpdir "subdir"))
  tmpdir)
;; => "/tmp/.tmpXXXXXX" （削除されない）
```

---

## 実用例

### CSVファイル処理

```qi
;; CSVを読み込んで処理
(io/read-file "data.csv")
  |> (fn [content] (split content "\n"))
  |> (map (fn [line] (split line ",")))
  |> (filter (fn [row] (> (len row) 2)))
```

### パイプラインと組み合わせ

```qi
;; ログファイルを並列処理
("logs"
 |> (io/list-dir :pattern "*.log")
 |> (map io/read-file)
 |> (map process-log)
 |> (reduce merge))
```

### エンコーディング変換

```qi
;; Shift_JIS → UTF-8変換
(io/read-file "legacy.csv" :encoding :sjis)
 |> csv/parse
 |> (map transform)
 |> csv/stringify
 |> (fn [s] (io/write-file s "modern_utf8.csv"))
```

### 各国のExcel対応

```qi
;; 日本: Excel用CSV（Shift_JIS）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "japan_excel.csv" :encoding :sjis)))

;; 中国（簡体字）: Excel用CSV（GBK）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "china_excel.csv" :encoding :gbk)))

;; 台湾・香港（繁体字）: Excel用CSV（Big5）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "taiwan_excel.csv" :encoding :big5)))

;; 多言語混在: UTF-16LE（Excel推奨、BOM付き）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "multilang_excel.csv" :encoding :utf-16le)))
```

### 一時ファイルでのデータ処理

```qi
;; 大きなデータを一時ファイルで処理
(defn process-large-data [url]
  (let [tmp (io/temp-file)]
    ;; データをダウンロードして一時ファイルに保存
    (http/get url :output tmp)
    ;; 一時ファイルを処理
    (let [result (process-file tmp)]
      ;; 関数終了後、tmpは自動削除される
      result)))

;; 複数の一時ファイルを使用
(defn merge-files [files output]
  (let [tmpdir (io/temp-dir)
        processed (files
                   |> (map (fn [f]
                         (let [tmp (path/join tmpdir (path/basename f))]
                           (io/copy-file f tmp)
                           (process-file tmp)
                           tmp))))]
    ;; 処理済みファイルをマージ
    (merge-all processed output)
    ;; 関数終了後、tmpdirと中身は自動削除される
    output))
```

### 安全な書き込み

```qi
;; 既存ファイル保護
(io/write-file data "important.txt"
               :if-exists :error
               :create-dirs true)

;; エンコーディング不明ファイルの処理
(io/read-file "unknown.txt" :encoding :auto)
 |> process
 |> (fn [s] (io/write-file s "output.txt" :encoding :utf-8-bom))
```

---

## 関数一覧

### ファイルI/O
- `io/read-file` - ファイル全体を読み込み
- `io/read-lines` - 行ごと読み込み
- `io/write-file` - ファイルに書き込み（上書き）
- `io/append-file` - ファイルに追記

### ファイルシステム操作
- `io/list-dir` - ディレクトリ一覧取得（グロブパターン対応）
- `io/create-dir` - ディレクトリ作成
- `io/delete-file` - ファイル削除
- `io/delete-dir` - ディレクトリ削除
- `io/copy-file` - ファイルコピー
- `io/move-file` - ファイル移動・名前変更

### メタデータ
- `io/file-info` - ファイル情報取得
- `io/file-exists?` - ファイル存在確認
- `io/is-file?` - ファイル判定
- `io/is-dir?` - ディレクトリ判定

### 一時ファイル
- `io/temp-file` - 一時ファイル作成（自動削除）
- `io/temp-file-keep` - 一時ファイル作成（削除しない）
- `io/temp-dir` - 一時ディレクトリ作成（自動削除）
- `io/temp-dir-keep` - 一時ディレクトリ作成（削除しない）
