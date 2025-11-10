# Standard Library - String Manipulation (str/)

**60+ String Manipulation Functions**

All functions belong to the `str/` module.

---

## Search

```qi
;; str/contains? - Check if contains substring
(str/contains? "hello world" "world")     ;; => true

;; str/starts-with? - Check if starts with prefix
(str/starts-with? "hello" "he")           ;; => true

;; str/ends-with? - Check if ends with suffix
(str/ends-with? "hello" "lo")             ;; => true

;; str/index-of - Find first occurrence (nil if not found)
(str/index-of "hello world" "world")      ;; => 6
(str/index-of "hello" "xyz")              ;; => nil

;; str/last-index-of - Find last occurrence (nil if not found)
(str/last-index-of "hello hello" "hello") ;; => 6
(str/last-index-of "hello" "xyz")         ;; => nil
```

---

## Basic Transformations

```qi
;; str/upper - Convert to uppercase
(str/upper "hello")                       ;; => "HELLO"

;; str/lower - Convert to lowercase
(str/lower "HELLO")                       ;; => "hello"

;; str/capitalize - Capitalize first letter only
(str/capitalize "hello")                  ;; => "Hello"

;; str/title - Title case
(str/title "hello world")                 ;; => "Hello World"

;; str/trim - Remove leading/trailing whitespace
(str/trim "  hello  ")                    ;; => "hello"

;; str/trim-left - Remove left whitespace
(str/trim-left "  hello  ")               ;; => "hello  "

;; str/trim-right - Remove right whitespace
(str/trim-right "  hello  ")              ;; => "  hello"

;; str/repeat - Repeat string
(str/repeat "-" 80)                       ;; => "----..." (80 chars)
(str/repeat "ab" 3)                       ;; => "ababab"

;; str/reverse - Reverse string
(str/reverse "hello")                     ;; => "olleh"
```

---

## Case Conversion

```qi
;; str/snake - Convert to snake_case
(str/snake "userName")                    ;; => "user_name"

;; str/camel - Convert to camelCase
(str/camel "user_name")                   ;; => "userName"

;; str/kebab - Convert to kebab-case
(str/kebab "userName")                    ;; => "user-name"

;; str/pascal - Convert to PascalCase
(str/pascal "user_name")                  ;; => "UserName"

;; str/split-camel - Split camelCase
(str/split-camel "userName")              ;; => ["user" "Name"]
```

---

## Split/Join

```qi
;; str/split - Split string
(str/split "a,b,c" ",")                   ;; => ["a" "b" "c"]

;; str/lines - Split into lines
(str/lines "hello\nworld")                ;; => ["hello" "world"]

;; str/words - Split into words
(str/words "hello world")                 ;; => ["hello" "world"]

;; str/chars - Split into characters
(str/chars "hello")                       ;; => ["h" "e" "l" "l" "o"]
```

---

## Replacement

```qi
;; str/replace - Replace all occurrences
(str/replace "hello world" "world" "qi")  ;; => "hello qi"

;; str/replace-first - Replace first occurrence only
(str/replace-first "aa bb aa" "aa" "cc")  ;; => "cc bb aa"

;; str/splice - Replace range
(str/splice "hello world" 6 11 "universe") ;; => "hello universe"
```

---

## Substring

```qi
;; str/slice - Get range
(str/slice "hello world" 0 5)             ;; => "hello"

;; str/take-str - Take first n characters (pipeline optimized)
(str/take-str 3 "hello")                  ;; => "hel"
("hello" |> (str/take-str 3))             ;; => "hel"

;; str/drop-str - Drop first n characters (pipeline optimized)
(str/drop-str 2 "hello")                  ;; => "llo"
("hello" |> (str/drop-str 2))             ;; => "llo"

;; str/sub-before - Get substring before delimiter
(str/sub-before "user@example.com" "@")   ;; => "user"

;; str/sub-after - Get substring after delimiter
(str/sub-after "user@example.com" "@")    ;; => "example.com"
```

---

## Formatting/Alignment

```qi
;; str/pad-left - Left pad
(str/pad-left "Total" 20)                 ;; => "               Total"

;; str/pad-right - Right pad
(str/pad-right "Name" 20)                 ;; => "Name               "

;; str/pad - Center align
(str/pad "hi" 10)                         ;; => "    hi    "
(str/pad "hi" 10 "*")                     ;; => "****hi****"

;; str/truncate - Truncate to length
(str/truncate "hello world" 8)            ;; => "hello..."

;; str/trunc-words - Truncate to word count
(str/trunc-words "hello world from qi" 2) ;; => "hello world..."

;; str/indent - Add indentation
(str/indent "hello\nworld" 2)             ;; => "  hello\n  world"

;; str/wrap - Wrap text at width
(str/wrap "hello world from qi" 10)       ;; => "hello\nworld from\nqi"
```

---

## Normalization

```qi
;; str/squish - Collapse consecutive whitespace to one (trim included)
(str/squish "  hello   world  \n")        ;; => "hello world"

;; str/expand-tabs - Convert tabs to spaces
(str/expand-tabs "\thello\tworld")        ;; => "    hello    world"
```

---

## Validation (Predicates)

```qi
;; str/digit? - Check if all digits
(str/digit? "12345")                      ;; => true

;; str/alpha? - Check if all alphabetic
(str/alpha? "hello")                      ;; => true

;; str/alnum? - Check if all alphanumeric
(str/alnum? "hello123")                   ;; => true

;; str/space? - Check if all whitespace
(str/space? "  \n\t")                     ;; => true

;; str/numeric? - Check if numeric representation
(str/numeric? "123.45")                   ;; => true

;; str/integer? - Check if integer representation
(str/integer? "123")                      ;; => true

;; str/blank? - Check if blank or empty
(str/blank? "  \n")                       ;; => true

;; str/ascii? - Check if all ASCII
(str/ascii? "hello")                      ;; => true

;; str/lower? - Check if all lowercase
(str/lower? "hello")                      ;; => true

;; str/upper? - Check if all uppercase
(str/upper? "HELLO")                      ;; => true
```

---

## URL/Web

```qi
;; str/slugify - Convert to URL/filename format
(str/slugify "Hello World! 2024")         ;; => "hello-world-2024"
(str/slugify "Café résumé")               ;; => "cafe-resume"

;; str/url-encode - URL encode
(str/url-encode "hello world")            ;; => "hello%20world"

;; str/url-decode - URL decode
(str/url-decode "hello%20world")          ;; => "hello world"

;; str/html-encode - HTML encode
(str/html-encode "<div>test</div>")       ;; => "&lt;div&gt;test&lt;/div&gt;"

;; str/html-decode - HTML decode
(str/html-decode "&lt;div&gt;test&lt;/div&gt;") ;; => "<div>test</div>"
```

---

## Encoding

```qi
;; str/to-base64 - Base64 encode
(str/to-base64 "hello")                   ;; => "aGVsbG8="

;; str/from-base64 - Base64 decode
(str/from-base64 "aGVsbG8=")              ;; => "hello"
```

---

## Parsing

```qi
;; str/parse-int - Parse to integer
(str/parse-int "123")                     ;; => 123

;; str/parse-float - Parse to float
(str/parse-float "3.14")                  ;; => 3.14
```

---

## Unicode

```qi
;; str/chars-count - Unicode character count
(str/chars-count "👨‍👩‍👧‍👦")                ;; => 1

;; str/bytes-count - Byte count
(str/bytes-count "👨‍👩‍👧‍👦")                ;; => 25
```

---

## Generation

```qi
;; str/hash - Generate hash value
(str/hash "hello")                        ;; => "2cf24dba5fb0a30e..."
(str/hash "hello" :sha256)                ;; SHA-256 (default)

;; str/uuid - Generate UUID
(str/uuid)                                ;; => "550e8400-e29b-41d4-a716-446655440000"
```

---

## NLP

```qi
;; str/word-count - Count words
(str/word-count "hello world")            ;; => 2
```

---

## Formatting

```qi
;; str/format - Placeholder replacement
(str/format "Hello, {}!" "World")         ;; => "Hello, World!"
(str/format "{} + {} = {}" 1 2 3)         ;; => "1 + 2 = 3"

;; str/format-decimal - Format decimal places
(str/format-decimal 3.14159 2)            ;; => "3.14"
(3.14159 |> (str/format-decimal _ 2))     ;; Use in pipeline

;; str/format-comma - Format with comma separators
(str/format-comma 1234567)                ;; => "1,234,567"
(1234567 |> str/format-comma)             ;; Use in pipeline

;; str/format-percent - Format as percentage
(str/format-percent 0.1234)               ;; => "12%"
(str/format-percent 0.1234 2)             ;; => "12.34%"
(0.856 |> (str/format-percent _ 1))       ;; => "85.6%"
```

---

## Practical Examples

### URL Processing

```qi
;; Generate URL parameters
(def params {:user "alice" :page 1 :sort "name"})

(params
 |> keys
 |> (map (fn [k] (str (name k) "=" (str/url-encode (get params k)))))
 |> (join "&"))
;; => "user=alice&page=1&sort=name"
```

### Text Formatting

```qi
;; Markdown code formatting
(defn format-code [code lang]
  f"```{lang}\n{(str/trim code)}\n```")

(format-code "  def x = 42  " "qi")
;; => "```qi\ndef x = 42\n```"
```

### Validation

```qi
;; Simple email validation
(defn valid-email? [email]
  (and
    (str/contains? email "@")
    (str/contains? (str/sub-after email "@") ".")
    (not (str/blank? (str/sub-before email "@")))))

(valid-email? "user@example.com")  ;; => true
(valid-email? "invalid")           ;; => false
```

### Data Transformation Pipeline

```qi
;; Normalize CSV headers
(def headers ["User Name" "E-Mail" "Created At"])

(headers
 |> (map str/lower)
 |> (map str/squish)
 |> (map (partial str/replace _ " " "_")))
;; => ["user_name" "e-mail" "created_at"]
```
