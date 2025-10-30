# qi-pro エージェント使用ガイド

## 前提条件

- Claude Code CLI がインストールされていること
- Node.js 18以上がインストールされていること

## インストール

```bash
# エージェントディレクトリに移動
cd examples/agent-sdk

# 実行権限を付与（既に付与済みの場合は不要）
chmod +x qi-pro
```

## 基本的な使い方

### 1. 単一ファイルのレビュー

```bash
./qi-pro test-sample.qi
```

エージェントは以下の問題を検出します：

- ネストした関数呼び出し → パイプライン化の提案
- エラーハンドリング不足 → `|>?`の使用提案
- 並列化可能な処理 → `||>`の使用提案
- 述語関数の命名規則違反 → `?`サフィックスの追加
- 複雑な条件分岐 → `match`の使用提案
- モジュール宣言の欠如 → `module`/`export`の追加提案

### 2. ディレクトリ全体のレビュー

```bash
./qi-pro ../cms/
```

CMSサーバーの全ファイルをレビューし、以下を評価します：

- モジュール構造の品質
- パイプライン演算子の使用状況
- エラーハンドリングの一貫性
- コードスタイルの遵守

### 3. 自動修正モード

```bash
# 注意: ファイルを上書きします！バックアップを取ってから実行してください
./qi-pro test-sample.qi --fix
```

エージェントが改善案を自動的に適用します。

## レビュー例

### 入力: test-sample.qi

```qi
;; 問題1: ネストした関数呼び出し
(defn calculate-sum [numbers]
  (reduce + (map (fn [x] (* x x)) (filter (fn [n] (> n 0)) numbers))))

;; 問題2: エラーハンドリングなし
(defn fetch-user-data [url]
  (let [response (http/get url)]
    (let [body (get response :body)]
      (json/parse body))))

;; 問題3: 並列化できるが逐次処理
(defn process-images [images]
  (map (fn [img]
         (let [resized (resize-image img)]
           (compress-image resized)))
       images))
```

### 出力: レビュー結果

```
=== Qi Pro Agent レビュー結果 ===

## ファイル: test-sample.qi

### 問題1: ネストした関数呼び出し (calculate-sum)

**現在のコード**:
```qi
(defn calculate-sum [numbers]
  (reduce + (map (fn [x] (* x x)) (filter (fn [n] (> n 0)) numbers))))
```

**改善案**:
```qi
(defn calculate-sum [numbers]
  (numbers
   |> (filter (fn [n] (> n 0)))
   |> (map (fn [x] (* x x)))
   |> (reduce +)))
```

**理由**:
- Flow-Oriented Programming: データの流れが明確になる
- 可読性向上: 上から下へ処理の流れが読める
- メンテナンス性: 各ステップの追加・削除が容易

---

### 問題2: エラーハンドリング不足 (fetch-user-data)

**現在のコード**:
```qi
(defn fetch-user-data [url]
  (let [response (http/get url)]
    (let [body (get response :body)]
      (json/parse body))))
```

**改善案**:
```qi
(defn fetch-user-data [url]
  (url
   |> http/get
   |>? (fn [resp] (get resp :body))
   |>? json/parse))
```

**理由**:
- Railway Pipeline: `http/get`や`json/parse`が失敗した場合のエラーハンドリング
- `|>?`を使用することで、エラー時は自動的にショートサーキット
- ネストの削減: `let`の多重ネストを解消

---

### 問題3: 並列化の機会 (process-images)

**現在のコード**:
```qi
(defn process-images [images]
  (map (fn [img]
         (let [resized (resize-image img)]
           (compress-image resized)))
       images))
```

**改善案**:
```qi
(defn process-images [images]
  (images ||> resize-image ||> compress-image))
```

**理由**:
- 並列処理: 画像処理はCPU集約的なため`||>`で並列化すべき
- パイプライン: 各画像が`resize-image` → `compress-image`の流れで処理される
- 簡潔性: `map`と`let`のネストが不要

⚠️ 注意: 要素数が少ない（10未満）場合は並列化のオーバーヘッドを考慮してください。

---

## 総評

- ✅ 正しい例: `correct-pipeline-example`は良い実装です
- ❌ 問題点: 6件の改善点を検出
- 📦 モジュール化: `module`宣言と`export`の追加を推奨

## 推奨アクション

1. 全てのネストした関数呼び出しをパイプラインに変換
2. I/O操作（HTTP、JSON）に`|>?`を使用
3. CPU集約的処理（画像処理等）に`||>`を使用
4. 述語関数に`?`サフィックスを追加
5. 複雑な条件分岐を`match`で書き換え
6. `module`宣言と`export`を追加

---

レビュー完了。改善案を適用する場合は`--fix`オプションを使用してください。
```

## 統合例: CMSサーバーのレビュー

```bash
./qi-pro ../cms/
```

### レビュー結果サマリー

```
=== Qi Pro Agent レビュー結果 ===

レビュー対象: 12ファイル

## 全体評価

✅ 優れている点:
- モジュール構造が適切に分割されている
- パイプライン演算子が効果的に使用されている
- Railway Pipelineによるエラーハンドリング
- `use :as`による明確な名前空間管理

⚠️ 改善の余地:
- 一部の関数で`match`が活用できる箇所がある
- 並列処理の機会がいくつか存在

## ファイル別詳細

### handlers/posts.qi

✅ 良い点:
- `|>?`によるエラー伝播が適切
- `match`による明確なルーティング
- 認証ミドルウェアの統合

改善案:
- クエリパラメータの解析部分でパイプラインを活用可能

### models/post.qi

✅ 良い点:
- データベースクエリ結果の処理にパイプライン使用
- 明確な関数名と責務分離

改善案:
- 特になし（良好な実装）

### utils/slug.qi

✅ 良い点:
- 完璧なパイプライン実装
- Flow-Oriented Programmingの模範例

```qi
(defn make-slug [title]
  (title
   |> string/lower
   |> (string/replace " " "-")
   |> (string/replace "[^a-z0-9-]" "")))
```

## 総評

このCMSサーバーはQi言語のベストプラクティスに従った高品質な実装です。
モジュール構造、パイプライン演算子、エラーハンドリングが適切に使用されています。
```

## トラブルシューティング

### エラー: "command not found"

```bash
# 実行権限を確認
ls -la qi-pro

# 実行権限がない場合
chmod +x qi-pro
```

### エラー: "Agent definition not found"

```bash
# qi-pro.mdが同じディレクトリに存在するか確認
ls -la qi-pro.md
```

### Node.jsのバージョンエラー

```bash
# Node.jsバージョン確認
node --version

# 18以上が必要です
# nvmを使用している場合
nvm install 18
nvm use 18
```

## カスタマイズ

### システムプロンプトの変更

`qi-pro.md`を編集してシステムプロンプトをカスタマイズできます：

```markdown
---
name: qi-pro
description: カスタム説明
tool_names:
  - Read
  - Write
  - Edit
  - Bash
---

# カスタムシステムプロンプト
...
```

### 新しいエージェントの作成

```bash
# 新しいエージェント定義を作成
cp qi-pro.md qi-test.md
cp qi-pro qi-test

# qi-test.md を編集してテスト専門エージェントに変更
# name: qi-test
# description: Qi言語のテストコード生成・評価エージェント
```

## 参考リンク

- [Claude Code SDK ドキュメント](https://docs.claude.com/claude-code)
- [Qi言語仕様書](../../docs/spec/README.md)
- [CMSサーバー実装例](../cms/README.md)
