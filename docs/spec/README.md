# Qi言語仕様書

**Qiの完全な言語仕様とリファレンス**

このディレクトリには、Qi言語の実装済み機能のみを記載した仕様書が含まれています。

---

## 📚 目次

### コア機能（★ウリ）

- **[02-flow-pipes.md](02-flow-pipes.md)** - パイプライン演算子とデータフロー ⭐
  - `|>`, `||>`, `|>?`, `tap>`, `~>` 演算子
  - stream（遅延評価）
  - データの流れを設計する

- **[03-concurrency.md](03-concurrency.md)** - 並行・並列処理 ⭐
  - go/chan（goroutine風）
  - async/await、pmap、pipeline
  - Atom（スレッドセーフな状態管理）

- **[04-match.md](04-match.md)** - パターンマッチング ⭐
  - データ構造の分解
  - ガード条件、orパターン
  - Railway Oriented Programming

### 基本

- **[01-overview.md](01-overview.md)** - Qiの概要
  - 言語哲学（Flow-Oriented Programming）
  - 設計原則
  - 基本設計

- **[05-syntax-basics.md](05-syntax-basics.md)** - 基本構文
  - データ型、リテラル、コメント
  - 特殊形式（def、fn、let、do、if、match、loop/recur）
  - 演算子

- **[06-data-structures.md](06-data-structures.md)** - データ構造
  - ベクター、リスト、マップ、セット
  - 高階関数（map、filter、reduce）
  - ソート、グループ化

- **[07-functions.md](07-functions.md)** - 関数
  - 関数定義（fn、defn）
  - クロージャ
  - 高階関数（comp、partial、apply、identity）

- **[08-error-handling.md](08-error-handling.md)** - エラー処理
  - Result型（{:ok/:error}）
  - try/catch
  - defer（リソース管理）

- **[09-modules.md](09-modules.md)** - モジュールシステム
  - module宣言、export
  - use、load
  - 名前空間管理

### 標準ライブラリ

- **[10-stdlib-string.md](10-stdlib-string.md)** - 文字列操作（60以上の関数）
  - 検索、変換、ケース変換、エンコード、バリデーション
- **[11-stdlib-http.md](11-stdlib-http.md)** - HTTPクライアント/サーバー
  - クライアント（GET/POST/PUT/DELETE）、サーバー（ルーティング、ミドルウェア）
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSON/YAML処理
  - パース、stringify、Result型統合
- **[13-stdlib-io.md](13-stdlib-io.md)** - ファイルI/O（エンコーディング対応）
  - ファイル読み書き、多言語エンコーディング（Shift_JIS、GBK、Big5等）
- **[14-stdlib-test.md](14-stdlib-test.md)** - テストフレームワーク ⭐ NEW
  - test/run、アサーション（assert-eq、assert、assert-not、assert-throws）
  - qi testコマンド（自動検出、シンプルな出力）

---

## 🎯 Qiの特徴

### 1. Flow-Oriented Programming

**「データは流れ、プログラムは流れを設計する」**

```qi
(data
 |> parse
 |> (filter valid?)
 |> (map transform)
 |> save)
```

### 2. 並行・並列を簡単に

**Qiのキモ - スレッドセーフで自然な並列化**

```qi
;; 並列パイプライン
(urls ||> http/get ||> json/parse)

;; goroutine風の並行処理
(def result (data ~> transform ~> process))
(recv! result)
```

### 3. パターンマッチング

**データの流れを分岐・変換**

```qi
(match response
  {:ok {:status 200 :body body}} -> (process-body body)
  {:ok {:status 404}} -> nil
  {:error e} -> (log-error e))
```

---

## 📖 ドキュメントの読み方

### 初心者向け

1. [01-overview.md](01-overview.md) - Qiとは何か？
2. [05-syntax-basics.md](05-syntax-basics.md) - 基本的な構文を学ぶ
3. [06-data-structures.md](06-data-structures.md) - データの扱い方
4. [02-flow-pipes.md](02-flow-pipes.md) - パイプラインを使ってみる
5. [10-stdlib-string.md](10-stdlib-string.md) - 文字列操作を学ぶ

### 中級者向け

1. [04-match.md](04-match.md) - パターンマッチングを活用
2. [07-functions.md](07-functions.md) - 関数型プログラミング
3. [08-error-handling.md](08-error-handling.md) - エラー処理の戦略
4. [03-concurrency.md](03-concurrency.md) - 並行処理を活用
5. [11-stdlib-http.md](11-stdlib-http.md) - HTTPクライアント/サーバーを作る
6. [13-stdlib-io.md](13-stdlib-io.md) - ファイルI/Oとエンコーディング

### 上級者向け

1. [03-concurrency.md](03-concurrency.md) - 3層並行処理アーキテクチャ
2. [09-modules.md](09-modules.md) - モジュール設計
3. [02-flow-pipes.md](02-flow-pipes.md) - stream（遅延評価）
4. [12-stdlib-json.md](12-stdlib-json.md) - JSON/YAMLパイプライン処理

---

## 🚀 未実装機能について

未実装機能やロードマップについては、プロジェクトルートの`ROADMAP.md`を参照してください。

---

## 📝 ドキュメントの方針

このディレクトリのドキュメントは：

- **実装済み機能のみを記載** - 全てのコード例は動作します
- **Phase表記なし** - 全て実装済みのため、Phase表記は削除
- **実用例重視** - 概念だけでなく、実際に使えるコード例を提供
- **Flow-Oriented** - Qiの哲学に沿った説明

---

## 🔗 関連ドキュメント

- **[SPEC.md.archive](../../SPEC.md.archive)** - 元の統合仕様書（アーカイブ）
- **[ROADMAP.md](../../ROADMAP.md)** - 未実装機能とロードマップ
- **[style-guide.md](../style-guide.md)** - コーディングスタイルガイド
- **[README.md](../../README.md)** - プロジェクト全体の説明

---

## 📜 ライセンス

このドキュメントはQi言語プロジェクトの一部であり、同じライセンスに従います。
