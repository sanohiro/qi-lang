# 標準ライブラリ - 一時ファイル・ディレクトリ（io/temp）

**安全な一時リソース管理**

一時ファイルと一時ディレクトリの作成・管理機能を提供します。自動削除機能により、リソースリークを防ぎます。

---

## 概要

一時ファイル・ディレクトリは以下の用途で使用されます：

- **テストデータの一時保存** - ユニットテスト、統合テスト
- **中間ファイルの生成** - データ変換パイプライン、ビルドプロセス
- **キャッシュファイル** - 外部API結果、計算結果の一時保存
- **ダウンロードファイル** - HTTP経由で取得したファイルの一時保存
- **作業ディレクトリ** - 複数ファイルを伴う処理の作業領域

すべての一時リソースは、システムの一時ディレクトリ（`/tmp` 等）に作成されます。

---

## 一時ファイル

### 自動削除（推奨）

```qi
;; io/temp-file - 一時ファイルを作成（プログラム終了時に自動削除）
(let [tmp (io/temp-file)]
  (io/write-file "temporary data" tmp)
  (println f"Created temp file: {tmp}")
  (process-file tmp))
;; プログラム終了時にファイルは自動的に削除される

;; 使用例: データのダウンロードと処理
(defn download-and-process [url]
  (let [tmp (io/temp-file)]
    ;; URLからデータをダウンロード
    (http/get url :output tmp)
    ;; 一時ファイルを処理
    (let [result (io/read-file tmp |> parse-data)]
      ;; 関数終了後、tmpは自動削除される
      result)))

;; 使用例: 複数の一時ファイル
(defn process-with-temp []
  (let [tmp1 (io/temp-file)
        tmp2 (io/temp-file)]
    (io/write-file "data 1" tmp1)
    (io/write-file "data 2" tmp2)
    (merge-files tmp1 tmp2)))
;; 両方のファイルが自動削除される
```

### 手動削除（永続化）

```qi
;; io/temp-file-keep - 一時ファイルを作成（削除しない）
(let [tmp (io/temp-file-keep)]
  (io/write-file "persistent data" tmp)
  (println f"Created: {tmp}")
  tmp)
;; => "/tmp/qi-12345.tmp" （削除されない、手動削除が必要）

;; 使用例: ユーザーに結果ファイルを提供
(defn export-to-temp [data]
  (let [tmp (io/temp-file-keep)]
    (io/write-file (json/stringify data) tmp :encoding :utf-8-bom)
    (println f"Export completed: {tmp}")
    (println "Please move or delete this file when done.")
    tmp))

;; 後で手動削除
(io/delete-file tmp)
```

---

## 一時ディレクトリ

### 自動削除（推奨）

```qi
;; io/temp-dir - 一時ディレクトリを作成（プログラム終了時に自動削除）
(let [tmpdir (io/temp-dir)]
  (println f"Temp directory: {tmpdir}")
  (io/write-file "data1" (path/join tmpdir "file1.txt"))
  (io/write-file "data2" (path/join tmpdir "file2.txt"))
  (process-directory tmpdir))
;; プログラム終了時にディレクトリと中身は自動的に削除される

;; 使用例: 複数ファイルの一時処理
(defn process-archive [archive-path]
  (let [tmpdir (io/temp-dir)]
    ;; アーカイブを一時ディレクトリに展開
    (archive/extract archive-path tmpdir)
    ;; 展開されたファイルを処理
    (io/list-dir tmpdir :recursive true)
      |> (map (fn [f] (path/join tmpdir f)))
      |> (map process-file)
      |> (reduce merge)
    ;; 関数終了後、tmpdir と中身は自動削除される
    ))

;; 使用例: ビルドディレクトリ
(defn build-project []
  (let [build-dir (io/temp-dir)]
    ;; ソースファイルを一時ディレクトリにコピー
    (io/copy-file "src/main.qi" (path/join build-dir "main.qi"))
    (io/copy-file "src/lib.qi" (path/join build-dir "lib.qi"))
    ;; ビルド処理
    (compile-files build-dir)
    ;; 成果物を取得
    (let [output (io/read-file (path/join build-dir "output.bin"))]
      ;; build-dirは自動削除される
      output)))
```

### 手動削除（永続化）

```qi
;; io/temp-dir-keep - 一時ディレクトリを作成（削除しない）
(let [tmpdir (io/temp-dir-keep)]
  (io/write-file "file1" (path/join tmpdir "data.txt"))
  (io/create-dir (path/join tmpdir "subdir"))
  tmpdir)
;; => "/tmp/qi-dir-12345" （削除されない）

;; 使用例: デバッグ用の出力ディレクトリ
(defn export-debug-info []
  (let [debug-dir (io/temp-dir-keep)]
    (io/write-file (json/stringify state) (path/join debug-dir "state.json"))
    (io/write-file logs (path/join debug-dir "logs.txt"))
    (println f"Debug info exported to: {debug-dir}")
    debug-dir))

;; 後で手動削除
(io/delete-dir tmpdir :recursive true)
```

---

## 自動削除の仕組み

### 削除タイミング

**自動削除される場合**（`io/temp-file`, `io/temp-dir`）:
- プログラムが正常終了したとき
- プログラムがエラーで終了したとき
- REPLセッションが終了したとき

**削除されない場合**（`io/temp-file-keep`, `io/temp-dir-keep`）:
- プログラム終了後も永続化される
- 手動で削除が必要（`io/delete-file`, `io/delete-dir`）

### 実装の詳細

```qi
;; 自動削除版は内部でハンドルを保持
;; プログラム終了時にOSが自動削除する
(io/temp-file)    ;; => ハンドル保持 → 自動削除

;; keep版は永続化
(io/temp-file-keep)  ;; => ハンドル解放 → 削除されない
```

### 注意事項

1. **長時間実行プログラム**: REPLや常駐プログラムでは、一時ファイルが溜まる可能性がある
2. **ディスク容量**: 大きな一時ファイルを大量に作成する場合は注意
3. **keep版の削除**: `io/temp-file-keep`で作成したファイルは必ず手動削除すること
4. **並行処理**: 一時ファイル名は自動的にユニークになるため、並行処理でも安全

---

## 実用例

### テストデータの管理

```qi
;; ユニットテスト用の一時ファイル
(defn test-file-processing []
  (let [tmp (io/temp-file)]
    ;; テストデータを書き込み
    (io/write-file "line1\nline2\nline3" tmp)
    ;; 関数をテスト
    (let [result (process-file tmp)]
      (assert (= result ["line1" "line2" "line3"]))
      ;; tmpは自動削除される
      true)))

;; 統合テスト用の一時ディレクトリ
(defn test-directory-processing []
  (let [tmpdir (io/temp-dir)]
    ;; テスト用のファイル構造を作成
    (io/write-file "config" (path/join tmpdir "config.txt"))
    (io/create-dir (path/join tmpdir "data"))
    (io/write-file "data1" (path/join tmpdir "data" "file1.txt"))
    ;; ディレクトリ処理をテスト
    (let [result (process-directory tmpdir)]
      (assert (= (len result) 2))
      ;; tmpdir と中身は自動削除される
      true)))
```

### データ変換パイプライン

```qi
;; CSV → JSON変換（一時ファイル使用）
(defn csv-to-json [csv-path json-path]
  (let [tmp (io/temp-file)]
    ;; CSVを読み込み
    (io/read-file csv-path)
      |> csv/parse
      ;; データを変換
      |> (map transform-row)
      ;; 一時ファイルにJSON出力
      |> json/stringify
      |> (fn [s] (io/write-file s tmp))
    ;; 検証
    (validate-json tmp)
    ;; 最終出力
    (io/copy-file tmp json-path)
    ;; tmpは自動削除される
    json-path))

;; 複数ファイルの結合（一時ディレクトリ使用）
(defn merge-csv-files [input-files output-path]
  (let [tmpdir (io/temp-dir)]
    ;; 各ファイルを正規化して一時ディレクトリに保存
    (input-files
      |> (map-indexed (fn [i f]
           (let [normalized (io/read-file f |> normalize-csv)]
             (io/write-file normalized
                           (path/join tmpdir f"file{i}.csv"))))))
    ;; 一時ディレクトリ内のファイルを結合
    (io/list-dir tmpdir)
      |> (map (fn [f] (path/join tmpdir f)))
      |> (map io/read-file)
      |> (join "\n")
      |> (fn [s] (io/write-file s output-path))
    ;; tmpdirは自動削除される
    output-path))
```

### HTTPダウンロードとキャッシュ

```qi
;; 大きなファイルのダウンロード
(defn download-large-file [url output-path]
  (let [tmp (io/temp-file)]
    ;; 一時ファイルにダウンロード
    (http/get url :output tmp :timeout 300)
    ;; チェックサム検証
    (if (verify-checksum tmp)
      (do
        (io/move-file tmp output-path)
        (println f"Download completed: {output-path}")
        output-path)
      (do
        ;; エラー時はtmpは自動削除される
        (error "Checksum verification failed")))))

;; キャッシュ付きAPIクライアント
(defn fetch-with-cache [url cache-duration]
  (let [cache-file (io/temp-file-keep)
        cache-age (if (io/file-exists? cache-file)
                    (- (time/now) (get (io/file-info cache-file) "modified"))
                    cache-duration)]
    (if (< cache-age cache-duration)
      ;; キャッシュが有効
      (io/read-file cache-file)
      ;; キャッシュが古い、再取得
      (let [data (http/get url |> get "body")]
        (io/write-file data cache-file)
        data))))
```

### バッチ処理

```qi
;; 大量ファイルの並列処理
(defn parallel-process-files [input-files]
  (let [tmpdir (io/temp-dir)]
    ;; 各ファイルを並列処理
    (input-files
      |> (pmap (fn [f]
           (let [tmp (path/join tmpdir (path/basename f))]
             ;; 一時ファイルに処理結果を保存
             (process-file f |> (fn [result] (io/write-file result tmp)))
             tmp)))
      ;; 処理結果を収集
      |> (map io/read-file)
      ;; tmpdirは自動削除される
      )))

;; 中間ファイルを使ったマルチステージ処理
(defn multi-stage-process [input]
  (let [stage1-file (io/temp-file)
        stage2-file (io/temp-file)
        stage3-file (io/temp-file)]
    ;; Stage 1: データのクリーニング
    (input
      |> clean-data
      |> (fn [s] (io/write-file s stage1-file)))
    ;; Stage 2: データの変換
    (io/read-file stage1-file
      |> transform-data
      |> (fn [s] (io/write-file s stage2-file)))
    ;; Stage 3: データの集計
    (io/read-file stage2-file
      |> aggregate-data
      |> (fn [s] (io/write-file s stage3-file)))
    ;; 最終結果を返す
    (let [result (io/read-file stage3-file)]
      ;; すべての一時ファイルは自動削除される
      result)))
```

### セキュアな一時ファイル処理

```qi
;; 機密データの一時保存（自動削除保証）
(defn process-sensitive-data [encrypted-data]
  (let [tmp (io/temp-file)]
    ;; 復号化したデータを一時ファイルに保存
    (decrypt encrypted-data |> (fn [s] (io/write-file s tmp)))
    ;; 処理
    (let [result (process-data tmp)]
      ;; tmpは自動削除される（機密データが残らない）
      result)))

;; パスワード保護されたファイルの展開
(defn extract-protected-archive [archive-path password]
  (let [tmpdir (io/temp-dir)]
    ;; 展開
    (archive/extract archive-path tmpdir :password password)
    ;; 処理
    (let [result (process-files tmpdir)]
      ;; tmpdirと中身は自動削除される
      result)))
```

---

## パイプライン統合

```qi
;; 一時ファイルを使ったパイプライン
(defn download-convert-upload [source-url target-url]
  (let [tmp (io/temp-file)]
    (source-url
      |> (http/get :output tmp)
      |> (fn [_] (io/read-file tmp))
      |> convert-format
      |> (http/post target-url :body _))
    ;; tmpは自動削除される
    ))

;; 一時ディレクトリを使った複雑なパイプライン
(defn batch-convert [input-dir output-dir]
  (let [tmpdir (io/temp-dir)]
    (input-dir
      |> (io/list-dir :pattern "*.txt")
      |> (map (fn [f]
           ;; 各ファイルを一時ディレクトリで処理
           (let [tmp (path/join tmpdir (path/basename f))]
             (io/read-file (path/join input-dir f)
               |> convert-content
               |> (fn [s] (io/write-file s tmp)))
             tmp)))
      ;; 処理済みファイルを出力ディレクトリにコピー
      |> (map (fn [tmp]
           (io/copy-file tmp (path/join output-dir (path/basename tmp))))))
    ;; tmpdirは自動削除される
    ))
```

---

## セキュリティ考慮事項

### 安全な一時ファイル作成

Qiの一時ファイル機能は、以下のセキュリティ対策を実装しています：

1. **ユニークなファイル名**: 予測不可能なランダムな名前を使用
2. **適切なパーミッション**: 他のユーザーから読み取り不可
3. **自動削除**: 機密データが残らない
4. **TOCTOU攻撃への対策**: ファイル作成と同時にハンドルを取得

```qi
;; ✅ 安全（推奨）
(let [tmp (io/temp-file)]
  (io/write-file sensitive-data tmp)
  (process-file tmp))
;; プログラム終了時に自動削除

;; ❌ 推奨されない（手動削除が必要、削除忘れのリスク）
(let [tmp "/tmp/myapp-data.txt"]
  (io/write-file sensitive-data tmp)
  (process-file tmp)
  (io/delete-file tmp))  ;; エラー時に削除されない可能性
```

### ベストプラクティス

1. **自動削除版を優先**: `io/temp-file` と `io/temp-dir` を使用
2. **機密データは一時ファイルで処理**: 処理後は自動削除される
3. **keep版は最小限に**: 本当に永続化が必要な場合のみ使用
4. **ディスク容量を考慮**: 大きなファイルを扱う場合は注意
5. **エラーハンドリング**: `try-catch`で囲んでも自動削除は機能する

```qi
;; エラー時も自動削除される
(defn safe-process [data]
  (let [tmp (io/temp-file)]
    (try
      (do
        (io/write-file data tmp)
        (risky-operation tmp))
      (catch e
        (println f"Error: {e}")
        ;; tmpは自動削除される
        nil))))
```

---

## 関数一覧

### 一時ファイル
- `io/temp-file` - 一時ファイル作成（自動削除）
- `io/temp-file-keep` - 一時ファイル作成（削除しない）

### 一時ディレクトリ
- `io/temp-dir` - 一時ディレクトリ作成（自動削除）
- `io/temp-dir-keep` - 一時ディレクトリ作成（削除しない）

---

## まとめ

- **自動削除版を推奨**: リソースリークを防ぐ
- **セキュア**: 機密データの一時処理に最適
- **シンプル**: 明示的な削除コードが不要
- **パイプライン統合**: Qiのパイプライン演算子と組み合わせて強力なデータ処理を実現
