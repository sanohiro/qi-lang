# Standard Library - Set Operations (set/)

**Mathematical set operations for collections**

All functions belong to the `set/` module.

> **Implementation**: `src/builtins/set.rs`
> **Feature**: `std-set`

---

## Overview

A set is a collection of unique elements without duplicates. In Qi, the `set/` module provides mathematical set operations.

Internally using `HashSet` for fast operations, these functions accept Lists or Vectors as input.

### Primary Use Cases

- **Deduplication** - Remove duplicates from lists
- **Finding Common Elements** - Extract elements common to multiple collections
- **Difference Detection** - Identify elements that exist in only one collection
- **Set Relationship Testing** - Check subset, superset, and disjoint relationships

---

## Set Operations

### set/union - Union

Returns the union of multiple collections (duplicates are automatically removed).

```qi
;; Basic usage
(set/union [1 2] [2 3])
;; => [1 2 3]

;; Three or more collections
(set/union [1 2] [2 3] [3 4])
;; => [1 2 3 4]

;; Empty collection
(set/union)
;; => []

;; Mixed List and Vector
(set/union (list 1 2) [2 3] (list 3 4))
;; => [1 2 3 4]
```

**Parameters**:
- `...collections`: Zero or more Lists/Vectors

**Returns**:
- List (all elements without duplicates)

---

### set/intersect - Intersection

Returns only elements common to all collections.

```qi
;; Basic usage
(set/intersect [1 2 3] [2 3 4])
;; => [2 3]

;; Common elements in three collections
(set/intersect [1 2 3 4] [2 3 4 5] [3 4 5 6])
;; => [3 4]

;; No common elements
(set/intersect [1 2] [3 4])
;; => []

;; Mixed List and Vector
(set/intersect (list 1 2 3) [2 3 4])
;; => [2 3]
```

**Parameters**:
- `collection1`: First List/Vector (required)
- `collection2`: Second List/Vector (required)
- `...more`: Additional Lists/Vectors (optional)

**Returns**:
- List (elements common to all collections)

---

### set/difference - Difference

Returns a set with all elements from subsequent arguments removed from the first argument.

```qi
;; Basic usage
(set/difference [1 2 3] [2])
;; => [1 3]

;; Exclude multiple collections
(set/difference [1 2 3 4 5] [2 3] [4])
;; => [1 5]

;; No elements to exclude
(set/difference [1 2 3] [4 5])
;; => [1 2 3]

;; Mixed List and Vector
(set/difference (list 1 2 3 4) [2 4])
;; => [1 3]
```

**Parameters**:
- `base`: Base List/Vector (required)
- `exclude1`: List/Vector to exclude (required)
- `...more`: Additional Lists/Vectors to exclude (optional)

**Returns**:
- List (base with specified elements removed)

---

### set/symmetric-difference - Symmetric Difference

Returns elements that exist in only one of the two collections (exclusive OR).

```qi
;; Basic usage
(set/symmetric-difference [1 2 3] [2 3 4])
;; => [1 4]

;; No common elements - all included
(set/symmetric-difference [1 2] [3 4])
;; => [1 2 3 4]

;; All common - empty result
(set/symmetric-difference [1 2] [1 2])
;; => []

;; Mixed List and Vector
(set/symmetric-difference (list 1 2 3) [2 3 4])
;; => [1 4]
```

**Parameters**:
- `set1`: First List/Vector (required)
- `set2`: Second List/Vector (required)

**Returns**:
- List (elements in one collection but not both)

**Mathematical Meaning**:
```
A △ B = (A - B) ∪ (B - A)
      = (A ∪ B) - (A ∩ B)
```

---

## Set Relationship Testing

### set/subset? - Subset Test

Tests whether the first argument is a subset of the second argument.

```qi
;; Basic usage
(set/subset? [1 2] [1 2 3])
;; => true

;; Identical sets are subsets
(set/subset? [1 2] [1 2])
;; => true

;; Not a subset
(set/subset? [1 2 3] [1 2])
;; => false

;; Empty set is a subset of all sets
(set/subset? [] [1 2 3])
;; => true

;; Mixed List and Vector
(set/subset? (list 1 2) [1 2 3 4])
;; => true
```

**Parameters**:
- `subset`: Candidate subset List/Vector (required)
- `superset`: Candidate superset List/Vector (required)

**Returns**:
- `true`: All elements of subset are in superset
- `false`: Otherwise

**Mathematical Meaning**:
```
A ⊆ B ⇔ ∀x ∈ A, x ∈ B
```

---

### set/superset? - Superset Test

Tests whether the first argument is a superset of the second argument.

```qi
;; Basic usage
(set/superset? [1 2 3] [1 2])
;; => true

;; Identical sets are supersets
(set/superset? [1 2] [1 2])
;; => true

;; Not a superset
(set/superset? [1 2] [1 2 3])
;; => false

;; All sets are supersets of the empty set
(set/superset? [1 2 3] [])
;; => true

;; Mixed List and Vector
(set/superset? (list 1 2 3 4) [1 2])
;; => true
```

**Parameters**:
- `superset`: Candidate superset List/Vector (required)
- `subset`: Candidate subset List/Vector (required)

**Returns**:
- `true`: All elements of subset are in superset
- `false`: Otherwise

**Note**: `set/superset?(A, B)` is equivalent to `set/subset?(B, A)`.

**Mathematical Meaning**:
```
A ⊇ B ⇔ B ⊆ A
```

---

### set/disjoint? - Disjoint Test

Tests whether two collections have no common elements.

```qi
;; Basic usage
(set/disjoint? [1 2] [3 4])
;; => true

;; Has common elements
(set/disjoint? [1 2 3] [3 4 5])
;; => false

;; Empty set is disjoint with all sets
(set/disjoint? [] [1 2 3])
;; => true

;; Mixed List and Vector
(set/disjoint? (list 1 2) [3 4])
;; => true
```

**Parameters**:
- `set1`: First List/Vector (required)
- `set2`: Second List/Vector (required)

**Returns**:
- `true`: No common elements
- `false`: At least one common element

**Mathematical Meaning**:
```
A ∩ B = ∅
```

---

## Practical Examples

### Deduplication

```qi
;; Remove duplicates from a list
(def items [1 2 2 3 3 3 4])

;; Union with itself = deduplication
(set/union items)
;; => [1 2 3 4]

;; Difference from distinct (set/union order is undefined)
(distinct items)
;; => [1 2 3 4] (preserves input order)
```

---

### User Permission Management

```qi
;; User's permissions
(def user-permissions [:read :write :delete])

;; Required permissions
(def required-permissions [:read :write])

;; Does user have all required permissions?
(set/subset? required-permissions user-permissions)
;; => true

;; Does user have admin permission?
(set/superset? user-permissions [:admin])
;; => false
```

---

### Tag Search

```qi
;; Article tags
(def article1-tags [:programming :rust :web])
(def article2-tags [:programming :python :cli])
(def article3-tags [:rust :systems])

;; Search for articles with "programming" or "rust" tags
(def search-tags [:programming :rust])

;; Does article1 have any common tags with search-tags?
(not (set/disjoint? article1-tags search-tags))
;; => true (:programming and :rust are common)

;; Collect all unique tags
(set/union article1-tags article2-tags article3-tags)
;; => [:programming :rust :web :python :cli :systems]
```

---

### Data Comparison and Diff Detection

```qi
;; Old version user IDs
(def old-users [101 102 103 104 105])

;; New version user IDs
(def new-users [103 104 105 106 107])

;; Deleted users
(set/difference old-users new-users)
;; => [101 102]

;; Added users
(set/difference new-users old-users)
;; => [106 107]

;; Changed users (deleted + added)
(set/symmetric-difference old-users new-users)
;; => [101 102 106 107]

;; Unchanged users
(set/intersect old-users new-users)
;; => [103 104 105]
```

---

### Multi-Condition Filtering

```qi
;; Filter IDs by condition
(def active-users [1 2 3 4 5])
(def premium-users [3 4 5 6])
(def verified-users [2 3 5 7])

;; Active AND premium users
(set/intersect active-users premium-users)
;; => [3 4 5]

;; Active but NOT premium users
(set/difference active-users premium-users)
;; => [1 2]

;; Premium OR verified users
(set/union premium-users verified-users)
;; => [3 4 5 6 2 7]

;; Users satisfying all three conditions
(set/intersect active-users premium-users verified-users)
;; => [3 5]
```

---

### Pipeline Integration

```qi
;; Extract users with specific roles from user data
(def users [
  {:id 1 :roles [:admin :user]}
  {:id 2 :roles [:user :guest]}
  {:id 3 :roles [:admin :moderator]}
  {:id 4 :roles [:user]}
])

;; Extract IDs of admins or moderators
(users
 |> (filter (fn [u]
              (not (set/disjoint? (:roles u) [:admin :moderator]))))
 |> (map :id))
;; => [1 3]

;; Extract users with all required permissions
(def required-perms [:read :write :execute])

(users
 |> (filter (fn [u]
              (set/subset? required-perms (:roles u))))
 |> (map :id))
;; => (returns if any user satisfies condition)
```

---

## Performance Considerations

### Internal Implementation

- All set operations use `HashSet` for fast performance
- Element lookup is O(1) average time complexity
- Overall set operations are O(n) complexity (n = number of elements)

### Usage with Large Data

```qi
;; Fast even with tens of thousands of IDs
(def old-ids (range 100000))
(def new-ids (range 50000 150000))

;; Calculate difference in O(n)
(set/difference old-ids new-ids)
;; => (0 1 2 ... 49999)

;; Cache when performing frequent checks
(def common-ids (set/intersect old-ids new-ids))

;; Fast membership test (combined with external functions)
(contains? (to-vector common-ids) 75000)
;; => true
```

---

## Comparison with Existing Collection Functions

### distinct vs set/union

```qi
;; distinct - preserves order and removes duplicates
(distinct [3 1 2 1 3])
;; => [3 1 2]

;; set/union - order is undefined (hash-based)
(set/union [3 1 2 1 3])
;; => [3 1 2] or [1 2 3] (implementation-dependent)
```

**When to use**:
- Order matters → `distinct`
- Order doesn't matter, speed matters → `set/union`

### filter vs set/intersect

```qi
(def allowed-ids [1 2 3])
(def all-items [{:id 1} {:id 2} {:id 4}])

;; filter - allows complex conditions
(all-items |> (filter (fn [item] (some? (find (fn [id] (= id (:id item))) allowed-ids)))))

;; For simple ID matching, set/intersect is more concise
(def item-ids (all-items |> (map :id)))
(set/intersect item-ids allowed-ids)
;; => [1 2]
```

---

## Summary

| Function | Description | Math Symbol |
|----------|-------------|-------------|
| `set/union` | Union | A ∪ B |
| `set/intersect` | Intersection | A ∩ B |
| `set/difference` | Difference | A - B |
| `set/symmetric-difference` | Symmetric difference | A △ B |
| `set/subset?` | Subset test | A ⊆ B |
| `set/superset?` | Superset test | A ⊇ B |
| `set/disjoint?` | Disjoint test | A ∩ B = ∅ |

**Features of Qi's Set Operations**:
- Supports both Lists and Vectors
- Always returns a List
- Fast `HashSet`-based operations
- Pipeline integration ready
- Supports zero or single arguments (union, difference, etc.)
