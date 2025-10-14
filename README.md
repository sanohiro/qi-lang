# Qi - A Lisp that flows

<p align="center">
  <img src="./assets/logo/qi-logo-full-512.png" alt="Qi Logo" width="400">
</p>

シンプル、高速、簡潔なモダンLisp系言語。パイプライン、パターンマッチング、並行・並列処理に強い。

## 特徴

- **シンプル**: 特殊形式8つのみ（`def` `fn` `let` `do` `if` `match` `try` `defer`）
- **パイプライン**: `|>` `|>?` `||>` `~>` でデータフローを直感的に記述
  - **Railway Pipeline (`|>?`)**: エラーハンドリングを流れに組み込む
  - **非同期パイプライン (`~>`)**: goroutine風の自動並行実行 ⭐ NEW
- **パターンマッチング**: 強力な `match` 式
  - **orパターン**: `1 | 2 | 3 -> "small"` で複数パターンを簡潔に記述 ⭐ NEW
- **文字列機能**: 複数行文字列 (`"""..."""`)、f-string、充実したフォーマット関数 ⭐ NEW
- **統計分析**: `stats` モジュールで平均・中央値・標準偏差など ⭐ NEW
- **Web開発**: JSON/HTTP完全対応、Railway Pipelineでエラーハンドリング
- **並行・並列**: 3層アーキテクチャ（go/chan、パイプライン、async/await）で簡単に並列化
- **デバッグ**: `inspect`、`time`でパイプライン内のデータを観察
- **多言語対応**: 英語・日本語のエラーメッセージ（環境変数 `QI_LANG` で設定）
- **安全性**: 名前衝突警告、`uvar` による変数衝突回避 ⭐ NEW

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

### Railway Pipeline - エラーハンドリング
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

### 非同期パイプライン - goroutine風の並行実行 ⭐ NEW
```lisp
;; 即座にチャネルを返し、バックグラウンドで実行
(def result (data ~> transform ~> process))
(recv! result)  ;; 結果を受信

;; 複数の非同期処理を並行実行
(def r1 (10 ~> inc ~> double))
(def r2 (20 ~> double ~> inc))
(println (recv! r1) (recv! r2))  ;; 両方並行実行
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

## モジュール構造

Qiは**2層モジュール設計**を採用しています：

### Core（90個）- グローバル名前空間
最もよく使う関数は、すぐに使えます：
```lisp
(map inc [1 2 3])           ; Core関数はそのまま
(filter even? [1 2 3 4])
(reduce + [1 2 3 4])
(tap println)               ; パイプライン内で副作用実行
```

### 専門モジュール（160個）- `module/function` 形式
専門的な機能は、名前空間を明示して使います：
```lisp
(io/read-file "data.txt")        ; ファイルI/O
(math/pow 2 8)                   ; 数学関数
(str/upper "hello")              ; 文字列操作
(json/parse "{\"a\": 1}")        ; JSON処理
(http/get "https://api.example.com")  ; HTTP通信
```

**主な専門モジュール**:
- **list**: 高度なリスト操作 - `list/frequencies`, `list/partition-by`
- **map**: 高度なマップ操作 - `map/select-keys`, `map/update-keys`
- **str**: 文字列操作（62個）- `str/upper`, `str/snake`, `str/to-base64`
- **math**: 数学関数 - `math/pow`, `math/sqrt`, `math/round`, `math/random-range`
- **stats**: 統計関数（6個）- `stats/mean`, `stats/median`, `stats/stddev`, `stats/percentile` ⭐ NEW
- **io**: ファイルI/O - `io/read-file`, `io/write-file`
- **json**: JSON処理 - `json/parse`, `json/stringify`
- **http**: HTTP通信 - `http/get`, `http/post`
- **stream**: ストリーム処理 - `stream/map`, `stream/filter`
- **async**: 並行処理（高度）- `async/await`, `async/all`

詳細は [SPEC.md](SPEC.md) を参照してください。

## ドキュメント

- [完全な言語仕様](SPEC.md) - 詳細な文法、組み込み関数、モジュールシステム
- [実装チュートリアル](TUTORIAL.md) - Rust、言語実装、Qi言語を同時に学ぶ
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

;; マップのキーを全て大文字に（専門モジュール）
(map/update-keys str/upper {:name "Alice" :city "Tokyo"})
;; => {"NAME" "Alice" "CITY" "Tokyo"}
```

### デバッグ・計測 ⭐ NEW

```lisp
;; データフローを観察（tap）
([1 2 3]
 |> (map inc)
 |> (tap println)        ;; (2 3 4)を出力してそのまま返す
 |> sum)                 ;; => 9

;; 整形表示
(data
 |> transform
 |> inspect          ;; 整形表示してそのまま返す
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

### 文字列フォーマット ⭐ NEW

```lisp
;; 複数行文字列（SQL、HTML、マークダウンなどに便利）
(def query """
  SELECT name, price
  FROM products
  WHERE price >= 1000
  ORDER BY price DESC
""")

;; 複数行f-string（テンプレートエンジン風）
(def gen-report (fn [data]
  f"""
  === Sales Report ===
  Date: {(:date data)}
  Total: ¥{(str/format-comma (:total data))}
  Growth: {(str/format-percent 1 (:growth data))}
  """))

(gen-report {:date "2024-01-15" :total 1234567 :growth 0.156})
;; =>
;; === Sales Report ===
;; Date: 2024-01-15
;; Total: ¥1,234,567
;; Growth: 15.6%

;; パイプラインで数値をフォーマット
(3.14159 |> (str/format-decimal 2))    ;; "3.14"
(1234567 |> (str/format-comma))        ;; "1,234,567"
(0.856 |> (str/format-percent 1))      ;; "85.6%"

;; 実用例: レポート生成パイプライン
(def format-price (fn [price]
  (price
   |> (str/format-comma)
   |> (fn [s] f"¥{s}"))))

(12345 |> format-price)  ;; "¥12,345"
```

## 開発状況

現在、言語仕様を策定中です。実装は未着手です。

## ライセンス

未定
