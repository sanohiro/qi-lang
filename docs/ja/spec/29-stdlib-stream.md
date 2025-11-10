# 標準ライブラリ - ストリーム（stream/）

**遅延評価による効率的なデータ処理 - メモリを節約し、無限データ構造を扱う**

Qiのストリームは、値を必要になるまで計算しない遅延評価（lazy evaluation）のデータ構造です。
メモリ効率的に大量データを処理でき、無限データ構造も扱えます。

> **実装**: `src/builtins/stream.rs`
>
> **特徴**:
> - 遅延評価により必要な分だけ計算
> - 無限データ構造（無限リスト、無限数列など）のサポート
> - パイプライン演算子（`|>`）との完璧な統合
> - ファイル/HTTPのストリーミングI/O対応

---

## ストリームとは

### 通常のコレクションとの違い

```qi
;; 通常のリスト: すべての要素をメモリに保持
(def nums [1 2 3 4 5 6 7 8 9 10])
(def squares (map (fn [x] (* x x)) nums))
;; => [1 4 9 16 25 36 49 64 81 100] （すべて計算済み）

;; ストリーム: 必要な分だけ計算
(def s (stream/range 1 11))
(def squares-stream (stream/map (fn [x] (* x x)) s))
;; => <Stream> （まだ計算していない）

;; 最初の3個だけ取得
(stream/realize (stream/take 3 squares-stream))
;; => (1 4 9) （3個だけ計算された）
```

### メリット

1. **メモリ効率**: 必要な分だけメモリを使用
2. **無限データ構造**: 終わりのないデータを表現可能
3. **計算の最適化**: 不要な計算をスキップ
4. **I/Oストリーミング**: 大きなファイルを一度に読み込まない

---

## ストリーム生成

### stream/stream - コレクションからストリーム作成

```qi
;; リストからストリーム
(def s (stream/stream [1 2 3 4 5]))
(stream/realize s)  ;; => (1 2 3 4 5)

;; ベクターからストリーム
(def s (stream/stream [10 20 30]))
(stream/realize s)  ;; => (10 20 30)
```

**引数**:
- `coll`: リストまたはベクター

**戻り値**: Stream

---

### stream/range - 範囲ストリーム

```qi
;; 0から9まで
(def s (stream/range 0 10))
(stream/realize s)  ;; => (0 1 2 3 4 5 6 7 8 9)

;; 1から5まで
(def s (stream/range 1 6))
(stream/realize s)  ;; => (1 2 3 4 5)

;; 空の範囲
(def s (stream/range 5 5))
(stream/realize s)  ;; => ()
```

**引数**:
- `start`: 開始値（含む）
- `end`: 終了値（含まない）

**戻り値**: Stream

---

### stream/iterate - 関数の反復適用による無限ストリーム

```qi
;; 1, 2, 4, 8, 16, 32, ...（2倍ずつ）
(def s (stream/iterate (fn [x] (* x 2)) 1))
(stream/realize (stream/take 6 s))  ;; => (1 2 4 8 16 32)

;; 1, 2, 3, 4, 5, ...（1ずつ増加）
(def s (stream/iterate inc 1))
(stream/realize (stream/take 5 s))  ;; => (1 2 3 4 5)

;; フィボナッチ数列
(def fib-stream (stream/iterate (fn [[a b]] [b (+ a b)]) [0 1]))
(stream/realize
  (stream/take 10 fib-stream)
  |> (map first))
;; => (0 1 1 2 3 5 8 13 21 34)
```

**引数**:
- `f`: 各要素に適用する関数
- `initial`: 初期値

**戻り値**: Stream（無限）

---

### stream/repeat - 同じ値の無限ストリーム

```qi
;; 42を無限に繰り返し
(def s (stream/repeat 42))
(stream/realize (stream/take 5 s))  ;; => (42 42 42 42 42)

;; 文字列を繰り返し
(def s (stream/repeat "hello"))
(stream/realize (stream/take 3 s))  ;; => ("hello" "hello" "hello")
```

**引数**:
- `value`: 繰り返す値

**戻り値**: Stream（無限）

---

### stream/cycle - リストを循環する無限ストリーム

```qi
;; [1 2 3] を循環
(def s (stream/cycle [1 2 3]))
(stream/realize (stream/take 8 s))  ;; => (1 2 3 1 2 3 1 2)

;; 曜日の循環
(def days (stream/cycle ["月" "火" "水" "木" "金" "土" "日"]))
(stream/realize (stream/take 10 days))
;; => ("月" "火" "水" "木" "金" "土" "日" "月" "火" "水")
```

**引数**:
- `coll`: リストまたはベクター（空でないこと）

**戻り値**: Stream（無限）

**エラー**:
- 空のコレクションを渡すとエラー

---

## ストリーム操作

### stream/map - 各要素に関数を適用

```qi
;; 各要素を2倍
(def s (stream/range 1 6))
(def s2 (stream/map (fn [x] (* x 2)) s))
(stream/realize s2)  ;; => (2 4 6 8 10)

;; 文字列の長さを取得
(def s (stream/stream ["hello" "world" "qi"]))
(def s2 (stream/map len s))
(stream/realize s2)  ;; => (5 5 2)

;; 無限ストリームにも適用可能
(def s (stream/iterate inc 1))
(def squares (stream/map (fn [x] (* x x)) s))
(stream/realize (stream/take 5 squares))  ;; => (1 4 9 16 25)
```

**引数**:
- `f`: 各要素に適用する関数
- `stream`: ストリーム

**戻り値**: Stream

---

### stream/filter - 条件に合う要素のみ

```qi
;; 偶数のみ
(def s (stream/range 1 11))
(def evens (stream/filter (fn [x] (= (% x 2) 0)) s))
(stream/realize evens)  ;; => (2 4 6 8 10)

;; 10より大きい数
(def s (stream/range 1 21))
(def large (stream/filter (fn [x] (> x 10)) s))
(stream/realize large)  ;; => (11 12 13 14 15 16 17 18 19 20)

;; 無限ストリームから3の倍数
(def s (stream/iterate inc 1))
(def multiples (stream/filter (fn [x] (= (% x 3) 0)) s))
(stream/realize (stream/take 5 multiples))  ;; => (3 6 9 12 15)
```

**引数**:
- `pred`: 述語関数（trueを返す要素のみ残す）
- `stream`: ストリーム

**戻り値**: Stream

---

### stream/take - 最初のn個を取得

```qi
;; 最初の5個
(def s (stream/range 0 100))
(def s2 (stream/take 5 s))
(stream/realize s2)  ;; => (0 1 2 3 4)

;; 無限ストリームを有限化
(def s (stream/repeat 42))
(def s2 (stream/take 3 s))
(stream/realize s2)  ;; => (42 42 42)

;; 0個
(def s (stream/range 1 10))
(def s2 (stream/take 0 s))
(stream/realize s2)  ;; => ()
```

**引数**:
- `n`: 取得する要素数（0以上の整数）
- `stream`: ストリーム

**戻り値**: Stream

---

### stream/drop - 最初のn個をスキップ

```qi
;; 最初の5個をスキップ
(def s (stream/range 0 10))
(def s2 (stream/drop 5 s))
(stream/realize s2)  ;; => (5 6 7 8 9)

;; 0個スキップ（何もしない）
(def s (stream/range 1 4))
(def s2 (stream/drop 0 s))
(stream/realize s2)  ;; => (1 2 3)

;; すべてスキップ
(def s (stream/range 1 4))
(def s2 (stream/drop 10 s))
(stream/realize s2)  ;; => ()
```

**引数**:
- `n`: スキップする要素数（0以上の整数）
- `stream`: ストリーム

**戻り値**: Stream

---

### stream/realize - ストリームをリストに変換

```qi
;; ストリームを実行してリストに変換
(def s (stream/range 1 6))
(stream/realize s)  ;; => (1 2 3 4 5)

;; 複雑な変換チェーンの実行
(def s (stream/range 1 11))
(stream/realize
  (s
   |> (stream/map (fn [x] (* x x)))
   |> (stream/filter (fn [x] (> x 20)))))
;; => (25 36 49 64 81 100)
```

**引数**:
- `stream`: ストリーム

**戻り値**: List

**注意**:
- 無限ストリームを`realize`すると無限ループになります
- 無限ストリームは必ず`take`で有限化してから`realize`すること

```qi
;; ❌ 悪い例: 無限ループ
(stream/realize (stream/repeat 42))  ;; 永遠に終わらない

;; ✅ 良い例: takeで有限化
(stream/realize (stream/take 5 (stream/repeat 42)))  ;; OK
```

---

## パイプラインとの統合

Qiのパイプライン演算子（`|>`）でストリーム操作を簡潔に記述できます。

### 基本的なパイプライン

```qi
;; ストリーム作成から実行まで
[1 2 3 4 5]
  |> stream/stream
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (> x 10)))
  |> stream/realize
;; => (16 25)

;; 範囲ストリーム
(stream/range 1 11)
  |> (stream/map (fn [x] (* x 2)))
  |> (stream/filter (fn [x] (> x 10)))
  |> (stream/take 3)
  |> stream/realize
;; => (12 14 16)
```

### 無限ストリームのパイプライン

```qi
;; 2のべき乗の最初の10個
1
  |> (stream/iterate (fn [x] (* x 2)))
  |> (stream/take 10)
  |> stream/realize
;; => (1 2 4 8 16 32 64 128 256 512)

;; 3の倍数で100未満のもの
1
  |> (stream/iterate inc)
  |> (stream/filter (fn [x] (= (% x 3) 0)))
  |> (stream/take-while (fn [x] (< x 100)))
  |> stream/realize
;; => (3 6 9 12 ... 96 99)
```

### 複雑な変換チェーン

```qi
;; 1から99までの平方数で3の倍数のもの
(stream/range 1 100)
  |> (stream/map (fn [x] (* x x)))
  |> (stream/filter (fn [x] (= (% x 3) 0)))
  |> (stream/take 5)
  |> stream/realize
;; => (9 36 81 144 225)

;; フィボナッチ数列の偶数項
(stream/iterate (fn [[a b]] [b (+ a b)]) [0 1])
  |> (stream/map first)
  |> (stream/filter (fn [x] (= (% x 2) 0)))
  |> (stream/take 8)
  |> stream/realize
;; => (0 2 8 34 144 610 2584 10946)
```

---

## 実用例

### 素数の無限ストリーム

```qi
;; 素数判定関数
(defn prime? [n]
  (if (< n 2)
    false
    (not (any? (fn [i] (= (% n i) 0)) (range 2 (+ 1 (sqrt n)))))))

;; 素数の無限ストリーム
(def primes
  (2
   |> (stream/iterate inc)
   |> (stream/filter prime?)))

;; 最初の20個の素数
(stream/realize (stream/take 20 primes))
;; => (2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71)

;; 100未満の素数
(stream/realize
  (stream/take-while (fn [x] (< x 100)) primes))
```

### 大量データの集計

```qi
;; 1から100万までの偶数の合計（メモリ効率的）
(stream/range 1 1000001)
  |> (stream/filter (fn [x] (= (% x 2) 0)))
  |> stream/realize
  |> (reduce + 0)
;; => 250000500000

;; 通常のリストだと100万個のメモリを消費するが、
;; ストリームなら必要な分だけ計算
```

### データ処理パイプライン

```qi
;; ログファイルのエラー抽出（概念）
(defn process-logs [data]
  (data
   |> stream/stream
   |> (stream/filter (fn [line] (str/contains? line "ERROR")))
   |> (stream/map parse-log-line)
   |> (stream/take 100)  ; 最初の100エラー
   |> stream/realize))

;; CSVデータの変換
(defn process-csv [rows]
  (rows
   |> stream/stream
   |> (stream/drop 1)  ; ヘッダースキップ
   |> (stream/map (fn [row] (str/split row ",")))
   |> (stream/filter (fn [cols] (> (len cols) 3)))
   |> (stream/map (fn [cols] {:id (first cols) :name (second cols)}))
   |> (stream/take 1000)
   |> stream/realize))
```

### 無限データ生成

```qi
;; 無限カウンター
(def counter (stream/iterate inc 0))

;; 時刻スタンプの無限シーケンス（概念）
(def timestamps
  (now)
   |> (stream/iterate (fn [t] (+ t 1000))))  ; 1秒ずつ

;; ランダム数値の無限ストリーム（概念）
(def randoms
  (stream/iterate (fn [_] (math/rand)) 0))
```

---

## ストリームのメリットとデメリット

### メリット

1. **メモリ効率**
   - 巨大なデータセットを扱える
   - 必要な分だけメモリを使用

2. **無限データ構造**
   - 終わりのないデータを表現可能
   - 数学的な概念をそのまま表現

3. **計算の遅延**
   - 不要な計算をスキップ
   - 早期終了が可能（take）

4. **合成可能**
   - map/filter/takeを自由に組み合わせ
   - パイプラインで直感的に記述

### デメリット

1. **ランダムアクセス不可**
   - インデックスで要素にアクセスできない
   - 先頭から順番に処理するのみ

2. **再利用不可**
   - ストリームは一度しか実行できない
   - 複数回使う場合は再生成が必要

3. **デバッグが難しい**
   - 遅延評価のため、どこで計算されるか不明瞭
   - `tap>`で観察すること

---

## パフォーマンスガイドライン

### ストリームを使うべき場合

- ✅ 大量データの処理（ファイル、ログ、データベース）
- ✅ 無限データ構造（数列、ジェネレーター）
- ✅ 早期終了が必要（最初のn個だけ）
- ✅ メモリが限られている

### 通常のコレクションを使うべき場合

- ❌ 小さなデータセット（数十〜数百要素）
- ❌ ランダムアクセスが必要
- ❌ 複数回走査が必要
- ❌ すべての要素を保持する必要がある

### パフォーマンス比較

```qi
;; ストリーム: メモリ効率的、遅延評価
(stream/range 1 1000001)
  |> (stream/map (fn [x] (* x x)))
  |> (stream/take 10)
  |> stream/realize
;; => 10個だけ計算、残りは計算しない

;; 通常のリスト: すべて計算
(range 1 1000001)
  |> (map (fn [x] (* x x)))
  |> (take 10)
;; => 100万個すべて計算してから10個取得（無駄）
```

---

## まとめ

Qiのストリームは、遅延評価により以下を実現します：

- **メモリ効率**: 大量データを扱える
- **無限データ構造**: 数学的な概念を表現
- **パイプライン統合**: `|>`で直感的に記述
- **早期終了**: 必要な分だけ計算

ストリームは、ファイルI/O、データ処理、無限数列など、様々な場面で威力を発揮します。
