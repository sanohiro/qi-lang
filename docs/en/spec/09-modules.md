# Module System

**Namespace Separation by File Unit**

Qi organizes modules on a per-file basis, separating namespaces.

---

## Basic Concepts

- **Search by file name, display by module name**: `use` searches by file name, module name can be changed with `module` declaration
- **use auto-loads**: `use` reads file + imports symbols
- **load is side-effects only**: Executes configuration files etc. without importing
- **No export = all public**: Without `export` declaration, everything is public (except `defn-`)
- **With export = selective public**: With `export` declaration, only explicitly listed items are public
- **defn- is completely private**: Always private regardless of `export`

---

## module Declaration

`module` specifies **the display name of a module** (optional).

### Basic Usage

```qi
;; http.qi
(module web-api)  ;; Change display name to 'web-api'

(defn get [url] ...)

;; Search is performed with 'http.qi'
;; Access becomes 'web-api/get'
```

### Specification

- **Optional**: If not present, file name (basename) becomes module name
- **Does not affect search**: `use http` searches for `http.qi` (not module name)
- **Changes display name only**: Prefix for access changes
- **One per file**: Multiple `module` declarations cause an error
- **Position**: Recommended at file top (technically allowed anywhere)
- **Return value**: `nil`

### Examples

```qi
;; http.qi
(module web-api)

;; User side
(use http)              ;; Searches for http.qi → Success
(web-api/get "...")     ;; OK (access by module name)
(http/get "...")        ;; Error (cannot access by file name)
```

---

## export Declaration

`export` controls **which symbols to publish** (optional).

### Mode A: No export declaration → Everything public (default)

```qi
;; utils.qi
(defn add [a b] (+ a b))        ;; Public (defn)
(defn multiply [a b] (* a b))   ;; Public (defn)
(defn- helper [x] (* x 2))      ;; Private (defn-)

;; All defn are automatically public
```

### Mode B: With export declaration → Selective public

```qi
;; utils.qi
(defn add [a b] (+ a b))        ;; Private (not in export)
(defn multiply [a b] (* a b))   ;; Public (in export)
(defn- helper [x] (* x 2))      ;; Private (always)

(export multiply)  ;; Only multiply is public
```

### Specification

- **Default public**: No `export` → all `defn` public, all `defn-` private
- **Selective public**: With `export` → only explicitly listed are public, others private
- **defn- is always private**: Error if written in `export`
- **Multiple declarations allowed**: Cumulative
- **Position**: Anywhere OK (recommended at end)
- **Return value**: `nil`

### Multiple export Declarations

```qi
(defn get [url] ...)
(defn post [url data] ...)
(defn put [url data] ...)

(export get)        ;; Only get public
(export post put)   ;; Add post, put (cumulative)
;; Result: get, post, put all public
```

---

## Module Definition Examples

### Pattern 1: Simple (no export)

```qi
;; math.qi
(defn add [a b] (+ a b))        ;; Public
(defn sub [a b] (- a b))        ;; Public
(defn- validate [x] (> x 0))    ;; Private

;; From outside: (math/add ...), (math/sub ...) OK
;;               (math/validate ...) Error
```

### Pattern 2: Explicit export

```qi
;; math.qi
(defn add [a b] (+ a b))        ;; Public (in export)
(defn sub [a b] (- a b))        ;; Private (not in export)
(defn multiply [a b] (* a b))   ;; Private (not in export)
(defn- validate [x] (> x 0))    ;; Private (always)

(export add)  ;; Only add public

;; From outside: (math/add ...) OK
;;               (math/sub ...), (math/multiply ...) Error
```

### Pattern 3: module declaration + export

```qi
;; http.qi
(module web-client)  ;; Change display name

(defn- build-url [base path] (str base "/" path))
(defn get [url] ...)
(defn post [url data] ...)
(defn internal-func [x] ...)  ;; Private (not exported)

(export get post)

;; From outside:
;; (use http)  ;; Searches for http.qi
;; (web-client/get ...) OK (access by module name)
;; (web-client/post ...) OK
;; (web-client/internal-func ...) Error (private)
```

---

## Import (use)

`use` performs **file loading + symbol import**.

### Pattern 1: Import specific functions only (recommended)

```qi
(use http :only [get post])
(get "https://...")                      ;; OK
```

### Pattern 2: Alias (module/function format)

```qi
(use http :as h)
(h/get "https://...")                    ;; OK
```

### Pattern 3: Import all

```qi
(use http :all)
(get "https://...")                      ;; OK
(post "https://..." {:data 123})         ;; OK
```

### Pattern 4: Path specification

```qi
(use "lib/utils" :only [format-date])    ;; Load lib/utils.qi
(use "./vendor/json" :as json)           ;; Relative path
```

---

## Module Name Resolution Rules

**Principle: Search by file name, access by module name**:

### Case 1: Module name only → Auto-search

```qi
(use http :only [get])
;; 1. Search for http.qi (./http.qi, ~/.qi/modules/http.qi, etc.)
;; 2. Check module declaration in http.qi
;;    - (module web-api) → Access as web-api/get
;;    - None → Access as http/get
```

### Case 2: Path specification → Search by basename

```qi
(use "lib/http" :only [get])
;; 1. Load lib/http.qi
;; 2. Check module declaration in lib/http.qi
;;    - (module web-api) → Access as web-api/get
;;    - None → Access as http/get (basename)
```

### Case 3: Alias with :as

```qi
(use "lib/http" :as h)
;; 1. Load lib/http.qi
;; 2. Use alias 'h' regardless of module declaration
;; => Access as h/get
```

### Case 4: Combination with module declaration

```qi
;; lib/http.qi
(module web-client)
(defn get [url] ...)

;; User side
(use "lib/http")
(web-client/get "...")  ;; OK (access by module name)

(use "lib/http" :as h)
(h/get "...")           ;; OK (alias takes priority)
```

---

## Name Collision Handling

```qi
;; lib1/utils.qi → (module string-utils)
;; lib2/utils.qi → (module string-utils)  ← Same module name!

(use "lib1/utils")  ;; Module name: string-utils
(use "lib2/utils")  ;; Error: module 'string-utils' already loaded

;; Solution 1: Alias with :as
(use "lib1/utils" :as utils1)
(use "lib2/utils" :as utils2)

;; Solution 2: Change module declaration in file
;; Change lib2/utils.qi to (module text-utils)
```

---

## load (Side Effects Only)

`load` only evaluates a file without importing symbols.
Used for executing configuration files or initialization scripts.

```qi
;; Load configuration file (side effects only)
(load "config.qi")

;; Difference from use
(use http)    ;; Load http.qi + import symbols
(load "init") ;; Only evaluate init.qi (no symbol import)
```

---

## Public/Private Decision Flow

```
Function definition public/private:

Defined with defn-
  → Always private (cannot export)

Defined with defn
  → Does export declaration exist?
      YES → Is it in export list?
              YES → Public
              NO  → Private
      NO  → Public (default)
```

---

## Practical Examples

### Creating a Library

```qi
;; lib/string-utils.qi
(module string-utils)

;; Public functions
(defn upper [s]
  (str/upper s))

(defn lower [s]
  (str/lower s))

;; Internal function (private)
(defn- validate [s]
  (and (string? s) (> (len s) 0)))

;; Explicit export
(export upper lower)
```

### Using a Library

```qi
;; main.qi
(use "lib/string-utils" :only [upper lower])

(upper "hello")  ;; => "HELLO"
(lower "WORLD")  ;; => "world"

;; Or
(use "lib/string-utils" :as str-util)
(str-util/upper "hello")  ;; => "HELLO"
```

### Loading Configuration Files

```qi
;; config.qi
(def api-key "secret-key")
(def db-host "localhost")

;; main.qi
(load "config")  ;; Side effects only (symbols not imported)

;; api-key and db-host are defined globally
(println api-key)  ;; => "secret-key"
```

---

## Initialization File (.qi/init.qi)

**Auto-load for REPL and One-liner Execution**

Qi automatically loads initialization files when running in REPL or one-liner (`-e` option) mode.

### Loading Order

```
1. ~/.qi/init.qi  (User global settings - priority)
2. ./.qi/init.qi  (Project local settings)
```

- If both files exist, **both are loaded**
- Files are skipped if they don't exist
- If an error occurs, a warning is displayed but execution continues

### Target Execution Modes

- ✅ **REPL mode** (`qi`)
- ✅ **One-liner mode** (`qi -e '...'`)
- ❌ Script file execution (`qi script.qi`) - Not applicable

### User Global Settings (~/.qi/init.qi)

Write settings you want to use across all Qi projects:

```qi
;; ~/.qi/init.qi - User global settings

;; Preload commonly used libraries
(use "table" :as table)
(use "string" :as str)

;; Debug helper function
(defn dbg [x]
  (do (println (str "DEBUG: " x))
      x))

;; Display list length and return
(defn show-len [lst]
  (do (println (str "length: " (len lst)))
      lst))
```

### Project Local Settings (./.qi/init.qi)

Write project-specific settings:

```qi
;; ./.qi/init.qi - Project local settings

;; Load project-specific library
(use "lib/utils" :as utils)

;; Project-specific constants
(def db-host "localhost")
(def db-port 5432)

;; Development helper
(defn reload-config []
  (load ".env"))
```

### One-liner Usage Example

When libraries are preloaded in initialization files, you can use them in one-liners without `use`:

```bash
# Write (use "table" :as table) in ~/.qi/init.qi

# Process CSV from pipe (table library already available)
cat data.csv | qi -e '(stdin |> split "\n" |> table/parse-csv |> (table/where (fn [row] (> (get row :age) 30))))'
```

### Notes

- Initialization files are REPL/one-liner only
- Not loaded when executing script files (use explicit `load`)
- Project local settings load **after** user global settings
- For same-name definitions, later loaded ones take priority
