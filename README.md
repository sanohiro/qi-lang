# Qi - A Lisp that flows

シンプル、高速、簡潔なモダンLisp系言語。パイプライン、パターンマッチング、並行・並列処理に強い。

## 特徴

- **シンプル**: 特殊形式8つのみ（`def` `fn` `let` `do` `if` `match` `try` `defer`）
- **パイプライン**: `|>` `|>?` `||>` でデータフローを直感的に記述
  - **Railway Pipeline (`|>?`)**: エラーハンドリングを流れに組み込む ⭐ NEW
- **パターンマッチング**: 強力な `match` 式
- **Web開発**: JSON/HTTP完全対応、Railway Pipelineでエラーハンドリング ⭐ NEW
- **並行・並列**: 3層アーキテクチャ（go/chan、パイプライン、async/await）で簡単に並列化
- **デバッグ**: `inspect`、`time`でパイプライン内のデータを観察 ⭐ NEW
- **多言語対応**: 英語・日本語のエラーメッセージ（環境変数 `QI_LANG` で設定）
- **安全なマクロ**: `uvar` による変数衝突回避

## 多言語対応

Qiは英語と日本語のエラーメッセージに対応しています。言語は環境変数で自動検出されます。

```bash
# システムのLANG環境変数を使用（macOS/Linuxのデフォルト）
# LANG=ja_JP.UTF-8 の場合、自動的に日本語になります
qi script.qi

# Qi専用の言語設定で上書き
QI_LANG=ja qi script.qi  # 日本語
QI_LANG=en qi script.qi  # 英語
```

**優先順位**: `QI_LANG` > `LANG` > デフォルト(en)

## Hello World

```lisp
(def greet (fn [name]
  f"Hello, {name}!"))

(greet "World")
;; "Hello, World!"
```

## パイプライン例

### 基本パイプライン
```lisp
(data
 |> parse-json
 |> (filter active?)
 |> (map :email)
 |> (join ", ")
 |> log)
```

### Railway Pipeline - エラーハンドリング ⭐ NEW
```lisp
;; Web APIからデータ取得 → 変換 → 保存
("https://api.example.com/users/123"
 |> http/get              ;; {:ok {...}} または {:error ...}
 |>? (fn [resp] {:ok (get resp "body")})
 |>? json/parse
 |>? validate-user
 |>? save-to-db)
;; エラーが起きたら自動的にショートサーキット！
```

## 使い方

```bash
# REPL起動
qi

# スクリプトファイル実行
qi script.qi

# ワンライナー実行
qi -e '(+ 1 2 3)'

# ファイルをロードしてREPL起動
qi -l utils.qi

# ヘルプ表示
qi --help
```

## ドキュメント

- [完全な言語仕様](SPEC.md) - 詳細な文法、組み込み関数、モジュールシステム
- [実用例](examples/web-api/) - Web API、JSON処理、Railway Pipelineの実例 ⭐ NEW

## 実装例

### Web API クライアント ⭐ NEW

```lisp
;; GitHub APIからユーザー情報を取得
(def fetch-user (fn [username]
  (str "https://api.github.com/users/" username)
  |> http/get
  |>? (fn [resp] {:ok (get resp "body")})
  |>? json/parse
  |>? (fn [user] {:ok (get user "name")})))

(fetch-user "octocat")  ;; => {:ok "The Octocat"}
```

### JSON データ変換 ⭐ NEW

```lisp
;; JSONパース → フィルタ → 変換 → JSON生成
(json-string
 |> json/parse
 |>? (fn [data] {:ok (get data "users")})
 |>? (fn [users] {:ok (filter (fn [u] (= (get u "city") "Tokyo")) users)})
 |>? (fn [users] {:ok (map (fn [u] (update u "age" inc)) users)})
 |>? json/pretty)
```

### コレクション操作 ⭐ NEW

```lisp
;; データ検索と変換
(def users [{:name "Alice" :age 30} {:name "Bob" :age 25}])

;; 条件に合う最初のユーザー
(find (fn [u] (>= (get u :age) 25)) users)

;; 全員が成人か？
(every? (fn [u] (>= (get u :age) 20)) users)  ;; => true

;; マップのキーを全て大文字に
(update-keys upper {:name "Alice" :city "Tokyo"})
;; => {"NAME" "Alice" "CITY" "Tokyo"}
```

### デバッグ・計測 ⭐ NEW

```lisp
;; データフローを観察
(data
 |> transform
 |> inspect              ;; 整形表示してそのまま返す
 |> validate)

;; パフォーマンス計測
(time (fn [] (reduce + (range 1000000))))
;; Elapsed: 0.234s
```

### 並行処理

```lisp
;; 複数URLを並列取得
(def urls ["https://api.example.com/1" "https://api.example.com/2"])

(urls
 ||> http/get            ;; 並列リクエスト
 |> (map extract-data)
 |> merge-results)
```

## 開発状況

現在、言語仕様を策定中です。実装は未着手です。

## ライセンス

未定
