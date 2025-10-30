# Qi Agent SDK

Claude Code用のQi言語専門エージェント

## 概要

このディレクトリには、Claude CodeのAgent SDKを使用したQi言語専門のコードレビューエージェントが含まれています。

## エージェント

### qi-pro

Qi言語のソースコードを評価し、言語仕様とベストプラクティスに基づいた改善提案を行う専門エージェントです。

**特徴**:
- Flow-Oriented Programmingの推奨
- パイプライン演算子の最適化提案
- 並行・並列処理のレビュー
- パターンマッチングの活用提案
- エラーハンドリングの改善
- モジュール設計の評価
- 標準ライブラリの活用提案

## 使い方

### 基本的な使用法

```bash
# 単一ファイルのレビュー
./qi-pro examples/hello.qi

# ディレクトリ全体のレビュー
./qi-pro examples/cms/

# ヘルプを表示
./qi-pro --help
```

### 自動修正モード

```bash
# 改善案を自動適用（注意: ファイルを上書きします）
./qi-pro examples/my-script.qi --fix
```

## エージェント定義

エージェントは以下のファイルで定義されています：

- `qi-pro.md` - エージェント定義（YAMLフロントマター + システムプロンプト）
- `qi-pro` - Node.jsラッパースクリプト（実行可能）

## レビュー観点

qi-proエージェントは以下の観点でコードをレビューします：

### 1. Flow-Oriented Programming

```qi
;; ✅ 良い例: パイプラインで流れを表現
(data
 |> parse
 |> (filter valid?)
 |> (map transform)
 |> save)

;; ❌ 悪い例: ネストした関数呼び出し
(save (map transform (filter valid? (parse data))))
```

### 2. 並行・並列処理

```qi
;; ✅ 良い例: 大量データの並列処理
(images ||> resize ||> compress ||> save)

;; ⚠️ 注意: 少量データでは並列化のオーバーヘッドが大きい
```

### 3. パターンマッチング

```qi
;; ✅ 良い例: データ構造の分解
(match response
  {:status 200 :body body} -> (process-success body)
  {:status 404} -> nil
  {:status s} -> (log-error s))
```

### 4. エラーハンドリング

```qi
;; ✅ 良い例: Railway Pipeline
(url
 |> http/get
 |>? (fn [resp] (get resp :body))
 |>? json/parse
 |>? validate-data)
```

### 5. モジュール設計

```qi
;; ✅ 良い例: 適切なモジュール分割
(module "auth")

(defn create-token [user] ...)
(defn verify-token [token] ...)

(export create-token verify-token)
```

### 6. 遅延評価

```qi
;; ✅ 良い例: 大きなファイルの効率的処理
(stream/file "large.log")
  |> (stream/filter error-line?)
  |> (stream/take 100)
  |> stream/realize
```

## 参照ドキュメント

エージェントは以下のドキュメントを参照してレビューを行います：

- `docs/spec/README.md` - 言語仕様の索引
- `docs/spec/01-overview.md` - 言語哲学
- `docs/spec/02-flow-pipes.md` - パイプライン演算子
- `docs/spec/03-concurrency.md` - 並行・並列処理
- `docs/spec/04-match.md` - パターンマッチング
- `docs/spec/FUNCTION-INDEX.md` - 全関数の索引

## 実装の詳細

### エージェント定義ファイル（qi-pro.md）

```markdown
---
name: qi-pro
description: Qi言語のコードを専門的に評価・改善する特化エージェント
tool_names:
  - Read
  - Write
  - Edit
  - Bash
---

# システムプロンプト
...
```

### ラッパースクリプト（qi-pro）

Node.jsスクリプトで、以下の処理を行います：

1. エージェント定義（qi-pro.md）を読み込み
2. YAMLフロントマターとシステムプロンプトを抽出
3. コマンドライン引数を解析
4. レビュー対象の.qiファイルを収集
5. Claude Codeへプロンプトを送信

## 例

### レビュー例

**入力コード** (`examples/test.qi`):

```qi
(defn process-users [conn]
  (let [users (db/query conn "SELECT * FROM users")]
    (let [valid-users (filter (fn [u] (not (nil? (get u :email)))) users)]
      (map (fn [u] (str (get u :name) " <" (get u :email) ">")) valid-users))))
```

**レビュー結果**:

```qi
;; 改善案:
;; 1. パイプラインでデータフローを明確に
;; 2. 述語関数を定義して可読性向上
;; 3. f-string構文を活用

(defn valid-user? [user]
  (some? (get user :email)))

(defn format-user [user]
  f"{(get user :name)} <{(get user :email)}>")

(defn process-users [conn]
  (db/query conn "SELECT * FROM users"
   |> (filter valid-user?)
   |> (map format-user)))
```

## 今後の拡張

- [ ] テストコード生成エージェント
- [ ] パフォーマンス最適化エージェント
- [ ] ドキュメント生成エージェント
- [ ] リファクタリング専門エージェント

## ライセンス

このエージェントはQi言語プロジェクトの一部であり、同じライセンスに従います。
