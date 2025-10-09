# Qi言語仕様（完全版）

## 言語概要

**Qi - A Lisp that flows**

シンプル、高速、簡潔なモダンLisp系言語。パイプライン、パターンマッチング、並行・並列処理に強い。

### 哲学
- **Simple**: 特殊形式8つ、記法最小限
- **Fast**: 軽量・高速起動・JITコンパイル
- **Concise**: 短い関数名、パイプライン、関数型

### ファイル拡張子
```
.qi
```

## 1. 基本設計

### 名前空間
**Lisp-1（Scheme派）** - 変数と関数は同じ名前空間
```lisp
(def add (fn [x y] (+ x y)))
(def op add)           ;; 関数を変数に代入
(op 1 2)               ;; 3
```

### nil と bool
**nil と bool は別物、ただし条件式では nil も falsy**
```lisp
nil false true          ;; 3つの異なる値
(if nil "yes" "no")     ;; "no" (nilはfalsy)
(if false "yes" "no")   ;; "no" (falseはfalsy)
(if 0 "yes" "no")       ;; "yes" (0はtruthy)
(if "" "yes" "no")      ;; "yes" (空文字もtruthy)

;; 明示的な比較
(= x nil)               ;; nilチェック
(= x false)             ;; falseチェック
```

### 名前空間の優先順位
**core が最優先（先勝）**
```lisp
;; coreの関数が優先
(get {:a 1} :a)         ;; マップのget

;; 他のモジュールは明示的に
(use str :as s)
(s/get "hello" 0)       ;; 文字列のget（"h"）
```

## 2. 特殊形式（8つ）

### `def` - グローバル定義
```lisp
(def x 42)
(def greet (fn [name] (str "Hello, " name)))
(def ops [+ - * /])
```

### `fn` - 関数定義
```lisp
(fn [x] (* x 2))
(fn [x y] (+ x y))
(fn [] (log "no args"))

;; 可変長引数
(fn [& args] (apply + args))

;; 分解
(fn [(x . y)] (list x y))
```

### `let` - ローカル束縛
```lisp
(let [x 10 y 20]
  (+ x y))

;; ネスト可能
(let [a 1]
  (let [b 2]
    (+ a b)))

;; 分解
(let [(x . y) '(a b c)]
  (list x y))  ;; (a (b c))
```

### `do` - 順次実行
```lisp
(do
  (log "first")
  (log "second")
  42)  ;; 最後の式の値を返す
```

### `if` - 条件分岐
```lisp
;; 基本形
(if test then else)

;; 実用例
(if (> x 10) "big" "small")

;; else省略可能（省略時はnil）
(if (valid? data)
  (process data))

;; ネスト
(if (> x 0)
    (if (< x 10) "small positive" "big positive")
    "negative or zero")
```

### `match` - パターンマッチング
```lisp
;; 値のマッチ
(match x
  0 -> "zero"
  1 -> "one"
  n -> (str "other: " n))

;; nil/boolの区別
(match result
  nil -> "not found"
  false -> "explicitly false"
  true -> "success"
  v -> (str "value: " v))

;; マップのマッチ
(match data
  {:type "user" :name n} -> (greet n)
  {:type "admin"} -> "admin"
  _ -> "unknown")

;; リストのマッチ
(match lst
  [] -> "empty"
  [x] -> x
  [x ...rest] -> (str "first: " x))

;; ガード条件
(match x
  n when (> n 0) -> "positive"
  n when (< n 0) -> "negative"
  _ -> "zero")
```

### `try` - エラー処理
```lisp
;; {:ok result} または {:error e} を返す
(match (try (risky-operation))
  {:ok result} -> result
  {:error e} -> (log e))

;; パイプラインと組み合わせ
(match (try
         (url
          |> http-get
          |> parse
          |> process))
  {:ok data} -> data
  {:error e} -> [])
```

### `defer` - 遅延実行
```lisp
;; スコープ終了時に実行
(def process-file (fn [path]
  (let [f (open path)]
    (do
      (defer (close f))  ;; 関数終了時に必ず実行
      (read f)))))

;; 複数のdefer（LIFO: 後入れ先出し）
(do
  (defer (log "3"))
  (defer (log "2"))
  (defer (log "1"))
  (work))
;; 実行順: work → "1" → "2" → "3"

;; エラー時も実行される
(def safe-process (fn []
  (do
    (defer (cleanup))
    (try (risky-op)))))
```

## 3. 演算子

### `|>` - パイプライン
```lisp
;; 左から右へデータを流す
(x |> f |> g |> h)
;; (h (g (f x))) と同じ

;; 実用例
(data
 |> parse-json
 |> (filter valid?)
 |> (map transform)
 |> (reduce merge {}))

;; 関数に複数引数を渡す
(10 |> (+ 5))  ;; (+ 10 5) = 15

;; 読みやすいデータ処理
(users
 |> (filter active?)
 |> (map :email)
 |> (take 10)
 |> (join ", "))
```

## 4. データ構造

### リスト
```lisp
(1 2 3)
()  ;; 空リスト
(first (1 2 3))  ;; 1
(rest (1 2 3))   ;; (2 3)
```

### マップ
```lisp
{:name "Alice" :age 30}
{}  ;; 空マップ
(get {:a 1} :a)           ;; 1
(:name {:name "Alice"})   ;; "Alice" (キーワードは関数)
```

### ベクタ
```lisp
[1 2 3]
(get [10 20 30] 1)  ;; 20
```

### 関数
```lisp
;; 関数もデータ
(def ops [+ - * /])
((get ops 0) 1 2)  ;; 3

;; 関数のマップ
(def handlers {:get handle-get :post handle-post})
((get handlers :get) request)
```

## 5. コア関数

### リスト操作
```lisp
map filter reduce       ;; 高階関数
first rest last         ;; アクセス
take drop               ;; 部分取得
flatten concat          ;; 結合
len empty?              ;; 情報
conj                    ;; 追加
range                   ;; (range 10) => (0 1 2 ... 9)
reverse                 ;; 反転
sort sort-by            ;; ソート
group-by                ;; グループ化
zip                     ;; リストを組み合わせ
```

### 数値・比較
```lisp
+ - * /                 ;; 算術演算
= < > <= >=             ;; 比較
inc dec                 ;; インクリメント/デクリメント
sum                     ;; 合計
abs                     ;; 絶対値
min max                 ;; 最小/最大
```

### 論理
```lisp
and or not
```

### マップ操作
```lisp
get keys vals           ;; アクセス
assoc dissoc merge      ;; 変更
update                  ;; 更新
select-keys             ;; キー選択
```

### 基本文字列
```lisp
str                     ;; 連結（core）
len empty?              ;; 長さ、空チェック（core）
```

### IO
```lisp
print log               ;; 出力
slurp spit              ;; ファイル読み書き
open close read write   ;; ファイル操作
```

### 並行・並列
```lisp
chan put take take-n    ;; チャネル
go                      ;; 並行実行
pmap                    ;; 並列map
loop recur              ;; ループ
```

### 状態管理
```lisp
atom                    ;; アトム作成
swap!                   ;; アトミック更新
deref                   ;; 値取得
@                       ;; derefの短縮形
reset!                  ;; 値を直接セット
```

### エラー処理
```lisp
error                   ;; 例外を投げる（回復不能）
;; 通常は {:ok ...} / {:error ...} を返す（回復可能）
```

### メタプログラミング
```lisp
uvar                    ;; 一意な変数を生成
variable                ;; 変数かどうかチェック
macro?                  ;; マクロかどうか
eval                    ;; 式を評価

;; 定数
vmark                   ;; uvarのマーカー
```

## 6. ループ構造

### `loop` / `recur`
```lisp
;; 基本形
(loop [var1 val1 var2 val2 ...]
  body
  (recur new-val1 new-val2 ...))

;; 階乗
(def factorial (fn [n]
  (loop [i n acc 1]
    (if (= i 0)
      acc
      (recur (dec i) (* acc i))))))

(factorial 5)  ;; 120

;; リスト処理
(def my-map (fn [f lst]
  (loop [items lst result []]
    (match items
      [] -> result
      [x ...rest] -> (recur rest (conj result (f x)))))))

;; while風
(def count-down (fn [n]
  (loop [i n]
    (if (<= i 0)
      "done"
      (do
        (log i)
        (recur (dec i)))))))
```

## 7. エラー処理戦略

### 回復可能 - {:ok/:error}
```lisp
;; 関数が結果を返す
(def divide (fn [x y]
  (if (= y 0)
    {:error "division by zero"}
    {:ok (/ x y)})))

(match (divide 10 2)
  {:ok result} -> result
  {:error e} -> (log e))

(def parse-int (fn [s]
  (match (try-parse s)
    nil -> {:error "not a number"}
    n -> {:ok n})))
```

### 回復不能 - error
```lisp
;; 致命的エラーは error で投げる
(def critical-init (fn []
  (if (not (file-exists? "config.qi"))
    (error "config.qi not found")
    (load-config))))

(def factorial (fn [n]
  (if (< n 0)
    (error "negative input not allowed")
    (loop [i n acc 1]
      (if (= i 0)
        acc
        (recur (dec i) (* acc i)))))))

;; try でキャッチ
(match (try (factorial -5))
  {:ok result} -> result
  {:error e} -> (log (str "Error: " e)))
```

## 8. ユニーク変数（uvars）

### 基本
```lisp
;; 一意な変数を生成
(def uvar ()
  (join))  ;; 新しいペアを返す

;; マーカー
(def vmark (join))

;; 変数判定
(def variable (x)
  (or (and (symbol x) (not (mem x '(nil t o apply))))
      (and (pair x) (id (car x) vmark))))
```

### マクロでの使用
```lisp
;; 変数名の衝突を避ける
(mac when (test & body)
  (let [g (uvar)]
    `(let [,g ,test]
       (if ,g (do ,@body)))))

;; 展開例
(when (> x 10)
  (print x))
;; ↓
(let [#<uvar:1> (> x 10)]
  (if #<uvar:1> (do (print x))))

;; 衝突しない
(let [g 5]
  (when (> x 10)
    (+ g 1)))  ;; gはユーザーの変数
```

### 安全なマクロ
```lisp
;; aif マクロ
(mac aif (test then & else)
  (let [it (uvar)]
    `(let [,it ,test]
       (if ,it ,then ,@else))))

;; 使用例（衝突なし）
(let [it 'outer]
  (aif (find even? [1 3 5])
       it        ;; aifのit（uvar）
       it))      ;; outerのit
;; => 'outer

;; or マクロ
(mac or (& args)
  (if (no args)
      nil
      (if (no (cdr args))
          (car args)
          (let [g (uvar)]
            `(let [,g ,(car args)]
               (if ,g ,g (or ,@(cdr args))))))))

;; do マクロ
(mac do (& body)
  (reduce (fn [x y]
            (let [v (uvar)]
              `((fn [,v] ,y) ,x)))
          body))

;; 複数のuvars
(mac letu (vars & body)
  `(withs ,(fuse [list _ `(uvar)] vars)
     ,@body))

;; 使用例
(mac my-complex-macro (x y)
  (letu (a b c)
    `(let [,a ,x]
       (let [,b ,y]
         (let [,c (+ ,a ,b)]
           (list ,a ,b ,c))))))
```

## 9. モジュールシステム

### モジュール定義
```lisp
;; http.qi
(module http)

(def get (fn [url] ...))
(def post (fn [url data] ...))

(export get post)
```

### インポート
```lisp
;; パターン1: 特定の関数のみ（推奨）
(use http :only [get post])
(get url)

;; パターン2: エイリアス
(use http :as h)
(h/get url)

;; パターン3: 全てインポート
(use http :all)
(get url)

;; パターン4: リネーム
(use http :only [get :as fetch])
(fetch url)
```

### 標準モジュール

#### core（自動インポート）
```lisp
;; 基本関数全て
map filter reduce
str len empty?
uvar variable
...
```

#### str - 文字列操作
```lisp
(use str :only [
  ;; 検索
  contains? starts-with? ends-with?
  index-of last-index-of
  
  ;; 変換
  upper lower capitalize title
  trim trim-left trim-right
  pad-left pad-right
  repeat reverse
  
  ;; 分割・結合
  split lines words chars
  join
  
  ;; 置換
  replace replace-first
  
  ;; 部分文字列
  slice take-str drop-str
  
  ;; エンコード
  to-base64 from-base64
  url-encode url-decode
  html-escape html-unescape
  
  ;; パース
  parse-int parse-float
  numeric? integer? blank?
  
  ;; ハッシュ
  hash uuid
  
  ;; NLP
  word-count slugify
  
  ;; フォーマット
  indent wrap truncate
])

;; 例
(use str :as s)
(s/upper "hello")  ;; "HELLO"
(s/split "a,b,c" ",")  ;; ["a" "b" "c"]
```

#### csv - CSV/TSV処理
```lisp
(use csv :only [
  parse parse-file
  format write-file
  process-file
])

;; 基本的な使用
(csv/parse "name,age\nAlice,30\nBob,25")
;; [{:name "Alice" :age "30"} {:name "Bob" :age "25"}]

;; オプション
(csv/parse text
  {:delimiter ","
   :header true
   :skip-empty true
   :trim true
   :types {:age :int}})

;; TSV
(csv/parse text {:delimiter "\t"})

;; ファイル
(csv/parse-file "data.csv")
(csv/write-file "output.csv" data)

;; 大きいファイル
(csv/process-file "huge.csv"
  (fn [row] (process row))
  {:batch-size 1000})
```

#### regex - 正規表現
```lisp
(use regex :only [
  match match-all
  test
  replace replace-all
  split
  compile
])

;; マッチ
(regex/match "hello123" #"\d+")
;; {:matched "123" :start 5 :end 8}

;; グループキャプチャ
(regex/match "Alice:30" #"(?<name>\w+):(?<age>\d+)")
;; {:matched "Alice:30" :groups {:name "Alice" :age "30"}}

;; テスト
(regex/test "hello123" #"\d+")  ;; true

;; 置換
(regex/replace "hello123" #"\d+" "X")  ;; "helloX"
(regex/replace-all "hello123world456" #"\d+" "X")  ;; "helloXworldX"

;; コールバック置換
(regex/replace-all "hello123world456" #"\d+"
  (fn [match] (* (parse-int match) 2)))
;; "hello246world912"

;; コンパイル（再利用）
(def email-pattern (regex/compile #"^[^@]+@[^@]+\.[^@]+$"))
(regex/test "test@example.com" email-pattern)
```

#### その他
```lisp
http      ;; HTTPクライアント
json      ;; JSONパース
db        ;; データベース
io        ;; ファイルIO
math      ;; 数学関数
time      ;; 日付・時刻
test      ;; テスト
```

## 10. 文字列リテラル

### 基本
```lisp
"hello"
"hello\nworld"
"say \"hello\""
```

### 複数行
```lisp
"""
This is a
multi-line
string
"""
```

### 補間（f-string）
```lisp
f"Hello, {name}! You are {age} years old."

;; 式も使える
f"Result: {(+ 1 2)}"  ;; "Result: 3"

;; ネスト可能
f"Items: {(join \", \" items)}"

;; 実用例
(def greet (fn [user]
  f"Welcome, {(:name user)}! You have {(:messages user)} new messages."))
```

## 11. 実用例

### Webスクレイパー
```lisp
(use http :only [get])

(def scrape-prices (fn [url]
  (match (try
    (url
     |> get
     |> parse-html
     |> (select ".price")
     |> (pmap extract-number)))
    {:ok prices} -> prices
    {:error e} -> (do (log e) []))))

(def all-prices
  (["https://shop1.com" "https://shop2.com"]
   |> (pmap scrape-prices)
   |> flatten
   |> (filter (fn [p] (> p 0)))))

(log f"Average: {(/ (sum all-prices) (len all-prices))}")
```

### 安全なマクロ（uvars使用）
```lisp
;; 衝突しないaif
(mac aif (test then & else)
  (let [it (uvar)]
    `(let [,it ,test]
       (if ,it ,then ,@else))))

;; 安全なwhen
(mac when (test & body)
  (let [g (uvar)]
    `(let [,g ,test]
       (if ,g (do ,@body)))))

;; 安全なor
(mac or (& args)
  (if (no args)
      nil
      (if (no (cdr args))
          (car args)
          (let [g (uvar)]
            `(let [,g ,(car args)]
               (if ,g ,g (or ,@(cdr args))))))))
```

### CSV処理
```lisp
(use csv)
(use str :as s)

(def clean-csv (fn [file]
  (file
   |> csv/parse-file
   |> (map (fn [row]
            {:name (s/trim (:name row))
             :email (s/lower (:email row))
             :age (parse-int (:age row))}))
   |> (filter (fn [row] 
               (match (:age row)
                 {:ok n} -> (> n 0)
                 _ -> false)))
   |> (csv/write-file "cleaned.csv"))))
```

### ログ解析
```lisp
(use regex :as re)
(use str :as s)

(def parse-log (fn [line]
  (match (re/match line #"^\[(?<level>\w+)\] (?<time>[\d:]+) - (?<msg>.+)$")
    {:groups {:level l :time t :msg m}} -> {:level l :time t :msg m}
    _ -> nil)))

(def analyze-logs (fn [file]
  (file
   |> slurp
   |> s/lines
   |> (map parse-log)
   |> (filter (fn [x] (not (= x nil))))
   |> (filter (fn [x] (= (:level x) "ERROR")))
   |> (group-by :msg)
   |> (map (fn [[msg entries]] {:msg msg :count (len entries)}))
   |> (sort-by :count)
   |> reverse)))
```

### チャットサーバー
```lisp
(def clients (atom #{}))

(def broadcast (fn [msg]
  (pmap (fn [c] (send c msg)) @clients)))

(def handle-client (fn [conn]
  (do
    (swap! clients conj conn)
    (defer (swap! clients dissoc conn))
    (go
      (loop [running true]
        (if running
          (match (recv conn)
            {:msg m} -> (do (broadcast m) (recur true))
            :close -> (recur false))
          nil))))))

(listen 8080 |> (map handle-client))
```

### データパイプライン
```lisp
(use str :as s)
(use csv)

(def process-logs (fn [file]
  (match (try
    (file
     |> csv/parse-file
     |> (filter (fn [e] (= (:level e) "ERROR")))
     |> (group-by :service)
     |> (map (fn [[k v]] {:service k :count (len v)}))
     |> (sort-by :count)
     |> reverse))
    {:ok data} -> data
    {:error e} -> [])))

(def results
  (dir-files "logs/*.csv")
  |> (pmap process-logs)
  |> flatten)

(csv/write-file "report.csv" results)
```

### URL構築
```lisp
(use str :as s)

(def build-url (fn [base path params]
  (let [query (params
               |> (map (fn [[k v]] f"{k}={(s/url-encode v)}"))
               |> (s/join "&"))]
    f"{base}/{path}?{query}")))

(build-url "https://api.example.com" "search"
           {:q "hello world" :limit 10})
;; "https://api.example.com/search?q=hello%20world&limit=10"
```

### テキスト処理
```lisp
(use str :as s)
(use regex :as re)

(def clean-text (fn [text]
  (text
   |> (re/replace-all #"\s+" " ")
   |> s/trim
   |> (s/truncate 1000))))

(def extract-emails (fn [text]
  (re/match-all text #"[^@\s]+@[^@\s]+\.[^@\s]+")
  |> (map :matched)))

(def word-frequency (fn [text]
  (text
   |> s/lower
   |> s/words
   |> (group-by identity)
   |> (map (fn [[word instances]] {:word word :count (len instances)}))
   |> (sort-by :count)
   |> reverse)))
```

## 12. 言語文化

### 命名規則
- **関数名**: 短く直感的（`len`, `trim`, `split`）
- **モジュール名**: 短く明確（`http`, `json`, `csv`, `regex`）
- **述語関数**: `?` で終わる（`empty?`, `valid?`）
- **破壊的操作**: `!` で終わる（`swap!`, `reset!`）

### コーディングスタイル
- パイプライン `|>` を積極的に使う
- 単純な分岐は `if`、パターンマッチは `match`
- `loop`/`recur` で末尾再帰
- `defer` でリソース管理
- 回復可能なエラーは `{:ok/:error}`、致命的なエラーは `error`
- f-string `f"..."` で文字列補間
- マクロでは `uvar` で変数衝突を回避
- 短い変数名OK（スコープが短ければ）

### 避けるべきこと
- 長い関数名・モジュール名
- 深いネスト（パイプラインを使う）
- グローバル変数の乱用
- core関数との名前衝突
- マクロで固定の変数名を使う（`uvar`を使う）
- 過度な最適化（まず動くコードを書く）

## 13. コマンドラインツール

```bash
# REPL起動
$ qi

# ファイル実行
$ qi run hello.qi

# プロジェクト作成
$ qi new myapp

# テスト実行
$ qi test

# ビルド
$ qi build myapp.qi

# パッケージ管理
$ qi install http json
$ qi update
```

## まとめ

**名前**: Qi  
**特殊形式**: `def` `fn` `let` `do` `if` `match` `try` `defer`（8つ）  
**演算子**: `|>`  
**ループ**: `loop` `recur`  
**エラー**: `error`（致命的）、`{:ok/:error}`（通常）  
**メタ**: `uvar` `variable` `vmark`（マクロ用）  
**データ**: リスト、マップ、ベクタ、関数  
**名前空間**: Lisp-1、coreが優先  
**nil/bool**: 別物、条件式では nil も falsy  
**並行**: `go`（並行）、`pmap`（並列）、チャネル  
**文字列**: f-string補間、str/csv/regexモジュール  
**哲学**: Simple, Fast, Concise - エネルギーの流れのようなプログラミング
