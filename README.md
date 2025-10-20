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
(defn greet [name]
  f"Hello, {name}!")

(println (greet "World"))
;; => Hello, World!
```

## パイプライン例

### 基本パイプライン
```lisp
;; 数値のフィルタリングと変換
([1 2 3 4 5 6 7 8 9 10]
 |> (filter (fn [x] (> x 5)))
 |> (map (fn [x] (* x 2)))
 |> (reduce + 0))
;; => 90

;; 文字列処理
("hello world"
 |> str/upper
 |> str/reverse)
;; => "DLROW OLLEH"
```

### Railway Pipeline - エラーハンドリング
```lisp
;; 普通の値を返すだけで自動的に{:ok}扱い
(defn validate-positive [x]
  (if (> x 0)
    x                          ;; 普通の値 → 自動で{:ok x}扱い
    {:error "Must be positive"}))

(defn double [x]
  (* x 2))                     ;; 普通の値 → 自動で{:ok ...}扱い

(defn format-result [x]
  f"Result: {x}")              ;; 普通の値 → 自動で{:ok ...}扱い

;; 成功ケース - 普通の値から開始できる！
(10
 |>? validate-positive
 |>? double
 |>? format-result)
;; => {:ok "Result: 20"}

;; エラーケース - エラーは自動的に伝播
(-5
 |>? validate-positive
 |>? double                    ;; 実行されない（ショートサーキット）
 |>? format-result)            ;; 実行されない
;; => {:error "Must be positive"}
```

### 並列パイプライン
```lisp
;; ||> で複数の処理を並列実行
([1 2 3 4 5]
 ||> (fn [x] (* x 2))
 ||> (fn [x] (+ x 10))
 ||> (fn [x] (* x x)))
;; => [144, 196, 256, 324, 400]
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
