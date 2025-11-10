# Standard Library - Environment Variables (env/)

Functions for getting, setting environment variables and loading .env files.

## Overview

The `env/` module provides the following features:

- **Environment Variable Retrieval** - Read system environment variables
- **Environment Variable Setting** - Set environment variables within the process
- **All Environment Variables** - Get all environment variables as a map
- **.env File Loading** - Bulk set environment variables from dotenv format files

---

## Getting Environment Variables

### env/get

Get an environment variable. Optionally return a default value if the variable doesn't exist.

```qi
(env/get key)
(env/get key default-value)
```

**Arguments:**
- `key` (string) - Environment variable name
- `default-value` (any, optional) - Return value if variable doesn't exist

**Return Value:**
- string - Environment variable value
- default-value - If variable doesn't exist and default value is specified
- nil - If variable doesn't exist and no default value is specified

**Examples:**

```qi
;; Get environment variable
(env/get "PATH")
;; => "/usr/local/bin:/usr/bin:/bin"

;; Non-existent variable (returns nil)
(env/get "NONEXISTENT_VAR")
;; => nil

;; Specify default value
(env/get "MISSING_VAR" "default-value")
;; => "default-value"

;; Existing variable (default value is ignored)
(env/get "PATH" "fallback")
;; => "/usr/local/bin:/usr/bin:/bin"
```

---

## Setting Environment Variables

### env/set

Set an environment variable. Affects the current process and its child processes.

```qi
(env/set key value)
```

**Arguments:**
- `key` (string) - Environment variable name
- `value` (string | number | boolean) - Value to set (converted to string)

**Return Value:** nil

**Examples:**

```qi
;; Set string value
(env/set "MY_VAR" "my-value")
(env/get "MY_VAR")
;; => "my-value"

;; Set number (converted to string)
(env/set "PORT" 8080)
(env/get "PORT")
;; => "8080"

;; Set boolean
(env/set "DEBUG" true)
(env/get "DEBUG")
;; => "true"

(env/set "ENABLED" false)
(env/get "ENABLED")
;; => "false"
```

---

## Getting All Environment Variables

### env/all

Get all environment variables as a map.

```qi
(env/all)
```

**Arguments:** none

**Return Value:** map - Keys are variable names, values are variable values

**Examples:**

```qi
;; Get all environment variables
(def env-vars (env/all))

;; Get specific variable
(get env-vars "HOME")
;; => "/home/user"

;; Count environment variables
(count (env/all))
;; => 42

;; Extract variables starting with PATH
(env/all
 |> keys
 |> (filter (fn [k] (str/starts-with? k "PATH"))))
;; => ["PATH" "PATHEXT"]
```

---

## Loading .env Files

### env/load-dotenv

Load a .env file and set environment variables. Supports dotenv format (`KEY=VALUE`).

```qi
(env/load-dotenv)
(env/load-dotenv path)
```

**Arguments:**
- `path` (string, optional) - Path to .env file (default: ".env")

**Return Value:** integer - Number of environment variables loaded

**Examples:**

```qi
;; Load default .env
(env/load-dotenv)
;; => 5

;; Specify custom path
(env/load-dotenv ".env.local")
;; => 3

;; Development environment settings
(env/load-dotenv ".env.development")
;; => 8
```

### .env File Format

`.env` files support the following format:

```bash
# Comment lines (start with #)
KEY=VALUE

# Quoted values (double or single quotes)
DATABASE_URL="postgresql://localhost/mydb"
API_KEY='secret-key-123'

# Unquoted
PORT=8080
DEBUG=true

# Whitespace is automatically trimmed
  TRIM_ME  =  value with spaces

# Empty lines are ignored

NODE_ENV=production
```

**Supported Features:**
- ✅ Basic `KEY=VALUE` format
- ✅ Comment lines (starting with `#`)
- ✅ Quoted values (`"..."` or `'...'`)
- ✅ Automatic trimming of whitespace around keys and values
- ✅ Empty lines ignored

**Limitations:**
- ❌ Variable expansion (`${VAR}` format) not supported
- ❌ Multiline values not supported
- ❌ Escape sequences (`\n`, `\t`, etc.) not supported

---

## Usage Examples

### Loading Configuration

```qi
;; Load .env at application startup
(defn load-config []
  (let [count (env/load-dotenv)]
    (println f"Loaded {count} environment variables")
    {:port (str/parse-int (env/get "PORT" "8080"))
     :host (env/get "HOST" "localhost")
     :debug (= (env/get "DEBUG") "true")}))

(def config (load-config))
;; Loaded 5 environment variables
;; => {:port 8080 :host "localhost" :debug true}
```

### Environment-specific Configuration Files

```qi
;; Load different .env based on NODE_ENV
(defn load-env-for-node-env []
  (let [node-env (env/get "NODE_ENV" "development")
        env-file (str ".env." node-env)]
    (println f"Loading {env-file}...")
    (env/load-dotenv env-file)))

(load-env-for-node-env)
```

### Database Connection String

```qi
;; Build database configuration from environment variables
(defn get-db-config []
  (let [db-url (env/get "DATABASE_URL")]
    (if db-url
        ;; Use DATABASE_URL if set
        {:url db-url}
        ;; Otherwise build from individual variables
        {:host (env/get "DB_HOST" "localhost")
         :port (str/parse-int (env/get "DB_PORT" "5432"))
         :database (env/get "DB_NAME" "myapp")
         :user (env/get "DB_USER" "postgres")
         :password (env/get "DB_PASSWORD" "")})))

(def db-config (get-db-config))
```

### Listing Environment Variables

```qi
;; Display all environment variables formatted
(defn print-env-vars []
  (env/all
   |> (map (fn [[k v]] f"{k} = {v}"))
   |> sort
   |> (each println)))

(print-env-vars)
;; HOME = /home/user
;; PATH = /usr/local/bin:/usr/bin
;; SHELL = /bin/zsh
;; ...
```

### Security: Masking Sensitive Information

```qi
;; Mask environment variables containing sensitive information
(def sensitive-keys ["PASSWORD" "SECRET" "KEY" "TOKEN"])

(defn mask-value [key value]
  (if (any? (fn [s] (str/contains? (str/upper key) s)) sensitive-keys)
      "********"
      value))

(defn print-env-safe []
  (env/all
   |> (map (fn [[k v]] [k (mask-value k v)]))
   |> (each (fn [[k v]] (println f"{k} = {v}")))))

(print-env-safe)
;; API_KEY = ********
;; DB_PASSWORD = ********
;; USER = alice
```

### Configuration Validation

```qi
;; Check if required environment variables are set
(defn validate-env [required-vars]
  (let [missing (filter (fn [var] (nil? (env/get var))) required-vars)]
    (if (empty? missing)
        :ok
        {:error "Missing environment variables" :missing missing})))

(validate-env ["PORT" "DATABASE_URL" "API_KEY"])
;; => {:error "Missing environment variables" :missing ["API_KEY"]}
```

### .env File Backup

```qi
;; Save current environment variables to file in .env format
(defn save-env-to-file [path keys]
  (let [lines (map (fn [k]
                     (let [v (env/get k)]
                       (if v
                           f"{k}={v}"
                           nil)))
                   keys)
        content (join "\n" (filter some? lines))]
    (io/write path content)))

;; Backup only specific keys
(save-env-to-file ".env.backup" ["PORT" "HOST" "DEBUG"])
```

---

## Error Handling

### File Not Found

```qi
(try
  (env/load-dotenv "nonexistent.env")
  :ok
  (fn [err]
    (println "Error:" err)
    :error))
;; Error: Failed to read .env file 'nonexistent.env': No such file or directory
;; => :error
```

### Invalid File Format

```qi
;; If .env file contains lines not in KEY=VALUE format
;; Error message includes line number and content

;; invalid.env contents:
;; KEY1=VALUE1
;; INVALID LINE WITHOUT EQUALS
;; KEY2=VALUE2

(try
  (env/load-dotenv "invalid.env")
  :ok
  (fn [err]
    (println err)))
;; Invalid format in .env file at line 2: INVALID LINE WITHOUT EQUALS
```

---

## Performance and Best Practices

### Load Once at Startup

```qi
;; ❌ Bad: Load on every request
(defn handle-request [req]
  (env/load-dotenv)  ;; File I/O every time
  (let [api-key (env/get "API_KEY")]
    ...))

;; ✅ Good: Load once at startup
(env/load-dotenv)

(defn handle-request [req]
  (let [api-key (env/get "API_KEY")]  ;; Get from memory
    ...))
```

### Cache in Configuration Object

```qi
;; ✅ Load and cache configuration at startup
(env/load-dotenv)

(def config
  {:port (str/parse-int (env/get "PORT" "8080"))
   :host (env/get "HOST" "localhost")
   :db-url (env/get "DATABASE_URL")
   :api-key (env/get "API_KEY")})

;; Use config afterwards
(defn start-server []
  (http/serve (:port config) (:host config) handler))
```

### Setting Default Values

```qi
;; ❌ Bad: Repeated nil checks
(let [port (env/get "PORT")]
  (if (nil? port)
      8080
      (str/parse-int port)))

;; ✅ Good: Specify default value in env/get
(str/parse-int (env/get "PORT" "8080"))
```

---

## Related Topics

- [File I/O](13-stdlib-io.md) - File reading and writing
- [String Manipulation](10-stdlib-string.md) - Parsing environment variables
- [Error Handling](08-error-handling.md) - Error handling with try/catch
- [Module System](09-modules.md) - Configuration management
