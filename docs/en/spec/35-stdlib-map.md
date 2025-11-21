# Standard Library - Map Extensions (map/)

**Advanced Map Operation Functions**

> **Implementation**: `src/builtins/map.rs`

This module provides advanced map operation features in addition to basic map operations (`get`, `keys`, `vals`, `assoc`, `dissoc`, `merge`, `get-in`, `update`, `update-in`).

**Difference from Basic Functions**:
- **Basic Functions** (`src/builtins/core_collections.rs`): Always available as core features
  - `get`, `keys`, `vals`, `assoc`, `dissoc`, `merge`
  - `get-in`, `update`, `update-in`
- **Extension Functions** (this module): Advanced operations for specific use cases
  - Modifying nested structures (`assoc-in`, `dissoc-in`)
  - Bulk transformation of keys/values (`update-keys`, `update-vals`)
  - Key selection (`select-keys`)

---

## Key Selection

### select-keys - Extract Only Specified Keys

Returns a new map containing only the specified keys.

```qi
;; Basic usage
(map/select-keys {:a 1 :b 2 :c 3} [:a :c])
;; => {:a 1, :c 3}

;; Non-existent keys are ignored
(map/select-keys {:a 1 :b 2} [:a :x :y])
;; => {:a 1}

;; Empty key list returns empty map
(map/select-keys {:a 1 :b 2} [])
;; => {}

;; Can specify with vector or list
(map/select-keys {:a 1 :b 2 :c 3} (list :b :c))
;; => {:b 2, :c 3}
```

**Use in Pipeline**:

```qi
;; Extract only necessary fields from API response
(user
 |> (map/select-keys _ [:id :name :email]))

;; Format records retrieved from database
(records
 |> (map (fn [r] (map/select-keys r [:user_id :created_at :status]))))
```

---

## Modifying Nested Structures

### assoc-in - Set Value in Nested Map

Sets a value at a path in a nested map. Automatically creates intermediate maps if they don't exist.

```qi
;; Basic usage
(map/assoc-in {} [:user :profile :name] "Alice")
;; => {:user {:profile {:name "Alice"}}}

;; Add to existing map
(map/assoc-in {:user {:age 30}} [:user :name] "Bob")
;; => {:user {:age 30, :name "Bob"}}

;; Deep nesting
(map/assoc-in {} [:app :config :db :host] "localhost")
;; => {:app {:config {:db {:host "localhost"}}}}

;; Overwrite existing value
(map/assoc-in {:user {:name "Alice"}} [:user :name] "Bob")
;; => {:user {:name "Bob"}}
```

**Use in Pipeline**:

```qi
;; Incremental configuration building
({}
 |> (map/assoc-in _ [:server :port] 8080)
 |> (map/assoc-in _ [:server :host] "0.0.0.0")
 |> (map/assoc-in _ [:db :host] "localhost")
 |> (map/assoc-in _ [:db :port] 5432))
;; => {:server {:port 8080, :host "0.0.0.0"},
;;     :db {:host "localhost", :port 5432}}
```

### dissoc-in - Remove Key from Nested Map

Removes a key at a path in a nested map.

```qi
;; Basic usage
(map/dissoc-in {:user {:name "Alice" :age 30}} [:user :age])
;; => {:user {:name "Alice"}}

;; Deep nesting
(map/dissoc-in {:a {:b {:c 1 :d 2}}} [:a :b :c])
;; => {:a {:b {:d 2}}}

;; Deleting the last key leaves an empty map
(map/dissoc-in {:a {:b {:c 1}}} [:a :b :c])
;; => {:a {:b {}}}

;; Non-existent path does nothing
(map/dissoc-in {:a 1} [:x :y :z])
;; => {:a 1}
```

**Use in Pipeline**:

```qi
;; Remove sensitive information
(user-data
 |> (map/dissoc-in _ [:credentials :password])
 |> (map/dissoc-in _ [:private :ssn]))
```

---

## Value Filtering

### filter-vals - Filter Values by Predicate Function

Returns a new map containing only values for which the predicate function returns true.

```qi
;; Basic usage
(map/filter-vals (fn [v] (> v 18)) {:alice 25 :bob 17 :charlie 30 :diana 16})
;; => {:alice 25, :charlie 30}

;; Filter by type
(map/filter-vals string? {:a "hello" :b 42 :c "world" :d true})
;; => {:a "hello", :c "world"}

;; Exclude empty strings
(map/filter-vals (fn [v] (not (empty? v))) {:name "Alice" :email "" :city "Tokyo"})
;; => {:name "Alice", :city "Tokyo"}

;; Filter by numeric range
(map/filter-vals (fn [v] (and (>= v 0) (<= v 100)))
  {:score1 85 :score2 -5 :score3 120 :score4 90})
;; => {:score1 85, :score4 90}
```

**Use in Pipeline**:

```qi
;; Extract only valid data from API response
(user-data
 |> (map/filter-vals _ (fn [v] (not (nil? v))))
 |> (map/filter-vals _ (fn [v] (not (empty? v)))))

;; Age restriction filter
(users
 |> (map (fn [u] (map/filter-vals u (fn [age] (>= age 18))))))
```

---

## Collection Grouping

### group-by - Group Collection by Key Function

Applies a key function to each element of a collection and returns a map grouping elements with the same key.

```qi
;; Basic usage
(map/group-by (fn [x] (% x 10)) [1 2 3 11 12 13 21 22 23])
;; => {1 [1 11 21], 2 [2 12 22], 3 [3 13 23]}

;; Group by map key
(map/group-by :type
  [{:type "A" :val 1} {:type "A" :val 2} {:type "B" :val 3}])
;; => {"A" [{:type "A" :val 1} {:type "A" :val 2}],
;;     "B" [{:type "B" :val 3}]}

;; Group by even/odd
(map/group-by (fn [x] (if (= (% x 2) 0) :even :odd)) [1 2 3 4 5 6])
;; => {:even [2 4 6], :odd [1 3 5]}

;; Group by string length
(map/group-by str/length ["a" "bb" "ccc" "dd" "e" "fff"])
;; => {1 ["a" "e"], 2 ["bb" "dd"], 3 ["ccc" "fff"]}
```

**Use in Pipeline**:

```qi
;; Group users by city
(users
 |> (map/group-by _ :city))

;; Aggregate logs by date
(logs
 |> (map/group-by _ (fn [log] (get log :date))))
```

---

## Recursive Map Merging

### deep-merge - Recursively Merge Nested Maps

Recursively merges multiple maps while preserving nested structure. Unlike regular `merge`, nested maps are also merged recursively.

```qi
;; Basic usage
(map/deep-merge {:a {:b 1}} {:a {:c 2}})
;; => {:a {:b 1, :c 2}}

;; Difference from regular merge
(merge {:a {:b 1}} {:a {:c 2}})
;; => {:a {:c 2}}  ;; Overwritten

(map/deep-merge {:a {:b 1}} {:a {:c 2}})
;; => {:a {:b 1, :c 2}}  ;; Merged

;; Merge multiple maps
(map/deep-merge
  {:a {:b 1}}
  {:a {:b 2 :c 3}}
  {:a {:d 4}})
;; => {:a {:b 2, :c 3, :d 4}}

;; Deep nesting
(map/deep-merge
  {:db {:host "localhost" :port 5432} :app {:name "MyApp"}}
  {:db {:port 3306 :user "admin"} :app {:version "1.0"}})
;; => {:db {:host "localhost", :port 3306, :user "admin"},
;;     :app {:name "MyApp", :version "1.0"}}

;; Merge with empty map
(map/deep-merge {} {:a 1})
;; => {:a 1}

(map/deep-merge)
;; => {}
```

**Use in Pipeline**:

```qi
;; Merge configuration files
(default-config
 |> (map/deep-merge _ user-config)
 |> (map/deep-merge _ env-config))

;; Merge API responses
(base-response
 |> (map/deep-merge _ additional-fields)
 |> (map/deep-merge _ metadata))
```

---

## Bulk Key/Value Transformation

### update-keys - Apply Function to All Keys

Applies a function to all keys in a map and returns a new map.

```qi
;; Basic usage
(map/update-keys str/upper {:name "Alice" :age 30})
;; => {"NAME" "Alice", "AGE" 30}

;; Key normalization
(map/update-keys str/lower {:Name "Alice" :AGE 30})
;; => {"name" "Alice", "age" 30}

;; Key transformation
(map/update-keys (fn [k] (str "prefix_" k)) {:a 1 :b 2})
;; => {"prefix_a" 1, "prefix_b" 2}

;; Keywords to strings
(map/update-keys name {:name "Alice" :age 30})
;; => {"name" "Alice", "age" 30}
```

**Use in Pipeline**:

```qi
;; JSON key normalization
(json-data
 |> (map/update-keys _ str/lower)
 |> (map/update-keys _ (fn [k] (str/replace k "-" "_"))))

;; Database column name conversion
(db-record
 |> (map/update-keys _ str/snake))
```

### update-vals - Apply Function to All Values

Applies a function to all values in a map and returns a new map.

```qi
;; Basic usage
(map/update-vals inc {:a 1 :b 2})
;; => {:a 2, :b 3}

;; Value type conversion
(map/update-vals str {:a 1 :b 2 :c 3})
;; => {:a "1", :b "2", :c "3"}

;; Value normalization
(map/update-vals str/trim {:name "  Alice  " :city "  Tokyo  "})
;; => {:name "Alice", :city "Tokyo"}

;; Number transformation
(map/update-vals (fn [x] (* x 2)) {:a 10 :b 20 :c 30})
;; => {:a 20, :b 40, :c 60}
```

**Use in Pipeline**:

```qi
;; Price calculation
(prices
 |> (map/update-vals _ (fn [p] (* p 1.1))))  ;; 10% increase

;; Data normalization
(form-data
 |> (map/update-vals _ str/trim)
 |> (map/update-vals _ str/lower))
```

---

## Practical Examples

### Configuration Merging

```qi
;; Merge default and user configurations
(def default-config
  {:server {:host "localhost" :port 8080}
   :db {:host "localhost" :port 5432}
   :cache {:ttl 3600}})

(def user-config
  {:server {:port 3000}
   :db {:host "db.example.com"}})

;; Incremental merge
(default-config
 |> (map/assoc-in _ [:server :port] (get-in user-config [:server :port]))
 |> (map/assoc-in _ [:db :host] (get-in user-config [:db :host])))
```

### API Response Formatting

```qi
;; Extract only necessary fields and transform keys
(defn format-user-response [user]
  (user
   |> (map/select-keys _ [:id :name :email :created_at])
   |> (map/update-keys _ str/camel)))

(format-user-response
  {:id 1 :name "Alice" :email "alice@example.com"
   :created_at "2024-01-01" :password_hash "..." :salt "..."})
;; => {"id" 1, "name" "Alice", "email" "alice@example.com",
;;     "createdAt" "2024-01-01"}
```

### Database Record Transformation

```qi
;; Transform database records for frontend
(defn transform-records [records]
  (records
   |> (map (fn [r]
             (r
              |> (map/select-keys _ [:user_id :username :status :created_at])
              |> (map/update-keys _ str/camel)
              |> (map/update-vals _ (fn [v]
                                      (if (string? v)
                                        (str/trim v)
                                        v))))))))

(transform-records
  [{:user_id 1 :username " alice " :status "active" :created_at "2024-01-01" :internal_id 999}
   {:user_id 2 :username " bob " :status "inactive" :created_at "2024-01-02" :internal_id 1000}])
;; => [{"userId" 1, "username" "alice", "status" "active", "createdAt" "2024-01-01"}
;;     {"userId" 2, "username" "bob", "status" "inactive", "createdAt" "2024-01-02"}]
```

### Removing Sensitive Information

```qi
;; Remove sensitive information before logging
(defn sanitize-for-log [data]
  (data
   |> (map/dissoc-in _ [:user :credentials :password])
   |> (map/dissoc-in _ [:user :credentials :api_key])
   |> (map/dissoc-in _ [:payment :card_number])
   |> (map/dissoc-in _ [:payment :cvv])))

(sanitize-for-log
  {:user {:name "Alice"
          :credentials {:username "alice" :password "secret" :api_key "xyz123"}}
   :payment {:card_number "1234-5678-9012-3456" :cvv "123" :amount 1000}})
;; => {:user {:name "Alice", :credentials {:username "alice"}},
;;     :payment {:amount 1000}}
```

### Dynamic Configuration Building

```qi
;; Build configuration from environment variables
(defn build-config [env]
  ({}
   |> (map/assoc-in _ [:server :host] (get env "SERVER_HOST" "localhost"))
   |> (map/assoc-in _ [:server :port] (str/parse-int (get env "SERVER_PORT" "8080")))
   |> (map/assoc-in _ [:db :host] (get env "DB_HOST" "localhost"))
   |> (map/assoc-in _ [:db :port] (str/parse-int (get env "DB_PORT" "5432")))
   |> (map/assoc-in _ [:db :name] (get env "DB_NAME" "app"))
   |> (map/assoc-in _ [:cache :enabled] (= (get env "CACHE_ENABLED" "true") "true"))
   |> (map/assoc-in _ [:cache :ttl] (str/parse-int (get env "CACHE_TTL" "3600")))))
```

### Configuration File Merging

```qi
;; Merge default, environment-specific, and user configurations
(def default-config
  {:server {:host "localhost" :port 8080 :timeout 30}
   :db {:host "localhost" :port 5432 :pool_size 10}
   :cache {:enabled true :ttl 3600}})

(def production-config
  {:server {:host "0.0.0.0" :port 80}
   :db {:host "prod-db.example.com" :ssl true}})

(def user-overrides
  {:cache {:ttl 7200}
   :db {:pool_size 20}})

;; Merge configurations with deep-merge
(def final-config
  (map/deep-merge default-config production-config user-overrides))
;; => {:server {:host "0.0.0.0", :port 80, :timeout 30},
;;     :db {:host "prod-db.example.com", :port 5432, :pool_size 20, :ssl true},
;;     :cache {:enabled true, :ttl 7200}}
```

### Data Aggregation and Filtering

```qi
;; Group user data by city and extract only active adult users
(def users
  [{:name "Alice" :city "Tokyo" :status "active" :age 25}
   {:name "Bob" :city "Osaka" :status "inactive" :age 17}
   {:name "Charlie" :city "Tokyo" :status "active" :age 30}
   {:name "Diana" :city "Osaka" :status "active" :age 16}])

;; Group by city
(def by-city (map/group-by :city users))
;; => {"Tokyo" [{...Alice...} {...Charlie...}],
;;     "Osaka" [{...Bob...} {...Diana...}]}

;; Extract only active adult users from each city
(defn active-adults [user-list]
  (user-list
   |> (filter (fn [u] (= (get u :status) "active")) _)
   |> (filter (fn [u] (>= (get u :age) 18)) _)))

(map/update-vals active-adults by-city)
;; => {"Tokyo" [{...Alice...} {...Charlie...}],
;;     "Osaka" []}
```

---

## Function List

### Key Selection
- `map/select-keys` - Extract only specified keys

### Nested Operations
- `map/assoc-in` - Set value in nested map
- `map/dissoc-in` - Remove key from nested map
- `map/deep-merge` - Recursively merge nested maps

### Bulk Transformation
- `map/update-keys` - Apply function to all keys
- `map/update-vals` - Apply function to all values

### Filtering and Grouping
- `map/filter-vals` - Filter values by predicate function
- `map/group-by` - Group collection by key function

---

## Performance Notes

- All functions return new maps (immutable)
- `update-keys` and `update-vals` apply functions to each element, which can be time-consuming for large maps
- `assoc-in` recursively copies nested structures, so processing cost increases with deep nesting
- For frequent updates, consider using atoms (`atom`) or agents (`agent`)
