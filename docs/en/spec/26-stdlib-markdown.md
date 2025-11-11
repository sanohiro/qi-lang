# Standard Library - Markdown Processing (markdown/)

**Markdown Generation and Parsing Library**

All functions belong to the `markdown/` module.

---

## Overview

The `markdown/` module provides the following features:

- **Markdown Generation** - Generate headers, lists, tables, code blocks, etc.
- **Markdown Parsing** - Convert Markdown strings to AST
- **Code Block Extraction** - Extract code blocks from Markdown
- **AST Manipulation** - Convert AST back to Markdown strings

---

## Generation Functions

### markdown/header

Generate a Markdown header.

```qi
(markdown/header level text)
```

**Arguments:**
- `level` (integer) - Header level (1-6)
- `text` (string) - Header text

**Returns:** string - Markdown header string

**Examples:**

```qi
(markdown/header 1 "Title")
;; => "# Title"

(markdown/header 2 "Subtitle")
;; => "## Subtitle"

(markdown/header 3 "Section")
;; => "### Section"

;; Use in pipeline
(["Introduction" "Getting Started" "Examples"]
 |> (map-indexed (fn [i text] (markdown/header (+ i 1) text)))
 |> (join "\n\n"))
;; => "# Introduction\n\n## Getting Started\n\n### Examples"
```

---

### markdown/list

Generate a Markdown unordered list (bullet list).

```qi
(markdown/list items)
```

**Arguments:**
- `items` (list | vector) - List or vector of items

**Returns:** string - Markdown list string

**Examples:**

```qi
(markdown/list ["Apple" "Banana" "Cherry"])
;; => "- Apple\n- Banana\n- Cherry"

;; Numbers and mixed types are auto-converted
(markdown/list [1 2 3])
;; => "- 1\n- 2\n- 3"

;; Use in pipeline
(["Task 1" "Task 2" "Task 3"]
 |> markdown/list
 |> println)
;; - Task 1
;; - Task 2
;; - Task 3
```

---

### markdown/ordered-list

Generate a Markdown ordered list.

```qi
(markdown/ordered-list items)
```

**Arguments:**
- `items` (list | vector) - List or vector of items

**Returns:** string - Markdown ordered list string

**Examples:**

```qi
(markdown/ordered-list ["First" "Second" "Third"])
;; => "1. First\n2. Second\n3. Third"

;; Generate step-by-step instructions
(def steps ["Install Qi" "Write code" "Run program"])
(markdown/ordered-list steps)
;; => "1. Install Qi\n2. Write code\n3. Run program"
```

---

### markdown/table

Generate a Markdown table. The first row is treated as the header.

```qi
(markdown/table rows)
```

**Arguments:**
- `rows` (list | vector) - List or vector of rows. Each row is also a list or vector

**Returns:** string - Markdown table string

**Examples:**

```qi
;; Basic table
(def data [
  ["Name" "Age" "City"]
  ["Alice" 30 "Tokyo"]
  ["Bob" 25 "Osaka"]])

(markdown/table data)
;; => "| Name | Age | City |
;;     | --- | --- | --- |
;;     | Alice | 30 | Tokyo |
;;     | Bob | 25 | Osaka |"

;; Convert CSV data to table
(csv/read-file "users.csv"
 |> markdown/table
 |> io/write "users.md")
```

**Errors:**
- Table is empty
- Row is not a list or vector
- Column count mismatch

---

### markdown/code-block

Generate a Markdown code block.

```qi
(markdown/code-block lang code)
```

**Arguments:**
- `lang` (string | nil) - Programming language name (nil for no language specification)
- `code` (string) - Code string

**Returns:** string - Markdown code block string

**Examples:**

```qi
;; Qi code block
(markdown/code-block "qi" "(+ 1 2)")
;; => "```qi\n(+ 1 2)\n```"

;; JavaScript code block
(markdown/code-block "js" "console.log('Hello')")
;; => "```js\nconsole.log('Hello')\n```"

;; No language specified
(markdown/code-block nil "plain text")
;; => "```\nplain text\n```"

;; Read from file and create code block
(io/read "example.qi"
 |> (markdown/code-block "qi" _)
 |> println)
```

---

### markdown/link

Generate a Markdown link.

```qi
(markdown/link text url)
```

**Arguments:**
- `text` (string) - Link text
- `url` (string) - Link URL

**Returns:** string - Markdown link string

**Examples:**

```qi
(markdown/link "GitHub" "https://github.com")
;; => "[GitHub](https://github.com)"

(markdown/link "Documentation" "/docs")
;; => "[Documentation](/docs)"

;; Generate list of links
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

Generate Markdown image syntax.

```qi
(markdown/image alt src)
```

**Arguments:**
- `alt` (string) - Alternative text
- `src` (string) - Image path or URL

**Returns:** string - Markdown image string

**Examples:**

```qi
(markdown/image "Logo" "logo.png")
;; => "![Logo](logo.png)"

(markdown/image "Screenshot" "https://example.com/img.jpg")
;; => "![Screenshot](https://example.com/img.jpg)"

;; Generate image gallery
(def images ["img1.png" "img2.png" "img3.png"])
(images
 |> (map-indexed (fn [i src]
                   (markdown/image f"Image {(+ i 1)}" src)))
 |> (join "\n\n"))
```

---

### markdown/join

Join multiple Markdown elements with double newlines.

```qi
(markdown/join parts)
```

**Arguments:**
- `parts` (list | vector) - List or vector of Markdown strings

**Returns:** string - Joined Markdown string

**Examples:**

```qi
;; Join multiple elements
(markdown/join [
  (markdown/header 1 "Report")
  (markdown/header 2 "Summary")
  "This is a summary."
  (markdown/list ["Item 1" "Item 2"])])
;; => "# Report\n\n## Summary\n\nThis is a summary.\n\n- Item 1\n- Item 2"

;; Use in pipeline
(["# Title" "Content here" "More content"]
 |> markdown/join
 |> io/write "document.md")
```

---

## Parsing Functions

### markdown/parse

Convert a Markdown string to an AST.

```qi
(markdown/parse text)
```

**Arguments:**
- `text` (string) - Markdown string

**Returns:** list - List of block elements. Each element is a map

**Supported block elements:**
- `header` - Header (`:type "header"`, `:level`, `:text`)
- `paragraph` - Paragraph (`:type "paragraph"`, `:text`)
- `list` - List (`:type "list"`, `:ordered`, `:items`)
- `code-block` - Code block (`:type "code-block"`, `:lang`, `:code`)

**Examples:**

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

;; Extract headers only
(markdown/parse md-text
 |> (filter (fn [block] (= (:type block) "header")))
 |> (map (fn [h] (:text h))))
;; => ["Title"]
```

---

### markdown/stringify

Convert an AST to a Markdown string. Inverse operation of `markdown/parse`.

```qi
(markdown/stringify blocks)
```

**Arguments:**
- `blocks` (list | vector) - List or vector of block elements

**Returns:** string - Markdown string

**Examples:**

```qi
(def ast [
  {:type "header" :level 1 :text "Title"}
  {:type "paragraph" :text "Hello, world!"}
  {:type "list" :ordered false :items ["Item 1" "Item 2"]}])

(markdown/stringify ast)
;; => "# Title\n\nHello, world!\n\n- Item 1\n- Item 2"

;; Parse, edit, and regenerate
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

Extract only code blocks from Markdown.

```qi
(markdown/extract-code-blocks text)
```

**Arguments:**
- `text` (string) - Markdown string

**Returns:** list - List of code blocks. Each element is a map with `:lang` and `:code`

**Examples:**

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

;; Execute only Qi code blocks
(markdown/extract-code-blocks md-doc
 |> (filter (fn [block] (= (:lang block) "qi")))
 |> (each (fn [block] (eval-string (:code block)))))

;; Save code blocks to files
(markdown/extract-code-blocks md-doc
 |> (each-indexed (fn [i block]
                    (io/write f"code-{i}.{(:lang block)}"
                              (:code block)))))
```

---

## Practical Examples

### Document Generation

```qi
;; Auto-generate function documentation
(defn gen-function-doc [func-name args desc examples]
  (markdown/join [
    (markdown/header 3 func-name)
    desc
    (markdown/header 4 "Arguments")
    (markdown/list args)
    (markdown/header 4 "Examples")
    (markdown/code-block "qi" (join "\n\n" examples))]))

(gen-function-doc
  "map"
  ["f - transformation function" "coll - collection"]
  "Apply a function to each element of a collection."
  ["(map inc [1 2 3])" "(map str/upper [\"a\" \"b\"])"])
```

---

### README Generator

```qi
;; Generate project README
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

### Static Site Generator

```qi
;; Convert Markdown files to HTML using a template
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

;; Process all Markdown files
(io/glob "content/*.md"
 |> (each (fn [file]
            (let [html (render-page file (io/read "template.html"))
                  out-file (str/replace file ".md" ".html")]
              (io/write out-file html)))))
```

---

### Report Generation

```qi
;; Auto-generate reports from data
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

### Blog Post Conversion

```qi
;; Parse Markdown with frontmatter
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

### Test Case Extraction

```qi
;; Extract and run test cases from documentation
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

### Markdown→PDF Conversion Pipeline

```qi
;; Convert Markdown to HTML and generate PDF
(defn md-to-pdf [md-file pdf-file]
  (io/read md-file
   |> markdown/parse
   |> markdown/stringify
   |> (str/replace _ "# " "<h1>" _)  ;; Simple HTML conversion
   |> (str/replace _ "## " "<h2>" _)
   |> (fn [html]
        (sh/exec f"wkhtmltopdf - {pdf-file}" :stdin html))))

(md-to-pdf "document.md" "document.pdf")
```

---

## Pipeline Integration Examples

```qi
;; Convert CSV data to Markdown table and generate report
(csv/read-file "sales.csv"
 |> markdown/table
 |> (fn [table]
      (markdown/join [
        (markdown/header 1 "Sales Data")
        table]))
 |> io/write "sales-report.md")

;; Generate table of contents from all Markdown files in directory
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

## Error Handling

```qi
;; Table generation error handling
(try
  (markdown/table [])  ;; Empty table
  (catch e
    (println f"Error: {e}")))
;; Error: Table cannot be empty

;; Invalid header level
(try
  (markdown/header 7 "Invalid")  ;; Level must be 1-6
  (catch e
    (println f"Error: {e}")))
;; Error: Header level must be between 1 and 6
```

---

## Performance and Best Practices

### Processing Large Documents

```qi
;; ❌ Bad: Repeated string concatenation
(defn generate-large-doc-bad [items]
  (reduce (fn [acc item]
            (str acc (markdown/header 3 item) "\n\n"))
          ""
          items))

;; ✅ Good: Build list then join once
(defn generate-large-doc-good [items]
  (items
   |> (map (fn [item] (markdown/header 3 item)))
   |> markdown/join))
```

---

### Caching

```qi
;; ✅ Cache parse results
(def parsed-doc (markdown/parse (io/read "large-doc.md")))

;; Multiple filtering operations
(def headers (filter (fn [b] (= (:type b) "header")) parsed-doc))
(def lists (filter (fn [b] (= (:type b) "list")) parsed-doc))
(def code-blocks (filter (fn [b] (= (:type b) "code-block")) parsed-doc))
```

---

## Function Reference

| Function | Description | Use Case |
|----------|-------------|----------|
| `markdown/header` | Generate header | `# Title` |
| `markdown/list` | Generate bullet list | `- Item` |
| `markdown/ordered-list` | Generate ordered list | `1. Item` |
| `markdown/table` | Generate table | Table syntax |
| `markdown/code-block` | Generate code block | `` ```lang\ncode\n``` `` |
| `markdown/link` | Generate link | `[text](url)` |
| `markdown/image` | Generate image | `![alt](src)` |
| `markdown/join` | Join elements | Join with double newlines |
| `markdown/parse` | Markdown→AST | Structure parsing |
| `markdown/stringify` | AST→Markdown | AST reconstruction |
| `markdown/extract-code-blocks` | Extract code blocks | Extract `` ```...``` `` |

---

## Related Topics

- [String Manipulation](10-stdlib-string.md) - Text processing
- [File I/O](13-stdlib-io.md) - File reading and writing
- [CSV Processing](22-stdlib-csv.md) - Data conversion
