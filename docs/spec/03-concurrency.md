# 並行・並列処理 - Qiの真髄

**Qiは並行・並列処理を第一級市民として扱う言語です。**

「並列、並行を簡単にできるのはQiのキモ」- これがQiの設計哲学の核心です。

---

## 設計哲学

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

---

## Layer 1: go/chan（基盤）

**Go風の並行処理**

### チャネル作成

```qi
(chan)       ;; 無制限バッファ
(chan 10)    ;; バッファサイズ10
```

### 送受信

```qi
;; チャネルへの送信と受信
(def ch (chan))
(def value 42)

(send! ch value)              ;; チャネルに送信
(recv! ch)                    ;; ブロッキング受信
(recv! ch :timeout 1000)      ;; タイムアウト付き受信（ミリ秒）
(try-recv! ch)                ;; 非ブロッキング受信
(close! ch)                   ;; チャネルクローズ
```

### goroutine

```qi
(go (println "async!"))

(def result-ch (chan))
(go (send! result-ch (* 2 3)))
(recv! result-ch)  ;; 6
```

### 使用例: 並列計算

```qi
;; 複数のgoroutineで並列計算
(def ch (chan))

(go (send! ch (* 2 3)))
(go (send! ch (* 4 5)))
(go (send! ch (* 6 7)))

[(recv! ch) (recv! ch) (recv! ch)]  ;; => [6 20 42]
```

### select! - 複数チャネルの待ち合わせ

```qi
;; 複数のチャネルから最初に来たデータを処理
(def ch1 (chan))
(def ch2 (chan))

(go (send! ch1 "from ch1"))
(go (send! ch2 "from ch2"))

(select!
  ch1 (fn [val] (println "Got from ch1:" val))
  ch2 (fn [val] (println "Got from ch2:" val)))
```

### Structured Concurrency（構造化並行処理）

```qi
;; スコープ作成
(def ctx (make-scope))

;; スコープ内でgoroutine起動
(async/scope-go ctx (fn []
  (loop [i 0]
    (if (async/cancelled? ctx)
      (println "cancelled")
      (do
        (println i)
        (sleep 100)
        (recur (inc i)))))))

;; スコープ内の全goroutineをキャンセル
(async/cancel! ctx)

;; with-scope関数（便利版）
(async/with-scope (fn [ctx]
  (async/scope-go ctx task1)
  (async/scope-go ctx task2)
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

;; async/pfilter - 並列filter
([1 2 3 4 5 6] |> (async/pfilter (fn [x] (= (% x 2) 0))))
;; => (2 4 6)

;; async/preduce - 並列reduce
([1 2 3 4 5] |> (async/preduce + 0))
;; => 15

;; async/parallel-do - 複数式の並列実行
(async/parallel-do
  (println "Task 1")
  (println "Task 2")
  (println "Task 3"))
```

### パイプライン処理

```qi
;; pipeline - n並列でxf変換をchに適用
(def ch (chan))
(pipeline 4 (fn [x] (* x 2)) ch)
```

### ファンアウト/ファンイン

```qi
;; fan-out - 1つのチャネルをn個に分岐
(def ch (chan))
(def output-chs (fan-out ch 3))

;; fan-in - 複数チャネルを1つに合流
(def ch1 (chan))
(def ch2 (chan))
(def ch3 (chan))
(def merged (fan-in [ch1 ch2 ch3]))
```

---

## Layer 3: async/await（高レベル）

**モダンな非同期処理**

### 基本的なawait

```qi
(def p (go (fn [] (+ 1 2 3))))
(async/await p)  ;; => 6
```

### Promise チェーン

```qi
(-> (go (fn [] 10))
    (async/then (fn [x] (* x 2)))
    (async/then (fn [x] (+ x 1)))
    (async/await))  ;; => 21
```

### Promise.all風

```qi
(def promises [(go (fn [] 1)) (go (fn [] 2)) (go (fn [] 3))])
(async/await (async/all promises))  ;; => [1 2 3]
```

### Promise.race風

```qi
(def promises [(go (fn [] "slow")) (go (fn [] "fast"))])
(async/await (async/race promises))  ;; => "fast"
```

### エラーハンドリング

```qi
(async/catch promise (fn [e] (println "Error:" e)))
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
(go (swap! counter inc))
(go (swap! counter inc))
(go (swap! counter inc))

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
- **rayon**: データ並列（pmap, async/pfilter, preduce等）
- **parking_lot**: 高性能RwLock
- **Arc<RwLock<_>>**: Evaluatorの完全スレッドセーフ化

---

## 関数一覧

### Layer 1 (go/chan)

- `chan`: チャネル作成
- `send!`: 送信
- `recv!`: ブロッキング受信
- `recv! :timeout`: タイムアウト付き受信
- `try-recv!`: 非ブロッキング受信
- `close!`: チャネルクローズ
- `go`: goroutine起動
- `select!`: 複数チャネル待ち合わせ
- `make-scope`: スコープ作成
- `async/scope-go`: スコープ内goroutine
- `async/cancel!`: スコープキャンセル
- `async/cancelled?`: キャンセル確認
- `async/with-scope`: スコープ自動管理

### Layer 2 (Pipeline)

- `pmap`: 並列map
- `async/pfilter`: 並列filter
- `async/preduce`: 並列reduce
- `async/parallel-do`: 複数式の並列実行
- `pipeline`: パイプライン処理
- `pipeline/map`: パイプラインmap
- `pipeline/filter`: パイプラインfilter
- `fan-out`: ファンアウト
- `fan-in`: ファンイン

### Layer 3 (async/await)

- `async/await`: Promiseを待機
- `async/then`: Promiseチェーン
- `async/catch`: エラーハンドリング
- `async/all`: 複数Promiseを並列実行
- `async/race`: 最速のPromiseを返す

### 状態管理

- `atom`: アトム作成
- `deref` (`@`): 値取得
- `swap!`: 関数で更新（アトミック）
- `reset!`: 値を直接セット
