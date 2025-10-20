# Qi言語 完全関数索引

**全ての組み込み関数・特殊形式・演算子の一覧**

> このファイルは `./scripts/list_qi_functions.sh` で自動生成されています。
> 最新の索引を取得するには、プロジェクトルートで以下を実行してください：
> ```bash
> ./scripts/list_qi_functions.sh
> ```

---

## 📖 使い方

このドキュメントは、Qiの全ての言語要素を網羅した索引です。

- **特殊形式**: 言語の基本的な構文要素（def, fn, match等）
- **演算子**: パイプライン演算子など（|>, |>?, ||>等）
- **頻出シンボル**: よく使われる関数（map, filter, reduce等）
- **組み込み関数**: カテゴリ別に整理された全関数

---

## 🔧 特殊形式

Qiの特殊形式は、通常の関数呼び出しとは異なる評価規則を持つ構文要素です。

### 定義 (definition)
- `def` - グローバル変数定義
- `defn` - 関数定義（糖衣構文）
- `defn-` - プライベート関数定義

### 制御フロー (control-flow)
- `if` - 条件分岐
- `do` - 順次実行
- `loop` - ループ構造
- `recur` - 末尾再帰

### 束縛 (binding)
- `let` - ローカル束縛

### 関数 (function)
- `fn` - 匿名関数・ラムダ式

### パターンマッチング (pattern-matching)
- `match` - パターンマッチング

### エラー処理 (error-handling)
- `try` - 例外のキャッチ
- `defer` - リソース解放の保証

### マクロ (macro)
- `mac` - マクロ定義

### モジュール (module)
- `module` - モジュール宣言
- `export` - エクスポート宣言
- `use` - モジュールのインポート
- `flow` - フロー定義（将来実装予定）

---

## ⚡ 演算子

### パイプライン演算子 (pipe-operators) ⭐
- `|>` - 逐次パイプ
- `|>?` - Railway Pipeline（エラー処理）
- `||>` - 並列パイプ（自動pmap化）
- `~>` - 非同期パイプ（goroutine風）

### アロー演算子 (arrow-operators)
- `->` - スレッドファースト
- `=>` - match文のアロー

### クオート演算子 (quote-operators)
- `'` - クオート
- `` ` `` - クオートquote
- `,` - アンクオート
- `,@` - アンクオートスプライス

### パターン演算子 (pattern-operators)
- `|` - orパターン

### 特殊演算子 (special-operators)
- `@` - deref糖衣構文
- `...` - rest構文

---

## 🔤 頻出シンボル

Qi処理系で頻繁に使われるシンボルは、内部で最適化されています。

### アクセサ (accessors)
- `get`, `assoc`

### コレクション (collections)
- `list`, `vector`, `map`, `filter`, `reduce`, `first`, `rest`, `cons`, `concat`

### I/O (io)
- `print`, `println`

### 演算子 (operators)
- `+`, `-`, `*`, `/`, `=`, `<`, `>`, `<=`, `>=`, `not=`

### 述語 (predicates)
- `number?`, `fn?`, `string?`, `list?`, `vector?`, `map?`, `nil?`, `empty?`

---

## 🎯 頻出キーワード

Qi処理系で頻繁に使われるキーワードは、内部で最適化されています。

### データ (data)
- `:name`, `:value`, `:id`, `:type`, `:title`, `:description`, `:data`

### HTTP (http)
- `:status`, `:message`, `:body`, `:headers`, `:method`, `:path`, `:query`, `:params`, `:request`, `:response`

### Result型 (result)
- `:ok`, `:error`

### 時間 (time)
- `:created`, `:updated`, `:timestamp`

---

## 📦 組み込み関数（カテゴリ別）

### args - コマンドライン引数
- `all` - 全引数取得
- `get` - 引数取得
- `parse` - 引数パース
- `count` - 引数数

### cmd - コマンド実行
- `exec` - コマンド実行
- `sh` - シェルコマンド実行
- `pipe` - パイプ実行
- `lines` - 行ごとに実行
- `stream-lines` - ストリーム（行）
- `stream-bytes` - ストリーム（バイト）
- `interactive` - 対話的実行
- `write` - 標準入力に書き込み
- `read-line` - 1行読み込み
- `wait` - プロセス待機

### core/collections - コレクション操作
- `first`, `rest`, `last`, `nth` - アクセス
- `len`, `count` - サイズ
- `cons`, `conj`, `concat` - 連結
- `reverse`, `sort`, `distinct` - 順序・重複
- `range`, `repeat` - 生成
- `take`, `drop`, `zip` - 変換
- 他多数（詳細は[06-data-structures.md](06-data-structures.md)を参照）

### core/functions - 高階関数
- `identity` - 恒等関数
- `constantly` - 定数関数
- `partial` - 部分適用
- `comp` - 関数合成
- `apply` - 関数適用

### core/io-logic - 基本I/O・論理
- `print` - 出力
- `println` - 出力（改行付き）
- `not` - 否定
- `error` - エラー発生

### core/numeric - 数値演算
- `+`, `-`, `*`, `/`, `%` - 四則演算
- `abs`, `min`, `max` - 数値関数
- `inc`, `dec`, `sum` - 増減・合計
- `=`, `<`, `>`, `<=`, `>=` - 比較

### core/predicates - 述語関数（23個）
**型チェック（11個）**:
- `nil?`, `list?`, `vector?`, `map?`, `string?`
- `integer?`, `float?`, `number?`
- `keyword?`, `function?`, `atom?`

**コレクション（3個）**:
- `coll?`, `sequential?`, `empty?`

**状態（4個）**:
- `some?`, `true?`, `false?`, `error?`

**数値（5個）**:
- `even?`, `odd?`, `positive?`, `negative?`, `zero?`

詳細は[05-syntax-basics.md](05-syntax-basics.md)を参照。

### core/state-meta - 状態管理・メタ
- `atom` - Atom作成
- `deref` - 参照解決（`@`も可）
- `swap!` - Atom更新
- `reset!` - Atom設定
- `eval` - 式評価
- `uvar` - 未束縛変数
- `variable` - 変数定義
- `macro?` - マクロ判定

### core/string - 文字列基本
- `str` - 文字列結合
- `split` - 分割
- `join` - 結合

### core/util - ユーティリティ
- `to-int`, `to-float`, `to-string` - 型変換
- `now`, `timestamp` - 時刻
- `sleep` - スリープ

### data/csv - CSV処理
- `parse` - CSVパース
- `stringify` - CSV文字列化
- `read-file` - ファイル読み込み
- `write-file` - ファイル書き込み
- `read-stream` - ストリーム読み込み

### data/json - JSON処理
- `parse` - JSONパース → [12-stdlib-json.md](12-stdlib-json.md)
- `stringify` - JSON文字列化
- `pretty` - 整形出力

### data/yaml - YAML処理
- `parse` - YAMLパース → [12-stdlib-json.md](12-stdlib-json.md)
- `stringify` - YAML文字列化
- `pretty` - 整形出力

### db - データベース
PostgreSQL/MySQL/SQLite対応。詳細は実装ドキュメントを参照。

**接続管理**:
- `connect`, `close`

**クエリ実行**:
- `exec`, `query`, `query-one`
- `prepare`, `exec-prepared`, `query-prepared`

**トランザクション**:
- `begin`, `commit`, `rollback`

**スキーマ操作**:
- `table-list`, `column-list`, `table-exists?`, `column-exists?`
- `create-table`, `drop-table`
- `add-column`, `drop-column`
- `list-indexes`, `create-index`, `drop-index`

**ユーティリティ**:
- `escape-string`, `escape-identifier`

### ds - データ構造（Queue/Stack）
**Queue**:
- `queue/new`, `queue/enqueue`, `queue/dequeue`
- `queue/peek`, `queue/empty?`, `queue/size`

**Stack**:
- `stack/new`, `stack/push`, `stack/pop`
- `stack/peek`, `stack/empty?`, `stack/size`

### env - 環境変数
- `get` - 環境変数取得
- `set` - 環境変数設定
- `all` - 全環境変数
- `load-dotenv` - .envファイル読み込み

### flow/control - フロー制御
- `branch` - 分岐

### fn - 高階関数（拡張）
**変換**:
- `map`, `filter`, `reduce` - 基本変換
- `pmap`, `pfilter`, `preduce` - 並列変換
- `partition`, `group-by` - グループ化
- `map-lines` - 行ごとにmap

**更新**:
- `update`, `update-in` - マップ更新
- `count-by` - カウント集計

**関数生成**:
- `complement` - 述語の否定
- `juxt` - 並置

**副作用**:
- `tap>`, `tap` - 副作用タップ

### go - 並行処理（goroutine風） ⭐
詳細は[03-concurrency.md](03-concurrency.md)を参照。

**チャネル**:
- `chan`, `send!`, `recv!`, `close!`, `chan-closed?`

**Promise**:
- `then`, `catch`

**実行**:
- `go` - goroutine実行
- `pipeline`, `pipeline-map`, `pipeline-filter` - パイプライン
- `select!` - チャネル選択
- `parallel-do` - 並列実行

**状態管理**:
- `atom`, `swap!`, `reset!`, `deref` - Atom
- `scope`, `scope-go`, `with-scope` - スコープ

### io - ファイルI/O
詳細は[13-stdlib-io.md](13-stdlib-io.md)を参照。

**ファイル読み書き**:
- `read-file`, `write-file`, `append-file`, `read-lines`
- `file-stream`, `write-stream`

**ファイル操作**:
- `file-exists?`, `list-dir`, `create-dir`
- `delete-file`, `delete-dir`
- `copy-file`, `move-file`
- `file-info`, `is-file?`, `is-dir?`

**一時ファイル**:
- `temp-file`, `temp-dir`, `cleanup-temp`

### list - リスト操作（拡張）
- `take-while`, `drop-while`, `split-at`
- `interleave`, `frequencies`
- `sort-by`, `chunk`
- `max-by`, `min-by`, `sum-by`
- `find`, `find-index`
- `every?`, `some?`
- `zipmap`, `partition-by`, `take-nth`
- `keep`, `dedupe`, `drop-last`

### log - ロギング
- `debug`, `info`, `warn`, `error` - ログ出力
- `set-level` - ログレベル設定
- `set-format` - ログフォーマット設定

### map - マップ操作
- `select-keys` - キー選択
- `assoc-in` - ネストした値の設定
- `dissoc-in` - ネストした値の削除
- `update-keys` - キー更新
- `update-vals` - 値更新

### markdown - Markdown生成
- `header`, `bold`, `italic` - スタイル
- `code`, `codeblock` - コードブロック
- `link`, `list`, `table` - 構造
- `quote`, `hr` - その他
- `escape` - エスケープ

### math - 数学関数
詳細は[15-stdlib-math.md](15-stdlib-math.md)を参照。

- `pow`, `sqrt` - べき乗・平方根
- `round`, `floor`, `ceil` - 丸め
- `clamp` - 範囲制限

**乱数（std-math feature）**:
- `rand`, `rand-int`, `random-range`, `shuffle`

### net/http - HTTPクライアント
詳細は[11-stdlib-http.md](11-stdlib-http.md)を参照。

**リクエスト**:
- `get`, `post`, `put`, `delete`, `patch`, `head`, `options`
- `request` - 汎用リクエスト

**ストリーミング**:
- `get-stream`, `post-stream`, `request-stream`

### path - パス操作
- `join` - パス結合
- `basename`, `dirname` - パス分解
- `extension`, `stem` - 拡張子
- `absolute`, `normalize` - 正規化
- `is-absolute?`, `is-relative?` - 判定

### profile - プロファイリング
- `enable`, `disable` - プロファイラ制御
- `reset` - リセット
- `report` - レポート出力

### server - HTTPサーバー
詳細は[11-stdlib-http.md](11-stdlib-http.md)を参照。

**サーバー**:
- `serve` - サーバー起動
- `router` - ルーター

**レスポンス**:
- `ok`, `json`, `not-found`, `no-content`

**ミドルウェア**:
- `with-logging`, `with-cors`, `with-json-body`

**静的ファイル**:
- `static-file`, `static-dir`

### set - セット操作
- `union` - 和集合
- `intersection` - 積集合
- `difference` - 差集合
- `subset?`, `superset?` - 部分集合判定

### stats - 統計関数
- `mean`, `median`, `mode` - 代表値
- `stddev`, `variance` - 分散
- `min`, `max`, `sum`, `product` - 基本統計
- `percentile` - パーセンタイル

### stream - 遅延評価ストリーム
詳細は[02-flow-pipes.md](02-flow-pipes.md)を参照。

**生成**:
- `stream`, `range`, `iterate`, `repeat`, `cycle`

**変換**:
- `map`, `filter`, `take`, `drop`

**実行**:
- `realize` - ストリームを実体化

### string - 文字列操作（60以上の関数）
詳細は[10-stdlib-string.md](10-stdlib-string.md)を参照。

主要関数のみ記載。完全なリストはドキュメント参照。

### test - テストフレームワーク
詳細は[14-stdlib-test.md](14-stdlib-test.md)を参照。

**アサーション**:
- `assert-eq`, `assert-ne` - 等価性
- `assert-true`, `assert-false` - 真偽値
- `assert-nil` - nil判定
- `assert-throws` - 例外判定

**実行**:
- `run` - テスト実行
- `summary` - サマリ表示
- `clear` - クリア

### time - 時刻操作
- `now-iso` - 現在時刻（ISO 8601）
- `from-unix`, `to-unix` - Unix時刻変換
- `format` - フォーマット
- `today` - 今日の日付
- `add-days`, `add-hours`, `add-minutes` - 加算
- `sub-days`, `sub-hours`, `sub-minutes` - 減算
- `diff-days`, `diff-hours`, `diff-minutes` - 差分
- `before?`, `after?`, `between?` - 比較
- `parse` - パース
- `year`, `month`, `day`, `hour`, `minute`, `second`, `weekday` - 要素取得

### util - ユーティリティ（拡張）
- `inspect` - デバッグ用の整形出力

### zip - ZIP圧縮
- `create` - ZIP作成
- `extract` - ZIP展開
- `list` - 内容一覧
- `gzip`, `gunzip` - gzip圧縮・展開

---

## 📊 統計

- **関数カテゴリ数**: 38
- **タグ付けファイル数**: 38
- **特殊形式数**: 9
- **演算子グループ数**: 5

---

## 🔄 索引の更新

この索引は、ソースコードの`@qi-doc`タグから自動生成されています。

最新の索引を取得するには：

```bash
cd /Users/hiro/Projects/qi-lang
./scripts/list_qi_functions.sh
```

---

## 📚 関連ドキュメント

- [README.md](README.md) - 仕様書の索引
- [QUICK-REFERENCE.md](QUICK-REFERENCE.md) - クイックリファレンス
- 各種詳細ドキュメント（02-flow-pipes.md等）
