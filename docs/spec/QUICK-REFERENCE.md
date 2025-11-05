# Qi言語 クイックリファレンス

**1ページで学ぶQiの基本**

---

## 📌 基本構文

### データ型

```qi
42                ;; 整数
3.14              ;; 浮動小数点
"hello"           ;; 文字列
f"Hello, {name}"  ;; f-string（文字列補間）
true / false      ;; 真偽値
nil               ;; nil
:keyword          ;; キーワード
[1 2 3]           ;; ベクタ
'(1 2 3)          ;; リスト（クオート必須）
{:name "Alice"}   ;; マップ
```

### 定義

```qi
(def x 42)                          ;; 変数定義
(defn greet [name] (str "Hello, " name))  ;; 関数定義
(let [x 10 y 20] (+ x y))          ;; ローカル束縛
```

### 制御構造

```qi
(if (> x 10) "big" "small")        ;; if
(do (println "1") (println "2"))   ;; 順次実行
(loop [i 0] (if (< i 10) (recur (inc i)) i))  ;; ループ
```

---

## ⚡ パイプライン演算子（★ウリ）

```qi
;; |> - 逐次パイプ
(data |> parse |> transform |> save)

;; |>? - Railway Pipeline（エラー処理）
(input |>? validate |>? parse |>? process)
;; {:error ...} でショートサーキット、それ以外は成功

;; ||> - 並列パイプ（自動的にpmap化）
([1 2 3 4] ||> heavy-process)  ;; 並列実行

;; ~> - 非同期パイプ（goroutine風）
(def result (data ~> transform))
(go/recv! result)

;; tap> - 副作用タップ（デバッグ用）
(data |> parse |> (tap print) |> save)
```

---

## 🔀 パターンマッチング（★ウリ）

```qi
(match value
  {:ok data} -> (process data)
  {:error e} -> (log e)
  _ -> "default")

;; ガード条件
(match x
  n when (> n 0) -> "positive"
  n when (< n 0) -> "negative"
  _ -> "zero")

;; ベクタの分解
(match [1 2 3]
  [a b c] -> (+ a b c))  ;; => 6
```

---

## 🚀 並行・並列処理（★ウリ）

### goroutine風

```qi
;; チャネル作成
(def ch (go/chan))

;; 送受信
(go/send! ch 42)
(def val (go/recv! ch))  ;; => 42

;; goroutineで実行
(go/run (println "async!"))
```

### 並列map/filter/reduce

```qi
(pmap (fn [x] (* x 2)) [1 2 3 4])     ;; 並列map
(pfilter even? [1 2 3 4])              ;; 並列filter
(preduce + [1 2 3 4] 0)                ;; 並列reduce (fn collection init)
```

### Atom（スレッドセーフな状態管理）

```qi
(def counter (atom 0))
(swap! counter inc)        ;; => 1
(reset! counter 0)         ;; => 0
(deref counter)            ;; => 0 または @counter
```

---

## 📦 コレクション操作

### アクセス

```qi
(first [1 2 3])            ;; => 1
(rest [1 2 3])             ;; => (2 3)
(last [1 2 3])             ;; => 3
(nth [10 20 30] 1)         ;; => 20
```

### 変換

```qi
(map inc [1 2 3])          ;; => [2 3 4]
(filter even? [1 2 3 4])   ;; => [2 4]
(reduce + 0 [1 2 3])       ;; => 6
(take 2 [1 2 3 4])         ;; => [1 2]
(drop 2 [1 2 3 4])         ;; => [3 4]
```

### 連結・ソート

```qi
(concat [1 2] [3 4])       ;; => [1 2 3 4]
(cons 0 [1 2 3])           ;; => [0 1 2 3]
(sort [3 1 4])             ;; => [1 3 4]
(reverse [1 2 3])          ;; => [3 2 1]
(distinct [1 2 2 3])       ;; => [1 2 3]
```

---

## 🔍 述語関数

```qi
;; 型チェック
(nil? x) (number? x) (string? x) (list? x) (vector? x) (map? x)

;; 状態チェック
(some? x)      ;; nilでない
(empty? coll)  ;; 空のコレクション
(error? x)     ;; {:error ...} 形式

;; 数値述語
(even? 2) (odd? 3) (positive? 1) (negative? -1) (zero? 0)
```

---

## ⚠️ エラー処理

### Railway Pipeline（推奨）

```qi
(defn divide [x y]
  (if (= y 0)
    {:error "division by zero"}
    (/ x y)))

(10 |>? (fn [x] (divide 100 x)))  ;; => 10
(0 |>? (fn [x] (divide 100 x)))   ;; => {:error "division by zero"}

;; error?述語で判定
(if (error? result)
  (log "エラー")
  (process result))
```

### try/catch

```qi
(match (try (risky-operation))
  {:error e} -> (log e)
  result -> result)
```

### defer（リソース管理）

```qi
(defn process-file [path]
  (let [f (open path)]
    (do
      (defer (close f))  ;; 関数終了時に必ず実行
      (read f))))
```

---

## 🌐 HTTP・JSON

### HTTPクライアント

```qi
;; シンプル版（ボディのみ）
(def resp (http/get "https://api.example.com/data"))
(def data (json/parse resp))

;; 詳細版（ステータス・ヘッダー・ボディ）
(def resp (http/get! "https://api.example.com/data"))
(def data (json/parse (get resp :body)))
```

### HTTPサーバー

```qi
(defn handler [req]
  (server/json {:message "Hello, World!"}))

(server/serve handler {:port 3000})
```

### JSON

```qi
(json/parse "{\"name\":\"Alice\"}")  ;; => {:name "Alice"}
(json/stringify {:name "Bob"})       ;; => "{\"name\":\"Bob\"}"
```

---

## 📁 ファイルI/O

```qi
(io/read-file "data.txt")                ;; ファイル読み込み
(io/write-file "output.txt" "content")   ;; ファイル書き込み
(io/read-lines "data.txt")               ;; 行ごとに読み込み
```

---

## 🧮 数学関数

```qi
(math/pow 2 3)      ;; => 8
(math/sqrt 16)      ;; => 4.0
(math/round 3.14)   ;; => 3.0
(math/rand)         ;; ランダム [0.0, 1.0)
(math/rand-int 10)  ;; ランダム整数 [0, 10)
```

---

## 🧪 テスト

```qi
(test/assert-eq (+ 1 2) 3)
(test/assert (> 5 3))
(test/assert-throws (fn [] (error "test")))

;; テスト実行
(test/run)
```

---

## 💡 Tips

### リストとベクタの使い分け

- **ベクタ `[...]`**: デフォルト（JSON互換、高速）
- **リスト `'(...)`**: 再帰的処理、Lisp的な処理

### 並列化の目安

- **使う**: CPU集約的、I/O待ち、要素数100+
- **使わない**: 軽量処理、要素数10未満

### エラー処理の選択

- **Railway Pipeline (`|>?`)**: API、ファイルIO、パース
- **try/catch**: 予期しないエラー、サードパーティライブラリ

---

## 📚 詳細ドキュメント

完全なリファレンスは [docs/spec/](.) を参照してください。

- [02-flow-pipes.md](02-flow-pipes.md) - パイプライン演算子
- [03-concurrency.md](03-concurrency.md) - 並行・並列処理
- [04-match.md](04-match.md) - パターンマッチング
- [06-data-structures.md](06-data-structures.md) - データ構造
- [08-error-handling.md](08-error-handling.md) - エラー処理
