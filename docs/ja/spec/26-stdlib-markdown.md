# 標準ライブラリ - Markdown処理（markdown/）

**Markdown生成・解析ライブラリ**

すべての関数は `markdown/` モジュールに属します。

---

## 概要

`markdown/` モジュールは以下の機能を提供します：

- **Markdown生成** - ヘッダー、リスト、テーブル、コードブロックなどの生成
- **Markdown解析** - Markdown文字列からASTへの変換
- **コードブロック抽出** - Markdownからコードブロックを抽出
- **AST操作** - ASTをMarkdown文字列に再変換

---

## 生成関数

### markdown/header

Markdownヘッダーを生成します。

```qi
(markdown/header level text)
```

**引数:**
- `level` (integer) - ヘッダーレベル (1-6)
- `text` (string) - ヘッダーテキスト

**戻り値:** string - Markdownヘッダー文字列

**例:**

```qi
(markdown/header 1 "Title")
;; => "# Title"

(markdown/header 2 "Subtitle")
;; => "## Subtitle"

(markdown/header 3 "Section")
;; => "### Section"

;; パイプラインで使用
(["Introduction" "Getting Started" "Examples"]
 |> (map-indexed (fn [i text] (markdown/header (+ i 1) text)))
 |> (join "\n\n"))
;; => "# Introduction\n\n## Getting Started\n\n### Examples"
```

---

### markdown/list

Markdown箇条書きリスト（順不同リスト）を生成します。

```qi
(markdown/list items)
```

**引数:**
- `items` (list | vector) - 項目のリストまたはベクター

**戻り値:** string - Markdownリスト文字列

**例:**

```qi
(markdown/list ["Apple" "Banana" "Cherry"])
;; => "- Apple\n- Banana\n- Cherry"

;; 数値や混合型も自動変換
(markdown/list [1 2 3])
;; => "- 1\n- 2\n- 3"

;; パイプラインで使用
(["Task 1" "Task 2" "Task 3"]
 |> markdown/list
 |> println)
;; - Task 1
;; - Task 2
;; - Task 3
```

---

### markdown/ordered-list

Markdown番号付きリストを生成します。

```qi
(markdown/ordered-list items)
```

**引数:**
- `items` (list | vector) - 項目のリストまたはベクター

**戻り値:** string - Markdown番号付きリスト文字列

**例:**

```qi
(markdown/ordered-list ["First" "Second" "Third"])
;; => "1. First\n2. Second\n3. Third"

;; 手順書を生成
(def steps ["Install Qi" "Write code" "Run program"])
(markdown/ordered-list steps)
;; => "1. Install Qi\n2. Write code\n3. Run program"
```

---

### markdown/table

Markdownテーブルを生成します。最初の行がヘッダーとして扱われます。

```qi
(markdown/table rows)
```

**引数:**
- `rows` (list | vector) - 行のリストまたはベクター。各行もリストまたはベクター

**戻り値:** string - Markdownテーブル文字列

**例:**

```qi
;; 基本的なテーブル
(def data [
  ["Name" "Age" "City"]
  ["Alice" 30 "Tokyo"]
  ["Bob" 25 "Osaka"]])

(markdown/table data)
;; => "| Name | Age | City |
;;     | --- | --- | --- |
;;     | Alice | 30 | Tokyo |
;;     | Bob | 25 | Osaka |"

;; CSVデータをテーブルに変換
(csv/read-file "users.csv"
 |> markdown/table
 |> io/write "users.md")
```

**エラー:**
- テーブルが空の場合
- 行がリストまたはベクターでない場合
- 列数が不一致の場合

---

### markdown/code-block

Markdownコードブロックを生成します。

```qi
(markdown/code-block lang code)
```

**引数:**
- `lang` (string | nil) - プログラミング言語名（nilの場合は言語指定なし）
- `code` (string) - コード文字列

**戻り値:** string - Markdownコードブロック文字列

**例:**

```qi
;; Qiコードブロック
(markdown/code-block "qi" "(+ 1 2)")
;; => "```qi\n(+ 1 2)\n```"

;; JavaScriptコードブロック
(markdown/code-block "js" "console.log('Hello')")
;; => "```js\nconsole.log('Hello')\n```"

;; 言語指定なし
(markdown/code-block nil "plain text")
;; => "```\nplain text\n```"

;; ファイルから読み込んでコードブロックに
(io/read "example.qi"
 |> (markdown/code-block "qi" _)
 |> println)
```

---

### markdown/link

Markdownリンクを生成します。

```qi
(markdown/link text url)
```

**引数:**
- `text` (string) - リンクテキスト
- `url` (string) - リンク先URL

**戻り値:** string - Markdownリンク文字列

**例:**

```qi
(markdown/link "GitHub" "https://github.com")
;; => "[GitHub](https://github.com)"

(markdown/link "Documentation" "/docs")
;; => "[Documentation](/docs)"

;; リンクリストを生成
(def links [
  {:name "Home" :url "/"}
  {:name "About" :url "/about"}
  {:name "Contact" :url "/contact"}])

(links
 |> (map (fn [l] (markdown/link (:name l) (:url l))))
 |> markdown/list)
;; => "- [Home](/)\n- [About](/about)\n- [Contact](/contact)"
```

---

### markdown/image

Markdown画像記法を生成します。

```qi
(markdown/image alt src)
```

**引数:**
- `alt` (string) - 代替テキスト
- `src` (string) - 画像パスまたはURL

**戻り値:** string - Markdown画像文字列

**例:**

```qi
(markdown/image "Logo" "logo.png")
;; => "![Logo](logo.png)"

(markdown/image "Screenshot" "https://example.com/img.jpg")
;; => "![Screenshot](https://example.com/img.jpg)"

;; 画像ギャラリーを生成
(def images ["img1.png" "img2.png" "img3.png"])
(images
 |> (map-indexed (fn [i src]
                   (markdown/image f"Image {(+ i 1)}" src)))
 |> (join "\n\n"))
```

---

### markdown/join

複数のMarkdown要素を2つの改行で結合します。

```qi
(markdown/join parts)
```

**引数:**
- `parts` (list | vector) - Markdown文字列のリストまたはベクター

**戻り値:** string - 結合されたMarkdown文字列

**例:**

```qi
;; 複数の要素を結合
(markdown/join [
  (markdown/header 1 "Report")
  (markdown/header 2 "Summary")
  "This is a summary."
  (markdown/list ["Item 1" "Item 2"])])
;; => "# Report\n\n## Summary\n\nThis is a summary.\n\n- Item 1\n- Item 2"

;; パイプラインで使用
(["# Title" "Content here" "More content"]
 |> markdown/join
 |> io/write "document.md")
```

---

## 解析関数

### markdown/parse

Markdown文字列をASTに変換します。

```qi
(markdown/parse text)
```

**引数:**
- `text` (string) - Markdown文字列

**戻り値:** list - ブロック要素のリスト。各要素はマップ

**サポートするブロック要素:**
- `header` - ヘッダー（`:type "header"`, `:level`, `:text`）
- `paragraph` - 段落（`:type "paragraph"`, `:text`）
- `list` - リスト（`:type "list"`, `:ordered`, `:items`）
- `code-block` - コードブロック（`:type "code-block"`, `:lang`, `:code`）

**例:**

```qi
(def md-text "# Title

Hello, world!

- Item 1
- Item 2

```qi
(+ 1 2)
```")

(markdown/parse md-text)
;; => [{:type "header" :level 1 :text "Title"}
;;     {:type "paragraph" :text "Hello, world!"}
;;     {:type "list" :ordered false :items ["Item 1" "Item 2"]}
;;     {:type "code-block" :lang "qi" :code "(+ 1 2)"}]

;; ヘッダーのみ抽出
(markdown/parse md-text
 |> (filter (fn [block] (= (:type block) "header")))
 |> (map (fn [h] (:text h))))
;; => ["Title"]
```

---

### markdown/stringify

ASTをMarkdown文字列に変換します。`markdown/parse` の逆操作です。

```qi
(markdown/stringify blocks)
```

**引数:**
- `blocks` (list | vector) - ブロック要素のリストまたはベクター

**戻り値:** string - Markdown文字列

**例:**

```qi
(def ast [
  {:type "header" :level 1 :text "Title"}
  {:type "paragraph" :text "Hello, world!"}
  {:type "list" :ordered false :items ["Item 1" "Item 2"]}])

(markdown/stringify ast)
;; => "# Title\n\nHello, world!\n\n- Item 1\n- Item 2"

;; 解析して編集して再生成
(def modified
  (markdown/parse original-text
   |> (map (fn [block]
             (if (= (:type block) "header")
                 (assoc block :text (str/upper (:text block)))
                 block)))
   |> markdown/stringify))
```

---

### markdown/extract-code-blocks

Markdownからコードブロックのみを抽出します。

```qi
(markdown/extract-code-blocks text)
```

**引数:**
- `text` (string) - Markdown文字列

**戻り値:** list - コードブロックのリスト。各要素は `:lang` と `:code` を持つマップ

**例:**

```qi
(def md-doc "# Examples

```qi
(+ 1 2)
```

Some text here.

```js
console.log('Hello')
```")

(markdown/extract-code-blocks md-doc)
;; => [{:lang "qi" :code "(+ 1 2)"}
;;     {:lang "js" :code "console.log('Hello')"}]

;; Qiコードブロックのみ実行
(markdown/extract-code-blocks md-doc
 |> (filter (fn [block] (= (:lang block) "qi")))
 |> (each (fn [block] (eval-string (:code block)))))

;; コードブロックをファイルに保存
(markdown/extract-code-blocks md-doc
 |> (each-indexed (fn [i block]
                    (io/write f"code-{i}.{(:lang block)}"
                              (:code block)))))
```

---

## 実用例

### ドキュメント生成

```qi
;; 関数ドキュメントを自動生成
(defn gen-function-doc [func-name args desc examples]
  (markdown/join [
    (markdown/header 3 func-name)
    desc
    (markdown/header 4 "引数")
    (markdown/list args)
    (markdown/header 4 "例")
    (markdown/code-block "qi" (join "\n\n" examples))]))

(gen-function-doc
  "map"
  ["f - 変換関数" "coll - コレクション"]
  "コレクションの各要素に関数を適用します。"
  ["(map inc [1 2 3])" "(map str/upper [\"a\" \"b\"])"])
```

---

### READMEジェネレーター

```qi
;; プロジェクトのREADMEを生成
(defn generate-readme [project]
  (markdown/join [
    (markdown/header 1 (:name project))
    (:description project)
    (markdown/header 2 "Features")
    (markdown/list (:features project))
    (markdown/header 2 "Installation")
    (markdown/code-block "bash" (:install-cmd project))
    (markdown/header 2 "Usage")
    (markdown/code-block (:lang project) (:usage-example project))
    (markdown/header 2 "License")
    (:license project)]))

(def project-info {
  :name "My Project"
  :description "A cool project"
  :features ["Fast" "Simple" "Reliable"]
  :install-cmd "npm install my-project"
  :lang "js"
  :usage-example "const mp = require('my-project');\nmp.run();"
  :license "MIT"})

(generate-readme project-info
 |> io/write "README.md")
```

---

### 静的サイトジェネレーター

```qi
;; Markdownファイルをテンプレートを使ってHTMLに変換
(defn render-page [md-file template]
  (let [content (io/read md-file)
        ast (markdown/parse content)
        title (-> ast
                  (filter (fn [b] (= (:type b) "header")))
                  first
                  :text)
        body (markdown/stringify ast)]
    (str/replace template "{{{title}}}" title
     |> (str/replace _ "{{{body}}}" body))))

;; 全Markdownファイルを処理
(io/glob "content/*.md"
 |> (each (fn [file]
            (let [html (render-page file (io/read "template.html"))
                  out-file (str/replace file ".md" ".html")]
              (io/write out-file html)))))
```

---

### レポート生成

```qi
;; データからレポートを自動生成
(defn generate-sales-report [sales-data]
  (let [total (reduce + (map :amount sales-data))
        avg (/ total (count sales-data))
        top-sales (take 5 (reverse (sort-by :amount sales-data)))]
    (markdown/join [
      (markdown/header 1 "Sales Report")
      (markdown/header 2 "Summary")
      f"Total sales: ${total}"
      f"Average: ${avg}"
      (markdown/header 2 "Top 5 Sales")
      (markdown/table
        (cons ["Date" "Product" "Amount"]
              (map (fn [s] [(:date s) (:product s) (:amount s)])
                   top-sales)))])))

(def sales [
  {:date "2024-01-01" :product "Widget A" :amount 1500}
  {:date "2024-01-02" :product "Widget B" :amount 2000}
  {:date "2024-01-03" :product "Widget A" :amount 1200}])

(generate-sales-report sales
 |> io/write "report.md")
```

---

### ブログ記事変換

```qi
;; フロントマター付きMarkdownを解析
(defn parse-blog-post [md-text]
  (let [lines (str/lines md-text)
        frontmatter-end (index-of lines "---" 1)
        frontmatter (take frontmatter-end (drop 1 lines))
        content (drop (+ frontmatter-end 1) lines |> (join "\n"))
        metadata (frontmatter
                  |> (map (fn [line] (str/split line ":")))
                  |> (reduce (fn [acc [k v]]
                               (assoc acc (keyword (str/trim k))
                                         (str/trim v)))
                             {}))]
    {:metadata metadata
     :content content
     :ast (markdown/parse content)}))

(def blog-post "---
title: My First Post
date: 2024-01-01
tags: qi, programming
---

# Introduction

This is my first blog post!")

(parse-blog-post blog-post)
;; => {:metadata {:title "My First Post"
;;                :date "2024-01-01"
;;                :tags "qi, programming"}
;;     :content "# Introduction\n\nThis is my first blog post!"
;;     :ast [...]}
```

---

### テストケース抽出

```qi
;; ドキュメントからテストケースを抽出して実行
(defn extract-and-run-tests [md-file]
  (let [content (io/read md-file)
        code-blocks (markdown/extract-code-blocks content)
        qi-tests (filter (fn [b] (= (:lang b) "qi")) code-blocks)]
    (doseq [test qi-tests]
      (try
        (eval-string (:code test))
        (println "✓ Test passed")
        (catch e
          (println f"✗ Test failed: {e}"))))))

(extract-and-run-tests "docs/examples.md")
```

---

### Markdown→PDF変換パイプライン

```qi
;; MarkdownをHTMLに変換してPDF生成
(defn md-to-pdf [md-file pdf-file]
  (io/read md-file
   |> markdown/parse
   |> markdown/stringify
   |> (str/replace _ "# " "<h1>" _)  ;; 簡易HTML変換
   |> (str/replace _ "## " "<h2>" _)
   |> (fn [html]
        (sh/exec f"wkhtmltopdf - {pdf-file}" :stdin html))))

(md-to-pdf "document.md" "document.pdf")
```

---

## パイプライン統合例

```qi
;; CSVデータをMarkdownテーブルに変換してレポート生成
(csv/read-file "sales.csv"
 |> markdown/table
 |> (fn [table]
      (markdown/join [
        (markdown/header 1 "Sales Data")
        table]))
 |> io/write "sales-report.md")

;; ディレクトリ内の全Markdownファイルの目次を生成
(io/glob "docs/*.md"
 |> (map (fn [file]
           (let [content (io/read file)
                 title (-> content
                           markdown/parse
                           (filter (fn [b] (= (:type b) "header")))
                           first
                           :text)]
             (markdown/link title file))))
 |> markdown/list
 |> (fn [toc]
      (markdown/join [
        (markdown/header 1 "Table of Contents")
        toc]))
 |> io/write "TOC.md")
```

---

## エラーハンドリング

```qi
;; テーブル生成のエラー処理
(try
  (markdown/table [])  ;; 空のテーブル
  (catch e
    (println f"Error: {e}")))
;; Error: Table cannot be empty

;; 不正なヘッダーレベル
(try
  (markdown/header 7 "Invalid")  ;; レベルは1-6
  (catch e
    (println f"Error: {e}")))
;; Error: Header level must be between 1 and 6
```

---

## パフォーマンスとベストプラクティス

### 大きなドキュメントの処理

```qi
;; ❌ 悪い例: 文字列連結を繰り返す
(defn generate-large-doc-bad [items]
  (reduce (fn [acc item]
            (str acc (markdown/header 3 item) "\n\n"))
          ""
          items))

;; ✅ 良い例: リストを作ってから一度に結合
(defn generate-large-doc-good [items]
  (items
   |> (map (fn [item] (markdown/header 3 item)))
   |> markdown/join))
```

---

### キャッシュの活用

```qi
;; ✅ パース結果をキャッシュ
(def parsed-doc (markdown/parse (io/read "large-doc.md")))

;; 複数回フィルタリング
(def headers (filter (fn [b] (= (:type b) "header")) parsed-doc))
(def lists (filter (fn [b] (= (:type b) "list")) parsed-doc))
(def code-blocks (filter (fn [b] (= (:type b) "code-block")) parsed-doc))
```

---

## 関数一覧

| 関数 | 説明 | 用途 |
|------|------|------|
| `markdown/header` | ヘッダー生成 | `# Title` |
| `markdown/list` | 箇条書きリスト生成 | `- Item` |
| `markdown/ordered-list` | 番号付きリスト生成 | `1. Item` |
| `markdown/table` | テーブル生成 | テーブル記法 |
| `markdown/code-block` | コードブロック生成 | `` ```lang\ncode\n``` `` |
| `markdown/link` | リンク生成 | `[text](url)` |
| `markdown/image` | 画像生成 | `![alt](src)` |
| `markdown/join` | 要素結合 | 2つの改行で結合 |
| `markdown/parse` | Markdown→AST | 構造解析 |
| `markdown/stringify` | AST→Markdown | AST再構築 |
| `markdown/extract-code-blocks` | コードブロック抽出 | `` ```...``` `` 抽出 |

---

## 関連項目

- [文字列操作](10-stdlib-string.md) - テキスト処理
- [ファイルI/O](13-stdlib-io.md) - ファイルの読み書き
- [CSV処理](22-stdlib-csv.md) - データ変換
