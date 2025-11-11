# Standard Library - Command Execution (cmd/)

**Execute external commands and shell scripts**

All functions belong to the `cmd/` module.

---

## Overview

The cmd module provides functions for executing external commands and shell scripts from Qi. Integrated with pipeline operators (`|>`), you can treat command execution as data flow.

**Key Features**:
- Command execution with exit code retrieval
- Standard input/output control
- Pipeline integration (Qi → Command → Qi)
- Stream processing (line-based & byte-based)
- Bidirectional interactive processes

**Cross-platform Support**:
- **Unix/Linux/macOS**: Execute shell commands via `/bin/sh`
- **Windows**: Execute commands via `cmd.exe`

**Note**: This module is compiled with the `cmd-exec` feature.

---

## Basic Execution

### cmd/exec - Execute Command (Returns Exit Code)

**Arguments**: Command (string or vector)
**Returns**: Exit code (integer)
**Error**: When command is not found

```qi
;; Via shell (string)
(cmd/exec "ls -la")                    ;; => 0
(cmd/exec "false")                     ;; => 1

;; Direct execution (vector)
(cmd/exec ["ls" "-la"])                ;; => 0
(cmd/exec ["git" "status"])            ;; => 0

;; Error checking with exit code
(let [code (cmd/exec "test -f data.txt")]
  (if (= code 0)
    (println "File exists")
    (println "File does not exist")))
```

**Shell vs Direct Execution**:
- **String**: Via shell (pipes, redirects, environment variable expansion available)
- **Vector**: Direct execution (prevents shell injection, faster)

### cmd/exec! - Execute Command (Detailed Version)

**Arguments**: Command (string or vector)
**Returns**: `{:stdout "..." :stderr "..." :exit 0}` map

```qi
;; Get stdout, stderr, and exit code via shell
(cmd/exec! "cat *.txt | grep pattern | wc -l")
;; => {:stdout "      42\n" :stderr "" :exit 0}

;; Same with direct execution
(cmd/exec! ["ls" "-la"])
;; => {:stdout "total 48\ndrwxr-xr-x ...\n" :stderr "" :exit 0}

;; Detailed error information
(cmd/exec! "ls non-existent-file")
;; => {:stdout ""
;;     :stderr "ls: non-existent-file: No such file or directory\n"
;;     :exit 1}

;; Use results destructured
(let [result (cmd/exec! "git status --porcelain")
      stdout (get result "stdout")
      exit (get result "exit")]
  (if (= exit 0)
    (if (str/blank? stdout)
      (println "Working directory is clean")
      (println f"Changes detected:\n{stdout}"))
    (println "Not a git repository")))
```

**When to Use**:
- `cmd/exec` - When you only need the exit code (simple)
- `cmd/exec!` - When you need stdout or stderr content

---

## Pipeline Integration

### cmd/pipe - Pass Input to Command

**Arguments**: Command, [input data (string or list)]
**Returns**: Standard output (string)
**Error**: When exit code is non-zero

```qi
;; Execute command alone
(cmd/pipe "ls -la")
;; => "total 48\ndrwxr-xr-x  5 user  staff  160 Jan  1 12:00 .\n..."

;; Pass input via pipeline
("hello\nworld\n" |> (cmd/pipe "sort"))
;; => "hello\nworld\n"

;; Pass list (each element becomes a line)
(["line1" "line2" "line3"] |> (cmd/pipe "wc -l"))
;; => "       3\n"

;; Practical example: Process JSON with jq
(http/get "https://api.example.com/data")
  |> (get _ "body")
  |> (cmd/pipe "jq '.users[] | .name'")
  |> str/lines
  |> (map str/trim)
;; => ["Alice" "Bob" "Charlie"]
```

### cmd/pipe! - Execute Command (Detailed Version)

**Arguments**: Command, [input data]
**Returns**: `[stdout stderr exitcode]` vector

```qi
;; Get all: stdout, stderr, exit code
(cmd/pipe! "cat test.txt")
;; => ["file content\n" "" 0]

;; Use with destructuring
(let [[out err code] (cmd/pipe! "ls -la")]
  (if (= code 0)
    (println out)
    (println f"Error: {err}")))
```

### cmd/lines - Split Text into Lines (Helper)

**Arguments**: Text
**Returns**: List of lines

```qi
;; Split command output into lines
("a\nb\nc" |> cmd/lines)
;; => ["a" "b" "c"]

;; Practical pipeline example
(cmd/pipe "ls -1")
  |> cmd/lines
  |> (filter (fn [f] (str/ends-with? f ".qi")))
  |> (map (fn [f] (str/replace f ".qi" "")))
;; => ["main" "lib" "test"]
```

---

## Stream Processing

### cmd/stream-lines - Line-based Stream

**Arguments**: Command (string or vector)
**Returns**: Stream (each element is a line string)

```qi
;; Process log file as stream
(def log-stream (cmd/stream-lines "tail -f /var/log/app.log"))

;; Get first 10 lines
(log-stream |> (stream/take 10) |> realize)

;; Filter while processing
(cmd/stream-lines "cat large.log")
  |> (stream/filter (fn [line] (str/contains? line "ERROR")))
  |> (stream/take 100)
  |> realize
  |> (map println)
```

### cmd/stream-bytes - Byte-based Stream

**Arguments**: Command, [chunk size (default 4096)]
**Returns**: Stream (each element is Base64-encoded bytes)

```qi
;; Process large file in chunks
(cmd/stream-bytes "cat large-file.bin")
  |> (stream/take 10)  ;; First 10 chunks (40KB)
  |> realize

;; Custom chunk size (8KB)
(cmd/stream-bytes "curl -L https://example.com/video.mp4" 8192)
  |> (stream/each process-chunk)
```

---

## Interactive Processes

### cmd/interactive - Launch Bidirectional Process

**Arguments**: Command (string or vector)
**Returns**: Process handle (Map format)

```qi
;; Launch Python interpreter
(def py (cmd/interactive "python3 -i"))

;; Send command
(cmd/write py "print(1+1)\n")

;; Read result
(cmd/read-line py)  ;; => "2"

;; Terminate process
(cmd/write py "exit()\n")
(cmd/wait py)       ;; => {:exit 0 :stderr ""}
```

### cmd/write - Write to Process

**Arguments**: Process handle, data (string)
**Returns**: nil

```qi
;; Send multiple lines to REPL
(cmd/write py "def greet(name):\n")
(cmd/write py "    return f'Hello, {name}!'\n")
(cmd/write py "\n")
(cmd/write py "print(greet('Qi'))\n")
```

### cmd/read-line - Read Line from Process

**Arguments**: Process handle
**Returns**: Read line (string), or `nil` on EOF

```qi
;; Read result
(cmd/read-line py)  ;; => "Hello, Qi!"

;; Read all output
(defn read-all [proc]
  (loop [lines []]
    (let [line (cmd/read-line proc)]
      (if (some? line)
        (recur (conj lines line))
        lines))))

(read-all py)  ;; => ["line1" "line2" "line3"]
```

### cmd/wait - Wait for Process Termination

**Arguments**: Process handle
**Returns**: `{:exit exit_code :stderr "..."}`

```qi
;; Terminate process and get result
(cmd/write py "exit()\n")
(cmd/wait py)
;; => {:exit 0 :stderr ""}
```

---

## Practical Examples

### Build Tool Integration

```qi
;; Execute Cargo build
(defn cargo-build [target]
  (let [[out err code] (cmd/pipe! f"cargo build --release --bin {target}")]
    (if (= code 0)
      (do
        (println "Build successful!")
        (println out)
        :ok)
      (do
        (println "Build failed:")
        (println err)
        :error))))

(cargo-build "qi-lang")
```

### Git Operations

```qi
;; Check git status
(defn git-status []
  (let [result (cmd/exec! "git status --porcelain")]
    (if (= (get result "exit") 0)
      (let [stdout (get result "stdout")]
        (if (str/blank? stdout)
          {:clean true :files []}
          {:clean false :files (str/lines stdout)}))
      {:error (get result "stderr")})))

(git-status)
;; => {:clean false :files ["M src/main.rs" "?? new-file.txt"]}

;; Commit changes
(defn git-commit [message]
  (do
    (cmd/exec "git add .")
    (let [code (cmd/exec f"git commit -m '{message}'")]
      (if (= code 0)
        (println "Commit successful!")
        (println "Commit failed (no changes?)")))))

(git-commit "feat: Add new feature")
```

---

## Error Handling

### Handle Command Failures

```qi
;; Catch errors with try-catch
(try
  (cmd/pipe "grep pattern file.txt")
  (catch e
    (println f"Search failed: {e}")
    nil))

;; Check exit code with pipe!
(let [[out err code] (cmd/pipe! "test -f data.txt")]
  (if (= code 0)
    (println "File exists")
    (println "File does not exist")))

;; Get error message with exec!
(let [result (cmd/exec! "ls non-existent")]
  (if (= (get result "exit") 0)
    (get result "stdout")
    (do
      (println f"Error: {(get result 'stderr')}")
      nil)))
```

---

## Security Considerations

### Prevent Command Injection

```qi
;; ❌ Dangerous: Pass user input directly to shell
(defn bad-search [pattern file]
  (cmd/pipe f"grep {pattern} {file}"))  ;; Injection possible!

(bad-search "test; rm -rf /" "data.txt")  ;; Dangerous!

;; ✅ Safe: Pass as vector (no shell)
(defn safe-search [pattern file]
  (cmd/pipe ["grep" pattern file]))

(safe-search "test; rm -rf /" "data.txt")  ;; Safe (treated as literal string)

;; ✅ Safe: Escape processing
(defn escape-shell-arg [s]
  (str "'" (str/replace s "'" "'\\''") "'"))

(defn safe-search-shell [pattern file]
  (cmd/pipe f"grep {(escape-shell-arg pattern)} {(escape-shell-arg file)}"))

(safe-search-shell "test; rm -rf /" "data.txt")  ;; Safe
```

### Path Validation

```qi
;; ❌ Dangerous: Path traversal attack
(defn bad-read-file [filename]
  (cmd/pipe f"cat {filename}"))

(bad-read-file "../../../etc/passwd")  ;; Dangerous!

;; ✅ Safe: Path validation
(defn safe-read-file [filename base-dir]
  (let [fullpath (path/join base-dir filename)
        canonical (path/canonicalize fullpath)]
    (if (str/starts-with? canonical base-dir)
      (cmd/pipe ["cat" canonical])
      (throw "Invalid path"))))

(safe-read-file "../../../etc/passwd" "/var/data")  ;; Error
(safe-read-file "file.txt" "/var/data")  ;; OK
```

---

## Function Reference

### Basic Execution
- `cmd/exec` - Execute command (returns exit code)
- `cmd/exec!` - Execute command (detailed version, returns stdout/stderr/exit)

### Pipeline Integration
- `cmd/pipe` - Pass stdin to command (returns stdout)
- `cmd/pipe!` - Execute command (returns [stdout stderr exit])
- `cmd/lines` - Split text into list of lines

### Stream Processing
- `cmd/stream-lines` - Stream command stdout line by line
- `cmd/stream-bytes` - Stream command stdout byte by byte

### Interactive Processes
- `cmd/interactive` - Launch bidirectional process
- `cmd/write` - Write to process stdin
- `cmd/read-line` - Read one line from process stdout
- `cmd/wait` - Wait for process termination

---

## Performance Optimization

### Executing Many Commands

```qi
;; ❌ Slow: Execute via shell each time
(files |> (map (fn [f] (cmd/exec f"wc -l {f}"))))

;; ✅ Fast: Combine into one command
(files |> (fn [fs] (join fs " ")) |> (cmd/pipe "wc -l"))

;; ✅ Fast: Parallel execution
(files |> (pmap (fn [f] (cmd/exec ["wc" "-l" f]))))
```

### Memory-Efficient Streaming

```qi
;; ❌ Memory inefficient: Load all data at once
(cmd/pipe "cat large-file.txt")
  |> str/lines
  |> (map process-line)

;; ✅ Memory efficient: Process as stream
(cmd/stream-lines "cat large-file.txt")
  |> (stream/map process-line)
  |> (stream/take 1000)
  |> realize
```
