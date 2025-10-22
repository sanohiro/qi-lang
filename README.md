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
- **認証・認可**: JWT認証、Argon2パスワードハッシュ、認証ミドルウェア
- **データベース**: PostgreSQL/MySQL/SQLite対応（統一インターフェース）
- **KVS**: Redis対応（統一インターフェース、将来Memcached/InMemory対応予定）
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
;; {:error}以外は全て成功扱い（:okラップなし！）
(defn validate-positive [x]
  (if (> x 0)
    x                          ;; 普通の値 → 成功
    {:error "Must be positive"}))

(defn double [x]
  (* x 2))                     ;; 普通の値 → 成功

(defn format-result [x]
  f"Result: {x}")              ;; 普通の値 → 成功

;; 成功ケース - 値がそのまま流れる
(10
 |>? validate-positive
 |>? double
 |>? format-result)
;; => "Result: 20"

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

## Quick Start

### インストール

```bash
# Rustがインストールされている場合
cargo install --path .

# または
cargo build --release
```

### プロジェクトを作成

```bash
# 基本的なプロジェクト
qi new my-project
cd my-project
qi main.qi

# HTTPサーバープロジェクト
qi new myapi --template http-server
cd myapi
qi main.qi
```

### 利用可能なテンプレート

```bash
qi template list
qi template info http-server
```

### REPL（対話型実行環境）

```bash
qi

# REPL内で
qi:1> (+ 1 2 3)
6

qi:2> ([1 2 3 4 5] |> (map (fn [x] (* x 2))))
Vector([Integer(2), Integer(4), Integer(6), Integer(8), Integer(10)])
```

### その他のコマンド

```bash
# スクリプトファイル実行
qi script.qi

# ワンライナー実行
qi -e '(+ 1 2 3)'

# ヘルプ表示
qi --help
```

## テスト

### ユニットテスト（高速）

```bash
# 通常のテスト（Dockerなし）
cargo test

# 特定のモジュールをテスト
cargo test parser
cargo test eval
```

### 統合テスト（Docker自動起動）

PostgreSQL、MySQL、Redisの統合テストは、testcontainersを使用してDockerコンテナを自動起動・削除します。

**前提条件**: Dockerがインストールされている必要があります。

```bash
# 統合テスト実行（PostgreSQL + MySQL + Redis）
cargo test --features integration-tests

# 個別実行
cargo test --features integration-tests --test integration_postgres
cargo test --features integration-tests --test integration_mysql
cargo test --features integration-tests --test integration_redis
```

**動作**:
- テスト開始時にDockerコンテナが自動起動（ポート自動割り当て）
- テスト終了時にコンテナが自動削除
- イメージは残るため、次回のテスト実行が高速

## ドキュメント

### はじめに
- **[Lisp系言語の基礎](docs/tutorial/00-lisp-basics.md)** 📚 Lisp初心者向け - 括弧の読み方（5分）
- **[チュートリアル](docs/tutorial/01-getting-started.md)** ⭐ 初心者向け - Qiをはじめよう
- **[CLIリファレンス](docs/cli.md)** - `qi`コマンドの使い方
- **[プロジェクト管理](docs/project.md)** - qi.toml、テンプレート、カスタマイズ

### 言語リファレンス
- **[言語仕様書](docs/spec/)** - Qiの完全な仕様とリファレンス
  - [パイプライン演算子](docs/spec/02-flow-pipes.md) - `|>`, `|>?`, `||>`, `~>`
  - [並行・並列処理](docs/spec/03-concurrency.md) - `go`, `chan`
  - [パターンマッチング](docs/spec/04-match.md) - `match`式
  - [エラー処理](docs/spec/08-error-handling.md) - `try`, `defer`
- **[標準ライブラリ](docs/spec/10-stdlib-string.md)** - 60以上の組み込み関数

## ライセンス

MIT OR Apache-2.0 のデュアルライセンスです。お好きな方を選択してください。

- [LICENSE-MIT](LICENSE-MIT) - MITライセンス
- [LICENSE-APACHE](LICENSE-APACHE) - Apache License 2.0

詳細は各ライセンスファイルを参照してください。
