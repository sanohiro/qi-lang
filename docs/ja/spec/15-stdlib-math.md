# 標準ライブラリ：数学関数

**数値演算・数学関数**

---

## 基本演算

Qiは基本的な算術演算子（`+`, `-`, `*`, `/`, `%`）を提供しています。
すべての演算子は整数・浮動小数の両方に対応し、浮動小数が1つでも含まれる場合は浮動小数で返します。

```qi
(+ 1 2 3)        ;; => 6
(+ 1.5 2 3)      ;; => 6.5（浮動小数を含む）
(- 10 3)         ;; => 7
(* 2 3 4)        ;; => 24
(/ 10 2)         ;; => 5
(% 10 3)         ;; => 1（剰余）
(% 10.5 3)       ;; => 1.5（浮動小数にも対応）
```

### 便利な数値関数

```qi
;; min - 最小値
(min 1 2 3)          ;; => 1
(min 1.5 2 3)        ;; => 1.5（浮動小数にも対応）

;; max - 最大値
(max 1 2 3)          ;; => 3
(max 1.5 2 3)        ;; => 3（浮動小数にも対応）

;; sum - 合計（コレクションの要素を合計）
(sum [1 2 3])        ;; => 6
(sum [1.5 2 3])      ;; => 6.5（浮動小数にも対応）

;; inc - インクリメント（1を加算）
(inc 5)              ;; => 6
(inc 5.5)            ;; => 6.5（浮動小数にも対応）

;; dec - デクリメント（1を減算）
(dec 5)              ;; => 4
(dec 5.5)            ;; => 4.5（浮動小数にも対応）

;; abs - 絶対値
(abs -5)             ;; => 5
(abs -5.5)           ;; => 5.5
```

---

## 数学関数

### べき乗・平方根

```qi
;; math/pow - べき乗
(math/pow 2 3)      ;; => 8 (2^3)
(math/pow 10 2)     ;; => 100
(math/pow 2 -1)     ;; => 0.5

;; math/sqrt - 平方根
(math/sqrt 4)       ;; => 2.0
(math/sqrt 9)       ;; => 3.0
(math/sqrt 2)       ;; => 1.4142135623730951
```

### 丸め

```qi
;; math/round - 四捨五入
(math/round 3.4)    ;; => 3
(math/round 3.5)    ;; => 4
(math/round -3.5)   ;; => -4

;; math/floor - 切り捨て（負の無限大方向）
(math/floor 3.9)    ;; => 3
(math/floor -3.1)   ;; => -4

;; math/ceil - 切り上げ（正の無限大方向）
(math/ceil 3.1)     ;; => 4
(math/ceil -3.9)    ;; => -3
```

### 範囲制限

```qi
;; math/clamp - 値を範囲内に制限
(math/clamp 5 0 10)     ;; => 5 (範囲内)
(math/clamp -5 0 10)    ;; => 0 (最小値)
(math/clamp 15 0 10)    ;; => 10 (最大値)
(math/clamp -7 -10 -5)  ;; => -7 (負の範囲)
```

---

## 乱数生成（Feature: std-math）

乱数関連の関数は`std-math` featureが有効な場合に利用可能です。

```qi
;; math/rand - 0.0以上1.0未満の乱数
(math/rand)              ;; => 0.7234...

;; math/rand-int - 整数の乱数（0以上n未満）
(math/rand-int 10)       ;; => 0-9の整数

;; math/random-range - 範囲指定の乱数
(math/random-range 10 20)  ;; => 10-20の整数

;; math/shuffle - リストをシャッフル
(math/shuffle [1 2 3 4 5])  ;; => [3 1 5 2 4] (ランダム)
```

---

## パイプラインでの使用

数学関数はパイプライン演算子と組み合わせて使用できます。

```qi
;; 値の範囲制限と丸め
(16.5 |> math/sqrt |> math/round)
;; => 4

;; 連続計算
(2 |> (math/pow _ 3) |> (math/clamp _ 0 10))
;; => 8

;; データ処理
(map (fn [x] (x |> math/sqrt |> math/round)) [4 9 16 25])
;; => (2 3 4 5)
```

---

## 使用例

### 統計計算

```qi
;; 平方根の平均（RMS）
(defn rms [numbers]
  (let [squares (map (fn [x] (math/pow x 2)) numbers)
        sum (reduce + 0 squares)
        mean (/ sum (len numbers))]
    (math/sqrt mean)))

(rms [1 2 3 4 5])  ;; => 3.316...
```

### 範囲制限

```qi
;; スコアを0-100に制限
(defn normalize-score [score]
  (math/clamp (math/round score) 0 100))

(map normalize-score [-10 45.7 99.2 150])
;; => (0 46 99 100)
```

### 乱数によるサンプリング

```qi
;; ランダムなサンプルを取得
(defn random-sample [n coll]
  (take n (math/shuffle coll)))

(random-sample 3 [1 2 3 4 5 6 7 8 9 10])
;; => (7 2 9) など
```

---

## 関数一覧

### 基本演算・数値関数

| 関数 | 説明 | 例 |
|------|------|-----|
| `+`, `-`, `*`, `/`, `%` | 基本演算（整数・浮動小数対応） | `(+ 1.5 2 3)` → `6.5` |
| `min` | 最小値（整数・浮動小数対応） | `(min 1.5 2 3)` → `1.5` |
| `max` | 最大値（整数・浮動小数対応） | `(max 1.5 2 3)` → `3` |
| `sum` | 合計（整数・浮動小数対応） | `(sum [1.5 2 3])` → `6.5` |
| `inc` | インクリメント（整数・浮動小数対応） | `(inc 5.5)` → `6.5` |
| `dec` | デクリメント（整数・浮動小数対応） | `(dec 5.5)` → `4.5` |
| `abs` | 絶対値 | `(abs -5.5)` → `5.5` |

### 数学関数

| 関数 | 説明 | 例 |
|------|------|-----|
| `math/pow` | べき乗 | `(math/pow 2 3)` → `8` |
| `math/sqrt` | 平方根 | `(math/sqrt 16)` → `4.0` |
| `math/round` | 四捨五入 | `(math/round 3.5)` → `4` |
| `math/floor` | 切り捨て | `(math/floor 3.9)` → `3` |
| `math/ceil` | 切り上げ | `(math/ceil 3.1)` → `4` |
| `math/clamp` | 範囲制限 | `(math/clamp 15 0 10)` → `10` |

### 乱数関数（Feature: std-math）

| 関数 | 説明 | 例 |
|------|------|-----|
| `math/rand` | 乱数(0.0-1.0) | `(math/rand)` → `0.7234...` |
| `math/rand-int` | 整数乱数 | `(math/rand-int 10)` → `0-9` |
| `math/random-range` | 範囲乱数 | `(math/random-range 10 20)` → `10-20` |
| `math/shuffle` | シャッフル | `(math/shuffle [1 2 3])` → `[3 1 2]` |

---

## 注意事項

- **整数と浮動小数点**: `math/pow`は整数を返す場合があります。`math/sqrt`は常に浮動小数点を返します。
- **Feature gates**: 乱数関連の関数（`rand`, `rand-int`, `random-range`, `shuffle`）は`std-math` featureが必要です。
- **範囲制限**: `math/clamp`は引数順序が`(value, min, max)`です。
