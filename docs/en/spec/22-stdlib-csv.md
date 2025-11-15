# Standard Library - CSV Processing (csv/)

**RFC 4180 Compliant CSV Processing Library**

All functions belong to the `csv/` module.

---

## Parsing

### csv/parse

Parses a CSV string into a list of lists (`[[string]]`).

```qi
;; Basic usage
(def csv-text "name,age,city
Alice,30,Tokyo
Bob,25,Osaka")

(csv/parse csv-text)
;=> [["name" "age" "city"]
;    ["Alice" "30" "Tokyo"]
;    ["Bob" "25" "Osaka"]]

;; Custom delimiter (TSV)
(def tsv-text "name\tage\tcity
Alice\t30\tTokyo")

(csv/parse tsv-text :delimiter "\t")
;=> [["name" "age" "city"]
;    ["Alice" "30" "Tokyo"]]

;; Quoted fields (RFC 4180 compliant)
(def quoted-csv "name,description
\"Alice, Bob\",\"She said \"\"hello\"\"\"")

(csv/parse quoted-csv)
;=> [["name" "description"]
;    ["Alice, Bob" "She said \"hello\""]]
```

**Arguments**:
- `text` - CSV formatted string
- `:delimiter` - Optional. Delimiter character (default: `","`). Must be a single character

**Returns**: List of lists (`[[string]]`)

**RFC 4180 Compliant Features**:
- Double-quote field enclosing
- `""` escape sequence inside quotes
- CRLF / LF newline support
- Preserves newlines, commas, and quotes inside quoted fields

---

## Serialization

### csv/stringify

Converts a list of lists (or vector of vectors) into a CSV string.

```qi
;; Basic usage
(def data [
  ["name" "age" "city"]
  ["Alice" 30 "Tokyo"]
  ["Bob" 25 "Osaka"]])

(csv/stringify data)
;=> "name,age,city
;    Alice,30,Tokyo
;    Bob,25,Osaka"

;; Supports various types
(def mixed-data [
  ["string" "number" "float" "bool" "nil"]
  ["text" 123 3.14 true nil]])

(csv/stringify mixed-data)
;=> "string,number,float,bool,nil
;    text,123,3.14,true,"

;; Special characters (auto-quoting)
(def special-data [
  ["name" "description"]
  ["Alice, Bob" "She said \"hello\""]])

(csv/stringify special-data)
;=> "name,description
;    \"Alice, Bob\",\"She said \"\"hello\"\"\""
```

**Arguments**:
- `data` - List of lists or vector of vectors

**Returns**: CSV formatted string

**Supported Types**:
- `String` - Output as-is
- `Integer` - Converted to string
- `Float` - Converted to string
- `Bool` - `"true"` / `"false"`
- `Nil` - Empty string

**Auto-quoting**: Fields containing the following are automatically enclosed in double quotes:
- Comma (`,`)
- Double quote (`"`)
- Newline (`\n`, `\r`)

---

## File Reading

### csv/read-file

Reads and parses a CSV file.

```qi
;; Read from file
(def data (csv/read-file "users.csv"))

;; Separate headers and data
(def headers (first data))
(def rows (rest data))

(println f"Columns: {(join \", \" headers)}")
(println f"Rows: {(len rows)}")
```

**Arguments**:
- `path` - CSV file path (string)

**Returns**: List of lists (`[[string]]`)

**Error**: Throws error if file doesn't exist or cannot be read

---

### csv/read-stream

Reads a large CSV file as a stream. Processes rows one-by-one in a memory-efficient manner.

```qi
;; Process large file with stream
(def stream (csv/read-stream "large-data.csv"))

;; Skip header
(def header (stream/next stream))
(println f"Columns: {(join \", \" header)}")

;; Process row by row
(stream/for-each stream
  (fn [row]
    (println f"Processing: {(first row)}")))

;; Pipeline processing
(csv/read-stream "sales.csv")
 |> (stream/drop 1)  ;; Skip header
 |> (stream/filter (fn [row] (> (parse-int (nth row 2)) 1000)))
 |> (stream/take 10)
 |> stream/to-list
```

**Arguments**:
- `path` - CSV file path (string)

**Returns**: Stream (each element is a list)

**Use Cases**:
- Processing large CSV files (tens of MB to GB)
- Memory-efficient row-by-row processing
- Filtering and transformation via pipelines

---

## File Writing

### csv/write-file

Writes data to a CSV file. Pipeline-friendly.

```qi
;; Direct write
(def data [
  ["name" "age" "city"]
  ["Alice" 30 "Tokyo"]
  ["Bob" 25 "Osaka"]])

(csv/write-file data "output.csv")

;; Use in pipeline
(data
 |> (map (fn [row] (map str/upper row)))  ;; Uppercase all
 |> (csv/write-file _ "output-upper.csv"))

;; Transform and save
(csv/read-file "input.csv")
 |> (filter (fn [row] (!= (first row) "name")))  ;; Exclude header
 |> (filter (fn [row] (> (parse-int (nth row 1)) 25)))  ;; age > 25
 |> (csv/write-file _ "filtered.csv")
```

**Arguments**:
- `data` - List of lists or vector of vectors
- `path` - Output file path (string)

**Returns**: `nil`

**Function**: Convenience function combining `csv/stringify` + `io/write-file`

---

## Practical Examples

### Reading and Transforming CSV Files

```qi
;; Read CSV and convert to list of maps
(defn csv->maps [csv-data]
  (let [headers (first csv-data)
        rows (rest csv-data)]
    (map (fn [row]
           (zipmap headers row))
         rows)))

(def users-csv (csv/read-file "users.csv"))
(def users (csv->maps users-csv))
;=> [{:name "Alice" :age "30" :city "Tokyo"}
;    {:name "Bob" :age "25" :city "Osaka"}]

;; Filtering
(def tokyo-users
  (filter (fn [u] (= (get u :city) "Tokyo"))
          users))
```

### Data Cleaning Pipeline

```qi
;; Clean CSV data
(csv/read-file "raw-data.csv")
 |> (map (fn [row]
           (map str/trim row)))  ;; Remove extra whitespace
 |> (filter (fn [row]
              (not (str/blank? (first row)))))  ;; Exclude empty rows
 |> (fn [data]
      (let [headers (first data)
            rows (rest data)]
        (cons headers
              (distinct rows))))  ;; Remove duplicate rows
 |> (csv/write-file _ "cleaned-data.csv")
```

### Large File Aggregation

```qi
;; Count rows matching condition in 1M+ row CSV file
(defn count-high-sales [filepath threshold]
  (let [stream (csv/read-stream filepath)
        _ (stream/next stream)]  ;; Skip header
    (stream
     |> (stream/filter (fn [row]
                         (> (parse-float (nth row 2)) threshold)))
     |> stream/count)))

(println f"High-value sales: {(count-high-sales \"sales.csv\" 10000)} rows")
```

### TSV Processing

```qi
;; Tab-separated values (TSV)
(def tsv-text (io/read-file "data.tsv"))
(def data (csv/parse tsv-text :delimiter "\t"))

;; Convert to CSV and save
(csv/write-file data "data.csv")
```

### Merging and Exporting Data

```qi
;; Merge multiple CSV files
(defn merge-csv-files [files output]
  (let [headers (-> files
                    first
                    csv/read-file
                    first)
        all-rows (files
                  |> (mapcat (fn [f]
                               (-> f
                                   csv/read-file
                                   rest))))]  ;; Exclude headers from each file
    (cons headers all-rows)
     |> (csv/write-file _ output)))

(merge-csv-files ["jan.csv" "feb.csv" "mar.csv"] "q1-sales.csv")
```

### Database Import

```qi
;; Bulk insert CSV data into database
(defn import-csv-to-db [csv-file table-name conn]
  (let [data (csv/read-file csv-file)
        headers (first data)
        rows (rest data)]
    (doseq [row rows]
      (let [values (zipmap (map keyword headers) row)
            sql (str/format "INSERT INTO {} VALUES ({})"
                           table-name
                           (join ", " (repeat (len row) "?")))]
        (db/execute conn sql (vals values))))))

;; Memory-efficient stream import
(defn stream-import-csv-to-db [csv-file table-name conn]
  (let [stream (csv/read-stream csv-file)
        headers (stream/next stream)]  ;; Get headers
    (stream/for-each stream
      (fn [row]
        (let [values (zipmap (map keyword headers) row)]
          (db/insert conn table-name values))))))
```

---

## Error Handling

```qi
;; File read error handling
(try
  (csv/read-file "data.csv")
  (catch e
    (println f"CSV read error: {e}")
    []))

;; Parse error handling
(try
  (csv/parse "invalid\ncsv\ndata")
  (catch e
    (println f"Parse error: {e}")
    []))

;; Write error handling
(try
  (csv/write-file data "/invalid/path/file.csv")
  (catch e
    (println f"Write error: {e}")
    nil))
```

---

## Performance Guide

### Small to Medium Files (< 10MB)

```qi
;; Use csv/read-file (simple)
(def data (csv/read-file "data.csv"))
(doseq [row data]
  (process-row row))
```

### Large Files (> 10MB)

```qi
;; Use csv/read-stream (memory-efficient)
(def stream (csv/read-stream "large-data.csv"))
(stream/for-each stream process-row)
```

### Extra Large Files (> 1GB)

```qi
;; Stream + pipeline (parallelizable)
(csv/read-stream "huge-data.csv")
 |> (stream/drop 1)  ;; Skip header
 |> (stream/map parse-row)
 |> (stream/filter valid-row?)
 |> (stream/for-each process-row)
```

---

## Function Reference

| Function | Description | Use Case |
|----------|-------------|----------|
| `csv/parse` | Parse CSV string | Parse from text |
| `csv/stringify` | Convert data to CSV string | Generate CSV string |
| `csv/read-file` | Read CSV file | Small to medium files |
| `csv/read-stream` | Read CSV file as stream | Large files |
| `csv/write-file` | Write data to CSV file | File output |

---

## Specification Compliance

This library is **RFC 4180** compliant:

- ✅ CRLF / LF newline support
- ✅ Double-quote field enclosing
- ✅ `""` escape for double quotes
- ✅ Preserves newlines and commas in quoted fields
- ✅ Custom delimiters (TSV, etc.)

---

## References

- **RFC 4180**: Common Format and MIME Type for CSV Files
- **Related Modules**:
  - `io/` - File I/O
  - `str/` - String manipulation
  - `stream/` - Stream processing
  - `table/` - Table processing (grouping, aggregation)
