# チュートリアル: Qiをはじめよう

このチュートリアルでは、Qiプロジェクトの作成から実行まで、実際に手を動かしながら学んでいきます。

> **📚 Lisp系言語が初めての方へ**  
> Qiは**Lisp系言語**です。括弧の書き方に慣れていない場合は、まず以下をご覧ください：  
> ➡️ **[Lisp系言語の基礎](00-lisp-basics.md)** - 括弧の読み方・基本的な考え方（5分で読めます）

## 目次

- [環境の確認](#環境の確認)
- [はじめてのプロジェクト](#はじめてのプロジェクト)
- [コードを書いてみる](#コードを書いてみる)
- [HTTPサーバーを作る](#httpサーバーを作る)
- [REPLで試す](#replで試す)
- [次のステップ](#次のステップ)
---

## 環境の確認

まず、Qiが正しくインストールされているか確認しましょう。

```bash
qi --version
```

バージョン情報が表示されればOKです。

```
Qi Programming Language v0.1.0
```

ヘルプも見てみましょう：

```bash
qi --help
```

---

## はじめてのプロジェクト

### プロジェクトの作成

\`qi new\`コマンドで新しいプロジェクトを作成します。

```bash
qi new hello-qi
```

いくつかの質問が表示されます。Enterキーを押すとデフォルト値が使用されます。

```
新しいQiプロジェクトを作成します

プロジェクト名 [hello-qi]:
バージョン [0.1.0]:
説明 (省略可): My first Qi project
著者名 (省略可): Your Name
ライセンス [MIT]:

新しいQiプロジェクトが作成されました: hello-qi

次のステップ:
  cd hello-qi
  qi main.qi
```

### プロジェクト構造

作成されたプロジェクトを確認しましょう：

```bash
cd hello-qi
ls -la
```

以下のファイル・ディレクトリが作成されています：

```
hello-qi/
├── qi.toml         # プロジェクト設定
├── main.qi         # メインファイル
├── src/            # ライブラリコード
│   └── lib.qi
├── examples/       # サンプルコード
│   └── example.qi
└── tests/          # テストコード
    └── test.qi
```

### qi.tomlを見てみる

プロジェクト設定ファイルを開いてみましょう：

```bash
cat qi.toml
```

```toml
[project]
name = "hello-qi"
version = "0.1.0"
description = "My first Qi project"
authors = ["Your Name"]
license = "MIT"
qi-version = "0.1.0"

[dependencies]

[features]
default = []
```

これはプロジェクトのメタデータです。必要に応じて編集できます。

### 実行してみる

さっそく実行してみましょう：

```bash
qi main.qi
```

出力：
```
Hello, Qi!
こんにちは、hello-qiさん！
Vector([Integer(16), Integer(25)])
```

成功です！テンプレートが生成したサンプルコードが実行されました。

---

## コードを書いてみる

### main.qiを編集

\`main.qi\`を開いて、内容を確認しましょう：

```qi
;; main.qi - エントリーポイント

(println "Hello, Qi!")

;; 挨拶関数
(defn greet [name]
  (str "こんにちは、" name "さん！"))

;; 関数を使用
(println (greet "hello-qi"))

;; パイプライン例
(println ([1 2 3 4 5]
          |> (map (fn [x] (* x x)))
          |> (filter (fn [x] (> x 10)))))
```

### 自分のコードを追加

ファイルの最後に以下を追加してみましょう：

```qi
;; 自分で書いたコード
(println "\n=== 自分で書いたコード ===")

;; 偶数をフィルタして合計
(def numbers [1 2 3 4 5 6 7 8 9 10])
(def sum-of-evens
  (numbers
   |> (filter (fn [x] (= (% x 2) 0)))
   |> (reduce + 0)))

(println f"偶数の合計: {sum-of-evens}")

;; マップのデータ処理
(def users [
  {:name "Alice" :age 30}
  {:name "Bob" :age 25}
  {:name "Charlie" :age 35}
])

(def adult-names
  (users
   |> (filter (fn [u] (>= (get u :age) 30)))
   |> (map (fn [u] (get u :name)))))

(println f"30歳以上: {adult-names}")
```

実行してみます：

```bash
qi main.qi
```

出力：
```
...
=== 自分で書いたコード ===
偶数の合計: 30
30歳以上: Vector([String("Alice"), String("Charlie")])
```

### Qiの特徴を体験

**パイプライン演算子 \`|>\`**:
```qi
[1 2 3 4 5]
|> (map (fn [x] (* x 2)))
|> (filter (fn [x] (> x 5)))
|> (reduce + 0)
```

データを左から右に流すように書けます！

---

## HTTPサーバーを作る

次は、HTTPサーバーを作ってみましょう。

### プロジェクトの作成

\`http-server\`テンプレートを使います：

```bash
cd ..
qi new myapi --template http-server
cd myapi
```

### 構造を確認

```bash
cat qi.toml
```

\`[features]\`に注目：
```toml
[features]
default = ["http-server", "format-json"]
```

HTTP機能とJSON機能が有効になっています。

### サーバーコードを見る

```bash
cat main.qi
```

```qi
;; myapi - HTTP Server
;;
;; JSON APIサーバー
;; 実行: qi main.qi

(println "=== myapi HTTP Server ===")

;; ルートハンドラー
(defn handle-root [req]
  (server/json {:message "Welcome to myapi!"
                :version "0.1.0"
                :endpoints ["/api/hello" "/api/users"]}))

;; Hello APIハンドラー
(defn handle-hello [req]
  (let [name (get (get req :params) "name" "World")]
    (server/json {:message (str "Hello, " name "!")})))

;; Users APIハンドラー
(defn handle-users [req]
  (server/json {:users [{:id 1 :name "Alice"}
                        {:id 2 :name "Bob"}
                        {:id 3 :name "Charlie"}]}))

;; メインハンドラー（ルーティング）
(defn handler [req]
  (let [path (get req :path)]
    (match path
      "/" -> (handle-root req)
      "/api/hello" -> (handle-hello req)
      "/api/users" -> (handle-users req)
      _ -> (server/json {:error "Not Found"} 404))))

;; サーバー起動
(def port 3000)
(println f"Starting server on http://localhost:{port}")
(println "Press Ctrl+C to stop")

(server/serve port handler)
```

### サーバーを起動

```bash
qi main.qi
```

出力：
```
=== myapi HTTP Server ===
Starting server on http://localhost:3000
Press Ctrl+C to stop
Listening on http://0.0.0.0:3000
```

### APIをテスト

別のターミナルで：

```bash
# ルート
curl http://localhost:3000/

# Hello API
curl http://localhost:3000/api/hello
curl http://localhost:3000/api/hello?name=Alice

# Users API
curl http://localhost:3000/api/users
```

### カスタマイズしてみる

\`main.qi\`に新しいエンドポイントを追加してみましょう：

```qi
;; Status APIハンドラー（追加）
(defn handle-status [req]
  (server/json {:status "ok"
                :uptime 123
                :version "0.1.0"}))

;; handlerに追加
(defn handler [req]
  (let [path (get req :path)]
    (match path
      "/" -> (handle-root req)
      "/api/hello" -> (handle-hello req)
      "/api/users" -> (handle-users req)
      "/api/status" -> (handle-status req)  ; 追加
      _ -> (server/json {:error "Not Found"} 404))))
```

サーバーを再起動（Ctrl+Cで停止して再実行）して、テスト：

```bash
curl http://localhost:3000/api/status
```

---

## REPLで試す

### REPLを起動

```bash
qi
```

```
Qi Programming Language v0.1.0
Press Ctrl+C to interrupt, Ctrl+D to exit
Type :help for available commands

qi:1>
```

### 基本的な計算

```qi
qi:1> (+ 1 2 3)
6

qi:2> (* 2 (+ 3 4))
14

qi:3> (/ 10 3)
3
```

### 変数と関数を定義

```qi
qi:4> (def x 10)
10

qi:5> (defn square [n] (* n n))
Function(square)

qi:6> (square x)
100
```

### パイプラインを試す

```qi
qi:7> ([1 2 3 4 5] |> (map (fn [x] (* x 2))))
Vector([Integer(2), Integer(4), Integer(6), Integer(8), Integer(10)])

qi:8> ([1 2 3 4 5]
        |> (map (fn [x] (* x 2)))
        |> (filter (fn [x] (> x 5)))
        |> (reduce + 0))
30
```

### REPLコマンド

```qi
qi:9> :help
利用可能なコマンド:
  :help                    ヘルプを表示
  :doc <name>              Show documentation for a function
  :vars                    定義されている変数を表示
  ...

qi:10> :vars
定義されている変数:
  x

qi:11> :funcs
定義されている関数:
  square

qi:12> :doc map
(ドキュメントが表示されます)
```

### ファイルをロード

```qi
qi:13> :load src/lib.qi
Loading src/lib.qi...
Loaded

qi:14> (greet "REPL")
"こんにちは、REPLさん！"
```

---

## 次のステップ

おめでとうございます！これでQiの基本的な使い方をマスターしました。

### さらに学ぶ

- **[言語仕様](../spec/README.md)** - Qi言語の全機能を学ぶ
  - [パイプライン演算子](../spec/02-flow-pipes.md) - \`|>\`, \`|>?\`, \`||>\`, \`~>\`
  - [並行・並列処理](../spec/03-concurrency.md) - \`go\`, \`chan\`
  - [パターンマッチング](../spec/04-match.md) - \`match\`式
  - [エラー処理](../spec/08-error-handling.md) - \`try\`, \`defer\`

- **[標準ライブラリ](../spec/10-stdlib-string.md)** - 60以上の組み込み関数
  - 文字列操作
  - HTTP クライアント/サーバー
  - JSON/YAML処理
  - ファイルI/O

- **[CLIリファレンス](../cli.md)** - \`qi\`コマンドの詳細

- **[プロジェクト管理](../project.md)** - qi.toml、テンプレート、カスタマイズ

### プロジェクトのアイデア

- **CLIツール**: ファイル処理、データ変換
- **WebAPI**: RESTful API、マイクロサービス
- **データパイプライン**: ログ解析、ETL処理
- **自動化スクリプト**: タスク自動化、バッチ処理

### コミュニティ

- [GitHubリポジトリ](https://github.com/yourusername/qi-lang) - Issue報告、PR歓迎！
- ドキュメントの改善提案も大歓迎です

---

Happy coding with Qi! 🚀
