# 標準ライブラリ - 環境変数（env/）

環境変数の取得、設定、.envファイルの読み込みなどを行う関数群です。

## 概要

`env/` モジュールは以下の機能を提供します：

- **環境変数の取得** - システム環境変数の読み取り
- **環境変数の設定** - プロセス内での環境変数の設定
- **全環境変数の取得** - すべての環境変数をマップとして取得
- **.envファイルの読み込み** - dotenv形式のファイルから環境変数を一括設定

---

## 環境変数の取得

### env/get

環境変数を取得します。変数が存在しない場合、オプションでデフォルト値を返すことができます。

```qi
(env/get key)
(env/get key default-value)
```

**引数:**
- `key` (string) - 環境変数名
- `default-value` (any, optional) - 変数が存在しない場合の戻り値

**戻り値:**
- string - 環境変数の値
- default-value - 変数が存在せず、デフォルト値が指定されている場合
- nil - 変数が存在せず、デフォルト値が指定されていない場合

**例:**

```qi
;; 環境変数を取得
(env/get "PATH")
;; => "/usr/local/bin:/usr/bin:/bin"

;; 存在しない変数（nilを返す）
(env/get "NONEXISTENT_VAR")
;; => nil

;; デフォルト値を指定
(env/get "MISSING_VAR" "default-value")
;; => "default-value"

;; 既存の変数（デフォルト値は無視される）
(env/get "PATH" "fallback")
;; => "/usr/local/bin:/usr/bin:/bin"
```

---

## 環境変数の設定

### env/set

環境変数を設定します。現在のプロセスとその子プロセスに影響します。

```qi
(env/set key value)
```

**引数:**
- `key` (string) - 環境変数名
- `value` (string | number | boolean) - 設定する値（文字列に変換されます）

**戻り値:** nil

**例:**

```qi
;; 文字列を設定
(env/set "MY_VAR" "my-value")
(env/get "MY_VAR")
;; => "my-value"

;; 数値を設定（文字列に変換される）
(env/set "PORT" 8080)
(env/get "PORT")
;; => "8080"

;; 真偽値を設定
(env/set "DEBUG" true)
(env/get "DEBUG")
;; => "true"

(env/set "ENABLED" false)
(env/get "ENABLED")
;; => "false"
```

---

## 全環境変数の取得

### env/all

すべての環境変数をマップとして取得します。

```qi
(env/all)
```

**引数:** なし

**戻り値:** map - キーが環境変数名、値が環境変数の値

**例:**

```qi
;; すべての環境変数を取得
(def env-vars (env/all))

;; 特定の変数を取得
(get env-vars "HOME")
;; => "/home/user"

;; 環境変数の数をカウント
(count (env/all))
;; => 42

;; PATH で始まる環境変数を抽出
(env/all
 |> keys
 |> (filter (fn [k] (str/starts-with? k "PATH"))))
;; => ["PATH" "PATHEXT"]
```

---

## .envファイルの読み込み

### env/load-dotenv

.envファイルを読み込んで環境変数に設定します。dotenv形式（`KEY=VALUE`）をサポートします。

```qi
(env/load-dotenv)
(env/load-dotenv path)
```

**引数:**
- `path` (string, optional) - .envファイルのパス（デフォルト: ".env"）

**戻り値:** integer - 読み込まれた環境変数の数

**例:**

```qi
;; デフォルトの .env を読み込み
(env/load-dotenv)
;; => 5

;; カスタムパスを指定
(env/load-dotenv ".env.local")
;; => 3

;; 開発環境用の設定
(env/load-dotenv ".env.development")
;; => 8
```

### .envファイルの形式

`.env` ファイルは以下の形式をサポートします：

```bash
# コメント行（#で始まる）
KEY=VALUE

# 値のクオート（ダブルまたはシングル）
DATABASE_URL="postgresql://localhost/mydb"
API_KEY='secret-key-123'

# クオートなし
PORT=8080
DEBUG=true

# 空白は自動的にトリム
  TRIM_ME  =  value with spaces

# 空行は無視される

NODE_ENV=production
```

**サポート機能:**
- ✅ 基本的な `KEY=VALUE` 形式
- ✅ コメント行（`#` で始まる）
- ✅ 値のクオート（`"..."` または `'...'`）
- ✅ キーと値の前後の空白を自動トリム
- ✅ 空行を無視

**制限事項:**
- ❌ 変数展開（`${VAR}` 形式）は未サポート
- ❌ 複数行の値は未サポート
- ❌ エスケープシーケンス（`\n`, `\t` など）は未サポート

---

## 使用例

### 設定の読み込み

```qi
;; アプリケーション起動時に .env を読み込み
(defn load-config []
  (let [count (env/load-dotenv)]
    (println f"Loaded {count} environment variables")
    {:port (str/parse-int (env/get "PORT" "8080"))
     :host (env/get "HOST" "localhost")
     :debug (= (env/get "DEBUG") "true")}))

(def config (load-config))
;; Loaded 5 environment variables
;; => {:port 8080 :host "localhost" :debug true}
```

### 環境ごとの設定ファイル

```qi
;; NODE_ENV に応じて異なる .env を読み込み
(defn load-env-for-node-env []
  (let [node-env (env/get "NODE_ENV" "development")
        env-file (str ".env." node-env)]
    (println f"Loading {env-file}...")
    (env/load-dotenv env-file)))

(load-env-for-node-env)
```

### データベース接続文字列の取得

```qi
;; 環境変数からデータベース設定を構築
(defn get-db-config []
  (let [db-url (env/get "DATABASE_URL")]
    (if db-url
        ;; DATABASE_URL が設定されている場合はそれを使用
        {:url db-url}
        ;; なければ個別の環境変数から構築
        {:host (env/get "DB_HOST" "localhost")
         :port (str/parse-int (env/get "DB_PORT" "5432"))
         :database (env/get "DB_NAME" "myapp")
         :user (env/get "DB_USER" "postgres")
         :password (env/get "DB_PASSWORD" "")})))

(def db-config (get-db-config))
```

### 環境変数の一覧表示

```qi
;; すべての環境変数を整形して表示
(defn print-env-vars []
  (env/all
   |> (map (fn [[k v]] f"{k} = {v}"))
   |> sort
   |> (each println)))

(print-env-vars)
;; HOME = /home/user
;; PATH = /usr/local/bin:/usr/bin
;; SHELL = /bin/zsh
;; ...
```

### セキュリティ: 機密情報のマスク

```qi
;; 機密情報を含む環境変数をマスクして表示
(def sensitive-keys ["PASSWORD" "SECRET" "KEY" "TOKEN"])

(defn mask-value [key value]
  (if (any? (fn [s] (str/contains? (str/upper key) s)) sensitive-keys)
      "********"
      value))

(defn print-env-safe []
  (env/all
   |> (map (fn [[k v]] [k (mask-value k v)]))
   |> (each (fn [[k v]] (println f"{k} = {v}")))))

(print-env-safe)
;; API_KEY = ********
;; DB_PASSWORD = ********
;; USER = alice
```

### 設定の検証

```qi
;; 必須の環境変数が設定されているかチェック
(defn validate-env [required-vars]
  (let [missing (filter (fn [var] (nil? (env/get var))) required-vars)]
    (if (empty? missing)
        :ok
        {:error "Missing environment variables" :missing missing})))

(validate-env ["PORT" "DATABASE_URL" "API_KEY"])
;; => {:error "Missing environment variables" :missing ["API_KEY"]}
```

### .envファイルのバックアップ

```qi
;; 現在の環境変数を .env 形式でファイルに保存
(defn save-env-to-file [path keys]
  (let [lines (map (fn [k]
                     (let [v (env/get k)]
                       (if v
                           f"{k}={v}"
                           nil)))
                   keys)
        content (join "\n" (filter some? lines))]
    (io/write path content)))

;; 特定のキーのみをバックアップ
(save-env-to-file ".env.backup" ["PORT" "HOST" "DEBUG"])
```

---

## エラーハンドリング

### ファイルが見つからない場合

```qi
(try
  (env/load-dotenv "nonexistent.env")
  :ok
  (fn [err]
    (println "Error:" err)
    :error))
;; Error: Failed to read .env file 'nonexistent.env': No such file or directory
;; => :error
```

### 不正なファイル形式

```qi
;; .env ファイルに `KEY=VALUE` 形式でない行がある場合
;; エラーメッセージに行番号と内容が含まれます

;; invalid.env の内容:
;; KEY1=VALUE1
;; INVALID LINE WITHOUT EQUALS
;; KEY2=VALUE2

(try
  (env/load-dotenv "invalid.env")
  :ok
  (fn [err]
    (println err)))
;; Invalid format in .env file at line 2: INVALID LINE WITHOUT EQUALS
```

---

## パフォーマンスとベストプラクティス

### 起動時に1度だけ読み込む

```qi
;; ❌ 悪い例: リクエストごとに読み込む
(defn handle-request [req]
  (env/load-dotenv)  ;; 毎回ファイルI/Oが発生
  (let [api-key (env/get "API_KEY")]
    ...))

;; ✅ 良い例: 起動時に1度だけ読み込む
(env/load-dotenv)

(defn handle-request [req]
  (let [api-key (env/get "API_KEY")]  ;; メモリから取得
    ...))
```

### 設定オブジェクトにキャッシュ

```qi
;; ✅ 起動時に設定を読み込んでキャッシュ
(env/load-dotenv)

(def config
  {:port (str/parse-int (env/get "PORT" "8080"))
   :host (env/get "HOST" "localhost")
   :db-url (env/get "DATABASE_URL")
   :api-key (env/get "API_KEY")})

;; 以降は config から取得
(defn start-server []
  (http/serve (:port config) (:host config) handler))
```

### デフォルト値の設定

```qi
;; ❌ 悪い例: nilチェックを繰り返す
(let [port (env/get "PORT")]
  (if (nil? port)
      8080
      (str/parse-int port)))

;; ✅ 良い例: env/get でデフォルト値を指定
(str/parse-int (env/get "PORT" "8080"))
```

---

## 関連項目

- [ファイルI/O](13-stdlib-io.md) - ファイルの読み書き
- [文字列操作](10-stdlib-string.md) - 環境変数の解析
- [エラー処理](08-error-handling.md) - try/catchによるエラーハンドリング
- [モジュールシステム](09-modules.md) - 設定の管理
