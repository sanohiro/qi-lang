# 並行処理

Qiの並行処理機能について学びます。

## Qiの並行処理モデル

Qiは**goroutine風の軽量スレッド**と**チャネル**でCSP（Communicating Sequential Processes）を実現します。

## go - goroutineの起動

### 基本的な使い方

```lisp
(go (fn []
  (println "Hello from goroutine")))

; メインスレッドは続行
(println "Main thread")
```

### 実用例

```lisp
; 時間のかかる処理を並行実行
(go (fn []
  (sleep 2000)
  (println "Task 1 done")))

(go (fn []
  (sleep 1000)
  (println "Task 2 done")))

(println "Started tasks")
; すぐに返る
```

## chan - チャネル

スレッド間でデータを送受信します。

### 作成

```lisp
; 無制限チャネル
(def ch (chan))

; 有制限チャネル（バッファサイズ10）
(def ch (chan 10))
```

### 送受信

```lisp
(def ch (chan))

; 送信スレッド
(go (fn []
  (send! ch "Hello")
  (send! ch "World")))

; 受信
(recv! ch)  ; => "Hello"
(recv! ch)  ; => "World"
```

### チャネルのクローズ

```lisp
(def ch (chan))

(go (fn []
  (send! ch 1)
  (send! ch 2)
  (close! ch)))

; 受信ループ
(loop []
  (def val (recv! ch))
  (if (nil? val)
    (println "Channel closed")
    (do
      (println val)
      (recur))))
```

## 並列処理の実践例

### 例1: 並列ダウンロード

```lisp
(defn fetch-url [url]
  (println f"Fetching {url}...")
  (http/get url))

(def urls
  ["https://api.example.com/data1"
   "https://api.example.com/data2"
   "https://api.example.com/data3"])

; チャネルを作成
(def results (chan))

; 各URLを並列取得
(map (fn [url]
       (go (fn []
         (def result (fetch-url url))
         (send! results result))))
     urls)

; 結果を収集
(def responses
  (loop [i 0 acc []]
    (if (>= i (len urls))
      acc
      (recur (+ i 1) (conj acc (recv! results))))))
```

### 例2: ワーカープール

```lisp
(defn worker [id jobs results]
  (go (fn []
    (loop []
      (def job (recv! jobs))
      (if (nil? job)
        (println f"Worker {id} done")
        (do
          (println f"Worker {id} processing {job}")
          (def result (* job job))  ; 処理
          (send! results result)
          (recur)))))))

; チャネル作成
(def jobs (chan 100))
(def results (chan 100))

; ワーカー起動（3つ）
(map (fn [i] (worker i jobs results)) (range 3))

; ジョブを投入
(map (fn [n] (send! jobs n)) (range 1 11))

; チャネルをクローズ
(close! jobs)

; 結果を収集
(loop [i 0 acc []]
  (if (>= i 10)
    acc
    (recur (+ i 1) (conj acc (recv! results)))))
```

## pmap - 並列map

簡単に並列処理ができます。

```lisp
; 通常のmap
(map expensive-function items)

; 並列map
(pmap expensive-function items)

; 例
(defn slow-inc [x]
  (sleep 100)
  (+ x 1))

(time (map slow-inc (range 10)))   ; 約1秒
(time (pmap slow-inc (range 10)))  ; 並列で速い
```

## async/await パターン

### then / catch

```lisp
; Promiseのような使い方
(def handle
  (go (fn []
    (def response (http/get "https://api.example.com"))
    response)))

; 結果を待つ
(async/await handle)
; => {...}

; エラーハンドリング
(match (async/await handle)
  {:ok result} -> (process result)
  {:error err} -> (println f"Error: {err}"))
```

### then - 成功時の処理

```lisp
(def task
  (go (fn []
    (sleep 1000)
    42)))

(async/then task (fn [result]
  (println f"Result: {result}")))
```

### catch - エラー時の処理

```lisp
(def task
  (go (fn []
    (error "Something went wrong"))))

(async/catch task (fn [err]
  (println f"Error: {err}")))
```

## select! - 複数チャネルからの受信

```lisp
; 最初に準備ができたチャネルから受信
(def ch1 (chan))
(def ch2 (chan))

(go (fn []
  (sleep 100)
  (send! ch1 "from ch1")))

(go (fn []
  (sleep 50)
  (send! ch2 "from ch2")))

(async/select!
  ch1 -> (fn [val] (println f"Got {val} from ch1"))
  ch2 -> (fn [val] (println f"Got {val} from ch2")))
; => "Got from ch2 from ch2"（ch2が先）
```

## with-scope - スコープ管理

goroutineのライフサイクルを管理します。

```lisp
(async/with-scope (fn [scope]
  ; scopeにgoroutineを登録
  (async/scope-go scope (fn []
    (sleep 1000)
    (println "Task 1")))

  (async/scope-go scope (fn []
    (sleep 2000)
    (println "Task 2")))

  ; すべてのgoroutineの完了を待つ
))
```

## parallel-do - 並列実行

複数の式を並列実行します。

```lisp
(async/parallel-do
  (do (sleep 1000) (println "Task 1") 1)
  (do (sleep 2000) (println "Task 2") 2)
  (do (sleep 1500) (println "Task 3") 3))
; => [1 2 3]（すべて完了してから返る）
```

## Atom - 共有状態

スレッドセーフな状態管理です。

### 基本的な使い方

```lisp
; アトムの作成
(def counter (atom 0))

; 値の取得
@counter         ; => 0
(deref counter)  ; => 0

; 値の設定
(reset! counter 10)
@counter         ; => 10

; 値の更新（関数適用）
(swap! counter inc)
@counter         ; => 11

(swap! counter + 5)
@counter         ; => 16
```

### 並行アクセス

```lisp
(def counter (atom 0))

; 複数のgoroutineから更新
(map (fn [_]
       (go (fn []
         (loop [i 0]
           (if (< i 100)
             (do
               (swap! counter inc)
               (recur (+ i 1)))
             nil)))))
     (range 10))

; 少し待つ
(sleep 1000)

@counter  ; => 1000（競合なし）
```

## 実践例

### 例1: バッチ処理

```lisp
(defn process-batch [items]
  (def results (chan))

  ; 各アイテムを並列処理
  (map (fn [item]
         (go (fn []
           (def result (process-item item))
           (send! results result))))
       items)

  ; 結果を収集
  (loop [i 0 acc []]
    (if (>= i (len items))
      acc
      (recur (+ i 1) (conj acc (recv! results))))))

(process-batch (range 1 101))
```

### 例2: タイムアウト付き処理

```lisp
(defn fetch-with-timeout [url timeout-ms]
  (def result (chan))
  (def timeout-ch (chan))

  ; タイムアウトタイマー
  (go (fn []
    (sleep timeout-ms)
    (send! timeout-ch :timeout)))

  ; 実際の処理
  (go (fn []
    (def response (http/get url))
    (send! result response)))

  ; どちらか先に完了した方を返す
  (async/select!
    result -> (fn [r] {:ok r})
    timeout-ch -> (fn [_] {:error "Timeout"})))
```

### 例3: 並列検索

```lisp
(defn parallel-search [query sources]
  (def results (chan))

  ; 各ソースで並列検索
  (map (fn [source]
         (go (fn []
           (def hits (search source query))
           (send! results {:source source :hits hits}))))
       sources)

  ; 結果を統合
  (loop [i 0 all-results []]
    (if (>= i (len sources))
      all-results
      (recur (+ i 1) (concat all-results (recv! results))))))

(parallel-search "rust"
                 ["github" "stackoverflow" "reddit"])
```

### 例4: レート制限

```lisp
(defn rate-limited-requests [urls requests-per-second]
  (def interval (/ 1000 requests-per-second))
  (def results [])

  (loop [remaining urls]
    (if (empty? remaining)
      results
      (do
        (def url (first remaining))
        (def response (http/get url))
        (def results (conj results response))
        (sleep interval)
        (recur (rest remaining))))))

; 1秒あたり5リクエスト
(rate-limited-requests urls 5)
```

### 例5: リアルタイムログ監視

```lisp
(def log-chan (chan 100))

; ログ収集goroutine
(go (fn []
  (loop []
    (def log-line (read-log-line))
    (send! log-chan log-line)
    (recur))))

; ログ処理goroutine（複数）
(map (fn [worker-id]
       (go (fn []
         (loop []
           (def line (recv! log-chan))
           (process-log worker-id line)
           (recur)))))
     (range 5))

; メインスレッドは他の処理を継続
(println "Log monitoring started")
```

## デバッグとトラブルシューティング

### デッドロックの回避

```lisp
; 悪い例（デッドロック）
(def ch1 (chan 0))  ; バッファなし
(send! ch1 "value")  ; ブロック（受信者がいない）

; 良い例
(def ch1 (chan 1))  ; バッファあり
(send! ch1 "value")  ; ブロックしない

; または
(go (fn []
  (send! ch1 "value")))  ; 別goroutineで送信
```

### チャネルのクローズ忘れ

```lisp
; 悪い例
(loop []
  (def val (recv! ch))  ; チャネルがクローズされないと永遠に待つ
  (process val)
  (recur))

; 良い例
(loop []
  (def val (recv! ch))
  (if (nil? val)
    (println "Done")
    (do
      (process val)
      (recur))))
```

## まとめ

Qiの並行処理：

- **go**: goroutineの起動
- **chan**: チャネル（送受信）
- **send! / recv!**: データの送受信
- **close!**: チャネルのクローズ
- **pmap**: 並列map
- **atom**: スレッドセーフな状態
- **async/await**: 非同期パターン
- **select!**: 複数チャネルからの受信

CSPモデルで安全な並行処理！

## 次のステップ

次は[実践的なプログラミング](./06-practical.md)を学びます。ファイルI/O、HTTP、データベースなどの実用的な機能を使いましょう。
