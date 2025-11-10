# Standard Library - Temporary Files and Directories (io/temp)

**Safe Temporary Resource Management**

Provides functions for creating and managing temporary files and directories. Automatic cleanup prevents resource leaks.

---

## Overview

Temporary files and directories are used for:

- **Test data storage** - Unit tests, integration tests
- **Intermediate file generation** - Data transformation pipelines, build processes
- **Cache files** - External API results, computation results
- **Downloaded files** - Temporary storage for HTTP-fetched files
- **Working directories** - Work areas for multi-file processing

All temporary resources are created in the system's temporary directory (`/tmp` etc.).

---

## Temporary Files

### Auto-delete (Recommended)

```qi
;; io/temp-file - Create temporary file (auto-deleted on program exit)
(let [tmp (io/temp-file)]
  (io/write-file "temporary data" tmp)
  (println f"Created temp file: {tmp}")
  (process-file tmp))
;; File is automatically deleted when the program exits

;; Example: Download and process data
(defn download-and-process [url]
  (let [tmp (io/temp-file)]
    ;; Download data from URL
    (http/get url :output tmp)
    ;; Process temporary file
    (let [result (io/read-file tmp |> parse-data)]
      ;; tmp is auto-deleted after function returns
      result)))

;; Example: Multiple temporary files
(defn process-with-temp []
  (let [tmp1 (io/temp-file)
        tmp2 (io/temp-file)]
    (io/write-file "data 1" tmp1)
    (io/write-file "data 2" tmp2)
    (merge-files tmp1 tmp2)))
;; Both files are auto-deleted
```

### Manual deletion (Persist)

```qi
;; io/temp-file-keep - Create temporary file (not deleted)
(let [tmp (io/temp-file-keep)]
  (io/write-file "persistent data" tmp)
  (println f"Created: {tmp}")
  tmp)
;; => "/tmp/qi-12345.tmp" (not deleted, manual deletion required)

;; Example: Provide result file to user
(defn export-to-temp [data]
  (let [tmp (io/temp-file-keep)]
    (io/write-file (json/stringify data) tmp :encoding :utf-8-bom)
    (println f"Export completed: {tmp}")
    (println "Please move or delete this file when done.")
    tmp))

;; Manual deletion later
(io/delete-file tmp)
```

---

## Temporary Directories

### Auto-delete (Recommended)

```qi
;; io/temp-dir - Create temporary directory (auto-deleted on program exit)
(let [tmpdir (io/temp-dir)]
  (println f"Temp directory: {tmpdir}")
  (io/write-file "data1" (path/join tmpdir "file1.txt"))
  (io/write-file "data2" (path/join tmpdir "file2.txt"))
  (process-directory tmpdir))
;; Directory and contents are automatically deleted when the program exits

;; Example: Process multiple files temporarily
(defn process-archive [archive-path]
  (let [tmpdir (io/temp-dir)]
    ;; Extract archive to temporary directory
    (archive/extract archive-path tmpdir)
    ;; Process extracted files
    (io/list-dir tmpdir :recursive true)
      |> (map (fn [f] (path/join tmpdir f)))
      |> (map process-file)
      |> (reduce merge)
    ;; tmpdir and contents are auto-deleted after function returns
    ))

;; Example: Build directory
(defn build-project []
  (let [build-dir (io/temp-dir)]
    ;; Copy source files to temporary directory
    (io/copy-file "src/main.qi" (path/join build-dir "main.qi"))
    (io/copy-file "src/lib.qi" (path/join build-dir "lib.qi"))
    ;; Build process
    (compile-files build-dir)
    ;; Get output
    (let [output (io/read-file (path/join build-dir "output.bin"))]
      ;; build-dir is auto-deleted
      output)))
```

### Manual deletion (Persist)

```qi
;; io/temp-dir-keep - Create temporary directory (not deleted)
(let [tmpdir (io/temp-dir-keep)]
  (io/write-file "file1" (path/join tmpdir "data.txt"))
  (io/create-dir (path/join tmpdir "subdir"))
  tmpdir)
;; => "/tmp/qi-dir-12345" (not deleted)

;; Example: Debug output directory
(defn export-debug-info []
  (let [debug-dir (io/temp-dir-keep)]
    (io/write-file (json/stringify state) (path/join debug-dir "state.json"))
    (io/write-file logs (path/join debug-dir "logs.txt"))
    (println f"Debug info exported to: {debug-dir}")
    debug-dir))

;; Manual deletion later
(io/delete-dir tmpdir :recursive true)
```

---

## Auto-deletion Mechanism

### Deletion Timing

**Auto-deleted** (`io/temp-file`, `io/temp-dir`):
- When the program exits normally
- When the program exits with an error
- When the REPL session ends

**Not deleted** (`io/temp-file-keep`, `io/temp-dir-keep`):
- Persists after program exit
- Manual deletion required (`io/delete-file`, `io/delete-dir`)

### Implementation Details

```qi
;; Auto-delete version holds internal handle
;; OS automatically deletes on program exit
(io/temp-file)    ;; => Handle retained → auto-deleted

;; Keep version persists
(io/temp-file-keep)  ;; => Handle released → not deleted
```

### Notes

1. **Long-running programs**: In REPL or daemon programs, temporary files may accumulate
2. **Disk space**: Be careful when creating many large temporary files
3. **Keep version deletion**: Always manually delete files created with `io/temp-file-keep`
4. **Concurrency**: Temporary file names are automatically unique, safe for concurrent processing

---

## Practical Examples

### Test Data Management

```qi
;; Temporary file for unit tests
(defn test-file-processing []
  (let [tmp (io/temp-file)]
    ;; Write test data
    (io/write-file "line1\nline2\nline3" tmp)
    ;; Test function
    (let [result (process-file tmp)]
      (assert (= result ["line1" "line2" "line3"]))
      ;; tmp is auto-deleted
      true)))

;; Temporary directory for integration tests
(defn test-directory-processing []
  (let [tmpdir (io/temp-dir)]
    ;; Create test file structure
    (io/write-file "config" (path/join tmpdir "config.txt"))
    (io/create-dir (path/join tmpdir "data"))
    (io/write-file "data1" (path/join tmpdir "data" "file1.txt"))
    ;; Test directory processing
    (let [result (process-directory tmpdir)]
      (assert (= (len result) 2))
      ;; tmpdir and contents are auto-deleted
      true)))
```

### Data Transformation Pipeline

```qi
;; CSV → JSON conversion (using temporary file)
(defn csv-to-json [csv-path json-path]
  (let [tmp (io/temp-file)]
    ;; Read CSV
    (io/read-file csv-path)
      |> csv/parse
      ;; Transform data
      |> (map transform-row)
      ;; Output JSON to temporary file
      |> json/stringify
      |> (fn [s] (io/write-file s tmp))
    ;; Validate
    (validate-json tmp)
    ;; Final output
    (io/copy-file tmp json-path)
    ;; tmp is auto-deleted
    json-path))

;; Merge CSV files (using temporary directory)
(defn merge-csv-files [input-files output-path]
  (let [tmpdir (io/temp-dir)]
    ;; Normalize each file and save to temporary directory
    (input-files
      |> (map-indexed (fn [i f]
           (let [normalized (io/read-file f |> normalize-csv)]
             (io/write-file normalized
                           (path/join tmpdir f"file{i}.csv"))))))
    ;; Merge files in temporary directory
    (io/list-dir tmpdir)
      |> (map (fn [f] (path/join tmpdir f)))
      |> (map io/read-file)
      |> (join "\n")
      |> (fn [s] (io/write-file s output-path))
    ;; tmpdir is auto-deleted
    output-path))
```

### HTTP Download and Caching

```qi
;; Download large file
(defn download-large-file [url output-path]
  (let [tmp (io/temp-file)]
    ;; Download to temporary file
    (http/get url :output tmp :timeout 300)
    ;; Verify checksum
    (if (verify-checksum tmp)
      (do
        (io/move-file tmp output-path)
        (println f"Download completed: {output-path}")
        output-path)
      (do
        ;; On error, tmp is auto-deleted
        (error "Checksum verification failed")))))

;; API client with cache
(defn fetch-with-cache [url cache-duration]
  (let [cache-file (io/temp-file-keep)
        cache-age (if (io/file-exists? cache-file)
                    (- (time/now) (get (io/file-info cache-file) "modified"))
                    cache-duration)]
    (if (< cache-age cache-duration)
      ;; Cache is valid
      (io/read-file cache-file)
      ;; Cache is old, refetch
      (let [data (http/get url |> get "body")]
        (io/write-file data cache-file)
        data))))
```

### Batch Processing

```qi
;; Parallel file processing
(defn parallel-process-files [input-files]
  (let [tmpdir (io/temp-dir)]
    ;; Process each file in parallel
    (input-files
      |> (pmap (fn [f]
           (let [tmp (path/join tmpdir (path/basename f))]
             ;; Save processing result to temporary file
             (process-file f |> (fn [result] (io/write-file result tmp)))
             tmp)))
      ;; Collect results
      |> (map io/read-file)
      ;; tmpdir is auto-deleted
      )))

;; Multi-stage processing with intermediate files
(defn multi-stage-process [input]
  (let [stage1-file (io/temp-file)
        stage2-file (io/temp-file)
        stage3-file (io/temp-file)]
    ;; Stage 1: Data cleaning
    (input
      |> clean-data
      |> (fn [s] (io/write-file s stage1-file)))
    ;; Stage 2: Data transformation
    (io/read-file stage1-file
      |> transform-data
      |> (fn [s] (io/write-file s stage2-file)))
    ;; Stage 3: Data aggregation
    (io/read-file stage2-file
      |> aggregate-data
      |> (fn [s] (io/write-file s stage3-file)))
    ;; Return final result
    (let [result (io/read-file stage3-file)]
      ;; All temporary files are auto-deleted
      result)))
```

### Secure Temporary File Processing

```qi
;; Temporary storage of sensitive data (auto-delete guaranteed)
(defn process-sensitive-data [encrypted-data]
  (let [tmp (io/temp-file)]
    ;; Save decrypted data to temporary file
    (decrypt encrypted-data |> (fn [s] (io/write-file s tmp)))
    ;; Process
    (let [result (process-data tmp)]
      ;; tmp is auto-deleted (sensitive data does not remain)
      result)))

;; Extract password-protected archive
(defn extract-protected-archive [archive-path password]
  (let [tmpdir (io/temp-dir)]
    ;; Extract
    (archive/extract archive-path tmpdir :password password)
    ;; Process
    (let [result (process-files tmpdir)]
      ;; tmpdir and contents are auto-deleted
      result)))
```

---

## Pipeline Integration

```qi
;; Pipeline using temporary file
(defn download-convert-upload [source-url target-url]
  (let [tmp (io/temp-file)]
    (source-url
      |> (http/get :output tmp)
      |> (fn [_] (io/read-file tmp))
      |> convert-format
      |> (http/post target-url :body _))
    ;; tmp is auto-deleted
    ))

;; Complex pipeline using temporary directory
(defn batch-convert [input-dir output-dir]
  (let [tmpdir (io/temp-dir)]
    (input-dir
      |> (io/list-dir :pattern "*.txt")
      |> (map (fn [f]
           ;; Process each file in temporary directory
           (let [tmp (path/join tmpdir (path/basename f))]
             (io/read-file (path/join input-dir f)
               |> convert-content
               |> (fn [s] (io/write-file s tmp)))
             tmp)))
      ;; Copy processed files to output directory
      |> (map (fn [tmp]
           (io/copy-file tmp (path/join output-dir (path/basename tmp))))))
    ;; tmpdir is auto-deleted
    ))
```

---

## Security Considerations

### Safe Temporary File Creation

Qi's temporary file functionality implements the following security measures:

1. **Unique file names**: Uses unpredictable random names
2. **Proper permissions**: Not readable by other users
3. **Auto-deletion**: Sensitive data does not remain
4. **TOCTOU attack prevention**: Gets handle at the same time as file creation

```qi
;; ✅ Safe (recommended)
(let [tmp (io/temp-file)]
  (io/write-file sensitive-data tmp)
  (process-file tmp))
;; Automatically deleted on program exit

;; ❌ Not recommended (manual deletion required, risk of forgetting)
(let [tmp "/tmp/myapp-data.txt"]
  (io/write-file sensitive-data tmp)
  (process-file tmp)
  (io/delete-file tmp))  ;; May not be deleted on error
```

### Best Practices

1. **Prefer auto-delete version**: Use `io/temp-file` and `io/temp-dir`
2. **Process sensitive data in temporary files**: Auto-deleted after processing
3. **Minimize keep version use**: Only when persistence is truly needed
4. **Consider disk space**: Be careful when handling large files
5. **Error handling**: Auto-deletion works even with `try-catch`

```qi
;; Auto-deleted even on error
(defn safe-process [data]
  (let [tmp (io/temp-file)]
    (try
      (do
        (io/write-file data tmp)
        (risky-operation tmp))
      (catch e
        (println f"Error: {e}")
        ;; tmp is auto-deleted
        nil))))
```

---

## Function List

### Temporary Files
- `io/temp-file` - Create temporary file (auto-deleted)
- `io/temp-file-keep` - Create temporary file (not deleted)

### Temporary Directories
- `io/temp-dir` - Create temporary directory (auto-deleted)
- `io/temp-dir-keep` - Create temporary directory (not deleted)

---

## Summary

- **Prefer auto-delete version**: Prevents resource leaks
- **Secure**: Ideal for temporary processing of sensitive data
- **Simple**: No explicit deletion code required
- **Pipeline integration**: Combines with Qi's pipeline operators for powerful data processing
