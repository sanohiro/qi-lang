# 標準ライブラリ - マップ拡張（map/）

**高度なマップ操作関数**

> **実装**: `src/builtins/map.rs`

このモジュールは、基本的なマップ操作（`get`, `keys`, `vals`, `assoc`, `dissoc`, `merge`, `get-in`, `update`, `update-in`）に加えて、より高度なマップ操作機能を提供します。

**基本関数との違い**:
- **基本関数**（`src/builtins/core_collections.rs`）: コア機能として常に利用可能
  - `get`, `keys`, `vals`, `assoc`, `dissoc`, `merge`
  - `get-in`, `update`, `update-in`
- **拡張関数**（このモジュール）: 特定のユースケース向けの高度な操作
  - ネストした構造の変更（`assoc-in`, `dissoc-in`）
  - キー・値の一括変換（`update-keys`, `update-vals`）
  - キーの選択（`select-keys`）

---

## キーの選択

### select-keys - 指定したキーのみ抽出

指定したキーのみを含む新しいマップを返します。

```qi
;; 基本的な使い方
(map/select-keys {:a 1 :b 2 :c 3} [:a :c])
;; => {:a 1, :c 3}

;; 存在しないキーは無視される
(map/select-keys {:a 1 :b 2} [:a :x :y])
;; => {:a 1}

;; 空のキーリストは空マップを返す
(map/select-keys {:a 1 :b 2} [])
;; => {}

;; Vectorでもリストでも指定可能
(map/select-keys {:a 1 :b 2 :c 3} (list :b :c))
;; => {:b 2, :c 3}
```

**パイプラインで使う**:

```qi
;; APIレスポンスから必要なフィールドのみ抽出
(user
 |> (map/select-keys _ [:id :name :email]))

;; データベースから取得したレコードの整形
(records
 |> (map (fn [r] (map/select-keys r [:user_id :created_at :status]))))
```

---

## ネストした構造の変更

### assoc-in - ネストしたマップに値を設定

ネストしたマップのパスに値を設定します。中間のマップが存在しない場合は自動的に作成されます。

```qi
;; 基本的な使い方
(map/assoc-in {} [:user :profile :name] "Alice")
;; => {:user {:profile {:name "Alice"}}}

;; 既存のマップに追加
(map/assoc-in {:user {:age 30}} [:user :name] "Bob")
;; => {:user {:age 30, :name "Bob"}}

;; 深いネスト
(map/assoc-in {} [:app :config :db :host] "localhost")
;; => {:app {:config {:db {:host "localhost"}}}}

;; 既存の値を上書き
(map/assoc-in {:user {:name "Alice"}} [:user :name] "Bob")
;; => {:user {:name "Bob"}}
```

**パイプラインで使う**:

```qi
;; 設定の段階的な構築
({}
 |> (map/assoc-in _ [:server :port] 8080)
 |> (map/assoc-in _ [:server :host] "0.0.0.0")
 |> (map/assoc-in _ [:db :host] "localhost")
 |> (map/assoc-in _ [:db :port] 5432))
;; => {:server {:port 8080, :host "0.0.0.0"},
;;     :db {:host "localhost", :port 5432}}
```

### dissoc-in - ネストしたマップからキーを削除

ネストしたマップのパスにあるキーを削除します。

```qi
;; 基本的な使い方
(map/dissoc-in {:user {:name "Alice" :age 30}} [:user :age])
;; => {:user {:name "Alice"}}

;; 深いネスト
(map/dissoc-in {:a {:b {:c 1 :d 2}}} [:a :b :c])
;; => {:a {:b {:d 2}}}

;; 最後のキーを削除すると空マップが残る
(map/dissoc-in {:a {:b {:c 1}}} [:a :b :c])
;; => {:a {:b {}}}

;; 存在しないパスは何もしない
(map/dissoc-in {:a 1} [:x :y :z])
;; => {:a 1}
```

**パイプラインで使う**:

```qi
;; センシティブ情報の削除
(user-data
 |> (map/dissoc-in _ [:credentials :password])
 |> (map/dissoc-in _ [:private :ssn]))
```

---

## キー・値の一括変換

### update-keys - マップのすべてのキーに関数を適用

マップのすべてのキーに関数を適用し、新しいマップを返します。

```qi
;; 基本的な使い方
(map/update-keys str/upper {:name "Alice" :age 30})
;; => {"NAME" "Alice", "AGE" 30}

;; キーの正規化
(map/update-keys str/lower {:Name "Alice" :AGE 30})
;; => {"name" "Alice", "age" 30}

;; キーの変換
(map/update-keys (fn [k] (str "prefix_" k)) {:a 1 :b 2})
;; => {"prefix_a" 1, "prefix_b" 2}

;; キーワードから文字列へ
(map/update-keys name {:name "Alice" :age 30})
;; => {"name" "Alice", "age" 30}
```

**パイプラインで使う**:

```qi
;; JSONキーの正規化
(json-data
 |> (map/update-keys _ str/lower)
 |> (map/update-keys _ (fn [k] (str/replace k "-" "_"))))

;; データベースカラム名の変換
(db-record
 |> (map/update-keys _ str/snake))
```

### update-vals - マップのすべての値に関数を適用

マップのすべての値に関数を適用し、新しいマップを返します。

```qi
;; 基本的な使い方
(map/update-vals inc {:a 1 :b 2})
;; => {:a 2, :b 3}

;; 値の型変換
(map/update-vals str {:a 1 :b 2 :c 3})
;; => {:a "1", :b "2", :c "3"}

;; 値の正規化
(map/update-vals str/trim {:name "  Alice  " :city "  Tokyo  "})
;; => {:name "Alice", :city "Tokyo"}

;; 数値の変換
(map/update-vals (fn [x] (* x 2)) {:a 10 :b 20 :c 30})
;; => {:a 20, :b 40, :c 60}
```

**パイプラインで使う**:

```qi
;; 価格の計算
(prices
 |> (map/update-vals _ (fn [p] (* p 1.1))))  ;; 10%増

;; データの正規化
(form-data
 |> (map/update-vals _ str/trim)
 |> (map/update-vals _ str/lower))
```

---

## 実用例

### 設定マージ

```qi
;; デフォルト設定とユーザー設定をマージ
(def default-config
  {:server {:host "localhost" :port 8080}
   :db {:host "localhost" :port 5432}
   :cache {:ttl 3600}})

(def user-config
  {:server {:port 3000}
   :db {:host "db.example.com"}})

;; 段階的なマージ
(default-config
 |> (map/assoc-in _ [:server :port] (get-in user-config [:server :port]))
 |> (map/assoc-in _ [:db :host] (get-in user-config [:db :host])))
```

### APIレスポンスの整形

```qi
;; 必要なフィールドのみ抽出してキーを変換
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

### データベースレコードの変換

```qi
;; データベースから取得したレコードをフロントエンド用に変換
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

### センシティブ情報の削除

```qi
;; ログ出力前にセンシティブ情報を削除
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

### 設定の動的構築

```qi
;; 環境変数から設定を構築
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

## 関数一覧

### キー選択
- `map/select-keys` - 指定したキーのみ抽出

### ネスト操作
- `map/assoc-in` - ネストしたマップに値を設定
- `map/dissoc-in` - ネストしたマップからキーを削除

### 一括変換
- `map/update-keys` - すべてのキーに関数を適用
- `map/update-vals` - すべての値に関数を適用

---

## パフォーマンスノート

- すべての関数は新しいマップを返します（イミュータブル）
- `update-keys`と`update-vals`は関数を各要素に適用するため、大きなマップでは処理時間がかかります
- `assoc-in`はネストした構造を再帰的にコピーするため、深いネストでは処理コストが高くなります
- 頻繁に更新する場合は、アトム（`atom`）やエージェント（`agent`）の利用を検討してください
