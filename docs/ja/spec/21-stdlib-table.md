# 標準ライブラリ - テーブル処理（table/）

**SQL/awkライクなテーブル操作ライブラリ**

`table` ライブラリは、CSV、JSON、データベース結果などの表形式データ（list of maps）を扱うための便利な関数を提供します。既存のコレクション関数のシンタックスシュガーとして実装されており、SQL的な操作を簡潔に記述できます。

---

## 使い方

```qi
(use "table" :as table)
```

---

## グループ化

### table/group-by

テーブルをキーでグループ化します（SQLの `GROUP BY` 相当）。

```qi
;; キーワードでグループ化
(def products [
  {:id 1 :name "Laptop" :category "electronics" :price 1000}
  {:id 2 :name "Mouse" :category "electronics" :price 25}
  {:id 3 :name "Book" :category "books" :price 15}])

(table/group-by products :category)
;=> {"electronics" [{...}, {...}], "books" [{...}]}

;; 関数でグループ化（カスタムロジック）
(table/group-by products
  (fn [p]
    (if (< (get p :price) 50)
      "cheap"
      "expensive")))
;=> {"cheap" [{...}, {...}], "expensive" [{...}]}
```

**引数**:
- `table` - テーブルデータ（list of maps or arrays）
- `key-selector` - キーワード（`:category`）または関数

**戻り値**: `{group-key: [rows...], ...}` 形式のマップ

**実装**: `list/group-by` のラッパー

---

## 重複除去

### table/distinct-table

テーブルから重複行を除去します（SQLの `DISTINCT` 相当）。

```qi
(def products [
  {:id 1 :name "Laptop" :category "electronics" :price 1000}
  {:id 2 :name "Mouse" :category "electronics" :price 25}
  {:id 3 :name "Laptop" :category "electronics" :price 1000}])  ;; 重複

;; 全カラムで重複除去
(table/distinct-table products)
;=> 3行 → 2行（完全に一致する行を除去）

;; 特定キーで重複除去
(table/distinct-table products :name :price)
;=> :name と :price の組み合わせで重複除去
```

**引数**:
- `table` - テーブルデータ
- `keys...` - 比較するキー（可変長引数、省略時は全カラム）

**戻り値**: 重複除去されたテーブル

---

## 実用例

### カテゴリ別の集計

```qi
(use "table" :as table)

(def sales-data [
  {:date "2024-01-01" :category "electronics" :amount 1000}
  {:date "2024-01-01" :category "books" :amount 200}
  {:date "2024-01-02" :category "electronics" :amount 1500}
  {:date "2024-01-02" :category "books" :amount 300}])

;; カテゴリ別にグループ化
(def by-category (table/group-by sales-data :category))
;=> {"electronics" [2 items], "books" [2 items]}

;; 各グループの集計
(def electronics-items (get by-category "String(\"electronics\")"))
(def total (reduce (fn [sum item] (+ sum (get item :amount))) 0 electronics-items))
(println f"Total: {total}")
;=> Total: 2500
```

### データクリーニング

```qi
;; CSVからロードしたデータの重複除去
(def raw-data (csv/read-file "users.csv"))
(def cleaned (table/distinct-table raw-data :email))
(println f"重複除去: {(len raw-data)} → {(len cleaned)} 件")
```

### パイプラインとの組み合わせ

```qi
(sales-data
 |> (fn [data] (table/group-by data :date))
 |> (fn [grouped] (println f"日付別: {(len (keys grouped))} 日分")))
```

---

## こんな時に使う

1. **CSVやJSONからロードしたデータの集計**
   ```qi
   (def data (csv/read-file "sales.csv"))
   (table/group-by data :date)
   ```

2. **データベース結果の後処理**
   ```qi
   (def results (db/query conn "SELECT * FROM orders"))
   (table/group-by results :customer_id)
   ```

3. **ログファイルの分析**
   ```qi
   (def logs (json/parse (io/read-file "app.log")))
   (table/group-by logs :level)  ;; ERROR, WARN, INFO ごとに集計
   ```

4. **重複データのクリーニング**
   ```qi
   (def cleaned (table/distinct-table data :email))
   ```

5. **awk/SQL的なデータ操作**
   - SQLの `GROUP BY` や `DISTINCT` と同様の操作をQiで実現

---

## 関数一覧

| 関数 | 説明 | SQL相当 |
|------|------|---------|
| `table/group-by` | キーでグループ化 | `GROUP BY` |
| `table/distinct-table` | 重複除去 | `DISTINCT` |

---

## 参考

- **サンプルコード**: `examples/23-table-processing.qi`
- **実装**: `std/lib/table.qi`
- **関連関数**:
  - `list/group-by` - 基底のグループ化関数
  - `distinct` - 重複除去（全要素）
  - `filter` - 条件フィルタ（SQLの `WHERE` 相当）
  - `map` - 射影（SQLの `SELECT` 相当）
  - `reduce` - 集約（SQLの `SUM`, `AVG` 相当）

---

## 今後の拡張予定

Phase 2 以降で以下の機能を検討中：

- `table/select` - カラム選択（射影）
- `table/where` - 条件フィルタ
- `table/order-by` - ソート
- `table/join` - テーブル結合
- `table/aggregate` - 集約関数（sum, avg, count等）
- `table/pivot` - ピボットテーブル

現在の実装はPhase 1として、最も使用頻度の高い`group-by`と`distinct-table`のみを提供しています。
