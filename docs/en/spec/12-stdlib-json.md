# Standard Library - JSON/YAML

**JSON and YAML Processing**

---

## JSON Processing (json/)

### Basic Operations

```qi
;; json/parse - Parse JSON string
(json/parse "{\"name\":\"Alice\",\"age\":30}")
;; => {"name" "Alice" "age" 30}

;; json/stringify - Stringify value to JSON (compact)
(json/stringify {"name" "Bob" "age" 25})
;; => "{\"name\":\"Bob\",\"age\":25}"

;; json/pretty - Stringify value to formatted JSON
(json/pretty {"name" "Bob" "age" 25})
;; => "{\n  \"name\": \"Bob\",\n  \"age\": 25\n}"
```

### Safe Parsing with Result Type

All JSON functions return values as-is on success and return `{:error message}` on failure.

```qi
;; Successful parse
(match (json/parse "{\"valid\": true}")
  {:error e} -> (log e)
  data -> data)
;; => {"valid" true}

;; Error handling
(match (json/parse "{invalid json}")
  {:error e} -> (println "Parse error:" e)
  data -> data)
;; => "Parse error: ..."
```

### Pipeline Usage

```qi
;; API response parse → transform → save (HTTP throws exceptions, so catch with try)
(match (try
         ("https://api.example.com/users/123"
          |> http/get
          |>? json/parse
          |>? (fn [data] (assoc data "processed" true))
          |>? json/pretty
          |>? (fn [json] (io/write-file "output.json" json))))
  {:error e} -> (log/error "Failed:" e)
  result -> result)
```

---

## YAML Processing (yaml/)

**Pure Rust Implementation - Using serde_yaml**

### Basic Operations

```qi
;; yaml/parse - Parse YAML string
(yaml/parse "name: Alice\nage: 30\ntags:\n  - dev\n  - ops")
;; => {"name" "Alice" "age" 30 "tags" ["dev" "ops"]}

;; yaml/stringify - Stringify value to YAML
(yaml/stringify {"name" "Bob" "age" 25 "tags" ["backend" "devops"]})
;; => "name: Bob\nage: 25\ntags:\n- backend\n- devops\n"

;; yaml/pretty - Stringify value to formatted YAML (same as yaml/stringify)
(yaml/pretty {"server" {"host" "localhost" "port" 8080}})
;; => "server:\n  host: localhost\n  port: 8080\n"
```

### Configuration File Processing

```qi
;; Parse config file and get port (I/O throws exceptions, so catch with try)
(match (try
         ("config.yaml"
          |> io/read-file
          |>? yaml/parse
          |>? (fn [conf] (get-in conf ["server" "port"]))))
  {:error e} -> (log/error "Failed:" e)
  port -> port)
;; => 8080

;; Data transformation pipeline (JSON → YAML)
(match (try
         ("data.json"
          |> io/read-file
          |>? json/parse
          |>? yaml/stringify
          |>? (fn [yaml] (io/write-file "data.yaml" yaml))))
  {:error e} -> (log/error "Failed:" e)
  result -> result)
```

### YAML Features

- Optimal for configuration files (more readable than JSON/TOML)
- Automatic indentation formatting
- JSON compatible (YAML is a superset of JSON)
- Error handling: Returns value as-is on success, `{:error "..."}` on failure

---

## Practical Examples

### Fetching and Saving API Data

```qi
(defn fetch-and-save [url output-file]
  (match (try
           (url
            |> http/get
            |>? json/parse
            |>? json/pretty
            |>? (fn [json-str] (io/write-file output-file json-str))))
    {:error e} -> (log/error "Failed:" e)
    result -> result))

(fetch-and-save "https://api.github.com/users/octocat" "user.json")
```

### Loading Configuration Files

```qi
(defn load-config [path]
  (match (try
           (path
            |> io/read-file
            |>? yaml/parse
            |>? (fn [config]
                  ;; Validation
                  (if (get config "version")
                    config
                    {:error "Missing version field"}))))
    {:error e} -> {:error e}
    config -> config))

(match (load-config "config.yaml")
  {:error e} -> (println "Error:" e)
  config -> (println "Config loaded:" config))
```

### Data Transformation

```qi
;; JSON → YAML conversion
(defn json-to-yaml [input-file output-file]
  (match (try
           (input-file
            |> io/read-file
            |>? json/parse
            |>? yaml/stringify
            |>? (fn [yaml-str] (io/write-file output-file yaml-str))))
    {:error e} -> (log/error "Failed:" e)
    result -> result))

(json-to-yaml "data.json" "data.yaml")

;; YAML → JSON conversion
(defn yaml-to-json [input-file output-file]
  (match (try
           (input-file
            |> io/read-file
            |>? yaml/parse
            |>? json/stringify
            |>? (fn [json-str] (io/write-file output-file json-str))))
    {:error e} -> (log/error "Failed:" e)
    result -> result))

(yaml-to-json "config.yaml" "config.json")
```

### Batch Processing

```qi
;; Parse multiple JSON files in parallel
(def files ["data1.json" "data2.json" "data3.json"])

(files
 ||> (fn [f] (try (io/read-file f)))
 ||> (fn [content]
       (match content
         {:error e} -> {:error e}
         c -> (json/parse c)))
 |> (filter (fn [result] (not (error? result)))))
```

### Error Handling Patterns

```qi
;; Error handling in pipelines
(defn process-json [json-str]
  (match (json/parse json-str)
    {:error e} -> (do
                    (log/error "Parse failed:" e)
                    {:error e})
    data -> (do
              (println "Parsed successfully")
              (assoc data "timestamp" (now)))))

;; Try multiple parsing formats
(defn try-parse-formats [input-str]
  (match (json/parse input-str)
    {:error _} -> (yaml/parse input-str)
    data -> data))
```

---

## Type Mapping

### Qi → JSON/YAML

| Qi Type | JSON | YAML |
|---------|------|------|
| `nil` | `null` | `null` |
| `true/false` | `true/false` | `true/false` |
| Integer/Float | Number | Number |
| String | String | String |
| Vector/List | Array | List |
| Map | Object | Map |
| Keyword | String | String |

### JSON/YAML → Qi

| JSON/YAML | Qi Type |
|-----------|---------|
| `null` | `nil` |
| `true/false` | `true/false` |
| Number | Integer or Float |
| String | String |
| Array | Vector |
| Object/Map | Map |
