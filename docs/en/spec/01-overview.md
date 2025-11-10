# Qi Language Overview

**Qi - A Lisp that flows**

A simple, fast, and concise modern Lisp with strong support for pipelines, pattern matching, and concurrency/parallelism.

**Making parallelism and concurrency easy is Qi's core strength** - Thread-safe design with 3-layer concurrency architecture.

> **Note**: See [style-guide.md](../style-guide.md) for coding style and formatting conventions.

---

## Language Philosophy - Flow-Oriented Programming

### Core Concept

**"Data flows, programs design the flow"**

Qi embodies **Flow-Oriented Programming**:

1. **Data flow as a first-class citizen**
   - Pipeline operator `|>` at the language's core
   - `match` as a control structure for branching and transforming flows
   - Combine small transformations to create larger flows
   - Unix philosophy's "Do One Thing Well" realized in functional style

2. **Simple, Fast, Concise**
   - **Simple**: 9 special forms, minimal notation, gentle learning curve
   - **Fast**: Lightweight, fast startup, future JIT compilation
   - **Concise**: Short function names, pipelines, `defn` sugar for expressiveness

3. **Energy Flow**
   - Data flows unidirectionally (left-to-right, top-to-bottom)
   - Side effects observed via taps (`tap>`)
   - Parallel processing expressed as flow branching/merging
   - **Easy parallelism & concurrency** - Natural parallelization with thread-safe design

4. **Pragmatism**
   - Practical utility over Lisp purity
   - Modern features actively adopted (f-strings, pattern matching)
   - Batteries included (rich string operations, utilities)

---

## Design Principles

1. **Readability > Writability**
   - Pipelines read top-to-bottom, left-to-right
   - Data flow visible at a glance

2. **Composability**
   - Combine small functions to build larger processes
   - Each step independently testable

3. **Progressive Disclosure**
   - Beginners: Start with basic `|>`
   - Intermediate: Leverage `match`, `loop`
   - Advanced: Master parallel processing

4. **Runtime Efficiency**
   - Pipeline optimization
   - Lazy evaluation to avoid unnecessary computation
   - Natural scaling through parallelism

---

## File Extension

```
.qi
```

---

## Usage - Running Qi

Qi supports 3 execution modes + standard input:

### 1. REPL (Interactive Mode)

```bash
qi
# or
qi -l utils.qi  # Preload file and start REPL
```

### 2. Script File Execution

```bash
qi script.qi
qi examples/hello.qi
```

### 3. One-liner Execution

```bash
qi -e '(println "Hello!")'
qi -e '((range 1 10) |> (map (fn [x] (* x x))) |> sum)'
```

### 4. Standard Input Execution

**Following Unix Philosophy - Pipeline Integration**

```bash
# From echo
echo '(println "Hello from stdin!")' | qi -

# Multiline script with heredoc
qi - <<'EOF'
(def data [1 2 3 4 5])
(def result (data |> (map (fn [x] (* x x))) |> sum))
(println (str "Sum of squares: " result))
EOF

# Generate Qi script from other commands
jq -r '.script' config.json | qi -
curl -s https://example.com/script.qi | qi -

# No temporary files needed (security improvement)
echo "$SECRET_SCRIPT" | qi -

# Dynamic script execution in CI/CD
cat automation.qi | qi -
```

#### Why Standard Input Execution Matters

1. **Unix Philosophy** - Other tools (python/node/ruby) support `-`
2. **Pipeline Integration** - Execute command output directly
3. **Security** - No sensitive scripts left on disk
4. **Dynamic Generation** - Other tools generate Qi code → immediate execution
5. **CI/CD Support** - Inject scripts from GitHub Secrets

```bash
# Real-world example: Extract and execute Qi script from JSON
cat automation.json | jq -r '.tasks.cleanup.script' | qi -

# Real-world example: Dynamically generate data processing script
./generate-processor.sh --type=csv --columns=3 | qi -
```

---

## Basic Design

### Namespace

**Lisp-1 (Scheme-style)** - Variables and functions share the same namespace

```qi
(def add (fn [x y] (+ x y)))
(def op add)           ;; Assign function to variable
(op 1 2)               ;; 3
```

### nil and bool

**nil and bool are distinct, but nil is falsy in conditionals**

```qi
nil false true          ;; Three distinct values
(if nil "yes" "no")     ;; "no" (nil is falsy)
(if false "yes" "no")   ;; "no" (false is falsy)
(if 0 "yes" "no")       ;; "yes" (0 is truthy)
(if "" "yes" "no")      ;; "yes" (empty string is truthy)

;; Explicit comparison
(= x nil)               ;; nil check
(= x false)             ;; false check
```

### Type System

Qi is dynamically typed with the following basic types:

- **Numbers**: Integers, floating-point
- **Strings**: UTF-8, f-string support
- **Booleans**: `true`, `false`
- **nil**: Represents absence of value
- **Vectors**: `[1 2 3]`
- **Maps**: `{:key "value"}`
- **Lists**: `'(1 2 3)` (quote required)
- **Functions**: First-class objects
- **Keywords**: `:keyword`
