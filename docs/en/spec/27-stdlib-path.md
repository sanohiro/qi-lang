# Standard Library - Path Operations (path/)

**Cross-platform path manipulation**

All functions belong to the `path/` module.

---

## Path Joining

### path/join

Joins multiple path elements to create a platform-appropriate path.

```qi
;; Basic usage
(path/join "dir" "subdir" "file.txt")
;; Unix: => "dir/subdir/file.txt"
;; Windows: => "dir\\subdir\\file.txt"

;; Joining with absolute path
(path/join "/usr" "local" "bin")
;; => "/usr/local/bin"

;; Variable arguments
(path/join "a" "b" "c" "d" "e")
;; => "a/b/c/d/e"

;; With pipeline
(["logs" "2024" "01" "app.log"]
 |> (apply path/join))
;; => "logs/2024/01/app.log"
```

**Arguments**:
- `parts...` - Path elements to join (variadic, minimum 1)

**Returns**: Joined path string

**Cross-platform**:
- Unix/Linux/macOS: `/` separator
- Windows: `\` separator

---

## Path Parsing

### path/basename

Extracts the file name part (last element) from a path.

```qi
;; Get file name
(path/basename "/path/to/file.txt")
;; => "file.txt"

;; Get directory name
(path/basename "/path/to/dir")
;; => "dir"

;; Windows path
(path/basename "C:\\Users\\Alice\\Document.docx")
;; => "Document.docx"

;; Root directory
(path/basename "/")
;; => ""

;; Use in pipeline
(io/list-dir "downloads")
 |> (map path/basename)
;; => ["file1.txt" "file2.pdf" "image.png"]
```

**Arguments**:
- `path` - Path string

**Returns**: File name string (empty string if not available)

---

### path/dirname

Extracts the directory part (parent directory) from a path.

```qi
;; Get directory part
(path/dirname "/path/to/file.txt")
;; => "/path/to"

;; Multiple levels
(path/dirname "/a/b/c/d.txt")
;; => "/a/b/c"

;; Windows path
(path/dirname "C:\\Users\\Alice\\file.txt")
;; => "C:\\Users\\Alice"

;; Root level
(path/dirname "/file.txt")
;; => "/"

;; Relative path
(path/dirname "dir/file.txt")
;; => "dir"

;; Get parent directory in pipeline
(file-path
 |> path/dirname
 |> io/create-dir)
```

**Arguments**:
- `path` - Path string

**Returns**: Directory part string (empty string if not available)

---

### path/extension

Extracts the file extension.

```qi
;; Get extension
(path/extension "file.txt")
;; => "txt"

(path/extension "archive.tar.gz")
;; => "gz"

;; No extension
(path/extension "README")
;; => ""

;; Dot file
(path/extension ".gitignore")
;; => ""

;; Path included
(path/extension "/path/to/document.pdf")
;; => "pdf"

;; Filter by extension
(io/list-dir "src")
 |> (filter (fn [f] (= (path/extension f) "rs")))
;; => ["/path/to/main.rs" "/path/to/lib.rs"]
```

**Arguments**:
- `path` - Path string

**Returns**: Extension without dot (empty string if no extension)

**Note**: Returns only the part after the last dot (`file.tar.gz` → `"gz"`)

---

### path/stem

Extracts the file name without extension.

```qi
;; File name without extension
(path/stem "file.txt")
;; => "file"

(path/stem "document.pdf")
;; => "document"

;; Multiple dots
(path/stem "archive.tar.gz")
;; => "archive.tar"

;; No extension
(path/stem "README")
;; => "README"

;; Path included
(path/stem "/path/to/report.docx")
;; => "report"

;; Transform file name in pipeline
("data.csv"
 |> path/stem
 |> (fn [s] (str s "_processed.json")))
;; => "data_processed.json"
```

**Arguments**:
- `path` - Path string

**Returns**: File name without extension (empty string if not available)

---

## Path Conversion

### path/absolute

Converts a relative path to an absolute path.

```qi
;; Convert relative to absolute path
(path/absolute "data/file.txt")
;; => "/Users/alice/project/data/file.txt"

;; Already absolute path
(path/absolute "/usr/local/bin")
;; => "/usr/local/bin"

;; Current directory
(path/absolute ".")
;; => "/Users/alice/project"

;; Parent directory reference
(path/absolute "../other")
;; => "/Users/alice/other"

;; Use in pipeline
(relative-paths
 |> (map path/absolute))
```

**Arguments**:
- `path` - Path string

**Returns**: Absolute path string

**Note**: Resolves relative to current working directory

---

### path/normalize

Normalizes a path by resolving `.` and `..` components.

```qi
;; Resolve . and ..
(path/normalize "a/./b/../c")
;; => "a/c"

(path/normalize "/path/to/../other/./file.txt")
;; => "/path/other/file.txt"

;; Merge consecutive slashes
(path/normalize "a//b///c")
;; => "a/b/c"

;; Complex path
(path/normalize "/a/b/c/../../d/./e/../f")
;; => "/a/d/f"

;; Use in pipeline
(user-input-path
 |> path/normalize
 |> path/absolute)
```

**Arguments**:
- `path` - Path string

**Returns**: Normalized path string

**Note**:
- `.` components are ignored
- `..` components move to parent directory (stack-based)
- Does not move above root

---

## Path Validation

### path/is-absolute?

Checks if a path is absolute.

```qi
;; Check absolute path
(path/is-absolute? "/usr/bin")
;; => true

(path/is-absolute? "relative/path")
;; => false

;; Windows path
(path/is-absolute? "C:\\Program Files")
;; => true

(path/is-absolute? "Documents\\file.txt")
;; => false

;; Use in pipeline
(paths
 |> (filter path/is-absolute?))
```

**Arguments**:
- `path` - Path string

**Returns**: `true` or `false`

**Platform-specific detection**:
- Unix/Linux/macOS: starts with `/`
- Windows: starts with `C:\` or `\\server\`

---

### path/is-relative?

Checks if a path is relative.

```qi
;; Check relative path
(path/is-relative? "data/file.txt")
;; => true

(path/is-relative? "/usr/local")
;; => false

;; Current directory
(path/is-relative? ".")
;; => true

(path/is-relative? "..")
;; => true

;; Use in pipeline
(paths
 |> (filter path/is-relative?)
 |> (map path/absolute))
```

**Arguments**:
- `path` - Path string

**Returns**: `true` or `false`

**Note**: Opposite of `path/is-absolute?`

---

## Practical Examples

### File Path Construction

```qi
;; Project structure construction
(def project-root "/Users/alice/project")
(def src-dir (path/join project-root "src"))
(def test-dir (path/join project-root "tests"))
(def main-file (path/join src-dir "main.qi"))

;; User data directory
(defn user-data-path [username filename]
  (path/join "/data" "users" username filename))

(user-data-path "alice" "profile.json")
;; => "/data/users/alice/profile.json"
```

### File Name Transformation

```qi
;; Generate backup file name
(defn backup-filename [original-path]
  (let [dir (path/dirname original-path)
        name (path/stem original-path)
        ext (path/extension original-path)
        timestamp (time/now |> time/format "yyyyMMdd_HHmmss")]
    (path/join dir (str name "_backup_" timestamp "." ext))))

(backup-filename "/data/document.txt")
;; => "/data/document_backup_20240115_143025.txt"
```

### Path Analysis Pipeline

```qi
;; Get file information
(defn file-info [filepath]
  {:path filepath
   :absolute (path/absolute filepath)
   :dir (path/dirname filepath)
   :name (path/basename filepath)
   :stem (path/stem filepath)
   :ext (path/extension filepath)
   :is-absolute (path/is-absolute? filepath)})

(file-info "src/main.qi")
;; => {:path "src/main.qi"
;;     :absolute "/Users/alice/project/src/main.qi"
;;     :dir "src"
;;     :name "main.qi"
;;     :stem "main"
;;     :ext "qi"
;;     :is-absolute false}
```

### Directory Traversal

```qi
;; Get all Qi files in directory
(defn find-qi-files [dir]
  (io/list-dir dir :recursive true)
   |> (filter (fn [f] (= (path/extension f) "qi")))
   |> (map path/absolute))

(find-qi-files "src")
;; => ["/project/src/main.qi"
;;     "/project/src/lib.qi"
;;     "/project/src/utils/helpers.qi"]
```

### Safe Path Handling

```qi
;; Validate user input path
(defn safe-path [base-dir user-input]
  (let [full-path (-> user-input
                      path/normalize
                      (fn [p] (path/join base-dir p))
                      path/absolute)]
    ;; Prevent access outside base directory
    (if (str/starts-with? full-path base-dir)
      full-path
      (throw "Invalid path: outside base directory"))))

(safe-path "/data/users/alice" "../bob/secret.txt")
;; Error: Invalid path: outside base directory

(safe-path "/data/users/alice" "documents/file.txt")
;; => "/data/users/alice/documents/file.txt"
```

### Extension-based Processing

```qi
;; Process file based on extension
(defn process-file [filepath]
  (match (path/extension filepath)
    "txt" (io/read-file filepath)
    "json" (-> filepath io/read-file json/parse)
    "csv" (csv/read-file filepath)
    "qi" (load-file filepath)
    _ (throw (str "Unsupported file type: " filepath))))

;; Batch process files in directory
(io/list-dir "data")
 |> (map process-file)
 |> (filter some?)
```

### Cross-platform Support

```qi
;; OS-specific config file path
(defn config-path []
  (let [home (env/get "HOME" (env/get "USERPROFILE"))]
    (if (str/contains? (env/get "OS" "") "Windows")
      (path/join home "AppData" "Roaming" "MyApp" "config.json")
      (path/join home ".config" "myapp" "config.json"))))

;; Log file path generation
(defn log-path [app-name]
  (let [log-dir (if (str/contains? (env/get "OS" "") "Windows")
                  (path/join (env/get "PROGRAMDATA") app-name "logs")
                  (path/join "/var" "log" app-name))]
    (path/join log-dir (str app-name ".log"))))
```

### Batch Relative Path Conversion

```qi
;; Convert relative paths to absolute in project
(def project-files [
  "src/main.qi"
  "tests/test_main.qi"
  "docs/README.md"])

(def absolute-files
  (project-files
   |> (map path/absolute)
   |> (map path/normalize)))
```

---

## Cross-platform Support

### Path Separators

- **Unix/Linux/macOS**: `/`
- **Windows**: `\` (display requires `\\` escape)

The `path/` module automatically uses platform-specific separators.

### Absolute Path Formats

**Unix/Linux/macOS**:
```qi
(path/is-absolute? "/usr/local/bin")  ;; => true
(path/is-absolute? "~/Documents")     ;; => false (~ not expanded)
```

**Windows**:
```qi
(path/is-absolute? "C:\\Program Files")  ;; => true
(path/is-absolute? "\\\\server\\share")  ;; => true (UNC path)
(path/is-absolute? "relative\\path")     ;; => false
```

### Path Normalization Differences

**Unix/Linux/macOS**:
```qi
(path/normalize "/usr/./local/../bin")
;; => "/usr/bin"
```

**Windows**:
```qi
(path/normalize "C:\\Users\\.\\Alice\\..\\Bob")
;; => "C:\\Users\\Bob"
```

---

## Function List

| Function | Description | Purpose |
|----------|-------------|---------|
| `path/join` | Join path elements | Path construction |
| `path/basename` | Get file name | File name parsing |
| `path/dirname` | Get directory part | Parent directory |
| `path/extension` | Get extension | File type detection |
| `path/stem` | Get file name without extension | File name transformation |
| `path/absolute` | Convert to absolute path | Path resolution |
| `path/normalize` | Normalize path | Resolve `.` and `..` |
| `path/is-absolute?` | Check if absolute | Path validation |
| `path/is-relative?` | Check if relative | Path validation |

---

## References

- **Related Modules**:
  - `io/` - File I/O
  - `env/` - Environment variables
  - `str/` - String operations
- **Platforms**:
  - Unix/Linux/macOS and Windows support
  - Uses Rust's `std::path` module
