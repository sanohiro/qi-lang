# モジュールシステム

**ファイル単位での名前空間分離**

Qiではファイル単位でモジュールを構成し、名前空間を分離します。

---

## 基本概念

- **探索はファイル名、表示はmodule名**: `use`の探索はファイル名、モジュール名は`module`宣言で変更可
- **use が自動ロード**: `use`でファイルを読み込み＋インポート
- **load は副作用のみ**: 設定ファイル等を実行するだけ
- **exportなし = 全公開**: `export`宣言がなければ全て公開（`defn-`除く）
- **exportあり = 選択公開**: `export`宣言があれば、明示したもののみ公開
- **defn- は完全非公開**: `export`に関係なく常に非公開

---

## module宣言

`module`は**モジュールの表示名**を指定します（オプショナル）。

### 基本的な使い方

```qi
;; http.qi
(module web-api)  ;; 表示名を 'web-api' にする

(defn get [url] ...)

;; 探索は 'http.qi' で行われる
;; アクセスは 'web-api/get' になる
```

### 仕様

- **オプショナル**: なければファイル名（basename）がモジュール名
- **探索には影響しない**: `use http` は `http.qi` を探す（module名ではない）
- **表示名のみ変更**: アクセスする際のプレフィックスが変わる
- **1ファイル1回のみ**: 複数の`module`宣言はエラー
- **位置**: ファイル先頭推奨（技術的にはどこでもOK）
- **戻り値**: `nil`

### 例

```qi
;; http.qi
(module web-api)

;; 使う側
(use http)              ;; http.qi を探す → 成功
(web-api/get "...")     ;; OK（module名でアクセス）
(http/get "...")        ;; Error（ファイル名ではアクセスできない）
```

---

## export宣言

`export`は**公開するシンボル**を制御します（オプショナル）。

### モードA: export宣言なし → 全て公開（デフォルト）

```qi
;; utils.qi
(defn add [a b] (+ a b))        ;; 公開（defnなので）
(defn multiply [a b] (* a b))   ;; 公開（defnなので）
(defn- helper [x] (* x 2))      ;; 非公開（defn-なので）

;; 全てのdefnが自動的に公開される
```

### モードB: export宣言あり → 選択公開

```qi
;; utils.qi
(defn add [a b] (+ a b))        ;; 非公開（exportにない）
(defn multiply [a b] (* a b))   ;; 公開（exportにある）
(defn- helper [x] (* x 2))      ;; 非公開（常に）

(export multiply)  ;; multiplyのみ公開
```

### 仕様

- **デフォルト公開**: `export`なし → 全`defn`が公開、全`defn-`が非公開
- **選択公開**: `export`あり → 明示したもののみ公開、それ以外は非公開
- **defn-は常に非公開**: `export`に書いてもエラー
- **複数宣言可**: 累積される
- **位置**: どこでもOK（末尾推奨）
- **戻り値**: `nil`

### 複数のexport宣言

```qi
(defn get [url] ...)
(defn post [url data] ...)
(defn put [url data] ...)

(export get)        ;; get のみ公開
(export post put)   ;; post, put も追加（累積）
;; 結果: get, post, put が全て公開
```

---

## モジュール定義の例

### パターン1: シンプル（exportなし）

```qi
;; math.qi
(defn add [a b] (+ a b))        ;; 公開
(defn sub [a b] (- a b))        ;; 公開
(defn- validate [x] (> x 0))    ;; 非公開

;; 外部から: (math/add ...), (math/sub ...) OK
;;          (math/validate ...) Error
```

### パターン2: 明示的export

```qi
;; math.qi
(defn add [a b] (+ a b))        ;; 公開（exportにある）
(defn sub [a b] (- a b))        ;; 非公開（exportにない）
(defn multiply [a b] (* a b))   ;; 非公開（exportにない）
(defn- validate [x] (> x 0))    ;; 非公開（常に）

(export add)  ;; addのみ公開

;; 外部から: (math/add ...) OK
;;          (math/sub ...), (math/multiply ...) Error
```

### パターン3: module宣言 + export

```qi
;; http.qi
(module web-client)  ;; 表示名を変更

(defn- build-url [base path] (str base "/" path))
(defn get [url] ...)
(defn post [url data] ...)
(defn internal-func [x] ...)  ;; exportしないので非公開

(export get post)

;; 外部から:
;; (use http)  ;; http.qi を探す
;; (web-client/get ...) OK（module名でアクセス）
;; (web-client/post ...) OK
;; (web-client/internal-func ...) Error（非公開）
```

---

## インポート（use）

`use`は**ファイルの読み込み＋シンボルのインポート**を行います。

### パターン1: 特定の関数のみインポート（推奨）

```qi
(use http :only [get post])
(get "https://...")                      ;; OK
```

### パターン2: エイリアス（module/function形式）

```qi
(use http :as h)
(h/get "https://...")                    ;; OK
```

### パターン3: 全てインポート

```qi
(use http :all)
(get "https://...")                      ;; OK
(post "https://..." {:data 123})         ;; OK
```

### パターン4: パス指定

```qi
(use "lib/utils" :only [format-date])    ;; lib/utils.qi を読み込み
(use "./vendor/json" :as json)           ;; 相対パス
```

---

## モジュール名の決定ルール

**探索はファイル名、アクセスはmodule名**の原則:

### ケース1: モジュール名のみ → 自動探索

```qi
(use http :only [get])
;; 1. http.qi を探索（./http.qi, ~/.qi/modules/http.qi 等）
;; 2. http.qi 内の module宣言を確認
;;    - (module web-api) → アクセスは web-api/get
;;    - なし → アクセスは http/get
```

### ケース2: パス指定 → basename で探索

```qi
(use "lib/http" :only [get])
;; 1. lib/http.qi を読み込み
;; 2. lib/http.qi 内の module宣言を確認
;;    - (module web-api) → アクセスは web-api/get
;;    - なし → アクセスは http/get (basename)
```

### ケース3: :as でエイリアス

```qi
(use "lib/http" :as h)
;; 1. lib/http.qi を読み込み
;; 2. module宣言に関係なく、エイリアス 'h' を使用
;; => アクセスは h/get
```

### ケース4: module宣言との組み合わせ

```qi
;; lib/http.qi
(module web-client)
(defn get [url] ...)

;; 使う側
(use "lib/http")
(web-client/get "...")  ;; OK（module名でアクセス）

(use "lib/http" :as h)
(h/get "...")           ;; OK（エイリアスが優先）
```

---

## 名前衝突の処理

```qi
;; lib1/utils.qi → (module string-utils)
;; lib2/utils.qi → (module string-utils)  ← 同じmodule名！

(use "lib1/utils")  ;; モジュール名: string-utils
(use "lib2/utils")  ;; Error: module 'string-utils' already loaded

;; 解決策1: :as でエイリアス
(use "lib1/utils" :as utils1)
(use "lib2/utils" :as utils2)

;; 解決策2: ファイルのmodule宣言を変更
;; lib2/utils.qi を (module text-utils) に変更
```

---

## load（副作用のみ実行）

`load`はファイルを評価するだけで、シンボルはインポートしません。
設定ファイルや初期化スクリプトの実行に使用します。

```qi
;; 設定ファイルを読み込み（副作用のみ）
(load "config.qi")

;; useとの違い
(use http)    ;; http.qi を読み込み＋シンボルをインポート
(load "init") ;; init.qi を評価するだけ（シンボルはインポートしない）
```

---

## 公開/非公開の決定フロー

```
関数定義の公開/非公開:

defn- で定義
  → 常に非公開（exportできない）

defn で定義
  → export宣言が存在するか？
      YES → exportリストに含まれるか？
              YES → 公開
              NO  → 非公開
      NO  → 公開（デフォルト）
```

---

## 実用例

### ライブラリの作成

```qi
;; lib/string-utils.qi
(module string-utils)

;; 公開関数
(defn upper [s]
  (str/upper s))

(defn lower [s]
  (str/lower s))

;; 内部関数（非公開）
(defn- validate [s]
  (and (string? s) (> (len s) 0)))

;; 明示的にexport
(export upper lower)
```

### ライブラリの使用

```qi
;; main.qi
(use "lib/string-utils" :only [upper lower])

(upper "hello")  ;; => "HELLO"
(lower "WORLD")  ;; => "world"

;; または
(use "lib/string-utils" :as str-util)
(str-util/upper "hello")  ;; => "HELLO"
```

### 設定ファイルの読み込み

```qi
;; config.qi
(def api-key "secret-key")
(def db-host "localhost")

;; main.qi
(load "config")  ;; 副作用のみ（シンボルはインポートされない）

;; api-keyやdb-hostはグローバルに定義されている
(println api-key)  ;; => "secret-key"
```

---

## 初期化ファイル（.qi/init.qi）

**REPLおよびワンライナー実行時の自動ロード**

Qiは、REPLおよびワンライナー（`-e`オプション）実行時に、初期化ファイルを自動的にロードします。

### 読み込み順序

```
1. ~/.qi/init.qi  （ユーザーグローバル設定・優先）
2. ./.qi/init.qi  （プロジェクトローカル設定）
```

- 両方のファイルが存在する場合、**両方ロード**されます
- ファイルが存在しない場合はスキップされます
- エラーがあった場合は警告が表示されますが、実行は継続します

### 対象となる実行モード

- ✅ **REPLモード** (`qi`)
- ✅ **ワンライナーモード** (`qi -e '...'`)
- ❌ スクリプトファイル実行 (`qi script.qi`) - 対象外

### ユーザーグローバル設定（~/.qi/init.qi）

すべてのQiプロジェクトで共通に使いたい設定を記述します：

```qi
;; ~/.qi/init.qi - ユーザーグローバル設定

;; よく使うライブラリをプリロード
(use "table" :as table)
(use "string" :as str)

;; デバッグ用ヘルパー関数
(defn dbg [x]
  (do (println (str "DEBUG: " x))
      x))

;; リストの長さを表示してから返す
(defn show-len [lst]
  (do (println (str "length: " (len lst)))
      lst))
```

### プロジェクトローカル設定（./.qi/init.qi）

プロジェクト固有の設定を記述します：

```qi
;; ./.qi/init.qi - プロジェクトローカル設定

;; プロジェクト固有のライブラリをロード
(use "lib/utils" :as utils)

;; プロジェクト固有の定数
(def db-host "localhost")
(def db-port 5432)

;; 開発用のヘルパー
(defn reload-config []
  (load ".env"))
```

### ワンライナーでの活用例

初期化ファイルでライブラリをプリロードしておくと、ワンライナーでuse不要で使えます：

```bash
# ~/.qi/init.qi に (use "table" :as table) を記述しておく

# パイプからCSVを処理（tableライブラリがすでに利用可能）
cat data.csv | qi -e '(stdin |> split "\n" |> table/parse-csv |> (table/where (fn [row] (> (get row :age) 30))))'
```

### 注意点

- 初期化ファイルはREPL/ワンライナー専用です
- スクリプトファイル実行時には読み込まれません（明示的に`load`を使用）
- プロジェクトローカル設定は、ユーザーグローバル設定の**後**に読み込まれます
- 同名の定義がある場合、後から読み込まれた方が優先されます
