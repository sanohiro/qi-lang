# 認証・認可

**JWT（JSON Web Token）とパスワードハッシュによる認証機能**

Qiは、モダンなWebアプリケーションで必要とされる認証機能を標準ライブラリとして提供します。

---

## 目次

- [概要](#概要)
- [JWT（JSON Web Token）](#jwtjson-web-token)
  - [jwt/sign - トークン生成](#jwtsign---トークン生成)
  - [jwt/verify - トークン検証](#jwtverify---トークン検証)
  - [jwt/decode - トークンデコード](#jwtdecode---トークンデコード)
- [パスワードハッシュ](#パスワードハッシュ)
  - [password/hash - パスワードハッシュ化](#passwordhash---パスワードハッシュ化)
  - [password/verify - パスワード検証](#passwordverify---パスワード検証)
- [実用例](#実用例)

---

## 概要

### 提供機能

- **JWT認証**: ステートレスなトークンベース認証
  - トークン生成（jwt/sign）
  - トークン検証（jwt/verify）
  - トークンデコード（jwt/decode）
  - HS256/HS384/HS512アルゴリズム対応

- **パスワードハッシュ**: Argon2による安全なパスワード管理
  - ハッシュ化（password/hash）
  - 検証（password/verify）
  - ソルト自動生成

### feature flag

```toml
# Cargo.toml
features = ["auth-jwt", "auth-password"]
```

デフォルトで有効です。

---

## JWT（JSON Web Token）

### jwt/sign - トークン生成

**ペイロードと秘密鍵からJWTトークンを生成します。**

```qi
(jwt/sign payload secret)
(jwt/sign payload secret algorithm)
(jwt/sign payload secret algorithm exp)
```

#### 引数

- `payload`: マップ（トークンに含めるデータ）
- `secret`: 文字列（署名用の秘密鍵）
- `algorithm`: 文字列（オプション、デフォルト: "HS256"）
  - "HS256", "HS384", "HS512" に対応
- `exp`: 整数（オプション、有効期限の秒数）

#### 戻り値

- 成功: `{:ok "トークン文字列"}`
- 失敗: `{:error "エラーメッセージ"}`

#### 使用例

```qi
;; 基本的な使い方（デフォルトはHS256）
(def result (jwt/sign {:user_id 123 :name "Alice"} "my-secret"))
;; => {:ok "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."}

;; アルゴリズムを指定
(jwt/sign {:role "admin"} "secret" "HS384")

;; 有効期限を指定（3600秒 = 1時間）
(jwt/sign {:user_id 42} "secret" "HS256" 3600)

;; パイプラインでの使用
({:user_id 999 :role "user"}
 |> (jwt/sign "my-secret")
 |>? (fn [token] (println "Token:" token)))
```

---

### jwt/verify - トークン検証

**JWTトークンを検証し、ペイロードを取り出します。**

```qi
(jwt/verify token secret)
(jwt/verify token secret algorithm)
```

#### 引数

- `token`: 文字列（検証するJWTトークン）
- `secret`: 文字列（検証用の秘密鍵）
- `algorithm`: 文字列（オプション、デフォルト: "HS256"）

#### 戻り値

- 成功: `{:ok ペイロードマップ}`
- 失敗: `{:error "エラーメッセージ"}`

#### 使用例

```qi
;; トークンを検証してペイロードを取得
(def result (jwt/verify token "my-secret"))
;; => {:ok {:user_id 123 :name "Alice"}}

;; 検証失敗の例
(jwt/verify "invalid-token" "secret")
;; => {:error "Invalid token"}

;; パイプラインでの使用
(token
 |>? (jwt/verify "my-secret")
 |>? (fn [payload] (get payload :user_id)))
;; => {:ok 123}
```

---

### jwt/decode - トークンデコード

**JWTトークンを検証せずにデコードします（署名検証なし）。**

トークンの内容を確認するためのデバッグ用関数です。

```qi
(jwt/decode token)
```

#### 引数

- `token`: 文字列（デコードするJWTトークン）

#### 戻り値

- 成功: `{:ok {:header ヘッダマップ :payload ペイロードマップ}}`
- 失敗: `{:error "エラーメッセージ"}`

#### 使用例

```qi
;; トークンをデコード（署名検証なし）
(def result (jwt/decode token))
;; => {:ok {:header {:typ "JWT" :alg "HS256"}
;;          :payload {:user_id 123 :name "Alice"}}}

;; ヘッダーだけ取得
(jwt/decode token
 |>? (fn [data] (get data :header)))
;; => {:ok {:typ "JWT" :alg "HS256"}}
```

⚠️ **注意**: この関数は署名を検証しません。デバッグやテスト用途にのみ使用してください。

---

## パスワードハッシュ

### password/hash - パスワードハッシュ化

**パスワードをArgon2でハッシュ化します。**

Argon2は、bcryptよりも高速で安全なパスワードハッシュアルゴリズムです。

```qi
(password/hash password)
```

#### 引数

- `password`: 文字列（ハッシュ化するパスワード）

#### 戻り値

- 成功: ハッシュ文字列（Value::String）
- 失敗: エラー（例外）

#### 使用例

```qi
;; パスワードをハッシュ化
(def hash (password/hash "my-password"))
;; => "$argon2id$v=19$m=19456,t=2,p=1$..."

;; 同じパスワードでも異なるハッシュが生成される（ソルト自動生成）
(def hash1 (password/hash "test"))
(def hash2 (password/hash "test"))
(println (= hash1 hash2))  ;; => false
```

---

### password/verify - パスワード検証

**パスワードとハッシュを比較して検証します。**

```qi
(password/verify password hash)
```

#### 引数

- `password`: 文字列（検証するパスワード）
- `hash`: 文字列（password/hashで生成されたハッシュ）

#### 戻り値

- `true`: パスワードが一致
- `false`: パスワードが不一致

#### 使用例

```qi
;; パスワードを検証
(def hash (password/hash "my-password"))

(password/verify "my-password" hash)  ;; => true
(password/verify "wrong-password" hash)  ;; => false

;; 不正なハッシュ形式の場合はfalseを返す
(password/verify "test" "invalid-hash")  ;; => false
```

---

## 実用例

### ユーザーログイン機能

```qi
;; ユーザー登録
(defn register-user [username password]
  (let [hash (password/hash password)]
    {:username username
     :password_hash hash}))

;; ユーザーログイン
(defn login [username password stored-hash]
  (if (password/verify password stored-hash)
    (jwt/sign {:username username} "app-secret" "HS256" 3600)
    {:error "Invalid credentials"}))

;; 使用例
(def user (register-user "alice" "secret123"))
;; => {:username "alice" :password_hash "$argon2id$..."}

(def token-result (login "alice" "secret123" (get user :password_hash)))
;; => {:ok "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."}
```

### APIの認証ミドルウェア

```qi
;; トークンから認証情報を取得
(defn authenticate [token]
  (token
   |>? (jwt/verify "app-secret")
   |>? (fn [payload] (get payload :username))))

;; 保護されたエンドポイント
(defn protected-endpoint [request]
  (let [auth-header (get-in request [:headers :authorization])
        token (string/replace-first auth-header "Bearer " "")
        auth-result (authenticate token)]
    (match auth-result
      {:ok username} -> {:status 200 :body (str "Hello, " username)}
      {:error _} -> {:status 401 :body "Unauthorized"})))
```

### パスワード変更フロー

```qi
;; パスワード変更
(defn change-password [old-password new-password stored-hash]
  (if (password/verify old-password stored-hash)
    (password/hash new-password)
    {:error "Current password is incorrect"}))

;; 使用例
(def current-hash "$argon2id$v=19$m=19456,t=2,p=1$...")
(def new-hash (change-password "old-pass" "new-pass" current-hash))
;; => "$argon2id$v=19$m=19456,t=2,p=1$..." (新しいハッシュ)
```

---

## セキュリティベストプラクティス

### JWT

1. **秘密鍵の管理**
   - 秘密鍵は環境変数で管理する
   - 十分に長く、ランダムな文字列を使用する

2. **有効期限の設定**
   - トークンには必ず有効期限を設定する
   - 短い有効期限（1時間以内）を推奨

3. **HTTPS必須**
   - トークンは必ずHTTPS経由で送信する
   - HTTPでは盗聴のリスクがある

### パスワードハッシュ

1. **ソルトの自動生成**
   - `password/hash`は自動的にソルトを生成します
   - 同じパスワードでも異なるハッシュが生成されます

2. **Argon2の利点**
   - bcryptより高速で安全
   - メモリハード関数（GPU攻撃に強い）
   - 2015年のPassword Hashing Competition優勝

3. **ハッシュの保存**
   - ハッシュはそのままデータベースに保存できます
   - アルゴリズム、パラメータ、ソルト、ハッシュが含まれています

---

## 関連ドキュメント

- **[11-stdlib-http.md](11-stdlib-http.md)** - HTTPサーバーとの統合
- **[12-stdlib-json.md](12-stdlib-json.md)** - JSONデータの扱い
- **[08-error-handling.md](08-error-handling.md)** - Result型パターン
- **[17-stdlib-database.md](17-stdlib-database.md)** - データベース統合

---

## 実装の詳細

### 使用クレート

- **jsonwebtoken** (v9.2) - JWT生成・検証
- **argon2** (v0.5) - パスワードハッシュ

### feature flags

```toml
# デフォルトで有効
default = ["auth-jwt", "auth-password", ...]

# 個別に無効化も可能
minimal = []  # 認証機能なし
```

---

## まとめ

Qiの認証ライブラリは、シンプルで安全な認証機能を提供します。

- **JWT**: ステートレスなトークン認証
- **Argon2**: モダンで安全なパスワードハッシュ
- **Result型**: エラー処理の統一

これらの機能を組み合わせることで、モダンなWebアプリケーションの認証システムを簡単に構築できます。
