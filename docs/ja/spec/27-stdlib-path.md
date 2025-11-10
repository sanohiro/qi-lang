# 標準ライブラリ - パス操作（path/）

**クロスプラットフォーム対応のパス操作**

すべての関数は `path/` モジュールに属します。

---

## パス結合

### path/join

複数のパス要素を結合して、プラットフォームに適したパスを生成します。

```qi
;; 基本的な使い方
(path/join "dir" "subdir" "file.txt")
;; Unix: => "dir/subdir/file.txt"
;; Windows: => "dir\\subdir\\file.txt"

;; 絶対パスとの結合
(path/join "/usr" "local" "bin")
;; => "/usr/local/bin"

;; 可変長引数
(path/join "a" "b" "c" "d" "e")
;; => "a/b/c/d/e"

;; パイプラインと組み合わせ
(["logs" "2024" "01" "app.log"]
 |> (apply path/join))
;; => "logs/2024/01/app.log"
```

**引数**:
- `parts...` - 結合するパス要素（可変長、最低1つ）

**戻り値**: 結合されたパス文字列

**クロスプラットフォーム**:
- Unix/Linux/macOS: `/` 区切り
- Windows: `\` 区切り

---

## パス解析

### path/basename

パスからファイル名部分（最後の要素）を取得します。

```qi
;; ファイル名取得
(path/basename "/path/to/file.txt")
;; => "file.txt"

;; ディレクトリ名取得
(path/basename "/path/to/dir")
;; => "dir"

;; Windowsパス
(path/basename "C:\\Users\\Alice\\Document.docx")
;; => "Document.docx"

;; ルートディレクトリ
(path/basename "/")
;; => ""

;; パイプラインで使用
(io/list-dir "downloads")
 |> (map path/basename)
;; => ["file1.txt" "file2.pdf" "image.png"]
```

**引数**:
- `path` - パス文字列

**戻り値**: ファイル名部分の文字列（取得できない場合は空文字列）

---

### path/dirname

パスからディレクトリ部分（親ディレクトリ）を取得します。

```qi
;; ディレクトリ部分取得
(path/dirname "/path/to/file.txt")
;; => "/path/to"

;; 複数階層
(path/dirname "/a/b/c/d.txt")
;; => "/a/b/c"

;; Windowsパス
(path/dirname "C:\\Users\\Alice\\file.txt")
;; => "C:\\Users\\Alice"

;; ルート直下
(path/dirname "/file.txt")
;; => "/"

;; 相対パス
(path/dirname "dir/file.txt")
;; => "dir"

;; パイプラインで親ディレクトリ取得
(file-path
 |> path/dirname
 |> io/create-dir)
```

**引数**:
- `path` - パス文字列

**戻り値**: ディレクトリ部分の文字列（取得できない場合は空文字列）

---

### path/extension

ファイルの拡張子を取得します。

```qi
;; 拡張子取得
(path/extension "file.txt")
;; => "txt"

(path/extension "archive.tar.gz")
;; => "gz"

;; 拡張子なし
(path/extension "README")
;; => ""

;; ドットファイル
(path/extension ".gitignore")
;; => ""

;; パスが含まれる場合
(path/extension "/path/to/document.pdf")
;; => "pdf"

;; 拡張子でフィルタリング
(io/list-dir "src")
 |> (filter (fn [f] (= (path/extension f) "rs")))
;; => ["/path/to/main.rs" "/path/to/lib.rs"]
```

**引数**:
- `path` - パス文字列

**戻り値**: 拡張子（ドットなし）。拡張子がない場合は空文字列

**注意**: 最後のドットより後の部分のみを返します（`file.tar.gz` → `"gz"`）

---

### path/stem

ファイル名から拡張子を除いた部分を取得します。

```qi
;; 拡張子なしファイル名
(path/stem "file.txt")
;; => "file"

(path/stem "document.pdf")
;; => "document"

;; 複数ドット
(path/stem "archive.tar.gz")
;; => "archive.tar"

;; 拡張子なし
(path/stem "README")
;; => "README"

;; パスが含まれる場合
(path/stem "/path/to/report.docx")
;; => "report"

;; パイプラインでファイル名変換
("data.csv"
 |> path/stem
 |> (fn [s] (str s "_processed.json")))
;; => "data_processed.json"
```

**引数**:
- `path` - パス文字列

**戻り値**: 拡張子を除いたファイル名（取得できない場合は空文字列）

---

## パス変換

### path/absolute

相対パスを絶対パスに変換します。

```qi
;; 相対パスを絶対パスに変換
(path/absolute "data/file.txt")
;; => "/Users/alice/project/data/file.txt"

;; 既に絶対パスの場合はそのまま
(path/absolute "/usr/local/bin")
;; => "/usr/local/bin"

;; カレントディレクトリ基準
(path/absolute ".")
;; => "/Users/alice/project"

;; 親ディレクトリ参照
(path/absolute "../other")
;; => "/Users/alice/other"

;; パイプラインで使用
(relative-paths
 |> (map path/absolute))
```

**引数**:
- `path` - パス文字列

**戻り値**: 絶対パス文字列

**注意**: カレントディレクトリを基準に解決します

---

### path/normalize

パスを正規化し、`.` と `..` を解決します。

```qi
;; . と .. を解決
(path/normalize "a/./b/../c")
;; => "a/c"

(path/normalize "/path/to/../other/./file.txt")
;; => "/path/other/file.txt"

;; 連続スラッシュを統合
(path/normalize "a//b///c")
;; => "a/b/c"

;; 複雑なパス
(path/normalize "/a/b/c/../../d/./e/../f")
;; => "/a/d/f"

;; パイプラインで使用
(user-input-path
 |> path/normalize
 |> path/absolute)
```

**引数**:
- `path` - パス文字列

**戻り値**: 正規化されたパス文字列

**注意**:
- `.` は無視されます
- `..` は親ディレクトリに移動（スタック方式）
- ルートより上には移動しません

---

## パス判定

### path/is-absolute?

パスが絶対パスかどうかを判定します。

```qi
;; 絶対パス判定
(path/is-absolute? "/usr/bin")
;; => true

(path/is-absolute? "relative/path")
;; => false

;; Windowsパス
(path/is-absolute? "C:\\Program Files")
;; => true

(path/is-absolute? "Documents\\file.txt")
;; => false

;; パイプラインで使用
(paths
 |> (filter path/is-absolute?))
```

**引数**:
- `path` - パス文字列

**戻り値**: `true` または `false`

**プラットフォーム別判定**:
- Unix/Linux/macOS: `/` で始まる
- Windows: `C:\` や `\\server\` で始まる

---

### path/is-relative?

パスが相対パスかどうかを判定します。

```qi
;; 相対パス判定
(path/is-relative? "data/file.txt")
;; => true

(path/is-relative? "/usr/local")
;; => false

;; カレントディレクトリ
(path/is-relative? ".")
;; => true

(path/is-relative? "..")
;; => true

;; パイプラインで使用
(paths
 |> (filter path/is-relative?)
 |> (map path/absolute))
```

**引数**:
- `path` - パス文字列

**戻り値**: `true` または `false`

**注意**: `path/is-absolute?` の逆です

---

## 実用例

### ファイルパスの構築

```qi
;; プロジェクト構造の構築
(def project-root "/Users/alice/project")
(def src-dir (path/join project-root "src"))
(def test-dir (path/join project-root "tests"))
(def main-file (path/join src-dir "main.qi"))

;; ユーザーデータディレクトリ
(defn user-data-path [username filename]
  (path/join "/data" "users" username filename))

(user-data-path "alice" "profile.json")
;; => "/data/users/alice/profile.json"
```

### ファイル名変換

```qi
;; バックアップファイル名生成
(defn backup-filename [original-path]
  (let [dir (path/dirname original-path)
        name (path/stem original-path)
        ext (path/extension original-path)
        timestamp (time/now |> time/format "yyyyMMdd_HHmmss")]
    (path/join dir (str name "_backup_" timestamp "." ext))))

(backup-filename "/data/document.txt")
;; => "/data/document_backup_20240115_143025.txt"
```

### パス解析パイプライン

```qi
;; ファイル情報の取得
(defn file-info [filepath]
  {:path filepath
   :absolute (path/absolute filepath)
   :dir (path/dirname filepath)
   :name (path/basename filepath)
   :stem (path/stem filepath)
   :ext (path/extension filepath)
   :is-absolute (path/is-absolute? filepath)})

(file-info "src/main.qi")
;; => {:path "src/main.qi"
;;     :absolute "/Users/alice/project/src/main.qi"
;;     :dir "src"
;;     :name "main.qi"
;;     :stem "main"
;;     :ext "qi"
;;     :is-absolute false}
```

### ディレクトリトラバース

```qi
;; ディレクトリ内の全Qiファイルを取得
(defn find-qi-files [dir]
  (io/list-dir dir :recursive true)
   |> (filter (fn [f] (= (path/extension f) "qi")))
   |> (map path/absolute))

(find-qi-files "src")
;; => ["/project/src/main.qi"
;;     "/project/src/lib.qi"
;;     "/project/src/utils/helpers.qi"]
```

### 安全なパス処理

```qi
;; ユーザー入力のパスを検証
(defn safe-path [base-dir user-input]
  (let [full-path (-> user-input
                      path/normalize
                      (fn [p] (path/join base-dir p))
                      path/absolute)]
    ;; ベースディレクトリ外へのアクセスを防ぐ
    (if (str/starts-with? full-path base-dir)
      full-path
      (throw "Invalid path: outside base directory"))))

(safe-path "/data/users/alice" "../bob/secret.txt")
;; エラー: Invalid path: outside base directory

(safe-path "/data/users/alice" "documents/file.txt")
;; => "/data/users/alice/documents/file.txt"
```

### 拡張子別処理

```qi
;; 拡張子に応じてファイルを処理
(defn process-file [filepath]
  (match (path/extension filepath)
    "txt" (io/read-file filepath)
    "json" (-> filepath io/read-file json/parse)
    "csv" (csv/read-file filepath)
    "qi" (load-file filepath)
    _ (throw (str "Unsupported file type: " filepath))))

;; ディレクトリ内のファイルを一括処理
(io/list-dir "data")
 |> (map process-file)
 |> (filter some?)
```

### クロスプラットフォーム対応

```qi
;; OSに応じた設定ファイルパス
(defn config-path []
  (let [home (env/get "HOME" (env/get "USERPROFILE"))]
    (if (str/contains? (env/get "OS" "") "Windows")
      (path/join home "AppData" "Roaming" "MyApp" "config.json")
      (path/join home ".config" "myapp" "config.json"))))

;; ログファイルパス生成
(defn log-path [app-name]
  (let [log-dir (if (str/contains? (env/get "OS" "") "Windows")
                  (path/join (env/get "PROGRAMDATA") app-name "logs")
                  (path/join "/var" "log" app-name))]
    (path/join log-dir (str app-name ".log"))))
```

### 相対パスの一括変換

```qi
;; プロジェクト内の相対パスを絶対パスに変換
(def project-files [
  "src/main.qi"
  "tests/test_main.qi"
  "docs/README.md"])

(def absolute-files
  (project-files
   |> (map path/absolute)
   |> (map path/normalize)))
```

---

## クロスプラットフォーム対応

### パス区切り文字

- **Unix/Linux/macOS**: `/`
- **Windows**: `\`（表示は `\\` のエスケープが必要）

`path/` モジュールは自動的にプラットフォーム固有の区切り文字を使用します。

### 絶対パスの形式

**Unix/Linux/macOS**:
```qi
(path/is-absolute? "/usr/local/bin")  ;; => true
(path/is-absolute? "~/Documents")     ;; => false（~ は展開されない）
```

**Windows**:
```qi
(path/is-absolute? "C:\\Program Files")  ;; => true
(path/is-absolute? "\\\\server\\share")  ;; => true（UNCパス）
(path/is-absolute? "relative\\path")     ;; => false
```

### パス正規化の違い

**Unix/Linux/macOS**:
```qi
(path/normalize "/usr/./local/../bin")
;; => "/usr/bin"
```

**Windows**:
```qi
(path/normalize "C:\\Users\\.\\Alice\\..\\Bob")
;; => "C:\\Users\\Bob"
```

---

## 関数一覧

| 関数 | 説明 | 用途 |
|------|------|------|
| `path/join` | パス要素を結合 | パス構築 |
| `path/basename` | ファイル名取得 | ファイル名解析 |
| `path/dirname` | ディレクトリ部分取得 | 親ディレクトリ |
| `path/extension` | 拡張子取得 | ファイルタイプ判定 |
| `path/stem` | 拡張子なしファイル名 | ファイル名変換 |
| `path/absolute` | 絶対パスに変換 | パス解決 |
| `path/normalize` | パス正規化 | `.` と `..` の解決 |
| `path/is-absolute?` | 絶対パス判定 | パス検証 |
| `path/is-relative?` | 相対パス判定 | パス検証 |

---

## 参考

- **関連モジュール**:
  - `io/` - ファイル入出力
  - `env/` - 環境変数
  - `str/` - 文字列操作
- **プラットフォーム**:
  - Unix/Linux/macOS、Windows両対応
  - Rustの `std::path` モジュールを使用
