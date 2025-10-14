# Qi Style Guide

Qiコードのフォーマット規則とベストプラクティスをまとめたスタイルガイドです。

`qi fmt`コマンドは、このガイドに基づいてコードを自動整形します。

---

## 目次

1. [フォーマットルール](#1-フォーマットルール)
2. [命名規則](#2-命名規則)
3. [ベストプラクティス](#3-ベストプラクティス)
4. [アンチパターン](#4-アンチパターン)
5. [フォーマッター設定](#5-フォーマッター設定)

---

## 1. フォーマットルール

### 1.1 インデント

**2スペース**を使用します。タブは使用しません。

```qi
;; ✅ 良い例
(defn factorial [n]
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

;; ❌ 悪い例（4スペース）
(defn factorial [n]
    (if (<= n 1)
        1
        (* n (factorial (- n 1)))))
```

### 1.2 行の長さ

**最大100文字**を推奨します。

```qi
;; ✅ 100文字以内
(some-function arg1 arg2 arg3 arg4 arg5)

;; ✅ 超える場合は改行
(some-very-long-function-name-that-would-exceed-the-limit
  arg1
  arg2
  arg3)
```

### 1.3 パイプライン

パイプライン演算子は**先頭に配置**し、各ステップを改行します。

```qi
;; ✅ 良い例
(data
  |> validate
  |> transform
  |> (filter active?)
  |> (map format)
  |> save)

;; ✅ 短い場合は1行でもOK
("hello" |> str/upper |> str/reverse)

;; ❌ 悪い例（演算子が末尾）
(data |>
  validate |>
  transform |>
  save)
```

#### 並列パイプライン (`||>`)

```qi
(data
  ||> process1
  ||> process2
  ||> process3
  |> collect-results)
```

#### Railway Pipeline (`|>?`)

```qi
(request
  |>? validate-input
  |>? check-permissions
  |>? execute-action
  |>? format-response)
```

### 1.4 特殊形式

#### `def` / `defn`

```qi
;; ✅ def
(def pi 3.14159)

(def config
  {:host "localhost"
   :port 8080})

;; ✅ defn
(defn greet [name]
  (println f"Hello, {name}!"))

(defn complex-function [arg1 arg2 arg3]
  (let [result (process arg1 arg2)]
    (if (valid? result)
      (transform result arg3)
      nil)))
```

#### `let`

束縛とボディを明確に分離します。

```qi
;; ✅ 良い例
(let [x 10
      y 20
      sum (+ x y)]
  (* sum 2))

;; ✅ 複雑な束縛
(let [user (get-user id)
      profile (get-profile user)
      settings (get-settings user)]
  (render-page user profile settings))
```

#### `if`

条件、then節、else節を揃えます。

```qi
;; ✅ 単純な場合
(if (> x 10)
  "big"
  "small")

;; ✅ 複雑な場合
(if (and (valid? user)
         (active? user)
         (authorized? user))
  (process-request user)
  (reject-request user))
```

#### `do`

各式を同じインデントで揃えます。

```qi
;; ✅ 良い例
(do
  (log/info "Starting process")
  (initialize-resources)
  (execute-main-task)
  (cleanup-resources)
  (log/info "Process completed"))
```

#### `match`

パターンとアクションの間に**両側スペース**を入れた`->`を配置します。

```qi
;; ✅ 良い例
(match value
  nil -> "empty"
  0 -> "zero"
  n when (> n 0) -> "positive"
  _ -> "negative")

;; ✅ 複雑なパターン
(match [x y]
  [0 0] -> "origin"
  [0 _] -> "y-axis"
  [_ 0] -> "x-axis"
  [x y] -> f"point ({x}, {y})")

;; ✅ orパターン
(match status
  :pending | :processing -> "in-progress"
  :completed | :success -> "done"
  :failed | :error -> "error")

;; ✅ 長いアクション
(match status
  :pending -> (do
                (log/info "Processing request")
                (process-request req))
  :completed -> (notify-user user)
  :failed -> (retry-with-backoff req))
```

#### `try` / `defer`

```qi
;; ✅ try
(try
  (risky-operation)
  (another-operation)
  (catch e
    (log/error f"Failed: {e}")
    nil))

;; ✅ defer
(do
  (defer (cleanup-resources))
  (do-main-work))
```

### 1.5 関数呼び出し

#### 短い引数

```qi
(+ 1 2 3)
(str/upper "hello")
(map inc [1 2 3])
```

#### 長い引数

```qi
;; ✅ 各引数を改行
(create-user
  "alice"
  "alice@example.com"
  30
  "123 Main St")

;; ✅ 関数定義も同様
(defn process-user [username
                    email
                    age
                    address]
  ...)
```

### 1.6 データ構造

#### ベクタ `[]`

```qi
;; ✅ 短い場合
[1 2 3 4 5]

;; ✅ 長い場合
[first-element
 second-element
 third-element
 fourth-element]

;; ✅ 入れ子
[[1 2 3]
 [4 5 6]
 [7 8 9]]
```

#### マップ `{}`

キーと値を縦に揃えます。

```qi
;; ✅ 基本形
{:name "Alice"
 :age 30
 :email "alice@example.com"}

;; ✅ 短い場合は1行でもOK
{:x 10 :y 20}

;; ✅ 入れ子
{:user {:name "Alice"
        :age 30
        :email "alice@example.com"}
 :status :active
 :created-at "2025-01-13"}
```

#### リスト `()`

```qi
'(1 2 3 4 5)

'(first-item
  second-item
  third-item)
```

### 1.7 文字列

#### 通常の文字列

```qi
"Hello, World!"

;; ✅ 長い文字列は複数行文字列を使用
"""
This is a very long string that spans multiple lines.
You can write it naturally without worrying about line breaks.
"""
```

#### F-string

補間部分はそのまま保持します。

```qi
;; ✅ 1行
(println f"Hello, {name}! You are {age} years old.")

;; ✅ 複数行
(println f"""
  Dear {name},

  Your account balance is {balance}.
  Thank you for using our service.
  """)
```

> `qi fmt` は文字列リテラルの改行・クォート・エスケープ表現を再構成せず、入力時の形を尊重します。

### 1.8 コメント

#### 行コメント

```qi
;; ✅ トップレベル: セミコロン2つ
;; This is a top-level comment
;; explaining the following code.

(def x 10)

;; ✅ インライン: セミコロン1つ（前に2スペース）
(def x 10)  ; This is an inline comment
```

> コメントは削除・結合されません。必要な説明は安心して書き残してください。

#### セクション区切り

```qi
;; ========================================
;; Data Processing Functions
;; ========================================

(defn process-data [data]
  ...)

(defn transform-data [data]
  ...)


;; ========================================
;; Helper Functions
;; ========================================

(defn helper-1 []
  ...)
```

### 1.9 空行

#### トップレベル定義の間: **推奨1行（0〜2行許容）**

通常は 1 行の空行で区切りますが、読みやすさのために 0〜2 行の範囲で調整して構いません。

```qi
(def x 10)

(def y 20)

(def config
  {:host "localhost"
   :port 8080})
```

#### `def` / `defn` / `defn-` の前: **必ず1行空ける（コメントは除外）**

トップレベルの定義フォーム（`def`、`defn`、`defn-`）の直前には最低 1 行の空行を入れてブロックを明確にします。説明用コメントを直前に置く場合は、そのコメントに続けて定義を書いて構いません。

```qi
(def cache (atom {}))

(defn clear-cache []
  ...)

;; kick entry point
(defn main []
  ...)

;; internal helper
;; コメントの直後に defn- を書いても良い
(defn- build-index [entries]
  ...)
```

#### セクション区切り: **2行**

```qi
(defn helper-1 [] ...)
(defn helper-2 [] ...)


;; ========================================
;; Public API
;; ========================================

(defn public-api-1 [] ...)
(defn public-api-2 [] ...)
```

### 1.10 モジュールシステム

#### `use`宣言

```qi
;; ✅ 短い場合
(use str :as s)
(use list :only [map filter reduce])

;; ✅ 長い場合
(use http
  :only [get post put delete
         request with-headers
         json xml])

;; ✅ 複数のuse（ファイル冒頭にまとめる）
(use str :as s)
(use list :as l)
(use io :only [read-file write-file])
(use http :only [get post])
```

#### `export`宣言

```qi
;; ✅ 短い場合
(export [func1 func2 func3])

;; ✅ 長い場合
(export
  [public-api-1
   public-api-2
   public-api-3
   helper-function
   utility-function])
```

### 1.11 フォーマッターポリシー

Qi のフォーマッタは「コードの意味を変えず、作者が意図したテキスト表現を尊重する」ことを目的とします。

- コメントや空白は削除・結合せず、設定範囲内で最小限の調整にとどめる
- 文字列リテラルの再エスケープや正規化は行わない
- トップレベル定義の区切りは `.qi-format.edn` の `blank-lines-between-defs` を基準に正規化する
- `def`/`defn`/`defn-` の直前には最低 1 行の空行を確保し、コメント直後の定義を許容する
- ガイド未定義のケースは既存のレイアウトを可能な限り保持し、将来的にルールを追加する

---

## 2. 命名規則

### 2.1 関数名

**ケバブケース**を使用します。

```qi
;; ✅ 良い例
(defn get-user [id] ...)
(defn process-payment [amount] ...)
(defn calculate-total-price [items] ...)

;; ❌ 悪い例
(defn getUser [id] ...)          ; camelCase
(defn process_payment [amount] ...) ; snake_case
```

### 2.2 述語関数

末尾に`?`を付けます。

```qi
(defn active? [user] ...)
(defn valid-email? [email] ...)
(defn empty? [coll] ...)
```

### 2.3 破壊的操作

末尾に`!`を付けます。

```qi
(defn send! [chan value] ...)
(defn reset! [atom value] ...)
(defn swap! [atom f] ...)
```

### 2.4 変数名

**ケバブケース**を使用します。

```qi
(def max-connections 100)
(def api-base-url "https://api.example.com")
(let [user-id 123
      user-name "alice"]
  ...)
```

### 2.5 定数

通常の変数と同じ**ケバブケース**を使用します。

```qi
(def pi 3.14159)
(def max-retry-count 3)
(def default-timeout 30000)
```

### 2.6 キーワード

**ケバブケース**を使用します。

```qi
{:user-id 123
 :user-name "alice"
 :created-at "2025-01-13"}
```

### 2.7 プライベート関数

`defn-`を使用します。

```qi
(defn- internal-helper [x]
  "モジュール内でのみ使用"
  (* x 2))

(defn public-api [x]
  "外部に公開"
  (internal-helper x))
```

---

## 3. ベストプラクティス

### 3.1 パイプラインを活用する

```qi
;; ✅ パイプライン
(data
  |> validate
  |> transform
  |> save)

;; ❌ ネストした関数呼び出し
(save (transform (validate data)))
```

### 3.2 matchでパターンマッチを使う

```qi
;; ✅ match
(match value
  nil -> "empty"
  [x] -> f"single: {x}"
  [x ...rest] -> f"multiple")

;; ❌ 複雑なif-else
(if (nil? value)
  "empty"
  (if (= (count value) 1)
    f"single: {(first value)}"
    "multiple"))
```

### 3.3 分解を活用する

```qi
;; ✅ let分解
(let [{:name n :age a} user]
  (println f"{n} is {a} years old"))

;; ✅ 関数引数分解
(defn greet [{:name n}]
  (println f"Hello, {n}!"))

;; ✅ match分解
(match coords
  [x y] -> (+ x y))
```

### 3.4 useで名前空間を整理

```qi
;; ✅ エイリアスを使用
(use str :as s)
(s/upper "hello")

;; ✅ 必要な関数のみインポート
(use list :only [map filter reduce])
```

### 3.5 早期リターンを活用

```qi
;; ✅ ガード節
(defn process [data]
  (if (nil? data)
    nil
    (do
      (validate data)
      (transform data))))

;; ✅ matchでの早期リターン
(match status
  :invalid -> nil
  :pending -> (process-pending)
  :completed -> result)
```

### 3.6 f-stringを活用

```qi
;; ✅ f-string
(println f"User {name} has {count} items")

;; ❌ str連結
(println (str "User " name " has " count " items"))
```

---

## 4. アンチパターン

### 4.1 深いネスト

```qi
;; ❌ 深いネスト
(if condition1
  (if condition2
    (if condition3
      (do-something)
      default)
    default)
  default)

;; ✅ パイプラインやmatch
(value
  |>? validate1
  |>? validate2
  |>? validate3
  |>? do-something)
```

### 4.2 長すぎる関数

```qi
;; ❌ 100行を超える関数
(defn huge-function [...]
  ;; 100+ lines
  )

;; ✅ 小さな関数に分割
(defn- step1 [x] ...)
(defn- step2 [x] ...)
(defn main-function [x]
  (x |> step1 |> step2))
```

### 4.3 グローバル状態の乱用

```qi
;; ❌ グローバル変数の直接変更
(def counter 0)
(defn increment [] (set! counter (+ counter 1)))

;; ✅ atomを使用
(def counter (atom 0))
(defn increment [] (swap! counter inc))
```

### 4.4 意味のない変数名

```qi
;; ❌ 悪い例
(defn f [x y z] ...)
(let [a 1 b 2] ...)

;; ✅ 良い例
(defn calculate-total [price quantity tax] ...)
(let [user-id 1 user-name "alice"] ...)
```

---

## 5. フォーマッター設定

### 5.1 デフォルト設定

```edn
;; .qi-format.edn
{:indent-width 2
 :max-line-length 100
 :pipeline-newline true
 :pipeline-operator-position :leading
 :match-arrow-spacing :both
 :align-map-values true
 :sort-use-declarations false
 :blank-lines-between-defs 1
 :blank-lines-between-sections 2}
```

### 5.2 設定項目

| 項目 | デフォルト | 説明 |
|------|-----------|------|
| `indent-width` | `2` | インデント幅（スペース数） |
| `max-line-length` | `100` | 最大行幅（文字数） |
| `pipeline-newline` | `true` | パイプラインを改行するか |
| `pipeline-operator-position` | `:leading` | パイプライン演算子の位置（`:leading` or `:trailing`） |
| `match-arrow-spacing` | `:both` | matchアローのスペース（`:both`, `:before`, `:after`, `:none`） |
| `align-map-values` | `true` | マップの値を揃えるか |
| `sort-use-declarations` | `false` | use宣言をソートするか |
| `blank-lines-between-defs` | `1` | トップレベル定義間の空行を 0〜2 行の範囲で正規化（`def`/`defn`/`defn-` 前は最低 1 行確保） |
| `blank-lines-between-sections` | `2` | セクション間の空行数 |

現行スタイルガイドの値がデフォルトですが、チームポリシーに合わせて `.qi-format.edn` で上書きしても問題ありません。

`.qi-format.edn` はフォームを含むディレクトリ、またはカレントディレクトリのルートに配置します。次のような EDN マップを書いてください：

```clojure
{:indent-width 2
 :blank-lines-between-defs 1
 :max-line-length 100}
```

数値以外の値や未対応キーは無視され、パーサーエラーになった場合はデフォルト値にフォールバックします。

### 5.3 使用方法

```bash
# ファイルをフォーマット（上書き）
qi fmt src/main.qi

# ファイルをフォーマット（標準出力）
qi fmt --check src/main.qi

# ディレクトリを再帰的にフォーマット
qi fmt src/

# 標準入力からフォーマット
cat src/main.qi | qi fmt --stdin
```

### 5.4 フォーマッターチェックリスト

`qi fmt` は以下のポイントを満たすよう実装されています（設定値で挙動が変化するものは `.qi-format.edn` 参照）。

- 文字列リテラルの改行・クォート・エスケープ表現は入力を尊重し再エンコードしない
- コメントを削除・結合せず、意味を壊さない範囲で位置を維持する
- `blank-lines-between-defs` に従ってトップレベル定義間の空行数を 0〜2 行に正規化する
- `def`/`defn`/`defn-` 直前には最低 1 行の空行を確保し、直前コメントを許容する
- 本ガイドで定義されていないレイアウトは、既存の構造を極力保持する

---

## 参考

このスタイルガイドは以下を参考にしています：

- [Clojure Style Guide](https://github.com/bbatsov/clojure-style-guide)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- [Elixir Style Guide](https://github.com/christopheradams/elixir_style_guide)

---

## ライセンス

MIT
