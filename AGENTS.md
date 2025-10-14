# AGENTS.md

**AI開発者向けガイド - Qiプログラミング言語**

このドキュメントは、AIエージェント（LLMベースの開発支援ツール）がQi言語プロジェクトを支援する際のガイドラインです。

---

## プロジェクト概要

**Qi（キー）** は、Flow-Oriented Programmingを核とした実用的なLisp方言です。

### 言語の特徴

1. **Flow-Oriented Programming** - データは流れ、プログラムは流れを設計する
   - パイプライン演算子: `|>`, `||>`, `|>?`, `tap>`, `~>`
   - Railway Oriented Programming（Result型統合）
   - データフローを可視化する構文

2. **並行・並列処理の第一級サポート**
   - 3層アーキテクチャ: go/chan（Layer 1）、pipeline（Layer 2）、async/await（Layer 3）
   - スレッドセーフな設計（Arc<RwLock<_>>）
   - 自然な並列化（`||>`、`pmap`）

3. **パターンマッチング**
   - データ構造の分解
   - ガード条件、orパターン、:as束縛
   - Railway Pipeline（`|>?`）との統合

4. **シンプルさ**
   - 9つの特殊形式のみ（def, defn, fn, let, do, if, match, loop/recur, mac）
   - Lisp-1名前空間
   - 実用性重視の設計

---

## ドキュメント構造

### 実装済み機能: `docs/spec/`

| ファイル | 内容 |
|---------|------|
| `README.md` | ドキュメント索引、読み方ガイド |
| `01-overview.md` | Qiの概要、言語哲学 |
| `02-flow-pipes.md` | パイプライン演算子（★売り） |
| `03-concurrency.md` | 並行・並列処理（★売り） |
| `04-match.md` | パターンマッチング（★売り） |
| `05-syntax-basics.md` | 基本構文、特殊形式 |
| `06-data-structures.md` | ベクター、リスト、マップ、セット |
| `07-functions.md` | 関数、クロージャ、高階関数 |
| `08-error-handling.md` | Result型、try/catch、defer |
| `09-modules.md` | モジュールシステム |
| `10-stdlib-string.md` | 文字列操作（60以上の関数） |
| `11-stdlib-http.md` | HTTPクライアント/サーバー |
| `12-stdlib-json.md` | JSON/YAML処理 |
| `13-stdlib-io.md` | ファイルI/O（エンコーディング対応） |

**重要**: `docs/spec/`に記載されているのは**実装済み機能のみ**です。すべてのコード例は動作します。

### 未実装機能: `ROADMAP.md`

未実装機能や将来の計画は `ROADMAP.md` を参照してください：
- **優先度高**: テストフレームワーク、PostgreSQL/MySQL、認証・認可、ファイル監視等
- **優先度中**: flow DSL、match拡張、regex拡張、タイムゾーン対応
- **優先度低**: JITコンパイル、名前空間システム

---

## コーディング規約

### 基本原則

1. **簡潔性**: 冗長なコードを避け、必要最小限の記述で目的を達成する
2. **Flow-Oriented**: データの流れを意識した設計
3. **スレッドセーフ**: 並行処理を常に考慮
4. **Rustの文化を尊重**: モダンなRustコードを書く
5. **Pure Rust**: C/C++依存ライブラリは避ける

### Rustコーディングスタイル

- **必ずドキュメントコメントを書く**（`///` or `//!`）
- **変更後は `cargo fmt` を実行**
- **グローバル状態は `LazyLock` を使用**
- **オプショナル機能は条件付きコンパイル** (`#[cfg(feature = "...")]`)

### i18n（国際化）

すべてのユーザー向けメッセージは多言語対応すること：
- エラーメッセージ: `fmt_msg(MsgKey::XxxError, &[...])`
- UIメッセージ: `ui_msg(UiMsg::XxxMessage)`
- ハードコードされた文字列は禁止

---

## ビルド・テスト

### 基本コマンド

```bash
# ビルド
cargo build

# テスト
cargo test

# フォーマット
cargo fmt

# 実行
cargo run -- -e '(+ 1 2 3)'
```

### Feature Flags

```bash
# 最小構成ビルド
cargo build --no-default-features --features minimal

# 特定機能を有効化
cargo build --features db-sqlite,http-server
```

---

## 実装時のチェックリスト

新しい機能を追加する際は、以下を確認：

- [ ] **ドキュメント更新**: `docs/spec/` の対応ファイルを更新
- [ ] **テスト追加**: 機能追加時は必ずテストを書く
- [ ] **i18n対応**: ユーザー向けメッセージは多言語化
- [ ] **Feature Gate**: オプショナルな機能は条件付きコンパイル
- [ ] **スレッドセーフ**: 並行処理でも安全か確認
- [ ] **言語文化**: Flow-Oriented Programmingの原則に従う
- [ ] **フォーマット**: `cargo fmt` を実行
- [ ] **警告解消**: コンパイル警告がないことを確認

---

## 言語文化

Qiの設計原則を理解し、尊重してください：

### Flow-Oriented Programming

```qi
;; データは流れる
(data
 |> parse
 |> (filter valid?)
 |> (map transform)
 |> save)

;; エラーも流れる（Railway Pipeline）
(url
 |> http/get
 |>? json/parse
 |>? (fn [data] {:ok (get data "result")}))
```

### シンプルさ

- 複雑な機能より、組み合わせ可能なシンプルな機能
- 特殊形式は増やさない（現在9個のみ）
- データ駆動設計（コードではなくデータで表現）

### 実用性

- 理論より実用性を優先
- Web開発、データ処理、スクリプティングに最適化
- 学習コストを最小限に

---

## サンプルコード

### パイプライン処理

```qi
;; GitHub APIからユーザー情報を取得
("https://api.github.com/users/octocat"
 |> http/get
 |>? (fn [resp] {:ok (get resp "body")})
 |>? json/parse
 |>? (fn [data] {:ok (get data "name")}))
;; => {:ok "The Octocat"}
```

### 並列処理

```qi
;; 複数URLを並列取得
(def urls ["https://api.github.com/users/user1"
           "https://api.github.com/users/user2"
           "https://api.github.com/users/user3"])

(urls ||> http/get ||> json/parse)
;; => [result1 result2 result3]
```

### パターンマッチング

```qi
(match response
  {:ok {:status 200 :body body}} -> (process-body body)
  {:ok {:status 404}} -> nil
  {:error e} -> (log/error e))
```

---

## 開発時の注意

### やるべきこと

- **実装済み機能を追加**: `docs/spec/` を更新
- **未実装機能を計画**: `ROADMAP.md` に追加
- **テストを書く**: すべての機能にテストを追加
- **フォーマット**: `cargo fmt` を実行

### やってはいけないこと

- **Phase表記の追加**: `docs/spec/` にPhaseマーカー（✅、🚧）を書かない
- **未実装機能の記載**: `docs/spec/` には実装済み機能のみ
- **C/C++依存**: Pure Rustクレートのみ使用
- **ハードコード**: ユーザー向けメッセージは必ずi18n化

---

## リファレンス

- **言語仕様**: `docs/spec/README.md`
- **ロードマップ**: `ROADMAP.md`
- **スタイルガイド**: `docs/style-guide.md`
- **プロジェクト概要**: `README.md`
- **開発者向け詳細**: `CLAUDE.md`（Claude Code専用）

---

## 問い合わせ

このドキュメントは、AIエージェントがQiプロジェクトを理解し、開発を支援するためのものです。人間の開発者は `README.md` と `CLAUDE.md` を参照してください。
