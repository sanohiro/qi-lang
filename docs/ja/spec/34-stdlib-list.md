# 標準ライブラリ - リスト拡張操作（list/）

**18個のリスト拡張関数**

リストとベクターに対する高度な操作を提供します。基本的な操作（map、filter、reduce等）はCoreモジュールに含まれています。

> **実装**: `src/builtins/list.rs`
>
> **関連ドキュメント**: [06-data-structures.md](06-data-structures.md) - 基本的なリスト操作

---

## 概要

`list/`モジュールは、Coreモジュールの基本的なリスト操作（map、filter、reduce等）を補完する高度な関数を提供します。

**Coreとの違い**:
- **Core**: 基本操作（map、filter、reduce、take、drop等） - グローバル名前空間
- **list/**: 拡張操作（グループ化、検索、変換、述語等） - `list/`プレフィックスが必要

---

## 条件付き取得・削除

### take-while - 条件を満たす間要素を取得

```qi
;; 基本的な使い方
(take-while (fn [x] (< x 5)) [1 2 3 6 7 4])
;; => (1 2 3)

;; 空白行まで取得
(def lines ["Line 1" "Line 2" "" "Line 3"])
(take-while (fn [s] (not (str/blank? s))) lines)
;; => ("Line 1" "Line 2")

;; パイプラインで使う
([1 2 3 6 7 4] |> (take-while (fn [x] (< x 5))))
;; => (1 2 3)
```

### drop-while - 条件を満たす間要素をスキップ

```qi
;; 基本的な使い方
(drop-while (fn [x] (< x 5)) [1 2 3 6 7 4])
;; => (6 7 4)

;; ヘッダー行をスキップ
(def lines ["# Header" "---" "Content 1" "Content 2"])
(drop-while (fn [s] (str/starts-with? s "#")) lines)
;; => ("---" "Content 1" "Content 2")

;; パイプラインで使う
([1 2 3 6 7 4] |> (drop-while (fn [x] (< x 5))))
;; => (6 7 4)
```

---

## 分割・結合

### list/split-at - 指定位置でリストを分割

```qi
;; 基本的な使い方
(list/split-at 2 [1 2 3 4 5])
;; => [(1 2) (3 4 5)]

;; 先頭3個と残りに分割
(list/split-at 3 ["a" "b" "c" "d" "e"])
;; => [("a" "b" "c") ("d" "e")]

;; パイプラインで使う
([1 2 3 4 5] |> (list/split-at 2))
;; => [(1 2) (3 4 5)]
```

### list/chunk - 固定サイズで分割

```qi
;; 基本的な使い方
(list/chunk 2 [1 2 3 4 5 6])
;; => ((1 2) (3 4) (5 6))

;; 3個ずつ分割
(list/chunk 3 [1 2 3 4 5 6 7 8])
;; => ((1 2 3) (4 5 6) (7 8))

;; パイプラインで使う
([1 2 3 4 5 6] |> (list/chunk 2))
;; => ((1 2) (3 4) (5 6))
```

### list/interleave - 2つのリストを交互に結合

```qi
;; 基本的な使い方
(list/interleave [1 2 3] [4 5 6])
;; => (1 4 2 5 3 6)

;; 短い方に合わせて結合
(list/interleave [1 2] [4 5 6 7])
;; => (1 4 2 5)

;; キーと値を交互に配置
(list/interleave [:a :b :c] [1 2 3])
;; => (:a 1 :b 2 :c 3)
```

### list/zipmap - 2つのリストからマップを生成

```qi
;; 基本的な使い方
(list/zipmap [:a :b :c] [1 2 3])
;; => {:a 1, :b 2, :c 3}

;; キーと値を組み合わせてマップ生成
(def keys [:name :age :email])
(def values ["Alice" 30 "alice@example.com"])
(list/zipmap keys values)
;; => {:name "Alice", :age 30, :email "alice@example.com"}
```

---

## 取得・選択

### list/take-nth - n番目ごとに取得

```qi
;; 基本的な使い方
(list/take-nth 2 [1 2 3 4 5 6])
;; => (1 3 5)

;; 3個おきに取得
(list/take-nth 3 [0 1 2 3 4 5 6 7 8 9])
;; => (0 3 6 9)

;; パイプラインで使う
([1 2 3 4 5 6] |> (list/take-nth 2))
;; => (1 3 5)
```

### list/drop-last - 末尾n個を削除

```qi
;; 基本的な使い方
(list/drop-last 2 [1 2 3 4 5])
;; => (1 2 3)

;; 末尾1個を削除
(list/drop-last 1 [1 2 3])
;; => (1 2)

;; パイプラインで使う
([1 2 3 4 5] |> (list/drop-last 2))
;; => (1 2 3)
```

---

## 検索・探索

### find - 条件を満たす最初の要素

```qi
;; 基本的な使い方
(find (fn [x] (> x 5)) [1 7 3])
;; => 7

;; 偶数を検索
(find even? [1 3 4 5])
;; => 4

;; 見つからない場合はnil
(find (fn [x] (> x 10)) [1 2 3])
;; => nil

;; パイプラインで使う
([1 7 3] |> (find (fn [x] (> x 5))))
;; => 7
```

### list/find-index - 条件を満たす最初の要素のインデックス

```qi
;; 基本的な使い方
(list/find-index (fn [x] (> x 5)) [1 7 3])
;; => 1

;; 偶数のインデックスを検索
(list/find-index even? [1 3 4 5])
;; => 2

;; 見つからない場合はnil
(list/find-index (fn [x] (> x 10)) [1 2 3])
;; => nil
```

---

## 述語（全体チェック）

### list/every? - すべての要素が条件を満たすか

```qi
;; 基本的な使い方
(list/every? (fn [x] (> x 0)) [1 2 3])
;; => true

;; すべて偶数かチェック
(list/every? even? [2 4 6])
;; => true

;; 1つでも条件を満たさない場合
(list/every? even? [2 4 5])
;; => false

;; パイプラインで使う
([1 2 3] |> (list/every? (fn [x] (> x 0))))
;; => true
```

### list/some? - いずれかの要素が条件を満たすか

```qi
;; 基本的な使い方
(list/some? (fn [x] (> x 5)) [1 7 3])
;; => true

;; いずれか偶数があるかチェック
(list/some? even? [1 3 5])
;; => false

;; パイプラインで使う
([1 7 3] |> (list/some? (fn [x] (> x 5))))
;; => true
```

**注**: `some?`（1引数）はCore述語関数として、nilでないかをチェックする述語です。`list/some?`は2引数で、コレクションに対する述語チェックです。

---

## ソート・集約

### list/sort-by - キー関数でソート

```qi
;; 基本的な使い方
(list/sort-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => ({:name "Bob" :age 25} {:name "Alice" :age 30})

;; 文字列長でソート
(list/sort-by len ["zzz" "a" "bb"])
;; => ("a" "bb" "zzz")

;; パイプラインで使う
([{:name "Bob" :age 25} {:name "Alice" :age 30}]
 |> (list/sort-by (fn [u] (get u :age))))
```

### list/max-by - キー関数で最大値を取得

```qi
;; 基本的な使い方
(list/max-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => {:name "Alice" :age 30}

;; 文字列長が最大のものを取得
(list/max-by len ["a" "bbb" "cc"])
;; => "bbb"

;; 空リストの場合はnil
(list/max-by identity [])
;; => nil
```

### list/min-by - キー関数で最小値を取得

```qi
;; 基本的な使い方
(list/min-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => {:name "Bob" :age 25}

;; 文字列長が最小のものを取得
(list/min-by len ["aaa" "b" "cc"])
;; => "b"

;; 空リストの場合はnil
(list/min-by identity [])
;; => nil
```

### list/sum-by - キー関数で合計

```qi
;; 基本的な使い方
(list/sum-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => 55

;; 文字列長の合計
(list/sum-by len ["a" "bb" "ccc"])
;; => 6

;; パイプラインで使う
([{:name "Bob" :age 25} {:name "Alice" :age 30}]
 |> (list/sum-by (fn [u] (get u :age))))
;; => 55
```

---

## グループ化・頻度

### list/frequencies - 出現頻度を集計

```qi
;; 基本的な使い方
(list/frequencies [1 2 2 3 3 3])
;; => {"1" 1, "2" 2, "3" 3}

;; 文字列の頻度
(list/frequencies ["a" "b" "a" "c" "b" "a"])
;; => {"a" 3, "b" 2, "c" 1}

;; パイプラインで使う
([1 2 2 3 3 3] |> list/frequencies)
;; => {"1" 1, "2" 2, "3" 3}
```

### list/partition-by - 連続する値を述語関数でグループ化

```qi
;; 基本的な使い方
(list/partition-by even? [1 1 2 2 3 3])
;; => ((1 1) (2 2) (3 3))

;; 連続する同じ値でグループ化
(list/partition-by identity [1 1 2 2 2 3 3])
;; => ((1 1) (2 2 2) (3 3))

;; 文字列長でグループ化
(list/partition-by len ["a" "b" "cc" "dd" "eee"])
;; => (("a" "b") ("cc" "dd") ("eee"))
```

---

## 変換・加工

### keep - nilを除外してmap

```qi
;; 基本的な使い方
(keep (fn [x] (when (even? x) (* x 2))) [1 2 3 4])
;; => (4 8)

;; nilを返す要素は除外される
(keep (fn [x] (when (> x 2) x)) [1 2 3 4])
;; => (3 4)

;; パイプラインで使う
([1 2 3 4] |> (keep (fn [x] (when (even? x) (* x 2)))))
;; => (4 8)
```

### list/dedupe - 連続する重複を削除

```qi
;; 基本的な使い方
(list/dedupe [1 1 2 2 3 3])
;; => (1 2 3)

;; 非連続の重複は残る
(list/dedupe [1 2 1 2])
;; => (1 2 1 2)

;; パイプラインで使う
([1 1 2 2 3 3] |> list/dedupe)
;; => (1 2 3)
```

**distinct との違い**:
- `distinct`: すべての重複を削除（`[1 2 1 2]` → `[1 2]`）
- `list/dedupe`: 連続する重複のみ削除（`[1 2 1 2]` → `(1 2 1 2)`）

---

## 実用例

### データ分析パイプライン

```qi
;; 偶数のみ取得して合計
([1 2 3 4 5 6]
 |> (filter even?)
 |> (reduce + 0))
;; => 12

;; グループ化して集計
(def data [1 1 2 2 2 3 3])
(data
 |> list/frequencies)
;; => {"1" 2, "2" 3, "3" 2}
```

### ユーザー検索・集計

```qi
(def users [
  {:name "Alice" :age 30 :dept "Sales"}
  {:name "Bob" :age 25 :dept "Dev"}
  {:name "Carol" :age 35 :dept "Sales"}
])

;; 最年長のユーザーを取得
(users |> (list/max-by (fn [u] (get u :age))))
;; => {:name "Carol" :age 35 :dept "Sales"}

;; 年齢の合計
(users |> (list/sum-by (fn [u] (get u :age))))
;; => 90

;; 部署がSalesのユーザーを検索
(users |> (find (fn [u] (= (get u :dept) "Sales"))))
;; => {:name "Alice" :age 30 :dept "Sales"}
```

### データ変換

```qi
;; CSVデータを2列ずつ処理
(def csv-row ["Name" "Alice" "Age" "30" "City" "Tokyo"])
(csv-row |> (list/chunk 2))
;; => (("Name" "Alice") ("Age" "30") ("City" "Tokyo"))

;; キーと値からマップを生成
(list/zipmap [:name :age :city] ["Alice" 30 "Tokyo"])
;; => {:name "Alice", :age 30, :city "Tokyo"}

;; 3個おきにサンプリング
(def large-data (range 1000))
(large-data |> (list/take-nth 100))
;; => (0 100 200 300 400 500 600 700 800 900)
```

### ログ解析

```qi
;; ログファイルの処理
(def logs [
  "INFO: Starting..."
  "INFO: Connected"
  "ERROR: Connection lost"
  "INFO: Retrying..."
])

;; ERRORログを検索
(logs |> (find (fn [s] (str/starts-with? s "ERROR:"))))
;; => "ERROR: Connection lost"

;; すべてINFOかチェック
(logs |> (list/every? (fn [s] (str/starts-with? s "INFO:"))))
;; => false

;; いずれかERRORがあるかチェック
(logs |> (list/some? (fn [s] (str/starts-with? s "ERROR:"))))
;; => true
```

### バッチ処理

```qi
;; 大量のデータを100件ずつ処理
(def data (range 1000))

(data
 |> (list/chunk 100)
 |> (each (fn [batch]
            (println f"Processing batch of {(len batch)} items...")
            ;; バッチ処理ロジック
            )))
```

---

## 関数一覧

### 条件付き取得・削除
- `take-while` - 条件を満たす間要素を取得
- `drop-while` - 条件を満たす間要素をスキップ
- `list/drop-last` - 末尾n個を削除

### 分割・結合
- `list/split-at` - 指定位置で分割
- `list/chunk` - 固定サイズで分割
- `list/interleave` - 2つのリストを交互に結合
- `list/zipmap` - 2つのリストからマップを生成

### 取得・選択
- `list/take-nth` - n番目ごとに取得

### 検索・探索
- `find` - 条件を満たす最初の要素
- `list/find-index` - 条件を満たす最初の要素のインデックス

### 述語
- `list/every?` - すべての要素が条件を満たすか
- `list/some?` - いずれかの要素が条件を満たすか

### ソート・集約
- `list/sort-by` - キー関数でソート
- `list/max-by` - キー関数で最大値を取得
- `list/min-by` - キー関数で最小値を取得
- `list/sum-by` - キー関数で合計

### グループ化・頻度
- `list/frequencies` - 出現頻度を集計
- `list/partition-by` - 連続する値を述語関数でグループ化

### 変換・加工
- `keep` - nilを除外してmap
- `list/dedupe` - 連続する重複を削除
