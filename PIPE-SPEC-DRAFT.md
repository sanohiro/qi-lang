# Qi パイプライン & Flow 拡張案

## 🎯 目的
- Qi の「A Lisp that flows」哲学を強化
- Unix的パイプラインを言語第一級構文として扱う
- 小さい処理を組み合わせて大きな流れを作る

---

## 1. 演算子軸

| 演算子 | 意味 | 展開先 |
|--------|------|--------|
| `|>`   | 通常パイプ（逐次） | 関数適用 |
| `||>`  | 並列パイプ（pmap） | 並列 map 化 |
| `~>`   | 非同期パイプ（go/chan） | go + chan |

---

## 2. Flow マクロ — 流れのDSL

`flow` はパイプラインを**直感的・構造的に書くためのマクロ**。  
分岐・合流・副作用タップを含みながら、横方向の処理を表現。

```lisp
(flow data
  |> parse
  |> branch
       [valid? |> transform |> save]
       [else   |> handle-error |> log])
```

### 特徴
- `branch`：条件ごとの流れ分岐
- `merge`：複数流れを合流
- `tap>`：副作用観察（ログなど）
- 内部的には `|>` / `||>` / `~>` に展開

---

## 3. ストリーム処理（遅延seq）

```lisp
(files "*.log"
  |> stream
  |> (filter error?)
  |> (map parse-line)
  |> take 10
  |> print)
```
- 大きなデータを遅延seqで処理
- メモリ消費を最小化

---

## 4. 並列 × ストリーム

```lisp
(urls
  ||> http-get
  |> stream-parse
  |> (filter valid?)
  |> process)
```
- `||>` は並列化
- 並列結果もストリームに流せる

---

## 5. 副作用タップ

```lisp
(data
  |> clean
  |> tap> log
  |> analyze
  |> save)
```
- Unixの `tee` 相当
- デバッグやモニタリング用途

---

## 6. 再利用可能な小パイプ

```lisp
(def normalize-text
  (flow
    |> trim
    |> lower
    |> replace-all #"s+" " "))

(texts |> normalize-text |> unique)
```
- 小パイプを関数として再利用可能
- Unixの小さなコマンドを組み合わせる感覚

---

## 7. 簡易マクロ案

```lisp
(mac stream (src)
  `(lazy-seq (open-stream ~src)))

(mac ||> (f)
  `(pmap ~f))

(mac tap> (f)
  `(fn [x] (do (~f x) x)))

(mac flow (& body)
  ;; 各セクションを順にパイプ接続する簡易展開
  (reduce (fn [a b] `(~b ~a)) body))
```

---

## 8. デザイン意図
- 演算子は `|>` / `||>` / `~>` の三種に固定 → シンプル
- `flow` マクロで分岐・合流も表現
- 遅延seqでストリーム化し、巨大データにも対応
- 再利用可能な「小パイプ」が言語文化になる
