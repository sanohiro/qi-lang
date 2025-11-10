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

---

## Function List

### Key Selection
- `map/select-keys` - Extract only specified keys

### Nested Operations
- `map/assoc-in` - Set value in nested map
- `map/dissoc-in` - Remove key from nested map

### Bulk Transformation
- `map/update-keys` - Apply function to all keys
- `map/update-vals` - Apply function to all values

---

## Performance Notes

- All functions return new maps (immutable)
- `update-keys` and `update-vals` apply functions to each element, which can be time-consuming for large maps
- `assoc-in` recursively copies nested structures, so processing cost increases with deep nesting
- For frequent updates, consider using atoms (`atom`) or agents (`agent`)
