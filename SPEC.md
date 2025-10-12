# Qi言語仕様

## 言語概要

**Qi - A Lisp that flows**

シンプル、高速、簡潔なモダンLisp系言語。パイプライン、パターンマッチング、並行・並列処理に強い。

**並列、並行を簡単にできるのはQiのキモ** - スレッドセーフな設計と3層並行処理アーキテクチャ。

**実装状況**: 本仕様書には計画中の機能も含まれています。実装済みの機能には ✅ マーク、未実装の機能には 🚧 マークを付記しています。

---

## 言語哲学 - Flow-Oriented Programming

### 核となる思想

**「データは流れ、プログラムは流れを設計する」**

Qiは**Flow-Oriented Programming**（流れ指向プログラミング）を体現します：

1. **データの流れが第一級市民**
   - パイプライン演算子 `|>` が言語の中心
   - `match` は流れを分岐・変換する制御構造（`=> 変換` で流れを継続）
   - 小さな変換を組み合わせて大きな流れを作る
   - Unix哲学の「Do One Thing Well」を関数型で実現

2. **Simple, Fast, Concise**
   - **Simple**: 特殊形式9つ、記法最小限、学習曲線が緩やか
   - **Fast**: 軽量・高速起動・将来的にJITコンパイル
   - **Concise**: 短い関数名、パイプライン、`defn`糖衣構文で表現力豊か

3. **エネルギーの流れ**
   - データは一方向に流れる（左から右、上から下）
   - 副作用はタップ（`tap>`）で観察
   - 並列処理は流れの分岐・合流として表現
   - **並行・並列を簡単に** - スレッドセーフな設計で自然な並列化

4. **実用主義**
   - Lisp的純粋性より実用性を優先
   - モダンな機能（f-string、パターンマッチング）を積極採用
   - バッテリー同梱（豊富な文字列操作、ユーティリティ）

---

### Flow哲学の進化

Qiは段階的にFlow機能を強化していきます：

**フェーズ1（✅ 現在）**:
- `|>` 基本パイプライン - 逐次処理
- `match` 基本パターンマッチング - 構造分解と分岐

**フェーズ2（✅ 完了）**:

*パイプライン強化*:
- ✅ `||>` 並列パイプライン - 自動的にpmap化（実装済み）
- ✅ `tap>` 副作用タップ - デバッグ・ログ観察（実装済み）
- 🚧 `flow` DSL - 分岐・合流を含む複雑な流れ（未実装）

*match強化* ⭐ **Qi独自の差別化要素**:
- ✅ `:as` 束縛 - 部分と全体を両方使える
- ✅ `=> 変換` - マッチ時にパイプライン的変換（matchの中に流れを埋め込む）
- 🚧 `or` パターン - 複数パターンで同じ処理（`1 | 2 | 3 -> "small"`）（未実装）

**フェーズ3（✅ 完了）**:
- ✅ 並列処理基盤 - スレッドセーフEvaluator、pmap完全並列化（実装済み）
- ✅ 並行処理 - go/chan、パイプライン、async/await（実装済み）
- ✅ `~>` 非同期パイプライン - go/chan統合（実装済み）
- ✅ `stream` 遅延評価ストリーム - 巨大データ処理（無限データ構造対応）（実装済み）
- 🔜 再利用可能な「小パイプ」文化の確立（進行中）

---

### 設計原則

1. **読みやすさ > 書きやすさ**
   - パイプラインは上から下、左から右に読める
   - データの流れが一目で分かる

2. **合成可能性**
   - 小さな関数を組み合わせて大きな処理を作る
   - 各ステップは独立してテスト可能

3. **段階的開示**
   - 初心者: 基本的な `|>` から始められる
   - 中級者: `match`、`loop`、マクロを活用
   - 上級者: メタプログラミング、並列処理を駆使

4. **実行時の効率**
   - パイプラインは最適化される
   - 遅延評価で不要な計算を回避
   - 並列処理で自然にスケール

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

---

## 2. パイプライン拡張 - Flow DSL

### 🎯 ビジョン: 流れを設計する言語

Qiはパイプライン演算子を段階的に拡張し、**データの流れを直感的に表現**できる言語を目指します。

---

### パイプライン演算子の体系

| 演算子 | 意味 | 状態 | 用途 |
|--------|------|------|------|
| `|>` | 逐次パイプ | ✅ 実装済み | 基本的なデータ変換 |
| `\|>?` | Railway パイプ | ✅ 実装済み | エラーハンドリング、Result型の連鎖 |
| `||>` | 並列パイプ | ✅ 実装済み | 自動的にpmap化、リスト処理の並列化 |
| `tap>` | 副作用タップ | ✅ 実装済み | デバッグ、ログ、モニタリング（関数として） |
| `~>` | 非同期パイプ | ✅ 実装済み | go/chan統合、非同期IO |

---

### ✅ `|>` 基本パイプライン（実装済み）

**左から右へデータを流す**

```lisp
;; 基本
(data |> parse |> transform |> save)

;; ネスト回避
(data
 |> (filter valid?)
 |> (map transform)
 |> (reduce merge {}))

;; 引数付き関数
(10 |> (+ 5) |> (* 2))  ;; (+ 10 5) |> (* 2) => 30

;; 実用例: URL構築
(params
 |> (map (fn [[k v]] f"{k}={v}"))
 |> (join "&")
 |> (str base-url "?" _))
```

---

### ✅ `||>` 並列パイプライン（実装済み）

**自動的にpmapに展開**

```lisp
;; 並列処理
(urls ||> http-get ||> parse-json)
;; ↓ 展開
(urls |> (pmap http-get) |> (pmap parse-json))

;; 基本的な使い方
([1 2 3 4 5] ||> inc)  ;; (2 3 4 5 6)

;; CPU集約的処理
(images ||> resize ||> compress ||> save)

;; データ分析
(files
 ||> load-csv
 ||> analyze
 |> merge-results)  ;; 最後は逐次でマージ

;; 複雑なパイプライン
(data
 ||> (fn [x] (* x 2))
 |> (filter (fn [n] (> n 50)))
 |> sum)
```

**実装**:
- lexer: `||>`を`Token::ParallelPipe`として認識
- parser: `x ||> f` → `(pmap f x)`に展開
- 現在はシングルスレッド版pmapを使用
- 将来的にEvaluatorをスレッドセーフ化すれば真の並列化が可能

---

### ✅ `|>?` Railway Pipeline（実装済み）⭐ **Phase 4.5の主要機能**

**エラーハンドリングを流れの中に組み込む** - Railway Oriented Programming

```lisp
;; Result型: {:ok value} または {:error message}
;; |>? は {:ok value} なら次の関数に値を渡し、{:error e} ならショートサーキット

;; 基本的な使い方
({:ok 10}
 |>? (fn [x] {:ok (* x 2)})
 |>? (fn [x] {:ok (+ x 5)}))
;; => {:ok 25}

;; エラー時はショートサーキット
({:ok 10}
 |>? (fn [x] {:error "Something went wrong"})
 |>? (fn [x] {:ok (* x 2)}))  ;; この関数は実行されない
;; => {:error "Something went wrong"}

;; JSONパース + データ変換
("{\"name\":\"Alice\",\"age\":30}"
 |> json/parse                    ;; => {:ok {...}}
 |>? (fn [data] {:ok (get data "name")})
 |>? (fn [name] {:ok (upper name)}))
;; => {:ok "ALICE"}

;; HTTPリクエスト + エラーハンドリング
("https://api.example.com/users/123"
 |> http/get                      ;; => {:ok {:status 200 :body "..."}}
 |>? (fn [resp] (get resp "body"))
 |>? json/parse
 |>? (fn [data] {:ok (get data "user")}))
;; エラー時は自動的に伝播

;; 複雑な処理チェーン
(user-id
 |> (str "https://api.example.com/users/" _)
 |> http/get
 |>? (fn [resp]
       (if (= (get resp "status") 200)
         {:ok (get resp "body")}
         {:error "Failed to fetch"}))
 |>? json/parse
 |>? validate-user
 |>? save-to-db)
```

**使い分け**:
- `|>`: 通常のデータ変換（エラーなし）
- `|>?`: エラーが起こりうる処理（API、ファイルIO、パース）

**実装**:
- lexer: `|>?`を`Token::PipeRailway`として認識
- parser: `x |>? f` → `(_railway-pipe f x)`に展開
- `_railway-pipe`: Result型マップを検査し、`:ok`なら関数適用、`:error`ならそのまま返す

**設計哲学**:
エラーハンドリングを流れの一部として表現。try-catchのネストを避け、データフローが明確になる。JSONやHTTPなどのWeb開発機能と完璧に統合。

---

### ✅ `tap>` 副作用タップ（実装済み）

**流れを止めずに観察**（Unix `tee`相当）

```lisp
;; tap>は関数として実装
(def tap> (fn [f]
  (fn [x]
    (do
      (f x)
      x))))

;; デバッグ
(data
 |> clean
 |> ((tap> (fn [x] (print f"After clean: {x}"))))
 |> analyze
 |> ((tap> (fn [x] (print f"After analyze: {x}"))))
 |> save)

;; ログ
(requests
 |> ((tap> log-request))
 |> process
 |> ((tap> log-response)))

;; 簡潔な使い方
([1 2 3]
 |> (map inc)
 |> ((tap> print))
 |> sum)
```

**実装**:
- 高階関数として実装
- `(tap> f)`は`(fn [x] (do (f x) x))`を返す
- パイプライン内で`((tap> f))`として使用

---

### 🔜 `flow` マクロ - 構造化された流れ（近未来）

**分岐・合流を含む複雑なパイプライン**

```lisp
;; 基本的なflow
(flow data
  |> parse
  |> transform
  |> save)

;; 分岐
(flow data
  |> parse
  |> branch
       [valid?   |> process |> save]
       [invalid? |> log-error]
       [else     |> quarantine])

;; タップとの組み合わせ
(flow request
  |> tap> log-request
  |> validate
  |> process
  |> tap> log-response
  |> format-result)

;; 再利用可能な小パイプ
(def normalize-text
  (flow |> trim |> lower |> (replace #"\\s+" " ")))

(texts |> normalize-text |> unique)
```

---

### ✅ `~>` 非同期パイプライン（実装済み）

**並行処理との統合 - goroutine風の非同期実行**

`~>` 演算子はパイプラインをgoroutineで自動実行し、結果をチャネルで返します。

```lisp
;; 基本的な非同期パイプライン
(def result (data ~> transform ~> process))  ; 即座にチャネルを返す
(recv! result)  ; 結果を受信

;; 複数の非同期処理
(def r1 (10 ~> inc ~> double))
(def r2 (20 ~> double ~> inc))
(println (recv! r1) (recv! r2))  ; 並行実行

;; goブロック内でも利用可能
(go
  (data ~> transform ~> (send! output-chan)))
```

---

### ✅ `stream` 遅延評価（実装済み）

**巨大データの効率的処理 - 遅延評価と無限データ構造**

Streamは値を必要になるまで計算しない遅延評価のデータ構造です。
無限データ構造や大きなデータセットをメモリ効率的に扱えます。

#### Stream作成

```lisp
;; コレクションからストリーム作成
(stream [1 2 3 4 5])

;; 範囲ストリーム
(range-stream 0 10)  ;; 0から9まで

;; 無限ストリーム：同じ値を繰り返し
(repeat 42)  ;; 42, 42, 42, ...

;; 無限ストリーム：リストを循環
(cycle [1 2 3])  ;; 1, 2, 3, 1, 2, 3, ...

;; 無限ストリーム：関数を反復適用
(iterate (fn [x] (* x 2)) 1)  ;; 1, 2, 4, 8, 16, 32, ...
```

#### Stream変換

```lisp
;; map: 各要素に関数を適用
(def s (range-stream 1 6))
(def s2 (stream-map (fn [x] (* x 2)) s))
(realize s2)  ;; (2 4 6 8 10)

;; filter: 条件に合う要素のみ
(def s (range-stream 1 11))
(def s2 (stream-filter (fn [x] (= (% x 2) 0)) s))
(realize s2)  ;; (2 4 6 8 10)

;; take: 最初のn個を取得（無限ストリームを有限化）
(def s (repeat 42))
(def s2 (stream-take 5 s))
(realize s2)  ;; (42 42 42 42 42)

;; drop: 最初のn個をスキップ
(def s (range-stream 0 10))
(def s2 (stream-drop 5 s))
(realize s2)  ;; (5 6 7 8 9)
```

#### Stream実行

```lisp
;; realize: ストリームをリストに変換（全要素を計算）
(realize (stream [1 2 3]))  ;; (1 2 3)

;; ⚠️ 注意: 無限ストリームをrealizeすると無限ループ
;; (realize (repeat 42))  ;; NG: 永遠に終わらない

;; 正しい使い方: takeで有限化してからrealize
(realize (stream-take 5 (repeat 42)))  ;; OK
```

#### パイプラインとの統合

```lisp
;; 既存の |> パイプライン演算子で使える
[1 2 3 4 5]
  |> stream
  |> (stream-map (fn [x] (* x x)))
  |> (stream-filter (fn [x] (> x 10)))
  |> realize
;; (16 25)

;; 無限ストリームの処理
1
  |> (iterate (fn [x] (* x 2)))
  |> (stream-take 10)
  |> realize
;; (1 2 4 8 16 32 64 128 256 512)

;; 複雑な変換チェーン
(range-stream 1 100)
  |> (stream-map (fn [x] (* x x)))
  |> (stream-filter (fn [x] (= (% x 3) 0)))
  |> (stream-take 5)
  |> realize
;; (9 36 81 144 225)
```

#### 実用例

```lisp
;; 素数の無限ストリーム（概念）
(def primes
  (2
   |> (iterate inc)
   |> (stream-filter prime?)))

(realize (stream-take 10 primes))  ;; 最初の10個の素数

;; フィボナッチ数列
(def fib-stream
  (iterate
    (fn [[a b]] [b (+ a b)])
    [0 1]))

(realize
  (stream-take 10 fib-stream)
  |> (map first))  ;; (0 1 1 2 3 5 8 13 21 34)

;; データ処理パイプライン
(defn process-data [data]
  (data
   |> stream
   |> (stream-map parse)
   |> (stream-filter valid?)
   |> (stream-take 1000)
   |> realize))
```

#### ✅ I/Oストリーム（実装済み）

**ファイルとHTTPデータの遅延読み込み - テキスト＆バイナリ対応**

##### テキストモード（行ベース）

```lisp
;; file-stream: ファイルを行ごとに遅延読み込み（io.rs）
(file-stream "large.log")
  |> (stream-filter error-line?)
  |> (stream-map parse)
  |> (stream-take 100)
  |> realize

;; http/get-stream: HTTPレスポンスを行ごとに読み込み（http.rs）
(http/get-stream "https://api.example.com/data")
  |> (stream-take 10)
  |> realize

;; http/post-stream: POSTリクエストでストリーミング受信
(http/post-stream "https://api.example.com/upload" {:data "value"})
  |> (stream-take 10)
  |> realize

;; http/request-stream: 詳細設定でストリーミング
(http/request-stream {
  :method "GET"
  :url "https://api.example.com/stream"
})
  |> (stream-filter important?)
  |> realize
```

##### バイナリモード（バイトチャンク）

```lisp
;; file-stream :bytes - ファイルを4KBチャンクで読み込み
(file-stream "image.png" :bytes)
  |> (stream-take 10)
  |> realize
;; => Vector of Integers (bytes) のリスト

;; http/get-stream :bytes - HTTPバイナリダウンロード
(http/get-stream "https://example.com/file.bin" :bytes)
  |> (stream-map process-chunk)
  |> realize

;; バイト処理の例
(def bytes (first (realize (stream-take 1 (file-stream "data.bin" :bytes)))))
(def sum (reduce + bytes))  ; バイトの合計
(println sum)

;; 画像ダウンロード
(http/get-stream "https://example.com/logo.png" :bytes)
  |> realize
  |> flatten
  |> (write-bytes "logo.png")  ; write-bytes は将来実装
```

**モード比較**:

| モード | 用途 | 戻り値 | 例 |
|--------|------|--------|-----|
| テキスト（デフォルト） | ログ、CSV、JSON | String（行ごと） | `(file-stream "data.txt")` |
| バイナリ（`:bytes`） | 画像、動画、バイナリ | Vector of Integers（4KBチャンク） | `(file-stream "image.png" :bytes)` |

;; CSVファイルの処理
(file-stream "data.csv")
  |> (stream-drop 1)  ; ヘッダースキップ
  |> (stream-map (fn [line] (split line ",")))
  |> (stream-filter (fn [cols] (> (len cols) 2)))
  |> (stream-take 1000)
  |> realize

;; HTTPからJSONを取得してパース
(http/get-stream "https://jsonplaceholder.typicode.com/todos/1")
  |> realize
  |> (join "\n")
  |> json/parse  ; json モジュールが実装されたら使える
```

**実用例: ログファイル解析**

```lisp
;; 大きなログファイルをメモリ効率的に処理
(defn analyze-logs [file]
  (file-stream file
   |> (stream-filter (fn [line] (contains? line "ERROR")))
   |> (stream-map parse-log-line)
   |> (stream-take 100)  ; 最初の100エラー
   |> realize))

;; 結果を取得
(def errors (analyze-logs "/var/log/app.log"))
(println (str "Found " (len errors) " errors"))
```

---

### パイプライン文化

**Unix哲学 × 関数型 × Lisp**

```lisp
;; 小さなパイプを定義
(def clean-text
  (flow |> trim |> lower |> remove-punctuation))

(def extract-emails
  (flow |> (split "\\s+") |> (filter email?)))

(def dedupe
  (flow |> sort |> unique))

;; 組み合わせて使う
(document
 |> clean-text
 |> extract-emails
 |> dedupe
 |> (join ", "))
```

---

## 3. 特殊形式（9つ）✅

### ✅ `def` - グローバル定義
```lisp
(def x 42)
(def greet (fn [name] (str "Hello, " name)))
(def ops [+ - * /])
```

### ✅ `defn` - 関数定義（糖衣構文）
```lisp
;; 基本形式
(defn greet [name]
  (str "Hello, " name))

;; 可変長引数
(defn sum [& nums]
  (reduce + 0 nums))

;; ドキュメント付き（将来サポート予定）
(defn greet "挨拶する" [name]
  (str "Hello, " name))

;; defnは以下のように展開される
(defn greet [name] body)
;; ↓
(def greet (fn [name] body))
```

**Note**: `defn`は`def + fn`の糖衣構文です。ドキュメント文字列/マップは認識されますが、現在は無視されます（将来のドキュメントシステムで活用予定）。

### ✅ `fn` - 関数定義
```lisp
(fn [x] (* x 2))
(fn [x y] (+ x y))
(fn [] (log "no args"))

;; 可変長引数
(fn [& args] (apply + args))

;; 分解
(fn [(x . y)] (list x y))
```

### ✅ `let` - ローカル束縛
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

### ✅ `do` - 順次実行
```lisp
(do
  (log "first")
  (log "second")
  42)  ;; 最後の式の値を返す
```

### ✅ `if` - 条件分岐
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

### ✅ `match` - パターンマッチング（Flow-Oriented）

Qiのパターンマッチは**データの流れを分岐させる制御構造**です。単なる条件分岐ではなく、データ構造を分解・変換・検証しながら処理を振り分けます。

#### ✅ 基本パターン（実装済み）

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

#### ✅ 拡張パターン（実装済み - Flow強化）

**1. `:as` 束縛 - 部分と全体の両方を使う** ✅
```lisp
;; パターンマッチした全体を変数に束縛
(match data
  {:user {:name n :age a} :as u} -> (do
    (log u)           ;; 全体をログ
    (process n a)))   ;; 部分を処理

;; ネストした構造でも使える
(match response
  {:body {:user u :posts ps} :as body} -> (cache body)
  {:error e :as err} -> (log err))
```

**2. `=> 変換` - マッチ時にデータを流す** ✅ ⭐ **Qi独自の強力な機能**
```lisp
;; 束縛と同時に変換関数を適用（パイプライン的）
(match data
  {:price p => parse-float} -> (calc-tax p)
  {:name n => lower} -> (log n)
  {:created-at t => parse-date} -> (format t))

;; 複数の変換をつなげる
(match input
  {:raw r => trim => lower => (split " ")} -> (process-words r))

;; 実用例: APIレスポンス処理
(match (http-get "/api/user")
  {:body b => parse-json} -> (extract-user b)
  {:status s => str} when (= s "404") -> nil
  _ -> (error "unexpected response"))
```

#### 🚧 将来の拡張パターン

**3. `or` パターン - 複数パターンで同じ処理** 🚧
```lisp
;; 複数の値にマッチ
(match status
  (200 or 201 or 204) -> "success"
  (400 or 401 or 403) -> "client error"
  (500 or 502 or 503) -> "server error"
  _ -> "unknown")

;; 複数の構造にマッチ
(match event
  ({:type "click"} or {:type "tap"}) -> (handle-interaction)
  ({:type "scroll"} or {:type "drag"}) -> (handle-movement))
```

**4. ネスト + ガード - 構造的な条件分岐**
```lisp
;; 深いネストでも読みやすい
(match request
  {:user {:age a :country c}} when (and (>= a 18) (= c "JP")) -> (allow)
  {:user {:age a}} when (< a 18) -> (error "age restriction")
  _ -> (deny))

;; Flow的な読み方: データ構造を分解 → ガードで検証 → 処理
```

**5. ワイルドカード `_` - 関心のある部分だけ抽出**
```lisp
;; 一部のフィールドだけ使う
(match data
  {:user {:name _ :age a :city c}} -> (process-location a c)
  {:error _} -> "error occurred")

;; リストの一部をスキップ
(match coords
  [_ y _] -> y  ;; y座標だけ取り出す
  _ -> 0)
```

**6. 配列の複数束縛**
```lisp
;; 複数要素を同時に束縛
(match data
  [{:id id1} {:id id2}] -> (compare id1 id2)
  [first ...rest] -> (process-batch first rest))

;; パイプラインと組み合わせ
(match (coords |> (take 2))
  [x y] -> (distance x y)
  _ -> 0)
```

#### 🚧 将来検討

**`and` 条件** - 複雑な論理式（必要性を見極め中）
```lisp
(match x
  (> 0 and < 100) -> "in range"
  _ -> "out of range")
```

#### matchの設計哲学

1. **データの流れを分岐させる**: matchは単なるif-elseではなく、データ構造を分解して流れを作る
2. **変換を埋め込む**: `=> 変換` でmatch内部でパイプラインを実現
3. **読みやすさ優先**: パターンが上から下に読める、条件が一目で分かる
4. **段階的開示**: 基本パターンから始めて、必要に応じて拡張機能を使う

### ✅ `try` - エラー処理
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

### ✅ `defer` - 遅延実行
```lisp
;; スコープ終了時に実行
(defn process-file [path]
  (let [f (open path)]
    (do
      (defer (close f))  ;; 関数終了時に必ず実行
      (read f))))

;; 複数のdefer（LIFO: 後入れ先出し）
(do
  (defer (log "3"))
  (defer (log "2"))
  (defer (log "1"))
  (work))
;; 実行順: work → "1" → "2" → "3"

;; エラー時も実行される
(defn safe-process []
  (do
    (defer (cleanup))
    (try (risky-op))))
```

## 4. 演算子

### ✅ `|>` - パイプライン
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

## 5. データ構造

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

;; ✅ マップへのアクセス（実装済み）
(get {:a 1} :a)           ;; 1
(:name {:name "Alice"})   ;; "Alice" (キーワードは関数として使える)
(:age {:name "Bob" :age 30})  ;; 30

;; エラーケース
(:notexist {:name "Alice"})  ;; エラー: キーが見つかりません
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

## 6. コア関数

Qiの組み込み関数は**Flow-oriented**哲学に基づき、データの流れと変換を重視した設計になっています。

### モジュール構成

Qiは**2層モジュール設計**を採用しています：

**Core（90個）** - グローバル名前空間、自動インポート
- 特殊形式・演算子（11個）: `def`, `fn`, `let`, `do`, `if`, `match`, `try`, `defer`, `|>`, `||>`, `|>?`
- リスト操作（29個）: `first`, `rest`, `last`, `nth`, `take`, `drop`, `map`, `filter`, `reduce`, `pmap`, `tap`, `find`, `every`, `some`, etc.
- マップ操作（9個）: `get`, `keys`, `vals`, `assoc`, `dissoc`, `merge`, `get-in`, `update-in`, `update`
- 数値・比較（17個）: `+`, `-`, `*`, `/`, `%`, `inc`, `dec`, `abs`, `min`, `max`, `sum`, `=`, `!=`, `<`, `>`, `<=`, `>=`
- 文字列（3個）: `str`, `split`, `join`
- 述語・型判定（22個）: `nil?`, `list?`, `vector?`, `map?`, `string?`, `integer?`, `float?`, `number?`, etc.
- 並行処理（5個）: `go`, `chan`, `send!`, `recv!`, `close!`
- 論理・I/O（4個）: `not`, `print`, `println`, `error` (※ `and`, `or`は特殊形式)
- 状態管理（4個）: `atom`, `deref`, `swap!`, `reset!`
- メタプログラミング（4個）: `eval`, `uvar`, `variable`, `macro?`
- 型変換（3個）: `to-int`, `to-float`, `to-string`
- 日時（3個）: `now`, `timestamp`, `sleep`
- デバッグ（1個）: `time` (dbg/time)

**専門モジュール** - 明示的インポートまたは `module/function` 形式で使用
- **list**: 高度なリスト操作（18個）- `list/frequencies`, `list/sort-by`, `list/group-by`, etc.
- **map**: 高度なマップ操作（5個）- `map/select-keys`, `map/update-keys`, etc.
- **fn**: 高階関数（3個）- `fn/complement`, `fn/juxt`, `fn/tap>`
- **set**: 集合演算（7個）- `set/union`, `set/intersect`, `set/difference`, etc.
- **math**: 数学関数（10個）- `math/pow`, `math/sqrt`, `math/round`, etc.
- **io**: ファイルI/O（19個）- `io/read-file`, `io/write-file`, `io/list-dir`, `io/temp-file`, etc.
- **path**: パス操作（9個）- `path/join`, `path/basename`, `path/dirname`, etc.
- **env**: 環境変数（4個）- `env/get`, `env/set`, `env/load-dotenv`, etc.
- **log**: 構造化ログ（6個）- `log/info`, `log/warn`, `log/error`, `log/set-level`, etc.
- **dbg**: デバッグ（2個）- `dbg/inspect`, `dbg/time`
- **async**: 並行処理（高度）（16個）- `async/await`, `async/then`, `async/pfilter`, etc.
- **pipeline**: パイプライン処理（5個）- `pipeline/pipeline`, `pipeline/map`, etc.
- **stream**: ストリーム処理（11個）- `stream/stream`, `stream/map`, etc.
- **str**: 文字列操作（62個）- `str/upper`, `str/lower`, `str/snake`, etc.
- **json**: JSON処理（3個）- `json/parse`, `json/stringify`, `json/pretty`
- **http**: HTTPクライアント（11個）- `http/get`, `http/post`, `http/get-stream`, etc.
- **server**: HTTPサーバー（16個）- `server/serve`, `server/router`, `server/ok`, `server/json`, ミドルウェア、静的ファイル配信など
- **csv**: CSV処理（5個）- `csv/parse`, `csv/stringify`, `csv/read-file`, etc.
- **zip**: ZIP圧縮・解凍（6個）- `zip/create`, `zip/extract`, `zip/list`, `zip/gzip`, etc.
- **args**: コマンドライン引数パース（4個）- `args/all`, `args/get`, `args/parse`, `args/count`
- **db**: データベース（11個）- `db/connect`, `db/query`, `db/exec`, `db/begin`, `db/commit`, etc.

**使用例**:
```lisp
;; Core関数はそのまま使える
(data |> filter valid? |> map transform |> sort)

;; 専門モジュール関数は module/function 形式
(io/read-file "data.txt")
(math/pow 2 8)
(list/frequencies [1 2 2 3])

;; useで短縮可能
(use io :only [read-file])
(read-file "data.txt")
```

### リスト操作

#### 基本操作（✅ 実装済み）
```lisp
;; アクセス
first rest last         ;; 最初、残り、最後
nth                     ;; n番目の要素取得
take drop               ;; 部分取得
len count empty?        ;; 長さ、空チェック（count は len のエイリアス）

;; 追加・結合
cons conj               ;; 要素追加
concat                  ;; リスト連結
flatten                 ;; 平坦化（全階層）

;; 生成・変換
range                   ;; (range 10) => (0 1 2 ... 9)
reverse                 ;; 反転
```

#### 高階関数（✅ 実装済み）
```lisp
map filter reduce       ;; 基本の高階関数
pmap                    ;; 並列map（現在はシングルスレッド実装）
tap                     ;; 副作用タップ（値を返しつつ副作用実行）
```

**tapの使用例**:
```lisp
;; パイプライン内でのデバッグ
([1 2 3]
 |> (map inc)
 |> (tap println)       ;; (2 3 4)を出力して、そのまま次に渡す
 |> sum)                ;; => 9

;; データの流れを観察
(def data {:name "Alice" :age 30})
(data
 |> (tap println)       ;; Map({"name": String("Alice"), "age": Integer(30)})
 |> keys)               ;; => (:name :age)
```

#### コレクション検索・述語（✅ 実装済み）
```lisp
;; ✅ Phase 4.5で実装
find                    ;; 条件を満たす最初の要素: (find (fn [x] (> x 5)) [1 7 3]) => 7
find-index              ;; 条件を満たす最初のインデックス: (find-index (fn [x] (> x 5)) [1 7 3]) => 1
every?                  ;; 全要素が条件を満たすか: (every? (fn [x] (> x 0)) [1 2 3]) => true
some?                   ;; いずれかが条件を満たすか: (some? (fn [x] (> x 5)) [1 7 3]) => true
```

**使用例**:
```lisp
;; ユーザーを探す
(def users [{:name "Alice" :age 30} {:name "Bob" :age 25}])
(find (fn [u] (= (get u :name) "Bob")) users)  ;; {:name "Bob" :age 25}

;; 全員成人か確認
(every? (fn [u] (>= (get u :age) 20)) users)  ;; true

;; データパイプラインでの活用
(users
 |> (filter (fn [u] (>= (get u :age) 25)))
 |> (find (fn [u] (= (get u :name) "Alice"))))
```

#### ソート・集約（✅ 実装済み）
```lisp
sort                    ;; ソート（整数・浮動小数点・文字列対応）
sort-by                 ;; キー指定ソート: (sort-by :age users)
distinct                ;; 重複排除
partition               ;; 述語で2分割: (partition even? [1 2 3 4]) => [(2 4) (1 3)]
group-by                ;; キー関数でグループ化
frequencies             ;; 出現頻度: [1 2 2 3] => {1: 1, 2: 2, 3: 1}
count-by                ;; 述語でカウント: (count-by even? [1 2 3 4]) => {true: 2, false: 2}
```

**使用例**:
```lisp
;; ソート
(sort [3 1 4 1 5])  ;; (1 1 3 4 5)
(sort ["zebra" "apple" "banana"])  ;; ("apple" "banana" "zebra")

;; 重複排除してソート
([5 2 8 2 9 1 3 8 4]
 |> distinct
 |> sort)  ;; (1 2 3 4 5 8 9)

;; グループ化
(group-by (fn [n] (% n 3)) [1 2 3 4 5 6 7 8 9])
;; {0: (3 6 9), 1: (1 4 7), 2: (2 5 8)}
```

#### 集約・分析（✅ 全て実装済み）
```lisp
;; ✅ 実装済み
sort-by                 ;; キー指定ソート: (sort-by :age users)
frequencies             ;; 出現頻度: [1 2 2 3] => {1: 1, 2: 2, 3: 1}
count-by                ;; 述語でカウント: (count-by even? [1 2 3 4]) => {true: 2, false: 2}
max-by min-by           ;; 条件に基づく最大/最小
sum-by                  ;; キー関数で合計
```

**設計メモ**: `frequencies`と`count-by`はデータ分析でよく使う。`group-by`と組み合わせると強力。

#### 集合演算（✅ 実装済み）
```lisp
;; set/モジュールで実装済み
set/union                   ;; 和集合: (set/union [1 2] [2 3]) => [1 2 3]
set/intersect               ;; 積集合: (set/intersect [1 2 3] [2 3 4]) => [2 3]
set/difference              ;; 差集合: (set/difference [1 2 3] [2]) => [1 3]
set/symmetric-difference    ;; 対称差: (set/symmetric-difference [1 2 3] [2 3 4]) => [1 4]
set/subset?                 ;; 部分集合判定
set/superset?               ;; 上位集合判定
set/disjoint?               ;; 互いに素判定
```

**Flow哲学との関係**: 集合演算はデータフィルタリングで頻出。パイプラインと相性が良い。

#### チャンク・分割（✅ 実装済み）
```lisp
;; ✅ 実装済み
chunk                   ;; 固定サイズで分割: (chunk 3 [1 2 3 4 5]) => ([1 2 3] [4 5])
take-while drop-while   ;; 述語が真の間取得/削除

;; 🔜 優先度: 中
partition-all           ;; partitionの全要素版
```

### 数値・比較

#### 算術演算（✅ 実装済み）
```lisp
+ - * / %               ;; 基本演算
inc dec                 ;; インクリメント/デクリメント
sum                     ;; 合計
abs                     ;; 絶対値
min max                 ;; 最小/最大
```

#### 比較（✅ 実装済み）
```lisp
= != < > <= >=          ;; 比較演算子
```

#### 数学関数（✅ 実装済み）
```lisp
;; ✅ 実装済み（coreに含まれる）
pow                     ;; べき乗: (pow 2 8) => 256
sqrt                    ;; 平方根: (sqrt 16) => 4
round floor ceil        ;; 丸め: (round 3.7) => 4
clamp                   ;; 範囲制限: (clamp 1 10 15) => 10
rand                    ;; 0.0以上1.0未満の乱数
rand-int                ;; 0以上n未満の整数乱数

;; 🔜 優先度: 中（mathモジュールでもOK）
sin cos tan             ;; 三角関数
log exp                 ;; 対数・指数
```

**設計方針**: `pow`/`sqrt`/`round`/`clamp`/`rand`はcoreに。三角関数などは将来`math`モジュールへ。

#### 統計（✅ 実装済み）
```lisp
;; stats/モジュールで実装済み
stats/mean              ;; 平均
stats/median            ;; 中央値
stats/mode              ;; 最頻値
stats/stddev            ;; 標準偏差
stats/variance          ;; 分散
stats/percentile        ;; パーセンタイル
```

### 論理（✅ 全て実装済み）
```lisp
and or not
```

### マップ操作

#### 基本操作（✅ 実装済み）
```lisp
get keys vals           ;; アクセス
assoc dissoc            ;; キーの追加・削除
merge                   ;; マージ: (merge {:a 1} {:b 2}) => {:a 1 :b 2}
select-keys             ;; キー選択: (select-keys {:a 1 :b 2 :c 3} [:a :c]) => {:a 1 :c 3}
```

#### ネスト操作（✅ 実装済み）⭐ **Flow哲学の核心**
```lisp
;; ✅ 実装済み（JSON/Web処理で必須）
update                  ;; 値を関数で更新
update-in               ;; ネスト更新: (update-in m [:user :age] inc)
get-in                  ;; ネスト取得: (get-in m [:user :name] "default")
assoc-in                ;; ネスト追加
dissoc-in               ;; ネスト削除
```

#### マップ一括変換（✅ 実装済み）
```lisp
;; ✅ Phase 4.5で実装
update-keys             ;; 全キーに関数適用: (update-keys (fn [k] (str k "!")) {:a 1}) => {"a!" 1}
update-vals             ;; 全値に関数適用: (update-vals (fn [v] (* v 2)) {:a 1 :b 2}) => {:a 2 :b 4}
zipmap                  ;; キーと値のリストからマップ生成: (zipmap [:a :b] [1 2]) => {:a 1 :b 2}
```

**使用例**:
```lisp
;; すべてのキーを大文字に
(update-keys upper {:name "Alice" :age 30})  ;; {"NAME" "Alice" "AGE" 30}

;; すべての値を2倍に
(def prices {:apple 100 :banana 50})
(update-vals (fn [p] (* p 2)) prices)  ;; {:apple 200 :banana 100}

;; データ変換パイプライン
(prices
 |> (update-vals (fn [p] (* p 1.1)))  ;; 10%値上げ
 |> (update-vals round))              ;; 丸める
```

**ネスト操作の使用例**:
```lisp
;; update: 値を関数で変換
(def user {:name "Alice" :age 30})
(update user :age inc)  ;; {:name "Alice" :age 31}

;; update-in: ネスト構造の更新（Web/JSON処理で超頻出）
(def state {:user {:profile {:visits 10}}})
(update-in state [:user :profile :visits] inc)
;; {:user {:profile {:visits 11}}}

;; get-in: ネストアクセス（デフォルト値付き）
(get-in {:user {:name "Bob"}} [:user :name] "guest")  ;; "Bob"
(get-in {} [:user :name] "guest")  ;; "guest"

;; パイプラインで威力発揮
(state
 |> (fn [s] (update-in s [:user :profile :visits] inc))
 |> (fn [s] (assoc-in s [:user :last-seen] (now))))
```

**設計メモ**: ネスト操作はQiの強み。JSONやWeb APIレスポンスの処理が直感的になる。一括変換関数と組み合わせることでデータ変換が簡潔に書ける。

### 関数型プログラミング基礎

#### 基本ツール（✅ 全て実装済み）
```lisp
;; ✅ 実装済み（関数型の必須ツール）
identity                ;; 引数をそのまま返す: (identity 42) => 42
constantly              ;; 常に同じ値を返す関数: ((constantly 42) x) => 42
comp                    ;; 関数合成: ((comp f g) x) => (f (g x))
partial                 ;; 部分適用: (def add5 (partial + 5))
apply                   ;; リストを引数として適用: (apply + [1 2 3]) => 6
complement              ;; 述語の否定: ((complement even?) 3) => true
juxt                    ;; 複数関数を並列適用: ((juxt inc dec) 5) => [6 4]
```

**使用例**:
```lisp
;; identity: フィルタや変換のデフォルト
(filter identity [1 false nil 2 3])  ;; (1 2 3)

;; comp: パイプラインの代替（右から左）
(def process (comp upper trim))
(process "  hello  ")  ;; "HELLO"

;; constantly: デフォルト値生成
(def get-or-default (fn [m k] (get m k (constantly "N/A"))))
```

**設計メモ**: `identity`/`comp`/`apply`は関数型の基礎。パイプライン（`|>`）と`comp`は補完関係。

### 文字列操作

#### Core関数（✅ 実装済み）
```lisp
str                     ;; 連結
split join              ;; 分割・結合
upper lower trim        ;; 変換
len empty?              ;; 長さ、空チェック
map-lines               ;; 各行に関数適用
```

#### 拡張機能（🔜 strモジュールで提供予定）
SPEC.mdの「標準ライブラリ > str」セクション参照。60以上の文字列関数を提供予定。

### 述語関数（✅ 全て実装済み）
```lisp
;; 型判定
nil? list? vector? map? string? keyword?
integer? float? number? fn?
coll?           ;; コレクション型か（list/vector/map）
sequential?     ;; シーケンシャル型か（list/vector）

;; 状態チェック
empty?
some?           ;; nilでないか

;; 論理値判定
true?           ;; 厳密にtrueか
false?          ;; 厳密にfalseか

;; 数値判定
even? odd?
positive? negative? zero?
```

### IO・ファイル操作

#### 基本I/O（✅ 実装済み）
```lisp
print                   ;; 標準出力
println                 ;; 改行付き出力
read-file               ;; ファイル読み込み
read-lines              ;; 行ごとに読み込み（メモリ効率）
write-file              ;; ファイル書き込み（上書き）
append-file             ;; ファイル追記
file-exists?            ;; ファイル存在確認
```

**使用例**:
```lisp
;; ファイル読み書き
(write-file "/tmp/test.txt" "Hello, Qi!")
(def content (read-file "/tmp/test.txt"))
(print content)  ;; "Hello, Qi!"

;; 追記
(append-file "/tmp/test.txt" "\nSecond line")

;; パイプラインで処理
(read-file "data.csv"
 |> split "\n"
 |> (fn [lines] (map parse-line lines))
 |> (fn [data] (filter valid? data)))
```

#### 拡張I/O（全て実装済み）
```lisp
;; ✅ 実装済み（上記の基本I/Oに含まれる）
```

### Web開発・ユーティリティ ⭐ **Phase 4.5新機能**

#### JSON処理（✅ 実装済み）
```lisp
;; ✅ Phase 4.5で実装
json/parse              ;; JSON文字列をパース: "{\"a\":1}" => {:ok {:a 1}}
json/stringify          ;; 値をJSON化（コンパクト）
json/pretty             ;; 値を整形JSON化
```

**使用例**:
```lisp
;; JSONパース
(def json-str "{\"name\":\"Alice\",\"age\":30,\"tags\":[\"dev\",\"lisp\"]}")
(json/parse json-str)
;; => {:ok {"name" "Alice" "age" 30 "tags" ["dev" "lisp"]}}

;; JSON生成
(def data {"name" "Bob" "age" 25})
(json/stringify data)  ;; => {:ok "{\"name\":\"Bob\",\"age\":25}"}
(json/pretty data)     ;; => {:ok "{\n  \"name\": \"Bob\",\n  ..."}

;; データパイプライン
(data
 |> (assoc _ "active" true)
 |> json/pretty
 |>? (fn [json] {:ok (write-file "output.json" json)}))
```

#### HTTP クライアント（✅ 実装済み）
```lisp
;; ✅ Phase 4.5で完全実装
http/get                ;; HTTP GET: (http/get "https://...") => {:ok {:status 200 :body "..."}}
http/post               ;; HTTP POST: (http/post "url" {:key "value"})
http/put                ;; HTTP PUT
http/delete             ;; HTTP DELETE
http/patch              ;; HTTP PATCH
http/head               ;; HTTP HEAD
http/options            ;; HTTP OPTIONS
http/request            ;; 詳細設定: (http/request {:method "GET" :url "..." :headers {...}})

;; 非同期版
http/get-async          ;; 非同期GET: Channelを返す
http/post-async         ;; 非同期POST: Channelを返す
```

**使用例**:
```lisp
;; 基本的なGET
(http/get "https://httpbin.org/get")
;; => {:ok {:status 200 :headers {...} :body "..."}}

;; POSTでJSON送信
(def user {:name "Alice" :email "alice@example.com"})
(http/post "https://api.example.com/users" user)

;; カスタムヘッダ付きリクエスト
(http/request {
  :method "POST"
  :url "https://api.example.com/data"
  :headers {"Authorization" "Bearer token123"}
  :body {:data "value"}
  :timeout 5000
})

;; Railway Pipelineと組み合わせ
("https://api.github.com/users/octocat"
 |> http/get
 |>? (fn [resp] {:ok (get resp "body")})
 |>? json/parse
 |>? (fn [data] {:ok (get data "name")}))
;; => {:ok "The Octocat"}

;; 非同期リクエスト
(def ch (http/get-async "https://api.example.com/data"))
(def resp (recv! ch))  ;; ブロッキング受信
```

**エラーハンドリング**:
```lisp
;; エラー時は {:error {...}} を返す
(http/get "https://invalid-domain-12345.com")
;; => {:error {:type "connection" :message "..."}}

;; Railway Pipelineで自動的にエラー伝播
("https://invalid.com/api"
 |> http/get
 |>? (fn [resp] {:ok (get resp "body")})  ;; 実行されない
 |>? json/parse)                          ;; 実行されない
;; => {:error {...}}
```

**HTTPコンテンツ圧縮**:

HTTPクライアント・サーバー共に、gzip/deflate/brotli圧縮をサポートしています。

**クライアント側**:
```lisp
;; 自動解凍（デフォルトで有効）
;; サーバーが gzip/deflate/brotli で圧縮したレスポンスを自動的に解凍
(http/get "https://example.com/api")  ;; 圧縮されたレスポンスも自動解凍

;; 送信時の圧縮（Content-Encodingヘッダーで指定）
(http/post "https://example.com/api"
  {:data "large payload"}
  {:headers {"content-encoding" "gzip"}})  ;; ボディを自動的にgzip圧縮して送信
```

**サーバー側**:
```lisp
;; リクエストボディの自動解凍
;; クライアントが Content-Encoding: gzip で送信した場合、自動的に解凍
(def handler
  (fn [req]
    (let [body (get req "body")]  ;; 既に解凍済み
      (server/ok body))))

;; レスポンス圧縮はserver/with-compressionミドルウェアで実現（後述）
```

**HTTP認証**:

HTTP Basic AuthとBearer Token認証をサポートしています。

**クライアント側**:
```lisp
;; Basic Auth
(http/request {
  "url" "https://api.example.com/data"
  "basic-auth" ["username" "password"]})  ;; 自動的にAuthorizationヘッダーを設定

;; Bearer Token
(http/request {
  "url" "https://api.example.com/data"
  "bearer-token" "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."})  ;; Authorization: Bearer ...
```

**サーバー側**（後述）:
- `server/with-basic-auth`: Basic認証ミドルウェア
- `server/with-bearer`: Bearer Token抽出ミドルウェア

#### HTTP サーバー（✅ 実装済み - Phase 5）

**Flow-Oriented な Web アプリケーション構築**

Qiの哲学（Flow-Oriented Programming）に沿った、ハンドラーはパイプライン、ルーティングはデータという設計です。

**基本関数**:
```lisp
;; レスポンスヘルパー
server/ok                 ;; 200 OKレスポンス: (server/ok "Hello!")
server/json               ;; JSONレスポンス: (server/json {:message "hello"})
server/not-found          ;; 404レスポンス: (server/not-found "Not Found")
server/no-content         ;; 204 No Contentレスポンス

;; ルーティング & サーバー
server/router             ;; ルーター作成: (server/router routes)
server/serve              ;; サーバー起動: (server/serve app {:port 3000})

;; ミドルウェア
server/with-logging       ;; ロギングミドルウェア
server/with-cors          ;; CORSミドルウェア
server/with-json-body     ;; JSONボディ自動パースミドルウェア
server/with-compression   ;; レスポンス圧縮ミドルウェア（gzip）
server/with-basic-auth    ;; Basic認証ミドルウェア
server/with-bearer        ;; Bearer Token抽出ミドルウェア

;; 静的ファイル配信
server/static-file        ;; 単一ファイル配信: (server/static-file "path/to/file")
server/static-dir         ;; ディレクトリ配信: (server/static-dir "public")
```

**使用例 - シンプルなサーバー**:
```lisp
;; ハンドラー（リクエスト -> レスポンス）
(def hello-handler
  (fn [req]
    (server/ok "Hello, World!")))

;; ルート定義（データ駆動）
(def routes
  [["/" (assoc {} "get" hello-handler)]])

;; ルーターを作成
(def app (server/router routes))

;; サーバー起動
(server/serve app {"port" 3000})
;; => HTTP server started on http://127.0.0.1:3000
```

**使用例 - JSON API with パスパラメータ**:
```lisp
;; ハンドラーはパイプラインで構成
(def list-users
  (fn [req]
    (server/json {"users" [{"id" 1 "name" "Alice"}
                           {"id" 2 "name" "Bob"}]})))

;; パスパラメータを使う（✅ 実装済み）
(def get-user
  (fn [req]
    (let [params (get req "params")
          user-id (get params "id")]
      (server/json {"id" user-id "name" "Alice"}))))

;; 複数のパスパラメータ
(def get-post
  (fn [req]
    (let [params (get req "params")
          user-id (get params "user_id")
          post-id (get params "post_id")]
      (server/json {"user_id" user-id "post_id" post-id}))))

(def create-user
  (fn [req]
    ;; リクエストボディは req の "body" キーから取得
    (server/json {"status" "created"} {"status" 201})))

;; ルート定義 - データ構造なので検査・変換可能
;; ✅ パスパラメータ: /users/:id 形式をサポート
(def routes
  [["/api/users" (assoc {} "get" list-users "post" create-user)]
   ["/api/users/:id" (assoc {} "get" get-user)]
   ["/api/users/:user_id/posts/:post_id" (assoc {} "get" get-post)]])

;; アプリ起動（タイムアウト設定可能）
(def app (server/router routes))
(server/serve app {"port" 8080 "host" "0.0.0.0" "timeout" 30})
;; => HTTP server started on http://0.0.0.0:8080 (timeout: 30s)
```

**リクエストオブジェクト**:
```lisp
;; リクエストは以下の構造のマップ:
{:method "get"                    ;; HTTPメソッド（小文字）
 :path "/api/users/123"           ;; リクエストパス
 :query "page=1&limit=10"         ;; クエリ文字列（生）
 :query-params {"page" "1"        ;; ✅ クエリパラメータ（自動パース）
                "limit" "10"}
 :headers {"content-type" "application/json" ...}
 :body "..."                      ;; リクエストボディ（文字列）
 :params {"id" "123"}}            ;; ✅ パスパラメータ（マッチした場合のみ）
```

**レスポンスオブジェクト**:
```lisp
;; ハンドラーは以下の構造のマップを返す:
{:status 200                ;; HTTPステータスコード
 :headers {"Content-Type" "text/plain; charset=utf-8" ...}
 :body "Hello, World!"}     ;; レスポンスボディ
```

**実装済み機能**:
- ✅ **データ駆動**: ルーティングは検査・変換可能なデータ構造
- ✅ **パイプライン**: ハンドラーは `|>` で流れが明確
- ✅ **合成可能**: すべてが関数で、ミドルウェアも関数
- ✅ **スレッドセーフ**: 並列リクエスト処理に対応
- ✅ **Flow-Oriented**: Qiの哲学を体現
- ✅ **パスパラメータ**: `/users/:id` 形式をサポート（複数パラメータ対応）
- ✅ **クエリパラメータ**: `?page=1&limit=10` を自動パース、配列対応、URLデコード
- ✅ **タイムアウト**: リクエストタイムアウトを設定可能（デフォルト30秒）
- ✅ **ミドルウェア**: ロギング、CORS、JSONボディパース（複数重ね可能）
- ✅ **静的ファイル配信**: HTML、CSS、JS、画像、フォントなどのバイナリファイル対応

**メモリ管理**:
- リクエストごとにクリーンな環境を使用
- Arc による自動メモリ管理
- 長時間実行サービスに適した設計（SPEC.md「14. メモリ管理」参照）

**設定オプション**:
```lisp
(server/serve app {
  "port" 3000           ;; ポート番号（デフォルト: 3000）
  "host" "0.0.0.0"      ;; ホスト（デフォルト: "127.0.0.1"）
  "timeout" 30})        ;; タイムアウト秒数（デフォルト: 30）
```

**ミドルウェアシステム**（✅ 実装済み）:

Qiのミドルウェアは**ハンドラーをラップして機能を追加する高階関数**です。複数のミドルウェアを重ねることで、横断的な関心事を分離できます。

```lisp
;; ミドルウェア関数
server/with-logging       ;; リクエスト/レスポンスをログ出力
server/with-cors          ;; CORSヘッダーを追加
server/with-json-body     ;; リクエストボディを自動的にJSONパース
server/with-compression   ;; レスポンスボディをgzip圧縮
server/with-basic-auth    ;; Basic認証（ユーザー名・パスワード検証）
server/with-bearer        ;; Bearer Token抽出（検証はユーザーコード）
server/with-no-cache      ;; キャッシュ無効化ヘッダーを追加
server/with-cache-control ;; カスタムCache-Controlヘッダーを追加
```

**使用例 - ミドルウェアの基本**:
```lisp
;; 1. ロギングミドルウェア
(def logging-handler
  (server/with-logging
    (fn [req]
      (server/ok "Hello!"))))

;; リクエスト時: [HTTP] GET /logging
;; レスポンス時: [HTTP] -> 200

;; 2. CORSミドルウェア
(def cors-handler
  (server/with-cors
    (fn [req]
      (server/json {"message" "CORS enabled"}))))

;; レスポンスに自動的にCORSヘッダーが追加される:
;; Access-Control-Allow-Origin: *
;; Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
;; Access-Control-Allow-Headers: Content-Type, Authorization

;; 3. JSONボディパースミドルウェア
(def json-handler
  (server/with-json-body
    (fn [req]
      (let [json-data (get req "json")]
        (server/json {"received" json-data})))))

;; リクエストボディが自動的にパースされ、req["json"]に格納される
;; curl -X POST ... -d '{"name":"alice"}'
;; => req["json"] = {"name" "alice"}

;; 4. レスポンス圧縮ミドルウェア
(def compressed-handler
  (server/with-compression
    (fn [req]
      (server/ok "Large response body that will be compressed..."))))

;; レスポンスが1KB以上の場合、自動的にgzip圧縮される
;; レスポンスヘッダー: Content-Encoding: gzip

;; 5. Basic認証ミドルウェア
(def protected-handler
  (server/with-basic-auth
    (fn [req]
      (server/ok "Protected content"))
    {"users" {"admin" "secret123" "user" "pass456"}}))

;; 認証が必要なエンドポイント
;; 正しいユーザー名/パスワードがない場合、401 Unauthorizedを返す
;; curl -u admin:secret123 http://localhost:3000/admin
;; => "Protected content"

;; ルーター全体に認証を適用
(def protected-app
  (server/with-basic-auth
    (server/router routes)
    {"users" {"admin" "secret"}}))

;; 全てのルートが認証必須になる

;; 6. Bearer Token抽出ミドルウェア
(def api-handler
  (server/with-bearer
    (fn [req]
      (let [token (get req "bearer-token")]
        (if (= token "valid-token-12345")
          (server/json {"status" "authenticated" "data" "..."})
          (server/json {"status" "unauthorized"} {"status" 401}))))))

;; Authorization: Bearer valid-token-12345 ヘッダーからトークンを抽出
;; req["bearer-token"] にトークン文字列が格納される
;; 検証はユーザーコードで実行（JWTライブラリ等を使用可能）
```

**使用例 - ミドルウェアの組み合わせ**:
```lisp
;; 複数のミドルウェアを重ねて使用（外側から順に適用）
(def api-handler
  (server/with-logging        ;; 最外層: ログを出力
    (server/with-cors         ;; 中層: CORSヘッダーを追加
      (server/with-json-body  ;; 最内層: JSONを自動パース
        (fn [req]
          (let [data (get req "json")
                user (get data "user")]
            (server/json {"message" (str "Hello, " user "!")})))))))

(def routes
  [["/api" (assoc {} "post" api-handler)]])

(def app (server/router routes))
(server/serve app {"port" 3000})

;; curl -X POST http://localhost:3000/api -d '{"user":"alice"}'
;; [HTTP] POST /api              <- ログ出力
;; [HTTP] -> 200                 <- ログ出力
;; {"message":"Hello, alice!"}   <- JSON + CORSヘッダー付きレスポンス
```

**ミドルウェアのカスタマイズ**:
```lisp
;; CORSのオリジンを指定
(def cors-handler
  (server/with-cors
    (fn [req] (server/json {"data" "..."}))
    {"origins" ["https://example.com" "https://api.example.com"]}))

;; 圧縮の最小サイズを指定（デフォルト: 1024バイト = 1KB）
(def custom-compressed-handler
  (server/with-compression
    (fn [req] (server/ok "Response body..."))
    {"min-size" 512}))  ;; 512バイト以上で圧縮

;; 認証とJSON処理を組み合わせ
(def secure-api-handler
  (server/with-basic-auth
    (server/with-json-body
      (fn [req]
        (let [data (get req "json")]
          (server/json {"received" data "authenticated" true}))))
    {"users" {"api-user" "api-pass"}}))

;; Bearer認証でAPI保護
(def bearer-api-handler
  (server/with-bearer
    (server/with-json-body
      (fn [req]
        (let [token (get req "bearer-token")
              data (get req "json")]
          (if (= token "secret-api-token")
            (server/json {"data" data "status" "ok"})
            (server/json {"error" "invalid token"} {"status" 401})))))))

;; キャッシュ無効化（APIレスポンス等）
(def no-cache-handler
  (server/with-no-cache
    (fn [req] (server/json {"data" "dynamic content" "timestamp" (now)}))))
;; レスポンスヘッダー:
;; Cache-Control: no-store, no-cache, must-revalidate, private
;; Pragma: no-cache
;; Expires: 0

;; カスタムキャッシュ制御（静的コンテンツ等）
(def cached-handler
  (server/with-cache-control
    (fn [req] (server/ok "Static content"))
    {"max-age" 3600 "public" true}))
;; レスポンスヘッダー: Cache-Control: max-age=3600, public

;; 詳細なキャッシュ制御
(def immutable-handler
  (server/with-cache-control
    (fn [req] (server/ok "Immutable content with hash in filename"))
    {"max-age" 31536000 "public" true "immutable" true}))
;; 1年間キャッシュ + immutable（バージョニングされたアセット用）
;; Cache-Control: max-age=31536000, public, immutable
```

**クエリパラメータの使用例**:
```lisp
;; 基本的なクエリパラメータ
(def search-handler
  (fn [req]
    (let [params (get req "query-params")
          query (get params "q")
          page (get params "page")
          limit (get params "limit")]
      (server/json {"search" query "page" page "limit" limit}))))

;; GET /search?q=lisp&page=1&limit=20
;; => {"search":"lisp","page":"1","limit":"20"}

;; 配列パラメータ（同じキーが複数）
(def filter-handler
  (fn [req]
    (let [params (get req "query-params")
          tags (get params "tags")]  ;; 配列になる
      (server/json {"tags" tags}))))

;; GET /filter?tags=ruby&tags=python&tags=lisp
;; => {"tags":["ruby","python","lisp"]}

;; URLエンコードされた値も自動デコード
;; GET /search?q=Hello%20World
;; => {"search":"Hello World"}
```

**静的ファイル配信**（✅ 実装済み）:

Qiは静的ファイル（HTML、CSS、JavaScript、画像、フォントなど）の配信をネイティブサポートします。バイナリファイルも正しく処理されます。

```lisp
;; ミドルウェア関数
server/static-file        ;; 単一ファイルを配信するレスポンスを生成
server/static-dir         ;; ディレクトリから静的ファイルを配信するハンドラーを生成
```

**使用例 - ディレクトリ配信**:
```lisp
;; publicディレクトリ配下の静的ファイルを配信
(def routes
  [["/" (assoc {} "get" (server/static-dir "./public"))]])

(def app (server/router routes))
(server/serve app {"port" 3000})

;; GET /index.html  => ./public/index.html を配信
;; GET /style.css   => ./public/style.css を配信
;; GET /logo.png    => ./public/logo.png を配信（バイナリ）
;; GET /            => ./public/index.html を自動配信
```

**使用例 - 単一ファイル配信**:
```lisp
;; 特定のファイルを直接配信
(def favicon-handler
  (fn [req]
    (server/static-file "./public/favicon.ico")))

(def routes
  [["/favicon.ico" (assoc {} "get" favicon-handler)]])
```

**使用例 - APIと静的ファイルの組み合わせ**:
```lisp
;; APIエンドポイントと静的ファイルを同時に提供
(def routes
  [["/api/users" (assoc {} "get" list-users "post" create-user)]
   ["/api/users/:id" (assoc {} "get" get-user)]
   ["/" (assoc {} "get" (server/static-dir "./public"))]])  ;; 静的ファイル

(def app (server/router routes))
(server/serve app {"port" 8080})

;; GET /api/users         => APIレスポンス（JSON）
;; GET /api/users/123     => APIレスポンス（JSON）
;; GET /index.html        => 静的ファイル（HTML）
;; GET /app.js            => 静的ファイル（JavaScript）
;; GET /logo.png          => 静的ファイル（PNG画像）
```

**機能**:
- ✅ **Content-Type自動判定**: 拡張子から適切なMIMEタイプを設定（HTML、CSS、JS、画像、フォント、PDF等）
- ✅ **バイナリファイル対応**: 画像、フォント、PDFなどをデータ損失なく配信
- ✅ **index.html自動配信**: ディレクトリへのリクエストで自動的にindex.htmlを配信
- ✅ **セキュリティ**: パストラバーサル攻撃（`..`を含むパス）を自動検出・拒否
- ✅ **プレフィックスマッチング**: `/` に配置した静的ハンドラーはすべてのサブパスにマッチ

**対応ファイル形式**（Content-Type自動判定）:
- テキスト: `.html`, `.css`, `.js`, `.json`, `.xml`, `.txt`, `.md`
- 画像: `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg`, `.ico`, `.webp`
- フォント: `.woff`, `.woff2`, `.ttf`, `.otf`
- アーカイブ: `.pdf`, `.zip`, `.gz`
- その他: `application/octet-stream`（デフォルト）

**制限事項**:
- ⚠️ タイムアウトは非同期処理に適用されるが、Qiの同期的なブロッキング操作（`sleep`など）には効かない
- ⚠️ クエリパラメータの値は文字列のまま（数値への自動変換は行わない）
- ⚠️ **静的ファイル配信はファイル全体をメモリに読み込む**
  - **最大ファイルサイズ: 10MB**
  - これを超えるファイルはエラーになる
  - リクエストボディも全体をメモリに読み込む
  - **メモリ使用量** = ファイルサイズ × 同時リクエスト数
  - **推奨**: 大きなファイル（動画、大きなPDFなど）は別のCDNやリバースプロキシで配信

**メモリ使用量の例**:
```lisp
;; 小さいファイル（推奨）
GET /logo.png (100KB) × 10同時リクエスト = 1MB メモリ

;; 中規模ファイル（許容範囲）
GET /document.pdf (5MB) × 10同時リクエスト = 50MB メモリ

;; 大きいファイル（制限あり）
GET /large.pdf (10MB) × 10同時リクエスト = 100MB メモリ

;; 大きすぎるファイル（エラー）
GET /video.mp4 (50MB) → エラー: "file too large: 52428800 bytes (max: 10485760 bytes / 10 MB)"
```

**将来の拡張予定** 🚧:
- **ストリーミングレスポンス** 🎯 優先度高
  - 大きなファイル（動画、大きなPDF等）をチャンク単位で配信
  - メモリ効率的な実装（Rust: `tokio::fs::File` + `ReaderStream`）
  - Range requests 対応（部分ダウンロード、動画シーク）
- カスタムミドルウェア作成API
- クエリパラメータの型自動推論（`"42"` → `42`）
- ストリーミングリクエスト（大きなファイルアップロード）
- WebSocket サポート
- セッション管理
- グレースフルシャットダウン

#### デバッグ・計測（✅ 実装済み）
```lisp
;; ✅ Phase 4.5で実装
inspect                 ;; 値を整形表示してそのまま返す（パイプライン用）
time                    ;; 関数実行時間を計測
```

**使用例**:
```lisp
;; inspect: データフローを観察
(def data {"name" "Alice" "scores" [95 87 92]})
(data
 |> (assoc _ "average" 91.3)
 |> inspect              ;; 整形表示してそのまま返す
 |> (update-vals inc))

;; time: パフォーマンス計測
(time (fn []
  (reduce + (range 1000000))))
;; Elapsed: 0.234s
;; => 499999500000

;; パイプライン内で使用
(urls
 ||> http/get
 |> (fn [responses] (time (fn [] (process responses))))
 |> save-results)
```

**設計哲学**:
- JSONとHTTPは常にResult型 `{:ok value}` / `{:error e}` を返す
- Railway Pipeline `|>?` と完璧に統合
- デバッグ関数はパイプライン内で使いやすい設計
- 非同期版はChannelを返し、Layer 1 (go/chan) と統合

### 並行・並列処理 - Qiの真髄

**Qiは並行・並列処理を第一級市民として扱う言語です。**

「並列、並行を簡単にできるのはQiのキモ」- これがQiの設計哲学の核心です。

#### 設計哲学

Qiの並行・並列処理は**3層アーキテクチャ**で構成されます：

```
┌─────────────────────────────────────┐
│  Layer 3: async/await (高レベル)     │  ← 使いやすさ（I/O、API）
│  - async, await, then, catch        │
├─────────────────────────────────────┤
│  Layer 2: Pipeline (中レベル)        │  ← 関数型らしさ
│  - pmap, pipeline, fan-out/in       │
├─────────────────────────────────────┤
│  Layer 1: go/chan (低レベル基盤)     │  ← パワーと柔軟性
│  - go, chan, send!, recv!, close!   │
└─────────────────────────────────────┘
```

**すべてgo/chanの上に構築** - シンプルで一貫性のあるアーキテクチャ。

#### ✅ 全て実装済み

**実装状態**:
- ✅ Evaluatorを完全スレッドセーフ化（Arc<RwLock<_>>）
- ✅ pmapでユーザー定義関数も並列実行可能
- ✅ Atomはスレッドセーフ（RwLock使用）
- ✅ Layer 1: go/chan完全実装
- ✅ Layer 2: Pipeline完全実装
- ✅ Layer 3: async/await完全実装

**Layer 1: go/chan（基盤）** - Go風の並行処理 ✅
```lisp
;; チャネル作成 ✅
(chan)                  ;; 無制限バッファ
(chan 10)               ;; バッファサイズ10

;; 送受信 ✅
(send! ch value)        ;; チャネルに送信
(recv! ch)              ;; ブロッキング受信
(recv! ch :timeout 1000) ;; タイムアウト付き受信（ミリ秒） ✅
(try-recv! ch)          ;; 非ブロッキング受信（nilまたは値）
(close! ch)             ;; チャネルクローズ

;; 複数チャネル待ち合わせ ✅
(select!
  [[ch1 (fn [v] (handle-ch1 v))]
   [ch2 (fn [v] (handle-ch2 v))]
   [:timeout 1000 (fn [] (handle-timeout))]])

;; goroutine風 ✅
(go (println "async!"))
(go (send! ch (expensive-calc)))

;; futureとしても使える ✅
(def result (go (expensive-calc)))
(deref result)          ;; 結果待ち

;; Structured Concurrency（構造化並行処理） ✅
(def ctx (make-scope))  ;; スコープ作成
(scope-go ctx (fn []    ;; スコープ内でgoroutine起動
  (loop [i 0]
    (if (cancelled? ctx)
      (println "cancelled")
      (do
        (println i)
        (sleep 100)
        (recur (inc i)))))))
(cancel! ctx)           ;; スコープ内の全goroutineをキャンセル

;; with-scope関数（便利版） ✅
(with-scope (fn [ctx]
  (scope-go ctx task1)
  (scope-go ctx task2)
  ;; スコープ終了時に自動キャンセル
  ))
```

**Layer 2: Pipeline（構造化並行処理）** - 関数型スタイル ✅
```lisp
;; 並列コレクション操作 ✅
pmap                    ;; 並列map（rayon使用）
pfilter                 ;; 並列filter ✅
preduce                 ;; 並列reduce ✅
parallel-do             ;; 複数式の並列実行 ✅

;; パイプライン処理 ✅
(pipeline n xf ch)      ;; n並列でxf変換をchに適用

;; ファンアウト/ファンイン ✅
(fan-out ch n)          ;; 1つのチャネルをn個に分岐
(fan-in chs)            ;; 複数チャネルを1つに合流

;; データパイプライン ✅
(-> data
    (pipeline-map 4 transform)     ;; 4並列で変換
    (pipeline-filter 2 predicate)  ;; 2並列でフィルタ
    (into []))
```

**Layer 3: async/await（高レベル）** - モダンな非同期処理 ✅
```lisp
;; 基本的なawait
(def p (go (fn [] (+ 1 2 3))))
(await p)  ;; => 6

;; Promise チェーン
(-> (go (fn [] 10))
    (then (fn [x] (* x 2)))
    (then (fn [x] (+ x 1)))
    (await))  ;; => 21

;; Promise.all風
(def promises [(go (fn [] 1)) (go (fn [] 2)) (go (fn [] 3))])
(await (all promises))  ;; => [1 2 3]

;; Promise.race風
(def promises [(go (fn [] "slow")) (go (fn [] "fast"))])
(await (race promises))  ;; => "fast"

;; エラーハンドリング
(catch promise (fn [e] (println "Error:" e)))
```

**実装済み・実装予定の関数一覧**:

**Layer 1 (go/chan)**:
- ✅ `chan`: チャネル作成
- ✅ `send!`: 送信
- ✅ `recv!`: ブロッキング受信
- ✅ `recv! :timeout`: タイムアウト付き受信
- ✅ `try-recv!`: 非ブロッキング受信
- ✅ `close!`: チャネルクローズ
- ✅ `go`: goroutine起動
- ✅ `select!`: 複数チャネル待ち合わせ
- ✅ `make-scope`: スコープ作成
- ✅ `scope-go`: スコープ内goroutine
- ✅ `cancel!`: スコープキャンセル
- ✅ `cancelled?`: キャンセル確認
- ✅ `with-scope`: スコープ自動管理

**Layer 2 (Pipeline)**:
- ✅ `pmap`: 並列map
- ✅ `pfilter`: 並列filter
- ✅ `preduce`: 並列reduce
- ✅ `parallel-do`: 複数式の並列実行
- ✅ `pipeline`: パイプライン処理
- ✅ `pipeline-map`: パイプラインmap
- ✅ `pipeline-filter`: パイプラインfilter
- ✅ `fan-out`: ファンアウト
- ✅ `fan-in`: ファンイン

**Layer 3 (async/await)**:
- ✅ `await`: Promiseを待機
- ✅ `then`: Promiseチェーン
- ✅ `catch`: エラーハンドリング
- ✅ `all`: 複数Promiseを並列実行
- ✅ `race`: 最速のPromiseを返す

#### 実装技術スタック

- **crossbeam-channel**: Go風チャネル実装（select!マクロも提供）
- **rayon**: データ並列（pmap, pfilter, preduce等）
- **parking_lot**: 高性能RwLock
- **tokio** (将来): async/await実行時

### ✅ 状態管理 - Atom（実装済み）

Qiの状態管理は**Atom**（アトム）を使います。Atomは参照透過性を保ちながら、必要な場所だけで状態を持つための仕組みです。

#### 基本操作

```lisp
;; ✅ 実装済み
atom                    ;; アトム作成
deref                   ;; 値取得
@                       ;; derefの短縮形（@counter => (deref counter)）
swap!                   ;; 関数で更新（アトミック）
reset!                  ;; 値を直接セット
```

#### アトムの作成と参照

```lisp
;; カウンター
(def counter (atom 0))

;; 値を取得
(deref counter)  ;; 0

;; 値を更新
(reset! counter 10)
(deref counter)  ;; 10

;; 関数で更新（現在の値を使う）
(swap! counter inc)
(deref counter)  ;; 11

(swap! counter + 5)
(deref counter)  ;; 16
```

#### 実用例1: カウンター

```lisp
;; リクエストカウンター
(def request-count (atom 0))

(defn handle-request [req]
  (do
    (swap! request-count inc)
    (process req)))

;; 現在のカウント確認
(deref request-count)  ;; 処理したリクエスト数
```

#### 実用例2: 状態を持つキャッシュ

```lisp
;; キャッシュ
(def cache (atom {}))

(defn get-or-fetch [key fetch-fn]
  (let [cached (get (deref cache) key)]
    (if cached
      cached
      (let [value (fetch-fn)]
        (do
          (swap! cache assoc key value)
          value)))))

;; 使用例
(get-or-fetch :user-123 (fn [] (fetch-from-db :user-123)))
```

#### 実用例3: 接続管理（deferと組み合わせ）

```lisp
;; アクティブな接続を管理
(def clients (atom #{}))

(defn handle-connection [conn]
  (do
    (swap! clients conj conn)
    (defer (swap! clients dissoc conn))  ;; 確実にクリーンアップ
    (process-connection conn)))

;; アクティブ接続数
(len (deref clients))
```

#### 実用例4: 複雑な状態更新

```lisp
;; アプリケーション状態
(def app-state (atom {
  :users {}
  :posts []
  :status "running"
}))

;; ユーザー追加
(defn add-user [user]
  (swap! app-state (fn [state]
    (assoc state :users
      (assoc (get state :users) (get user :id) user)))))

;; 投稿追加
(defn add-post [post]
  (swap! app-state (fn [state]
    (assoc state :posts (conj (get state :posts) post)))))

;; 状態確認
(deref app-state)
```

#### Atomの設計哲学

1. **局所的な状態**: グローバル変数の代わりに、必要な場所だけでAtomを使う
2. **swap!の原子性**: 更新が確実に適用される（競合状態を回避）
3. **関数型との共存**: 純粋関数とAtomを組み合わせる
4. **deferと相性が良い**: リソース管理で威力を発揮

#### ✅ @ 構文（実装済み）

```lisp
;; derefの短縮形
(deref counter)  ;; 従来
@counter         ;; 短縮形

;; どちらも同じ意味
(print (deref state))
(print @state)

;; パイプラインで便利
(def cache (atom {:user-123 {:name "Alice"}}))
(get @cache :user-123)  ;; {:name "Alice"}

;; 関数の引数としても使える
(def users (atom [{:name "Alice"} {:name "Bob"}]))
(first @users)  ;; {:name "Alice"}
(map (fn [u] (get u :name)) @users)  ;; ("Alice" "Bob")
```

### ✅ エラー処理（全て実装済み）
```lisp
;; ✅ 実装済み
try                     ;; エラーを {:ok ...} / {:error ...} に変換
error                   ;; 例外を投げる（回復不能）
```

### ✅ メタプログラミング（実装済み）
```lisp
;; ✅ 実装済み
mac                     ;; マクロ定義
quasiquote (`)          ;; テンプレート
unquote (,)             ;; 値の埋め込み
unquote-splice (,@)     ;; リストの展開
uvar                    ;; 一意な変数を生成（マクロの衛生性）
variable                ;; 変数かどうかチェック
macro?                  ;; マクロかどうか
eval                    ;; 式を評価
```

## 7. ループ構造

### ✅ `loop` / `recur`（実装済み）

末尾再帰最適化を実現するための特殊形式です。

```lisp
;; 基本形
(loop [var1 val1 var2 val2 ...]
  body
  (recur new-val1 new-val2 ...))

;; 階乗（5! = 120）
(defn factorial [n]
  (loop [i n acc 1]
    (if (= i 0)
      acc
      (recur (dec i) (* acc i)))))

(factorial 5)  ;; 120

;; カウントダウン
(defn count-down [n]
  (loop [i n]
    (if (<= i 0)
      "done"
      (do
        (print i)
        (recur (dec i))))))

;; リスト処理（matchと組み合わせる場合は要実装）
;; 現在は以下のような形で実装可能：
(defn sum-list [lst]
  (loop [items lst result 0]
    (if (empty? items)
      result
      (recur (rest items) (+ result (first items))))))

(sum-list [1 2 3 4 5])  ;; 15
```

**実装のポイント**:
- `loop`は新しい環境を作成し、変数を初期値で束縛
- `recur`は特別なエラーとして扱い、`loop`でキャッチして変数を更新
- 通常の再帰と異なり、スタックを消費しない（末尾再帰最適化）
```

## 8. エラー処理戦略

### エラー処理の3層構造

Qiは用途に応じて3つのエラー処理方法を提供します：

1. **Result型 (`{:ok/:error}`)** - 回復可能なエラー、Railway Pipeline
2. **try/catchブロック** - 例外のキャッチとリカバリ
3. **defer** - リソース解放の保証（`finally`の代替）

---

### 1. Result型 - Railway Pipeline ✅ **推奨パターン**

**用途**: API、ファイルIO、パース等の失敗が予想される処理

```lisp
;; Result型を返す関数
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}
    {:ok (/ x y)}))

;; Railway Pipelineで処理
(user-input
 |> validate
 |>? parse-number
 |>? (fn [n] (divide 100 n))
 |>? format-result)
;; エラーは自動的に伝播

;; またはmatchで処理
(match (divide 10 2)
  {:ok result} -> result
  {:error e} -> (log e))
```

**設計哲学**: エラーをデータとして扱い、パイプラインの中で流す。

---

### 2. try/catch - 例外処理 ✅

**用途**: 予期しないエラーのキャッチ、サードパーティコードの呼び出し

```lisp
;; try-catchブロック
(match (try (risky-operation))
  {:ok result} -> result
  {:error e} -> (handle-error e))

;; ネスト可能
(match (try
         (let data (parse-data input))
         (process data))
  {:ok result} -> result
  {:error e} -> {:error (str "Failed: " e)})
```

**注意**: Qiには`finally`がありません。代わりに`defer`を使います（下記参照）。

---

### 3. defer - リソース解放の保証 ✅ **finallyの代替**

**用途**: ファイル、接続、ロックなどのリソース管理

```lisp
;; deferで確実にクリーンアップ
(defn process-file [path]
  (let f (open-file path))
  (defer (close-file f))  ;; 関数終了時に必ず実行
  (let data (read-file f))
  (transform data))

;; 複数のdeferはスタック的に実行（後入れ先出し）
(defn complex-operation []
  (let conn (open-connection))
  (defer (close-connection conn))
  (let lock (acquire-lock))
  (defer (release-lock lock))
  (let file (open-file "data.txt"))
  (defer (close-file file))
  ;; 処理...
  ;; 終了時: close-file → release-lock → close-connection
  )

;; エラー時もdeferは実行される
(defn safe-process []
  (let res (allocate-resource))
  (defer (free-resource res))
  (if (error-condition?)
    (error "something went wrong")  ;; deferは実行される
    (process res)))
```

**設計哲学**:
- `finally`よりシンプル - 関数のどこにでも書ける
- 強力 - 複数のdeferを組み合わせられる
- Go言語のdeferと同じ設計
- Lisp的 - 特殊な構文を増やさない

**なぜfinallyがないのか**: `defer`の方が柔軟で、複数のリソース管理が直感的。try-catch-finallyのネストより読みやすい。

---

### 回復可能 - {:ok/:error}
```lisp
;; 関数が結果を返す
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}
    {:ok (/ x y)}))

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

(defn factorial [n]
  (if (< n 0)
    (error "negative input not allowed")
    (loop [i n acc 1]
      (if (= i 0)
        acc
        (recur (dec i) (* acc i))))))

;; try でキャッチ
(match (try (factorial -5))
  {:ok result} -> result
  {:error e} -> (log (str "Error: " e)))
```

## 9. ユニーク変数（uvars）

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

## 10. モジュールシステム（✅ 基本機能実装済み）

### ✅ モジュール定義
```lisp
;; http.qi
(module http)

(defn get [url] ...)
(defn post [url data] ...)

(export get post)
```

### インポート
```lisp
;; ✅ パターン1: 特定の関数のみ（推奨・実装済み）
(use http :only [get post])
(get url)

;; 🚧 パターン2: エイリアス（未実装）
(use http :as h)
(h/get url)

;; ✅ パターン3: 全てインポート（実装済み）
(use http :all)
(get url)

;; 🚧 パターン4: リネーム（未実装）
(use http :only [get :as fetch])
(fetch url)
```

**実装状況メモ**:
- ✅ `module` / `export` - モジュール定義・エクスポート
- ✅ `use :only [...]` - 特定関数のインポート
- ✅ `use :all` - 全てインポート
- ✅ 循環参照検出
- ✅ `use :as` - エイリアス機能（実装済み）

### 標準モジュール

#### ✅ core（自動インポート・87個）
Coreモジュールは自動的にグローバル名前空間にインポートされます。

```lisp
;; 特殊形式・演算子（11個）
def fn let do if match try defer
|> ||> |>?

;; リスト操作（29個）
first rest last nth len count
take drop cons conj concat flatten range reverse
map filter reduce pmap tap
find every some take-while drop-while
sort distinct
identity comp partial apply constantly

;; マップ操作（9個）
get keys vals assoc dissoc merge
get-in update-in update

;; 数値・比較（17個）
+ - * / % inc dec abs min max sum
= != < > <= >=

;; 文字列（3個）
str split join

;; 述語・型判定（22個）
nil? list? vector? map? string?
integer? float? number? keyword?
function? atom? coll? sequential?
empty? some? true? false?
even? odd? positive? negative? zero?

;; 並行処理（5個）
go chan send! recv! close!

;; 論理・I/O（4個）
not print println error
;; 注: and, or は特殊形式（遅延評価のため）

;; 状態管理（4個）
atom deref swap! reset!

;; メタプログラミング（4個）
eval uvar variable macro?

;; 型変換（3個）
to-int to-float to-string

;; 日時（3個）
now timestamp sleep

;; デバッグ（1個）
time
```

#### 専門モジュール

##### ✅ list - 高度なリスト操作（18個）
```lisp
list/frequencies list/sort-by list/count-by
list/max-by list/min-by list/sum-by list/find-index
list/partition list/partition-by list/group-by list/keep
list/zip list/chunk list/zipmap
list/interleave list/take-nth list/dedupe
list/split-at list/drop-last
```

##### ✅ map - 高度なマップ操作（5個）
```lisp
map/select-keys
map/assoc-in map/dissoc-in
map/update-keys map/update-vals
```

##### ✅ fn - 高階関数（3個）
```lisp
fn/complement fn/juxt fn/tap>
```

##### ✅ set - 集合演算（7個）
```lisp
set/union set/intersect set/difference
set/subset? set/superset? set/disjoint?
set/symmetric-difference
```

##### ✅ math - 数学関数（10個）
```lisp
math/pow math/sqrt
math/round math/floor math/ceil math/clamp
math/rand math/rand-int
```

##### ✅ time - 日付・時刻（25個）
```lisp
time/now-iso time/today
time/from-unix time/to-unix time/format time/parse
time/add-days time/add-hours time/add-minutes
time/sub-days time/sub-hours time/sub-minutes
time/diff-days time/diff-hours time/diff-minutes
time/before? time/after? time/between?
time/year time/month time/day
time/hour time/minute time/second time/weekday
```

##### ✅ io - ファイルI/O（19個） - グローバルエンコーディング対応（日中韓欧露）

**ファイル読み書き**:
- `io/read-file` - ファイル読み込み（エンコーディング指定・自動検出対応）
- `io/write-file` - ファイル書き込み（エンコーディング指定、if-exists、create-dirs対応）
- `io/append-file` - ファイル追記
- `io/write-stream` - ストリーム→ファイル書き込み
- `io/read-lines` - 行ごと読み込み

**ファイルシステム操作**:
- `io/list-dir` - ディレクトリ一覧取得（グロブパターン対応）
- `io/create-dir` - ディレクトリ作成
- `io/delete-file` - ファイル削除
- `io/delete-dir` - ディレクトリ削除
- `io/copy-file` - ファイルコピー
- `io/move-file` - ファイル移動・名前変更

**メタデータ**:
- `io/file-info` - ファイル情報取得
- `io/file-exists?` - ファイル存在確認
- `io/is-file?` - ファイル判定
- `io/is-dir?` - ディレクトリ判定

**エンコーディングサポート** - グローバル対応:

**Unicode**:
- `:utf-8` (デフォルト、BOM自動除去)
- `:utf-8-bom` (BOM付きUTF-8、Excel対応)
- `:utf-16le` (UTF-16LE、BOM付き、Excel多言語対応)
- `:utf-16be` (UTF-16BE、BOM付き)

**日本語**:
- `:sjis` / `:shift-jis` (Shift_JIS/Windows-31J、日本Windows/Excel)
- `:euc-jp` (EUC-JP、Unix系)
- `:iso-2022-jp` (JIS、メール)

**中国語**:
- `:gbk` (GBK、中国本土・シンガポール、簡体字Windows/Excel)
- `:gb18030` (GB18030、中国国家規格、GBK上位互換)
- `:big5` (Big5、台湾・香港、繁体字Windows/Excel)

**韓国語**:
- `:euc-kr` (EUC-KR、韓国Windows/Excel)

**欧州**:
- `:windows-1252` / `:cp1252` / `:latin1` (西欧、米国Windows/Excel)
- `:windows-1251` / `:cp1251` (ロシア・キリル文字圏Windows/Excel)

**自動検出**:
- `:auto` (BOM検出 → UTF-8 → 各地域エンコーディングを順次試行)

```lisp
;; ============================================
;; 基本的な読み書き
;; ============================================

;; シンプル（UTF-8）
(io/read-file "data.txt")
(io/write-file content "output.txt")

;; ============================================
;; エンコーディング指定
;; ============================================

;; Shift_JIS（日本語Windows/Excel）
(io/read-file "legacy.csv" :encoding :sjis)
(io/write-file data "for_excel.csv" :encoding :sjis)

;; UTF-8 BOM付き（Excel用CSV）
(io/write-file csv-data "excel.csv" :encoding :utf-8-bom)

;; 自動検出（エンコーディング不明なファイル）
(io/read-file "unknown.txt" :encoding :auto)

;; ============================================
;; 書き込みオプション
;; ============================================

;; ファイル存在時の動作
(io/write-file data "out.txt" :if-exists :error)      ;; 存在したらエラー
(io/write-file data "out.txt" :if-exists :skip)       ;; 存在したらスキップ
(io/write-file data "out.txt" :if-exists :append)     ;; 追記
(io/write-file data "out.txt" :if-exists :overwrite)  ;; 上書き（デフォルト）

;; ディレクトリ自動作成
(io/write-file data "path/to/out.txt" :create-dirs true)

;; 複数オプション組み合わせ
(io/write-file data "backup/data.csv"
               :encoding :sjis
               :if-exists :error
               :create-dirs true)

;; ============================================
;; 実用例 - 各国のExcel/レガシーシステム対応
;; ============================================

;; 日本: Excel用CSV（Shift_JIS）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "japan_excel.csv" :encoding :sjis)))

;; 中国（簡体字）: Excel用CSV（GBK）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "china_excel.csv" :encoding :gbk)))

;; 台湾・香港（繁体字）: Excel用CSV（Big5）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "taiwan_excel.csv" :encoding :big5)))

;; 韓国: Excel用CSV（EUC-KR）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "korea_excel.csv" :encoding :euc-kr)))

;; 西欧・米国: Excel用CSV（Windows-1252）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "europe_excel.csv" :encoding :windows-1252)))

;; 多言語混在: UTF-16LE（Excel推奨、BOM付き）
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "multilang_excel.csv" :encoding :utf-16le)))

;; レガシーエンコーディング → UTF-8変換
(io/read-file "legacy.csv" :encoding :sjis)
 |> csv/parse
 |> (map transform)
 |> csv/stringify
 |> (fn [s] (io/write-file s "modern_utf8.csv"))

;; エンコーディング不明ファイルの自動検出
(io/read-file "unknown.txt" :encoding :auto)
 |> process
 |> (fn [s] (io/write-file s "output.txt" :encoding :utf-8-bom))

;; 安全な書き込み（既存ファイル保護）
(io/write-file data "important.txt"
               :if-exists :error
               :create-dirs true)

;; ============================================
;; ファイルシステム操作
;; ============================================

;; ディレクトリ一覧取得
(io/list-dir ".")                                ;; カレントディレクトリ
(io/list-dir "logs" :pattern "*.log")            ;; ログファイルのみ
(io/list-dir "src" :pattern "*.rs" :recursive true)  ;; 再帰的に検索

;; ディレクトリ操作
(io/create-dir "data/backup")                    ;; 親も自動作成
(io/delete-dir "temp")                           ;; 空ディレクトリ削除
(io/delete-dir "old_data" :recursive true)       ;; 中身ごと削除

;; ファイル操作
(io/copy-file "data.txt" "data_backup.txt")      ;; コピー
(io/move-file "old.txt" "new.txt")               ;; 移動・名前変更
(io/delete-file "temp.txt")                      ;; 削除

;; メタデータ取得
(def info (io/file-info "data.txt"))
(get info "size")                                ;; ファイルサイズ
(get info "modified")                            ;; 更新日時（UNIXタイムスタンプ）
(get info "is-dir")                              ;; ディレクトリか
(get info "is-file")                             ;; ファイルか

;; 判定
(io/is-file? "data.txt")                         ;; true
(io/is-dir? "data")                              ;; true
(io/file-exists? "config.json")                  ;; true/false

;; パイプラインと組み合わせ
("logs"
 |> (io/list-dir :pattern "*.log")
 |> (map io/read-file)
 |> (map process-log)
 |> (reduce merge))
```

**一時ファイル・ディレクトリ**:
- `io/temp-file` - 一時ファイル作成（自動削除）
- `io/temp-file-keep` - 一時ファイル作成（削除しない）
- `io/temp-dir` - 一時ディレクトリ作成（自動削除）
- `io/temp-dir-keep` - 一時ディレクトリ作成（削除しない）

```lisp
;; ============================================
;; 一時ファイル（自動削除）
;; ============================================

;; 一時ファイルを作成して使用（プログラム終了時に自動削除）
(let [tmp (io/temp-file)]
  (io/write-file "temporary data" tmp)
  (process-file tmp))
;; プログラム終了時にtmpは自動的に削除される

;; 一時ディレクトリを作成（自動削除）
(let [tmpdir (io/temp-dir)]
  (io/write-file "data1" (path/join tmpdir "file1.txt"))
  (io/write-file "data2" (path/join tmpdir "file2.txt"))
  (process-directory tmpdir))
;; プログラム終了時にtmpdirと中身は自動的に削除される

;; ============================================
;; 一時ファイル（削除しない）
;; ============================================

;; 永続的な一時ファイルを作成（手動で削除が必要）
(let [tmp (io/temp-file-keep)]
  (io/write-file "persistent data" tmp)
  (println f"Created: {tmp}")
  tmp)
;; => "/tmp/.tmpXXXXXX" （削除されない）

;; 永続的な一時ディレクトリを作成
(let [tmpdir (io/temp-dir-keep)]
  (io/create-dir (path/join tmpdir "subdir"))
  tmpdir)
;; => "/tmp/.tmpXXXXXX" （削除されない）

;; ============================================
;; 実用例: 一時ファイルでのデータ処理
;; ============================================

;; 大きなデータを一時ファイルで処理
(defn process-large-data [url]
  (let [tmp (io/temp-file)]
    ;; データをダウンロードして一時ファイルに保存
    (http/get url :output tmp)
    ;; 一時ファイルを処理
    (let [result (process-file tmp)]
      ;; 関数終了後、tmpは自動削除される
      result)))

;; 複数の一時ファイルを使用
(defn merge-files [files output]
  (let [tmpdir (io/temp-dir)
        processed (files
                   |> (map (fn [f]
                         (let [tmp (path/join tmpdir (path/basename f))]
                           (io/copy-file f tmp)
                           (process-file tmp)
                           tmp))))]
    ;; 処理済みファイルをマージ
    (merge-all processed output)
    ;; 関数終了後、tmpdirと中身は自動削除される
    output))

;; ============================================
;; 実用例: ビルドの一時ディレクトリ
;; ============================================

;; ビルド成果物を一時ディレクトリで作成してからコピー
(defn build-project [source-dir output-dir]
  (let [build-dir (io/temp-dir)]
    (try
      (do
        ;; 一時ディレクトリでビルド
        (compile-sources source-dir build-dir)
        (run-tests build-dir)
        ;; 成功したら出力ディレクトリにコピー
        (io/copy-file build-dir output-dir)
        {:ok true})
      (catch e
        ;; エラーが起きても一時ディレクトリは自動削除される
        {:error e})))))
```

**注意**: パイプラインでキーワード引数を使う場合は無名関数でラップしてください。

##### ✅ path - パス操作（9個）

プラットフォーム非依存のパス操作を提供。

**パス操作**:
- `path/join` - パス結合
- `path/basename` - ファイル名取得
- `path/dirname` - ディレクトリ名取得
- `path/extension` - 拡張子取得
- `path/stem` - 拡張子なしファイル名取得
- `path/absolute` - 絶対パス化
- `path/normalize` - パス正規化

**パス判定**:
- `path/is-absolute?` - 絶対パス判定
- `path/is-relative?` - 相対パス判定

```lisp
;; パス結合
(path/join "dir" "subdir" "file.txt")            ;; "dir/subdir/file.txt"
(path/join "/usr" "local" "bin")                 ;; "/usr/local/bin"

;; ファイル名・ディレクトリ名取得
(path/basename "/path/to/file.txt")              ;; "file.txt"
(path/dirname "/path/to/file.txt")               ;; "/path/to"
(path/extension "file.txt")                      ;; "txt"
(path/extension "archive.tar.gz")                ;; "gz"
(path/stem "file.txt")                           ;; "file"
(path/stem "archive.tar.gz")                     ;; "archive.tar"

;; パス正規化
(path/normalize "a/./b/../c")                    ;; "a/c"
(path/normalize "/usr/local/../bin")             ;; "/usr/bin"

;; 絶対パス化
(path/absolute "relative/path")                  ;; "/current/dir/relative/path"

;; パス判定
(path/is-absolute? "/usr/bin")                   ;; true
(path/is-absolute? "relative/path")              ;; false
(path/is-relative? "data/file.txt")              ;; true

;; パイプラインと組み合わせ
("logs"
 |> io/list-dir
 |> (filter (fn [p] (= (path/extension p) "log")))
 |> (map (fn [p] (path/join "archive" (path/basename p))))
 |> (map (fn [dst] (io/copy-file (path/join "logs" (path/basename dst)) dst))))
```

##### ✅ env - 環境変数（4個）

アプリケーション設定や環境依存の値を管理。dotenvファイルサポート。

**環境変数操作**:
- `env/get` - 環境変数取得（デフォルト値対応）
- `env/set` - 環境変数設定
- `env/all` - 全環境変数をマップで取得
- `env/load-dotenv` - .envファイルを読み込み

```lisp
;; 環境変数取得
(env/get "HOME")                                 ;; "/Users/username"
(env/get "PORT" "3000")                          ;; デフォルト値付き

;; 環境変数設定
(env/set "API_KEY" "secret123")
(env/set "DEBUG" "true")

;; 全環境変数取得
(def all-env (env/all))                          ;; {:PATH "..." :HOME "..." ...}

;; .envファイル読み込み
(env/load-dotenv)                                ;; ".env"を読み込み
(env/load-dotenv ".env.production")              ;; 特定のファイル

;; .envファイル形式
;; # コメント
;; DATABASE_URL=postgresql://localhost:5432/mydb
;; API_KEY=secret123
;; DEBUG=true
;; HOST="0.0.0.0"
;; NAME='My App'

;; APIサーバー設定例
(env/load-dotenv)
(def config
  {:port (to-int (env/get "PORT" "3000"))
   :host (env/get "HOST" "localhost")
   :db-url (env/get "DATABASE_URL")
   :debug (= (env/get "DEBUG" "false") "true")})
```

##### ✅ log - 構造化ログ（6個）

プロダクション対応の構造化ログ出力。レベルフィルタリング、JSON形式対応。

**ログ出力**:
- `log/debug` - DEBUGレベルログ
- `log/info` - INFOレベルログ
- `log/warn` - WARNレベルログ
- `log/error` - ERRORレベルログ

**設定**:
- `log/set-level` - ログレベル設定（debug/info/warn/error）
- `log/set-format` - 出力形式設定（text/json）

```lisp
;; 基本的なログ出力
(log/info "サーバー起動")
;; => [2025-10-11T21:40:37.312+0900] INFO サーバー起動

;; 構造化データ付きログ
(log/info "API呼び出し" {:method "GET" :path "/users" :status 200})
;; => [2025-10-11T21:40:37.312+0900] INFO API呼び出し | method=GET path=/users status=200

;; エラーログ
(log/error "データベース接続失敗" {:error "connection refused" :db "users"})

;; ログレベル設定（デフォルト: info）
(log/set-level "debug")                          ;; DEBUG以上を出力
(log/set-level "warn")                           ;; WARN以上のみ出力

;; JSON形式で出力（構造化ログ）
(log/set-format "json")
(log/info "API呼び出し" {:method "GET" :path "/users" :status 200})
;; => {"timestamp":"2025-10-11T21:40:37.364+0900","level":"INFO","message":"API呼び出し","method":"GET","path":"/users","status":"200"}

;; APIサーバーでの使用例
(do
  (log/set-format "json")                        ;; 本番環境はJSON
  (log/set-level (if (env/get "DEBUG") "debug" "info"))

  (log/info "サーバー起動" {:port 3000 :env "production"})

  ;; リクエストログ
  (fn [request]
    (log/info "HTTP Request"
              {:method (get request :method)
               :path (get request :path)
               :ip (get request :ip)})))
```

##### ✅ dbg - デバッグ（2個）
```lisp
dbg/inspect dbg/time
```

##### ✅ async - 並行処理（高度）（16個）
```lisp
;; チャネル拡張
async/try-recv! async/select!

;; Structured Concurrency
async/make-scope async/scope-go async/with-scope
async/cancel! async/cancelled?

;; 並列処理
async/pfilter async/preduce async/parallel-do

;; Promise
async/await async/then async/catch async/all async/race
```

##### ✅ pipeline - パイプライン処理（5個）
```lisp
pipeline/pipeline pipeline/map pipeline/filter
pipeline/fan-out pipeline/fan-in
```

##### ✅ stream - ストリーム処理（11個）
```lisp
stream/stream stream/range stream/repeat stream/cycle
stream/take stream/drop stream/realize stream/iterate
stream/map stream/filter stream/file
```

##### ✅ zip - ZIP圧縮・解凍（6個）
```lisp
zip/create zip/extract zip/list zip/add
zip/gzip zip/gunzip
```

##### ✅ args - コマンドライン引数パース（4個）
```lisp
args/all args/get args/parse args/count
```

##### ✅ db - データベース（Phase 1: 基本操作）

**概要**:
- **統一API**: 複数データベースに対する共通インターフェース
- **標準サポート**: SQLite（組み込み、外部依存なし）
- **拡張可能**: PostgreSQL、MySQL、DuckDB等は外部パッケージとして提供予定
- **セキュアAPI**: プレースホルダー + サニタイズ機能

**Phase 1 機能** (✅ 実装済み):
```lisp
db/connect              ;; データベースに接続
db/query                ;; SQLクエリを実行（複数行）
db/query-one            ;; SQLクエリを実行（1行のみ）
db/exec                 ;; SQL文を実行（INSERT/UPDATE/DELETE）
db/close                ;; 接続を閉じる
db/sanitize             ;; 値をサニタイズ
db/sanitize-identifier  ;; 識別子をサニタイズ
db/escape-like          ;; LIKE句のパターンをエスケープ
db/begin                ;; トランザクション開始
db/commit               ;; トランザクションコミット
db/rollback             ;; トランザクションロールバック
```

**Phase 2 機能** (🚧 部分実装):
- ✅ トランザクション: `db/begin`, `db/commit`, `db/rollback` - 実装済み
- 🚧 メタデータAPI: `db/tables`, `db/columns`, `db/indexes`, `db/foreign-keys` - 未実装
- 🚧 ストアド実行: `db/call` (関数のRETURN値、プロシージャのOUT/INOUT/結果セット対応) - 未実装
- 🚧 クエリ情報: `db/query-info` - 未実装
- 🚧 機能検出: `db/supports?`, `db/driver-info` - 未実装

**Phase 3 機能** (🚧 未実装):
- 🚧 コネクションプーリング - 未実装

**基本的な使い方**:
```lisp
;; 接続（SQLite）
(def conn (db/connect "sqlite:app.db"))
(def conn (db/connect "sqlite::memory:"))  ;; インメモリDB

;; 接続オプション
(def conn (db/connect "sqlite:app.db" {
  "timeout" 30000        ;; タイムアウト(ms)
  "read-only" false      ;; 読み取り専用
}))

;; テーブル作成
(db/exec conn "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)" [])

;; データ挿入（プレースホルダー使用）
(db/exec conn "INSERT INTO users (name, age) VALUES (?, ?)" ["Alice" 30])
(db/exec conn "INSERT INTO users (name, age) VALUES (?, ?)" ["Bob" 25])

;; 全データ取得
(def users (db/query conn "SELECT * FROM users" []))
;; => [{"id" 1 "name" "Alice" "age" 30} {"id" 2 "name" "Bob" "age" 25}]

;; 条件付き取得
(def alice (db/query-one conn "SELECT * FROM users WHERE name = ?" ["Alice"]))
;; => {"id" 1 "name" "Alice" "age" 30}

;; 更新
(def affected (db/exec conn "UPDATE users SET age = ? WHERE name = ?" [31 "Alice"]))
;; => 1

;; 削除
(def deleted (db/exec conn "DELETE FROM users WHERE id = ?" [2]))
;; => 1

;; 接続を閉じる
(db/close conn)
```

**クエリオプション**:
```lisp
;; タイムアウト、limit、offset指定
(db/query conn "SELECT * FROM users" [] {
  "timeout" 5000    ;; タイムアウト(ms)
  "limit" 10        ;; 最大取得行数
  "offset" 20       ;; スキップ行数
})
```

**サニタイズ（動的SQL用）**:
```lisp
;; ⚠️ 基本的にプレースホルダー(?)を使用すべき
;; サニタイズは動的SQL構築時のみ使用

;; 値のエスケープ（シングルクォート対応）
(db/sanitize conn "O'Reilly")
;; => "O''Reilly"

;; 識別子のエスケープ（テーブル名、カラム名）
(db/sanitize-identifier conn "user_name")
;; => "\"user_name\""

;; LIKE句のパターンエスケープ
(db/escape-like conn "50%_off")
;; => "50\\%\\_off"
```

**安全なクエリ例**:
```lisp
;; ✅ 推奨: プレースホルダー使用
(db/query conn "SELECT * FROM users WHERE age > ?" [min-age])

;; ⚠️ 動的カラム名: sanitize-identifier使用
(def col-name (db/sanitize-identifier conn user-input))
(def sql (str "SELECT " col-name " FROM users"))
(db/query conn sql [])

;; ⚠️ LIKE検索: escape-like使用
(def pattern (db/escape-like conn user-search))
(db/query conn "SELECT * FROM products WHERE name LIKE ? ESCAPE '\\'"
          [(str "%" pattern "%")])
```

**パイプラインとの組み合わせ**:
```lisp
;; データ処理パイプライン
(db/query conn "SELECT * FROM users WHERE age > ?" [25])
  |> (filter (fn [u] (starts-with? (get u "name") "A")))
  |> (map (fn [u] (assoc u "senior" true)))
  |> (take 10)

;; CSV → DB インポート
(io/read-file "users.csv")
  |> csv/parse
  |> (map (fn [row]
            (db/exec conn
                     "INSERT INTO users (name, age) VALUES (?, ?)"
                     [(get row "name") (to-int (get row "age"))])))
```

**将来の拡張（Phase 2+）**:
```lisp
;; トランザクション（Phase 2）
(db/transaction conn
  (fn []
    (db/exec conn "INSERT INTO accounts ..." [...])
    (db/exec conn "UPDATE balance ..." [...]))
  {"isolation" "serializable" "timeout" 10000})

;; メタデータAPI（Phase 2）
(db/tables conn)
;; => ["users" "products" "orders"]

(db/columns conn "users")
;; => [{"name" "id" "type" "INTEGER" "nullable" false "primary-key" true}
;;     {"name" "name" "type" "TEXT" "nullable" false}]

;; ストアドプロシージャ/ファンクション実行（Phase 2）
;; 関数（RETURN値あり）
(db/call conn "add" [1 2])
;; => 3

;; プロシージャ（結果セット）
(db/call conn "get_users_by_age" [25])
;; => [{"id" 1 "name" "Alice" "age" 30} ...]

;; プロシージャ（OUTパラメータ）
(db/call conn "calc" [2 3] {"out" ["sum" "product"]})
;; => {"out" {"sum" 5 "product" 6}}

;; プロシージャ（INOUTパラメータ）
(db/call conn "increment" [10] {"inout" ["value"]})
;; => {"inout" {"value" 11}}

;; 複数結果セット + OUTパラメータ
(db/call conn "complex_proc" [arg1] {"out" ["status" "message"]})
;; => {"result-sets" [[rows1...] [rows2...]]
;;     "out" {"status" 0 "message" "Success"}}

;; 機能検出
(db/supports? conn "stored-procedures")
;; => true (PostgreSQL, MySQL) / false (SQLite)

;; 外部DB（外部パッケージとして提供予定）
(db/connect "postgresql://localhost/mydb")
(db/connect "mysql://user:pass@localhost/mydb")
(db/connect "duckdb:analytics.db")
```

**設計思想**:
- **統一API**: Python DBAPI 2.0やGo database/sqlのような統一インターフェース
- **ドライバー分離**: SQLiteは標準、他DBは外部パッケージ
- **安全第一**: プレースホルダー推奨、サニタイズは補助的
- **エスケープ責任**: 各ドライバーがDB固有のエスケープルールを実装
- **Phase分け**: Phase 1(基本), Phase 2(メタデータ), Phase 3(プーリング)

#### ✅ str - 文字列操作（ほぼ完全実装）
```lisp
(use str :only [
  ;; 検索 ✅
  contains? starts-with? ends-with?
  index-of last-index-of

  ;; 基本変換 ✅
  upper lower capitalize title
  trim trim-left trim-right
  pad-left pad-right pad               ;; pad-left/rightは左右詰め、padは中央揃え
  repeat reverse

  ;; ケース変換（重要） ✅
  snake        ;; "userName" -> "user_name"
  camel        ;; "user_name" -> "userName"
  kebab        ;; "userName" -> "user-name"
  pascal       ;; "user_name" -> "UserName"
  split-camel  ;; "userName" -> ["user", "Name"]

  ;; 分割・結合 ✅
  split join lines words chars

  ;; 置換 ✅
  replace replace-first splice

  ;; 部分文字列 ✅
  slice take-str drop-str              ;; リストのtake/dropと区別
  sub-before sub-after                 ;; 区切り文字で前後を取得

  ;; 整形・配置 ✅
  truncate trunc-words

  ;; 正規化・クリーンアップ（重要） ✅
  squish                               ;; 連続空白を1つに、前後trim
  expand-tabs                          ;; タブをスペースに変換

  ;; 判定（バリデーション） ✅
  digit? alpha? alnum?
  space? lower? upper?
  numeric? integer? blank? ascii?

  ;; URL/Web ✅
  slugify              ;; "Hello World!" -> "hello-world"
  url-encode url-decode
  html-encode html-decode              ;; HTMLエンコード/デコード（旧: html-escape/unescape）

  ;; エンコード ✅
  to-base64 from-base64

  ;; パース ✅
  parse-int parse-float

  ;; Unicode ✅
  chars-count bytes-count  ;; Unicode文字数/バイト数

  ;; 高度な変換
  slugify      ;; ✅ "Hello World!" -> "hello-world"
  ;; unaccent  ;; 🚧 未実装 アクセント除去 "café" -> "cafe"

  ;; 生成 ✅
  hash uuid

  ;; 🚧 未実装
  random       ;; ランダム文字列生成
  map-lines    ;; 各行に関数を適用

  ;; NLP ✅
  word-count

  ;; フォーマット ✅
  format                  ;; プレースホルダー置換
  format-decimal          ;; 小数点桁数指定
  format-comma            ;; 3桁カンマ区切り
  format-percent          ;; パーセント表示
  indent wrap
])

;; 例
(use str :as s)

;; 基本
(s/upper "hello")  ;; "HELLO"
(s/split "a,b,c" ",")  ;; ["a" "b" "c"]
(s/repeat "-" 80)  ;; "----------------..." (80個)
(s/repeat "ab" 3)  ;; "ababab"

;; 検索
(s/contains? "hello world" "world")  ;; true
(s/starts-with? "hello" "he")  ;; true
(s/ends-with? "hello" "lo")  ;; true
(s/index-of "hello world" "world")  ;; 6
(s/last-index-of "hello hello" "hello")  ;; 6

;; ケース変換（重要）
(s/snake "userName")    ;; "user_name"
(s/kebab "userName")    ;; "user-name"
(s/camel "user_name")   ;; "userName"
(s/pascal "user_name")  ;; "UserName"

;; Slugify（Web開発必須）
(s/slugify "Hello World! 2024")  ;; "hello-world-2024"
(s/slugify "Café résumé")        ;; "cafe-resume"

;; 整形・配置
(s/pad-left "Total" 20)          ;; "               Total"
(s/pad-right "Name" 20)          ;; "Name               "
(s/pad "hi" 10)                  ;; "    hi    " (中央揃え)
(s/pad "hi" 10 "*")              ;; "****hi****"
(s/trunc-words article 10)       ;; 最初の10単語まで

;; 正規化（超重要）
(s/squish "  hello   world  \n")  ;; "hello world"
(s/expand-tabs "\thello\tworld")  ;; "    hello    world"

;; 判定（バリデーション）
(s/digit? "12345")   ;; true
(s/alpha? "hello")   ;; true
(s/alnum? "hello123") ;; true
(s/space? "  \n\t")  ;; true
(s/numeric? "123.45") ;; true
(s/integer? "123")   ;; true
(s/blank? "  \n")    ;; true
(s/ascii? "hello")   ;; true

;; 行操作
(s/map-lines s/trim text)
(s/map-lines #(str "> " %) quote)  ;; 各行にプレフィックス

;; Unicode
(s/chars-count "👨‍👩‍👧‍👦")  ;; 1 (視覚的な文字数)
(s/bytes-count "👨‍👩‍👧‍👦")  ;; 25 (バイト数)

;; 部分文字列
(s/take-str "hello" 3)       ;; "hel"
(s/drop-str "hello" 2)       ;; "llo"
(s/sub-before "user@example.com" "@")  ;; "user"
(s/sub-after "user@example.com" "@")   ;; "example.com"
(s/slice "hello world" 0 5)  ;; "hello"

;; 高度な変換
(s/splice "hello world" 6 11 "universe")  ;; "hello universe"
(s/title "hello world")                    ;; "Hello World"
(s/reverse "hello")                        ;; "olleh"
(s/chars "hello")                          ;; ["h" "e" "l" "l" "o"]

;; パース
(s/parse-int "123")    ;; 123
(s/parse-float "3.14") ;; 3.14

;; フォーマット - レイアウト
(s/indent "hello\nworld" 2)      ;; "  hello\n  world"
(s/wrap "hello world from qi" 10) ;; "hello\nworld from\nqi"
(s/truncate "hello world" 8)     ;; "hello..."
(s/trunc-words "hello world from qi" 2) ;; "hello world..."

;; フォーマット - プレースホルダー置換（Python/Rust風）
(s/format "Hello, {}!" "World")           ;; "Hello, World!"
(s/format "{} + {} = {}" 1 2 3)           ;; "1 + 2 = 3"
(s/format "Name: {}, Age: {}" "Alice" 30) ;; "Name: Alice, Age: 30"

;; フォーマット - 数値整形（パイプライン対応）
;; format-decimal: 小数点桁数を指定
(s/format-decimal 2 3.14159)     ;; "3.14"
(3.14159 |> (s/format-decimal 2)) ;; "3.14" (パイプラインで使用)

;; format-comma: 3桁カンマ区切り
(s/format-comma 1234567)          ;; "1,234,567"
(1234567 |> (s/format-comma))     ;; "1,234,567" (パイプラインで使用)
(s/format-comma 1234.5678)        ;; "1,234.5678"

;; format-percent: パーセント表示
(s/format-percent 0.1234)         ;; "12%" (デフォルトで0桁)
(s/format-percent 2 0.1234)       ;; "12.34%" (2桁指定)
(0.856 |> (s/format-percent 1))   ;; "85.6%" (パイプラインで使用)

;; 実用例: 価格表示のパイプライン
(defn format-price [price]
  (price
   |> (s/format-comma)
   |> (str/join "" ["¥" _])))

(format-price 1234567)  ;; "¥1,234,567"

;; 実用例: レポート生成
(defn gen-report [data]
  f"""
  Sales Report
  ============
  Total: {(s/format-comma (:total data))}
  Growth: {(s/format-percent 1 (:growth data))}
  """
)

(gen-report {:total 1234567 :growth 0.156})
;; =>
;; Sales Report
;; ============
;; Total: 1,234,567
;; Growth: 15.6%

;; NLP
(s/word-count "hello world")     ;; 2

;; ✅ エンコード/デコード（実装済み）
(s/to-base64 "hello")            ;; "aGVsbG8="
(s/from-base64 "aGVsbG8=")       ;; "hello"
(s/url-encode "hello world")     ;; "hello%20world"
(s/url-decode "hello%20world")   ;; "hello world"
(s/html-encode "<div>test</div>") ;; "&lt;div&gt;test&lt;/div&gt;"
(s/html-decode "&lt;div&gt;test&lt;/div&gt;") ;; "<div>test</div>"

;; ✅ ハッシュ/UUID（実装済み）
(s/hash "hello")                 ;; "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
(s/hash "hello" :sha256)         ;; SHA-256 (デフォルト)
(s/uuid)                         ;; "550e8400-e29b-41d4-a716-446655440000"
(s/uuid :v4)                     ;; UUID v4 (デフォルト)

;; 生成（未実装）
(s/random 16)          ;; "d7f3k9m2p5q8w1x4"
(s/random 16 :hex)     ;; "3f8a9c2e1b4d7056"
(s/random 16 :alnum)   ;; "aB3dE7fG9hJ2kL5m"
```

#### ✅ csv - CSV処理（実装済み）

**コア関数（パイプライン対応）**:
- `csv/parse` - CSV文字列をパース
- `csv/stringify` - データをCSV文字列に変換

**便利関数**:
- `csv/read-file` - ファイルを直接読み込み（`io/read-file` + `csv/parse`と同等）
- `csv/write-file` - ファイルに直接書き込み（`csv/stringify` + `io/write-file`と同等）
- `csv/read-stream` - ストリームとして読み込み

```lisp
;; 基本的な使い方（RFC 4180準拠、ダブルクォートエスケープ対応）
(csv/parse "name,age\n\"Alice\",30\n\"Bob\",25")
;; => (("name" "age") ("Alice" "30") ("Bob" "25"))

(csv/stringify '(("name" "age") ("Alice" "30")))
;; => "name,age\nAlice,30\n"

;; ✨ パイプライン推奨パターン - データが左から右へ流れる
("data.csv"
 |> io/read-file        ;; ファイル → 文字列
 |> csv/parse           ;; 文字列 → データ
 |> (filter active?)    ;; データ処理
 |> (map transform)
 |> csv/stringify       ;; データ → 文字列
 |> (io/write-file "output.csv"))  ;; 文字列 → ファイル

;; 便利関数 - シンプルな読み書き
(csv/read-file "data.csv")  ;; => (("name" "age") ("Alice" "30") ...)
(data |> (csv/write-file "output.csv"))  ;; データをCSV形式で保存

;; ストリーム処理（巨大ファイル対応）
("huge.csv"
 |> csv/read-stream
 |> (stream/take 1000)
 |> (stream/map transform)
 |> (io/write-stream "processed.txt"))

;; 実用例: CSVデータの変換パイプライン
(defn process-users []
  ("users.csv"
   |> io/read-file
   |> csv/parse
   |> rest                    ;; ヘッダー行をスキップ
   |> (filter (fn [row]       ;; 30歳以上のみ
        (>= (str/parse-int (nth row 1)) 30)))
   |> (map (fn [row]          ;; 年齢を+1
        (update row 1 (fn [age] (str (inc (str/parse-int age)))))))
   |> (cons '("name" "age"))  ;; ヘッダーを追加
   |> (csv/write-file "users_processed.csv"))))  ;; 便利関数で保存
```

#### ✅ regex - 正規表現（基本実装）

**実装済み機能**:
- `str/re-find` - パターンマッチング（最初の一致を検索）
- `str/re-matches` - 完全マッチチェック（文字列全体がパターンに一致するか）
- `str/re-replace` - 正規表現による置換

```lisp
(use str :as s)

;; パターンマッチ - 最初の一致を検索
(s/re-find "hello123world" "\\d+")
;; => "123"

;; 完全マッチチェック - 文字列全体がパターンに一致するか
(s/re-matches "hello123" "\\w+")
;; => true

(s/re-matches "hello 123" "\\w+")
;; => false (スペースがあるため)

;; 置換 - パターンに一致する部分を置換
(s/re-replace "hello123world456" "\\d+" "X")
;; => "helloXworldX"

;; パイプラインでの使用
("hello123world" |> (s/re-find "\\d+"))
;; => "123"

;; 実用例: メールアドレスの抽出
(defn extract-email [text]
  (s/re-find text "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"))

(extract-email "Contact: test@example.com for details")
;; => "test@example.com"

;; 実用例: バリデーション
(defn valid-username? [name]
  (s/re-matches name "^[a-zA-Z0-9_]{3,16}$"))

(valid-username? "user_123")  ;; => true
(valid-username? "ab")        ;; => false (短すぎる)
```

**将来の拡張（未実装）**:
- グループキャプチャ（名前付き・番号付き）
- `match-all` - 全マッチの取得
- `split` - 正規表現による分割
- `compile` - パターンのプリコンパイル
- コールバック置換
```

#### 🔜 math - 数学関数（計画中）

**設計方針**: Flow-orientedに合わせ、パイプラインで使いやすく。

```lisp
(use math :only [
  ;; 🔥 最優先（coreに含めても良い）
  pow sqrt                    ;; べき乗・平方根
  round floor ceil            ;; 丸め
  clamp                       ;; 範囲制限

  ;; ⚡ 高優先（数値計算の基本）
  abs                         ;; 絶対値（coreにもある）
  sign                        ;; 符号（-1, 0, 1）
  mod                         ;; 剰余（%との違いは負数の扱い）
  gcd lcm                     ;; 最大公約数・最小公倍数

  ;; 三角関数
  sin cos tan
  asin acos atan atan2
  sinh cosh tanh

  ;; 指数・対数
  exp log log10 log2

  ;; 乱数
  random                      ;; [0, 1)の乱数
  random-int                  ;; 整数乱数
  random-range                ;; 範囲指定乱数
  choice                      ;; リストからランダム選択
  shuffle                     ;; シャッフル

  ;; その他
  factorial
  prime?

  ;; 定数
  pi e tau
])

;; 使用例 - Flow-oriented設計
;; 1. パイプラインで使える
([1 2 3 4 5]
 |> (map (fn [x] (math/pow x 2)))
 |> math/mean)  ;; 平方の平均: 11.0

;; 2. 範囲制限（Web APIで頻出）
(user-input
 |> parse-int
 |> (fn [n] (math/clamp n 1 100)))  ;; 1-100に制限

;; 3. 統計処理
(defn analyze [data]
  {:mean (math/mean data)
   :median (math/median data)
   :stddev (math/stddev data)
   :p95 (math/percentile data 95)})

(analyze [10 20 30 40 50])
;; {:mean 30 :median 30 :stddev 14.14 :p95 48}

;; 4. 乱数（テストデータ生成で便利）
(math/random-int 1 100)  ;; 1-100の整数
(math/choice [:red :green :blue])
(math/shuffle [1 2 3 4 5])

;; 5. 丸め処理（金額計算など）
(price
 |> (* 1.08)              ;; 消費税
 |> math/round)           ;; 小数点以下四捨五入

;; 6. 三角関数（ゲーム・グラフィックス）
(defn rotate-point [x y angle]
  (let [rad (* angle (/ math/pi 180))]
    {:x (- (* x (math/cos rad)) (* y (math/sin rad)))
     :y (+ (* x (math/sin rad)) (* y (math/cos rad)))}))
```

**実装優先度**:
- ✅ Phase 1: pow, sqrt, round, floor, ceil, clamp（実装済み）
- ✅ Phase 2: random系（実装済み）
- ✅ Phase 3: statsモジュール（独立モジュールとして実装済み）
- Phase 4: 三角関数・対数（必要になったら）

#### ✅ stats - 統計関数（実装済み）

**データ分析のための統計モジュール**

```lisp
(use stats :only [
  mean              ;; 平均値
  median            ;; 中央値
  mode              ;; 最頻値
  variance          ;; 分散
  stddev            ;; 標準偏差
  percentile        ;; パーセンタイル（0-100）
])

;; 使用例
(def data [1 2 3 4 5 5 6 7 8 9 10])

;; 基本統計量
(stats/mean data)       ; => 5.454545...
(stats/median data)     ; => 5
(stats/mode data)       ; => 5

;; 分散・標準偏差
(stats/variance data)   ; => 7.272727...
(stats/stddev data)     ; => 2.697...

;; パーセンタイル
(stats/percentile data 50)   ; => 5.0 (中央値と同じ)
(stats/percentile data 95)   ; => 9.5

;; パイプラインで使える
(test-scores
 |> (filter passing?)
 |> stats/mean
 |> (fn [avg] (println f"Average: {avg}")))
```

**設計方針**:
- コレクション（リストまたはベクタ）を引数に取る
- すべての要素が数値である必要がある
- 空のコレクションはエラー
- Flow-oriented設計でパイプラインに組み込める

#### ✅ zip - ZIP圧縮・解凍とgzip（実装済み）

**ZIP圧縮・解凍のための汎用モジュール**

```lisp
(use zip :only [
  create            ;; ZIPファイルを作成
  extract           ;; ZIPファイルを解凍
  list              ;; ZIP内容を一覧表示
  add               ;; 既存ZIPにファイルを追加
  gzip              ;; gzip圧縮（単一ファイル）
  gunzip            ;; gzip解凍（単一ファイル）
])

;; ============================================
;; ZIP圧縮
;; ============================================

;; 単一ファイルをZIP化
(zip/create "archive.zip" "document.txt")

;; 複数ファイルをZIP化
(zip/create "archive.zip" ["file1.txt" "file2.txt" "data.csv"])

;; ディレクトリ全体をZIP化（再帰的）
(zip/create "backup.zip" "myproject/")

;; ============================================
;; ZIP解凍
;; ============================================

;; カレントディレクトリに解凍
(zip/extract "archive.zip")

;; 指定ディレクトリに解凍
(zip/extract "archive.zip" "extracted/")

;; ============================================
;; ZIP内容の確認
;; ============================================

;; ZIP内のファイル一覧を取得
(zip/list "archive.zip")
;; => [{:name "file1.txt" :size 1024 :compressed-size 512 :is-dir false}
;;     {:name "dir/" :size 0 :compressed-size 0 :is-dir true}
;;     {:name "dir/file2.txt" :size 2048 :compressed-size 1024 :is-dir false}]

;; パイプラインで処理
("archive.zip"
 |> zip/list
 |> (filter (fn [entry] (not (:is-dir entry))))
 |> (map :name))
;; => ["file1.txt" "dir/file2.txt"]

;; ============================================
;; 既存ZIPへのファイル追加
;; ============================================

;; 単一ファイルを追加
(zip/add "archive.zip" "newfile.txt")

;; 複数ファイルを追加
(zip/add "archive.zip" ["file3.txt" "file4.txt"])

;; ディレクトリを追加
(zip/add "archive.zip" "newdir/")

;; ============================================
;; gzip圧縮（単一ファイル）
;; ============================================

;; ファイルをgzip圧縮（.gz拡張子を自動付与）
(zip/gzip "largefile.txt")
;; => "largefile.txt.gz"を作成

;; 出力ファイル名を指定
(zip/gzip "largefile.txt" "output.gz")

;; ============================================
;; gzip解凍
;; ============================================

;; gzipファイルを解凍（.gz拡張子を自動除去）
(zip/gunzip "largefile.txt.gz")
;; => "largefile.txt"を作成

;; 出力ファイル名を指定
(zip/gunzip "data.gz" "data.txt")

;; ============================================
;; 実用例: ログファイルのアーカイブ
;; ============================================

;; 古いログをgzip圧縮してアーカイブ
(defn archive-logs [log-dir archive-name]
  (let [logs (io/list-dir log-dir :pattern "*.log")]
    ;; 各ログファイルをgzip圧縮
    (logs |> (map zip/gzip))
    ;; 圧縮ファイルをZIPにまとめる
    (let [gz-files (io/list-dir log-dir :pattern "*.gz")]
      (zip/create archive-name gz-files)
      ;; 元の.gzファイルを削除
      (gz-files |> (map io/delete-file)))))

(archive-logs "logs/" "logs-2025-01.zip")

;; ============================================
;; 実用例: バックアップと復元
;; ============================================

;; プロジェクトをバックアップ
(defn backup-project [project-dir backup-file]
  (zip/create backup-file project-dir)
  (println f"Backup created: {backup-file}"))

(backup-project "myapp/" "backups/myapp-2025-01-11.zip")

;; バックアップから復元
(defn restore-project [backup-file restore-dir]
  (zip/extract backup-file restore-dir)
  (println f"Restored to: {restore-dir}"))

(restore-project "backups/myapp-2025-01-11.zip" "restored/")
```

**設計方針**:
- ZIP圧縮にはDeflateアルゴリズムを使用（一般的なZIP形式）
- ディレクトリ構造を保持したまま圧縮・解凍
- gzipは単一ファイル向けの高速圧縮
- Pure Rustクレート（zip, flate2）を使用

#### ✅ args - コマンドライン引数パース（実装済み）

**CLI/サーバーアプリケーションのための引数パース**

```lisp
(use args :only [
  all               ;; 全コマンドライン引数を取得
  get               ;; 指定位置の引数を取得
  parse             ;; 引数をパース（フラグ・オプション・位置引数）
  count             ;; 引数の数を取得
])

;; ============================================
;; 基本的な引数アクセス
;; ============================================

;; 全引数を取得
(args/all)
;; プログラム実行: ./qi script.qi arg1 arg2
;; => ["./qi" "script.qi" "arg1" "arg2"]

;; 引数の数を取得
(args/count)
;; => 4

;; 指定位置の引数を取得
(args/get 0)           ;; => "./qi" (プログラム名)
(args/get 1)           ;; => "script.qi" (第1引数)
(args/get 2)           ;; => "arg1" (第2引数)

;; デフォルト値を指定
(args/get 5 "default") ;; => "default" (存在しない場合)
(args/get 10)          ;; => nil (存在せずデフォルトもない場合)

;; ============================================
;; 高度な引数パース（GNU形式）
;; ============================================

;; フラグ・オプション・位置引数を自動解析
(args/parse)
;; プログラム実行: ./qi script.qi --verbose --port 3000 -df input.txt
;; => {:flags ["verbose" "d" "f"]
;;     :options {"port" "3000"}
;;     :args ["./qi" "script.qi" "input.txt"]}

;; 解析ルール:
;; - "--flag"               → フラグ（真偽値）
;; - "--key=value"          → オプション（キー・値ペア）
;; - "--key value"          → オプション（キー・値ペア）
;; - "-abc"                 → 短縮フラグ（a, b, c の3つ）
;; - その他                 → 位置引数

;; ============================================
;; 実用例: CLIツール
;; ============================================

;; シンプルなファイル処理ツール
(defn main []
  (let [parsed (args/parse)
        flags (:flags parsed)
        options (:options parsed)
        files (:args parsed)]

    ;; フラグのチェック
    (let [verbose? (contains? flags "verbose")
          help? (contains? flags "help")]

      (if help?
        (print-help)
        (do
          ;; オプションの取得
          (let [output (map/get options "output" "output.txt")
                format (map/get options "format" "json")]

            ;; ファイル処理
            (when verbose?
              (println f"Processing {(count files)} files..."))

            (files
             |> (drop 2)  ;; プログラム名とスクリプト名をスキップ
             |> (map process-file)
             |> (fn [results] (save-results results output format)))

            (when verbose?
              (println "Done!")))))))))

;; 使用例:
;; ./qi tool.qi --verbose --output results.json --format json data1.txt data2.txt

;; ============================================
;; 実用例: 設定のオーバーライド
;; ============================================

(defn load-config []
  (let [parsed (args/parse)
        options (:options parsed)

        ;; デフォルト設定
        default-config {:host "localhost"
                       :port 3000
                       :debug false}

        ;; コマンドライン引数でオーバーライド
        config (default-config
                |> (fn [c] (if (map/has-key? options "host")
                             (assoc c :host (map/get options "host"))
                             c))
                |> (fn [c] (if (map/has-key? options "port")
                             (assoc c :port (parse-int (map/get options "port")))
                             c))
                |> (fn [c] (if (contains? (:flags parsed) "debug")
                             (assoc c :debug true)
                             c)))]
    config))

;; 使用例:
;; ./qi server.qi --host 0.0.0.0 --port 8080 --debug
;; => {:host "0.0.0.0" :port 8080 :debug true}

;; ============================================
;; 実用例: サブコマンド処理
;; ============================================

(defn main []
  (let [subcommand (args/get 2)  ;; 第2引数（プログラム名、スクリプト名の次）
        rest-args (args/all |> (drop 3))]

    (match subcommand
      "init"    -> (cmd-init rest-args)
      "build"   -> (cmd-build rest-args)
      "test"    -> (cmd-test rest-args)
      "deploy"  -> (cmd-deploy rest-args)
      _         -> (println "Unknown command. Use: init, build, test, or deploy"))))

;; 使用例:
;; ./qi cli.qi init myproject
;; ./qi cli.qi build --release
;; ./qi cli.qi test --verbose
```

**設計方針**:
- GNU形式の引数解析をサポート（--long, -short）
- シンプルな位置引数アクセスから高度なパースまで対応
- Flow-oriented設計でパイプラインと組み合わせ可能
- CLIツールとサーバーアプリケーション両方で使用可能

#### ✅ time - 日付・時刻（25個）（実装済み）

**設計方針**: ISO 8601準拠。Flow-orientedな変換・操作。

```lisp
(use time :only [
  ;; 🔥 最優先（現在時刻取得）
  now                         ;; 現在時刻（Unixタイムスタンプ）
  now-iso                     ;; 現在時刻（ISO 8601文字列）
  today                       ;; 今日の日付（YYYY-MM-DD）

  ;; 生成・パース
  from-unix                   ;; Unixタイムスタンプから
  from-iso                    ;; ISO文字列から
  parse                       ;; 文字列をパース（フォーマット指定）

  ;; フォーマット
  format                      ;; カスタムフォーマット
  to-iso                      ;; ISO 8601文字列に
  to-unix                     ;; Unixタイムスタンプに

  ;; 要素アクセス
  year month day              ;; 年月日
  hour minute second          ;; 時分秒
  weekday                     ;; 曜日（0=日曜）

  ;; 演算
  add-days add-hours add-minutes
  sub-days sub-hours sub-minutes
  diff-days diff-hours diff-minutes

  ;; 比較
  before? after? between?

  ;; ユーティリティ
  start-of-day end-of-day
  start-of-month end-of-month
  weekend? leap-year?

  ;; タイムゾーン
  to-utc to-local
  timezone
])

;; 使用例 - Flow-oriented設計
;; 1. 現在時刻の取得
(time/now)       ;; 1736553600 (Unixタイムスタンプ)
(time/now-iso)   ;; "2025-01-11T03:00:00Z"
(time/today)     ;; "2025-01-11"

;; 2. パイプラインで変換
(time/now
 |> time/from-unix
 |> (fn [t] (time/add-days t 7))    ;; 7日後
 |> time/to-iso)
;; "2025-01-18T03:00:00Z"

;; 3. パースとフォーマット
(defn format-date [date-str]
  (date-str
   |> (fn [s] (time/parse s "%Y-%m-%d"))
   |> (fn [t] (time/format t "%B %d, %Y"))))

(format-date "2025-01-11")  ;; "January 11, 2025"

;; 4. 実用例：期限チェック
(defn is-expired? [expires-at]
  (time/before? expires-at (time/now)))

(def session {:created-at (time/now)
              :expires-at (time/add-hours (time/now) 24)})

(is-expired? (:expires-at session))  ;; false

;; 5. データ集計（パイプライン）
(logs
 |> (filter (fn [log]
      (time/between? (:timestamp log)
                     (time/today)
                     (time/now))))
 |> (map (fn [log] {:date (time/format (:timestamp log) "%Y-%m-%d")
                    :level (:level log)}))
 |> (group-by :date))

;; 6. 営業日計算（カスタム関数）
(defn add-business-days [date n]
  (loop [current date remaining n]
    (if (<= remaining 0)
      current
      (let [next-day (time/add-days current 1)]
        (if (time/weekend? next-day)
          (recur next-day remaining)
          (recur next-day (dec remaining)))))))

;; 7. 相対時間表示（SNS的）
(defn relative-time [timestamp]
  (let [diff (time/diff-minutes timestamp (time/now))]
    (match diff
      n when (< n 60) -> f"{n}分前"
      n when (< n 1440) -> f"{(/ n 60)}時間前"
      n -> f"{(/ n 1440)}日前")))
```

**実装優先度**:
- Phase 1: now, now-iso, from-unix, to-iso, format（基本的な取得と変換）
- Phase 2: add-*, diff-*（演算）
- Phase 3: parse, before?, after?（パースと比較）
- Phase 4: タイムゾーン対応

**設計メモ**:
- 内部表現はUnixタイムスタンプ（i64）
- ISO 8601文字列との相互変換を重視
- Flow-orientedなので、パイプラインで変換しやすく
- タイムゾーンはデフォルトUTC、必要に応じてローカルに変換

#### 🚧 その他（全て未実装）
```lisp
http      ;; HTTPクライアント
json      ;; JSONパース
db        ;; データベース
io        ;; ファイルIO拡張
test      ;; テストフレームワーク
```

## 11. 文字列リテラル

### ✅ 基本（実装済み）
```lisp
"hello"
"hello\nworld"
"say \"hello\""
```

### ✅ 複数行（実装済み）

Python風の`"""`を使った複数行文字列をサポートしています。

```lisp
;; 基本的な複数行文字列
"""
This is a
multi-line
string
"""

;; エスケープシーケンスも利用可能
"""Line 1\nLine 2\nLine 3"""

;; SQLクエリなどに便利
(def query """
  SELECT name, age
  FROM users
  WHERE age >= 18
  ORDER BY name
""")

;; JSONやHTML、マークダウンの埋め込みに便利
(def html """
<!DOCTYPE html>
<html>
  <body>
    <h1>Hello, World!</h1>
  </body>
</html>
""")
```

### ✅ 複数行f-string（実装済み）

f-stringでも複数行が使えます。`f"""..."""`の形式です。

```lisp
;; 変数を含む複数行文字列
(def name "Alice")
(def age 30)

f"""
Name: {name}
Age: {age}
Status: Active
"""

;; テンプレートエンジンのように使える
(defn gen-email [user]
  f"""
  Dear {(:name user)},

  Your order #{(:order-id user)} has been confirmed.
  Total: ${(:total user)}

  Thank you for your purchase!
  """
)

(gen-email {:name "Bob" :order-id 12345 :total 99.99})
;; => メール本文が生成される
```

### ✅ 補間（f-string）（実装済み）

f-stringは`f"...{expr}..."`の形式で、`{}`内に変数や式を埋め込むことができます。

```lisp
;; 基本的な使い方
f"Hello, World!"  ;; => "Hello, World!"

;; 変数の補間
(def name "Alice")
f"Hello, {name}!"  ;; => "Hello, Alice!"

;; 式も使える
f"Result: {(+ 1 2)}"  ;; => "Result: 3"

;; リストやベクタの補間
f"List: {[1 2 3]}"  ;; => "List: [1 2 3]"

;; マップアクセス（getまたはキーワード関数）
(def user {:name "Bob" :age 30})
f"Name: {(get user :name)}, Age: {(get user :age)}"
;; => "Name: Bob, Age: 30"

;; キーワードを関数として使う（より簡潔）
f"Name: {(:name user)}, Age: {(:age user)}"
;; => "Name: Bob, Age: 30"

;; エスケープ
f"Escaped: \{not interpolated\}"  ;; => "Escaped: {not interpolated}"

;; ネスト可能（文字列関数と組み合わせ）
(def items ["apple" "banana" "cherry"])
f"Items: {(join \", \" items)}"  ;; => "Items: apple, banana, cherry"

;; 実用例（キーワード関数を使った簡潔な記述）
(defn greet [user]
  f"Welcome, {(:name user)}! You have {(:messages user)} new messages.")

(greet {:name "Alice" :messages 3})
;; => "Welcome, Alice! You have 3 new messages."
```

**対応する値の型**:
- 文字列: そのまま埋め込み
- 数値（整数・浮動小数点）: 文字列に変換
- bool/nil: "true"/"false"/"nil"に変換
- キーワード: `:keyword`形式で埋め込み
- リスト/ベクタ/マップ: 表示形式で埋め込み
- 関数: `<function>`または`<native-fn:name>`に変換
```

## 12. 実用例

### Webスクレイパー
```lisp
(use http :only [get])

(defn scrape-prices [url]
  (match (try
    (url
     |> get
     |> parse-html
     |> (select ".price")
     |> (pmap extract-number)))
    {:ok prices} -> prices
    {:error e} -> (do (log e) [])))

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

(defn clean-csv [file]
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
   |> (csv/write-file "cleaned.csv")))
```

### ログ解析
```lisp
(use regex :as re)
(use str :as s)

(defn parse-log [line]
  (match (re/match line #"^\[(?<level>\w+)\] (?<time>[\d:]+) - (?<msg>.+)$")
    {:groups {:level l :time t :msg m}} -> {:level l :time t :msg m}
    _ -> nil))

(defn analyze-logs [file]
  (file
   |> slurp
   |> s/lines
   |> (map parse-log)
   |> (filter (fn [x] (not (= x nil))))
   |> (filter (fn [x] (= (:level x) "ERROR")))
   |> (group-by :msg)
   |> (map (fn [[msg entries]] {:msg msg :count (len entries)}))
   |> (sort-by :count)
   |> reverse))
```

### チャットサーバー
```lisp
(def clients (atom #{}))

(defn broadcast [msg]
  (pmap (fn [c] (send c msg)) @clients))

(defn handle-client [conn]
  (do
    (swap! clients conj conn)
    (defer (swap! clients dissoc conn))
    (go
      (loop [running true]
        (if running
          (match (recv conn)
            {:msg m} -> (do (broadcast m) (recur true))
            :close -> (recur false))
          nil)))))

(listen 8080 |> (map handle-client))
```

### データパイプライン
```lisp
(use str :as s)
(use csv)

(defn process-logs [file]
  (match (try
    (file
     |> csv/parse-file
     |> (filter (fn [e] (= (:level e) "ERROR")))
     |> (group-by :service)
     |> (map (fn [[k v]] {:service k :count (len v)}))
     |> (sort-by :count)
     |> reverse))
    {:ok data} -> data
    {:error e} -> []))

(def results
  (dir-files "logs/*.csv")
  |> (pmap process-logs)
  |> flatten)

(csv/write-file "report.csv" results)
```

### URL構築
```lisp
(use str :as s)

(defn build-url [base path params]
  (let [query (params
               |> (map (fn [[k v]] f"{k}={(s/url-encode v)}"))
               |> (s/join "&"))]
    f"{base}/{path}?{query}"))

(build-url "https://api.example.com" "search"
           {:q "hello world" :limit 10})
;; "https://api.example.com/search?q=hello%20world&limit=10"
```

### テキスト処理
```lisp
(use str :as s)
(use regex :as re)

(defn clean-text [text]
  (text
   |> (re/replace-all #"\s+" " ")
   |> s/trim
   |> (s/truncate 1000)))

(defn extract-emails [text]
  (re/match-all text #"[^@\s]+@[^@\s]+\.[^@\s]+")
  |> (map :matched))

(defn word-frequency [text]
  (text
   |> s/lower
   |> s/words
   |> (group-by identity)
   |> (map (fn [[word instances]] {:word word :count (len instances)}))
   |> (sort-by :count)
   |> reverse))
```

## 13. 言語文化

### 命名規則
- **関数名**: 短く直感的（`len`, `trim`, `split`）
- **モジュール名**: 短く明確（`http`, `json`, `csv`, `regex`）
- **述語関数**: `?` で終わる（`empty?`, `valid?`）
- **破壊的操作**: `!` で終わる（`swap!`, `reset!`）

### コーディングスタイル

#### 簡潔性と可読性
Qiは「短く、わかりやすく、美しいコード」を目指します。

**基本原則**:
- **簡潔に書く**: 冗長なコードを避け、必要最小限の記述で目的を達成
- **わかりやすく書く**: 他人が読んでも理解しやすいコード
- **短い名前を使う**: 必要以上に長い関数名・変数名は避ける（スコープが短ければ短い名前でOK）
- **defnを優先**: 関数定義には `defn` を使う（`def` + `fn` より簡潔）

**例**:
```lisp
;; ❌ 冗長
(def calculate-sum-of-numbers (fn [list-of-numbers]
  (let [result (reduce (fn [accumulator current-value]
    (+ accumulator current-value)) 0 list-of-numbers)]
    result)))

;; ✅ 簡潔
(defn sum [nums]
  (reduce + 0 nums))
```

#### Flow First - データの流れを第一に

**データの流れを第一に考える**:
- パイプライン `|>` / `||>` / `tap>` を積極的に使う
- 左から右、上から下に読める流れを作る
- 小さな変換を組み合わせて大きな処理を構成

**適切なツールを選ぶ**:
- 単純な分岐は `if`、複雑なパターンは `match`
- `match` で構造を分解し、`:as` で全体を保持、`=> 変換` で流れを継続
- `loop`/`recur` で末尾再帰最適化
- `defer` でリソース管理（エラー時も実行される）
- 回復可能なエラーは `{:ok/:error}`、致命的なエラーは `error`

**モダンな機能を活用**:
- ✅ f-string `f"..."` で文字列補間（実装済み）
- マクロでは `uvar` で変数衝突を回避
- ✅ `match` の `:as` と `=> 変換` でmatch内に流れを埋め込む（実装済み）
- ✅ `tap>` でデバッグ・モニタリング（実装済み）
- 🚧 `flow` で複雑な流れを構造化（未実装）

**シンプルに保つ**:
- 短い変数名OK（スコープが短ければ）
- 再利用可能な「小パイプ」を定義
- 一つの関数は一つの責任

#### 並行処理ファースト

Qiは**並行・並列処理を第一級市民**として扱います。

**設計哲学**:
- チャネル（`chan`）とゴルーチン（`go`）がネイティブサポート
- 非同期処理（`async`/`await`）が組み込み
- スレッドセーフなAtom（`@value`）
- 全てのコアデータ構造はイミュータブル

**例**:
```lisp
;; チャネルを使った並行処理
(defn worker [ch]
  (loop []
    (let [data (<! ch)]
      (when data
        (process data)
        (recur)))))

;; 複数ワーカーの起動
(let [ch (chan)]
  (dotimes [i 5]
    (go (worker ch)))
  (>! ch data))
```

#### テストとドキュメント

**テストは必須**:
- 全ての機能にテストを書く
- `test/assert`, `test/assert-eq` を使用
- エッジケースもテスト

**ドキュメントを書く**:
- 関数にはドキュメントコメントを付ける
- 使用例を示す
- 複雑なロジックにはコメントを追加

**例**:
```lisp
(defn factorial [n]
  "nの階乗を計算する
   例: (factorial 5) => 120"
  (if (<= n 1)
    1
    (* n (factorial (dec n)))))

;; テスト
(test/assert-eq (factorial 5) 120)
(test/assert-eq (factorial 0) 1)
```

### 避けるべきこと

**コードの品質**:
- ❌ 長い関数名・モジュール名
- ❌ 冗長な記述（`def` + `fn` より `defn`）
- ❌ 深いネスト（パイプラインを使う）
- ❌ パイプラインを使わない冗長な中間変数
- ❌ テストなしでコードを書く

**設計の問題**:
- ❌ グローバル変数の乱用
- ❌ core関数との名前衝突
- ❌ マクロで固定の変数名を使う（`uvar`を使う）
- ❌ 拡張しづらい設計

**パフォーマンス**:
- ❌ 過度な最適化（まず動くコードを書く）
- ❌ 早すぎる最適化（プロファイル前の最適化）

**並行処理**:
- ❌ スレッドセーフでない共有状態（Atomを使う）
- ❌ ミュータブルなグローバル状態の共有

## 14. コマンドラインツール

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

## 15. メモリ管理と実行モデル ✅

### メモリ管理

Qiは**GC（ガベージコレクション）不要**の設計です。Rustの所有権システムと参照カウントで自動的にメモリ管理されます。

#### メモリ管理の仕組み

**基本原則:**
- **Arc（Atomic Reference Counting）**: スレッドセーフな参照カウント
  - `Function`, `Macro`, `Atom`, `Channel`, `Scope`, `Stream` に使用
  - 参照カウントが0になると自動的にメモリ解放
- **RwLock**: 読み書きロックでスレッドセーフな可変性を実現
- **値のコピー**: イミュータブルな値は必要に応じてクローン

**循環参照の防止:**
```rust
// 関数作成時に環境をクローンすることで循環参照を防ぐ
(fn [x] (+ x 1))  ; 環境全体がクローンされる
```

関数やクロージャ作成時に環境全体をクローンするため、循環参照は発生しません。

#### メモリ効率

**利点:**
- ✅ メモリリークなし（循環参照がない）
- ✅ スレッドセーフ（Arc + RwLock）
- ✅ 予測可能なメモリ解放（参照カウント）
- ✅ GCの停止時間なし

**トレードオフ:**
- 環境クローンのコスト（関数作成時）
- コレクションのクローンコスト（リスト、ベクタ、マップ）
- 関数型言語としては一般的なトレードオフ

#### 長時間実行サービスでの注意点

APIサーバーやHTTPサーバーなど、長時間実行するサービスを構築する場合の推奨事項:

**メモリ管理のベストプラクティス:**
```lisp
; ✅ リクエストごとにクリーンな環境を使う
(defn handle-request [req]
  (let [session (new-session req)]
    ;; 処理後、環境は自動的に解放される
    (process session)))

; ✅ 共有データはAtomで管理
(def cache (atom {}))

; ✅ 長時間保持するデータはArcで共有される
(def config (load-config))  ; 自動的にArcでラップ

; ⚠️ グローバルな状態の蓄積に注意
(def global-log (atom []))  ; 無限に増える可能性
```

**推奨設計:**
- リクエストごとにクリーンな環境を使用
- グローバル状態は最小限に抑える
- 定期的なクリーンアップ（TTL、メモリ上限など）
- ストリーミング処理で全データを保持しない

#### 将来の最適化案

現在の実装は小〜中規模アプリケーションに最適化されています。大規模・高負荷なサービスを構築する場合、以下の最適化を検討できます:

**1. Copy-on-Write (CoW):**
```lisp
; データをArcで共有し、変更時のみコピー
```

**2. 構造共有（Persistent Data Structures）:**
```lisp
; Clojureスタイルの永続データ構造
; 変更時も既存データを共有
```

**3. 環境の最適化:**
```lisp
; 環境全体ではなく、使用する変数のみキャプチャ
```

### スレッドセーフ性

Qiの並列・並行処理はスレッドセーフに設計されています:

**スレッドセーフな要素:**
- ✅ Evaluator: 複数スレッドから安全に使用可能
- ✅ 環境（Env）: RwLockで保護
- ✅ Atom: スレッドセーフな可変状態
- ✅ Channel: crossbeam-channelによるスレッドセーフ通信

**並列処理の例:**
```lisp
; pmapは自動的に並列実行（スレッドセーフ）
(pmap expensive-computation large-dataset)

; go/chanも完全にスレッドセーフ
(let [ch (chan)]
  (go (send ch 42))
  (recv ch))
```

### パフォーマンス特性

**起動時間**: 高速（Rustネイティブバイナリ）
**実行速度**: 中速〜高速（将来的にJITコンパイル予定）
**メモリ使用量**: 効率的（参照カウント、無駄なコピーなし）
**並列性能**: 優秀（スレッドセーフ設計、Rust並行処理基盤）

## 16. モジュール構成とビルド戦略 ✅

### 基本コンセプト

Qiは **「全部入り + Lazy Init + カスタムビルド可能」** の方針を採用します。

**設計思想**:
- ✅ **デフォルトは全機能有効（Pure Rustのみ）** - 「この環境では動かない」を防ぐ
- ✅ **Lazy Initialization** - 未使用機能はメモリ消費ゼロ
- ✅ **カスタムビルド可能** - 用途に応じてサイズ最適化
- ✅ **Pure Rust優先** - C依存を避け、クロスコンパイル容易に

### なぜ「全部入り」か？

Qiは**ライブラリではなく言語処理系**です：

| 種別 | feature戦略 | 理由 |
|------|-------------|------|
| **ライブラリ** | 細かく分割 | 依存する側が必要な機能だけ選ぶ |
| **言語処理系** | デフォルト全機能 | 「動かない」は最悪のUX |

参考: Python, Ruby, Node.js, Deno等は全部入り単一バイナリを配布。

### Feature階層構造

#### Tier 1: Core（オフ不可）

```rust
// 言語機能
parser, evaluator, value
def, defn, let, do, if, match, try

// 基本演算・データ構造
+, -, *, /, =, <, >
list, vector, map

// 並行処理基盤（Qiの核心）
go, chan, send!, recv!, close!
```

**依存クレート（必須）**:
- `parking_lot` - 高速Mutex/RwLock
- `crossbeam-channel` - go/chan実装
- `rayon` - pmap並列処理
- `regex` - 言語機能（文字列パターン）
- `once_cell` - Lazy Init基盤

#### Tier 2: Default ON（Pure Rust、オプションでOFF可能）

**データベース（Lazy Init）**:
```toml
db-sqlite    = ["rusqlite"]        # 組み込みDB、デフォルト推奨
db-postgres  = ["tokio-postgres"]  # Pure Rust PostgreSQL
db-mysql     = ["mysql_async"]     # Pure Rust MySQL
```

**Web通信（Lazy Init）**:
```toml
http-client  = ["reqwest"]         # HTTP client（rustls使用）
http-server  = ["hyper", "tokio"]  # HTTP server（async runtime）
```

**データフォーマット**:
```toml
format-json  = ["serde_json"]      # JSON（必須級）
format-csv   = []                   # CSV（自前実装、Pure Rust）
```

**文字列処理**:
```toml
string-encoding = ["base64", "urlencoding", "html-escape"]  # Web頻出
string-crypto   = ["sha2", "uuid"]                          # ハッシュ・UUID
encoding-extended = ["encoding_rs"]  # Shift_JIS等（サイズ中）
```

**ファイル・I/O**:
```toml
io-glob      = ["glob"]            # パターンマッチング
io-temp      = ["tempfile"]        # 一時ファイル
util-zip     = ["zip", "flate2"]   # 圧縮・解凍
```

**標準ライブラリ拡張**:
```toml
std-time     = ["chrono"]          # 日時処理（タイムゾーン含む）
std-math     = ["rand"]            # 乱数生成
std-stats    = []                   # 統計関数（自前実装）
std-set      = []                   # 集合演算（自前実装）
```

**開発支援**:
```toml
repl         = ["rustyline", "dirs"]  # 対話環境
dev-tools    = []                      # profile, test, dbg（自前実装）
```

#### Tier 3: Optional（デフォルトOFF、C依存等）

```toml
db-odbc      = ["odbc-api"]        # システムODBCドライバ依存
db-duckdb    = ["duckdb"]          # C++依存、サイズ巨大（~50MB）
```

### Cargo.toml構成例

```toml
[features]
# デフォルト: Pure Rust全部入り
default = [
    "db-sqlite", "db-postgres", "db-mysql",
    "http-client", "http-server",
    "format-json", "format-csv",
    "string-encoding", "string-crypto", "encoding-extended",
    "io-glob", "io-temp", "util-zip",
    "std-time", "std-math", "std-stats", "std-set",
    "repl", "dev-tools",
]

# プリセット構成
minimal       = []                    # 最小構成（組み込み・WASM用）
web-server    = ["http-server", "format-json", "db-sqlite"]
cli-tool      = ["repl", "format-json", "io-glob", "util-zip"]
data-processing = ["db-sqlite", "db-postgres", "format-json", "format-csv"]

# C依存含むフル機能
full = ["default", "db-odbc", "db-duckdb"]

# 個別機能
db-sqlite = ["dep:rusqlite"]
db-postgres = ["dep:tokio-postgres", "dep:postgres-types"]
# ... 以下略
```

### ビルド例

```bash
# デフォルト（Pure Rust全部入り）
cargo build --release
# サイズ: 15-20MB

# 最小構成
cargo build --release --no-default-features --features minimal
# サイズ: 3-5MB

# Webサーバー専用
cargo build --release --no-default-features --features web-server
# サイズ: 8-10MB

# データ処理専用
cargo build --release --no-default-features --features data-processing
# サイズ: 10-12MB

# フル機能（C依存含む）
cargo build --release --features full
# サイズ: 70-100MB（DuckDB含む）
```

### Lazy Initialization戦略

**コンセプト**: 「コードは持っているが、必要になるまで起動しない」

#### 即時初期化（軽量）
- Core言語機能
- 基本数学関数（math/*）
- 集合演算（set/*）
- 統計関数（stats/*）

#### Lazy Init必須（重量）
- **DBコネクションプール**: 初回 `db/connect` 時に初期化
- **HTTP client**: 初回リクエスト時にクライアント構築
- **HTTP server runtime**: `server/serve` 呼び出し時にtokio起動
- **正規表現キャッシュ**: 初回使用時にコンパイル

#### 実装例

```rust
// HTTP client のLazy Init
use once_cell::sync::Lazy;

static HTTP_CLIENT: Lazy<reqwest::blocking::Client> = Lazy::new(|| {
    reqwest::blocking::Client::builder()
        .user_agent("qi-lang/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
});

// 初回アクセス時のみ初期化、以降はキャッシュ使用
pub fn http_get(url: &str) -> Result<Value> {
    HTTP_CLIENT.get(url).send()  // 初回ここで初期化
}
```

```rust
// Server runtime のLazy Init
use once_cell::sync::OnceCell;

static SERVER_RUNTIME: OnceCell<tokio::runtime::Runtime> = OnceCell::new();

pub fn serve(...) -> Result<Value> {
    // server/serve が初めて呼ばれた時だけランタイム起動
    let rt = SERVER_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    });
    // サーバー起動...
}
```

### メモリフットプリント

| 状態 | メモリ使用量 | 説明 |
|------|--------------|------|
| 起動直後 | 5-10MB | Core + 基本関数のみ |
| DB使用時 | +10-20MB | コネクションプール確保 |
| Server起動時 | +5-10MB | tokioランタイム起動 |
| HTTP client使用時 | +2-5MB | reqwestクライアント |
| 未使用機能 | 0MB | コードはあるがメモリ取らない |

### エラーメッセージ（feature無効時）

feature無効化でビルドした場合、実行時に分かりやすいエラーを表示：

```lisp
(db/connect "postgres://localhost/db")
; エラー: PostgreSQL サポートは無効化されています。
; feature 'db-postgres' を有効にしてビルドしてください:
; cargo build --features db-postgres
```

### 配布戦略

| バージョン | 用途 | サイズ | ビルド方法 |
|------------|------|--------|------------|
| **Qi Standard** | 通常配布版 | 15-20MB | `cargo build --release` |
| **Qi Minimal** | 組み込み・WASM | 3-5MB | `--no-default-features --features minimal` |
| **Qi Full** | 全機能（C依存含む） | 70-100MB | `--features full` |

### 実装状況

| 項目 | 状態 |
|------|------|
| Feature構造設計 | ✅ 完了 |
| Core実装 | ✅ 完了 |
| Lazy Init基盤 | 🚧 実装予定 |
| PostgreSQL driver | 🚧 実装予定 |
| MySQL driver | 🚧 実装予定 |
| Feature-gated modules | 🚧 実装予定 |

### 参考ドキュメント

詳細設計は `BUILD-IDEA.md` を参照してください。

---

## まとめ

**名前**: Qi - A Lisp that flows

**哲学**: Flow-Oriented Programming - データの流れを設計する言語

---

### コア関数の実装優先度

Qiの**Flow-oriented**哲学と実用性を考慮した実装優先順位：

#### 🔥 フェーズ1完了 - 次はフェーズ2へ

**✅ 完了した機能**:

**1. ネスト操作** - JSON/Web処理の核心
```lisp
update update-in get-in assoc-in dissoc-in
```

**2. 関数型基礎** - 高階関数を書くための標準ツール
```lisp
identity constantly comp apply partial
```

**3. 集合演算** - データ分析・フィルタリング
```lisp
union intersect difference subset?
```

**4. 数値基本** - 計算の基礎
```lisp
pow sqrt round floor ceil clamp rand rand-int
```

#### ⚡ 高優先（コアを充実させる）

**5. ソート・集約拡張**
```lisp
sort-by frequencies count-by
```
理由: データ分析で頻出。`group-by`との相性良い。

**6. リスト分割**
```lisp
chunk take-while drop-while
```
理由: バッチ処理・ストリーム処理で便利。

**7. I/O拡張**
```lisp
println read-lines file-exists?
```
理由: ユーザビリティ向上。

#### 🎯 中優先（必要になったら）

**8. 集約関数**
```lisp
max-by min-by sum-by
```

**9. 高階関数拡張**
```lisp
partial complement juxt
```

**10. 統計**
```lisp
mean median stddev
```

---

### コア実装状況

**✅ 完全実装**:
- **特殊形式**: `def` `fn` `let` `do` `if` `match` `try` `defer`（8つ）
- **パイプライン**: `|>` 逐次パイプ、`||>` 並列パイプ
- **Flow制御**: `tap>` 副作用タップ（関数として）
- **ループ**: `loop` `recur` 末尾再帰最適化
- **エラー処理**: `try` `error` `defer`
- **マクロシステム**: `mac` `quasiquote` `unquote` `unquote-splice` `uvar` `variable` `macro?` `eval`
- **状態管理**: `atom` `@` `deref` `swap!` `reset!`（スレッドセーフ）
- **並列処理**: `pmap` `pfilter` `preduce` `parallel-do`（rayon使用、完全並列化済み）
- **スレッド安全**: Evaluator完全スレッドセーフ化（Arc<RwLock<_>>）
- **並行処理 Layer 1**: `go` `chan` `send!` `recv!` `recv!:timeout` `try-recv!` `close!` `select!` `make-scope` `scope-go` `cancel!` `cancelled?` `with-scope`
- **並行処理 Layer 2**: `pmap` `pfilter` `preduce` `parallel-do` `pipeline` `pipeline-map` `pipeline-filter` `fan-out` `fan-in`
- **並行処理 Layer 3**: `await` `then` `catch` `all` `race`
- **遅延評価（Stream）**: `stream` `range-stream` `repeat` `cycle` `iterate` `stream-map` `stream-filter` `stream-take` `stream-drop` `realize` `file-stream` `http/get-stream` `http/post-stream` `http/request-stream`
- **データ型**: nil, bool, 整数, 浮動小数点, 文字列, シンボル, キーワード, リスト, ベクタ, マップ, 関数, アトム, チャネル, スコープ, Stream, Uvar
- **文字列**: f-string補間
- **モジュール**: 基本機能（`module`/`export`/`use :only`/`:all`）
- **名前空間**: Lisp-1、coreが優先
- **ネスト操作**: `update` `update-in` `get-in` `assoc-in` `dissoc-in`
- **関数型基礎**: `identity` `constantly` `comp` `apply` `partial`
- **集合演算**: `union` `intersect` `difference` `subset?`
- **数学関数**: `pow` `sqrt` `round` `floor` `ceil` `clamp` `rand` `rand-int`

**✅ match拡張** ⭐ **Qi独自の差別化機能** - **実装済み**:
- `:as` 束縛（部分と全体を両方使える）
- `=> 変換`（マッチ時にパイプライン的変換） - **他のLispにない独自機能**

**🔜 近未来（Flow強化）**:

*パイプライン拡張*:
- `flow` DSL（分岐・合流を含む構造化パイプライン）

*match拡張（追加予定）*:
- `or` パターン（複数パターンで同じ処理）
- 配列の複数束縛（`[x y]` で同時束縛）

*Stream I/O拡張*:
- ✅ `file-stream`（io.rs）ファイルストリーミング **実装済み**
- ✅ `http/get-stream` `http/post-stream` `http/request-stream`（http.rs）HTTPストリーミング **実装済み**
- 🚧 `tail-stream`（リアルタイムログ監視）**将来実装**

**🚧 将来**:
- 標準モジュール群（str/csv/regex/http/json）

### 実装状況サマリー

#### ✅ 実装済み（v0.1.0）

**特殊形式（8つ）**: `def` `fn` `let` `do` `if` `match` `try` `defer`

**パイプライン演算子**: `|>` 逐次、`||>` 並列、`tap>` タップ

**組み込み関数（150個以上）**:
- **リスト操作（26）**: map, filter, reduce, first, rest, last, take, drop, concat, flatten, range, reverse, nth, zip, sort, sort-by, distinct, partition, group-by, frequencies, count-by, chunk, take-while, drop-while, max-by, min-by, sum-by
- **数値演算（11）**: +, -, *, /, %, abs, min, max, inc, dec, sum
- **比較（6）**: =, !=, <, >, <=, >=
- **マップ操作（12）**: get, keys, vals, assoc, dissoc, merge, select-keys, update, update-in, get-in, assoc-in, dissoc-in
- **文字列（6 core + 60+ str）**: str, split, join, upper, lower, trim, map-lines ＋ strモジュールで60以上
- **述語（9）**: nil?, list?, vector?, map?, string?, keyword?, integer?, float?, empty?
- **高階関数（13）**: map, filter, reduce, pmap, partition, group-by, map-lines, identity, constantly, comp, apply, partial, count-by, complement, juxt
- **集合演算（4）**: union, intersect, difference, subset?
- **数学関数（8）**: pow, sqrt, round, floor, ceil, clamp, rand, rand-int
- **状態管理（5）**: atom, @, deref, swap!, reset!
- **並行処理 Layer 1（13）**: go, chan, send!, recv!, recv!:timeout, try-recv!, close!, select!, make-scope, scope-go, cancel!, cancelled?, with-scope
- **並行処理 Layer 2（9）**: pmap, pfilter, preduce, parallel-do, pipeline, pipeline-map, pipeline-filter, fan-out, fan-in
- **並行処理 Layer 3（5）**: await, then, catch, all, race
- **遅延評価 Stream（14）**: stream, range-stream, repeat, cycle, iterate, stream-map, stream-filter, stream-take, stream-drop, realize, file-stream, http/get-stream, http/post-stream, http/request-stream
- **エラー処理（2）**: try, error
- **メタ（7）**: mac, uvar, variable, macro?, eval, quasiquote, unquote
- **論理（3）**: and, or, not
- **I/O（7）**: print, println, read-file, read-lines, write-file, append-file, file-exists?

**データ型**: nil, bool, 整数, 浮動小数点, 文字列, シンボル, キーワード, リスト, ベクタ, マップ, 関数, アトム, チャネル, スコープ, Stream, Uvar

**先進機能**:
- f-string補間
- match拡張（:as束縛、=> 変換） ⭐ Qi独自
- マクロの衛生性（uvar）
- 末尾再帰最適化（loop/recur）
- defer（エラー時も実行保証）
- **遅延評価Stream**（無限データ構造、メモリ効率的処理）
- **3層並行処理アーキテクチャ** ⭐ Qi独自
  - Layer 1: go/chan（Go風基盤）
  - Layer 2: pipeline（構造化並行処理）
  - Layer 3: async/await（モダンAPI）

#### 🔜 次期実装予定（優先度順）

**フェーズ1: コア強化（✅ 完了）**
1. ✅ ネスト操作: update, update-in, get-in, assoc-in, dissoc-in
2. ✅ 関数型基礎: identity, constantly, comp, apply, partial
3. ✅ 集合演算: union, intersect, difference
4. ✅ 数値基本: pow, sqrt, round, floor, ceil, clamp, rand, rand-int

**フェーズ2: 分析・集約（✅ 完了）**
5. ✅ sort-by, frequencies, count-by
6. ✅ chunk, take-while, drop-while
7. ✅ println, read-lines, file-exists?

**フェーズ3: 高度機能（✅ 完了）**
8. ✅ max-by, min-by, sum-by
9. ✅ complement, juxt（partialはフェーズ1で完了）

**フェーズ4: 並行・並列処理（✅ 完了）**
10. ✅ 完全スレッドセーフ化（Arc<RwLock<_>>）
11. ✅ pmapの完全並列化（rayon）
12. ✅ Layer 1: go/chan実装
13. ✅ Layer 2: pipeline実装
14. ✅ Layer 3: async/await実装

**フェーズ4.5: Web開発機能（✅ 完了）**
15. ✅ Railway Pipeline (`|>?`)
16. ✅ JSON/HTTP完全実装
17. ✅ デバッグ関数（inspect, time）
18. ✅ コレクション拡張（find, every?, some?, zipmap, update-keys, update-vals等）

**フェーズ5: 並行・並列処理の完成（✅ 完了）**
19. ✅ 並列コレクション完成（pfilter, preduce）
20. ✅ select!とタイムアウト（recv! :timeout, select!）
21. ✅ Structured Concurrency（make-scope, scope-go, cancel!, cancelled?, with-scope）
22. ✅ parallel-do（複数式の並列実行）

**フェーズ5.5: アプリケーション開発機能（✅ 完了）**
23. ✅ ZIP圧縮・解凍モジュール（zip/create, zip/extract, zip/list, zip/add, zip/gzip, zip/gunzip）
24. ✅ コマンドライン引数パースモジュール（args/all, args/get, args/parse, args/count）

**フェーズ6: 統計・高度な処理**
25. mean, median, stddev

#### 🚧 将来の計画

**APIサーバー・アプリケーション開発機能（優先度高）**:

1. **HTTPサーバー** 🔥
   - ルーティング（GET, POST, PUT, DELETE, PATCH）
   - ミドルウェアシステム（認証、CORS、ロギング、エラーハンドリング）
   - 静的ファイル配信
   - WebSocket対応
   - ストリーミングレスポンス

2. **テストフレームワーク** ⚡
   - `test/deftest` - テスト定義
   - `test/assert`, `test/assert-eq`, `test/assert-throws` - アサーション
   - `test/run-tests` - テスト実行
   - テストカバレッジ計測

3. **データベース接続** 🎯
   - PostgreSQL, MySQL, SQLite対応
   - コネクションプール
   - トランザクション管理
   - ORM機能（オプション）

4. **認証・認可** 🎯
   - JWT（JSON Web Token）
   - セッション管理
   - OAuth2対応
   - パスワードハッシュ（bcrypt, argon2）

5. **ファイル監視** 📁
   - `fs/watch` - ファイル・ディレクトリ監視
   - 変更検知イベント（作成、更新、削除、リネーム）
   - ホットリロード機能

6. **ログ高度機能** 📊
   - ログ出力先指定（ファイル、標準出力、syslog）
   - ログローテーション（サイズ、日付ベース）
   - ログ圧縮（gzip）
   - 非同期ログ出力（パフォーマンス向上）

7. **圧縮・解凍** 🗜️
   - `zip/create` - ZIP作成
   - `zip/extract` - ZIP解凍
   - `zip/list` - ZIP内容一覧
   - `zip/add` - ZIPにファイル追加
   - ストリーミング圧縮・解凍

8. **メトリクス・モニタリング** 📈
   - カウンター、ゲージ、ヒストグラム
   - Prometheus形式エクスポート
   - APM（Application Performance Monitoring）連携

**その他の計画**:
- 標準モジュール群の完全版（✅ str完全版, ✅ csv, ✅ http client, ✅ json, 🚧 regex）
- ✅ 非同期パイプライン演算子（~>）
- ✅ ストリーム処理（stream）
- ✅ 遅延ストリーム（stream）
- flow DSL（構造化パイプライン）

### 実装の方針

**Qiの強み = Flow + Match + Nest**
1. パイプライン（|>, ||>, tap>）でデータの流れを表現
2. match拡張（:as, =>）で複雑な構造を扱う
3. ネスト操作（*-in系）でJSON/Webを直感的に

**実装優先度の基準**:
- Flow哲学との親和性
- Web/JSON処理での実用性
- 実装コストと効果のバランス

---

## このドキュメントについて

### ドキュメントの保守

このSPEC.mdは、Qi言語の**生きたドキュメント**です。実装の変更に合わせて常に更新されます。

**更新ルール**:
- ✅ **実装が変わったら更新**: 新機能追加、既存機能の変更時は必ずSPEC.mdを更新
- ✅ **実装状況を正確に**: ✅（実装済み）、🚧（未実装）、🔜（計画中）のマーカーを正確に記載
- ✅ **例も最新に**: コード例は現在の実装で動作するものを記載
- ✅ **defnを優先**: 関数定義の例は `defn` を使用（def, defnの説明以外）

**関連ドキュメント**:
- `README.md`: プロジェクト概要、インストール方法
- `TUTORIAL.md`: Qi言語とプログラミング言語実装の学習ドキュメント
- `CLAUDE.md`: 開発者向けのソースコードルールとガイドライン

### ドキュメントの構成

1. **言語概要**: Qiの哲学と特徴
2. **基本設計・特殊形式**: 言語の基礎文法
3. **データ構造・コア関数**: 組み込み機能
4. **実用例・言語文化**: ベストプラクティス
5. **実装計画**: 今後の方向性

---

## 将来の改善計画

### ✅ 名前衝突の警告システム（実装済み）

#### 機能概要
`def` で既存の変数、関数、ビルトイン関数を再定義しようとすると、警告を表示します。
エラーではないため、処理は継続されます。

```lisp
;; ビルトイン関数の再定義
(def inc (fn [x] (* x 2)))
;; 警告: ビルトイン関数を再定義しています: 'inc' (inc)

;; 関数の再定義
(def my-fn (fn [x] x))
(def my-fn (fn [x] (* x 2)))
;; 警告: 関数を再定義しています: 'my-fn'

;; 変数の再定義
(def x 10)
(def x 20)
;; 警告: 変数を再定義しています: 'x'
```

#### 実装詳細
- `def` 評価時に既存の束縛をチェック
- ビルトイン関数、ユーザー定義関数、変数を区別して警告
- 英語・日本語の多言語対応
- エラーではなく警告のため、処理は継続（Lisp的自由を尊重）

---

### 名前空間システム 🚧 **Phase 6以降（低優先度）**

現在のQiはグローバル名前空間のみ。大規模開発では名前衝突が問題になる可能性。

**検討事項**:
```lisp
;; 案1: Clojure風
(ns myapp.core)
(def map {...})  ;; myapp.core/map

(myapp.core/map ...)  ;; 自分のmap
(core/map ...)        ;; 組み込みmap

;; 案2: モジュールシステム拡張
(module myapp
  (def map {...}))

(myapp/map ...)
```

**決定**: Phase 1では**やらない**
- 設計思想（シンプル）に反する
- 小〜中規模プロジェクトでは不要
- 必要になったら検討

---

## ドキュメントシステム

Qiは柔軟なドキュメントシステムを提供します。

### 概要

- **多言語対応**（i18n）：現在は日本語（ja）と英語（en）
- **ハイブリッド型**：ソースコード内記述と外部ファイルの両方に対応
- **3つの記述レベル**：シンプルな文字列から詳細な構造化まで
- **遅延読み込み**：メモリ効率を重視し、必要時のみロード

### 記述方法

```lisp
;; 1. 文字列形式（シンプル）
(def greet
  "指定された名前で挨拶する"
  (fn [name]
    (str "Hello, " name "!")))

;; 2. 構造化形式（詳細）
(def greet
  {:desc "指定された名前で挨拶する"
   :params [{:name "name" :type "string" :desc "挨拶する相手の名前"}]
   :returns {:type "string" :desc "挨拶メッセージ"}
   :examples ["(greet \"Alice\") ;=> \"Hello, Alice!\""]}
  (fn [name]
    (str "Hello, " name "!")))

;; 3. 外部参照形式（大規模）
(def complex-function
  :see-ja "docs/ja/complex-function.qi"
  :see-en "docs/en/complex-function.qi"
  (fn [x y z]
    ;; 実装
    ))
```

### ディレクトリ構造

```
project/
  main.qi
  docs/
    ja/my-module.qi
    en/my-module.qi

qi (バイナリ)
std/
  ja/io-module.qi
  en/io-module.qi
```

### 言語フォールバック

1. 現在の言語（環境変数 `QI_LANG`）
2. 英語（`en`）
3. 表示なし

### 詳細仕様

詳細は [DOC_SYSTEM.md](DOC_SYSTEM.md) を参照してください。

---

## REPL（Read-Eval-Print Loop）

QiのREPLは強力な対話的開発環境を提供します。

### ✅ 基本機能

**起動**:
```bash
qi              # REPL起動
qi -l file.qi   # ファイルをロードしてREPL起動
```

**特徴**:
- **履歴**: 上下キーで過去のコマンド履歴（`~/.qi_history`に保存）
- **行編集**: Emacs風キーバインド（Ctrl+A/E/W等）
- **複数行入力**: 括弧が閉じるまで自動継続
- **タブ補完**: 関数名・変数名・REPLコマンドを補完

### ✅ REPLコマンド

すべてのコマンドは `:` で始まります：

```lisp
:help           ; コマンド一覧表示
:vars           ; 定義済み変数一覧
:funcs          ; ユーザー定義関数一覧
:builtins       ; ビルトイン関数（プレースホルダー）
:clear          ; 環境をクリア
:load <file>    ; ファイル読み込み
:reload         ; 最後のファイルを再読み込み
:quit           ; 終了
```

### 使用例

```lisp
qi:1> (defn double [x] (* x 2))
#<function>

qi:2> (double 21)
42

qi:3> :vars
No variables defined

qi:4> :funcs
User-defined functions:
  double

qi:5> :load utils.qi
Loading: utils.qi
Loaded!

qi:6> (double
     ...  21)
42

qi:7> :quit
```

### 複数行入力

括弧が閉じていない場合、自動的に複数行モードになります：

```lisp
qi:1> (defn sum [& nums]
     ...   (reduce + 0 nums))
#<function>

qi:2> (sum 1 2 3 4 5)
15
```

### タブ補完

関数名、変数名、REPLコマンドを補完できます：

```lisp
qi:1> :h<TAB>
:help

qi:2> dou<TAB>
double

qi:3> (defn triple [x] (* x 3))
#<function>

qi:4> tri<TAB>
triple
```

### キーボードショートカット

- **Ctrl+C**: 入力キャンセル（REPL継続）
- **Ctrl+D**: 終了
- **↑/↓**: 履歴移動
- **Ctrl+A**: 行頭
- **Ctrl+E**: 行末
- **Ctrl+W**: 単語削除
- **TAB**: 補完

---

## ビルド構成 - Feature Flags

Qiは**条件付きコンパイル（Feature Flags）**により、用途に応じて必要な機能だけを含めたビルドが可能です。これにより、組み込み環境やWASMなど制約の厳しい環境でも動作します。

### ビルドプリセット

#### `default` - フル機能（デフォルト）

全機能を含むビルド。開発や一般的な用途に最適。

```bash
cargo build
# または
cargo build --features default
```

**含まれる機能**:
- ✅ データベース（SQLite）
- ✅ Web通信（HTTPクライアント・サーバー）
- ✅ データフォーマット（JSON、CSV）
- ✅ 文字列処理（エンコーディング、暗号化）
- ✅ ファイル・I/O（glob、一時ファイル、ZIP）
- ✅ 標準ライブラリ拡張（時刻、数学、統計、集合）
- ✅ REPL・開発ツール

#### `minimal` - 最小構成

最小限の依存関係でビルド。組み込み環境やWASM向け。

```bash
cargo build --no-default-features --features minimal
```

**含まれる機能**:
- ✅ 基本ファイル操作（glob）
- ✅ コア関数（リスト、マップ、文字列基本）
- ✅ 並行・並列処理
- ✅ パイプライン・パターンマッチング

**除外される機能**:
- ❌ データベース（SQLite等）
- ❌ HTTP通信
- ❌ JSON（`parse-json`, `to-json`）
- ❌ 多重エンコーディング（UTF-8のみ）
- ❌ 時刻処理（`time/*`）
- ❌ 乱数生成（`math/rand*`、`math/shuffle`）
- ❌ 統計・集合関数
- ❌ ZIP圧縮、一時ファイル
- ❌ REPL

#### プリセット構成

特定用途に最適化されたプリセット：

```bash
# Webサーバー構成
cargo build --no-default-features --features web-server

# CLIツール構成
cargo build --no-default-features --features cli-tool

# データ処理構成
cargo build --no-default-features --features data-processing
```

| プリセット | 含まれる機能 |
|-----------|-------------|
| `web-server` | HTTPサーバー、JSON、SQLite |
| `cli-tool` | REPL、JSON、glob、ZIP |
| `data-processing` | SQLite、JSON、CSV |

### 個別機能フラグ

カスタムビルドには個別のfeatureフラグを組み合わせます：

```bash
# HTTPクライアントとJSONのみ
cargo build --no-default-features --features http-client,format-json

# データベースと統計のみ
cargo build --no-default-features --features db-sqlite,std-stats
```

#### データベース

| Feature | 関数 | 依存クレート |
|---------|------|-------------|
| `db-sqlite` | `db/connect`, `db/query`, `db/exec`, `db/close` 等（17関数） | rusqlite |

#### Web通信

| Feature | 関数 | 依存クレート |
|---------|------|-------------|
| `http-client` | `http-get`, `http-post`, `http-put`, `http-delete` | reqwest |
| `http-server` | `server/start`, `server/stop`, `server/on` 等（15関数） | hyper, tokio |

#### データフォーマット

| Feature | 関数 | 依存クレート |
|---------|------|-------------|
| `format-json` | `parse-json`, `to-json` | serde_json |
| `format-csv` | `csv/read`, `csv/write` | Pure Rust実装 |

#### 文字列処理

| Feature | 関数 | 依存クレート |
|---------|------|-------------|
| `string-encoding` | `to-base64`, `from-base64`, `url-encode`, `url-decode`, `html-escape`, `html-unescape` | base64, urlencoding, html-escape |
| `string-crypto` | `hash`, `uuid` | sha2, uuid |
| `encoding-extended` | 多重エンコーディング対応（`read-file`/`write-file`でShift_JIS、EUC-JP等） | encoding_rs |

**注**: `encoding-extended`を無効にすると、`read-file`/`write-file`はUTF-8のみ対応になります。

#### ファイル・I/O

| Feature | 関数 | 依存クレート |
|---------|------|-------------|
| `io-glob` | `list-dir`でglobパターン | glob |
| `io-temp` | `temp/file`, `temp/dir` | tempfile |
| `util-zip` | `zip/create`, `zip/extract`, `gzip`, `gunzip` | zip, flate2 |

#### 標準ライブラリ拡張

| Feature | 関数 | 依存クレート |
|---------|------|-------------|
| `std-time` | `time/now`, `time/parse`, `time/format` 等（17関数） | chrono |
| `std-math` | `math/rand`, `math/rand-int`, `math/random-range`, `math/shuffle` | rand |
| `std-stats` | `stats/mean`, `stats/median`, `stats/stddev` 等（6関数） | Pure Rust実装 |
| `std-set` | `set/union`, `set/intersect`, `set/diff` 等（7関数） | Pure Rust実装 |

#### 開発支援

| Feature | 内容 | 依存クレート |
|---------|------|-------------|
| `repl` | インタラクティブREPL | rustyline, dirs |
| `dev-tools` | プロファイラー（`profile/start`, `profile/stop`, `profile/report`） | Pure Rust実装 |

### ビルドサイズ比較

参考値（Release ビルド、macOS arm64）:

| 構成 | バイナリサイズ | 備考 |
|------|--------------|------|
| `default` | 〜15MB | 全機能含む |
| `minimal` | 〜3MB | 組み込み・WASM向け |
| `web-server` | 〜8MB | Webアプリ向け |
| `cli-tool` | 〜5MB | CLIツール向け |

### 機能チェック

プログラム内で機能の有無を確認したい場合：

```lisp
;; 実行時にエラーで判断
(try
  (parse-json "{\"key\": \"value\"}")
  (fn [err] (println "JSON feature not available")))

;; 関数の存在チェック
(if (fn? parse-json)
  (parse-json data)
  (println "JSON not supported"))
```

### 設計思想

Qiの条件付きコンパイルは以下の原則に基づいています：

1. **Pure Rust優先** - C/C++依存のクレートは避け、クロスコンパイルを容易に
2. **段階的導入** - 必要な機能から順に有効化
3. **明確なエラー** - 無効な機能を呼ぶと分かりやすいエラーメッセージ
4. **スレッドセーフ** - 全てのビルド構成でスレッドセーフを保証

---
