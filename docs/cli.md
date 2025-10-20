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
- `:help` - ヘルプを表示
- `:doc <name>` - 関数のドキュメントを表示
- `:vars` - 定義されている変数を表示
- `:funcs` - 定義されている関数を表示
- `:builtins [filter]` - 組み込み関数を表示（フィルタ可能）
- `:clear` - 環境をクリア
- `:load <file>` - ファイルを読み込む
- `:reload` - 最後に読み込んだファイルを再読み込み
- `:quit` - REPLを終了

**機能:**
- タブ補完（関数名、変数名、REPLコマンド）
- 履歴（`~/.qi_history` に保存）
- 複数行入力（括弧のバランスを自動判定）
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
こんにちは、Worldさん！

# ドキュメントを確認
qi:2> :doc map

# 組み込み関数を検索
qi:3> :builtins str
```

---

## 関連ドキュメント

- [プロジェクト管理とqi.toml仕様](project.md) - qi.toml、テンプレート、プロジェクト構造
- [チュートリアル](tutorial/01-getting-started.md) - 実践的な使い方
- [言語仕様](spec/README.md) - Qi言語の文法と機能
