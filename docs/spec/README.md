# Qi言語仕様書

**Qiの完全な言語仕様とリファレンス**

このディレクトリには、Qi言語の実装済み機能のみを記載した仕様書が含まれています。

---

## 📚 目次

**⚡ クイックスタート**: [クイックリファレンス](QUICK-REFERENCE.md) - 1ページでQiの基本を学ぶ

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
  - 特殊形式（def、fn、let、do、if、match、loop/recur、when、while、until、while-some、until-error）
  - 演算子

- **[06-data-structures.md](06-data-structures.md)** - データ構造
  - ベクター、リスト、マップ、セット
  - 高階関数（map、filter、reduce、each）
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
- **[15-stdlib-math.md](15-stdlib-math.md)** - 数学関数
  - べき乗・平方根（pow、sqrt）、丸め（round、floor、ceil）、範囲制限（clamp）
  - 乱数生成（rand、rand-int、random-range、shuffle）
- **[16-stdlib-auth.md](16-stdlib-auth.md)** - 認証・認可 ⭐ NEW
  - JWT（json web token）生成・検証・デコード
  - パスワードハッシュ（Argon2）
- **[17-stdlib-database.md](17-stdlib-database.md)** - データベース ⭐ NEW
  - PostgreSQL接続（クエリ実行、コマンド実行）
  - パラメータ化クエリ、Result型統合

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

## 🔍 関数・演算子索引

### 特殊形式（14個）

- `def`, `defn`, `defn-` - 定義 → [05-syntax-basics.md](05-syntax-basics.md)
- `fn` - 関数定義 → [05-syntax-basics.md](05-syntax-basics.md), [07-functions.md](07-functions.md)
- `let` - ローカル束縛 → [05-syntax-basics.md](05-syntax-basics.md)
- `if`, `do` - 制御構造 → [05-syntax-basics.md](05-syntax-basics.md)
- `when` - 条件が真のときのみ実行 → [05-syntax-basics.md](05-syntax-basics.md)
- `while` - 条件が真の間ループ → [05-syntax-basics.md](05-syntax-basics.md)
- `until` - 条件が真になるまでループ → [05-syntax-basics.md](05-syntax-basics.md)
- `while-some` - nilになるまでループ（束縛付き） → [05-syntax-basics.md](05-syntax-basics.md)
- `until-error` - エラーになるまでループ（束縛付き） → [05-syntax-basics.md](05-syntax-basics.md)
- `loop`, `recur` - ループ → [05-syntax-basics.md](05-syntax-basics.md)
- `match` - パターンマッチング → [04-match.md](04-match.md)
- `try`, `defer` - エラー処理 → [08-error-handling.md](08-error-handling.md)
- `mac` - マクロ → [05-syntax-basics.md](05-syntax-basics.md)
- `module`, `export`, `use` - モジュール → [09-modules.md](09-modules.md)

### パイプライン演算子（5個） ⭐

- `|>` - 逐次パイプ → [02-flow-pipes.md](02-flow-pipes.md)
- `|>?` - Railway Pipeline（エラー処理） → [02-flow-pipes.md](02-flow-pipes.md), [08-error-handling.md](08-error-handling.md)
- `||>` - 並列パイプ → [02-flow-pipes.md](02-flow-pipes.md)
- `~>` - 非同期パイプ → [02-flow-pipes.md](02-flow-pipes.md), [03-concurrency.md](03-concurrency.md)
- `tap>` - 副作用タップ → [02-flow-pipes.md](02-flow-pipes.md)

### コア関数（よく使う）

**数値演算**:
- `+`, `-`, `*`, `/`, `%` - 算術演算 → [05-syntax-basics.md](05-syntax-basics.md)
- `abs`, `min`, `max`, `inc`, `dec`, `sum` - 数値関数 → [06-data-structures.md](06-data-structures.md)
- `=`, `<`, `>`, `<=`, `>=` - 比較演算 → [05-syntax-basics.md](05-syntax-basics.md)

**コレクション**:
- `first`, `rest`, `last`, `nth` - アクセス → [06-data-structures.md](06-data-structures.md)
- `cons`, `conj`, `concat` - 連結 → [06-data-structures.md](06-data-structures.md)
- `take`, `drop`, `filter`, `map`, `reduce`, `each` - 変換 → [06-data-structures.md](06-data-structures.md)
- `sort`, `reverse`, `distinct` - ソート・重複削除 → [06-data-structures.md](06-data-structures.md)

**文字列**:
- `str`, `split`, `join` - 基本操作 → [05-syntax-basics.md](05-syntax-basics.md)
- 60以上の文字列関数 → [10-stdlib-string.md](10-stdlib-string.md)

**述語（23個）**:
- `nil?`, `some?`, `empty?` - nil/存在チェック → [05-syntax-basics.md](05-syntax-basics.md)
- `number?`, `string?`, `list?`, `vector?`, `map?` - 型チェック → [05-syntax-basics.md](05-syntax-basics.md)
- `even?`, `odd?`, `positive?`, `negative?`, `zero?` - 数値述語 → [05-syntax-basics.md](05-syntax-basics.md)
- `error?` - エラー判定 → [05-syntax-basics.md](05-syntax-basics.md), [08-error-handling.md](08-error-handling.md)

**I/O**:
- `print`, `println` - 出力 → [05-syntax-basics.md](05-syntax-basics.md)
- ファイルI/O → [13-stdlib-io.md](13-stdlib-io.md)

**並行処理** ⭐:
- `go/chan`, `go/send!`, `go/recv!` - goroutine風 → [03-concurrency.md](03-concurrency.md)
- `pmap`, `pfilter`, `preduce` - 並列map/filter/reduce → [03-concurrency.md](03-concurrency.md)
- `atom`, `swap!`, `reset!`, `deref` - スレッドセーフな状態管理 → [03-concurrency.md](03-concurrency.md)

### 標準ライブラリ関数

- **HTTP**: `http/get`, `http/post`, `server/serve` → [11-stdlib-http.md](11-stdlib-http.md)
- **JSON/YAML**: `json/parse`, `json/stringify`, `yaml/parse` → [12-stdlib-json.md](12-stdlib-json.md)
- **Math**: `math/pow`, `math/sqrt`, `math/round`, `math/rand` → [15-stdlib-math.md](15-stdlib-math.md)
- **Test**: `test/assert-eq`, `test/run` → [14-stdlib-test.md](14-stdlib-test.md)
- **String**: `string/upper`, `string/lower`, `string/trim`, 他60+ → [10-stdlib-string.md](10-stdlib-string.md)
- **Auth**: `jwt/sign`, `jwt/verify`, `password/hash`, `password/verify` → [16-stdlib-auth.md](16-stdlib-auth.md)
- **Database**: `db/connect`, `db/query`, `db/exec` (PostgreSQL/MySQL/SQLite) → [17-stdlib-database.md](17-stdlib-database.md)

**📑 完全な関数索引**: [FUNCTION-INDEX.md](FUNCTION-INDEX.md) - 全関数の詳細リファレンス（`./scripts/list_qi_functions.sh`で生成）

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

## 🌍 多言語対応

Qiは**エラーメッセージの多言語対応**をサポートしています。

### 使い方

環境変数`QI_LANG`で言語を指定できます：

```bash
# 日本語でエラーメッセージを表示
QI_LANG=ja qi script.qi

# 英語でエラーメッセージを表示（デフォルト）
QI_LANG=en qi script.qi
```

### 例

```bash
# 日本語エラー
$ QI_LANG=ja qi -e '(+ 1 "abc")'
エラー: 数値演算には数値のみを使用できます

# 英語エラー
$ QI_LANG=en qi -e '(+ 1 "abc")'
Error: Numeric operations require numbers only
```

現在サポートされている言語：
- **日本語** (`ja`) - デフォルト（日本人開発者向け）
- **英語** (`en`) - 国際対応

**実装**: `src/i18n.rs` でメッセージを一元管理しています。

---

## 🔗 関連ドキュメント

- **[SPEC.md.archive](../../SPEC.md.archive)** - 元の統合仕様書（アーカイブ）
- **[ROADMAP.md](../../ROADMAP.md)** - 未実装機能とロードマップ
- **[style-guide.md](../style-guide.md)** - コーディングスタイルガイド
- **[README.md](../../README.md)** - プロジェクト全体の説明

---

## 📜 ライセンス

このドキュメントはQi言語プロジェクトの一部であり、同じライセンスに従います。
