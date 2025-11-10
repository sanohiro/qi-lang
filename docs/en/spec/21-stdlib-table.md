# Standard Library - Table Processing (table/)

**SQL/awk-like Table Manipulation Library**

The `table` library provides convenient functions for working with tabular data (list of maps) such as CSV, JSON, and database results. Implemented as syntactic sugar over existing collection functions, it enables concise SQL-like operations.

---

## Usage

```qi
(use "table" :as table)
```

---

## Grouping

### table/group-by

Groups a table by key (equivalent to SQL `GROUP BY`).

```qi
;; Group by keyword
(def products [
  {:id 1 :name "Laptop" :category "electronics" :price 1000}
  {:id 2 :name "Mouse" :category "electronics" :price 25}
  {:id 3 :name "Book" :category "books" :price 15}])

(table/group-by products :category)
;=> {"electronics" [{...}, {...}], "books" [{...}]}

;; Group by function (custom logic)
(table/group-by products
  (fn [p]
    (if (< (get p :price) 50)
      "cheap"
      "expensive")))
;=> {"cheap" [{...}, {...}], "expensive" [{...}]}
```

**Parameters**:
- `table` - Table data (list of maps or arrays)
- `key-selector` - Keyword (`:category`) or function

**Returns**: Map in the format `{group-key: [rows...], ...}`

**Implementation**: Wrapper around `list/group-by`

---

## Deduplication

### table/distinct-table

Removes duplicate rows from a table (equivalent to SQL `DISTINCT`).

```qi
(def products [
  {:id 1 :name "Laptop" :category "electronics" :price 1000}
  {:id 2 :name "Mouse" :category "electronics" :price 25}
  {:id 3 :name "Laptop" :category "electronics" :price 1000}])  ;; Duplicate

;; Remove duplicates (all columns)
(table/distinct-table products)
;=> 3 rows → 2 rows (removes exact matches)

;; Remove duplicates (specific keys)
(table/distinct-table products :name :price)
;=> Deduplicate by :name + :price combination
```

**Parameters**:
- `table` - Table data
- `keys...` - Keys to compare (variadic, all columns if omitted)

**Returns**: Deduplicated table

---

## Practical Examples

### Group-by Category Aggregation

```qi
(use "table" :as table)

(def sales-data [
  {:date "2024-01-01" :category "electronics" :amount 1000}
  {:date "2024-01-01" :category "books" :amount 200}
  {:date "2024-01-02" :category "electronics" :amount 1500}
  {:date "2024-01-02" :category "books" :amount 300}])

;; Group by category
(def by-category (table/group-by sales-data :category))
;=> {"electronics" [2 items], "books" [2 items]}

;; Aggregate each group
(def electronics-items (get by-category "String(\"electronics\")"))
(def total (reduce (fn [sum item] (+ sum (get item :amount))) 0 electronics-items))
(println f"Total: {total}")
;=> Total: 2500
```

### Data Cleaning

```qi
;; Remove duplicates from CSV data
(def raw-data (csv/read-file "users.csv"))
(def cleaned (table/distinct-table raw-data :email))
(println f"Deduplicated: {(len raw-data)} → {(len cleaned)} records")
```

### Combined with Pipelines

```qi
(sales-data
 |> (fn [data] (table/group-by data :date))
 |> (fn [grouped] (println f"By date: {(len (keys grouped))} days")))
```

---

## Use Cases

1. **Aggregate CSV or JSON data**
   ```qi
   (def data (csv/read-file "sales.csv"))
   (table/group-by data :date)
   ```

2. **Post-process database results**
   ```qi
   (def results (db/query conn "SELECT * FROM orders"))
   (table/group-by results :customer_id)
   ```

3. **Analyze log files**
   ```qi
   (def logs (json/parse (io/read-file "app.log")))
   (table/group-by logs :level)  ;; Aggregate by ERROR, WARN, INFO
   ```

4. **Clean duplicate data**
   ```qi
   (def cleaned (table/distinct-table data :email))
   ```

5. **awk/SQL-like data manipulation**
   - Achieve SQL `GROUP BY` and `DISTINCT` operations in Qi

---

## Function List

| Function | Description | SQL Equivalent |
|----------|-------------|----------------|
| `table/group-by` | Group by key | `GROUP BY` |
| `table/distinct-table` | Remove duplicates | `DISTINCT` |

---

## References

- **Sample Code**: `examples/23-table-processing.qi`
- **Implementation**: `std/lib/table.qi`
- **Related Functions**:
  - `list/group-by` - Base grouping function
  - `distinct` - Deduplication (all elements)
  - `filter` - Conditional filtering (SQL `WHERE` equivalent)
  - `map` - Projection (SQL `SELECT` equivalent)
  - `reduce` - Aggregation (SQL `SUM`, `AVG` equivalent)

---

## Future Extensions

Phase 2 and beyond may include:

- `table/select` - Column selection (projection)
- `table/where` - Conditional filtering
- `table/order-by` - Sorting
- `table/join` - Table joins
- `table/aggregate` - Aggregation functions (sum, avg, count, etc.)
- `table/pivot` - Pivot tables

The current implementation is Phase 1, providing only the most frequently used functions: `group-by` and `distinct-table`.
