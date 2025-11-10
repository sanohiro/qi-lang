# 標準ライブラリ - ZIP圧縮・解凍（zip/）

**ZIP/gzip圧縮・解凍ライブラリ**

ZIP形式によるアーカイブ作成・展開、およびgzipによる単一ファイル圧縮をサポート。

> **Feature Gate**: このモジュールは `util-zip` featureでコンパイルされます。

---

## 基本操作

### ZIP作成

```qi
;; zip/create - ZIPファイルを作成
(zip/create "backup.zip" "data.txt" "config.json")
;; => "backup.zip"

;; 複数ファイルを圧縮
(zip/create "archive.zip" "file1.txt" "file2.txt" "file3.txt")

;; ディレクトリを丸ごと圧縮
(zip/create "logs.zip" "logs/")
;; => logsディレクトリ内のすべてのファイルが再帰的に追加される

;; 複数のファイルとディレクトリを混在
(zip/create "backup.zip" "README.md" "src/" "config/")
```

### ZIP解凍

```qi
;; zip/extract - ZIPファイルを解凍（カレントディレクトリに展開）
(zip/extract "backup.zip")
;; => "."

;; 展開先ディレクトリを指定
(zip/extract "archive.zip" "output/")
;; => "output/"

;; パイプラインでの使用
("backup.zip" |> zip/extract)
```

### ZIP内容確認

```qi
;; zip/list - ZIP内容一覧を取得
(zip/list "backup.zip")
;; => [
;;      {:name "data.txt" :size 1234 :compressed-size 567 :is-dir false}
;;      {:name "config.json" :size 456 :compressed-size 123 :is-dir false}
;;      {:name "logs/" :size 0 :compressed-size 0 :is-dir true}
;;    ]

;; ファイル名だけ取得
(zip/list "backup.zip"
 |> (map (fn [entry] (get entry "name"))))
;; => ["data.txt" "config.json" "logs/"]

;; 圧縮率を計算
(zip/list "backup.zip"
 |> (map (fn [entry]
           (let [original (get entry "size")
                 compressed (get entry "compressed-size")
                 ratio (if (> original 0)
                         (* 100 (/ compressed original))
                         0)]
             {:name (get entry "name")
              :ratio ratio}))))
```

### 既存ZIPに追加

```qi
;; zip/add - 既存ZIPにファイルを追加
(zip/add "backup.zip" "new-file.txt")
;; => "backup.zip"

;; 複数ファイルを追加
(zip/add "backup.zip" "file1.txt" "file2.txt" "dir/")

;; パイプラインでの使用
("backup.zip" |> (zip/add _ "update.txt"))
```

---

## gzip圧縮

### 基本操作

```qi
;; zip/gzip - ファイルをgzip圧縮
(zip/gzip "data.txt")
;; => "data.txt.gz" （自動的に.gz拡張子が付く）

;; 出力ファイル名を指定
(zip/gzip "data.txt" "backup.gz")
;; => "backup.gz"

;; zip/gunzip - gzipファイルを解凍
(zip/gunzip "data.txt.gz")
;; => "data.txt" （自動的に.gz拡張子が除去される）

;; 出力ファイル名を指定
(zip/gunzip "backup.gz" "restored.txt")
;; => "restored.txt"
```

### パイプラインでの使用

```qi
;; gzip圧縮パイプライン
("data.txt" |> zip/gzip)
;; => "data.txt.gz"

;; gzip解凍パイプライン
("data.txt.gz" |> zip/gunzip)
;; => "data.txt"

;; 圧縮・処理・解凍の連鎖
("large-data.txt"
 |> zip/gzip
 |> upload-to-server
 |> download-from-server
 |> zip/gunzip)
```

---

## 実用例

### バックアップシステム

```qi
;; 日次バックアップ
(defn create-backup [date]
  (let [backup-name f"backup-{date}.zip"]
    (zip/create backup-name "data/" "config/" "logs/")
    (println f"Backup created: {backup-name}")))

(create-backup "2024-11-10")
;; => "Backup created: backup-2024-11-10.zip"

;; 複数ディレクトリを個別にバックアップ
(def dirs ["data" "config" "logs"])

(dirs
 |> (map (fn [dir]
           (let [zip-file f"{dir}-backup.zip"]
             (zip/create zip-file f"{dir}/")
             {:dir dir :file zip-file})))
 |> (map println))
```

### ログファイルのローテーション

```qi
;; 古いログを圧縮してアーカイブ
(defn archive-old-logs [days]
  (let [cutoff (- (time/now) (* days 86400))]
    (io/list-dir "logs" :pattern "*.log")
    |> (filter (fn [file]
                 (let [info (io/file-info file)
                       modified (get info "modified")]
                   (< modified cutoff))))
    |> (map (fn [file]
              ;; gzip圧縮
              (let [gz (zip/gzip file)]
                ;; 元ファイル削除
                (io/delete-file file)
                gz)))))

;; 7日以前のログを圧縮
(archive-old-logs 7)
```

### 配布用アーカイブの作成

```qi
;; リリース用ZIPファイルを作成
(defn create-release [version]
  (let [archive-name f"myapp-{version}.zip"
        files ["README.md"
               "LICENSE"
               "bin/"
               "lib/"
               "docs/"]]
    ;; 一時ディレクトリに必要なファイルをコピー
    (let [tmpdir (io/temp-dir)
          dest-dir (path/join tmpdir f"myapp-{version}")]
      (io/create-dir dest-dir)

      ;; ファイルをコピー
      (files
       |> (map (fn [file]
                 (let [dest (path/join dest-dir (path/basename file))]
                   (if (io/is-dir? file)
                     (copy-dir-recursive file dest)
                     (io/copy-file file dest))))))

      ;; ZIPを作成
      (zip/create archive-name dest-dir)
      archive-name)))

(create-release "1.0.0")
;; => "myapp-1.0.0.zip"
```

### データのダウンロード・展開

```qi
;; Webからアーカイブをダウンロードして展開
(defn download-and-extract [url dest-dir]
  (let [tmpfile (io/temp-file)]
    ;; ダウンロード
    (http/get url :output tmpfile)

    ;; 展開
    (zip/extract tmpfile dest-dir)

    ;; 一時ファイルは自動削除される
    (println f"Extracted to: {dest-dir}")))

(download-and-extract "https://example.com/data.zip" "data/")
```

### 大量ファイルの処理

```qi
;; ディレクトリ内のファイルを個別に圧縮
(defn compress-all-files [dir]
  (io/list-dir dir :pattern "*.*")
  |> (filter io/is-file?)
  |> (map zip/gzip)
  |> (map println))

(compress-all-files "documents/")

;; 並列圧縮（大量ファイルを高速処理）
(defn compress-all-parallel [dir]
  (io/list-dir dir :pattern "*.*")
  |> (filter io/is-file?)
  ||> zip/gzip
  |> (map println))

(compress-all-parallel "logs/")
```

### バックアップの検証

```qi
;; ZIPファイルの整合性チェック
(defn verify-backup [zip-file expected-files]
  (let [entries (zip/list zip-file)
        names (entries |> (map (fn [e] (get e "name"))))]
    (expected-files
     |> (map (fn [file]
               (let [found (names |> (filter (fn [n] (str/contains? n file))))]
                 {:file file
                  :found (not (empty? found))})))
     |> (filter (fn [result] (not (get result "found")))))))

(def expected ["data.txt" "config.json" "logs/"])
(let [missing (verify-backup "backup.zip" expected)]
  (if (empty? missing)
    (println "Backup verified: all files present")
    (do
      (println "Missing files:")
      (missing |> (map (fn [m] (get m "file"))) |> (map println)))))
```

---

## エラーハンドリング

### ファイルが見つからない

```qi
;; try-catchでエラーを捕捉
(try
  (zip/create "backup.zip" "nonexistent.txt")
  (catch e
    (println f"Error: {e}")))
;; => "Error: zip/create: path does not exist: 'nonexistent.txt'"

;; 事前チェック
(defn safe-create-zip [zip-file files]
  (let [missing (files |> (filter (fn [f] (not (io/file-exists? f)))))]
    (if (empty? missing)
      (zip/create zip-file ..files)
      (do
        (println "Missing files:")
        (missing |> (map println))
        :error))))

(safe-create-zip "backup.zip" ["file1.txt" "file2.txt"])
```

### 展開先の保護

```qi
;; 展開先ディレクトリが存在する場合はスキップ
(defn safe-extract [zip-file dest-dir]
  (if (io/file-exists? dest-dir)
    (do
      (println f"Destination already exists: {dest-dir}")
      :skip)
    (zip/extract zip-file dest-dir)))

;; 上書き確認
(defn extract-with-confirm [zip-file dest-dir]
  (if (io/file-exists? dest-dir)
    (do
      (print f"Overwrite {dest-dir}? (y/n): ")
      (let [answer (io/stdin-line)]
        (if (= answer "y")
          (do
            (io/delete-dir dest-dir :recursive true)
            (zip/extract zip-file dest-dir))
          (println "Cancelled"))))
    (zip/extract zip-file dest-dir)))
```

### ディスク容量チェック

```qi
;; 展開前に容量を確認
(defn check-space-before-extract [zip-file dest-dir]
  (let [entries (zip/list zip-file)
        total-size (entries
                    |> (map (fn [e] (get e "size")))
                    |> (reduce + 0))]
    (println f"Archive size: {total-size} bytes")
    ;; TODO: ディスク空き容量チェック
    (zip/extract zip-file dest-dir)))
```

---

## パイプライン統合

### ファイル処理パイプライン

```qi
;; ファイルを処理してZIPに追加
(io/list-dir "data" :pattern "*.txt")
|> (map io/read-file)
|> (map process-text)
|> (map-indexed (fn [i content]
                  (let [tmpfile f"/tmp/processed-{i}.txt"]
                    (io/write-file content tmpfile)
                    tmpfile)))
|> (fn [files]
     (zip/create "processed.zip" ..files))
```

### ダウンロード・圧縮パイプライン

```qi
;; 複数URLからダウンロードして圧縮
(def urls ["https://example.com/file1.txt"
           "https://example.com/file2.txt"])

(urls
 ||> (fn [url]
       (let [tmpfile (io/temp-file)]
         (http/get url :output tmpfile)
         tmpfile))
 |> (fn [files]
      (zip/create "downloads.zip" ..files)))
```

### 圧縮率比較

```qi
;; 各ファイルの圧縮率をレポート
(defn compression-report [zip-file]
  (zip/list zip-file)
  |> (filter (fn [e] (not (get e "is-dir"))))
  |> (map (fn [entry]
            (let [name (get entry "name")
                  original (get entry "size")
                  compressed (get entry "compressed-size")
                  ratio (if (> original 0)
                          (- 100 (* 100 (/ compressed original)))
                          0)]
              {:name name
               :original original
               :compressed compressed
               :saved (- original compressed)
               :ratio ratio})))
  |> (map (fn [r]
            (println f"{(get r \"name\")}: {(get r \"ratio\")}% saved"))))

(compression-report "backup.zip")
;; => data.txt: 54% saved
;; => config.json: 32% saved
;; => image.png: 2% saved
```

---

## パフォーマンス考慮事項

### 大量ファイルの処理

```qi
;; 並列処理で高速化（ただし、zip/createは順次実行される）
(defn create-zip-fast [zip-file files]
  ;; ファイルの存在確認を並列実行
  (let [valid (files ||> io/file-exists? |> (filter identity))]
    (zip/create zip-file ..valid)))

;; 大きなディレクトリを分割して圧縮
(defn split-archive [dir max-files]
  (let [all-files (io/list-dir dir :recursive true)
        chunks (all-files |> (partition max-files))]
    (chunks
     |> (map-indexed (fn [i files]
                       (let [zip-file f"archive-part{i}.zip"]
                         (zip/create zip-file ..files)
                         zip-file))))))

(split-archive "large-dir" 1000)
;; => ["archive-part0.zip" "archive-part1.zip" ...]
```

### メモリ効率

```qi
;; 大きなファイルはgzipで個別圧縮（ZIPよりメモリ効率的）
(defn compress-large-files [pattern]
  (io/list-dir "." :pattern pattern)
  |> (filter (fn [f]
               (let [info (io/file-info f)]
                 (> (get info "size") 10000000)))) ;; 10MB以上
  |> (map zip/gzip)
  |> (map println))

(compress-large-files "*.log")
```

### 圧縮レベルの調整

現在の実装では `CompressionMethod::Deflated` のデフォルトレベルを使用しています。
将来的には圧縮レベル指定のサポートが追加される可能性があります。

```qi
;; 将来的な機能（未実装）
;; (zip/create "backup.zip" "data/" :compression :fast)
;; (zip/create "backup.zip" "data/" :compression :best)
```

---

## 関数一覧

### ZIP操作
- `zip/create` - ZIPファイルを作成（複数ファイル・ディレクトリ対応）
- `zip/extract` - ZIPファイルを解凍（展開先指定可能）
- `zip/list` - ZIP内容一覧を取得（サイズ・圧縮率情報含む）
- `zip/add` - 既存ZIPにファイルを追加

### gzip操作
- `zip/gzip` - ファイルをgzip圧縮（出力ファイル名指定可能）
- `zip/gunzip` - gzipファイルを解凍（出力ファイル名指定可能）

---

## ZIPとgzipの使い分け

### ZIP形式を使うべき場合
- 複数ファイルをまとめたい
- ディレクトリ構造を保持したい
- 個別ファイルの取り出しが必要
- Windows/Macでダブルクリックで開きたい
- 配布用アーカイブを作りたい

### gzip形式を使うべき場合
- 単一ファイルの圧縮
- Unix/Linuxでの標準的な圧縮
- ストリーミング圧縮が必要
- tarとの組み合わせ（tar.gz）
- ログファイルのローテーション

---

## よくあるパターン

### バックアップ・リストア

```qi
;; バックアップ作成
(defn backup [name]
  (let [timestamp (time/format (time/now) "%Y%m%d-%H%M%S")
        zip-file f"backup-{name}-{timestamp}.zip"]
    (zip/create zip-file "data/" "config/")
    (println f"Backup created: {zip-file}")
    zip-file))

;; リストア
(defn restore [zip-file]
  (let [tmpdir (io/temp-dir)]
    (zip/extract zip-file tmpdir)
    ;; 検証後に本番環境へコピー
    (println "Restored to temporary directory")
    tmpdir))
```

### データ配布

```qi
;; リリースパッケージ作成
(defn package-release [version files]
  (let [release-name f"release-v{version}.zip"]
    (zip/create release-name ..files)
    (println f"Release package: {release-name}")
    release-name))

;; 配布用に複数形式を作成
(defn create-distributions [version]
  {:zip (zip/create f"dist-{version}.zip" "dist/")
   :tar-gz (do
             (zip/create f"dist-{version}.tar" "dist/")
             (zip/gzip f"dist-{version}.tar"))})
```

### ログ管理

```qi
;; 日次ログアーカイブ
(defn archive-daily-logs []
  (let [yesterday (time/format (- (time/now) 86400) "%Y-%m-%d")
        log-files (io/list-dir "logs" :pattern f"*{yesterday}*.log")]
    (when (not (empty? log-files))
      (let [archive f"logs-{yesterday}.zip"]
        (zip/create archive ..log-files)
        ;; 元ファイル削除
        (log-files |> (map io/delete-file))
        archive))))
```
