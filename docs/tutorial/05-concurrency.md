# 第5章: 並行・並列処理を簡単に

**所要時間**: 35分

並行・並列処理は、Qiの最も強力な機能の一つです。**簡単に、安全に、高速な並列処理**が実現できます。

---

## なぜ並行・並列処理が必要か？

### シーケンシャル処理（遅い）

```qi
(def urls ["https://api1.com" "https://api2.com" "https://api3.com"])

; 順番に処理（合計3秒かかる）
(map http/get urls)
```

### 並列処理（速い）

```qi
; 並列で処理（約1秒で完了！）
(urls ||> http/get)
```

---

## 並列パイプライン: `||>`

`||>`は、コレクションの各要素を**並列に処理**します。

### 基本的な使い方

```qi
qi> ([1 2 3 4 5]
     ||> (fn [x] (* x x)))
; => [1 4 9 16 25]
```

**内部では自動的に`pmap`（並列map）が使われます。**

### 実用例1: 複数のURL取得

```qi
(def urls
  ["https://api.example.com/users/1"
   "https://api.example.com/users/2"
   "https://api.example.com/users/3"])

; 並列で取得
(urls
 ||> http/get
 ||> (fn [resp] (get resp :body))
 ||> json/parse)
```

---

## 並列map: `pmap`

`pmap`は、mapの並列版です。

```qi
qi> (pmap (fn [x] (* x 2)) [1 2 3 4 5])
; => [2 4 6 8 10]

qi> (defn heavy-process [x]
      (do
        (sleep 1000)  ; 1秒待つ
        (* x x)))

; 順番に処理（5秒かかる）
qi> (map heavy-process [1 2 3 4 5])
; => [1 4 9 16 25]

; 並列に処理（約1秒で完了）
qi> (pmap heavy-process [1 2 3 4 5])
; => [1 4 9 16 25]
```

---

## 並列filter: `go/pfilter`

`go/pfilter`は、filterの並列版です。

```qi
qi> (go/pfilter even? [1 2 3 4 5 6 7 8 9 10])
; => [2 4 6 8 10]

(defn is-prime? [n]
  ; 素数判定（重い処理）
  (if (<= n 1)
    false
    (loop [i 2]
      (if (>= (* i i) n)
        true
        (if (= (% n i) 0)
          false
          (recur (inc i)))))))

; 並列で素数を抽出
qi> (go/pfilter is-prime? (range 1 100))
; => [2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71 73 79 83 89 97]
```

---

## goroutine風の並行処理

Qiは、Go言語風の並行処理をサポートしています。

### チャネルの作成

```qi
qi> (def ch (go/chan))
```

### 送受信

```qi
; 送信
qi> (go/send! ch 42)

; 受信
qi> (go/recv! ch)
; => 42
```

### goroutineで実行

```qi
; バックグラウンドで実行
qi> (go/run (println "Hello from goroutine!"))
; => "Hello from goroutine!"
```

---

## 実用例1: 並列ダウンロード

```qi
(defn download-file [url]
  (println f"Downloading: {url}")
  (let [resp (http/get url)]
    (get resp :body)))

(def urls
  ["https://example.com/file1.txt"
   "https://example.com/file2.txt"
   "https://example.com/file3.txt"])

; 並列ダウンロード
(def results (urls ||> download-file))

; または pmap を使う
(def results (pmap download-file urls))
```

---

## 実用例2: 並列データ処理

```qi
(defn process-chunk [chunk]
  ; 重い処理
  (chunk
   |> (filter even?)
   |> (map (fn [x] (* x x)))
   |> (reduce + 0)))

; 大きなデータを分割
(def data (range 1 1000))
(def chunks
  [(take 250 data)
   (take 250 (drop 250 data))
   (take 250 (drop 500 data))
   (take 250 (drop 750 data))])

; 各チャンクを並列処理
(def results (chunks ||> process-chunk))

; 結果を集計
(reduce + 0 results)
```

---

## 実用例3: goroutineとチャネル

```qi
(def ch (go/chan))

; プロデューサー
(go/run
  (do
    (go/send! ch 1)
    (go/send! ch 2)
    (go/send! ch 3)
    (go/close! ch)))

; コンシューマー
(loop [acc []]
  (let [val (go/try-recv! ch)]
    (if (nil? val)
      acc
      (recur (conj acc val)))))
; => [1 2 3]
```

---

## 実用例4: ワーカープール

```qi
(defn worker [id ch]
  (loop []
    (let [task (go/try-recv! ch)]
      (if (nil? task)
        (println f"Worker {id} done")
        (do
          (println f"Worker {id} processing: {task}")
          ; 処理
          (sleep 100)
          (recur))))))

(def ch (go/chan))

; ワーカーを起動
(go/run (worker 1 ch))
(go/run (worker 2 ch))
(go/run (worker 3 ch))

; タスクを投入
(go/send! ch "Task 1")
(go/send! ch "Task 2")
(go/send! ch "Task 3")
(go/send! ch "Task 4")
(go/send! ch "Task 5")

; チャネルをクローズ
(go/close! ch)
```

---

## Atom: スレッドセーフな状態管理

Atomを使うと、複数のgoroutineから安全に状態を更新できます。

### 基本的な使い方

```qi
qi> (def counter (atom 0))

; 値を取得
qi> (deref counter)
; => 0

qi> @counter  ; 短縮形
; => 0

; 値を更新
qi> (swap! counter inc)
; => 1

qi> @counter
; => 1
```

### 実用例: 並列カウンター

```qi
(def counter (atom 0))

; 複数のgoroutineから安全にインクリメント
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))

(sleep 100)  ; 完了を待つ

qi> @counter
; => 5
```

---

## 並列処理のベストプラクティス

### 1. いつ並列化するか？

**並列化すべき場合**:
- CPU集約的な処理（計算、変換）
- I/O待ちが多い処理（HTTP、ファイル読み込み）
- 要素数が多い（目安：100要素以上）

**並列化すべきでない場合**:
- 軽量な処理（要素ごとの処理が1ms未満）
- 要素数が少ない（目安：10要素未満）
- 順序が重要な処理

### 2. パフォーマンス比較

```qi
; 軽量処理 - mapの方が速い
(map inc [1 2 3 4 5])  ; 高速

; 重い処理 - pmapの方が速い
(pmap heavy-process [1 2 3 ... 100])  ; 高速
```

### 3. デバッグのコツ

並列処理はデバッグが難しいので、まず逐次処理で動作確認：

```qi
; ✅ まず逐次処理で確認
(data |> (map process) |> verify)

; ✅ 動作確認後に並列化
(data ||> process |> verify)
```

---

## 練習問題

### 問題1: 並列2乗計算

リストの各要素を2乗して、その合計を並列に計算する関数を書いてください。

```qi
(defn parallel-sum-squares [numbers]
  ; ここを埋めてください
  )

; テスト
(parallel-sum-squares [1 2 3 4 5])  ; => 55
```

<details>
<summary>解答例</summary>

```qi
(defn parallel-sum-squares [numbers]
  (numbers
   ||> (fn [x] (* x x))
   |> (reduce + 0)))

; または pmap を使う
(defn parallel-sum-squares [numbers]
  (pmap (fn [x] (* x x)) numbers)
  |> (reduce + 0))
```

</details>

### 問題2: 並列フィルタと変換

偶数だけを並列に抽出して、2倍にする関数を書いてください。

```qi
(defn parallel-process [numbers]
  ; ここを埋めてください
  )

; テスト
(parallel-process [1 2 3 4 5 6])  ; => [4 8 12]
```

<details>
<summary>解答例</summary>

```qi
(defn parallel-process [numbers]
  (numbers
   |> (go/pfilter even?)
   ||> (fn [x] (* x 2))))
```

</details>

### 問題3: Atomでアクセスカウント

Atomを使って、複数のgoroutineからアクセスカウントを安全に更新する関数を書いてください。

```qi
(def access-count (atom 0))

(defn record-access []
  ; ここを埋めてください
  )

; テスト
(go/run (record-access))
(go/run (record-access))
(go/run (record-access))

(sleep 100)
@access-count  ; => 3
```

<details>
<summary>解答例</summary>

```qi
(def access-count (atom 0))

(defn record-access []
  (swap! access-count inc))
```

</details>

---

## まとめ

この章で学んだこと：

- ✅ 並列パイプライン (`||>`)
- ✅ 並列map/filter/reduce (`pmap`, `go/pfilter`, `go/preduce`)
- ✅ goroutine風の並行処理 (`go/run`, `go/chan`)
- ✅ Atom（スレッドセーフな状態管理）
- ✅ 並列処理のベストプラクティス

---

## 次のステップ

並行・並列処理をマスターしたら、最後は**WebアプリケーションとAPI**を学びましょう！

➡️ [第6章: WebアプリケーションとAPI](06-web-api.md)

HTTPサーバーとJSON APIの構築方法を学びます。
