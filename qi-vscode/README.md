# Qi Language VSCode Extension

Complete language support for **Qi - A Lisp that flows**.

## Features

### Syntax Highlighting
- **Keywords**: `def`, `defn`, `defn-`, `fn`, `let`, `if`, `do`, `match`, `try`, `defer`, `use`, `export`, `module`
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
- **Built-in Functions**: `go`, `chan`, `send!`, `recv!`, `atom`, `deref`, `swap!`, `reset!`, etc.
- **Module System**: Highlighting for `str/`, `list/`, `map/`, `io/`, `http/`, `db/`, etc.

### Editor Features
- Auto-closing pairs: `()`, `[]`, `{}`, `""`
- Bracket matching
- Comment toggling (`;` line comments)

### Commands (Planned)
- **Run Qi File** (`Ctrl+F5` / `Cmd+F5`)
- **Start Qi REPL** (`Ctrl+Shift+R` / `Cmd+Shift+R`)
- **Debug Qi File** (`F5`)
- **Format Document**
- **Show Documentation**

## Installation

### From Source
1. Clone this repository
2. Copy the `qi-vscode` folder to your VSCode extensions directory:
   - **Windows**: `%USERPROFILE%\.vscode\extensions`
   - **macOS**: `~/.vscode/extensions`
   - **Linux**: `~/.vscode/extensions`
3. Reload VSCode

### Development
1. Open this folder in VSCode
2. Press `F5` to launch Extension Development Host
3. Open a `.qi` file to see syntax highlighting

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
