## Qi 言語 — Markdown 対応構想まとめ  

---

### 1. 背景と目的
Qi は既に Flow-Oriented Programming・並列/非同期処理など、実用に耐える機能を持つ言語。  
しかし、**プロンプト生成やドキュメント構造化**といったエコシステム的活用を広げるため、Markdownサポートを追加する構想が浮上。  

Markdownは以下の理由で相性が良い:
- LLMとのやりとりでよく使われる（構造的で可読性高い）
- レポート・静的ドキュメント生成の中間フォーマットとして優秀
- Qiのパイプライン哲学（データ → 整形 → 出力）に自然に組み込める

---

### 2. 想定するサポート範囲
Qi側で担うのは**Markdown生成・加工まで**。  
HTMLやPDF化は`pandoc`など外部ツールに委任する方針。

---

#### **必須生成系関数**
| 関数 | 説明 | 例 |
|------|------|----|
| `markdown/header level text` | 見出し生成（#〜######） | `(markdown/header 2 "Report")` → `"## Report"` |
| `markdown/list items` | 箇条書きリスト | `(markdown/list ["A" "B"])` |
| `markdown/ordered-list items` | 番号付きリスト | `"1. A\n2. B"` |
| `markdown/table rows` | 表生成。最初の行はヘッダ | `["Name" "Score"]...` |
| `markdown/code-block lang code` | 言語指定付きコードブロック | ````qi\n(code)\n``` `` |
| `markdown/link text url` | ハイパーリンク | `[GitHub](https://github.com)` |
| `markdown/image alt src` | 画像記法 | `![Alt](path.png)` |

---

#### **加工系関数**
| 関数 | 説明 |
|------|------|
| `markdown/join parts` | 複数Markdown要素を改行結合 |
| `markdown/stringify` | AST等をMarkdown文字列化 |
| `markdown/parse text` | Markdown → AST（ブロック/インライン解析） |

---

#### **コード抽出系（新提案）**
| 関数 | 説明 | 出力形式 |
|------|------|----------|
| `markdown/extract-code-blocks text` | Markdown内の全コードブロック抽出。言語も取得。 | `[{lang "qi" code "(...)"} ...]` |

**用途例:**
- Qiコードだけ抜き出して`eval`
- Lang別に自動処理（例: `"python"`なら外部で実行）
- LLMに「コード部分だけ」渡すフィルタとして活用

---

### 3. 利用シナリオ例

#### プロンプト作成
```lisp
(data
 |> analyze-stats
 |> markdown/header 2 "Stats"
 |> markdown/table
 |> markdown/stringify)
```
→ LLMに直接渡せる整形済みMarkdownを生成

---

#### コード抽出と実行
```lisp
(md-doc
 |> markdown/extract-code-blocks
 |> (filter (fn [b] (= (get b "lang") "qi")))
 |> (map (fn [b] (eval (get b "code")))))
```

---

#### 言語別処理
```lisp
(for-each (fn [block]
  (match (get block "lang")
    "qi" -> (run-qi-code (get block "code"))
    "python" -> (run-python (get block "code"))
    _ -> (log "Unsupported lang")))
  (markdown/extract-code-blocks md-text))
```

---

### 4. 実装方針プラン
1. **生成系関数**は純粋関数で実装（文字列→文字列）
2. **パーサ**は最小限の正規表現＋行分割処理から着手
3. コードブロック抽出は ````lang\n...\n``` `` パターンで正規表現対応
4. 後から拡張可能なAST構造を採用（ヘッダ/リスト/表/コードなど）
5. 外部変換（HTML/PDF）は`cmd/pipe`経由で他ツールに委任

---

### 5. 優先度付きロードマップ
1. **最小実装**: header, list, ordered-list, table, code-block, join, stringify
2. **解析系追加**: parse, extract-code-blocks
3. **便利関数**: link, image
4. ドキュメント例とLLM活用チュートリアル

---

### 6. まとめ
- QiにMarkdown生成・加工機能を組み込むことで、**プロンプト作成・レポート生成・コード抽出**がパイプライン内で完結
- HTML/PDFなど最終フォーマットは外部に委ねることで、実装も軽量かつ保守しやすい
- 特に`markdown/extract-code-blocks`はLLM連携やソースコード処理の自動化に直結し、Qiの実用性をさらに高める
