# Qi CLI リファレンス

Qiコマンドラインツールの完全なリファレンスです。

## 目次

- [基本的な使い方](#基本的な使い方)
- [コマンド一覧](#コマンド一覧)
- [プロジェクト管理](#プロジェクト管理)
- [コード実行](#コード実行)
- [REPL](#repl)
- [環境変数](#環境変数)
- [終了コード](#終了コード)

---

## 基本的な使い方

```bash
qi [OPTIONS] [FILE]
```

引数なしで起動するとREPLモードになります。

---

## コマンド一覧

### プロジェクト管理

#### `qi new <name> [OPTIONS]`

新しいQiプロジェクトを作成します。

**引数:**
- `<name>` - プロジェクト名（ディレクトリ名）

**オプション:**
- `-t, --template <template>` - 使用するテンプレート（デフォルト: `basic`）

**例:**
```bash
# 基本的なプロジェクトを作成
qi new my-project

# HTTPサーバープロジェクトを作成
qi new myapi --template http-server
qi new myapi -t http-server
```

**動作:**
1. プロジェクトディレクトリを作成
2. プロジェクトメタデータを対話的に入力（名前、バージョン、説明、著者、ライセンス）
3. テンプレートからファイルをコピー
4. `qi.toml` を生成

---

#### `qi template list`

利用可能なテンプレート一覧を表示します。

**例:**
```bash
qi template list
```

**出力例:**
```
利用可能なテンプレート:
  basic            - 基本的なプロジェクト構造
  http-server      - JSON APIサーバー with ルーティング
```

---

#### `qi template info <name>`

テンプレートの詳細情報を表示します。

**引数:**
- `<name>` - テンプレート名

**例:**
```bash
qi template info http-server
```

**出力例:**
```
Template: http-server
Description: JSON APIサーバー with ルーティング
Author: Qi Team
Version: 0.1.0
Required features: http-server, format-json
Location: std/templates/http-server
```

---

### コード実行

#### `qi <file>`

Qiスクリプトファイルを実行します。

**引数:**
- `<file>` - 実行するスクリプトファイル（`.qi`）

**例:**
```bash
qi script.qi
qi main.qi
qi examples/example.qi
```

---

#### `qi -e <code>` / `qi -c <code>`

コードを直接実行します（ワンライナー）。

**オプション:**
- `-e, -c <code>` - 実行するQiコード

**stdin自動バインド:**

パイプから入力がある場合、自動的に`stdin`変数に全データ（文字列のベクター）が格納されます：

```bash
# 行数をカウント
cat data.txt | qi -e '(count stdin)'

# CSVデータを処理（空行を除外）
cat users.csv | qi -e '(stdin |> (filter (fn [x] (> (len x) 0))))'

# 空行を除外して大文字変換
echo -e "hello\n\nworld" | qi -e '(stdin |> (filter (fn [x] (> (len x) 0))) |> (map str/upper))'
```

**例:**
```bash
qi -e '(+ 1 2 3)'
# => 6

qi -e '(println "Hello, Qi!")'
# => Hello, Qi!

qi -c '([1 2 3 4 5] |> (map (fn [x] (* x 2))) |> (reduce + 0))'
# => 30
```

---

#### `qi -`

標準入力からコードを読み込んで実行します。

**例:**
```bash
echo '(println 42)' | qi -

cat script.qi | qi -
```

---

### REPL

#### `qi` (引数なし)

対話型REPLを起動します。

**例:**
```bash
qi
```

**REPLコマンド:**

**基本コマンド:**
- `:help` - ヘルプを表示
- `:doc <name>` - 関数のドキュメントを表示（カラー表示、戻り値、関連関数、類似関数の提案）
- `:vars` - 定義されている変数を表示
- `:funcs` - 定義されている関数を表示
- `:builtins [filter]` - 組み込み関数を表示（フィルタ可能）
- `:clear` - 環境をクリア
- `:load <file>` - ファイルを読み込む
- `:reload` - 最後に読み込んだファイルを再読み込み
- `:quit` - REPLを終了

**テスト・デバッグ:**
- `:test [path]` - テストファイルを実行（引数なしで全テスト）
- `:trace <function>` - 関数呼び出しをトレース（引数なしでトレース中の関数一覧を表示）
- `:untrace [function]` - トレースを停止（引数なしで全停止）

**ホットリロード（ファイル監視）:**
- `:watch <file>` - ファイルを監視して変更時に自動再読み込み
- `:unwatch [file]` - ファイル監視を停止（引数なしで全停止）

**マクロ（頻繁な操作の自動化）:**
- `:macro` - マクロ一覧を表示
- `:macro define <name> <command>` - マクロを定義（`~/.qi/macros`に保存）
- `:macro list` - マクロ一覧を表示
- `:macro delete <name>` - マクロを削除
- `:m <name>` - マクロを実行（短縮コマンド）

**プロファイリング:**
- `:profile start` - プロファイリングを開始
- `:profile stop` - プロファイリングを停止
- `:profile report` - 統計レポートを表示（合計、平均、最大、最小時間、最も遅い評価）
- `:profile clear` - プロファイリングデータをクリア

**並行処理デバッグ:**
- `:threads` - Rayonスレッドプール情報とアクティブなチャンネルのステータスを表示

**機能:**

**入力支援:**
- **Inline Hints** - 履歴から候補を自動表示（薄いグレー、→キーで補完）
- **タブ補完** - 関数名、変数名、REPLコマンド、特殊形式、パイプ演算子
- **Bracketed Paste** - 大量のコードを安全にコピペ

**シンタックスハイライト:**
- 特殊形式、演算子、文字列、数値、コメントを色分け
- **Rainbow Parentheses** - 括弧のネストレベルごとに色を変更（6色のレインボー）

**履歴管理:**
- 履歴の永続化（`~/.qi/history`）
- 重複履歴の自動除外
- **Ctrl+R** - インクリメンタル履歴検索
- **Alt+N / Alt+P** - 履歴の前後移動

**編集機能:**
- 複数行入力（括弧のバランスを自動判定）
- **Ctrl+_** - Undo
- **Alt+F / Alt+B** - 単語単位でのカーソル移動

**表示機能:**
- 結果のラベル表示（`$1`, `$2`で過去の結果を参照可能）
- 評価時間の自動表示（ミリ秒/マイクロ秒）
- テーブル形式での自動表示（`Vector<Map>`を検出）
- エラーメッセージのカラー表示

**その他:**
- Ctrl+C で入力キャンセル
- Ctrl+D または `:quit` で終了

---

#### `qi -l <file>` / `qi --load <file>`

ファイルを読み込んでからREPLを起動します。

**オプション:**
- `-l, --load <file>` - 読み込むファイル

**例:**
```bash
qi -l utils.qi
qi --load lib.qi
```

---

### その他のオプション

#### `qi -h` / `qi --help`

ヘルプメッセージを表示します。

```bash
qi --help
qi -h
```

---

#### `qi -v` / `qi --version`

バージョン情報を表示します。

```bash
qi --version
qi -v
```

---

#### `qi --upgrade`

Qiを最新版にアップグレードします（GitHub Releasesから取得）。

```bash
qi --upgrade
```

**動作:**
1. GitHubから最新リリースを確認
2. プラットフォームに対応したバイナリをダウンロード
3. 現在のバイナリを置き換え（バックアップを`.old`に作成）
4. アップグレード状況を表示

**対応プラットフォーム:**
- macOS (Apple Silicon / Intel)
- Linux (x86_64 / aarch64)
- Windows (x86_64)

**使用例:**
```bash
# 最新版にアップグレード
qi --upgrade

# 現在のバージョンを確認
qi --version
```

---

## 環境変数

### `QI_LANG`

Qiのメッセージ言語を設定します。

**値:**
- `ja` - 日本語
- `en` - 英語（デフォルト）

**例:**
```bash
QI_LANG=ja qi script.qi
QI_LANG=en qi --help
```

**動作:**
- エラーメッセージ、ヘルプメッセージ、REPLメッセージが指定された言語で表示される
- ドキュメント（`:doc`）も指定された言語で表示される

---

### `LANG`

システムのロケール設定。`QI_LANG`が設定されていない場合のフォールバック。

**例:**
```bash
LANG=ja_JP.UTF-8 qi script.qi
```

---

## 終了コード

| コード | 意味 |
|--------|------|
| `0` | 正常終了 |
| `1` | エラー発生（構文エラー、実行時エラー、ファイルが見つからない等） |

**例:**
```bash
qi script.qi
echo $?  # 終了コードを確認

qi -e '(+ 1 2 3)'
echo $?  # => 0

qi -e '(invalid syntax'
echo $?  # => 1
```

---

## テンプレート検索順序

`qi new`コマンドは、以下の順序でテンプレートを検索します：

1. `./.qi/templates/<name>/` - プロジェクトローカル
2. `~/.qi/templates/<name>/` - ユーザーグローバル
3. `<qi-binary-dir>/std/templates/<name>/` - インストール版
4. `std/templates/<name>/` - 開発版

最初に見つかったテンプレートが使用されます。

---

## 使用例

### プロジェクト作成の流れ

```bash
# テンプレート一覧を確認
qi template list

# HTTPサーバープロジェクトを作成
qi new myapi --template http-server

# プロジェクトディレクトリに移動
cd myapi

# サーバーを起動
qi main.qi
```

### ワンライナーでのデータ処理

```bash
# フィボナッチ数列
qi -e '(defn fib [n] (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2))))) (fib 10)'

# パイプライン処理
qi -e '([1 2 3 4 5] |> (filter (fn [x] (> x 2))) |> (map (fn [x] (* x 10))))'

# JSON処理（HTTP featureが有効な場合）
qi -e '(json/parse "{\"name\":\"Alice\",\"age\":30}")'
```

### REPLでの開発

```bash
# ライブラリを読み込んでREPL起動
qi -l src/lib.qi

# REPL内で関数をテスト
qi:1> (greet "World")
$1 => こんにちは、Worldさん！

# 過去の結果を参照
qi:2> $1
$2 => こんにちは、Worldさん！

# ドキュメントを確認（カラー表示、戻り値、関連関数も表示）
qi:3> :doc map

# 組み込み関数を検索
qi:4> :builtins str

# ファイルをホットリロード（開発中に便利）
qi:5> :watch src/lib.qi
Watching: src/lib.qi
# => ファイルを編集すると自動的に再読み込みされる

# よく使うコマンドをマクロに登録
qi:6> :macro define test (run-tests)
Macro 'test' defined: (run-tests)

qi:7> :m test
[Running macro 'test': (run-tests)]

# プロファイリングで性能を測定
qi:8> :profile start
Profiling started

qi:9> (heavy-computation 1000)
$3 => 42
(15ms)

qi:10> :profile report
Profiling Report:
  Total evaluations: 5
  Total time: 23ms
  Average time: 4.6ms
  Min time: 100µs
  Max time: 15ms

Slowest evaluations:
  1. Line 9 - 15ms
  2. Line 8 - 5ms
  ...

# 並行処理のデバッグ
qi:11> :threads
Rayon Thread Pool:
  Available parallelism: 8

Active Channels:
  ch - len: 0, is_empty: true

# テストファイルを実行
qi:12> :test tests/math_test.qi
Running tests in tests/math_test.qi...
テスト結果:
  ✓ addition
  ✓ subtraction
  ✓ multiplication

# 関数のトレース（デバッグ用）
qi:13> (defn factorial [n] (if (<= n 1) 1 (* n (factorial (- n 1)))))
qi:14> :trace factorial
Tracing function: factorial
qi:15> (factorial 3)
→ factorial(Integer(3))
→ factorial(Integer(2))
→ factorial(Integer(1))
$4 => 6

qi:16> :untrace factorial
Stopped tracing: factorial

# マクロ展開を確認
qi:17> (macroexpand '(defn add [a b] (+ a b)))
$5 => (def add (fn [a b] (+ a b)))

# 関数のソースコードを表示
qi:18> (source '+)
$6 => Native function: +
Implemented in: src/builtins/

This is a built-in function implemented in Rust.
Use :doc + to see documentation.
```

---

## 関連ドキュメント

- [プロジェクト管理とqi.toml仕様](project.md) - qi.toml、テンプレート、プロジェクト構造
- [チュートリアル](tutorial/01-getting-started.md) - 実践的な使い方
- [言語仕様](spec/README.md) - Qi言語の文法と機能
