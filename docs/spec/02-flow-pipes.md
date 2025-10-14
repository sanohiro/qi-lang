# パイプライン拡張 - Flow DSL

**流れを設計する言語**

Qiはパイプライン演算子を拡張し、**データの流れを直感的に表現**できる言語です。

---

## パイプライン演算子の体系

| 演算子 | 意味 | 用途 |
|--------|------|------|
| `|>` | 逐次パイプ | 基本的なデータ変換 |
| `\|>?` | Railway パイプ | エラーハンドリング、Result型の連鎖 |
| `||>` | 並列パイプ | 自動的にpmap化、リスト処理の並列化 |
| `tap>` | 副作用タップ | デバッグ、ログ、モニタリング |
| `~>` | 非同期パイプ | go/chan統合、非同期IO |

---

## `|>` 基本パイプライン

**左から右へデータを流す**

```qi
;; 基本
(data |> parse |> transform |> save)

;; ネスト回避
(data
 |> (filter valid?)
 |> (map transform)
 |> (reduce merge {}))

;; 引数付き関数
(10 |> (+ 5) |> (* 2))  ;; (+ 10 5) |> (* 2) => 30

;; _プレースホルダー: 任意の位置に値を挿入
(42 |> (+ 10 _ 3))  ;; (+ 10 42 3) => 55
("world" |> (str "Hello, " _))  ;; (str "Hello, " "world") => "Hello, world"

;; 実用例: URL構築
(params
 |> (map (fn [[k v]] f"{k}={v}"))
 |> (join "&")
 |> (str base-url "?" _))
```

---

## `||>` 並列パイプライン

**自動的にpmapに展開**

```qi
;; 並列処理
(urls ||> http/get ||> json/parse)
;; ↓ 展開
(urls |> (pmap http/get) |> (pmap json/parse))

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

---

## `|>?` Railway Pipeline

**エラーハンドリングを流れの中に組み込む** - Railway Oriented Programming

```qi
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
 |>? (fn [name] {:ok (str/upper name)}))
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

## `tap>` 副作用タップ

**流れを止めずに観察**（Unix `tee`相当）

```qi
;; デバッグ
(data
 |> clean
 |> (tap print)
 |> analyze
 |> (tap log)
 |> save)

;; ログ
(requests
 |> (tap log-request)
 |> process
 |> (tap log-response))

;; 簡潔な使い方
([1 2 3]
 |> (map inc)
 |> (tap print)
 |> sum)
```

**実装**:
- `tap`関数として実装
- パイプライン内で`|> (tap f)`として使用
- 関数を実行してから元の値を返す

---

## `~>` 非同期パイプライン

**並行処理との統合 - goroutine風の非同期実行**

`~>` 演算子はパイプラインをgoroutineで自動実行し、結果をチャネルで返します。

```qi
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

## `stream` 遅延評価

**巨大データの効率的処理 - 遅延評価と無限データ構造**

Streamは値を必要になるまで計算しない遅延評価のデータ構造です。
無限データ構造や大きなデータセットをメモリ効率的に扱えます。

### Stream作成

```qi
;; コレクションからストリーム作成
(stream/stream [1 2 3 4 5])

;; 範囲ストリーム
(stream/range 0 10)  ;; 0から9まで

;; 無限ストリーム：同じ値を繰り返し
(stream/repeat 42)  ;; 42, 42, 42, ...

;; 無限ストリーム：リストを循環
(stream/cycle [1 2 3])  ;; 1, 2, 3, 1, 2, 3, ...

;; 無限ストリーム：関数を反復適用
(stream/iterate (fn [x] (* x 2)) 1)  ;; 1, 2, 4, 8, 16, 32, ...
```

### Stream変換

```qi
;; map: 各要素に関数を適用
(def s (stream/range 1 6))
(def s2 (stream/map (fn [x] (* x 2)) s))
(stream/realize s2)  ;; (2 4 6 8 10)

;; filter: 条件に合う要素のみ
(def s (stream/range 1 11))
(def s2 (stream/filter (fn [x] (= (% x 2) 0)) s))
(stream/realize s2)  ;; (2 4 6 8 10)

;; take: 最初のn個を取得（無限ストリームを有限化）
(def s (stream/repeat 42))
(def s2 (stream/take 5 s))
(stream/realize s2)  ;; (42 42 42 42 42)

;; drop: 最初のn個をスキップ
(def s (stream/range 0 10))
(def s2 (stream/drop 5 s))
(stream/realize s2)  ;; (5 6 7 8 9)
```

### Stream実行

```qi
;; realize: ストリームをリストに変換（全要素を計算）
(stream/realize (stream/stream [1 2 3]))  ;; (1 2 3)

;; ⚠️ 注意: 無限ストリームをrealizeすると無限ループ
;; (stream/realize (stream/repeat 42))  ;; NG: 永遠に終わらない

;; 正しい使い方: takeで有限化してからrealize
(stream/realize (stream/take 5 (stream/repeat 42)))  ;; OK
```

### パイプラインとの統合

```qi
;; 既存の |> パイプライン演算子で使える
[1 2 3 4 5]
  |> stream/stream
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (> x 10)))
  |> stream/realize
;; (16 25)

;; 無限ストリームの処理
1
  |> (stream/iterate (fn [x] (* x 2)))
  |> (stream/take 10)
  |> stream/realize
;; (1 2 4 8 16 32 64 128 256 512)

;; 複雑な変換チェーン
(stream/range 1 100)
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (= (% x 3) 0)))
  |> (stream/take 5)
  |> stream/realize
;; (9 36 81 144 225)
```

### 実用例

```qi
;; 素数の無限ストリーム（概念）
(def primes
  (2
   |> (stream/iterate inc)
   |> (stream/filter prime?)))

(stream/realize (stream/take 10 primes))  ;; 最初の10個の素数

;; フィボナッチ数列
(def fib-stream
  (stream/iterate (fn [[a b]] [b (+ a b)]) [0 1]))

(stream/realize (stream/take 10 fib-stream)
  |> (map first))  ;; (0 1 1 2 3 5 8 13 21 34)

;; データ処理パイプライン
(defn process-data [data]
  (data
   |> stream
   |> (stream/map parse)
   |> (stream/filter valid?)
   |> (stream/take 1000)
   |> stream/realize))
```

### I/Oストリーム

**ファイルとHTTPデータの遅延読み込み - テキスト＆バイナリ対応**

#### テキストモード（行ベース）

```qi
;; stream/file: ファイルを行ごとに遅延読み込み
(stream/file "large.log")
  |> (stream/filter error-line?)
  |> (stream/map parse)
  |> (stream/take 100)
  |> stream/realize

;; http/get-stream: HTTPレスポンスを行ごとに読み込み
(http/get-stream "https://api.example.com/data")
  |> (stream/take 10)
  |> stream/realize

;; http/post-stream: POSTリクエストでストリーミング受信
(http/post-stream "https://api.example.com/upload" {:data "value"})
  |> (stream/take 10)
  |> stream/realize

;; http/request-stream: 詳細設定でストリーミング
(http/request-stream {
  :method "GET"
  :url "https://api.example.com/stream"
})
  |> (stream/filter important?)
  |> stream/realize
```

#### バイナリモード（バイトチャンク）

```qi
;; stream/file :bytes - ファイルを4KBチャンクで読み込み
(stream/file "image.png" :bytes)
  |> (stream/take 10)
  |> stream/realize
;; => Vector of Integers (bytes) のリスト

;; http/get-stream :bytes - HTTPバイナリダウンロード
(http/get-stream "https://example.com/file.bin" :bytes)
  |> (stream/map process-chunk)
  |> stream/realize

;; バイト処理の例
(def bytes (first (stream/realize (stream/take 1 (stream/file "data.bin" :bytes)))))
(def sum (reduce + bytes))  ; バイトの合計
(println sum)

;; 画像ダウンロード
(http/get-stream "https://example.com/logo.png" :bytes)
  |> stream/realize
  |> flatten
  |> (write-bytes "logo.png")  ; write-bytes は将来実装
```

**モード比較**:

| モード | 用途 | 戻り値 | 例 |
|--------|------|--------|-----|
| テキスト（デフォルト） | ログ、CSV、JSON | String（行ごと） | `(stream/file "data.txt")` |
| バイナリ（`:bytes`） | 画像、動画、バイナリ | Vector of Integers（4KBチャンク） | `(stream/file "image.png" :bytes)` |

```qi
;; CSVファイルの処理
(stream/file "data.csv")
  |> (stream/drop 1)  ; ヘッダースキップ
  |> (stream/map (fn [line] (split line ",")))
  |> (stream/filter (fn [cols] (> (len cols) 2)))
  |> (stream/take 1000)
  |> stream/realize

;; HTTPからJSONを取得してパース
(http/get-stream "https://jsonplaceholder.typicode.com/todos/1")
  |> stream/realize
  |> (join "\n")
  |> json/parse
```

**実用例: ログファイル解析**

```qi
;; 大きなログファイルをメモリ効率的に処理
(defn analyze-logs [file]
  (stream/file file
   |> (stream/filter (fn [line] (str/contains? line "ERROR")))
   |> (stream/map parse-log-line)
   |> (stream/take 100)  ; 最初の100エラー
   |> stream/realize))

;; 結果を取得
(def errors (analyze-logs "/var/log/app.log"))
(println (str "Found " (len errors) " errors"))
```

---

## パイプライン文化

**Unix哲学 × 関数型 × Lisp**

小さなパイプを定義して組み合わせることで、複雑な処理を構築できます。

```qi
;; 小さなパイプを定義
(def clean-text
  (fn [text]
    (text |> trim |> lower |> remove-punctuation)))

(def extract-emails
  (fn [text]
    (text |> (split "\\s+") |> (filter email?))))

(def dedupe
  (fn [coll]
    (coll |> sort |> unique)))

;; 組み合わせて使う
(document
 |> clean-text
 |> extract-emails
 |> dedupe
 |> (join ", "))
```
