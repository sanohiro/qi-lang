## ソースコードルール

### 基本原則
- 必要以上に長ったらしい名前（関数、変数）は使わないこと
- なるべくならコードは短く書くこと
- わかりやすく簡潔に書くこと
- 必ず関数などにはドキュメントコメントを記述すること
- Rustの文化を尊重すること
- ソースはモダンな書き方をすること。なるべく新しい書き方で（その方が簡潔なコードになると思うから）

### コード品質・フォーマット

#### torture test（地獄テスト）
- このテストの指示がされた場合はユーザーが書ける様々な書き方パターンを試すこと
- このテストは処理系の実装漏れをチェックするためのテストです
- このテストは負荷もかけるようなテストもすること
- qiファイルはdocs/specを元に書くこと。ただし明文化されていない。lisp的な書き方も許可する。
- **このテストは構文エラーを除き、qiファイルではなく実装で対応すること**。
- **Lisp系で許可されているLispの振る舞いはqiでも参考にしたい。ただし当然だがqi構文を優先する**。
- このテストはエラー洗い出しのテストです
- このテストでは構文エラーではないが、判断が難しいものはユーザーに問い合わせすること
- このテストでは扱えるデータパターンも色々ためすこと（例えばvectorやlistは片方でなく両方ためすとか、処理できないものはいらないが処理可能と思われるデータは色々な組み合わせでためす。例えばmapの中のvecotr、vectorのmapとか）

#### Clippy（静的解析）
- Rustファイル（`.rs`）の変更が完了したら、`cargo clippy --lib`を実行すること
- 警告が出た場合は、必要に応じて修正すること
- 自動修正可能な警告は`cargo clippy --fix --lib --allow-dirty`で修正できる
- 警告を抑制する場合は、理由をコメントで明記すること（例: `#[allow(clippy::only_used_in_recursion)]`）

#### フォーマット
- Clippyでの品質チェック後、必ず`cargo fmt`を実行すること
- IDEの自動フォーマットと一致させるため、コミット前にフォーマットを適用する
- 複数のRustファイルを変更した場合は、最後にまとめて実行してもよい

### Lazy初期化 (LazyLock)

グローバルな状態管理には`std::sync::LazyLock`を使用する。以下の場合に適用：

#### 使用すべきケース
- **グローバル状態管理**: ログ設定、プロファイラーデータ、グローバルキャッシュなど
- **一度だけ初期化**: 静的な設定、共有リソース、シングルトンパターン
- **スレッドセーフが必要**: 複数スレッドからアクセスされる可能性がある共有データ
- **初期化コストが高い**: 起動時に初期化すると遅延が発生するリソース

#### 実装例
```rust
use std::sync::LazyLock;
use parking_lot::RwLock;

// ✅ 良い例: グローバルログ設定
static LOG_CONFIG: LazyLock<RwLock<LogConfig>> = LazyLock::new(|| {
    RwLock::new(LogConfig {
        level: LogLevel::Info,
        format: LogFormat::Text,
    })
});

// ✅ 良い例: グローバルキャッシュ
static CONNECTIONS: LazyLock<Mutex<HashMap<String, Connection>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
```

#### 避けるべきケース
- **ローカルな初期化**: 関数内で完結する初期化は通常の変数で十分
- **const で書ける場合**: 定数で表現できるものはconstを使う

### 条件付きコンパイル (Feature Gates)

オプショナルな機能や依存クレートには条件付きコンパイルを使用する。

#### モジュールレベルの条件付きコンパイル

モジュール全体が特定の機能に依存する場合、ファイルの先頭に記述：

```rust
//! ZIP圧縮・解凍関数
//!
//! このモジュールは `util-zip` feature でコンパイルされます。

#![cfg(feature = "util-zip")]

use crate::value::Value;
// ... モジュール全体の実装
```

対応する`mod.rs`でのインポート：
```rust
#[cfg(feature = "util-zip")]
pub mod zip;
```

#### 関数レベルの条件付きコンパイル

モジュール内の一部の関数だけが依存する場合：

```rust
// base64, urlencoding, html-escape に依存する関数
#[cfg(feature = "string-encoding")]
use base64::{Engine as _, engine::general_purpose};

#[cfg(feature = "string-encoding")]
pub fn native_to_base64(args: &[Value]) -> Result<Value, String> {
    // ... 実装
}

// feature がない場合の代替実装（オプション）
#[cfg(not(feature = "string-encoding"))]
pub fn native_to_base64(args: &[Value]) -> Result<Value, String> {
    Err("base64 encoding is not available. Enable 'string-encoding' feature.".to_string())
}
```

#### 関数登録の条件付きコンパイル

`mod.rs`の`register_all()`で、feature-gated関数を別ブロックで登録：

```rust
pub fn register_all(env: &Arc<RwLock<Env>>) {
    // 常に有効な関数
    register_native!(env.write(),
        "math/pow" => math::native_pow,
        "math/sqrt" => math::native_sqrt,
        // ...
    );

    // 乱数関数（4個）- std-math feature が必要
    #[cfg(feature = "std-math")]
    register_native!(env.write(),
        "math/rand" => math::native_rand,
        "math/rand-int" => math::native_rand_int,
        "math/random-range" => math::native_random_range,
        "math/shuffle" => math::native_shuffle,
    );
}
```

#### 条件付きコンパイルを使うべきケース

1. **オプショナルな依存クレート**
   - 外部ライブラリに依存する機能（例: `rusqlite`, `reqwest`, `chrono`）

2. **特定環境では不要な機能**
   - 組み込み環境やWASMでは不要な機能（例: ファイルI/O拡張、REPL）

3. **バイナリサイズ削減が必要**
   - 最小構成ビルド（`minimal` feature）で除外したい機能

4. **Pure Rustのみで実装できない機能**
   - C/C++ライブラリ依存（将来的に追加予定の機能）

#### Cargo.tomlでのfeature定義

```toml
[features]
default = ["std-math", "string-encoding", ...]  # 通常ビルドで有効
minimal = ["io-glob"]  # 最小構成（基本的なI/Oのみ）

# 個別機能
std-math = ["dep:rand"]
string-encoding = ["dep:base64", "dep:urlencoding", "dep:html-escape"]
db-sqlite = ["dep:rusqlite"]
```

#### 実装時のチェックリスト

新しい機能を追加する際は以下を確認：

- [ ] オプショナルな依存クレートを使うか？ → `#[cfg(feature = "...")]`
- [ ] グローバル状態を管理するか？ → `LazyLock`
- [ ] スレッドセーフが必要か？ → `LazyLock` + `RwLock`/`Mutex`
- [ ] `mod.rs`で関数登録を条件付きにしたか？
- [ ] `Cargo.toml`の`default` featureに追加したか？（通常ビルドで有効にする場合）
- [ ] 依存クレートを`optional = true`にしたか？
- [ ] ドキュメントコメントにfeature要件を記載したか？

### テスト・品質
- ソースコードは必ずテストすること
- ビルド時に警告が出たら対応してほしい

### 設計・拡張性
- あとから拡張しやすくすること
- 共通化できるものは共通化すること
- プログラミング言語実装のセオリーはなるべく守り、実装の学習もしやすくすること

### 新機能実装前の確認（重要）

**新しい機能を実装する前に、必ず `docs/spec/` を確認すること。**

#### なぜ重要か

既存の設計パターンやインターフェースを無視して実装すると、以下の問題が発生します：

- ✅ **統一インターフェースの破壊**: 既に統一インターフェースが存在するのに、専用関数を作ってしまう
- ✅ **設計の重複**: 同じパターンを異なる方法で実装してしまう
- ✅ **ドキュメントとの不整合**: 仕様と実装が乖離する

#### 実装前のチェックリスト

新しい機能を実装する前に、以下を確認すること：

1. **既存の設計パターンを確認**
   ```bash
   # 関連する仕様ドキュメントを検索
   rg "データベース\|database\|kvs\|redis" docs/spec/

   # 類似機能の実装を検索
   rg "統一インターフェース\|unified interface" docs/spec/
   ```

2. **docs/spec/ の該当セクションを読む**
   - データベース関連 → `docs/spec/17-stdlib-database.md`
   - HTTP関連 → `docs/spec/11-stdlib-http.md`
   - 文字列操作 → `docs/spec/10-stdlib-string.md`
   - エラー処理 → `docs/spec/08-error-handling.md`

3. **既存の統一インターフェースがあるか確認**
   - RDBMS → `db/*` 統一インターフェース（PostgreSQL/MySQL/SQLite）
   - KVS → `kvs/*` 統一インターフェース（Redis/Memcached等）
   - HTTP → `http/*` 統一インターフェース

4. **専用関数が必要か検討**
   - 統一インターフェースで表現できる場合 → 専用関数は作らない（内部ドライバーのみ）
   - 統一インターフェースで表現できない場合のみ → 専用関数を追加（例: Redis Pub/Sub、PostgreSQL COPY）

#### 実装例：データベース機能の追加

❌ **悪い例**（既存設計を無視）:
```rust
// PostgreSQL専用関数を公開してしまう
pub fn native_pg_query(args: &[Value]) -> Result<Value, String> { ... }

// mod.rsで公開登録
register_native!(env.write(), "db/pg-query" => postgres::native_pg_query);
```

✅ **良い例**（既存の統一インターフェースに統合）:
```rust
// PostgreSQLドライバーを実装（内部のみ）
impl DbDriver for PostgresDriver { ... }
impl DbConnection for PostgresConnection { ... }

// db/connectで自動判別（公開インターフェース）
let driver = if url.starts_with("postgresql://") {
    Box::new(PostgresDriver::new())
} else { ... }
```

#### 設計文書の参照順序

1. **`docs/spec/README.md`** - 全体構造を把握
2. **該当カテゴリのmdファイル** - 詳細な設計を確認
3. **既存の実装** - `src/builtins/` で類似機能を検索
4. **設計に従って実装** - 統一インターフェースを尊重

### 並行処理
- 並列、並行をネイティブを第一級としているため、スレッドセーフは常に意識すること

### 依存関係
- クレートはよほどのことがない限りPure Rustのものを使用すること(C/C++のライブラリやコンパイルが必要なものは使用しないこと)

### 言語仕様との整合性

**重要**: SPEC.mdは `docs/spec/` ディレクトリに分割されました。以下の構造を参照すること：

#### 仕様ドキュメント構造

- **実装済み機能**: `docs/spec/` ディレクトリ
  - `README.md` - ドキュメント索引
  - `01-overview.md` - Qiの概要、言語哲学
  - `02-flow-pipes.md` - パイプライン演算子（★売り）
  - `03-concurrency.md` - 並行・並列処理（★売り）
  - `04-match.md` - パターンマッチング（★売り）
  - `05-syntax-basics.md` - 基本構文
  - `06-data-structures.md` - データ構造
  - `07-functions.md` - 関数
  - `08-error-handling.md` - エラー処理
  - `09-modules.md` - モジュールシステム
  - `10-stdlib-string.md` - 文字列操作（60以上の関数）
  - `11-stdlib-http.md` - HTTPクライアント/サーバー
  - `12-stdlib-json.md` - JSON/YAML処理
  - `13-stdlib-io.md` - ファイルI/O（エンコーディング対応）

- **未実装機能**: `ROADMAP.md`
  - 次期実装予定（テストフレームワーク、PostgreSQL/MySQL、認証・認可等）
  - 将来検討（flow DSL、match拡張、JITコンパイル等）

#### ドキュメント更新時の注意

- **実装済み機能を追加した場合**: `docs/spec/` の対応するファイルを更新
- **新しい機能を計画する場合**: `ROADMAP.md` に追加
- **言語の文化を守ること**: Flow-Oriented Programming、シンプルさを重視
- **元のSPEC.md**: アーカイブファイル（`SPEC.md.archive`）として保持、参照不要

## i18nルール（重要）

ユーザー向けのメッセージは必ずi18n化すること。以下のルールを厳守すること：

### 基本原則
- **すべてのエラーメッセージ・UIメッセージはi18n化する**
- プログラムが出力するものは多言語対応すること(今は英語と日本語のみでいい)
- ハードコードされた文字列でErrやformat!を書かない
- `Err("...")`や`format!("...")`の代わりに`fmt_msg(MsgKey::XxxError, &[...])`を使う
- `map_err(|e| format!("...", e))`も`map_err(|e| fmt_msg(MsgKey::XxxError, &[&e.to_string()]))`に変換する

### メッセージの分類
- **MsgKey**: エラーメッセージ用（fmt_msg関数で使用）
  - パーサーエラー、ランタイムエラー、I/Oエラー、HTTPエラーなど
  - 例: `MsgKey::FileNotFound`, `MsgKey::InvalidArgument`
- **UiMsg**: UIメッセージ用（ui_msg/fmt_ui_msg関数で使用）
  - ヘルプメッセージ、バージョン情報、プロンプトなど
  - 例: `UiMsg::Version`, `UiMsg::HelpUsage`

### i18n.rsへの追加手順
1. **MsgKey enumに新しいキーを追加**（コメントで英語の例を書く）
2. **英語メッセージを追加**（`Lang::En`セクション）
3. **日本語メッセージを追加**（`Lang::Ja`セクション）
4. **重複チェック**: 既存のキーで代用できないか確認する
5. **まとめられるか検討**: 似たようなメッセージは共通化できないか考える

### パラメータ埋め込み
- プレースホルダー（`{0}`, `{1}`, `{2}`など）の使用を推奨
- 例: `fmt_msg(MsgKey::FileNotFound, &["/path/to/file"])`
- ただし、パラメータの使用は必須ではない（固定メッセージもOK）

### 例外
- **HTTPサーバーのレスポンスメッセージ**: 英語のままでOK
  - クライアントに返すエラーレスポンスなど
  - 例: `"Not Found"`, `"Internal Server Error"`などはハードコード可
- **テストコードのpanic!やassert!**: i18n化不要

### 実装例
```rust
// ❌ 悪い例
Err("File not found".to_string())
.map_err(|e| format!("Failed to read: {}", e))

// ✅ 良い例
Err(fmt_msg(MsgKey::FileNotFound, &[path]))
.map_err(|e| fmt_msg(MsgKey::FailedToRead, &[&e.to_string()]))
```

## ドキュメンテーションシステム

### @qi-docタグによる言語要素の自動抽出

ソースコードに軽量なマーキング（`@qi-doc`タグ）を入れることで、言語要素（関数、特殊形式、演算子など）を自動抽出できるシステムを導入しています。

#### タグの種類

**組み込み関数用**:
```rust
/// @qi-doc:category <カテゴリ名>
/// @qi-doc:functions <関数リスト>
/// @qi-doc:note <補足情報>
```

**特殊形式用**:
```rust
/// @qi-doc:special-forms
/// @qi-doc:definition def, defn, defn-
/// @qi-doc:control-flow if, do, loop, recur
```

**演算子用**:
```rust
/// @qi-doc:tokens
/// @qi-doc:pipe-operators |>, |>?, ||>, ~>
/// @qi-doc:arrow-operators ->, =>
```

**頻出シンボル・キーワード用**:
```rust
/// @qi-doc:common-symbols
/// @qi-doc:io print, println
/// @qi-doc:collections list, vector, map, filter

/// @qi-doc:common-keywords
/// @qi-doc:result ok, error
/// @qi-doc:http status, body, headers
```

#### 言語要素の一覧取得

```bash
# すべての言語要素（特殊形式、演算子、関数など）を表示
./scripts/list_qi_functions.sh

# 出力例:
# === Qi Language Reference ===
#
# ## Special Forms
#   - definition def, defn, defn-
#   - control-flow if, do, loop, recur
#   - pattern-matching match
#   ...
#
# ## Operators
#   - pipe-operators |>, |>?, ||>, ~>
#   - arrow-operators ->, =>
#   ...
#
# ## Common Symbols
#   - collections list, vector, map, filter
#   ...
#
# ## Built-in Functions by Category
#   ### core/numeric
#     - +, -, *, /, %, abs, min, max
#   ...
```

特定の要素だけ抽出:
```bash
# 特殊形式だけ
rg '@qi-doc:(definition|control-flow|pattern-matching)' src/parser.rs

# 演算子だけ
rg '@qi-doc:.*-operators' src/lexer.rs

# 関数カテゴリだけ
rg '@qi-doc:category' src/builtins/*.rs
```

#### AIとの連携

ClaudeやCodexに以下のように依頼できます：

- 「@qi-docタグから言語仕様の全体像を抽出して」
- 「パイプ演算子の種類を教えて」
- 「特殊形式のリストを出して」
- 「data/jsonカテゴリの関数を教えて」

#### 現在のタグ付け状況

**完全タグ付け済み**:
- ✅ 特殊形式（parser.rs） - def, defn, fn, let, if, match, try, defer, loop, など
- ✅ 演算子（lexer.rs） - `|>`, `|>?`, `||>`, `~>`, `->`, `=>`, など
- ✅ 頻出シンボル（intern.rs） - print, map, filter, first, rest, など
- ✅ 頻出キーワード（intern.rs） - :ok, :error, :status, :body, など

**関数タグ付け済み（8カテゴリ）**:
- `core/numeric` - 基本演算・数値関数
- `core/string` - 文字列基本
- `core/collections` - コレクション操作
- `string` - 文字列拡張（60+関数）
- `data/json`, `data/yaml` - データフォーマット
- `net/http` - HTTPクライアント
- `math` - 数学関数

**未タグ付け**: 残り29の関数ファイル（段階的に追加予定）

### ドキュメント更新ルール

関数の追加や既存のインターフェース変更時は、以下のルールに従ってドキュメントを更新すること。

#### 新しい関数を追加した場合

**必須更新項目**（絶対に更新すること）:
- ✅ **`docs/spec/`** - 言語仕様ドキュメント
  - 該当するカテゴリのファイル（例: `10-stdlib-string.md`, `05-syntax-basics.md`）
  - `FUNCTION-INDEX.md` の関数一覧
- ✅ **`std/docs/`** - 標準ライブラリドキュメント
  - `std/docs/ja/*.qi` - 日本語ドキュメント
  - `std/docs/en/*.qi` - 英語ドキュメント

**更新例**:
```qi
;; std/docs/ja/string.qi に追加
(def __doc__new-function
  {:desc "新しい関数の説明"
   :params [{:name "arg1" :type "string" :desc "引数1の説明"}]
   :returns {:type "string" :desc "戻り値の説明"}
   :examples ["(new-function \"test\") ;=> \"result\""]})
```

#### 既存の関数のインターフェースを変更した場合

関数のシグネチャ（引数、戻り値、動作）を変更した場合は、以下の場所を**確認**し、必要に応じて更新すること:

**確認・更新が必要な場所**:
- 📄 **`README.md`** - 使用例やクイックスタートのコード
- 📄 **`docs/spec/*.md`** - 該当する仕様ドキュメント
- 📄 **`docs/style-guide/`** - スタイルガイド（存在する場合）
- 📄 **`docs/tutorial/`** - チュートリアルのコード例
- 📄 **`std/docs/ja/*.qi`** - 日本語ドキュメントの`:examples`
- 📄 **`std/docs/en/*.qi`** - 英語ドキュメントの`:examples`
- 📄 **`std/templates/`** - テンプレートファイル
- 📄 **`examples/*.qi`** - サンプルコード
- 📄 **ソースコード中のコメント** - Rustファイル内のドキュメントコメントやコード例

**作業フロー**:
1. 変更した関数名で全体を検索（`rg "function-name"`）
2. 見つかった箇所をレビュー
3. 古いインターフェースを使っている場合は更新
4. 動作確認（特に`examples/*.qi`は実行して確認）

**検索例**:
```bash
# 関数名で検索
rg "stream/range" README.md docs/ examples/ std/

# ドキュメントファイル内のコード例を検索
rg "^\s*\(stream/range" docs/ std/docs/ examples/
```

#### チェックリスト

新しい関数を追加したとき:
- [ ] `docs/spec/` の該当カテゴリファイルに追加
- [ ] `docs/spec/FUNCTION-INDEX.md` に追加
- [ ] `std/docs/ja/*.qi` に `__doc__関数名` を追加
- [ ] `std/docs/en/*.qi` に `__doc__関数名` を追加
- [ ] ソースコードの `@qi-doc` タグを更新（該当する場合）

既存の関数を変更したとき:
- [ ] README.md を検索・更新
- [ ] docs/spec/ を検索・更新
- [ ] docs/tutorial/ を検索・更新
- [ ] std/docs/ の `:examples` を更新
- [ ] examples/ を検索・更新・動作確認
- [ ] std/templates/ を検索・更新
- [ ] ソースコード中のコメント例を更新

## チャット

- 出力は日本語で行うこと
