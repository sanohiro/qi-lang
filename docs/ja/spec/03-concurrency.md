# 並行・並列処理 - Qiの真髄

**Qiは並行・並列処理を第一級市民として扱う言語です。**

「並列、並行を簡単にできるのはQiのキモ」- これがQiの設計哲学の核心です。

> **実装**: `src/builtins/concurrency.rs`, `src/builtins/fn.rs`

---

## 設計哲学

Qiの並行・並列処理は**3層アーキテクチャ**で構成されます：

```
┌─────────────────────────────────────┐
│  Layer 3: go/await (高レベル)        │  ← 使いやすさ（I/O、API）
│  - go/await, go/then, go/catch      │
├─────────────────────────────────────┤
│  Layer 2: Pipeline (中レベル)        │  ← 関数型らしさ
│  - pmap, go/pipeline, go/fan-out/in │
├─────────────────────────────────────┤
│  Layer 1: go/chan (低レベル基盤)     │  ← パワーと柔軟性
│  - go/run, go/chan, go/send!, ...   │
└─────────────────────────────────────┘
```

**すべてgo/名前空間に統一** - シンプルで一貫性のあるアーキテクチャ。

---

## Layer 1: go/chan（基盤）

**Go風の並行処理**

### チャネル作成

```qi
(go/chan)       ;; 無制限バッファ
(go/chan 10)    ;; バッファサイズ10
```

### 送受信

```qi
;; チャネルへの送信と受信
(def ch (go/chan))
(def value 42)

(go/send! ch value)              ;; チャネルに送信
(go/recv! ch)                    ;; ブロッキング受信
(go/recv! ch :timeout 1000)      ;; タイムアウト付き受信（ミリ秒）
(go/try-recv! ch)                ;; 非ブロッキング受信
(go/close! ch)                   ;; チャネルクローズ
```

### goroutine

```qi
(go/run (println "async!"))

(def result-ch (go/chan))
(go/run (go/send! result-ch (* 2 3)))
(go/recv! result-ch)  ;; 6
```

### 使用例: 並列計算

```qi
;; 複数のgoroutineで並列計算
(def ch (go/chan))

(go/run (go/send! ch (* 2 3)))
(go/run (go/send! ch (* 4 5)))
(go/run (go/send! ch (* 6 7)))

[(go/recv! ch) (go/recv! ch) (go/recv! ch)]  ;; => [6 20 42]
```

### select! - 複数チャネルの待ち合わせ

```qi
;; 複数のチャネルから最初に来たデータを処理
(def ch1 (go/chan))
(def ch2 (go/chan))

(go/run (go/send! ch1 "from ch1"))
(go/run (go/send! ch2 "from ch2"))

(go/select!
  ch1 (fn [val] (println "Got from ch1:" val))
  ch2 (fn [val] (println "Got from ch2:" val)))
```

### Structured Concurrency（構造化並行処理）

```qi
;; スコープ作成
(def ctx (go/make-scope))

;; スコープ内でgoroutine起動
(go/scope-go ctx (fn []
  (loop [i 0]
    (if (go/cancelled? ctx)
      (println "cancelled")
      (do
        (println i)
        (sleep 100)
        (recur (inc i)))))))

;; スコープ内の全goroutineをキャンセル
(go/cancel! ctx)

;; with-scope関数（便利版）
(go/with-scope (fn [ctx]
  (go/scope-go ctx task1)
  (go/scope-go ctx task2)
  ;; スコープ終了時に自動キャンセル
  ))
```

---

## Layer 2: Pipeline（構造化並行処理）

**関数型スタイルの並列処理**

### 並列コレクション操作

```qi
;; pmap - 並列map（rayon使用）
([1 2 3 4 5] |> (pmap (fn [x] (* x x))))
;; => (1 4 9 16 25)

;; go/pfilter - 並列filter
([1 2 3 4 5 6] |> (go/pfilter (fn [x] (= (% x 2) 0))))
;; => (2 4 6)

;; go/preduce - 並列reduce (fn collection init)
([1 2 3 4 5] |> (fn [data] (go/preduce + data 0)))
;; => 15

;; go/parallel-do - 複数式の並列実行
(go/parallel-do
  (println "Task 1")
  (println "Task 2")
  (println "Task 3"))
```

### パイプライン処理

```qi
;; pipeline - n並列でxf変換をchに適用
(def ch (go/chan))
(go/pipeline 4 (fn [x] (* x 2)) ch)
```

### ファンアウト/ファンイン

```qi
;; fan-out - 1つのチャネルをn個に分岐
(def ch (go/chan))
(def output-chs (go/fan-out ch 3))

;; fan-in - 複数チャネルを1つに合流
(def ch1 (go/chan))
(def ch2 (go/chan))
(def ch3 (go/chan))
(def merged (go/fan-in [ch1 ch2 ch3]))
```

---

## Layer 3: go/await（高レベル）

**モダンな非同期処理**

### 基本的なawait

```qi
(def p (go/run (fn [] (+ 1 2 3))))
(go/await p)  ;; => 6
```

### Promise チェーン

```qi
(-> (go/run (fn [] 10))
    (go/then (fn [x] (* x 2)))
    (go/then (fn [x] (+ x 1)))
    (go/await))  ;; => 21
```

### Promise.all風

```qi
(def promises [(go/run (fn [] 1)) (go/run (fn [] 2)) (go/run (fn [] 3))])
(go/await (go/all promises))  ;; => [1 2 3]
```

### Promise.race風

```qi
(def promises [(go/run (fn [] "slow")) (go/run (fn [] "fast"))])
(go/await (go/race promises))  ;; => "fast"
```

### エラーハンドリング

```qi
(go/catch promise (fn [e] (println "Error:" e)))
```

---

## 状態管理 - Atom

**スレッドセーフな状態管理**

Qiの状態管理は**Atom**（アトム）を使います。Atomは参照透過性を保ちながら、必要な場所だけで状態を持つための仕組みです。

### 基本操作

```qi
atom                    ;; アトム作成
deref                   ;; 値取得
@                       ;; derefの短縮形（@counter => (deref counter)）
swap!                   ;; 関数で更新（アトミック）
reset!                  ;; 値を直接セット
```

### アトムの作成と参照

```qi
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

### @ 構文（derefの短縮形）

```qi
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

### 実用例1: カウンター

```qi
;; リクエストカウンター
(def request-count (atom 0))

(defn handle-request [req]
  (do
    (swap! request-count inc)
    (process req)))

;; 現在のカウント確認
(deref request-count)  ;; 処理したリクエスト数
```

### 実用例2: 状態を持つキャッシュ

```qi
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

### 実用例3: 接続管理（deferと組み合わせ）

```qi
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

### 実用例4: 複雑な状態更新

```qi
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

### 実用例5: スレッドセーフなカウンター

```qi
(def counter (atom 0))

;; 複数のgoroutineから安全にインクリメント
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))

(sleep 100)  ;; 完了を待つ
(deref counter)  ;; => 3
```

### Atomの設計哲学

1. **局所的な状態**: グローバル変数の代わりに、必要な場所だけでAtomを使う
2. **swap!の原子性**: 更新が確実に適用される（競合状態を回避）
3. **関数型との共存**: 純粋関数とAtomを組み合わせる
4. **deferと相性が良い**: リソース管理で威力を発揮

---

## 実装技術スタック

- **crossbeam-channel**: Go風チャネル実装（select!マクロも提供）
- **rayon**: データ並列（pmap, go/pfilter, go/preduce等）
- **parking_lot**: 高性能RwLock
- **Arc<RwLock<_>>**: Evaluatorの完全スレッドセーフ化

---

## 関数一覧

### Layer 1 (go/chan)

- `go/chan`: チャネル作成
- `go/send!`: 送信
- `go/recv!`: ブロッキング受信
- `go/recv! :timeout`: タイムアウト付き受信
- `go/try-recv!`: 非ブロッキング受信
- `go/close!`: チャネルクローズ
- `go/run`: goroutine起動
- `go/select!`: 複数チャネル待ち合わせ
- `go/make-scope`: スコープ作成
- `go/scope-go`: スコープ内goroutine
- `go/cancel!`: スコープキャンセル
- `go/cancelled?`: キャンセル確認
- `go/with-scope`: スコープ自動管理

### Layer 2 (Pipeline)

- `pmap`: 並列map
- `go/pfilter`: 並列filter
- `go/preduce`: 並列reduce
- `go/parallel-do`: 複数式の並列実行
- `go/pipeline`: パイプライン処理
- `go/pipeline-map`: パイプラインmap
- `go/pipeline-filter`: パイプラインfilter
- `go/fan-out`: ファンアウト
- `go/fan-in`: ファンイン

### Layer 3 (go/await)

- `go/await`: Promiseを待機
- `go/then`: Promiseチェーン
- `go/catch`: エラーハンドリング
- `go/all`: 複数Promiseを並列実行
- `go/race`: 最速のPromiseを返す

### 状態管理

- `atom`: アトム作成
- `deref` (`@`): 値取得
- `swap!`: 関数で更新（アトミック）
- `reset!`: 値を直接セット
