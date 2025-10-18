# Qi - A Lisp that flows

<p align="center">
  <img src="./assets/logo/qi-logo-full-512.png" alt="Qi Logo" width="400">
</p>

**データの流れを設計するLisp系言語。パイプライン、パターンマッチング、並行処理に強い。**

## 特徴

- **パイプライン**: `|>` `|>?` `||>` `~>` でデータフローを直感的に記述
- **パターンマッチング**: 強力な `match` 式で分岐と変換を統合
- **並行・並列**: goroutine風の並行処理とチャネル、並列パイプライン
- **Web開発**: JSON/HTTP対応、Railway Pipelineでエラーハンドリング
- **f-string**: 文字列補間と複数行文字列（`"""..."""`）
- **多言語対応**: 英語・日本語のエラーメッセージ（`QI_LANG=ja`）


## Hello World

```lisp
(def greet (fn [name]
  f"Hello, {name}!"))

(greet "World")
;; "Hello, World!"
```

## パイプライン例

### 基本パイプライン
```lisp
(data
 |> parse-json
 |> (filter active?)
 |> (map :email)
 |> (join ", ")
 |> log)
```

### Railway Pipeline - エラーハンドリング
```lisp
;; Web APIからデータ取得 → 変換 → 保存
("https://api.example.com/users/123"
 |> http/get              ;; {:ok {...}} または {:error ...}
 |>? (fn [resp] {:ok (get resp "body")})
 |>? json/parse
 |>? validate-user
 |>? save-to-db)
;; エラーが起きたら自動的にショートサーキット！
```

### 非同期パイプライン - goroutine風の並行実行 ⭐ NEW
```lisp
;; 即座にチャネルを返し、バックグラウンドで実行
(def result (data ~> transform ~> process))
(recv! result)  ;; 結果を受信

;; 複数の非同期処理を並行実行
(def r1 (10 ~> inc ~> double))
(def r2 (20 ~> double ~> inc))
(println (recv! r1) (recv! r2))  ;; 両方並行実行
```

## 使い方

```bash
# REPL起動
qi

# スクリプトファイル実行
qi script.qi

# ワンライナー実行
qi -e '(+ 1 2 3)'

# テスト実行
qi test

# ヘルプ表示
qi --help
```

## ドキュメント

- **[docs/spec/](docs/spec/)** - Qi言語の完全な仕様書
- **[docs/style-guide.md](docs/style-guide.md)** - コーディングスタイルガイド

## ライセンス

MIT OR Apache-2.0 のデュアルライセンスです。お好きな方を選択してください。

- [LICENSE-MIT](LICENSE-MIT) - MITライセンス
- [LICENSE-APACHE](LICENSE-APACHE) - Apache License 2.0

詳細は各ライセンスファイルを参照してください。
