# Qi CLI Reference

Complete reference for the Qi command-line tool.

## Table of Contents

- [Basic Usage](#basic-usage)
- [Command List](#command-list)
- [Project Management](#project-management)
- [Code Execution](#code-execution)
- [REPL](#repl)
- [Environment Variables](#environment-variables)
- [Exit Codes](#exit-codes)

---

## Basic Usage

```bash
qi [OPTIONS] [FILE]
```

Starting without arguments launches REPL mode.

---

## Command List

### Project Management

#### `qi new <name> [OPTIONS]`

Creates a new Qi project.

**Arguments:**
- `<name>` - Project name (directory name)

**Options:**
- `-t, --template <template>` - Template to use (default: `basic`)

**Examples:**
```bash
# Create a basic project
qi new my-project

# Create an HTTP server project
qi new myapi --template http-server
qi new myapi -t http-server
```

**Behavior:**
1. Creates project directory
2. Interactively prompts for project metadata (name, version, description, author, license)
3. Copies files from template
4. Generates `qi.toml`

---

#### `qi template list`

Displays a list of available templates.

**Examples:**
```bash
qi template list
```

**Sample Output:**
```
Available templates:
  basic            - Basic project structure
  http-server      - JSON API server with routing
```

---

#### `qi template info <name>`

Displays detailed information about a template.

**Arguments:**
- `<name>` - Template name

**Examples:**
```bash
qi template info http-server
```

**Sample Output:**
```
Template: http-server
Description: JSON API server with routing
Author: Qi Team
Version: 0.1.0
Required features: http-server, format-json
Location: std/templates/http-server
```

---

### Code Execution

#### `qi <file>`

Executes a Qi script file.

**Arguments:**
- `<file>` - Script file to execute (`.qi`)

**Examples:**
```bash
qi script.qi
qi main.qi
qi examples/example.qi
```

---

#### `qi -e <code>` / `qi -c <code>`

Executes code directly (one-liner).

**Options:**
- `-e, -c <code>` - Qi code to execute

**Automatic stdin Binding:**

When input is provided via pipe, it is automatically stored in the `stdin` variable as a vector of strings:

```bash
# Count lines
cat data.txt | qi -e '(count stdin)'

# Process CSV data (exclude empty lines)
cat users.csv | qi -e '(stdin |> (filter (fn [x] (> (len x) 0))))'

# Exclude empty lines and convert to uppercase
echo -e "hello\n\nworld" | qi -e '(stdin |> (filter (fn [x] (> (len x) 0))) |> (map str/upper))'
```

**Examples:**
```bash
qi -e '(+ 1 2 3)'
# => 6

qi -e '(println "Hello, Qi!")'
# => Hello, Qi!

qi -c '([1 2 3 4 5] |> (map (fn [x] (* x 2))) |> (reduce + 0))'
# => 30
```

---

#### `qi -`

Reads and executes code from stdin.

**Examples:**
```bash
echo '(println 42)' | qi -

cat script.qi | qi -
```

---

### REPL

#### `qi` (no arguments)

Launches the interactive REPL.

**Examples:**
```bash
qi
```

**REPL Commands:**
- `:help` - Display help
- `:doc <name>` - Display function documentation
- `:vars` - Display defined variables
- `:funcs` - Display defined functions
- `:builtins [filter]` - Display built-in functions (filterable)
- `:clear` - Clear environment
- `:load <file>` - Load a file
- `:reload` - Reload the last loaded file
- `:quit` - Exit REPL

**Features:**
- Tab completion (function names, variable names, REPL commands)
- History (saved in `~/.qi_history`)
- Multi-line input (automatic parenthesis balance detection)
- Ctrl+C to cancel input
- Ctrl+D or `:quit` to exit

---

#### `qi -l <file>` / `qi --load <file>`

Loads a file and then launches the REPL.

**Options:**
- `-l, --load <file>` - File to load

**Examples:**
```bash
qi -l utils.qi
qi --load lib.qi
```

---

### Other Options

#### `qi -h` / `qi --help`

Displays help message.

```bash
qi --help
qi -h
```

---

#### `qi -v` / `qi --version`

Displays version information.

```bash
qi --version
qi -v
```

---

#### `qi --upgrade`

Upgrades Qi to the latest version from GitHub Releases.

```bash
qi --upgrade
```

**Behavior:**
1. Checks the latest release from GitHub
2. Downloads the appropriate binary for your platform
3. Replaces the current binary (backs up to `.old`)
4. Displays upgrade status

**Supported Platforms:**
- macOS (Apple Silicon / Intel)
- Linux (x86_64 / aarch64)
- Windows (x86_64)

**Examples:**
```bash
# Upgrade to the latest version
qi --upgrade

# Check current version
qi --version
```

---

## Environment Variables

### `QI_LANG`

Sets the language for Qi messages.

**Values:**
- `ja` - Japanese
- `en` - English (default)

**Examples:**
```bash
QI_LANG=ja qi script.qi
QI_LANG=en qi --help
```

**Behavior:**
- Error messages, help messages, and REPL messages are displayed in the specified language
- Documentation (`:doc`) is also displayed in the specified language

---

### `LANG`

System locale setting. Used as a fallback when `QI_LANG` is not set.

**Examples:**
```bash
LANG=ja_JP.UTF-8 qi script.qi
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Normal exit |
| `1` | Error occurred (syntax error, runtime error, file not found, etc.) |

**Examples:**
```bash
qi script.qi
echo $?  # Check exit code

qi -e '(+ 1 2 3)'
echo $?  # => 0

qi -e '(invalid syntax'
echo $?  # => 1
```

---

## Template Search Order

The `qi new` command searches for templates in the following order:

1. `./.qi/templates/<name>/` - Project-local
2. `~/.qi/templates/<name>/` - User-global
3. `<qi-binary-dir>/std/templates/<name>/` - Installed version
4. `std/templates/<name>/` - Development version

The first template found is used.

---

## Usage Examples

### Project Creation Workflow

```bash
# Check template list
qi template list

# Create an HTTP server project
qi new myapi --template http-server

# Navigate to project directory
cd myapi

# Start the server
qi main.qi
```

### Data Processing with One-liners

```bash
# Fibonacci sequence
qi -e '(defn fib [n] (if (<= n 1) n (+ (fib (- n 1)) (fib (- n 2))))) (fib 10)'

# Pipeline processing
qi -e '([1 2 3 4 5] |> (filter (fn [x] (> x 2))) |> (map (fn [x] (* x 10))))'

# JSON processing (when HTTP feature is enabled)
qi -e '(json/parse "{\"name\":\"Alice\",\"age\":30}")'
```

### Development with REPL

```bash
# Load library and start REPL
qi -l src/lib.qi

# Test functions in REPL
qi:1> (greet "World")
Hello, World!

# Check documentation
qi:2> :doc map

# Search built-in functions
qi:3> :builtins str
```

---

## Related Documentation

- [Project Management and qi.toml Specification](project.md) - qi.toml, templates, project structure
- [Tutorial](tutorial/01-getting-started.md) - Practical usage
- [Language Specification](spec/README.md) - Qi language syntax and features
