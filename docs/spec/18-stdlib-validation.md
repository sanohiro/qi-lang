# 標準ライブラリ：バリデーション

**スキーマベースのデータ検証**

---

## 概要

Qiのバリデーション機能は、スキーマ（マップ）に基づいてデータを検証します。これにより、APIリクエスト、設定ファイル、ユーザー入力などのデータ検証を宣言的に行うことができます。

### 主な特徴

- **スキーマベース**: マップでスキーマを定義
- **Railway Oriented Programming**: エラーを値として返す（`{:ok value}` または `{:error ...}`）
- **型安全**: 複数の型（string, integer, number, boolean, map, vector, list, keyword, symbol, nil, any）をサポート
- **ネストしたデータ対応**: ネストしたマップやコレクションの検証が可能
- **詳細なエラーメッセージ**: エラーコード、メッセージ、フィールド名を含む

---

## 基本的な使い方

### 型チェック

```qi
;; 文字列型のチェック
(def schema {:type "string"})
(validate schema "hello")  ;; => {:ok "hello"}
(validate schema 123)      ;; => {:error {:code "type-mismatch" :message "stringである必要があります"}}

;; 整数型のチェック
(def schema {:type "integer"})
(validate schema 42)    ;; => {:ok 42}
(validate schema 3.14)  ;; => {:error {:code "type-mismatch" :message "integerである必要があります"}}
```

### 必須フィールドチェック

```qi
(def schema {:type "string" :required true})
(validate schema "test")  ;; => {:ok "test"}
(validate schema nil)     ;; => {:error {:code "required" :message "必須フィールドです"}}

;; オプショナルフィールド（デフォルト）
(def schema {:type "string"})
(validate schema nil)  ;; => {:ok nil}  ;; 成功（オプショナル）
```

---

## 文字列のバリデーション

### 長さチェック

```qi
(def username-schema
  {:type "string"
   :min-length 3
   :max-length 20})

(validate username-schema "user123")     ;; => {:ok "user123"}
(validate username-schema "ab")          ;; => {:error {:code "min-length" :message "3文字以上である必要があります"}}
(validate username-schema "verylongusernamethatexceedslimit")
  ;; => {:error {:code "max-length" :message "20文字以下である必要があります"}}
```

### パターンマッチング（正規表現）

```qi
;; メールアドレス検証
(def email-schema
  {:type "string"
   :pattern "^[^@]+@[^@]+\\.[^@]+$"})

(validate email-schema "user@example.com")  ;; => {:ok "user@example.com"}
(validate email-schema "invalid-email")     ;; => {:error {:code "pattern" :message "パターンに一致しません: ..."}}

;; 英小文字のみ
(def lowercase-schema
  {:type "string"
   :pattern "^[a-z]+$"})

(validate lowercase-schema "hello")    ;; => {:ok "hello"}
(validate lowercase-schema "Hello123") ;; => {:error {:code "pattern" ...}}
```

---

## 数値のバリデーション

### 範囲チェック

```qi
(def age-schema
  {:type "integer"
   :min 0
   :max 150})

(validate age-schema 25)   ;; => {:ok 25}
(validate age-schema -5)   ;; => {:error {:code "min-value" :message "0以上である必要があります"}}
(validate age-schema 200)  ;; => {:error {:code "max-value" :message "150以下である必要があります"}}
```

### 正の数チェック

```qi
(def price-schema
  {:type "number"
   :positive true})

(validate price-schema 100.0)  ;; => {:ok 100.0}
(validate price-schema 0)      ;; => {:error {:code "positive" :message "正の数である必要があります"}}
(validate price-schema -10.5)  ;; => {:error {:code "positive" ...}}
```

---

## コレクションのバリデーション

### 要素数チェック

```qi
(def tags-schema
  {:type "vector"
   :min-items 1
   :max-items 5})

(validate tags-schema ["tag1" "tag2" "tag3"])  ;; => {:ok ["tag1" "tag2" "tag3"]}
(validate tags-schema [])                      ;; => {:error {:code "min-items" :message "1個以上である必要があります"}}
(validate tags-schema ["a" "b" "c" "d" "e" "f"])
  ;; => {:error {:code "max-items" :message "5個以下である必要があります"}}
```

---

## ネストしたマップのバリデーション

`:fields` を使って、マップ内の各フィールドにスキーマを定義できます。

```qi
(def user-schema
  {:type "map"
   :fields {:name {:type "string" :required true :min-length 1}
            :age {:type "integer" :min 0 :max 150}
            :email {:type "string" :pattern "^[^@]+@[^@]+\\.[^@]+$"}}})

;; 正常なデータ
(validate user-schema
  {:name "太郎" :age 25 :email "taro@example.com"})
;; => {:ok {:name "太郎" :age 25 :email "taro@example.com"}}

;; 空の名前（min-lengthエラー）
(validate user-schema {:name "" :age 25})
;; => {:error {:field ":name" :code "min-length" :message "1文字以上である必要があります"}}

;; 名前が欠落（requiredエラー）
(validate user-schema {:age 25})
;; => {:error {:field ":name" :code "required" :message "必須フィールドです"}}

;; 年齢が範囲外
(validate user-schema {:name "太郎" :age 200})
;; => {:error {:field ":age" :code "max-value" :message "150以下である必要があります"}}

;; オプショナルフィールド（email）は省略可能
(validate user-schema {:name "太郎"})
;; => {:ok {:name "太郎"}}  ;; age, emailはnil（オプショナル）
```

---

## 複雑なスキーマ例

### ユーザー登録フォーム

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
  {:username "ab"        ;; 短すぎる
   :password "short"     ;; 短すぎる
   :email "invalid"})    ;; パターン不一致
;; => {:error {:field ":username" :code "min-length" ...}}  ;; 最初のエラーを返す
```

---

## スキーマオプション一覧

### 共通オプション

| オプション | 型 | 説明 |
|-----------|-----|------|
| `:type` | string | データ型: `"string"`, `"integer"`, `"number"`, `"boolean"`, `"map"`, `"vector"`, `"list"`, `"keyword"`, `"symbol"`, `"nil"`, `"any"` |
| `:required` | bool | 必須フィールドかどうか（デフォルト: `false`） |

### 文字列専用

| オプション | 型 | 説明 |
|-----------|-----|------|
| `:min-length` | integer | 最小文字数 |
| `:max-length` | integer | 最大文字数 |
| `:pattern` | string | 正規表現パターン |

### 数値専用

| オプション | 型 | 説明 |
|-----------|-----|------|
| `:min` | integer/float | 最小値 |
| `:max` | integer/float | 最大値 |
| `:positive` | bool | 正の数かどうか（0より大きい） |
| `:integer` | bool | 整数かどうか（floatの場合） |

### コレクション専用

| オプション | 型 | 説明 |
|-----------|-----|------|
| `:min-items` | integer | 最小要素数 |
| `:max-items` | integer | 最大要素数 |

### マップ専用

| オプション | 型 | 説明 |
|-----------|-----|------|
| `:fields` | map | ネストしたフィールドのスキーマ（マップ） |

---

## エラーコード一覧

| コード | 説明 |
|--------|------|
| `type-mismatch` | 型が一致しない |
| `required` | 必須フィールドが欠落 |
| `min-length` | 文字列が短すぎる |
| `max-length` | 文字列が長すぎる |
| `pattern` | パターンに一致しない |
| `min-value` | 数値が小さすぎる |
| `max-value` | 数値が大きすぎる |
| `positive` | 正の数でない |
| `min-items` | 要素数が少なすぎる |
| `max-items` | 要素数が多すぎる |

---

## 使用例：HTTPサーバーでの検証

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

## 注意事項

- **オプショナルフィールド**: `:required` が `false` または未指定の場合、データが `nil` でも検証は成功します
- **:type "any"**: 任意の型を受け入れます（型チェックなし）
- **ネストしたマップ**: エラーが発生したフィールドは `:field` に含まれます（例: `":name"`, `":address:city"`）
- **正規表現**: ripgrep の構文に従います

---

## 関連ドキュメント

- [エラー処理](08-error-handling.md) - Railway Oriented Programming
- [データ構造](06-data-structures.md) - マップとキーワード
- [HTTPサーバー](11-stdlib-http.md) - サーバーでのバリデーション活用
