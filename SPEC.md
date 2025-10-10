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
   - **Simple**: 特殊形式8つ、記法最小限、学習曲線が緩やか
   - **Fast**: 軽量・高速起動・将来的にJITコンパイル
   - **Concise**: 短い関数名、パイプライン、関数型で表現力豊か

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

**フェーズ2（🔜 近未来）**:

*パイプライン強化*:
- `||>` 並列パイプライン - 自動的にpmap化
- `tap>` 副作用タップ - デバッグ・ログ観察
- `flow` DSL - 分岐・合流を含む複雑な流れ

*match強化* ⭐ **Qi独自の差別化要素**:
- `:as` 束縛 - 部分と全体を両方使える
- `=> 変換` - マッチ時にパイプライン的変換（matchの中に流れを埋め込む）
- `or` パターン - 複数パターンで同じ処理

**フェーズ3（🔜 進行中）**:
- ✅ 並列処理基盤 - スレッドセーフEvaluator、pmap完全並列化
- 🚧 並行処理 - go/chan、パイプライン、async/await
- 🚧 `~>` 非同期パイプライン - go/chan統合
- 🚧 `stream` 遅延評価ストリーム - 巨大データ処理
- 再利用可能な「小パイプ」文化の確立

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
| `||>` | 並列パイプ | ✅ 実装済み | 自動的にpmap化、リスト処理の並列化 |
| `tap>` | 副作用タップ | ✅ 実装済み | デバッグ、ログ、モニタリング（関数として） |
| `~>` | 非同期パイプ | 🚧 将来 | go/chan統合、非同期IO |

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

### 🚧 `~>` 非同期パイプライン（将来）

**並行処理との統合**

```lisp
;; 非同期HTTP
(urls ~> http-get-async ~> parse-json ~> process)

;; チャネルとの統合
(chan ~> (filter valid?) ~> process ~> output-chan)

;; go ブロック
(go
  (data ~> transform ~> (send output-chan _)))
```

---

### 🚧 `stream` 遅延評価（将来）

**巨大データの効率的処理**

```lisp
;; 大きなファイル
(files "*.log"
  |> stream
  |> (filter error-line?)
  |> (map parse)
  |> take 100
  |> print)

;; 無限ストリーム
(integers-from 0
  |> stream
  |> (filter prime?)
  |> take 10)

;; メモリ効率
(huge-csv
  |> stream-lines
  |> (map parse-csv)
  |> (filter valid?)
  |> write-output)
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

## 3. 特殊形式（8つ）✅

### ✅ `def` - グローバル定義
```lisp
(def x 42)
(def greet (fn [name] (str "Hello, " name)))
(def ops [+ - * /])
```

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

#### 🔜 将来の拡張パターン

**3. `or` パターン - 複数パターンで同じ処理** 🔜
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

## 5. コア関数

Qiの組み込み関数は**Flow-oriented**哲学に基づき、データの流れと変換を重視した設計になっています。

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
zip                     ;; 組み合わせ: (zip [1 2] [:a :b]) => ([1 :a] [2 :b])
```

#### 高階関数（✅ 実装済み）
```lisp
map filter reduce       ;; 基本の高階関数
pmap                    ;; 並列map（現在はシングルスレッド実装）
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

#### 集合演算（🔜 計画中）
```lisp
;; 🔜 優先度: 高
union                   ;; 和集合: (union [1 2] [2 3]) => [1 2 3]
intersect               ;; 積集合: (intersect [1 2 3] [2 3 4]) => [2 3]
difference              ;; 差集合: (difference [1 2 3] [2]) => [1 3]

;; 🔜 優先度: 低
subset? superset?       ;; 集合判定
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

#### 統計（🔜 計画中）
```lisp
;; 🔜 優先度: 中
mean median mode        ;; 平均、中央値、最頻値
stddev variance         ;; 標準偏差、分散
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

**使用例**:
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

**設計メモ**: ネスト操作はQiの強み。JSONやWeb APIレスポンスの処理が直感的になる。

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
;; チャネル作成
(chan)                  ;; 無制限バッファ
(chan 10)               ;; バッファサイズ10

;; 送受信
(send! ch value)        ;; チャネルに送信
(recv! ch)              ;; ブロッキング受信
(try-recv! ch)          ;; 非ブロッキング受信（nilまたは値）
(close! ch)             ;; チャネルクローズ

;; goroutine風
(go (println "async!"))
(go (send! ch (expensive-calc)))

;; futureとしても使える
(def result (go (expensive-calc)))
(deref result)          ;; 結果待ち
```

**Layer 2: Pipeline（構造化並行処理）** - 関数型スタイル ✅
```lisp
;; パイプライン処理
(pipeline n xf ch)      ;; n並列でxf変換をchに適用

;; ファンアウト/ファンイン
(fan-out ch n)          ;; 1つのチャネルをn個に分岐
(fan-in chs)            ;; 複数チャネルを1つに合流

;; データパイプライン
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

**実装済み関数一覧**:
- `await`: Promiseを待機
- `then`: Promiseチェーン
- `catch`: エラーハンドリング
- `all`: 複数Promiseを並列実行
- `race`: 最速のPromiseを返す

#### 実装技術スタック

- **crossbeam-channel**: Go風チャネル実装
- **rayon**: データ並列（pmap等）
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

(def handle-request (fn [req]
  (do
    (swap! request-count inc)
    (process req))))

;; 現在のカウント確認
(deref request-count)  ;; 処理したリクエスト数
```

#### 実用例2: 状態を持つキャッシュ

```lisp
;; キャッシュ
(def cache (atom {}))

(def get-or-fetch (fn [key fetch-fn]
  (let [cached (get (deref cache) key)]
    (if cached
      cached
      (let [value (fetch-fn)]
        (do
          (swap! cache assoc key value)
          value))))))

;; 使用例
(get-or-fetch :user-123 (fn [] (fetch-from-db :user-123)))
```

#### 実用例3: 接続管理（deferと組み合わせ）

```lisp
;; アクティブな接続を管理
(def clients (atom #{}))

(def handle-connection (fn [conn]
  (do
    (swap! clients conj conn)
    (defer (swap! clients dissoc conn))  ;; 確実にクリーンアップ
    (process-connection conn))))

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
(def add-user (fn [user]
  (swap! app-state (fn [state]
    (assoc state :users
      (assoc (get state :users) (get user :id) user))))))

;; 投稿追加
(def add-post (fn [post]
  (swap! app-state (fn [state]
    (assoc state :posts (conj (get state :posts) post))))))

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

## 6. ループ構造

### ✅ `loop` / `recur`（実装済み）

末尾再帰最適化を実現するための特殊形式です。

```lisp
;; 基本形
(loop [var1 val1 var2 val2 ...]
  body
  (recur new-val1 new-val2 ...))

;; 階乗（5! = 120）
(def factorial (fn [n]
  (loop [i n acc 1]
    (if (= i 0)
      acc
      (recur (dec i) (* acc i))))))

(factorial 5)  ;; 120

;; カウントダウン
(def count-down (fn [n]
  (loop [i n]
    (if (<= i 0)
      "done"
      (do
        (print i)
        (recur (dec i)))))))

;; リスト処理（matchと組み合わせる場合は要実装）
;; 現在は以下のような形で実装可能：
(def sum-list (fn [lst]
  (loop [items lst result 0]
    (if (empty? items)
      result
      (recur (rest items) (+ result (first items)))))))

(sum-list [1 2 3 4 5])  ;; 15
```

**実装のポイント**:
- `loop`は新しい環境を作成し、変数を初期値で束縛
- `recur`は特別なエラーとして扱い、`loop`でキャッチして変数を更新
- 通常の再帰と異なり、スタックを消費しない（末尾再帰最適化）
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

## 9. モジュールシステム（✅ 基本機能実装済み）

### ✅ モジュール定義
```lisp
;; http.qi
(module http)

(def get (fn [url] ...))
(def post (fn [url data] ...))

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
- 🚧 `use :as` - エイリアス機能（パース済み、評価未実装）

### 標準モジュール

#### ✅ core（自動インポート）
```lisp
;; 基本関数全て
map filter reduce
str len empty?
uvar variable
...
```

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
  unaccent     ;; 🚧 未実装 アクセント除去 "café" -> "cafe"

  ;; 生成 ✅
  hash uuid

  ;; 🚧 未実装
  random       ;; ランダム文字列生成
  map-lines    ;; 各行に関数を適用

  ;; NLP ✅
  word-count

  ;; フォーマット ✅
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

;; フォーマット
(s/indent "hello\nworld" 2)      ;; "  hello\n  world"
(s/wrap "hello world from qi" 10) ;; "hello\nworld from\nqi"
(s/truncate "hello world" 8)     ;; "hello..."
(s/trunc-words "hello world from qi" 2) ;; "hello world..."

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

#### 🚧 csv - CSV/TSV処理（未実装）
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

#### 🚧 regex - 正規表現（未実装）
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

  ;; 統計（データ分析用）
  mean median mode
  stddev variance
  percentile

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
(def analyze (fn [data]
  {:mean (math/mean data)
   :median (math/median data)
   :stddev (math/stddev data)
   :p95 (math/percentile data 95)}))

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
(def rotate-point (fn [x y angle]
  (let [rad (* angle (/ math/pi 180))]
    {:x (- (* x (math/cos rad)) (* y (math/sin rad)))
     :y (+ (* x (math/sin rad)) (* y (math/cos rad)))})))
```

**実装優先度**:
- Phase 1: pow, sqrt, round, floor, ceil, clamp
- Phase 2: random系（実用性高い）
- Phase 3: mean, median, stddev（データ分析用）
- Phase 4: 三角関数・対数（必要になったら）

#### 🔜 time/date - 日付・時刻（計画中）

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
(def format-date (fn [date-str]
  (date-str
   |> (fn [s] (time/parse s "%Y-%m-%d"))
   |> (fn [t] (time/format t "%B %d, %Y")))))

(format-date "2025-01-11")  ;; "January 11, 2025"

;; 4. 実用例：期限チェック
(def is-expired? (fn [expires-at]
  (time/before? expires-at (time/now))))

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
(def add-business-days (fn [date n]
  (loop [current date remaining n]
    (if (<= remaining 0)
      current
      (let [next-day (time/add-days current 1)]
        (if (time/weekend? next-day)
          (recur next-day remaining)
          (recur next-day (dec remaining))))))))

;; 7. 相対時間表示（SNS的）
(def relative-time (fn [timestamp]
  (let [diff (time/diff-minutes timestamp (time/now))]
    (match diff
      n when (< n 60) -> f"{n}分前"
      n when (< n 1440) -> f"{(/ n 60)}時間前"
      n -> f"{(/ n 1440)}日前"))))
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

## 10. 文字列リテラル

### ✅ 基本（実装済み）
```lisp
"hello"
"hello\nworld"
"say \"hello\""
```

### 🚧 複数行（未実装）
```lisp
"""
This is a
multi-line
string
"""
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
(def greet (fn [user]
  f"Welcome, {(:name user)}! You have {(:messages user)} new messages."))

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

### コーディングスタイル - Flow First

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
- 🔜 `match` の `:as` と `=> 変換` でmatch内に流れを埋め込む（近未来）
- 🔜 `tap>` でデバッグ・モニタリング（近未来）
- 🔜 `flow` で複雑な流れを構造化（近未来）

**シンプルに保つ**:
- 短い変数名OK（スコープが短ければ）
- 再利用可能な「小パイプ」を定義
- 一つの関数は一つの責任

### 避けるべきこと
- ❌ 長い関数名・モジュール名
- ❌ 深いネスト（パイプラインを使う）
- ❌ グローバル変数の乱用
- ❌ core関数との名前衝突
- ❌ マクロで固定の変数名を使う（`uvar`を使う）
- ❌ 過度な最適化（まず動くコードを書く）
- ❌ パイプラインを使わない冗長な中間変数

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
- **並列処理**: `pmap`（rayon使用、完全並列化済み）
- **スレッド安全**: Evaluator完全スレッドセーフ化（Arc<RwLock<_>>）
- **並行処理 Layer 1**: `go` `chan` `send!` `recv!` `try-recv!` `close!`
- **並行処理 Layer 2**: `pipeline` `pipeline-map` `pipeline-filter` `fan-out` `fan-in`
- **並行処理 Layer 3**: `await` `then` `catch` `all` `race`
- **データ型**: nil, bool, 整数, 浮動小数点, 文字列, シンボル, キーワード, リスト, ベクタ, マップ, 関数, アトム, チャネル, Uvar
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

**🔜 次フェーズ（並行処理）**:
- チャネル/go 並行処理（Layer 1: go/chan基盤）
- パイプライン並行処理（Layer 2: pipeline, fan-out/in）
- async/await（Layer 3: 高レベル非同期）
- `~>` 非同期パイプライン（go/chan統合）

**🚧 将来**:
- `stream` 遅延評価ストリーム
- 標準モジュール群（str/csv/regex/http/json）

### 実装状況サマリー

#### ✅ 実装済み（v0.1.0）

**特殊形式（8つ）**: `def` `fn` `let` `do` `if` `match` `try` `defer`

**パイプライン演算子**: `|>` 逐次、`||>` 並列、`tap>` タップ

**組み込み関数（136個以上）**:
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
- **並行処理 Layer 1（6）**: go, chan, send!, recv!, try-recv!, close!
- **並行処理 Layer 2（5）**: pipeline, pipeline-map, pipeline-filter, fan-out, fan-in
- **並行処理 Layer 3（5）**: await, then, catch, all, race
- **エラー処理（2）**: try, error
- **メタ（7）**: mac, uvar, variable, macro?, eval, quasiquote, unquote
- **論理（3）**: and, or, not
- **I/O（7）**: print, println, read-file, read-lines, write-file, append-file, file-exists?

**データ型**: nil, bool, 整数, 浮動小数点, 文字列, シンボル, キーワード, リスト, ベクタ, マップ, 関数, アトム, チャネル, Uvar

**先進機能**:
- f-string補間
- match拡張（:as束縛、=> 変換） ⭐ Qi独自
- マクロの衛生性（uvar）
- 末尾再帰最適化（loop/recur）
- defer（エラー時も実行保証）
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

**フェーズ5: 統計・高度な処理**
15. mean, median, stddev

#### 🚧 将来の計画
- 標準モジュール群（str完全版/csv/regex/http/json）
- 非同期パイプライン演算子（~>）
- ストリーム処理（stream）
- 遅延ストリーム（stream）
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
