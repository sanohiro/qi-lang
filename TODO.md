# TODO - 実装予定機能

このファイルは、将来実装予定の機能や改善点をリストアップしています。

## ✅ 分解機能の拡張（Destructuring Enhancement）- 実装完了

### 実装完了 (2025-01-13)

`...rest`構文やmapの分解が**let、fn/defn、match**のすべてで利用可能になりました。

### 以前の状態

`...rest`構文やmapの分解は**matchパターンマッチング**でのみ利用可能でした。

```qi
;; ✅ matchでは利用可能
(match [1 2 3 4]
  [x ...rest] -> (str "first: " x ", rest: " rest))

(match {:name "Alice" :age 30}
  {:name n :age a} -> (str n " is " a))
```

### 実装内容

以下の機能が実装されました：

#### ✅ 1. letで`...rest`構文

```qi
;; letでrestパターン
(let [[first ...rest] [1 2 3 4]]
  (str "first: " first ", rest: " rest))
;; => "first: 1, rest: (2 3 4)"

(let [[x y ...tail] [10 20 30 40]]
  {:x x :y y :tail tail})
;; => {:x 10, :y 20, :tail (30 40)}
```

#### ✅ 2. letでmapの分解

```qi
;; letでmap分解
(let [{:name n :age a} {:name "Alice" :age 30}]
  (str n " is " a))
;; => "Alice is 30"

;; :as束縛も対応
(let [{:name n :as person} {:name "Bob" :age 25}]
  [n person])
;; => ["Bob" {:name "Bob" :age 25}]
```

#### ✅ 3. fn/defnで`...rest`構文

```qi
;; fnの引数でrestパターン
(defn process-list [[first ...rest]]
  (str "first: " first ", rest: " rest))

(process-list [1 2 3])
;; => "first: 1, rest: (2 3)"
```

#### ✅ 4. fn/defnでmapの分解

```qi
;; fnの引数でmap分解
(defn greet [{:name n :age a}]
  (str n " is " a " years old"))

(greet {:name "Alice" :age 30})
;; => "Alice is 30 years old"
```

### 実装詳細

**変更したファイル**:
- `src/parser.rs`: `parse_defn()`, `parse_defn_private()`, `parse_fn()`に`LBrace`ケースを追加してmap分解に対応
- `SPEC.md`: fn、defn、letのセクションに実装例を追加

**備考**:
- `value.rs`の`FnParam::Map`は既に定義済みでした
- `parser.rs`の`parse_fn_param_vector()`と`parse_fn_param_map()`も既に実装済みでした
- `eval.rs`の`bind_fn_param()`も既に全パターンに対応済みでした
- 今回の実装で追加したのは、fn/defn/defn-のパーサーで`LBrace`トークンを認識してmap分解関数を呼び出す処理のみです

### テスト

`test_destructure.qi`に包括的なテストコードが含まれています。

---

## 🚧 今後の実装予定機能

以下は、SPEC.mdから抽出した未実装機能のリストです。

### 優先度: 高

#### 1. HTTPストリーミングレスポンス

**目的**: 大きなファイル（動画、大きなPDF等）をメモリ効率的に配信

**現状**:
- 現在は全ファイルをメモリに読み込んでからレスポンス
- 10MBを超えるファイルはエラーになる

**実装内容**:
- チャンク単位でのファイル配信
- Rust側: `tokio::fs::File` + `ReaderStream`
- メモリ使用量の削減

**関連**: SPEC.md line 2880-2883

---

### 優先度: 中

#### 2. データベース機能の拡張

**Phase 2 機能**:
- メタデータAPI
  - `db/tables` - テーブル一覧取得
  - `db/columns` - カラム情報取得
  - `db/indexes` - インデックス情報取得
  - `db/foreign-keys` - 外部キー情報取得
- ストアド実行
  - `db/call` - 関数のRETURN値、プロシージャのOUT/INOUT/結果セット対応
- クエリ情報
  - `db/query-info` - クエリのメタデータ取得
- 機能検出
  - `db/supports?` - データベース機能のサポート確認
  - `db/driver-info` - ドライバー情報取得

**Phase 3 機能**:
- コネクションプーリング

**関連**: SPEC.md line 4767-4776

---

#### 3. 正規表現の拡張機能

**実装予定の機能**:
- グループキャプチャ（名前付き・番号付き）
- `match-all` - 全マッチの取得
- `split` - 正規表現による分割
- `compile` - パターンのプリコンパイル
- コールバック置換

**現状**: 基本的な正規表現マッチは実装済み

**関連**: SPEC.md line 5475-5479

---

### 優先度: 低

#### 4. flow DSL - 分岐・合流を含む複雑な流れ

**目的**: 複雑なデータフローを構造化して記述

**現状**:
- 基本的な`flow`は実装済み
- 分岐・合流を含む複雑な流れは未実装

**関連**: SPEC.md line 60, 6396

---

#### 5. その他の標準モジュール

**未実装のモジュール**:
- `http` - HTTPクライアント（基本機能は実装済み、拡張が必要）
- `json` - JSONパース（基本機能は実装済み）
- `tail-stream` - リアルタイムログ監視

**関連**: SPEC.md line 6032-6037, 6986

---

#### 6. PostgreSQL / MySQL ドライバー

**目的**: 複数のデータベースに対応

**現状**: SQLiteのみ実装済み

**実装内容**:
- PostgreSQLドライバー
- MySQLドライバー
- Feature-gatedモジュール（条件付きコンパイル）

**関連**: SPEC.md line 6856-6863

---

### Phase 6以降（低優先度）

#### 7. 名前空間システム

**目的**: 大規模開発での名前衝突の回避

**現状**: グローバル名前空間のみ

**関連**: SPEC.md line 7208-7211

---

## メモ

- ✅ **既に実装済み**: モジュールシステム（use, defn-, export, module）、match orパターン、分解機能
- 実装の優先度は、HTTPストリーミングなどの実用的な機能を優先
- 標準ライブラリの拡張（HTTP、DB、正規表現）は中優先度
- パフォーマンス最適化やエコシステム機能は低優先度
