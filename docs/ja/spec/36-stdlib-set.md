# 標準ライブラリ - 集合演算（set/）

**数学的な集合演算を提供する標準ライブラリ**

すべての関数は `set/` モジュールに属します。

> **実装**: `src/builtins/set.rs`
> **Feature**: `std-set`

---

## 概要

集合（Set）とは、重複のない要素のコレクションです。Qi言語では、`set/` モジュールが数学的な集合演算を提供します。

内部的には `HashSet` を使用して高速な判定・演算を実現しており、入力としてListやVectorを受け取ります。

### 主な用途

- **重複排除** - リストから重複を削除
- **共通項検索** - 複数のコレクションに共通する要素を抽出
- **差分検出** - 一方にのみ存在する要素を特定
- **集合関係の判定** - 部分集合、上位集合、互いに素の判定

---

## 集合演算

### set/union - 和集合

複数のコレクションの和集合を返します（重複は自動的に削除されます）。

```qi
;; 基本的な使い方
(set/union [1 2] [2 3])
;; => [1 2 3]

;; 3つ以上のコレクションも可能
(set/union [1 2] [2 3] [3 4])
;; => [1 2 3 4]

;; 空のコレクション
(set/union)
;; => []

;; ListとVectorの混在も可能
(set/union (list 1 2) [2 3] (list 3 4))
;; => [1 2 3 4]
```

**パラメータ**:
- `...collections`: 0個以上のList/Vector

**戻り値**:
- List（重複を除いた全要素）

---

### set/intersect - 積集合

全てのコレクションに共通する要素のみを返します。

```qi
;; 基本的な使い方
(set/intersect [1 2 3] [2 3 4])
;; => [2 3]

;; 3つのコレクションの共通要素
(set/intersect [1 2 3 4] [2 3 4 5] [3 4 5 6])
;; => [3 4]

;; 共通要素がない場合
(set/intersect [1 2] [3 4])
;; => []

;; ListとVectorの混在も可能
(set/intersect (list 1 2 3) [2 3 4])
;; => [2 3]
```

**パラメータ**:
- `collection1`: 最初のList/Vector（必須）
- `collection2`: 2つ目のList/Vector（必須）
- `...more`: 追加のList/Vector（オプション）

**戻り値**:
- List（全てのコレクションに共通する要素）

---

### set/difference - 差集合

第1引数から、第2引数以降の要素を全て除いた集合を返します。

```qi
;; 基本的な使い方
(set/difference [1 2 3] [2])
;; => [1 3]

;; 複数のコレクションを除外
(set/difference [1 2 3 4 5] [2 3] [4])
;; => [1 5]

;; 除外する要素がない場合
(set/difference [1 2 3] [4 5])
;; => [1 2 3]

;; ListとVectorの混在も可能
(set/difference (list 1 2 3 4) [2 4])
;; => [1 3]
```

**パラメータ**:
- `base`: ベースとなるList/Vector（必須）
- `exclude1`: 除外するList/Vector（必須）
- `...more`: 追加で除外するList/Vector（オプション）

**戻り値**:
- List（baseから指定された要素を除いた結果）

---

### set/symmetric-difference - 対称差

どちらか一方にのみ存在する要素の集合を返します（排他的論理和）。

```qi
;; 基本的な使い方
(set/symmetric-difference [1 2 3] [2 3 4])
;; => [1 4]

;; 共通要素がない場合は全て含まれる
(set/symmetric-difference [1 2] [3 4])
;; => [1 2 3 4]

;; 全て共通の場合は空
(set/symmetric-difference [1 2] [1 2])
;; => []

;; ListとVectorの混在も可能
(set/symmetric-difference (list 1 2 3) [2 3 4])
;; => [1 4]
```

**パラメータ**:
- `set1`: 最初のList/Vector（必須）
- `set2`: 2つ目のList/Vector（必須）

**戻り値**:
- List（どちらか一方にのみ存在する要素）

**数学的意味**:
```
A △ B = (A - B) ∪ (B - A)
      = (A ∪ B) - (A ∩ B)
```

---

## 集合関係の判定

### set/subset? - 部分集合判定

第1引数が第2引数の部分集合かどうかを判定します。

```qi
;; 基本的な使い方
(set/subset? [1 2] [1 2 3])
;; => true

;; 同じ集合は部分集合
(set/subset? [1 2] [1 2])
;; => true

;; 部分集合でない場合
(set/subset? [1 2 3] [1 2])
;; => false

;; 空集合は全ての集合の部分集合
(set/subset? [] [1 2 3])
;; => true

;; ListとVectorの混在も可能
(set/subset? (list 1 2) [1 2 3 4])
;; => true
```

**パラメータ**:
- `subset`: 部分集合候補のList/Vector（必須）
- `superset`: 親集合候補のList/Vector（必須）

**戻り値**:
- `true`: subsetの全要素がsupersetに含まれる
- `false`: それ以外

**数学的意味**:
```
A ⊆ B ⇔ ∀x ∈ A, x ∈ B
```

---

### set/superset? - 上位集合判定

第1引数が第2引数の上位集合（スーパーセット）かどうかを判定します。

```qi
;; 基本的な使い方
(set/superset? [1 2 3] [1 2])
;; => true

;; 同じ集合は上位集合
(set/superset? [1 2] [1 2])
;; => true

;; 上位集合でない場合
(set/superset? [1 2] [1 2 3])
;; => false

;; 全ての集合は空集合の上位集合
(set/superset? [1 2 3] [])
;; => true

;; ListとVectorの混在も可能
(set/superset? (list 1 2 3 4) [1 2])
;; => true
```

**パラメータ**:
- `superset`: 上位集合候補のList/Vector（必須）
- `subset`: 部分集合候補のList/Vector（必須）

**戻り値**:
- `true`: subsetの全要素がsupersetに含まれる
- `false`: それ以外

**注**: `set/superset?(A, B)` は `set/subset?(B, A)` と同じ意味です。

**数学的意味**:
```
A ⊇ B ⇔ B ⊆ A
```

---

### set/disjoint? - 互いに素判定

2つのコレクションに共通要素がないかどうかを判定します。

```qi
;; 基本的な使い方
(set/disjoint? [1 2] [3 4])
;; => true

;; 共通要素がある場合
(set/disjoint? [1 2 3] [3 4 5])
;; => false

;; 空集合は全ての集合と互いに素
(set/disjoint? [] [1 2 3])
;; => true

;; ListとVectorの混在も可能
(set/disjoint? (list 1 2) [3 4])
;; => true
```

**パラメータ**:
- `set1`: 最初のList/Vector（必須）
- `set2`: 2つ目のList/Vector（必須）

**戻り値**:
- `true`: 共通要素が1つもない
- `false`: 少なくとも1つの共通要素がある

**数学的意味**:
```
A ∩ B = ∅
```

---

## 実用例

### 重複排除

```qi
;; リストから重複を削除
(def items [1 2 2 3 3 3 4])

;; 自分自身との和集合 = 重複排除
(set/union items)
;; => [1 2 3 4]

;; distinctとの違い（set/unionは順序が不定）
(distinct items)
;; => [1 2 3 4]（入力の順序を維持）
```

---

### ユーザー権限管理

```qi
;; ユーザーが持つ権限
(def user-permissions [:read :write :delete])

;; 必要な権限
(def required-permissions [:read :write])

;; 必要な権限を全て持っているか
(set/subset? required-permissions user-permissions)
;; => true

;; 管理者権限を持っているか
(set/superset? user-permissions [:admin])
;; => false
```

---

### タグ検索

```qi
;; 記事のタグ
(def article1-tags [:programming :rust :web])
(def article2-tags [:programming :python :cli])
(def article3-tags [:rust :systems])

;; 「programming」または「rust」のタグを持つ記事を検索
(def search-tags [:programming :rust])

;; article1とsearch-tagsに共通タグがあるか
(not (set/disjoint? article1-tags search-tags))
;; => true（:programming と :rust が共通）

;; 全てのタグを集約
(set/union article1-tags article2-tags article3-tags)
;; => [:programming :rust :web :python :cli :systems]
```

---

### データ比較・差分検出

```qi
;; 旧バージョンのユーザーID
(def old-users [101 102 103 104 105])

;; 新バージョンのユーザーID
(def new-users [103 104 105 106 107])

;; 削除されたユーザー
(set/difference old-users new-users)
;; => [101 102]

;; 追加されたユーザー
(set/difference new-users old-users)
;; => [106 107]

;; 変更のあったユーザー（削除 + 追加）
(set/symmetric-difference old-users new-users)
;; => [101 102 106 107]

;; 変更のないユーザー
(set/intersect old-users new-users)
;; => [103 104 105]
```

---

### マルチ条件フィルタリング

```qi
;; 条件ごとに該当するIDをフィルタ
(def active-users [1 2 3 4 5])
(def premium-users [3 4 5 6])
(def verified-users [2 3 5 7])

;; アクティブかつプレミアムなユーザー
(set/intersect active-users premium-users)
;; => [3 4 5]

;; アクティブだがプレミアムでないユーザー
(set/difference active-users premium-users)
;; => [1 2]

;; プレミアムまたは認証済みのユーザー
(set/union premium-users verified-users)
;; => [3 4 5 6 2 7]

;; 3つの条件全てを満たすユーザー
(set/intersect active-users premium-users verified-users)
;; => [3 5]
```

---

### パイプラインとの統合

```qi
;; ユーザーデータから特定の役割を持つユーザーを抽出
(def users [
  {:id 1 :roles [:admin :user]}
  {:id 2 :roles [:user :guest]}
  {:id 3 :roles [:admin :moderator]}
  {:id 4 :roles [:user]}
])

;; 管理者かモデレーターのIDを抽出
(users
 |> (filter (fn [u]
              (not (set/disjoint? (:roles u) [:admin :moderator]))))
 |> (map :id))
;; => [1 3]

;; 複数の必須権限を全て持つユーザーを抽出
(def required-perms [:read :write :execute])

(users
 |> (filter (fn [u]
              (set/subset? required-perms (:roles u))))
 |> (map :id))
;; => （条件を満たすユーザーがいれば返却）
```

---

## パフォーマンスの考慮

### 内部実装

- 全ての集合演算は `HashSet` ベースで高速動作
- 要素の判定は O(1) の平均計算量
- 集合演算全体は O(n) の計算量（n = 要素数）

### 大量データでの使用

```qi
;; 数万件のIDリストでも高速
(def old-ids (range 100000))
(def new-ids (range 50000 150000))

;; O(n) で差分を計算
(set/difference old-ids new-ids)
;; => (0 1 2 ... 49999)

;; 頻繁に判定する場合はキャッシュを活用
(def common-ids (set/intersect old-ids new-ids))

;; commonに含まれるかを高速判定（外部関数との組み合わせ）
(contains? (to-vector common-ids) 75000)
;; => true
```

---

## 既存のコレクション関数との使い分け

### distinct vs set/union

```qi
;; distinct - 順序を維持して重複削除
(distinct [3 1 2 1 3])
;; => [3 1 2]

;; set/union - 順序は不定（ハッシュベース）
(set/union [3 1 2 1 3])
;; => [3 1 2] または [1 2 3]（実装依存）
```

**選び方**:
- 順序が重要 → `distinct`
- 順序が不要で高速性重視 → `set/union`

### filter vs set/intersect

```qi
(def allowed-ids [1 2 3])
(def all-items [{:id 1} {:id 2} {:id 4}])

;; filter - 複雑な条件が可能
(all-items |> (filter (fn [item] (some? (find (fn [id] (= id (:id item))) allowed-ids)))))

;; 単純なID照合なら set/intersect が簡潔
(def item-ids (all-items |> (map :id)))
(set/intersect item-ids allowed-ids)
;; => [1 2]
```

---

## まとめ

| 関数 | 説明 | 数学記号 |
|------|------|----------|
| `set/union` | 和集合 | A ∪ B |
| `set/intersect` | 積集合 | A ∩ B |
| `set/difference` | 差集合 | A - B |
| `set/symmetric-difference` | 対称差 | A △ B |
| `set/subset?` | 部分集合判定 | A ⊆ B |
| `set/superset?` | 上位集合判定 | A ⊇ B |
| `set/disjoint?` | 互いに素判定 | A ∩ B = ∅ |

**Qiの集合演算の特徴**:
- ListとVectorの両方をサポート
- 戻り値は常にList
- `HashSet` ベースの高速演算
- パイプライン統合可能
- 0引数や単一引数もサポート（union、differenceなど）
