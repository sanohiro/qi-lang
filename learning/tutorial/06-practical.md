# 実践的なプログラミング

ファイルI/O、HTTP通信、データベースなど実用的な機能について学びます。

## ファイルI/O

### ファイルの読み書き

```qi
; ファイル全体を読み込む
(def content (io/read-file "data.txt"))
; => "ファイルの内容"

; ファイルに書き込む
(io/write-file "output.txt" "Hello, World!")

; 追記
(io/append-file "log.txt" "New entry\n")

; 行ごとに読み込む
(def lines (io/read-lines "data.txt"))
; => ["line1" "line2" "line3"]
```

### ファイルの存在確認

```qi
; ファイルが存在するか
(io/file-exists? "data.txt")    ; => true

; ファイルかディレクトリか
(io/is-file? "data.txt")        ; => true
(io/is-dir? "mydir")            ; => true
```

### ディレクトリ操作

```qi
; ディレクトリ内のファイル一覧
(io/list-dir ".")
; => ["file1.txt" "file2.txt" "subdir"]

; ディレクトリ作成
(io/create-dir "newdir")

; ディレクトリ削除
(io/delete-dir "olddir")

; ファイル削除
(io/delete-file "temp.txt")
```

### ファイル操作

```qi
; ファイルコピー
(io/copy-file "source.txt" "dest.txt")

; ファイル移動
(io/move-file "old.txt" "new.txt")

; ファイル情報
(io/file-info "data.txt")
; => {:size 1024 :modified 1234567890 ...}
```

### パス操作

```qi
; パスの結合
(path/join "dir" "subdir" "file.txt")
; => "dir/subdir/file.txt"

; ベース名
(path/basename "/path/to/file.txt")
; => "file.txt"

; ディレクトリ名
(path/dirname "/path/to/file.txt")
; => "/path/to"

; 拡張子
(path/extension "file.txt")
; => "txt"

; 拡張子なしの名前
(path/stem "file.txt")
; => "file"

; 絶対パス
(path/absolute "relative/path")
; => "/full/path/to/relative/path"
```

### 実用例: ログファイル処理

```qi
(defn process-logs [log-file]
  (do
    ; ログファイルを行ごとに読み込む
    (def lines (io/read-lines log-file))

    ; エラー行のみ抽出
    (def errors
      (filter (fn [line] (str/contains? line "ERROR")) lines))

    ; エラーをファイルに保存
    (io/write-file "errors.txt"
                   (str/join "\n" errors))

    ; エラー数を返す
    (len errors)))

(process-logs "app.log")
; => 42
```

## コマンドライン引数

### 引数の取得

```qi
; すべての引数
(args/all)
; => ["arg1" "arg2" "arg3"]

; n番目の引数（0始まり）
(args/get 0)
; => "arg1"

; 引数の数
(args/count)
; => 3
```

### 引数のパース

```qi
; オプション付き引数のパース
(def parsed (args/parse))
; 実行例: qi script.qi --name Alice --age 25 file.txt
; => {:flags {:name "Alice" :age "25"}
;     :args ["file.txt"]}

; 使用例
(defn main []
  (def opts (args/parse))
  (def name (get-in opts [:flags :name] "Guest"))
  (def age (get-in opts [:flags :age] "0"))

  (println f"Hello, {name}! You are {age} years old."))

(main)
```

## 環境変数

### 環境変数の読み書き

```qi
; 環境変数の取得
(env/get "HOME")
; => "/Users/username"

; デフォルト値付き
(def port (env/get "PORT" "8080"))

; 環境変数の設定
(env/set "MY_VAR" "value")

; すべての環境変数
(env/all)
; => {:HOME "/Users/username" :PATH "/usr/bin:..." ...}
```

### .envファイルの読み込み

```qi
; .envファイルを読み込んで環境変数に設定
(env/load-dotenv ".env")

; .envファイルの例:
; DATABASE_URL=postgres://localhost/mydb
; API_KEY=secret123
```

## HTTP通信

### HTTPクライアント

```qi
; GETリクエスト
(def response (http/get "https://api.example.com/users"))
; => {:status 200 :body "{\"users\":[...]}" :headers {...}}

; POSTリクエスト
(def response
  (http/post "https://api.example.com/users"
             {:headers {:content-type "application/json"}
              :body (json/stringify {:name "Alice" :age 25})}))

; その他のメソッド
(http/put url options)
(http/delete url options)
(http/patch url options)
(http/head url)
(http/options url)
```

### レスポンスの処理

```qi
(defn fetch-users []
  (def response (http/get "https://api.example.com/users"))

  (if (= (:status response) 200)
    ; 成功時: JSONをパース
    (json/parse (:body response))
    ; エラー時
    (error f"HTTP error: {(:status response)}")))

(fetch-users)
; => {:users [{:name "Alice" :age 25} ...]}
```

### HTTPサーバー

```qi
; シンプルなサーバー
(defn handle-request [req]
  (match (:method req)
    "GET" -> (server/ok "Hello, World!")
    _ -> (server/not-found)))

(server/serve {:port 8080 :handler handle-request})
```

### ルーター

```qi
; ルーターを使う
(def router
  (server/router
    [["GET" "/" (fn [req] (server/ok "Home"))]
     ["GET" "/about" (fn [req] (server/ok "About"))]
     ["POST" "/api/users" (fn [req]
       (def user (server/with-json-body req))
       (server/json {:created true :user user}))]]))

(server/serve {:port 8080 :handler router})
```

### ミドルウェア

```qi
; ミドルウェアの適用
(def app
  (-> router
      (server/with-logging)
      (server/with-cors)
      (server/with-compression)))

(server/serve {:port 8080 :handler app})

; 認証
(def protected-handler
  (server/with-basic-auth handler "username" "password"))

; キャッシュ制御
(def cached-handler
  (server/with-cache-control handler {:max-age 3600}))
```

### 静的ファイル配信

```qi
; ファイル配信
(defn handle [req]
  (match (:path req)
    "/" -> (server/static-file "index.html")
    _ -> (server/static-dir "public" req)))

(server/serve {:port 8080 :handler handle})
```

## データフォーマット

### JSON

```qi
; JSONパース
(def data (json/parse "{\"name\":\"Alice\",\"age\":25}"))
; => {:name "Alice" :age 25}

; JSON文字列化
(json/stringify {:name "Alice" :age 25})
; => "{\"name\":\"Alice\",\"age\":25}"

; 整形済みJSON
(json/pretty {:name "Alice" :age 25})
; => "{\n  \"name\": \"Alice\",\n  \"age\": 25\n}"
```

### YAML

```qi
; YAMLパース
(def config (yaml/parse (io/read-file "config.yaml")))

; YAML文字列化
(yaml/stringify {:database {:host "localhost" :port 5432}})
; => "database:\n  host: localhost\n  port: 5432\n"

; 整形済みYAML
(yaml/pretty config)
```

### CSV

```qi
; CSVパース
(def data (csv/parse "name,age\nAlice,25\nBob,30"))
; => [{:name "Alice" :age "25"}
;     {:name "Bob" :age "30"}]

; CSV文字列化
(csv/stringify [{:name "Alice" :age 25}
                {:name "Bob" :age 30}])
; => "name,age\nAlice,25\nBob,30\n"

; CSVファイルの読み書き
(csv/read-file "data.csv")
(csv/write-file "output.csv" data)
```

### 実用例: API + JSON

```qi
(defn fetch-and-save [url filename]
  (do
    ; APIからデータ取得
    (def response (http/get url))

    ; JSONパース
    (def data (json/parse (:body response)))

    ; 整形して保存
    (io/write-file filename (json/pretty data))

    (println f"Saved to {filename}")))

(fetch-and-save "https://api.example.com/data" "data.json")
```

## データベース

### 接続と基本操作

```qi
; データベースに接続
(def db (db/connect "sqlite:mydata.db"))

; テーブル作成
(db/exec db "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")

; データ挿入
(db/exec db "INSERT INTO users (name, age) VALUES ('Alice', 25)")
(db/exec db "INSERT INTO users (name, age) VALUES ('Bob', 30)")

; クエリ実行
(def users (db/query db "SELECT * FROM users"))
; => [{:id 1 :name "Alice" :age 25}
;     {:id 2 :name "Bob" :age 30}]

; 1行だけ取得
(def user (db/query-one db "SELECT * FROM users WHERE id = 1"))
; => {:id 1 :name "Alice" :age 25}

; 接続を閉じる
(db/close db)
```

### プリペアドステートメント（パラメータ化クエリ）

```qi
; 安全なクエリ（SQLインジェクション対策）
(def name (db/sanitize "Alice"))
(db/query db f"SELECT * FROM users WHERE name = {name}")

; LIKE句のエスケープ
(def pattern (db/escape-like "alice%"))
(db/query db f"SELECT * FROM users WHERE name LIKE {pattern}")
```

### トランザクション

```qi
; トランザクション開始
(db/begin db)

(try
  (do
    (db/exec db "INSERT INTO users (name, age) VALUES ('Charlie', 35)")
    (db/exec db "UPDATE users SET age = 26 WHERE name = 'Alice'")

    ; コミット
    (db/commit db))
  (fn [err]
    ; エラー時はロールバック
    (db/rollback db)
    (error err)))
```

### メタデータ取得

```qi
; テーブル一覧
(db/tables db)
; => ["users" "posts" "comments"]

; カラム一覧
(db/columns db "users")
; => [{:name "id" :type "INTEGER" ...}
;     {:name "name" :type "TEXT" ...}
;     {:name "age" :type "INTEGER" ...}]

; インデックス一覧
(db/indexes db "users")

; 外部キー一覧
(db/foreign-keys db "posts")
```

### 実用例: ユーザー管理

```qi
(defn create-user [db name age]
  (do
    (def safe-name (db/sanitize name))
    (db/exec db f"INSERT INTO users (name, age) VALUES ({safe-name}, {age})")
    (db/query-one db "SELECT last_insert_rowid() as id")))

(defn find-user [db id]
  (db/query-one db f"SELECT * FROM users WHERE id = {id}"))

(defn list-users [db]
  (db/query db "SELECT * FROM users ORDER BY name"))

; 使用例
(def db (db/connect "sqlite:app.db"))
(def user (create-user db "Alice" 25))
(println f"Created user with ID: {(:id user)}")
(db/close db)
```

## ログとデバッグ

### 構造化ログ

```qi
; ログレベル設定
(log/set-level :info)  ; :debug, :info, :warn, :error

; ログフォーマット設定
(log/set-format :json)  ; :text, :json

; ログ出力
(log/debug "Debug message")
(log/info "Application started")
(log/warn "Low memory")
(log/error "Database connection failed")

; 構造化ログ
(log/info {:event "user.login"
           :user "alice"
           :timestamp (timestamp)})
```

### プロファイリング

```qi
; プロファイリング開始
(profile/start)

; 測定したい処理
(defn slow-function []
  (loop [i 0 sum 0]
    (if (>= i 1000000)
      sum
      (recur (+ i 1) (+ sum i)))))

(slow-function)

; プロファイリング停止
(profile/stop)

; レポート出力
(profile/report)
; => {:functions [{:name "slow-function" :calls 1 :time 123.45} ...]}

; クリア
(profile/clear)
```

### 時間計測

```qi
; 関数実行時間の計測
(time (slow-function))
; 出力: Elapsed time: 123.45 ms
; => 結果
```

## 実践例

### 例1: Webスクレイピング

```qi
(defn scrape-titles [url]
  (do
    ; HTMLを取得
    (def response (http/get url))

    ; タイトルを抽出（簡易版）
    (def html (:body response))
    (def lines (str/lines html))

    ; <title>タグを含む行を探す
    (def title-line
      (find (fn [line] (str/contains? line "<title>")) lines))

    ; タイトルを抽出
    (if title-line
      (-> title-line
          (str/sub-after "<title>")
          (str/sub-before "</title>")
          str/trim)
      nil)))

(scrape-titles "https://example.com")
; => "Example Domain"
```

### 例2: RESTful APIサーバー

```qi
(def db (db/connect "sqlite:api.db"))

; データベース初期化
(db/exec db "CREATE TABLE IF NOT EXISTS posts (
  id INTEGER PRIMARY KEY,
  title TEXT,
  content TEXT,
  created_at INTEGER
)")

; ハンドラー
(defn handle-get-posts [req]
  (def posts (db/query db "SELECT * FROM posts ORDER BY created_at DESC"))
  (server/json posts))

(defn handle-create-post [req]
  (def body (server/with-json-body req))
  (def title (db/sanitize (:title body)))
  (def content (db/sanitize (:content body)))
  (def now (timestamp))

  (db/exec db f"INSERT INTO posts (title, content, created_at)
                VALUES ({title}, {content}, {now})")

  (server/json {:status "created"}))

(defn handle-get-post [req id]
  (def post (db/query-one db f"SELECT * FROM posts WHERE id = {id}"))
  (if post
    (server/json post)
    (server/not-found)))

; ルーター
(def router
  (server/router
    [["GET" "/posts" handle-get-posts]
     ["POST" "/posts" handle-create-post]
     ["GET" "/posts/:id" handle-get-post]]))

; サーバー起動
(server/serve {:port 3000
               :handler (-> router
                           (server/with-logging)
                           (server/with-cors))})

(log/info "Server started on port 3000")
```

### 例3: データETL処理

```qi
(defn etl-process []
  (do
    (log/info "ETL process started")

    ; Extract: CSVファイルから読み込み
    (def raw-data (csv/read-file "raw_data.csv"))
    (log/info f"Extracted {(len raw-data)} records")

    ; Transform: データ変換
    (def transformed
      (map (fn [row]
             {:id (to-int (:id row))
              :name (str/trim (str/upper (:name row)))
              :age (to-int (:age row))
              :email (str/lower (:email row))
              :processed_at (timestamp)})
           raw-data))

    ; データ検証
    (def valid-data
      (filter (fn [row]
                (and (> (:age row) 0)
                     (str/contains? (:email row) "@")))
              transformed))
    (log/info f"Validated {(len valid-data)} records")

    ; Load: データベースに保存
    (def db (db/connect "sqlite:warehouse.db"))
    (db/begin db)

    (try
      (do
        (map (fn [row]
               (db/exec db
                        "INSERT INTO users (id, name, age, email, processed_at)
                         VALUES (?, ?, ?, ?, ?)"
                        [(:id row) (:name row) (:age row)
                         (:email row) (:processed_at row)]))
             valid-data)
        (db/commit db)
        (log/info "ETL process completed successfully"))
      (fn [err]
        (db/rollback db)
        (log/error f"ETL process failed: {err}")))

    (db/close db)))

(etl-process)
```

### 例4: 設定ファイルの管理

```qi
(defn load-config [env]
  (do
    ; 環境ごとの設定ファイル
    (def config-file f"config/{env}.yaml")

    ; YAMLファイル読み込み
    (if (io/file-exists? config-file)
      (yaml/parse (io/read-file config-file))
      (error f"Config file not found: {config-file}"))))

(defn get-db-config [config]
  (get-in config [:database]))

; 使用例
(def env (env/get "APP_ENV" "development"))
(def config (load-config env))
(def db-config (get-db-config config))

(println f"Database: {(:host db-config)}:{(:port db-config)}")
```

### 例5: バッチ処理（並行）

```qi
(defn process-file [filename]
  (do
    (log/info f"Processing {filename}")
    (def content (io/read-file filename))
    (def lines (str/lines content))

    ; 何らかの処理
    (def result (len lines))

    (log/info f"Processed {filename}: {result} lines")
    {:file filename :lines result}))

(defn batch-process [directory]
  (do
    (log/info "Batch process started")

    ; ディレクトリ内のすべてのテキストファイル
    (def files
      (filter (fn [f] (str/ends-with? f ".txt"))
              (io/list-dir directory)))

    ; 並列処理
    (def results
      (pmap (fn [f] (process-file (path/join directory f)))
            files))

    (log/info f"Batch process completed: {(len results)} files")
    results))

(batch-process "data/input")
```

## コマンド実行

```qi
; シェルコマンドの実行
(def result (cmd/exec ["ls" "-la"]))
; => {:status 0 :stdout "..." :stderr ""}

; シェルで実行
(cmd/sh "echo Hello | grep Hello")
; => {:status 0 :stdout "Hello\n" :stderr ""}

; パイプライン
(cmd/pipe [["echo" "hello"]
           ["grep" "hello"]
           ["wc" "-l"]])
; => {:status 0 :stdout "1\n" :stderr ""}

; 行ごとに取得
(def lines (cmd/lines ["cat" "file.txt"]))
; => ["line1" "line2" "line3"]
```

## 一時ファイル

```qi
; 一時ファイル作成（自動削除）
(def temp (io/temp-file))
(io/write-file temp "temporary data")
; スコープ終了で自動削除

; 一時ファイル作成（保持）
(def temp (io/temp-file-keep))
(println temp)  ; => "/tmp/qi-XXXXXX"

; 一時ディレクトリ
(def temp-dir (io/temp-dir))
(io/write-file (path/join temp-dir "file.txt") "data")
```

## まとめ

実践的な機能：

- **ファイルI/O**: read-file, write-file, list-dir
- **コマンドライン**: args/parse, env/get
- **HTTP**: http/get, server/serve
- **データフォーマット**: json/parse, yaml/parse, csv/parse
- **データベース**: db/connect, db/query, db/exec
- **ログ**: log/info, profile/start
- **コマンド**: cmd/exec, cmd/sh

これらを組み合わせて実用的なアプリケーションを作成できます！

## 次のステップ

これでQi言語のチュートリアルは完了です。実際のプロジェクトを作成して、学んだことを活用しましょう。

さらに詳しい情報は：
- [実装ガイド](../impl/README.md) - Qiの内部実装を学ぶ
- [Rustガイド](../rust/README.md) - Rustの学習
- SPEC.md - 言語仕様の詳細
