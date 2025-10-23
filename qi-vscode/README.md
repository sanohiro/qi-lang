# Qi Language VSCode Extension

Complete language support for **Qi - A Lisp that flows**.

## Features

### Syntax Highlighting
- **Keywords**: `def`, `defn`, `defn-`, `fn`, `let`, `if`, `do`, `when`, `while`, `until`, `while-some`, `until-error`, `each`, `match`, `try`, `defer`, `loop`, `recur`, `use`, `export`, `module`
- **Keyword Literals**: `:keyword` syntax
- **Operators**:
  - Pipeline operators: `|>`, `||>`, `|>?`, `~>`
  - Match arrows: `->`, `=>`
  - Spread operator: `...`
  - Arithmetic: `+`, `-`, `*`, `/`, `%`, `=`, `!=`, `<`, `>`, `<=`, `>=`
- **Strings**:
  - Regular strings: `"hello"`
  - Triple-quoted strings: `"""multi-line"""`
  - F-strings: `f"Hello {name}"`
  - Triple-quoted f-strings: `f"""multi-line with {interpolation}"""`
- **Comments**: Line comments with `;`
- **Numbers**: Integer and floating-point literals
- **Constants**: `nil`, `true`, `false`
- **Predicates**: `nil?`, `list?`, `map?`, `string?`, `integer?`, etc.
- **Built-in Functions**: `atom`, `deref`, `swap!`, `reset!`, `print`, `println`, `error`, `tap`, etc.
- **Module System**: Highlighting for `str/`, `list/`, `map/`, `go/`, `io/`, `http/`, `db/`, `kvs/`, etc.

### Editor Features
- Auto-closing pairs: `()`, `[]`, `{}`, `""`
- Bracket matching
- Comment toggling (`;` line comments)
- **Code Formatting**: Automatic formatting using `qi fmt`

### Commands
- **Run Qi File** (`Ctrl+F5` / `Cmd+F5`) - Execute current Qi file
- **Start Qi REPL** (`Ctrl+Shift+R` / `Cmd+Shift+R`) - Start interactive REPL
- **Debug Qi File** (`F5`) - Debug current file with breakpoints and step execution
- **Format Document** - Format code according to docs/style-guide.md
- **Show Documentation** - Open Qi documentation

### Debugging
The extension provides full debugging support via the Debug Adapter Protocol (DAP):

- **Breakpoints**: Set breakpoints by clicking on the gutter
- **Step Execution**: Step over, step into, step out of functions
- **Call Stack**: View the current call stack
- **Variables**: Inspect variables and scopes (planned)
- **Continue/Pause**: Control program execution

To start debugging:
1. Open a `.qi` file
2. Set breakpoints by clicking on the line numbers
3. Press `F5` or click "Debug Qi File" in the command palette
4. Use the debug toolbar to step through your code

**Note**: Make sure the Qi executable is built with the `dap-server` feature:
```bash
cargo build --features dap-server --release
```

## Installation

### From Source
1. Clone this repository
2. Copy the `qi-vscode` folder to your VSCode extensions directory:
   - **Windows**: `%USERPROFILE%\.vscode\extensions`
   - **macOS**: `~/.vscode/extensions`
   - **Linux**: `~/.vscode/extensions`
3. Reload VSCode

### Development
1. Install dependencies:
   ```bash
   cd qi-vscode
   npm install
   ```
2. Compile TypeScript:
   ```bash
   npm run compile
   ```
3. Open this folder in VSCode
4. Press `F5` to launch Extension Development Host
5. Open a `.qi` file to test the extension

## Configuration

The extension can be configured via VSCode settings:

```json
{
  "qi.executablePath": "qi",
  "qi.enableFormatting": true,
  "qi.enableLinting": true,
  "qi.repl.autoStart": false
}
```

### Settings

- **`qi.executablePath`** (string, default: `"qi"`) - Path to the Qi executable. If `qi` is in your PATH, you can use `"qi"`. Otherwise, specify the full path.
- **`qi.enableFormatting`** (boolean, default: `true`) - Enable automatic code formatting with `qi fmt`.
- **`qi.enableLinting`** (boolean, default: `true`) - Enable linting (planned).
- **`qi.repl.autoStart`** (boolean, default: `false`) - Automatically start REPL when opening Qi files.

## Language Overview

Qi is a modern Lisp dialect focused on flow-oriented programming with:
- Pipeline operators for data transformation
- Pattern matching with destructuring
- Async/concurrent programming primitives
- Module system with public/private exports
- Rich standard library (HTTP, DB, JSON, CSV, etc.)

## Examples

```qi
;; Pipeline processing
(use str :as s)
(use list :only [take filter])

("hello world"
  |> s/upper
  |> (s/split " ")
  |> (filter (fn [w] (> (s/length w) 4)))
  |> first)
;=> "HELLO"

;; F-strings with interpolation
(def name "Alice")
(def age 30)
(println f"Hello, {name}! You are {age} years old.")
;; => Hello, Alice! You are 30 years old.

;; Pattern matching
(match [1 2 3]
  [] -> "empty"
  [x] -> f"single: {x}"
  [x y ...rest] -> f"x={x}, y={y}, rest={rest}")
;=> "x=1, y=2, rest=(3)"

;; Module system
(defn- private-helper []
  "Only visible in this module")

(defn public-api [data]
  "Public API function"
  (private-helper))

(export [public-api])
```

## Links

- [Qi Language Repository](https://github.com/sanohiro/qi-lang)
- [Language Specification](https://github.com/sanohiro/qi-lang/blob/master/SPEC.md)

## License

MIT
