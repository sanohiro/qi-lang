# 標準ライブラリ：統計関数

**統計計算・データ分析**

`stats/` モジュールは統計計算のための関数を提供します。平均値、中央値、分散、標準偏差などの基本的な統計量を簡単に計算できます。

このモジュールは `std-stats` feature でコンパイルされます。

---

## 概要

統計モジュールは以下の関数を提供します：

- **中心傾向の測度**: mean（平均）, median（中央値）, mode（最頻値）
- **散布度の測度**: variance（分散）, stddev（標準偏差）
- **位置の測度**: percentile（パーセンタイル）

すべての関数はリストまたはベクターを引数として受け取り、数値データ（整数と浮動小数点数）に対応しています。

---

## 中心傾向の測度

### stats/mean - 平均値

データの算術平均を計算します。

```qi
;; 基本的な使用
(stats/mean [1 2 3 4 5])              ;; => 3.0

;; 整数と浮動小数点数の混在
(stats/mean [1 2.5 3])                ;; => 2.166666...

;; リストでも使用可能
(stats/mean '(10 20 30))              ;; => 20.0

;; パイプラインでの使用
([1 2 3 4 5] |> stats/mean)           ;; => 3.0
```

**引数**:
- コレクション（リストまたはベクター）：数値のみを含むコレクション

**戻り値**:
- 浮動小数点数：算術平均

**エラー**:
- 空のコレクションの場合
- 非数値要素が含まれる場合

---

### stats/median - 中央値

データを昇順にソートした際の中央の値を返します。要素数が偶数の場合は、中央2つの値の平均を返します。

```qi
;; 奇数個のデータ
(stats/median [1 2 3 4 5])            ;; => 3.0

;; 偶数個のデータ（中央2つの平均）
(stats/median [1 2 3 4])              ;; => 2.5

;; 順不同のデータ（自動的にソート）
(stats/median [5 1 3 2 4])            ;; => 3.0

;; 浮動小数点数
(stats/median [1.5 2.0 2.5 3.0])      ;; => 2.25
```

**引数**:
- コレクション（リストまたはベクター）：数値のみを含むコレクション

**戻り値**:
- 浮動小数点数：中央値

**エラー**:
- 空のコレクションの場合
- 非数値要素が含まれる場合

---

### stats/mode - 最頻値

最も頻繁に出現する値を返します。

```qi
;; 基本的な使用
(stats/mode [1 2 2 3 3 3])            ;; => 3

;; 整数
(stats/mode [1 1 1 2 2 3])            ;; => 1

;; 浮動小数点数
(stats/mode [1.5 1.5 2.0 2.0 2.0])    ;; => 2.0

;; パイプラインでの使用
([1 2 2 3 3 3] |> stats/mode)         ;; => 3
```

**引数**:
- コレクション（リストまたはベクター）：数値のみを含むコレクション

**戻り値**:
- 数値：最頻値（元のデータ型を保持）

**エラー**:
- 空のコレクションの場合
- 非数値要素が含まれる場合

**注意**:
- 複数の値が同じ最大頻度を持つ場合、そのうちの1つが返されます（どれが返されるかは未定義）

---

## 散布度の測度

### stats/variance - 分散

データの分散（標本分散）を計算します。分散はデータのばらつきを表す指標です。

```qi
;; 基本的な使用
(stats/variance [1 2 3 4 5])          ;; => 2.0

;; 浮動小数点数
(stats/variance [1.0 2.0 3.0])        ;; => 0.666666...

;; パイプラインでの使用
([1 2 3 4 5] |> stats/variance)       ;; => 2.0
```

**計算式**:
```
variance = Σ(xi - mean)² / n
```

**引数**:
- コレクション（リストまたはベクター）：数値のみを含むコレクション

**戻り値**:
- 浮動小数点数：分散

**エラー**:
- 空のコレクションの場合
- 非数値要素が含まれる場合

---

### stats/stddev - 標準偏差

データの標準偏差を計算します。標準偏差は分散の平方根で、データのばらつきを元の単位で表します。

```qi
;; 基本的な使用
(stats/stddev [1 2 3 4 5])            ;; => 1.414213... (√2)

;; 浮動小数点数
(stats/stddev [2 4 6 8])              ;; => 2.236067... (√5)

;; パイプラインでの使用
([1 2 3 4 5] |> stats/stddev)         ;; => 1.414213...
```

**計算式**:
```
stddev = √variance
```

**引数**:
- コレクション（リストまたはベクター）：数値のみを含むコレクション

**戻り値**:
- 浮動小数点数：標準偏差

**エラー**:
- 空のコレクションの場合
- 非数値要素が含まれる場合

---

## 位置の測度

### stats/percentile - パーセンタイル

指定されたパーセンタイル位置の値を計算します。線形補間法を使用します。

```qi
;; 50パーセンタイル（中央値と同じ）
(stats/percentile [1 2 3 4 5] 50)     ;; => 3.0

;; 95パーセンタイル
(stats/percentile [1 2 3 4 5] 95)     ;; => 4.8

;; 25パーセンタイル（第1四分位数）
(stats/percentile [1 2 3 4 5] 25)     ;; => 2.0

;; 75パーセンタイル（第3四分位数）
(stats/percentile [1 2 3 4 5] 75)     ;; => 4.0

;; 浮動小数点数のパーセンタイル値
(stats/percentile [1 2 3 4 5] 50.5)   ;; => 3.02

;; パイプラインでの使用
([1 2 3 4 5] |> (stats/percentile _ 95))  ;; => 4.8
```

**引数**:
- コレクション（リストまたはベクター）：数値のみを含むコレクション
- パーセンタイル値（整数または浮動小数点数）：0〜100の範囲

**戻り値**:
- 浮動小数点数：指定されたパーセンタイル位置の値

**エラー**:
- 空のコレクションの場合
- 非数値要素が含まれる場合
- パーセンタイル値が0〜100の範囲外の場合

**注意**:
- 線形補間法を使用するため、実際のデータに存在しない値が返される場合があります

---

## パイプラインでの使用

統計関数はQiのパイプライン演算子と自然に組み合わせて使用できます。

```qi
;; データ処理パイプライン
(def data [10 20 30 40 50])

(data
 |> stats/mean
 |> (math/round _))
;; => 30

;; 複数の統計量を計算
(defn summary-stats [data]
  {:mean (stats/mean data)
   :median (stats/median data)
   :stddev (stats/stddev data)
   :min (apply min data)
   :max (apply max data)})

(summary-stats [1 2 3 4 5])
;; => {:mean 3.0, :median 3.0, :stddev 1.414..., :min 1, :max 5}

;; フィルタリングと統計
([1 2 3 4 5 6 7 8 9 10]
 |> (filter (fn [x] (> x 5)))
 |> stats/mean)
;; => 8.0
```

---

## 実用例

### データ分析パイプライン

```qi
;; テストスコアの分析
(def test-scores [85 90 78 92 88 76 95 89 84 91])

(defn analyze-scores [scores]
  (let [sorted (sort scores)
        n (len scores)]
    {:count n
     :mean (stats/mean scores)
     :median (stats/median scores)
     :stddev (stats/stddev scores)
     :min (first sorted)
     :max (last sorted)
     :q1 (stats/percentile scores 25)
     :q3 (stats/percentile scores 75)
     :p95 (stats/percentile scores 95)}))

(analyze-scores test-scores)
;; => {:count 10
;;     :mean 86.8
;;     :median 88.5
;;     :stddev 5.68...
;;     :min 76
;;     :max 95
;;     :q1 83.25
;;     :q3 91.25
;;     :p95 94.2}
```

### 異常値検出

```qi
;; 標準偏差を使った異常値検出（3シグマ法）
(defn detect-outliers [data threshold]
  (let [m (stats/mean data)
        sd (stats/stddev data)
        lower (- m (* threshold sd))
        upper (+ m (* threshold sd))]
    (filter (fn [x] (or (< x lower) (> x upper))) data)))

(def data [10 12 11 13 100 12 11 10 12])
(detect-outliers data 3)
;; => [100]  （100は異常値）
```

### 正規化（z-score）

```qi
;; データをz-scoreに変換（平均0、標準偏差1に正規化）
(defn z-score [data]
  (let [m (stats/mean data)
        sd (stats/stddev data)]
    (map (fn [x] (/ (- x m) sd)) data)))

(z-score [1 2 3 4 5])
;; => [-1.414... -0.707... 0.0 0.707... 1.414...]
```

### パーセンタイルによるランク付け

```qi
;; スコアのパーセンタイルランクを計算
(defn percentile-rank [data value]
  (let [sorted (sort data)
        n (len sorted)
        below (len (filter (fn [x] (< x value)) sorted))
        equal (len (filter (fn [x] (= x value)) sorted))]
    (* 100.0 (/ (+ below (/ equal 2.0)) n))))

(def scores [70 80 85 90 95])
(percentile-rank scores 85)
;; => 50.0  （85は50パーセンタイル）
```

### グループ別統計

```qi
;; カテゴリ別の統計量を計算
(def data
  [{:category "A" :value 10}
   {:category "A" :value 20}
   {:category "B" :value 30}
   {:category "B" :value 40}
   {:category "B" :value 50}])

(defn group-stats [data key-fn value-fn]
  (let [grouped (group-by key-fn data)]
    (map-vals
      (fn [items]
        (let [values (map value-fn items)]
          {:mean (stats/mean values)
           :median (stats/median values)
           :count (len values)}))
      grouped)))

(group-stats data
  (fn [x] (get x :category))
  (fn [x] (get x :value)))
;; => {"A" {:mean 15.0, :median 15.0, :count 2}
;;     "B" {:mean 40.0, :median 40.0, :count 3}}
```

### 移動平均

```qi
;; 単純移動平均（Simple Moving Average）
(defn moving-average [data window-size]
  (let [windows (partition window-size 1 data)]
    (map stats/mean windows)))

(moving-average [1 2 3 4 5 6 7 8 9 10] 3)
;; => [2.0 3.0 4.0 5.0 6.0 7.0 8.0 9.0]
```

### 相関係数の基礎計算

```qi
;; 2つのデータセットの共分散を計算
(defn covariance [x-data y-data]
  (let [n (len x-data)
        x-mean (stats/mean x-data)
        y-mean (stats/mean y-data)
        products (map (fn [x y] (* (- x x-mean) (- y y-mean)))
                      x-data y-data)]
    (/ (reduce + 0 products) n)))

(def x [1 2 3 4 5])
(def y [2 4 6 8 10])
(covariance x y)
;; => 4.0
```

---

## エラーハンドリング

統計関数は以下の場合にエラーを返します：

```qi
;; 空のコレクション
(stats/mean [])
;; => Error: stats/mean: collection must not be empty

;; 非数値要素
(stats/mean [1 2 "three" 4])
;; => Error: stats/mean: all elements must be numbers

;; 無効なパーセンタイル値
(stats/percentile [1 2 3] 150)
;; => Error: stats/percentile: invalid percentile (must be 0-100)

;; try/okを使ったエラーハンドリング
(try
  (stats/mean [])
  (fn [result]
    (match result
      [:ok value] (println f"Mean: {value}")
      [:error msg] (println f"Error: {msg}"))))
```

---

## 関数一覧

| 関数 | 説明 | 引数 | 戻り値 |
|------|------|------|--------|
| `stats/mean` | 算術平均 | コレクション | Float |
| `stats/median` | 中央値 | コレクション | Float |
| `stats/mode` | 最頻値 | コレクション | Number |
| `stats/variance` | 分散 | コレクション | Float |
| `stats/stddev` | 標準偏差 | コレクション | Float |
| `stats/percentile` | パーセンタイル | コレクション, パーセンタイル値(0-100) | Float |

---

## 数学関数との連携

統計関数は`math/`モジュールと組み合わせて使用できます：

```qi
;; 変動係数（Coefficient of Variation）
(defn cv [data]
  (let [m (stats/mean data)
        sd (stats/stddev data)]
    (* 100.0 (/ sd m))))

(cv [10 12 11 13 12])
;; => 10.5...  （変動係数: 10.5%）

;; 標準化されたデータの範囲
(defn standardized-range [data]
  (let [z-scores (z-score data)]
    {:min (apply min z-scores)
     :max (apply max z-scores)
     :range (- (apply max z-scores) (apply min z-scores))}))
```

---

## パフォーマンスに関する注意

- **ソート**: `median`と`percentile`はデータをソートするため、O(n log n)の時間計算量です
- **複数回の計算**: 同じデータに対して複数の統計量を計算する場合、一度に計算した方が効率的です
- **大規模データ**: 非常に大きなデータセット（100万要素以上）の場合、メモリ使用量に注意してください

```qi
;; 効率的な複数統計量の計算
(defn efficient-stats [data]
  ;; 一度だけ平均を計算し、分散と標準偏差で再利用
  (let [m (stats/mean data)
        v (stats/variance data)
        sd (math/sqrt v)]  ;; varianceから計算
    {:mean m
     :variance v
     :stddev sd}))
```

---

## 今後の拡張予定

将来のバージョンでは以下の機能が追加される可能性があります：

- 標本分散（不偏分散）のサポート
- 歪度（skewness）と尖度（kurtosis）
- 相関係数（correlation coefficient）
- 回帰分析の基本関数
- ヒストグラム生成
- 箱ひげ図のデータ計算

---

## 参考

- [数学関数](15-stdlib-math.md) - 数値演算・乱数生成
- [コレクション操作](05-syntax-basics.md) - filter, map, reduce
- [エラーハンドリング](08-error-handling.md) - try/ok, matchによるエラー処理
