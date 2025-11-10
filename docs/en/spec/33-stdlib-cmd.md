# Standard Library - Command Execution (cmd/)

**External Command and Shell Script Execution**

All functions belong to the `cmd/` module.

---

## Overview

The cmd module provides functions for executing external commands and shell scripts from Qi language. It integrates seamlessly with the pipeline operator (`|>`), allowing you to treat command execution as data flow.

**Key Features**:
- Command execution with exit code retrieval
- Shell script execution (via sh/cmd.exe)
- Standard I/O control
- Pipeline integration (Qi → Command → Qi)
- Stream processing (line-by-line, byte-by-byte)
- Bidirectional interactive processes

**Cross-platform Support**:
- **Unix/Linux/macOS**: Shell command execution via `/bin/sh`
- **Windows**: Command execution via `cmd.exe`

**Note**: This module is compiled with the `cmd-exec` feature.

---

## Basic Execution

### cmd/exec - Execute command (returns exit code)

**Arguments**: Command (string or vector)
**Returns**: Exit code (integer)
**Error**: When command is not found

```qi
;; Shell execution (string)
(cmd/exec "ls -la")                    ;; => 0
(cmd/exec "false")                     ;; => 1

;; Direct execution (vector)
(cmd/exec ["ls" "-la"])                ;; => 0
(cmd/exec ["git" "status"])            ;; => 0

;; Error handling with exit code
(let [code (cmd/exec "test -f data.txt")]
  (if (= code 0)
    (println "File exists")
    (println "File does not exist")))
```

**Shell vs Direct Execution**:
- **String**: Via shell (pipes, redirects, environment variable expansion)
- **Vector**: Direct execution (shell injection protection, faster)

---

## Shell Execution

### cmd/sh - Execute shell command (simple version)

**Arguments**: Command string
**Returns**: Exit code (integer)

```qi
;; Unix/Linux/macOS: via /bin/sh
(cmd/sh "ls -la | grep .qi")           ;; => 0

;; Windows: via cmd.exe /C
(cmd/sh "dir *.qi")                    ;; => 0

;; Pipes and redirects are available
(cmd/sh "cat *.txt | sort | uniq > result.txt")  ;; => 0
(cmd/sh "curl -s https://example.com | grep title")  ;; => 0

;; Multiple commands
(cmd/sh "cd build && make clean && make")  ;; => 0
```

### cmd/sh! - Execute shell command (detailed version)

**Arguments**: Command string
**Returns**: `{:stdout "..." :stderr "..." :exit 0}` map

```qi
;; Get stdout, stderr, and exit code
(cmd/sh! "cat *.txt | grep pattern | wc -l")
;; => {:stdout "      42\n" :stderr "" :exit 0}

;; Error details
(cmd/sh! "ls non-existent-file")
;; => {:stdout ""
;;     :stderr "ls: non-existent-file: No such file or directory\n"
;;     :exit 1}

;; Destructure result
(let [result (cmd/sh! "git status --porcelain")
      stdout (get result "stdout")
      exit (get result "exit")]
  (if (= exit 0)
    (if (str/blank? stdout)
      (println "Working directory is clean")
      (println f"Changes:\n{stdout}"))
    (println "Not a git repository")))
```

---

## Pipeline Integration

### cmd/pipe - Pass input to command

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

;; Practical example: Data formatting pipeline
(io/read-lines "data.csv")
  |> (map (fn [line] (str/split line ",")))
  |> (filter (fn [row] (> (len row) 2)))
  |> (map (fn [row] (join row "\t")))
  |> (fn [lines] (join lines "\n"))
  |> (cmd/pipe "sort -t $'\t' -k2,2")
  |> println
```

**Failure Behavior**:
```qi
;; Command failure throws an error
(try
  (cmd/pipe "grep pattern" "no match here")
  (catch e
    (println f"Command failed: {e}")))
;; => "Command failed: Command failed with exit code 1: ..."
```

### cmd/pipe! - Execute command (detailed version)

**Arguments**: Command, [input data]
**Returns**: `[stdout stderr exitcode]` vector

```qi
;; Get stdout, stderr, and exit code
(cmd/pipe! "cat test.txt")
;; => ["file content\n" "" 0]

;; Use with destructuring
(let [[out err code] (cmd/pipe! "ls -la")]
  (if (= code 0)
    (println out)
    (println f"Error: {err}")))

;; Pass input via pipeline
(["test"] |> (cmd/pipe! ["wc" "-l"]))
;; => ["       1\n" "" 0]

;; Practical example: Build tool execution
(defn build [target]
  (let [[out err code] (cmd/pipe! f"cargo build --release --bin {target}")]
    (if (= code 0)
      {:status :ok :output out}
      {:status :error :message err})))

(build "qi-lang")
;; => {:status :ok :output "   Compiling qi-lang v0.1.0\n..."}
```

### cmd/lines - Split text into list of lines (helper)

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

### cmd/stream-lines - Line-by-line stream

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

;; Practical example: Real-time log monitoring
(defn watch-errors [logfile]
  (cmd/stream-lines f"tail -f {logfile}")
    |> (stream/filter (fn [line] (str/contains? line "ERROR")))
    |> (stream/map (fn [line]
         (let [timestamp (time/now)]
           {:time timestamp :message line})))
    |> (stream/each send-alert))

(watch-errors "/var/log/app.log")
```

### cmd/stream-bytes - Byte-by-byte stream

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

;; Practical example: Download with progress
(defn download-with-progress [url output]
  (let [total (atom 0)]
    (cmd/stream-bytes f"curl -L {url}" 4096)
      |> (stream/each (fn [chunk]
           (let [size (len chunk)]
             (swap! total (fn [t] (+ t size)))
             (println f"Downloaded: {@total} bytes"))))
      |> (stream/reduce str/concat "")
      |> (fn [data] (io/write-file data output))))

(download-with-progress "https://example.com/file.zip" "/tmp/download.zip")
```

---

## Interactive Processes

### cmd/interactive - Launch bidirectional process

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

### cmd/write - Write to process

**Arguments**: Process handle, data (string)
**Returns**: nil

```qi
;; Send multiple lines to REPL
(cmd/write py "def greet(name):\n")
(cmd/write py "    return f'Hello, {name}!'\n")
(cmd/write py "\n")
(cmd/write py "print(greet('Qi'))\n")
```

### cmd/read-line - Read line from process

**Arguments**: Process handle
**Returns**: Read line (string), `nil` on EOF

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

### cmd/wait - Wait for process to terminate

**Arguments**: Process handle
**Returns**: `{:exit exit_code :stderr "..."}`

```qi
;; Terminate process and get result
(cmd/write py "exit()\n")
(cmd/wait py)
;; => {:exit 0 :stderr ""}
```

### Practical Example: Interactive Shell

```qi
(defn run-script [script-lines]
  (let [proc (cmd/interactive "python3 -i")]
    ;; Execute script
    (script-lines
     |> (map (fn [line] (cmd/write proc (str line "\n"))))
     |> realize)

    ;; Collect results
    (let [results (loop [acc []]
                    (let [line (cmd/read-line proc)]
                      (if (some? line)
                        (recur (conj acc line))
                        acc)))]
      ;; Terminate process
      (cmd/write proc "exit()\n")
      (cmd/wait proc)
      results)))

(run-script ["print(1+1)" "print('hello')" "print(2*3)"])
;; => ["2" "hello" "6"]
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
  (let [result (cmd/sh! "git status --porcelain")]
    (if (= (get result "exit") 0)
      (let [stdout (get result "stdout")]
        (if (str/blank? stdout)
          {:clean true :files []}
          {:clean false :files (str/lines stdout)}))
      {:error (get result "stderr")})))

(git-status)
;; => {:clean false :files ["M src/main.rs" "?? new-file.txt"]}

;; Commit git changes
(defn git-commit [message]
  (do
    (cmd/sh "git add .")
    (let [code (cmd/exec f"git commit -m '{message}'")]
      (if (= code 0)
        (println "Commit successful!")
        (println "Commit failed (no changes?)")))))

(git-commit "feat: Add new feature")
```

### Data Processing Pipeline

```qi
;; Process CSV with SQLite
(defn process-csv-with-sqlite [csv-file query]
  (let [db "/tmp/temp.db"]
    ;; Import CSV
    (cmd/sh f"sqlite3 {db} '.mode csv' '.import {csv-file} data'")

    ;; Execute SQL query
    (cmd/pipe f"sqlite3 -csv {db} \"{query}\"")
      |> str/lines
      |> (map (fn [line] (str/split line ",")))
      |> (map (fn [row] (zipmap ["id" "name" "value"] row)))))

(process-csv-with-sqlite "data.csv" "SELECT * FROM data WHERE value > 100")
;; => [{:id "1" :name "Alice" :value "120"} ...]
```

### System Monitoring

```qi
;; Monitor disk usage
(defn check-disk-usage []
  (cmd/pipe "df -h /")
    |> str/lines
    |> (drop 1)  ;; Skip header
    |> first
    |> (str/split _ " ")
    |> (filter (fn [s] (not (str/blank? s))))
    |> (fn [cols] {:filesystem (nth cols 0)
                   :size (nth cols 1)
                   :used (nth cols 2)
                   :avail (nth cols 3)
                   :percent (nth cols 4)}))

(check-disk-usage)
;; => {:filesystem "/dev/disk1s1" :size "931Gi" :used "450Gi"
;;     :avail "481Gi" :percent "49%"}

;; Monitor CPU usage
(defn watch-cpu []
  (cmd/stream-lines "top -l 0 -s 1")
    |> (stream/filter (fn [line] (str/contains? line "CPU usage")))
    |> (stream/map (fn [line]
         (let [parts (str/split line " ")]
           {:user (nth parts 2) :sys (nth parts 4)})))
    |> (stream/take 10)
    |> realize)

(watch-cpu)
;; => [{:user "15.5%" :sys "8.2%"} {:user "12.3%" :sys "6.1%"} ...]
```

### Test Script Execution

```qi
;; Run tests and summarize results
(defn run-tests []
  (let [[out err code] (cmd/pipe! "cargo test -- --nocapture")]
    {:passed (str/contains? out "test result: ok")
     :output out
     :errors err
     :exit-code code}))

(run-tests)
;; => {:passed true :output "running 15 tests\n..." :errors "" :exit-code 0}
```

### Multi-platform Commands

```qi
;; Execute OS-specific commands
(defn list-processes []
  (if (= (os/platform) "windows")
    (cmd/pipe "tasklist")
    (cmd/pipe "ps aux"))
  |> str/lines)

(list-processes)
;; Unix: => ["USER       PID %CPU %MEM ..." "root         1  0.0  0.1 ..." ...]
;; Windows: => ["Image Name           PID Session Name ..." "System Idle Process   0 ..." ...]
```

---

## Error Handling

### Handling Command Failures

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

;; Get error message with sh!
(let [result (cmd/sh! "ls non-existent")]
  (if (= (get result "exit") 0)
    (get result "stdout")
    (do
      (println f"Error: {(get result 'stderr')}")
      nil)))
```

### Timeout Handling

```qi
;; Execute with timeout (using external command)
(defn exec-with-timeout [cmd timeout-sec]
  (let [timeout-cmd (if (= (os/platform) "windows")
                       f"timeout /t {timeout-sec} && {cmd}"
                       f"timeout {timeout-sec} {cmd}")]
    (cmd/sh! timeout-cmd)))

(exec-with-timeout "sleep 10" 5)
;; => {:stdout "" :stderr "timeout: killed" :exit 124}
```

---

## Security Considerations

### Command Injection Protection

```qi
;; ❌ Dangerous: Pass user input directly to shell
(defn bad-search [pattern file]
  (cmd/pipe f"grep {pattern} {file}"))  ;; Injection possible!

(bad-search "test; rm -rf /" "data.txt")  ;; Dangerous!

;; ✅ Safe: Pass as vector (no shell)
(defn safe-search [pattern file]
  (cmd/pipe ["grep" pattern file]))

(safe-search "test; rm -rf /" "data.txt")  ;; Safe (treated as literal string)

;; ✅ Safe: Escape handling
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

### Permission Management

```qi
;; ✅ Run with minimal privileges
(defn run-sandboxed [cmd]
  ;; Run with user privileges (no sudo)
  (cmd/sh cmd))

;; ❌ Dangerous: Avoid sudo/admin privileges
;; (cmd/sh "sudo rm -rf /")  ;; Never do this!
```

---

## Function List

### Basic Execution
- `cmd/exec` - Execute command (returns exit code)
- `cmd/sh` - Execute shell command (simple version)
- `cmd/sh!` - Execute shell command (detailed version, stdout/stderr/exit)

### Pipeline Integration
- `cmd/pipe` - Pass input to command (returns stdout)
- `cmd/pipe!` - Execute command (returns [stdout stderr exit])
- `cmd/lines` - Split text into list of lines

### Stream Processing
- `cmd/stream-lines` - Stream command stdout line-by-line
- `cmd/stream-bytes` - Stream command stdout byte-by-byte

### Interactive Processes
- `cmd/interactive` - Launch bidirectional process
- `cmd/write` - Write to process stdin
- `cmd/read-line` - Read line from process stdout
- `cmd/wait` - Wait for process to terminate

---

## Performance Optimization

### Executing Many Commands

```qi
;; ❌ Slow: Execute via shell every time
(files |> (map (fn [f] (cmd/sh f"wc -l {f}"))))

;; ✅ Fast: Combine into single command
(files |> (fn [fs] (join fs " ")) |> (cmd/pipe "wc -l"))

;; ✅ Fast: Parallel execution
(files |> (pmap (fn [f] (cmd/exec ["wc" "-l" f]))))
```

### Memory Efficiency with Streams

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

---

## Debugging

```qi
;; Trace command execution
(defn trace-cmd [cmd]
  (do
    (println f"Executing: {cmd}")
    (let [[out err code] (cmd/pipe! cmd)]
      (println f"Exit code: {code}")
      (println f"stdout: {out}")
      (println f"stderr: {err}")
      [out err code])))

(trace-cmd "ls -la")

;; Check environment variables
(cmd/pipe "env")
  |> str/lines
  |> (filter (fn [line] (str/starts-with? line "PATH=")))
  |> (map println)
```
