# Standard Library - File I/O

**File Operations and Encoding Support**

---

## Basic I/O

### Output

```qi
;; print - Standard output (without newline)
(print "Hello")                     ;; Hello (no newline)

;; println - Standard output (with newline)
(println "Hello")                   ;; Hello\n
```

### Standard Input

```qi
;; io/stdin-line - Read one line from standard input (nil on EOF)
(io/stdin-line)                     ;; => "user input"
(io/stdin-line)                     ;; => nil (EOF)

;; Usage example: Process line by line
(defn process []
  (let [line (io/stdin-line)]
    (if (some? line)
      (do
        (line |> str/trim |> str/upper |> println)
        (process))
      nil)))

(process)

;; io/stdin-lines - Read all lines from standard input (for pipeline processing)
(io/stdin-lines)                    ;; => ["line1" "line2" "line3"]

;; Usage example: Pipeline processing (most Qi-like approach)
(io/stdin-lines
 |> (map str/trim)
 |> (filter (fn [s] (not (str/empty? s))))
 |> (map str/upper)
 |> (map println))

;; Combining with Unix pipes
;; $ cat data.txt | qi process.qi
```

### File Reading

```qi
;; io/read-file - Read entire file
(io/read-file "data.txt")           ;; => "file content..."

;; io/read-lines - Read line by line (memory efficient)
(io/read-lines "data.txt")          ;; => ["line1" "line2" "line3"]
```

### File Writing

```qi
;; io/write-file - Write to file (overwrite)
(io/write-file "Hello, Qi!" "/tmp/test.txt")

;; io/append-file - Append to file
(io/append-file "\nSecond line" "/tmp/test.txt")
```

### File Existence Check

```qi
;; io/file-exists? - Check file existence
(io/file-exists? "/tmp/test.txt")   ;; => true
```

---

## Encoding Support

### Unicode

```qi
;; :utf-8 (default, automatic BOM removal)
(io/read-file "data.txt")

;; :utf-8-bom (UTF-8 with BOM, for Excel compatibility)
(io/write-file data "excel.csv" :encoding :utf-8-bom)

;; :utf-16le (UTF-16LE with BOM, Excel multilingual support)
(io/write-file data "multilang_excel.csv" :encoding :utf-16le)

;; :utf-16be (UTF-16BE with BOM)
(io/write-file data "data.txt" :encoding :utf-16be)
```

### Japanese

```qi
;; :sjis / :shift-jis (Shift_JIS/Windows-31J, Japanese Windows/Excel)
(io/read-file "legacy.csv" :encoding :sjis)
(io/write-file data "for_excel.csv" :encoding :sjis)

;; :euc-jp (EUC-JP, Unix systems)
(io/read-file "unix_text.txt" :encoding :euc-jp)

;; :iso-2022-jp (JIS, email)
(io/read-file "mail.txt" :encoding :iso-2022-jp)
```

### Chinese

```qi
;; :gbk (GBK, mainland China/Singapore, simplified Chinese Windows/Excel)
(io/write-file data "china_excel.csv" :encoding :gbk)

;; :gb18030 (GB18030, Chinese national standard, GBK superset)
(io/write-file data "china_official.txt" :encoding :gb18030)

;; :big5 (Big5, Taiwan/Hong Kong, traditional Chinese Windows/Excel)
(io/write-file data "taiwan_excel.csv" :encoding :big5)
```

### Korean

```qi
;; :euc-kr (EUC-KR, Korean Windows/Excel)
(io/write-file data "korea_excel.csv" :encoding :euc-kr)
```

### European

```qi
;; :windows-1252 / :cp1252 / :latin1 (Western European, US Windows/Excel)
(io/write-file data "europe_excel.csv" :encoding :windows-1252)

;; :windows-1251 / :cp1251 (Russian/Cyrillic Windows/Excel)
(io/write-file data "russia_excel.csv" :encoding :windows-1251)
```

### Auto Detection

```qi
;; :auto (BOM detection → UTF-8 → try each regional encoding sequentially)
(io/read-file "unknown.txt" :encoding :auto)
```

---

## Write Options

### Behavior When File Exists

```qi
;; :if-exists option
(io/write-file data "out.txt" :if-exists :error)      ;; Error if exists
(io/write-file data "out.txt" :if-exists :skip)       ;; Skip if exists
(io/write-file data "out.txt" :if-exists :append)     ;; Append
(io/write-file data "out.txt" :if-exists :overwrite)  ;; Overwrite (default)
```

### Automatic Directory Creation

```qi
;; :create-dirs option
(io/write-file data "path/to/out.txt" :create-dirs true)

;; Combining multiple options
(io/write-file data "backup/data.csv"
               :encoding :sjis
               :if-exists :error
               :create-dirs true)
```

---

## Filesystem Operations

### Directory Listing

```qi
;; io/list-dir - Get directory listing
(io/list-dir ".")                                ;; Current directory
(io/list-dir "logs" :pattern "*.log")            ;; Log files only
(io/list-dir "src" :pattern "*.rs" :recursive true)  ;; Recursive search
```

### Directory Operations

```qi
;; io/create-dir - Create directory (auto-creates parent directories)
(io/create-dir "data/backup")

;; io/delete-dir - Delete directory
(io/delete-dir "temp")                           ;; Delete empty directory
(io/delete-dir "old_data" :recursive true)       ;; Delete with contents
```

### File Operations

```qi
;; io/copy-file - Copy file
(io/copy-file "data.txt" "data_backup.txt")

;; io/move-file - Move/rename file
(io/move-file "old.txt" "new.txt")

;; io/delete-file - Delete file
(io/delete-file "temp.txt")
```

### Metadata Retrieval

```qi
;; io/file-info - Get file information
(def info (io/file-info "data.txt"))
(get info "size")                                ;; File size
(get info "modified")                            ;; Modified time (UNIX timestamp)
(get info "is-dir")                              ;; Is directory
(get info "is-file")                             ;; Is file

;; Predicate functions
(io/is-file? "data.txt")                         ;; true
(io/is-dir? "data")                              ;; true
(io/file-exists? "config.json")                  ;; true/false
```

---

## Temporary Files and Directories

### Automatic Deletion (Recommended)

```qi
;; io/temp-file - Create temporary file (auto-deleted on program exit)
(let [tmp (io/temp-file)]
  (io/write-file "temporary data" tmp)
  (process-file tmp))
;; tmp is automatically deleted on program exit

;; io/temp-dir - Create temporary directory (auto-deleted)
(let [tmpdir (io/temp-dir)]
  (io/write-file "data1" (path/join tmpdir "file1.txt"))
  (io/write-file "data2" (path/join tmpdir "file2.txt"))
  (process-directory tmpdir))
;; tmpdir and its contents are automatically deleted on program exit
```

### Manual Deletion (Keep Version)

```qi
;; io/temp-file-keep - Create persistent temporary file
(let [tmp (io/temp-file-keep)]
  (io/write-file "persistent data" tmp)
  (println f"Created: {tmp}")
  tmp)
;; => "/tmp/.tmpXXXXXX" (not deleted, manual deletion required)

;; io/temp-dir-keep - Create persistent temporary directory
(let [tmpdir (io/temp-dir-keep)]
  (io/create-dir (path/join tmpdir "subdir"))
  tmpdir)
;; => "/tmp/.tmpXXXXXX" (not deleted)
```

---

## Practical Examples

### CSV File Processing

```qi
;; Read and process CSV
(io/read-file "data.csv")
  |> (fn [content] (split content "\n"))
  |> (map (fn [line] (split line ",")))
  |> (filter (fn [row] (> (len row) 2)))
```

### Combining with Pipelines

```qi
;; Parallel processing of log files
("logs"
 |> (io/list-dir :pattern "*.log")
 |> (map io/read-file)
 |> (map process-log)
 |> (reduce merge))
```

### Encoding Conversion

```qi
;; Shift_JIS → UTF-8 conversion
(io/read-file "legacy.csv" :encoding :sjis)
 |> csv/parse
 |> (map transform)
 |> csv/stringify
 |> (fn [s] (io/write-file s "modern_utf8.csv"))
```

### Excel Support for Different Countries

```qi
;; Japan: Excel CSV (Shift_JIS)
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "japan_excel.csv" :encoding :sjis)))

;; China (simplified): Excel CSV (GBK)
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "china_excel.csv" :encoding :gbk)))

;; Taiwan/Hong Kong (traditional): Excel CSV (Big5)
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "taiwan_excel.csv" :encoding :big5)))

;; Multilingual mix: UTF-16LE (Excel recommended, with BOM)
(data
 |> csv/stringify
 |> (fn [s] (io/write-file s "multilang_excel.csv" :encoding :utf-16le)))
```

### Data Processing with Temporary Files

```qi
;; Process large data with temporary file
(defn process-large-data [url]
  (let [tmp (io/temp-file)]
    ;; Download data and save to temporary file
    (http/get url :output tmp)
    ;; Process temporary file
    (let [result (process-file tmp)]
      ;; tmp is auto-deleted after function exit
      result)))

;; Using multiple temporary files
(defn merge-files [files output]
  (let [tmpdir (io/temp-dir)
        processed (files
                   |> (map (fn [f]
                         (let [tmp (path/join tmpdir (path/basename f))]
                           (io/copy-file f tmp)
                           (process-file tmp)
                           tmp))))]
    ;; Merge processed files
    (merge-all processed output)
    ;; tmpdir and contents are auto-deleted after function exit
    output))
```

### Safe Writing

```qi
;; Protect existing files
(io/write-file data "important.txt"
               :if-exists :error
               :create-dirs true)

;; Process file with unknown encoding
(io/read-file "unknown.txt" :encoding :auto)
 |> process
 |> (fn [s] (io/write-file s "output.txt" :encoding :utf-8-bom))
```

---

## Function List

### File I/O
- `io/read-file` - Read entire file
- `io/read-lines` - Read line by line
- `io/write-file` - Write to file (overwrite)
- `io/append-file` - Append to file

### Filesystem Operations
- `io/list-dir` - Get directory listing (glob pattern support)
- `io/create-dir` - Create directory
- `io/delete-file` - Delete file
- `io/delete-dir` - Delete directory
- `io/copy-file` - Copy file
- `io/move-file` - Move/rename file

### Metadata
- `io/file-info` - Get file information
- `io/file-exists?` - Check file existence
- `io/is-file?` - File predicate
- `io/is-dir?` - Directory predicate

### Temporary Files
- `io/temp-file` - Create temporary file (auto-deleted)
- `io/temp-file-keep` - Create temporary file (not deleted)
- `io/temp-dir` - Create temporary directory (auto-deleted)
- `io/temp-dir-keep` - Create temporary directory (not deleted)
