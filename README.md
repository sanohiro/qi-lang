# Qi - A Lisp that flows

シンプル、高速、簡潔なモダンLisp系言語。パイプライン、パターンマッチング、並行・並列処理に強い。

## 特徴

- **シンプル**: 特殊形式8つのみ（`def` `fn` `let` `do` `if` `match` `try` `defer`）
- **パイプライン**: `|>` 演算子でデータフローを直感的に記述
- **パターンマッチング**: 強力な `match` 式
- **多言語対応**: 英語・日本語のエラーメッセージ（環境変数 `QI_LANG` で設定）
- **並行・並列**: `go`、`pmap`、チャネルによる並行処理
- **安全なマクロ**: `uvar` による変数衝突回避
- **実用的なモジュール**: str、csv、regex、http など

## 多言語対応

Qiは英語と日本語のエラーメッセージに対応しています。

```bash
# 英語（デフォルト）
qi script.qi

# 日本語
QI_LANG=ja qi script.qi
```

## Hello World

```lisp
(def greet (fn [name]
  f"Hello, {name}!"))

(greet "World")
;; "Hello, World!"
```

## パイプライン例

```lisp
(use str :as s)

(data
 |> parse-json
 |> (filter active?)
 |> (map :email)
 |> (s/join ", ")
 |> log)
```

## インストール

```bash
# まだ実装されていません
qi install
```

## ドキュメント

- [完全な言語仕様](SPEC.md) - 詳細な文法、組み込み関数、モジュールシステム

## 例

### CSV処理

```lisp
(use csv)

("data.csv"
 |> csv/parse-file
 |> (filter valid?)
 |> (map process)
 |> (csv/write-file "output.csv"))
```

### エラー処理

```lisp
(match (try (risky-operation))
  {:ok result} -> result
  {:error e} -> (log e))
```

### 並行処理

```lisp
(def urls ["http://a.com" "http://b.com" "http://c.com"])

(urls
 |> (pmap http-get)
 |> (map parse)
 |> flatten)
```

## 開発状況

現在、言語仕様を策定中です。実装は未着手です。

## ライセンス

未定
