# Standard Library - Time Operations (time/)

**24+ time manipulation functions**

All functions belong to the `time/` module.

---

## Current Time Retrieval

### ISO 8601 Format

```qi
;; time/now-iso - Get current time in ISO 8601 format
(time/now-iso)                           ;; => "2025-11-10T15:30:45.123456789+00:00"

;; time/today - Get today's date in YYYY-MM-DD format
(time/today)                             ;; => "2025-11-10"
```

---

## Unix Timestamp Conversion

```qi
;; time/from-unix - Convert Unix timestamp (seconds) to ISO 8601 format
(time/from-unix 1699632000)              ;; => "2023-11-10T12:00:00+00:00"

;; time/to-unix - Convert ISO 8601 string to Unix timestamp (seconds)
(time/to-unix "2023-11-10T12:00:00+00:00") ;; => 1699632000
```

---

## Date/Time Formatting

### format - Format as arbitrary string

```qi
;; time/format - Format timestamp with specified format string
;; Arg 1: Unix timestamp (integer) or ISO 8601 string
;; Arg 2: Format string (strftime format)

(time/format 1699632000 "%Y-%m-%d")      ;; => "2023-11-10"
(time/format 1699632000 "%Y年%m月%d日")  ;; => "2023年11月10日"
(time/format 1699632000 "%H:%M:%S")      ;; => "12:00:00"

;; ISO 8601 strings are also accepted
(time/format "2023-11-10T12:00:00+00:00" "%Y/%m/%d %H:%M")
;; => "2023/11/10 12:00"

;; Pipeline usage
(time/now-iso
 |> (time/format _ "%Y-%m-%d %H:%M:%S")) ;; => "2025-11-10 15:30:45"
```

### Common strftime Format Specifiers

| Specifier | Meaning | Example |
|-----------|---------|---------|
| `%Y` | Year (4 digits) | 2025 |
| `%m` | Month (2 digits) | 11 |
| `%d` | Day (2 digits) | 10 |
| `%H` | Hour (24-hour, 2 digits) | 15 |
| `%I` | Hour (12-hour, 2 digits) | 03 |
| `%M` | Minute (2 digits) | 30 |
| `%S` | Second (2 digits) | 45 |
| `%p` | AM/PM | PM |
| `%A` | Weekday (full) | Sunday |
| `%a` | Weekday (abbr) | Sun |
| `%B` | Month name (full) | November |
| `%b` | Month name (abbr) | Nov |

---

## Date/Time Parsing

### parse - Parse with format specification

```qi
;; time/parse - Parse date string using format string
;; Arg 1: Date string
;; Arg 2: Format string (strftime format)
;; Returns: Unix timestamp (integer)

(time/parse "2023-11-10" "%Y-%m-%d")     ;; => 1699632000
(time/parse "2023年11月10日" "%Y年%m月%d日")
;; => 1699632000

(time/parse "10/11/2023 15:30" "%d/%m/%Y %H:%M")
;; => 1699632600
```

---

## Date/Time Arithmetic

### Addition

```qi
;; time/add-days - Add days to a date
(time/add-days 1699632000 7)             ;; => 1700236800 (7 days later)

;; time/add-hours - Add hours to a date
(time/add-hours 1699632000 24)           ;; => 1699718400 (24 hours later)

;; time/add-minutes - Add minutes to a date
(time/add-minutes 1699632000 30)         ;; => 1699633800 (30 minutes later)

;; ISO 8601 strings are also accepted
(time/add-days "2023-11-10T12:00:00+00:00" 1)
;; => 1699718400 (next day)

;; Pipeline usage
(time/now-iso
 |> time/to-unix
 |> (time/add-days _ 7)
 |> (time/format _ "%Y-%m-%d")) ;; => "2025-11-17" (7 days later)
```

### Subtraction

```qi
;; time/sub-days - Subtract days from a date
(time/sub-days 1699632000 7)             ;; => 1699027200 (7 days earlier)

;; time/sub-hours - Subtract hours from a date
(time/sub-hours 1699632000 24)           ;; => 1699545600 (24 hours earlier)

;; time/sub-minutes - Subtract minutes from a date
(time/sub-minutes 1699632000 30)         ;; => 1699630200 (30 minutes earlier)
```

### Difference Calculation

```qi
;; time/diff-days - Get the difference between two dates in days
(time/diff-days "2023-11-17T12:00:00+00:00" "2023-11-10T12:00:00+00:00")
;; => 7 (days)

;; time/diff-hours - Get the difference between two dates in hours
(time/diff-hours "2023-11-10T15:00:00+00:00" "2023-11-10T12:00:00+00:00")
;; => 3 (hours)

;; time/diff-minutes - Get the difference between two dates in minutes
(time/diff-minutes "2023-11-10T12:30:00+00:00" "2023-11-10T12:00:00+00:00")
;; => 30 (minutes)

;; Negative values are also returned
(time/diff-days "2023-11-10T12:00:00+00:00" "2023-11-17T12:00:00+00:00")
;; => -7 (date1 is before date2)
```

---

## Date/Time Comparison

```qi
;; time/before? - Check if date1 is before date2
(time/before? "2023-11-10T12:00:00+00:00" "2023-11-17T12:00:00+00:00")
;; => true

;; time/after? - Check if date1 is after date2
(time/after? "2023-11-17T12:00:00+00:00" "2023-11-10T12:00:00+00:00")
;; => true

;; time/between? - Check if date is between start and end (inclusive)
(time/between? "2023-11-15T12:00:00+00:00"
               "2023-11-10T12:00:00+00:00"
               "2023-11-20T12:00:00+00:00")
;; => true

;; Unix timestamps can also be used
(time/before? 1699632000 1700236800)     ;; => true
```

---

## Date/Time Component Extraction

```qi
;; time/year - Extract year from date
(time/year "2023-11-10T12:00:00+00:00")  ;; => 2023

;; time/month - Extract month from date (1-12)
(time/month "2023-11-10T12:00:00+00:00") ;; => 11

;; time/day - Extract day from date (1-31)
(time/day "2023-11-10T12:00:00+00:00")   ;; => 10

;; time/hour - Extract hour from date (0-23)
(time/hour "2023-11-10T12:00:00+00:00")  ;; => 12

;; time/minute - Extract minute from date (0-59)
(time/minute "2023-11-10T12:35:00+00:00") ;; => 35

;; time/second - Extract second from date (0-59)
(time/second "2023-11-10T12:00:45+00:00") ;; => 45

;; time/weekday - Extract weekday from date (0=Sunday, 1=Monday, ..., 6=Saturday)
(time/weekday "2023-11-10T12:00:00+00:00") ;; => 5 (Friday)

;; Unix timestamps can also be used
(time/year 1699632000)                   ;; => 2023
```

---

## Practical Examples

### Log Timestamps

```qi
;; Log with current timestamp
(defn log [level msg]
  (println (str/format "[{}] {} - {}"
                       (time/format (time/now-iso) "%Y-%m-%d %H:%M:%S")
                       level
                       msg)))

(log "INFO" "Application started")
;; => "[2025-11-10 15:30:45] INFO - Application started"
```

### Date/Time Calculation Pipeline

```qi
;; Calculate 9:00 AM, 7 days from now
(time/now-iso
 |> time/to-unix
 |> (time/add-days _ 7)
 |> (time/format _ "%Y-%m-%d 09:00:00")
 |> (time/parse _ "%Y-%m-%d %H:%M:%S"))
;; => 1700467200 (Unix timestamp for 9:00 AM, 7 days later)

;; Business day calculation (excluding weekends, 5 days ahead)
(defn add-business-days [date days]
  (let [target (time/add-days date days)]
    (let [wd (time/weekday target)]
      (cond
        (= wd 0) (time/add-days target 1)  ;; Sunday -> Monday
        (= wd 6) (time/add-days target 2)  ;; Saturday -> Monday
        :else target))))

(add-business-days (time/to-unix "2023-11-10T12:00:00+00:00") 5)
;; => 5 business days later, accounting for weekends
```

### Deadline Management

```qi
;; Check if task is overdue
(defn is-overdue? [due-date]
  (time/before? due-date (time/now-iso)))

(is-overdue? "2025-11-01T23:59:59+00:00") ;; => true

;; Days remaining until deadline
(defn days-until [due-date]
  (time/diff-days due-date (time/now-iso)))

(days-until "2025-12-25T00:00:00+00:00") ;; => 45 (45 days remaining)

;; Check if within deadline
(defn is-within-deadline? [start end]
  (time/between? (time/now-iso) start end))

(is-within-deadline? "2025-11-01T00:00:00+00:00" "2025-12-31T23:59:59+00:00")
;; => true
```

### Data Aggregation

```qi
;; Extract today's errors from log files
(defn today-errors [logs]
  (let [today (time/today)]
    (logs
     |> (filter (fn [log]
              (and
                (str/contains? log "ERROR")
                (str/starts-with? log today)))))))

(today-errors ["2025-11-10 10:30 ERROR: DB connection failed"
               "2025-11-09 14:20 ERROR: Timeout"
               "2025-11-10 15:45 ERROR: Invalid input"])
;; => ["2025-11-10 10:30 ERROR: DB connection failed"
;;     "2025-11-10 15:45 ERROR: Invalid input"]
```

### Time Range Filtering

```qi
;; Filter events by time period
(defn filter-by-period [events start end]
  (events
   |> (filter (fn [evt]
            (time/between? (get evt :timestamp) start end)))))

(def events [{:timestamp "2023-11-10T10:00:00+00:00" :type "login"}
             {:timestamp "2023-11-15T14:30:00+00:00" :type "purchase"}
             {:timestamp "2023-11-20T09:00:00+00:00" :type "logout"}])

(filter-by-period events
                  "2023-11-10T00:00:00+00:00"
                  "2023-11-16T00:00:00+00:00")
;; => [{:timestamp "2023-11-10T10:00:00+00:00" :type "login"}
;;     {:timestamp "2023-11-15T14:30:00+00:00" :type "purchase"}]
```

### Weekday Detection

```qi
;; Check if weekend
(defn is-weekend? [date]
  (let [wd (time/weekday date)]
    (or (= wd 0) (= wd 6))))

(is-weekend? "2023-11-11T12:00:00+00:00") ;; => true (Saturday)
(is-weekend? "2023-11-10T12:00:00+00:00") ;; => false (Friday)

;; Get weekday name
(def weekdays ["Sun" "Mon" "Tue" "Wed" "Thu" "Fri" "Sat"])

(defn weekday-name [date]
  (get weekdays (time/weekday date)))

(weekday-name "2023-11-10T12:00:00+00:00") ;; => "Fri"
```

### Date/Time Batch Processing

```qi
;; Generate hourly data points
(defn hourly-points [start-date hours]
  (stream/range 0 hours)
  |> (map (fn [h] (time/add-hours start-date h)))
  |> (map (fn [ts] (time/format ts "%Y-%m-%d %H:00:00")))
  |> collect)

(hourly-points (time/to-unix "2023-11-10T00:00:00+00:00") 24)
;; => ["2023-11-10 00:00:00" "2023-11-10 01:00:00" ... "2023-11-10 23:00:00"]
```

---

## Time Zones

**Important**: All time operations are performed in UTC (Coordinated Universal Time).

### ISO 8601 Format Times

- `time/now-iso` returns UTC timezone (`+00:00`)
- `time/format`, `time/parse` treat times as UTC
- For local timezones, manually calculate the offset

```qi
;; Current time in UTC
(time/now-iso)  ;; => "2025-11-10T15:30:45+00:00"

;; Convert to Japan time (UTC+9)
(time/now-iso
 |> time/to-unix
 |> (time/add-hours _ 9)
 |> (time/format _ "%Y-%m-%d %H:%M:%S JST"))
;; => "2025-11-11 00:30:45 JST"
```

---

## Error Handling

### Invalid Date/Time Strings

```qi
;; Invalid ISO 8601 format
(time/to-unix "invalid-date")
;; => Error: "time/to-unix: Invalid date format: invalid-date"

;; Parse failure
(time/parse "2023-13-45" "%Y-%m-%d")
;; => Error: "time/parse: Failed to parse date string '2023-13-45' with format '%Y-%m-%d'"
```

### Invalid Timestamps

```qi
;; Out-of-range Unix timestamp
(time/from-unix 99999999999999)
;; => Error: "time/from-unix: Invalid timestamp"
```

### Type Errors

```qi
;; Non-string argument
(time/to-unix 123)
;; => Error: "time/to-unix: Only accepts strings"

;; Non-integer days
(time/add-days "2023-11-10T12:00:00+00:00" "7")
;; => Error: "time/add-days (days): Only accepts integers"
```

---

## Function Reference

### Current Time Retrieval
- `time/now-iso` - Get current time in ISO 8601 format
- `time/today` - Get today's date in YYYY-MM-DD format

### Unix Timestamp Conversion
- `time/from-unix` - Unix timestamp → ISO 8601 format
- `time/to-unix` - ISO 8601 format → Unix timestamp

### Formatting/Parsing
- `time/format` - Format timestamp with specified format string
- `time/parse` - Parse date string using format string

### Date/Time Arithmetic (Addition)
- `time/add-days` - Add days
- `time/add-hours` - Add hours
- `time/add-minutes` - Add minutes

### Date/Time Arithmetic (Subtraction)
- `time/sub-days` - Subtract days
- `time/sub-hours` - Subtract hours
- `time/sub-minutes` - Subtract minutes

### Date/Time Arithmetic (Difference)
- `time/diff-days` - Get difference in days
- `time/diff-hours` - Get difference in hours
- `time/diff-minutes` - Get difference in minutes

### Date/Time Comparison
- `time/before?` - Check if date1 is before date2
- `time/after?` - Check if date1 is after date2
- `time/between?` - Check if date is between start and end

### Component Extraction
- `time/year` - Extract year
- `time/month` - Extract month (1-12)
- `time/day` - Extract day (1-31)
- `time/hour` - Extract hour (0-23)
- `time/minute` - Extract minute (0-59)
- `time/second` - Extract second (0-59)
- `time/weekday` - Extract weekday (0=Sunday, 1=Monday, ..., 6=Saturday)

---

## Design Notes

### Adoption of ISO 8601 Format

All string-based date/time representations use ISO 8601 format (RFC 3339). This provides:

- **Explicit timezone information**: Always includes timezone like `+00:00`
- **International standard**: Easy data exchange between programs
- **Sortable**: Correct ordering even with string comparison
- **Clarity**: No ambiguity in dates (MM/DD vs DD/MM, etc.)

### Unix Timestamp Compatibility

- Unix timestamps (integers) are convenient for external system integration and database storage
- `time/to-unix`, `time/from-unix` provide bi-directional conversion
- Calculation results are primarily returned as Unix timestamps (easier for pipeline processing)

### Argument Flexibility

Many functions accept both Unix timestamps and ISO 8601 strings:

```qi
;; Both are valid
(time/add-days 1699632000 7)
(time/add-days "2023-11-10T12:00:00+00:00" 7)
```

This makes pipeline processing smoother.
