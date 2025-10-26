# Qi Language VSCode 拡張機能

[English](./README.md) | [日本語](./README.ja.md)

**Qi - A Lisp that flows** の完全な言語サポート

## 機能

### シンタックスハイライト
- **キーワード**: `def`, `defn`, `defn-`, `fn`, `let`, `if`, `do`, `when`, `while`, `until`, `while-some`, `until-error`, `each`, `match`, `try`, `defer`, `loop`, `recur`, `use`, `export`, `module`
- **キーワードリテラル**: `:keyword` 構文
- **演算子**:
  - パイプライン演算子: `|>`, `||>`, `|>?`, `~>`
  - マッチ矢印: `->`, `=>`
  - スプレッド演算子: `...`
  - 算術演算子: `+`, `-`, `*`, `/`, `%`, `=`, `!=`, `<`, `>`, `<=`, `>=`
- **文字列**:
  - 通常の文字列: `"hello"`
  - 三重引用符文字列: `"""multi-line"""`
  - F文字列: `f"Hello {name}"`
  - 三重引用符F文字列: `f"""multi-line with {interpolation}"""`
- **コメント**: `;` による行コメント
- **数値**: 整数と浮動小数点リテラル
- **定数**: `nil`, `true`, `false`
- **述語**: `nil?`, `list?`, `map?`, `string?`, `integer?` など
- **組み込み関数**: `atom`, `deref`, `swap!`, `reset!`, `print`, `println`, `error`, `tap` など
- **モジュールシステム**: `str/`, `list/`, `map/`, `go/`, `io/`, `http/`, `db/`, `kvs/` などのハイライト

### エディタ機能
- 自動閉じ括弧: `()`, `[]`, `{}`, `""`
- 括弧マッチング
- コメントトグル（`;` 行コメント）

### コマンド
- **Run Qi File** (`Ctrl+F5` / `Cmd+F5`) - 現在のQiファイルを実行
- **Start Qi REPL** (`Ctrl+Shift+R` / `Cmd+Shift+R`) - 対話型REPLを起動
- **Debug Qi File** (`F5`) - ブレークポイントとステップ実行でデバッグ
- **Show Documentation** - Qiドキュメントを開く

### デバッグ
Debug Adapter Protocol (DAP) を使用した完全なデバッグサポート:

- **ブレークポイント**: ガター（行番号の左）をクリックしてブレークポイントを設定
- **ステップ実行**: 関数をステップオーバー、ステップイン、ステップアウト
- **コールスタック**: 現在のコールスタックを表示
- **変数**: 変数とスコープを検査
- **継続/一時停止**: プログラムの実行を制御
- **標準入力**: `.stdin` コマンドでプログラムに入力を送信

#### 基本的なデバッグ
1. `.qi` ファイルを開く
2. 行番号をクリックしてブレークポイントを設定
3. `F5` を押すか、コマンドパレットから "Debug Qi File" を実行
4. デバッグツールバーを使ってコードをステップ実行

#### プログラムへの入力送信
`io/stdin-line` や `io/stdin-lines` を使用するプログラムをデバッグする際、デバッグコンソール経由で入力を送信できます:

**単一行入力**:
```
.stdin Hello, World!
```

**複数行**（改行には `\n` を使用）:
```
.stdin Line 1\nLine 2\nLine 3
```

**タブ区切り値**:
```
.stdin Column1\tColumn2\tColumn3
```

サンプルプログラム:
```qi
;; test-stdin.qi
(println "Enter your name:")
(def name (io/stdin-line))
(println f"Hello, {name}!")
```

1. 2行目にブレークポイントを設定
2. F5でデバッグを開始
3. ブレークポイントまで継続
4. デバッグコンソールを開く
5. `.stdin Alice` と入力してEnter
6. 実行を継続 - "Hello, Alice!" と出力される

**注意**: Qi実行ファイルは `dap-server` フィーチャーでビルドする必要があります:
```bash
cargo build --features dap-server --release
```

## インストール

### ソースから
1. このリポジトリをクローン
2. `qi-vscode` フォルダをVSCode拡張機能ディレクトリにコピー:
   - **Windows**: `%USERPROFILE%\.vscode\extensions`
   - **macOS**: `~/.vscode/extensions`
   - **Linux**: `~/.vscode/extensions`
3. VSCodeをリロード

### 開発
1. 依存関係をインストール:
   ```bash
   cd qi-vscode
   npm install
   ```
2. TypeScriptをコンパイル:
   ```bash
   npm run compile
   ```
3. このフォルダをVSCodeで開く
4. `F5` を押して拡張機能開発ホストを起動
5. `.qi` ファイルを開いて拡張機能をテスト

## 設定

VSCodeの設定で拡張機能を設定できます:

```json
{
  "qi.executablePath": "qi",
  "qi.enableLinting": true,
  "qi.repl.autoStart": false
}
```

### 設定項目

- **`qi.executablePath`** (string, デフォルト: `"qi"`) - Qi実行ファイルのパス。`qi` がPATHにある場合は `"qi"` を使用できます。それ以外の場合はフルパスを指定してください。
- **`qi.enableLinting`** (boolean, デフォルト: `true`) - リンティングを有効化（計画中）。
- **`qi.repl.autoStart`** (boolean, デフォルト: `false`) - Qiファイルを開いたときに自動的にREPLを起動。

## 言語概要

Qiはフロー指向プログラミングに焦点を当てたモダンなLisp方言です:
- データ変換のためのパイプライン演算子
- 分配束縛を含むパターンマッチング
- 非同期/並行プログラミングプリミティブ
- public/privateエクスポートを持つモジュールシステム
- 豊富な標準ライブラリ（HTTP、DB、JSON、CSVなど）

## サンプル

```qi
;; パイプライン処理
(use str :as s)
(use list :only [take filter])

("hello world"
  |> s/upper
  |> (s/split " ")
  |> (filter (fn [w] (> (s/length w) 4)))
  |> first)
;=> "HELLO"

;; 補間付きF文字列
(def name "Alice")
(def age 30)
(println f"Hello, {name}! You are {age} years old.")
;; => Hello, Alice! You are 30 years old.

;; パターンマッチング
(match [1 2 3]
  [] -> "empty"
  [x] -> f"single: {x}"
  [x y ...rest] -> f"x={x}, y={y}, rest={rest}")
;=> "x=1, y=2, rest=(3)"

;; モジュールシステム
(defn- private-helper []
  "このモジュール内でのみ可視")

(defn public-api [data]
  "パブリックAPI関数"
  (private-helper))

(export [public-api])
```

## リンク

- [Qi言語リポジトリ](https://github.com/sanohiro/qi-lang)
- [言語仕様](https://github.com/sanohiro/qi-lang/blob/master/docs/spec/)

## ライセンス

MIT
