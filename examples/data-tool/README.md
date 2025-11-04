# Qi Data Tool - データ変換CLIツール

Qiのパイプライン処理と外部コマンド実行を活用したデータ変換CLIツールです。
jq/yqライクなJSON/YAML/TOML変換、SQL整形などを提供します。

## 特徴

- **パイプライン処理**: Qiの`|>`演算子を使った直感的なデータ変換
- **外部コマンド連携**: jq、yq、sqlformatなどの既存ツールと統合
- **STDIN/STDOUT対応**: Unixパイプライン完全対応
- **フォールバック**: 外部コマンドがない場合はQi標準ライブラリで処理

## 必要な外部コマンド

### 必須
- **jq** - JSON処理・整形
  ```bash
  brew install jq
  ```

### オプション（機能拡張）
- **yq** - YAML処理
  ```bash
  brew install yq
  ```

- **sqlformat** - SQL整形（Python sqlparseパッケージ）
  ```bash
  pip3 install sqlparse
  ```

## 使い方

### 基本構文

```bash
qi main.qi <command> [options] [input-file]
```

### コマンド一覧

#### JSON処理
- `json:pretty [file]` - JSON整形（読みやすく）
- `json:minify [file]` - JSON圧縮（1行に）
- `json:filter <jq-expr> [file]` - jqでフィルタリング

#### YAML ⇔ JSON変換
- `yaml:to-json [file]` - YAML → JSON変換
- `json:to-yaml [file]` - JSON → YAML変換

#### SQL整形
- `sql:format [file]` - SQL整形（キーワード大文字化）

#### その他
- `help` - ヘルプ表示

## 使用例

### JSON整形

```bash
# ファイルから読み込み
cd examples/data-tool
qi main.qi json:pretty test/sample.json

# パイプライン経由
cat test/sample.json | qi main.qi json:pretty

# 出力例:
{
  "users": [
    {
      "id": 1,
      "name": "Alice",
      "email": "alice@example.com",
      "active": true
    },
    ...
  ]
}
```

### JSON圧縮

```bash
qi main.qi json:minify test/sample.json

# 出力: 1行にまとめられたJSON
{"users":[{"id":1,"name":"Alice",...}]}
```

### jqフィルタリング

```bash
# アクティブなユーザーのみ抽出
qi main.qi json:filter '.users[] | select(.active == true)' test/sample.json

# 名前だけ抽出
qi main.qi json:filter '.users[].name' test/sample.json

# 出力:
"Alice"
"Charlie"
```

### YAML → JSON変換

```bash
qi main.qi yaml:to-json test/sample.yaml

# 出力: JSON形式に変換されたデータ
{
  "users": [
    {"id": 1, "name": "Alice", ...}
  ],
  ...
}
```

### JSON → YAML変換

```bash
qi main.qi json:to-yaml test/sample.json

# 出力: YAML形式に変換されたデータ
users:
  - id: 1
    name: Alice
    ...
```

### SQL整形

```bash
qi main.qi sql:format test/sample.sql

# 入力（1行の読みにくいSQL）:
select u.id,u.name from users u where u.active=true;

# 出力（整形されたSQL）:
SELECT u.id,
       u.name
FROM users u
WHERE u.active = TRUE;
```

### パイプライン連携

```bash
# curlでAPIからデータ取得 → JSON整形
curl -s https://api.example.com/users | qi main.qi json:pretty

# YAML → JSON → jqフィルタリング
cat config.yaml | qi main.qi yaml:to-json | jq '.database.host'

# JSON → TOML → ファイル保存
qi main.qi json:to-toml data.json > config.toml
```

## 実装のポイント

### パイプライン処理

Qiの`|>`演算子を使った関数型パイプライン：

```qi
;; JSON整形（jqがない場合）
(defn json-pretty [input]
  (input
   |> json/parse           ;; 文字列 → データ構造
   |> (fn [data] (json/stringify data 2))))  ;; データ構造 → 整形JSON
```

### 外部コマンド実行

`cmd/pipe`を使った外部コマンド連携（標準出力を文字列で返す）：

```qi
;; jqでフィルタリング
(defn json-filter [expr input]
  (try
    (input |> (cmd/pipe (str "jq '" expr "'")))
    (error "jq command failed")))
```

### フォールバック処理

外部コマンドがない場合はQi標準ライブラリで処理：

```qi
(defn json-pretty [input]
  (if (command-exists? "jq")
    (input |> (cmd/pipe "jq ."))    ;; jq使用（標準出力を取得）
    (input |> json/parse |> json/stringify)))  ;; Qi標準ライブラリ
```

## テストデータ

`test/`ディレクトリにサンプルデータが含まれています：

- `sample.json` - JSONサンプル
- `sample.yaml` - YAMLサンプル
- `sample.sql` - SQLサンプル

## トラブルシューティング

### エラー: jqコマンドが見つかりません

```bash
brew install jq
```

### エラー: yqコマンドが見つかりません

```bash
brew install yq
```

### エラー: sqlformatコマンドが見つかりません

```bash
pip3 install sqlparse
```

### 外部コマンドなしで使いたい

以下の機能は外部コマンド不要：
- `json:pretty`（jqなしでも動作）
- `json:minify`（jqなしでも動作）

以下の機能は外部コマンドが必須：
- `json:filter`（jq必須）
- `yaml:to-json`（yq必須）
- `json:to-yaml`（yq必須）
- `sql:format`（sqlformat必須）

## ライセンス

このサンプルコードはQi言語のexamplesとして提供されています。
