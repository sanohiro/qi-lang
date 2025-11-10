# Standard Library: Validation

**Schema-Based Data Validation**

---

## Overview

Qi's validation functionality validates data based on schemas (maps). This allows declarative validation of data such as API requests, configuration files, and user input.

### Key Features

- **Schema-Based**: Define schemas with maps
- **Railway Oriented Programming**: Return errors as values (`{:ok value}` or `{:error ...}`)
- **Type Safety**: Support for multiple types (string, integer, number, boolean, map, vector, list, keyword, symbol, nil, any)
- **Nested Data Support**: Can validate nested maps and collections
- **Detailed Error Messages**: Include error code, message, and field name

---

## Basic Usage

### Type Checking

```qi
;; String type check
(def schema {:type "string"})
(validate schema "hello")  ;; => {:ok "hello"}
(validate schema 123)      ;; => {:error {:code "type-mismatch" :message "must be string"}}

;; Integer type check
(def schema {:type "integer"})
(validate schema 42)    ;; => {:ok 42}
(validate schema 3.14)  ;; => {:error {:code "type-mismatch" :message "must be integer"}}
```

### Required Field Check

```qi
(def schema {:type "string" :required true})
(validate schema "test")  ;; => {:ok "test"}
(validate schema nil)     ;; => {:error {:code "required" :message "required field"}}

;; Optional field (default)
(def schema {:type "string"})
(validate schema nil)  ;; => {:ok nil}  ;; Success (optional)
```

---

## String Validation

### Length Check

```qi
(def username-schema
  {:type "string"
   :min-length 3
   :max-length 20})

(validate username-schema "user123")     ;; => {:ok "user123"}
(validate username-schema "ab")          ;; => {:error {:code "min-length" :message "must be at least 3 characters"}}
(validate username-schema "verylongusernamethatexceedslimit")
  ;; => {:error {:code "max-length" :message "must be at most 20 characters"}}
```

### Pattern Matching (Regex)

```qi
;; Email validation
(def email-schema
  {:type "string"
   :pattern "^[^@]+@[^@]+\\.[^@]+$"})

(validate email-schema "user@example.com")  ;; => {:ok "user@example.com"}
(validate email-schema "invalid-email")     ;; => {:error {:code "pattern" :message "does not match pattern: ..."}}

;; Lowercase letters only
(def lowercase-schema
  {:type "string"
   :pattern "^[a-z]+$"})

(validate lowercase-schema "hello")    ;; => {:ok "hello"}
(validate lowercase-schema "Hello123") ;; => {:error {:code "pattern" ...}}
```

---

## Numeric Validation

### Range Check

```qi
(def age-schema
  {:type "integer"
   :min 0
   :max 150})

(validate age-schema 25)   ;; => {:ok 25}
(validate age-schema -5)   ;; => {:error {:code "min-value" :message "must be at least 0"}}
(validate age-schema 200)  ;; => {:error {:code "max-value" :message "must be at most 150"}}
```

### Positive Number Check

```qi
(def price-schema
  {:type "number"
   :positive true})

(validate price-schema 100.0)  ;; => {:ok 100.0}
(validate price-schema 0)      ;; => {:error {:code "positive" :message "must be positive"}}
(validate price-schema -10.5)  ;; => {:error {:code "positive" ...}}
```

---

## Collection Validation

### Item Count Check

```qi
(def tags-schema
  {:type "vector"
   :min-items 1
   :max-items 5})

(validate tags-schema ["tag1" "tag2" "tag3"])  ;; => {:ok ["tag1" "tag2" "tag3"]}
(validate tags-schema [])                      ;; => {:error {:code "min-items" :message "must have at least 1 items"}}
(validate tags-schema ["a" "b" "c" "d" "e" "f"])
  ;; => {:error {:code "max-items" :message "must have at most 5 items"}}
```

---

## Nested Map Validation

You can define schemas for each field in a map using `:fields`.

```qi
(def user-schema
  {:type "map"
   :fields {:name {:type "string" :required true :min-length 1}
            :age {:type "integer" :min 0 :max 150}
            :email {:type "string" :pattern "^[^@]+@[^@]+\\.[^@]+$"}}})

;; Valid data
(validate user-schema
  {:name "Taro" :age 25 :email "taro@example.com"})
;; => {:ok {:name "Taro" :age 25 :email "taro@example.com"}}

;; Empty name (min-length error)
(validate user-schema {:name "" :age 25})
;; => {:error {:field ":name" :code "min-length" :message "must be at least 1 characters"}}

;; Missing name (required error)
(validate user-schema {:age 25})
;; => {:error {:field ":name" :code "required" :message "required field"}}

;; Age out of range
(validate user-schema {:name "Taro" :age 200})
;; => {:error {:field ":age" :code "max-value" :message "must be at most 150"}}

;; Optional field (email) can be omitted
(validate user-schema {:name "Taro"})
;; => {:ok {:name "Taro"}}  ;; age, email are nil (optional)
```

---

## Complex Schema Examples

### User Registration Form

```qi
(def signup-schema
  {:type "map"
   :fields {:username {:type "string" :required true :min-length 3 :max-length 20}
            :password {:type "string" :required true :min-length 8}
            :email {:type "string" :required true :pattern "^[^@]+@[^@]+\\.[^@]+$"}
            :age {:type "integer" :min 13 :max 150}
            :bio {:type "string" :max-length 500}}})

(validate signup-schema
  {:username "newuser"
   :password "securepass123"
   :email "user@example.com"
   :age 25})
;; => {:ok {...}}

(validate signup-schema
  {:username "ab"        ;; Too short
   :password "short"     ;; Too short
   :email "invalid"})    ;; Pattern mismatch
;; => {:error {:field ":username" :code "min-length" ...}}  ;; Returns first error
```

---

## Schema Options Reference

### Common Options

| Option | Type | Description |
|--------|------|-------------|
| `:type` | string | Data type: `"string"`, `"integer"`, `"number"`, `"boolean"`, `"map"`, `"vector"`, `"list"`, `"keyword"`, `"symbol"`, `"nil"`, `"any"` |
| `:required` | bool | Whether field is required (default: `false`) |

### String-Specific

| Option | Type | Description |
|--------|------|-------------|
| `:min-length` | integer | Minimum character count |
| `:max-length` | integer | Maximum character count |
| `:pattern` | string | Regular expression pattern |

### Numeric-Specific

| Option | Type | Description |
|--------|------|-------------|
| `:min` | integer/float | Minimum value |
| `:max` | integer/float | Maximum value |
| `:positive` | bool | Must be positive (greater than 0) |
| `:integer` | bool | Must be integer (for float type) |

### Collection-Specific

| Option | Type | Description |
|--------|------|-------------|
| `:min-items` | integer | Minimum item count |
| `:max-items` | integer | Maximum item count |

### Map-Specific

| Option | Type | Description |
|--------|------|-------------|
| `:fields` | map | Schema for nested fields (map) |

---

## Error Code Reference

| Code | Description |
|------|-------------|
| `type-mismatch` | Type does not match |
| `required` | Required field is missing |
| `min-length` | String is too short |
| `max-length` | String is too long |
| `pattern` | Does not match pattern |
| `min-value` | Number is too small |
| `max-value` | Number is too large |
| `positive` | Not a positive number |
| `min-items` | Too few items |
| `max-items` | Too many items |

---

## Usage Example: Validation in HTTP Server

```qi
(def user-create-schema
  {:type "map"
   :fields {:name {:type "string" :required true :min-length 1}
            :email {:type "string" :required true :pattern "^[^@]+@[^@]+\\.[^@]+$"}}})

(server/start 8080
  (fn [req]
    (def result (validate user-create-schema (get req :body)))
    (match result
      {:ok data}     (server/json {:status "ok" :user data})
      {:error error} (server/json {:status "error" :error error} :status 400))))
```

---

## Notes

- **Optional Fields**: If `:required` is `false` or unspecified, validation succeeds even if data is `nil`
- **:type "any"**: Accepts any type (no type checking)
- **Nested Maps**: Field where error occurred is included in `:field` (e.g., `":name"`, `":address:city"`)
- **Regular Expressions**: Follow ripgrep syntax

---

## Related Documentation

- [Error Handling](08-error-handling.md) - Railway Oriented Programming
- [Data Structures](06-data-structures.md) - Maps and Keywords
- [HTTP Server](11-stdlib-http.md) - Using Validation in Servers
