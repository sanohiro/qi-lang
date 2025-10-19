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

## チャット

- 出力は日本語で行うこと
