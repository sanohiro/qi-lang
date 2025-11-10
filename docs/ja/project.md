# Qiプロジェクト管理

Qiプロジェクトの構造、設定ファイル（qi.toml）、テンプレートシステムの完全なリファレンスです。

## 目次

- [プロジェクト構造](#プロジェクト構造)
- [qi.toml仕様](#qitoml仕様)
- [テンプレートシステム](#テンプレートシステム)
- [カスタムテンプレートの作成](#カスタムテンプレートの作成)

---

## プロジェクト構造

標準的なQiプロジェクトは以下の構造を持ちます：

```
my-project/
├── qi.toml           # プロジェクト設定ファイル
├── main.qi           # エントリーポイント
├── src/              # ライブラリコード
│   └── lib.qi
├── examples/         # サンプルコード
│   └── example.qi
└── tests/            # テストコード
    └── test.qi
```

### 各ファイル・ディレクトリの役割

#### `qi.toml`
プロジェクトのメタデータと設定を記述するTOML形式のファイル。詳細は[qi.toml仕様](#qitoml仕様)を参照。

#### `main.qi`
プロジェクトのエントリーポイント。`qi main.qi`で実行されます。

#### `src/`
再利用可能なライブラリコードを配置するディレクトリ。将来的にモジュールシステムで読み込めるようになる予定。

#### `examples/`
使用例やデモコードを配置するディレクトリ。ドキュメント目的や動作確認に使用。

#### `tests/`
テストコードを配置するディレクトリ。`qi test`コマンド（将来実装予定）で実行されます。

---

## qi.toml仕様

`qi.toml`はプロジェクトの設定ファイルで、TOML形式で記述します。

### 基本的な例

```toml
[project]
name = "my-project"
version = "0.1.0"
authors = ["Alice <alice@example.com>"]
description = "My awesome Qi project"
license = "MIT"
qi-version = "0.1.0"

[dependencies]
# 将来の拡張用（現在は未実装）

[features]
default = ["http-server", "format-json"]
```

---

### `[project]` セクション

プロジェクトのメタデータを定義します。

#### `name` (必須)
プロジェクト名。英数字、ハイフン、アンダースコアが使用可能。

```toml
name = "my-project"
```

#### `version` (必須)
プロジェクトのバージョン。セマンティックバージョニング推奨。

```toml
version = "0.1.0"
```

#### `authors` (任意)
プロジェクト作者のリスト。

```toml
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
```

#### `description` (任意)
プロジェクトの説明。

```toml
description = "A web API server written in Qi"
```

#### `license` (任意)
ライセンス識別子（SPDX形式推奨）。

```toml
license = "MIT"
license = "Apache-2.0"
license = "GPL-3.0-or-later"
```

#### `qi-version` (必須)
互換性のあるQiのバージョン。

```toml
qi-version = "0.1.0"
```

---

### `[dependencies]` セクション

依存関係を定義します（将来の拡張用、現在は未実装）。

```toml
[dependencies]
# 将来的に他のQiパッケージへの依存を記述
# my-lib = "1.0.0"
```

---

### `[features]` セクション

プロジェクトで使用する機能フラグを定義します。

#### `default`
デフォルトで有効にする機能のリスト。

```toml
[features]
default = []  # 基本機能のみ

# または
default = ["http-server", "format-json"]  # HTTP + JSON機能を有効化
```

#### 利用可能な機能

以下は、Qiランタイムで利用可能な機能フラグの例です：

| Feature | 説明 |
|---------|------|
| `http-server` | HTTPサーバー機能（`server/serve`, `server/json`など） |
| `http-client` | HTTPクライアント機能（`http/get`, `http/post`など） |
| `format-json` | JSON処理機能（`json/parse`, `json/stringify`など） |
| `format-yaml` | YAML処理機能（`yaml/parse`, `yaml/stringify`など） |
| `io-file` | ファイルI/O機能（`io/read-file`, `io/write-file`など） |
| `io-glob` | ファイルグロブ機能（`io/glob`など） |
| `db-sqlite` | SQLiteデータベース機能（将来実装予定） |
| `concurrency` | 並行処理機能（`go`, `chan`など） |
| `std-math` | 数学関数（`math/rand`, `math/sqrt`など） |
| `string-encoding` | 文字列エンコーディング（base64, URLエンコードなど） |

**注意**: 機能フラグは現在の実装ではドキュメント目的のみで、実行時の動作には影響しません（将来のビルドシステムで使用予定）。

---

## テンプレートシステム

`qi new`コマンドは、テンプレートを使用してプロジェクトを生成します。

### 組み込みテンプレート

#### `basic` (デフォルト)
基本的なプロジェクト構造。

**構造:**
```
project/
├── qi.toml
├── main.qi
├── src/
│   └── lib.qi
├── examples/
│   └── example.qi
└── tests/
    └── test.qi
```

**用途**: シンプルなスクリプトやライブラリ開発

---

#### `http-server`
JSON APIサーバー。

**構造:**
```
project/
├── qi.toml          # features = ["http-server", "format-json"]
├── main.qi          # サーバー実装
└── src/
```

**用途**: RESTful APIサーバー、Webバックエンド

**特徴:**
- ルーティング実装（`/`, `/api/hello`, `/api/users`）
- JSONレスポンス
- リクエストハンドラーの例

---

### テンプレートの検索順序

テンプレートは以下の順序で検索されます：

1. `./.qi/templates/<name>/` - プロジェクトローカル
2. `~/.qi/templates/<name>/` - ユーザーグローバル
3. `<qi-binary-dir>/std/templates/<name>/` - インストール版
4. `std/templates/<name>/` - 開発版

最初に見つかったテンプレートが使用されます。

---

## カスタムテンプレートの作成

独自のテンプレートを作成できます。

### テンプレートの構造

```
~/.qi/templates/my-template/
├── template.toml          # テンプレートのメタデータ
├── qi.toml.template       # プロジェクト設定テンプレート
├── main.qi.template       # メインファイルテンプレート
├── src/
│   └── lib.qi.template
└── tests/
    └── test.qi.template
```

### `template.toml`

テンプレートのメタデータを定義します。

```toml
[template]
name = "my-template"
description = "My custom template"
author = "Your Name"
version = "1.0.0"

[features]
required = ["http-server", "format-json"]
```

#### `[template]` セクション

| フィールド | 必須 | 説明 |
|-----------|------|------|
| `name` | ✓ | テンプレート名 |
| `description` | ✓ | テンプレートの説明 |
| `author` | | 作成者 |
| `version` | | テンプレートのバージョン |

#### `[features]` セクション

| フィールド | 説明 |
|-----------|------|
| `required` | このテンプレートで必要な機能フラグのリスト |

---

### テンプレートファイルの記法

テンプレートファイルには、変数置換と条件分岐が使用できます。

#### 変数置換

`{{ variable }}` 形式で変数を埋め込みます。

**利用可能な変数:**
- `{{ project_name }}` - プロジェクト名
- `{{ version }}` - バージョン
- `{{ author }}` - 著者名
- `{{ description }}` - 説明
- `{{ license }}` - ライセンス

**例:**
```toml
# qi.toml.template
[project]
name = "{{ project_name }}"
version = "{{ version }}"
description = "{{ description }}"
```

**生成後:**
```toml
[project]
name = "my-project"
version = "0.1.0"
description = "My awesome project"
```

---

#### 条件分岐

`{{ #if variable }}...{{ /if }}` 形式で条件分岐を記述します。

**例:**
```toml
# qi.toml.template
[project]
name = "{{ project_name }}"
version = "{{ version }}"
{{ #if author }}authors = ["{{ author }}"]{{ /if }}
{{ #if description }}description = "{{ description }}"{{ /if }}
{{ #if license }}license = "{{ license }}"{{ /if }}
qi-version = "0.1.0"
```

変数が空の場合、その行全体が削除されます。

**生成後（author, descriptionが空の場合）:**
```toml
[project]
name = "my-project"
version = "0.1.0"
license = "MIT"
qi-version = "0.1.0"
```

---

### ファイル名規則

- `.template` サフィックスを持つファイルは、サフィックスが削除されて出力されます
- 例: `main.qi.template` → `main.qi`
- `template.toml` はコピーされません

---

### テンプレートの配置

#### ユーザーグローバルテンプレート
```bash
mkdir -p ~/.qi/templates/my-template
cp -r my-template/* ~/.qi/templates/my-template/
```

#### プロジェクトローカルテンプレート
```bash
mkdir -p .qi/templates/my-template
cp -r my-template/* .qi/templates/my-template/
```

---

### テンプレートの使用

```bash
# テンプレート一覧を確認
qi template list

# カスタムテンプレートを使用
qi new my-project --template my-template

# テンプレート情報を確認
qi template info my-template
```

---

## 例: カスタムCLIツールテンプレート

コマンドラインツール用のテンプレートを作成する例：

### `~/.qi/templates/cli/template.toml`
```toml
[template]
name = "cli"
description = "Command-line tool template"
author = "Qi Team"
version = "1.0.0"

[features]
required = ["io-file", "string-encoding"]
```

### `~/.qi/templates/cli/qi.toml.template`
```toml
[project]
name = "{{ project_name }}"
version = "{{ version }}"
{{ #if author }}authors = ["{{ author }}"]{{ /if }}
{{ #if description }}description = "{{ description }}"{{ /if }}
{{ #if license }}license = "{{ license }}"{{ /if }}
qi-version = "0.1.0"

[dependencies]

[features]
default = ["io-file", "string-encoding"]
```

### `~/.qi/templates/cli/main.qi.template`
```qi
;; {{ project_name }} - Command-line tool
;;
;; 使い方: qi main.qi [OPTIONS] [ARGS]

(println "=== {{ project_name }} v{{ version }} ===")

;; コマンドライン引数のパース
(defn parse-args [args]
  {:command (first args)
   :options (rest args)})

;; メインロジック
(defn main [args]
  (let [parsed (parse-args args)]
    (match (get parsed :command)
      "help" -> (println "ヘルプメッセージ")
      nil -> (println "コマンドを指定してください")
      cmd -> (println f"不明なコマンド: {cmd}"))))

;; エントリーポイント
(main (get (env) :args []))
```

### 使用例
```bash
# CLIツールを作成
qi new mytool --template cli
cd mytool
qi main.qi help
```

---

## 関連ドキュメント

- [CLI リファレンス](cli.md) - `qi`コマンドの使い方
- [チュートリアル](tutorial/01-getting-started.md) - 実践的な使い方
- [言語仕様](spec/README.md) - Qi言語の文法と機能
