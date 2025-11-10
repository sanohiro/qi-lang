# Qi Project Management

Complete reference for Qi project structure, configuration files (qi.toml), and template system.

## Table of Contents

- [Project Structure](#project-structure)
- [qi.toml Specification](#qitoml-specification)
- [Template System](#template-system)
- [Creating Custom Templates](#creating-custom-templates)

---

## Project Structure

A standard Qi project has the following structure:

```
my-project/
├── qi.toml           # Project configuration file
├── main.qi           # Entry point
├── src/              # Library code
│   └── lib.qi
├── examples/         # Sample code
│   └── example.qi
└── tests/            # Test code
    └── test.qi
```

### Role of Each File and Directory

#### `qi.toml`
TOML-format file describing project metadata and configuration. See [qi.toml Specification](#qitoml-specification) for details.

#### `main.qi`
Project entry point. Executed with `qi main.qi`.

#### `src/`
Directory for reusable library code. Will be loadable via the module system in the future.

#### `examples/`
Directory for usage examples and demo code. Used for documentation purposes and behavior verification.

#### `tests/`
Directory for test code. Will be executed with `qi test` command (planned for future implementation).

---

## qi.toml Specification

`qi.toml` is the project configuration file, written in TOML format.

### Basic Example

```toml
[project]
name = "my-project"
version = "0.1.0"
authors = ["Alice <alice@example.com>"]
description = "My awesome Qi project"
license = "MIT"
qi-version = "0.1.0"

[dependencies]
# For future expansion (currently unimplemented)

[features]
default = ["http-server", "format-json"]
```

---

### `[project]` Section

Defines project metadata.

#### `name` (Required)
Project name. Alphanumeric characters, hyphens, and underscores are allowed.

```toml
name = "my-project"
```

#### `version` (Required)
Project version. Semantic versioning is recommended.

```toml
version = "0.1.0"
```

#### `authors` (Optional)
List of project authors.

```toml
authors = ["Alice <alice@example.com>", "Bob <bob@example.com>"]
```

#### `description` (Optional)
Project description.

```toml
description = "A web API server written in Qi"
```

#### `license` (Optional)
License identifier (SPDX format recommended).

```toml
license = "MIT"
license = "Apache-2.0"
license = "GPL-3.0-or-later"
```

#### `qi-version` (Required)
Compatible Qi version.

```toml
qi-version = "0.1.0"
```

---

### `[dependencies]` Section

Defines dependencies (for future expansion, currently unimplemented).

```toml
[dependencies]
# In the future, describe dependencies on other Qi packages
# my-lib = "1.0.0"
```

---

### `[features]` Section

Defines feature flags used by the project.

#### `default`
List of features to enable by default.

```toml
[features]
default = []  # Basic features only

# Or
default = ["http-server", "format-json"]  # Enable HTTP + JSON features
```

#### Available Features

Below are examples of feature flags available in the Qi runtime:

| Feature | Description |
|---------|-------------|
| `http-server` | HTTP server functionality (`server/serve`, `server/json`, etc.) |
| `http-client` | HTTP client functionality (`http/get`, `http/post`, etc.) |
| `format-json` | JSON processing functionality (`json/parse`, `json/stringify`, etc.) |
| `format-yaml` | YAML processing functionality (`yaml/parse`, `yaml/stringify`, etc.) |
| `io-file` | File I/O functionality (`io/read-file`, `io/write-file`, etc.) |
| `io-glob` | File glob functionality (`io/glob`, etc.) |
| `db-sqlite` | SQLite database functionality (planned for future implementation) |
| `concurrency` | Concurrency functionality (`go`, `chan`, etc.) |
| `std-math` | Math functions (`math/rand`, `math/sqrt`, etc.) |
| `string-encoding` | String encoding (base64, URL encoding, etc.) |

**Note**: Feature flags are currently for documentation purposes only and do not affect runtime behavior (planned for use in future build system).

---

## Template System

The `qi new` command generates projects using templates.

### Built-in Templates

#### `basic` (Default)
Basic project structure.

**Structure:**
```
project/
├── qi.toml
├── main.qi
├── src/
│   └── lib.qi
├── examples/
│   └── example.qi
└── tests/
    └── test.qi
```

**Use Case**: Simple scripts or library development

---

#### `http-server`
JSON API server.

**Structure:**
```
project/
├── qi.toml          # features = ["http-server", "format-json"]
├── main.qi          # Server implementation
└── src/
```

**Use Case**: RESTful API servers, web backends

**Features:**
- Routing implementation (`/`, `/api/hello`, `/api/users`)
- JSON responses
- Request handler examples

---

### Template Search Order

Templates are searched in the following order:

1. `./.qi/templates/<name>/` - Project-local
2. `~/.qi/templates/<name>/` - User-global
3. `<qi-binary-dir>/std/templates/<name>/` - Installed version
4. `std/templates/<name>/` - Development version

The first template found is used.

---

## Creating Custom Templates

You can create your own templates.

### Template Structure

```
~/.qi/templates/my-template/
├── template.toml          # Template metadata
├── qi.toml.template       # Project configuration template
├── main.qi.template       # Main file template
├── src/
│   └── lib.qi.template
└── tests/
    └── test.qi.template
```

### `template.toml`

Defines template metadata.

```toml
[template]
name = "my-template"
description = "My custom template"
author = "Your Name"
version = "1.0.0"

[features]
required = ["http-server", "format-json"]
```

#### `[template]` Section

| Field | Required | Description |
|-------|----------|-------------|
| `name` | ✓ | Template name |
| `description` | ✓ | Template description |
| `author` | | Creator |
| `version` | | Template version |

#### `[features]` Section

| Field | Description |
|-------|-------------|
| `required` | List of feature flags required by this template |

---

### Template File Notation

Template files can use variable substitution and conditional branching.

#### Variable Substitution

Embed variables using `{{ variable }}` format.

**Available Variables:**
- `{{ project_name }}` - Project name
- `{{ version }}` - Version
- `{{ author }}` - Author name
- `{{ description }}` - Description
- `{{ license }}` - License

**Example:**
```toml
# qi.toml.template
[project]
name = "{{ project_name }}"
version = "{{ version }}"
description = "{{ description }}"
```

**After Generation:**
```toml
[project]
name = "my-project"
version = "0.1.0"
description = "My awesome project"
```

---

#### Conditional Branching

Write conditionals using `{{ #if variable }}...{{ /if }}` format.

**Example:**
```toml
# qi.toml.template
[project]
name = "{{ project_name }}"
version = "{{ version }}"
{{ #if author }}authors = ["{{ author }}"]{{ /if }}
{{ #if description }}description = "{{ description }}"{{ /if }}
{{ #if license }}license = "{{ license }}"{{ /if }}
qi-version = "0.1.0"
```

If a variable is empty, the entire line is removed.

**After Generation (when author and description are empty):**
```toml
[project]
name = "my-project"
version = "0.1.0"
license = "MIT"
qi-version = "0.1.0"
```

---

### File Naming Conventions

- Files with `.template` suffix have the suffix removed when output
- Example: `main.qi.template` → `main.qi`
- `template.toml` is not copied

---

### Template Placement

#### User-Global Templates
```bash
mkdir -p ~/.qi/templates/my-template
cp -r my-template/* ~/.qi/templates/my-template/
```

#### Project-Local Templates
```bash
mkdir -p .qi/templates/my-template
cp -r my-template/* .qi/templates/my-template/
```

---

### Using Templates

```bash
# List available templates
qi template list

# Use custom template
qi new my-project --template my-template

# View template information
qi template info my-template
```

---

## Example: Custom CLI Tool Template

Example of creating a template for command-line tools:

### `~/.qi/templates/cli/template.toml`
```toml
[template]
name = "cli"
description = "Command-line tool template"
author = "Qi Team"
version = "1.0.0"

[features]
required = ["io-file", "string-encoding"]
```

### `~/.qi/templates/cli/qi.toml.template`
```toml
[project]
name = "{{ project_name }}"
version = "{{ version }}"
{{ #if author }}authors = ["{{ author }}"]{{ /if }}
{{ #if description }}description = "{{ description }}"{{ /if }}
{{ #if license }}license = "{{ license }}"{{ /if }}
qi-version = "0.1.0"

[dependencies]

[features]
default = ["io-file", "string-encoding"]
```

### `~/.qi/templates/cli/main.qi.template`
```qi
;; {{ project_name }} - Command-line tool
;;
;; Usage: qi main.qi [OPTIONS] [ARGS]

(println "=== {{ project_name }} v{{ version }} ===")

;; Parse command-line arguments
(defn parse-args [args]
  {:command (first args)
   :options (rest args)})

;; Main logic
(defn main [args]
  (let [parsed (parse-args args)]
    (match (get parsed :command)
      "help" -> (println "Help message")
      nil -> (println "Please specify a command")
      cmd -> (println f"Unknown command: {cmd}"))))

;; Entry point
(main (get (env) :args []))
```

### Usage Example
```bash
# Create CLI tool
qi new mytool --template cli
cd mytool
qi main.qi help
```

---

## Related Documentation

- [CLI Reference](cli.md) - How to use the `qi` command
- [Tutorial](tutorial/01-getting-started.md) - Practical usage
- [Language Specification](spec/README.md) - Qi language grammar and features
