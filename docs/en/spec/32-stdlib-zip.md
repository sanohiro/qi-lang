# Standard Library - ZIP Compression/Decompression (zip/)

**ZIP/gzip Compression and Decompression Library**

Supports archive creation/extraction using ZIP format, and single-file compression using gzip.

> **Feature Gate**: This module is compiled with the `util-zip` feature.

---

## Basic Operations

### Creating ZIP Files

```qi
;; zip/create - Create a ZIP file
(zip/create "backup.zip" "data.txt" "config.json")
;; => "backup.zip"

;; Compress multiple files
(zip/create "archive.zip" "file1.txt" "file2.txt" "file3.txt")

;; Compress entire directory
(zip/create "logs.zip" "logs/")
;; => All files in logs directory are added recursively

;; Mix files and directories
(zip/create "backup.zip" "README.md" "src/" "config/")
```

### Extracting ZIP Files

```qi
;; zip/extract - Extract ZIP file (to current directory)
(zip/extract "backup.zip")
;; => "."

;; Specify destination directory
(zip/extract "archive.zip" "output/")
;; => "output/"

;; Use in pipeline
("backup.zip" |> zip/extract)
```

### Listing ZIP Contents

```qi
;; zip/list - Get ZIP contents
(zip/list "backup.zip")
;; => [
;;      {:name "data.txt" :size 1234 :compressed-size 567 :is-dir false}
;;      {:name "config.json" :size 456 :compressed-size 123 :is-dir false}
;;      {:name "logs/" :size 0 :compressed-size 0 :is-dir true}
;;    ]

;; Get only file names
(zip/list "backup.zip"
 |> (map (fn [entry] (get entry "name"))))
;; => ["data.txt" "config.json" "logs/"]

;; Calculate compression ratios
(zip/list "backup.zip"
 |> (map (fn [entry]
           (let [original (get entry "size")
                 compressed (get entry "compressed-size")
                 ratio (if (> original 0)
                         (* 100 (/ compressed original))
                         0)]
             {:name (get entry "name")
              :ratio ratio}))))
```

### Adding to Existing ZIP

```qi
;; zip/add - Add files to existing ZIP
(zip/add "backup.zip" "new-file.txt")
;; => "backup.zip"

;; Add multiple files
(zip/add "backup.zip" "file1.txt" "file2.txt" "dir/")

;; Use in pipeline
("backup.zip" |> (zip/add _ "update.txt"))
```

---

## gzip Compression

### Basic Operations

```qi
;; zip/gzip - Compress file with gzip
(zip/gzip "data.txt")
;; => "data.txt.gz" (automatically adds .gz extension)

;; Specify output filename
(zip/gzip "data.txt" "backup.gz")
;; => "backup.gz"

;; zip/gunzip - Decompress gzip file
(zip/gunzip "data.txt.gz")
;; => "data.txt" (automatically removes .gz extension)

;; Specify output filename
(zip/gunzip "backup.gz" "restored.txt")
;; => "restored.txt"
```

### Using in Pipelines

```qi
;; gzip compression pipeline
("data.txt" |> zip/gzip)
;; => "data.txt.gz"

;; gzip decompression pipeline
("data.txt.gz" |> zip/gunzip)
;; => "data.txt"

;; Chain compress, process, decompress
("large-data.txt"
 |> zip/gzip
 |> upload-to-server
 |> download-from-server
 |> zip/gunzip)
```

---

## Practical Examples

### Backup System

```qi
;; Daily backup
(defn create-backup [date]
  (let [backup-name f"backup-{date}.zip"]
    (zip/create backup-name "data/" "config/" "logs/")
    (println f"Backup created: {backup-name}")))

(create-backup "2024-11-10")
;; => "Backup created: backup-2024-11-10.zip"

;; Backup multiple directories individually
(def dirs ["data" "config" "logs"])

(dirs
 |> (map (fn [dir]
           (let [zip-file f"{dir}-backup.zip"]
             (zip/create zip-file f"{dir}/")
             {:dir dir :file zip-file})))
 |> (map println))
```

### Log File Rotation

```qi
;; Archive old logs
(defn archive-old-logs [days]
  (let [cutoff (- (time/now) (* days 86400))]
    (io/list-dir "logs" :pattern "*.log")
    |> (filter (fn [file]
                 (let [info (io/file-info file)
                       modified (get info "modified")]
                   (< modified cutoff))))
    |> (map (fn [file]
              ;; Compress with gzip
              (let [gz (zip/gzip file)]
                ;; Delete original
                (io/delete-file file)
                gz)))))

;; Compress logs older than 7 days
(archive-old-logs 7)
```

### Creating Distribution Archives

```qi
;; Create release ZIP
(defn create-release [version]
  (let [archive-name f"myapp-{version}.zip"
        files ["README.md"
               "LICENSE"
               "bin/"
               "lib/"
               "docs/"]]
    ;; Copy necessary files to temporary directory
    (let [tmpdir (io/temp-dir)
          dest-dir (path/join tmpdir f"myapp-{version}")]
      (io/create-dir dest-dir)

      ;; Copy files
      (files
       |> (map (fn [file]
                 (let [dest (path/join dest-dir (path/basename file))]
                   (if (io/is-dir? file)
                     (copy-dir-recursive file dest)
                     (io/copy-file file dest))))))

      ;; Create ZIP
      (zip/create archive-name dest-dir)
      archive-name)))

(create-release "1.0.0")
;; => "myapp-1.0.0.zip"
```

### Download and Extract Data

```qi
;; Download archive from web and extract
(defn download-and-extract [url dest-dir]
  (let [tmpfile (io/temp-file)]
    ;; Download
    (http/get url :output tmpfile)

    ;; Extract
    (zip/extract tmpfile dest-dir)

    ;; Temporary file is automatically deleted
    (println f"Extracted to: {dest-dir}")))

(download-and-extract "https://example.com/data.zip" "data/")
```

### Processing Large Numbers of Files

```qi
;; Compress all files in directory individually
(defn compress-all-files [dir]
  (io/list-dir dir :pattern "*.*")
  |> (filter io/is-file?)
  |> (map zip/gzip)
  |> (map println))

(compress-all-files "documents/")

;; Parallel compression (fast processing for many files)
(defn compress-all-parallel [dir]
  (io/list-dir dir :pattern "*.*")
  |> (filter io/is-file?)
  ||> zip/gzip
  |> (map println))

(compress-all-parallel "logs/")
```

### Backup Verification

```qi
;; Check ZIP integrity
(defn verify-backup [zip-file expected-files]
  (let [entries (zip/list zip-file)
        names (entries |> (map (fn [e] (get e "name"))))]
    (expected-files
     |> (map (fn [file]
               (let [found (names |> (filter (fn [n] (str/contains? n file))))]
                 {:file file
                  :found (not (empty? found))})))
     |> (filter (fn [result] (not (get result "found")))))))

(def expected ["data.txt" "config.json" "logs/"])
(let [missing (verify-backup "backup.zip" expected)]
  (if (empty? missing)
    (println "Backup verified: all files present")
    (do
      (println "Missing files:")
      (missing |> (map (fn [m] (get m "file"))) |> (map println)))))
```

---

## Error Handling

### File Not Found

```qi
;; Catch errors with try-catch
(try
  (zip/create "backup.zip" "nonexistent.txt")
  (catch e
    (println f"Error: {e}")))
;; => "Error: zip/create: path does not exist: 'nonexistent.txt'"

;; Pre-check
(defn safe-create-zip [zip-file files]
  (let [missing (files |> (filter (fn [f] (not (io/file-exists? f)))))]
    (if (empty? missing)
      (zip/create zip-file ..files)
      (do
        (println "Missing files:")
        (missing |> (map println))
        :error))))

(safe-create-zip "backup.zip" ["file1.txt" "file2.txt"])
```

### Protecting Destination

```qi
;; Skip if destination directory exists
(defn safe-extract [zip-file dest-dir]
  (if (io/file-exists? dest-dir)
    (do
      (println f"Destination already exists: {dest-dir}")
      :skip)
    (zip/extract zip-file dest-dir)))

;; Confirm overwrite
(defn extract-with-confirm [zip-file dest-dir]
  (if (io/file-exists? dest-dir)
    (do
      (print f"Overwrite {dest-dir}? (y/n): ")
      (let [answer (io/stdin-line)]
        (if (= answer "y")
          (do
            (io/delete-dir dest-dir :recursive true)
            (zip/extract zip-file dest-dir))
          (println "Cancelled"))))
    (zip/extract zip-file dest-dir)))
```

### Disk Space Check

```qi
;; Check space before extraction
(defn check-space-before-extract [zip-file dest-dir]
  (let [entries (zip/list zip-file)
        total-size (entries
                    |> (map (fn [e] (get e "size")))
                    |> (reduce + 0))]
    (println f"Archive size: {total-size} bytes")
    ;; TODO: Check available disk space
    (zip/extract zip-file dest-dir)))
```

---

## Pipeline Integration

### File Processing Pipeline

```qi
;; Process files and add to ZIP
(io/list-dir "data" :pattern "*.txt")
|> (map io/read-file)
|> (map process-text)
|> (map-indexed (fn [i content]
                  (let [tmpfile f"/tmp/processed-{i}.txt"]
                    (io/write-file content tmpfile)
                    tmpfile)))
|> (fn [files]
     (zip/create "processed.zip" ..files))
```

### Download-Compress Pipeline

```qi
;; Download from multiple URLs and compress
(def urls ["https://example.com/file1.txt"
           "https://example.com/file2.txt"])

(urls
 ||> (fn [url]
       (let [tmpfile (io/temp-file)]
         (http/get url :output tmpfile)
         tmpfile))
 |> (fn [files]
      (zip/create "downloads.zip" ..files)))
```

### Compression Ratio Comparison

```qi
;; Report compression ratios for each file
(defn compression-report [zip-file]
  (zip/list zip-file)
  |> (filter (fn [e] (not (get e "is-dir"))))
  |> (map (fn [entry]
            (let [name (get entry "name")
                  original (get entry "size")
                  compressed (get entry "compressed-size")
                  ratio (if (> original 0)
                          (- 100 (* 100 (/ compressed original)))
                          0)]
              {:name name
               :original original
               :compressed compressed
               :saved (- original compressed)
               :ratio ratio})))
  |> (map (fn [r]
            (println f"{(get r \"name\")}: {(get r \"ratio\")}% saved"))))

(compression-report "backup.zip")
;; => data.txt: 54% saved
;; => config.json: 32% saved
;; => image.png: 2% saved
```

---

## Performance Considerations

### Processing Large Numbers of Files

```qi
;; Speed up with parallel processing (though zip/create executes sequentially)
(defn create-zip-fast [zip-file files]
  ;; Parallel existence check
  (let [valid (files ||> io/file-exists? |> (filter identity))]
    (zip/create zip-file ..valid)))

;; Split large directory and compress
(defn split-archive [dir max-files]
  (let [all-files (io/list-dir dir :recursive true)
        chunks (all-files |> (partition max-files))]
    (chunks
     |> (map-indexed (fn [i files]
                       (let [zip-file f"archive-part{i}.zip"]
                         (zip/create zip-file ..files)
                         zip-file))))))

(split-archive "large-dir" 1000)
;; => ["archive-part0.zip" "archive-part1.zip" ...]
```

### Memory Efficiency

```qi
;; Compress large files individually with gzip (more memory efficient than ZIP)
(defn compress-large-files [pattern]
  (io/list-dir "." :pattern pattern)
  |> (filter (fn [f]
               (let [info (io/file-info f)]
                 (> (get info "size") 10000000)))) ;; Over 10MB
  |> (map zip/gzip)
  |> (map println))

(compress-large-files "*.log")
```

### Compression Level Adjustment

The current implementation uses the default level of `CompressionMethod::Deflated`.
Support for specifying compression levels may be added in the future.

```qi
;; Future feature (not implemented)
;; (zip/create "backup.zip" "data/" :compression :fast)
;; (zip/create "backup.zip" "data/" :compression :best)
```

---

## Function List

### ZIP Operations
- `zip/create` - Create ZIP file (multiple files/directories supported)
- `zip/extract` - Extract ZIP file (destination specifiable)
- `zip/list` - Get ZIP contents (including size/compression info)
- `zip/add` - Add files to existing ZIP

### gzip Operations
- `zip/gzip` - Compress file with gzip (output filename specifiable)
- `zip/gunzip` - Decompress gzip file (output filename specifiable)

---

## Choosing Between ZIP and gzip

### Use ZIP Format When
- Need to bundle multiple files
- Need to preserve directory structure
- Need to extract individual files
- Want to open with double-click on Windows/Mac
- Creating distribution archives

### Use gzip Format When
- Compressing single files
- Standard compression on Unix/Linux
- Streaming compression needed
- Combining with tar (tar.gz)
- Log file rotation

---

## Common Patterns

### Backup and Restore

```qi
;; Create backup
(defn backup [name]
  (let [timestamp (time/format (time/now) "%Y%m%d-%H%M%S")
        zip-file f"backup-{name}-{timestamp}.zip"]
    (zip/create zip-file "data/" "config/")
    (println f"Backup created: {zip-file}")
    zip-file))

;; Restore
(defn restore [zip-file]
  (let [tmpdir (io/temp-dir)]
    (zip/extract zip-file tmpdir)
    ;; Copy to production after verification
    (println "Restored to temporary directory")
    tmpdir))
```

### Data Distribution

```qi
;; Create release package
(defn package-release [version files]
  (let [release-name f"release-v{version}.zip"]
    (zip/create release-name ..files)
    (println f"Release package: {release-name}")
    release-name))

;; Create multiple distribution formats
(defn create-distributions [version]
  {:zip (zip/create f"dist-{version}.zip" "dist/")
   :tar-gz (do
             (zip/create f"dist-{version}.tar" "dist/")
             (zip/gzip f"dist-{version}.tar"))})
```

### Log Management

```qi
;; Daily log archiving
(defn archive-daily-logs []
  (let [yesterday (time/format (- (time/now) 86400) "%Y-%m-%d")
        log-files (io/list-dir "logs" :pattern f"*{yesterday}*.log")]
    (when (not (empty? log-files))
      (let [archive f"logs-{yesterday}.zip"]
        (zip/create archive ..log-files)
        ;; Delete originals
        (log-files |> (map io/delete-file))
        archive))))
```
