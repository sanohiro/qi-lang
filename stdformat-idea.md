## Qi 標準サポートファイル形式・関数仕様まとめ

---

### 1. 現状標準サポートフォーマット
| フォーマット | モジュール | 主な関数 | 主用途 |
|--------------|-----------|----------|--------|
| **JSON** | `json` | `json/parse`, `json/stringify`, `json/pretty` | Web API, LLM, 構造化データ処理 |
| **CSV** | `csv` | `csv/parse`, `csv/stringify`, `csv/read-file` | 表形式データの入出力、ETL |
| **Markdown** | `markdown`（構想中） | `markdown/header`, `markdown/list`, `markdown/table`, `markdown/code-block`, `markdown/parse`, `markdown/stringify`, `markdown/extract-code-blocks` | ドキュメント生成、LLMプロンプト、レポート整形 |

---

### 2. 各フォーマット別 関数仕様と使用例

#### **JSON モジュール**
| 関数 | 説明 | 例 |
|------|------|----|
| `json/parse text` | JSON文字列 → Qiデータ構造（{:ok val} / {:error e}） | `(json/parse "{\"a\":1}")` |
| `json/stringify data` | Qiデータ → JSON文字列 | `(json/stringify {"a" 1})` |
| `json/pretty data` | Qiデータ → 整形JSON文字列 | `(json/pretty {"a" 1})` |
| **追加案** `json/merge ...` | 複数JSONマージ | |
| **追加案** `json/select path` | キー経路指定で部分抽出 | |

---

#### **CSV モジュール**
| 関数 | 説明 | 例 |
|------|------|----|
| `csv/parse text` | CSV文字列 → 2Dベクタ | `(csv/parse "a,b,c\n1,2,3")` |
| `csv/stringify rows` | 2Dベクタ → CSV文字列 | `(csv/stringify [["a" "b"] ["1" "2"]])` |
| `csv/read-file path` | CSVファイル読み込み | `(csv/read-file "data.csv")` |
| **追加案** `csv/write-file rows path` | CSVファイル書き込み | |
| **追加案** `csv/select-cols rows cols` | 指定列だけ抽出 | |
| **追加案** `csv/filter-rows rows pred` | 条件マッチ行のみ抽出 | |
| **追加案** 区切り指定（TSV/PSV対応） | `(csv/parse text :delimiter "\t")` | |

---

#### **Markdown モジュール（新規提案）**
| 関数 | 説明 | 例 |
|------|------|----|
| `markdown/header level text` | 見出し生成 | `(markdown/header 2 "Title") ;; "## Title"` |
| `markdown/list items` | 箇条書き | `(markdown/list ["A" "B"])` |
| `markdown/ordered-list items` | 番号付きリスト | `(markdown/ordered-list ["Step1" "Step2"])` |
| `markdown/table rows` | 表生成（初行ヘッダ） | `(markdown/table [["Name" "Age"] ["Alice" "30"]])` |
| `markdown/code-block lang code` | 言語指定付きコードブロック生成 | `(markdown/code-block "qi" "(println \"Hi\")")` |
| `markdown/link text url` | リンク生成 | `(markdown/link "GitHub" "https://github.com")` |
| `markdown/image alt src` | 画像挿入 | `(markdown/image "Logo" "/logo.png")` |
| `markdown/join parts` | 複数要素結合 | |
| `markdown/stringify ast` | AST → Markdown文字列 | |
| `markdown/parse text` | Markdown → AST | |
| `markdown/extract-code-blocks text` | コードブロック配列抽出（{:lang, :code}） | `(markdown/extract-code-blocks md)` |

---

### 3. サポートのメリット

#### 共通
- **ETLの一環として簡単に変換可能**  
  Qiの`|>`パイプラインでフォーマット変換・抽出・加工が完結。
- **外部ツール連携が容易**  
  `cmd/pipe`で`pandoc`, `jq`などを組み合わせられる。
- **LLMとの相性**  
  JSONとMarkdownはLLMプロンプトや結果整形にそのまま利用可能。

#### フォーマット別
- **JSON**: API、設定ファイル、LLM返答解析に必須。  
- **CSV**: Excelや業務データ連携の定番。Qiのストリーム機能で大容量も処理可能。  
- **Markdown**: 人間可読＋機械可読。レポートやLLMプロンプト生成の中間フォーマットとして強力。

---

### 4. 拡張候補フォーマット & 関数案
| フォーマット | 関数案 | 主用途 |
|--------------|--------|--------|
| **YAML** | `yaml/parse`, `yaml/stringify`, `yaml/merge` | 設定ファイル、CI/CD、Kubernetes |
| **TSV/PSV** | CSV拡張で`delimiter`指定 | データ分析分野で頻出 |
| **Excel(XLSX)** | `excel/read`, `excel/write`, `excel/sheet-names` | 業務表計算との連携 |
| **MessagePack/BSON** | `msgpack/encode`, `msgpack/decode`, `bson/encode`, `bson/decode` | 高速シリアライズ、IPC |

---

### 5. 使用例（複合パイプライン）

#### JSON → Markdown レポート
```lisp
("https://api.example.com/data"
 |> http/get
 |>? (fn [resp] {:ok (get resp "body")})
 |>? json/parse
 |> markdown/table
 |> markdown/stringify
 |> write-file "report.md")
```

#### CSV → JSON（カスタムカラム抽出）
```lisp
(csv/read-file "sales.csv")
 |> (csv/select-cols _ ["Product" "Revenue"])
 |> json/stringify
 |> write-file "sales.json")
```

#### Markdownコードブロック抽出 → 実行
```lisp
(file/read-file "doc.md"
 |> markdown/extract-code-blocks
 |> (filter (fn [b] (= (get b "lang") "qi")))
 |> (map (fn [b] (eval (get b "code")))))
```

---