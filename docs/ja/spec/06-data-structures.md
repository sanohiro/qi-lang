# データ構造

**Qiのコレクション型と操作**

> **実装**: `src/builtins/core_collections.rs`, `src/builtins/list.rs`, `src/builtins/map.rs`, `src/builtins/set.rs`

---

## リストとベクタの使い分け

### いつリストを使うか

**リスト `(...)`** は以下の場合に適しています：

- **再帰的なデータ処理** - first/restでの分解が自然
- **Lisp的な処理** - クオートされたコード、S式
- **関数型プログラミング** - パターンマッチングとの相性が良い
- **順次処理** - 先頭から順に処理する場合

```qi
;; 再帰的な処理
(defn sum [lst]
  (if (empty? lst)
    0
    (+ (first lst) (sum (rest lst)))))

(sum (list 1 2 3 4 5))  ;; => 15
```

### いつベクタを使うか

**ベクタ `[...]`** は以下の場合に適しています：

- **ランダムアクセス** - nth関数で任意の位置にO(1)でアクセス
- **パフォーマンス重視** - メモリ効率が良い
- **JSONとの相互変換** - JSON配列と自然に対応
- **大量のデータ** - 効率的なメモリ使用

```qi
;; ランダムアクセス
(def data [10 20 30 40 50])
(nth data 2)  ;; => 30 (高速)

;; JSONとの相互変換
(json/stringify {:items [1 2 3]})
;; => "{\"items\":[1,2,3]}"
```

### 実用的なガイドライン

- **デフォルトはベクタ** - 迷ったらベクタを使う（現代的、JSON互換）
- **パイプライン処理** - どちらでも同じように動作する
- **パターンマッチング** - 両方サポート、リストの方が慣習的
- **パフォーマンス** - ベクタの方が高速（ただし体感差は少ない）

```qi
;; どちらでも同じように動作
[1 2 3] |> (map inc) |> (filter even?)  ;; => [2 4]
(list 1 2 3) |> (map inc) |> (filter even?)  ;; => (2 4)
```

---

## ListとVectorの型維持ルール

Qiでは、コレクション操作関数は以下のルールに従って戻り値の型を決定します：

### 変換系関数（入力を加工する）
**入力の型を維持**します。List入力→List返却、Vector入力→Vector返却。

- `map`, `filter`, `reduce`（高階関数）
- `sort`, `distinct`, `reverse`
- `take`, `drop`, `rest`, `take-while`, `drop-while`

```qi
;; Vector入力→Vector返却
(map inc [1 2 3])              ;; => [2 3 4]
(filter even? [1 2 3 4])       ;; => [2 4]
(sort [3 1 4])                 ;; => [1 3 4]

;; List入力→List返却
(map inc (list 1 2 3))         ;; => (2 3 4)
(filter even? (list 1 2 3 4))  ;; => (2 4)
```

### 連結系関数（複数のコレクションを結合）
**第一引数の型を優先**します。

- `concat` - 第一引数がVectorならVector返却
- `zip` - 第一引数がVectorならVector返却
- `cons` - 第二引数（コレクション側）の型を維持

```qi
;; 第一引数の型を優先
(concat [1 2] (list 3 4))      ;; => [1 2 3 4]
(concat (list 1 2) [3 4])      ;; => (1 2 3 4)
(zip [1 2] (list "a" "b"))     ;; => [[1 "a"] [2 "b"]]

;; cons は第二引数の型を維持
(cons 0 [1 2 3])               ;; => [0 1 2 3]
(cons 0 (list 1 2 3))          ;; => (0 1 2 3)
```

### 構築系関数（新しいコレクションを生成）
**常にListを返却**します（Lisp系言語の慣習）。

- `range`, `repeat`, `flatten`
- `keys`, `vals`（マップ操作）

```qi
(range 5)                      ;; => (0 1 2 3 4)
(repeat 3 "x")                 ;; => ("x" "x" "x")
(flatten [[1 2] [3 4]])        ;; => (1 2 3 4)
```

### 型変換関数
必要に応じて明示的に型変換できます。

- `list` - 可変長引数からListを生成
- `vector` - 可変長引数からVectorを生成
- `to-list` - List/VectorをListに変換
- `to-vector` - List/VectorをVectorに変換

```qi
(list 1 2 3)                   ;; => (1 2 3)
(vector 1 2 3)                 ;; => [1 2 3]
(to-list [1 2 3])              ;; => (1 2 3)
(to-vector (list 1 2 3))       ;; => [1 2 3]
```

---

## ベクター

### 基本

```qi
[1 2 3]           ;; 数値のベクター
["a" "b" "c"]     ;; 文字列のベクター
[1 "hello" :key]  ;; 混在も可能
[]                ;; 空のベクター
```

### アクセス

```qi
;; nth - n番目の要素を取得（0から開始）
(nth [10 20 30] 1)     ;; => 20

;; first - 最初の要素
(first [10 20 30])     ;; => 10

;; last - 最後の要素
(last [10 20 30])      ;; => 30

;; rest - 最初以外の要素
(rest [10 20 30])      ;; => [20 30]
```

### 追加・結合

```qi
;; cons - 先頭に要素を追加（コレクションの型を維持）
(cons 0 [10 20 30])       ;; => [0 10 20 30]

;; conj - 末尾に要素を追加（Vectorは末尾追加、Listは先頭追加）
(conj [10 20 30] 40)      ;; => [10 20 30 40]

;; concat - リストを連結（第一引数の型を優先）
(concat [10 20] [30 40])  ;; => [10 20 30 40]
```

### 取得・スキップ

```qi
;; take - 最初のn個を取得（型を維持）
(take 2 [10 20 30 40])    ;; => [10 20]

;; drop - 最初のn個をスキップ（型を維持）
(drop 2 [10 20 30 40])    ;; => [30 40]

;; take-while - 述語が真の間取得
(take-while (fn [x] (< x 5)) [1 2 3 6 7 4])  ;; => (1 2 3)

;; drop-while - 述語が真の間削除
(drop-while (fn [x] (< x 5)) [1 2 3 6 7 4])  ;; => (6 7 4)

;; list/drop-last - 末尾n個を削除
(list/drop-last 2 [1 2 3 4 5])  ;; => (1 2 3)

;; list/take-nth - n個おきに取得
(list/take-nth 2 [1 2 3 4 5 6])  ;; => (1 3 5)

;; list/split-at - 指定位置で分割
(list/split-at 2 [1 2 3 4 5])  ;; => [(1 2) (3 4 5)]
```

### 変換

```qi
;; reverse - 順序を反転（型を維持）
(reverse [10 20 30])         ;; => [30 20 10]

;; flatten - ネストを平坦化（常にListを返却）
(flatten [[1 2] [3 [4 5]]])  ;; => (1 2 3 4 5)

;; distinct - 重複を排除（型を維持）
(distinct [1 2 2 3 3 3])     ;; => [1 2 3]

;; list/dedupe - 連続する重複を削除
(list/dedupe [1 1 2 2 3 3])  ;; => (1 2 3)
(list/dedupe [1 2 1 2])      ;; => (1 2 1 2) (非連続は残る)

;; sort - 昇順ソート（型を維持）
(sort [3 1 4 1 5])           ;; => [1 1 3 4 5]

;; list/interleave - 2つのリストを交互に結合
(list/interleave [1 2 3] [4 5 6])  ;; => (1 4 2 5 3 6)

;; list/chunk - 指定サイズで分割
(list/chunk 2 [1 2 3 4 5 6])  ;; => ((1 2) (3 4) (5 6))

;; list/zipmap - 2つのリストからマップを生成
(list/zipmap [:a :b :c] [1 2 3])  ;; => {:a 1, :b 2, :c 3}
```

### サイズ・状態

```qi
;; len - 要素数を返す
(len [10 20 30])      ;; => 3

;; count - 要素数を返す（lenのエイリアス）
(count [10 20 30])    ;; => 3

;; empty? - 空かどうかを判定
(empty? [])           ;; => true
(empty? [1])          ;; => false
```

---

## リスト

### 基本

```qi
'(1 2 3)          ;; クオート必須
'()               ;; 空リスト

(first '(1 2 3))  ;; => 1
(rest '(1 2 3))   ;; => (2 3)
```

### 生成

```qi
;; list - 可変長引数からListを生成
(list 1 2 3)      ;; => (1 2 3)
(list)            ;; => ()

;; range - 数値の範囲を生成
(range 5)         ;; => (0 1 2 3 4)
(range 2 5)       ;; => (2 3 4)

;; repeat - 同じ値をn回繰り返したリストを生成
(repeat 5 0)      ;; => (0 0 0 0 0)
(repeat 3 "a")    ;; => ("a" "a" "a")
(repeat 2 [1 2])  ;; => ([1 2] [1 2])
```

---

## バイナリデータ（Bytes）

Qiはバイナリデータを扱うための専用の`Bytes`型を提供します。画像ファイル、暗号化データ、ネットワークプロトコルなど、バイト列を直接扱う必要がある場面で使用します。

### 基本

```qi
;; 整数のベクターからBytesを生成（各要素は0-255）
(bytes [255 254 253])          ;; => #bytes[FF FE FD]
(bytes [0 1 2 3])              ;; => #bytes[00 01 02 03]
(bytes [72 101 108 108 111])   ;; => #bytes[48 65 6C 6C 6F] ;; "Hello"のASCII

;; Bytesを整数のベクターに変換
(bytes/to-vec #bytes[FF FE FD]) ;; => [255 254 253]
```

### 型チェック

```qi
;; bytes? - Bytes型かどうかを判定
(bytes? (bytes [1 2 3]))  ;; => true
(bytes? "string")         ;; => false
(bytes? [1 2 3])          ;; => false
```

### 比較

Bytes型は値として比較されます（ポインタ比較ではありません）。

```qi
(def b1 (bytes [1 2 3]))
(def b2 (bytes [1 2 3]))
(= b1 b2)  ;; => true（内容が同じなら等しい）
```

### データベースでの利用

Bytes型はデータベースのBLOB型として保存・取得できます（PostgreSQL、MySQL、SQLite）。

```qi
;; SQLiteの例
(def conn (db/connect "sqlite:///data.db" {}))
(def binary-data (bytes [137 80 78 71]))  ;; PNGヘッダー

;; BLOBとして保存
(db/exec conn "INSERT INTO files (data) VALUES (?)" [binary-data])

;; BLOBとして取得
(def rows (db/query conn "SELECT data FROM files" []))
(def retrieved (get (first rows) "data"))
(bytes? retrieved)  ;; => true
```

### HTTPでの利用

HTTPリクエスト/レスポンスのボディとしてBytes型を使用できます。

```qi
;; バイナリデータを送信
(http/post "https://api.example.com/upload"
  {:body (bytes [1 2 3])
   :headers {"Content-Type" "application/octet-stream"}})

;; バイナリレスポンスを受信
(def response (http/get "https://example.com/image.png"))
(bytes? (get response :body))  ;; => true（画像データ）
```

### JSON/YAMLでの利用

Bytes型はJSON/YAMLに変換する際、自動的にBase64エンコードされます。

```qi
(def data {:name "test.png" :content (bytes [137 80 78 71])})

;; JSON変換（Base64エンコード）
(json/stringify data)
;; => "{\"name\":\"test.png\",\"content\":\"iVBORw==\"}"

;; YAML変換（Base64エンコード）
(yaml/stringify data)
;; => "name: test.png\ncontent: iVBORw=="
```

---

## マップ

### 基本

```qi
{:name "Alice" :age 30}    ;; キーワードをキーにする
{"name" "Bob" "age" 25}    ;; 文字列をキーにする
{}                         ;; 空マップ
```

### マップのキーとして使える型

Qiのマップは、**型安全なキー**（`MapKey`型）のみをサポートします。以下の4つの型がキーとして使用できます：

| 型 | 例 | 用途 |
|---|---|---|
| **Keyword** | `:name`, `:age` | 最も一般的。構造化データのキー |
| **String** | `"name"`, `"email"` | JSON互換、外部データとの連携 |
| **Symbol** | `'foo`, `'bar` | マクロ、メタプログラミング |
| **Integer** | `0`, `1`, `42` | 配列風のアクセス、インデックス |

#### Floatはキーになれない

**理由**: 浮動小数点数はハッシュ化が不安定なため、マップのキーとして使用できません。

```qi
;; ✅ 使える型
{:name "Alice"}           ;; Keyword
{"email" "test@test.com"} ;; String
{42 "value"}              ;; Integer

;; ❌ エラー
{3.14 "pi"}               ;; Float - エラー: Floatはキーになれない
```

#### 内部実装

Qiの内部では、マップのキーは`MapKey` enum型で表現されます：

```rust
pub enum MapKey {
    Keyword(Arc<str>),   // :name
    Symbol(Arc<str>),    // 'symbol
    String(String),      // "text"
    Integer(i64),        // 42
}
```

この設計により：
- **型安全**: コンパイル時にキーの型をチェック
- **高速**: Arc<str>によるインターン化で比較が高速
- **メモリ効率**: 同じキーワード/シンボルはメモリを共有

### アクセス

```qi
;; get - キーで値を取得
(get {:name "Alice" :age 30} :name)   ;; => "Alice"
(get {:name "Alice"} :age 0)          ;; => 0 (デフォルト値)

;; キーワード・文字列を関数として使う（簡潔な記法）
(:name {:name "Alice" :age 30})       ;; => "Alice"
(:age {:name "Alice" :age 30})        ;; => 30
("name" {"name" "Bob" "age" 25})      ;; => "Bob"

;; パイプラインでの使用
(def response {:status 200 :body "OK"})
(response |> :status)                 ;; => 200

;; JSON等の外部データへのアクセス
(def user-data {"name" "Carol" "email" "carol@example.com"})
("email" user-data)                   ;; => "carol@example.com"

;; keys - 全キーを取得
(keys {:name "Alice" :age 30})        ;; => ("name" "age")

;; vals - 全値を取得
(vals {:name "Alice" :age 30})        ;; => ("Alice" 30)
```

### 追加・削除

```qi
;; assoc - キーと値を追加
(assoc {:name "Alice"} :age 30)           ;; => {:name "Alice" :age 30}

;; dissoc - キーを削除
(dissoc {:name "Alice" :age 30} :age)     ;; => {:name "Alice"}
```

### マージ・選択

```qi
;; merge - マップを結合
(merge {:a 1} {:b 2})                         ;; => {:a 1 :b 2}
(merge {:a 1} {:a 2})                         ;; => {:a 2} (右が優先)

;; map/select-keys - 指定したキーのみ抽出
(map/select-keys {:a 1 :b 2 :c 3} [:a :c])   ;; => {:a 1 :c 3}
```

### ネスト操作

```qi
;; update - 値を関数で更新
(update {:name "Alice" :age 30} :age inc)
;; => {:name "Alice" :age 31}

;; update-in - ネストした値を更新
(update-in {:user {:profile {:visits 10}}} [:user :profile :visits] inc)
;; => {:user {:profile {:visits 11}}}

;; get-in - ネストした値を取得
;; キーが存在しない場合はデフォルト値を返すが、値がnilの場合はnilを返す
(get-in {:user {:name "Bob"}} [:user :name])    ;; => "Bob"
(get-in {} [:user :name] "guest")               ;; => "guest"
(get-in {:key nil} [:key] "default")            ;; => nil（値がnilの場合）
(get-in {:other "val"} [:key] "default")        ;; => "default"（キーが存在しない場合）

;; map/assoc-in - ネストしたマップに値を設定
(map/assoc-in {} [:user :profile :name] "Alice")
;; => {:user {:profile {:name "Alice"}}}
(map/assoc-in {:user {:age 30}} [:user :name] "Bob")
;; => {:user {:age 30, :name "Bob"}}

;; map/dissoc-in - ネストしたマップからキーを削除
(map/dissoc-in {:user {:name "Alice" :age 30}} [:user :age])
;; => {:user {:name "Alice"}}
(map/dissoc-in {:a {:b {:c 1}}} [:a :b :c])
;; => {:a {:b {}}}
```

### マップの一括変換

```qi
;; map/map-keys - キーを変換
(map/map-keys str/upper {:name "Alice" :age 30})
;; => {"NAME" "Alice", "AGE" 30}

;; map/map-vals - 値を変換
(map/map-vals inc {:a 1 :b 2})
;; => {:a 2, :b 3}

;; map/filter-keys - キーでフィルタ
(map/filter-keys keyword? {:name "Alice" "age" 30})
;; => {:name "Alice"}

;; map/filter-vals - 値でフィルタ
(map/filter-vals even? {:a 1 :b 2 :c 3})
;; => {:b 2}
```

---

## セット（集合演算）

```qi
;; set/union - 和集合
(set/union [1 2] [2 3])                         ;; => [1 2 3]

;; set/intersect - 積集合
(set/intersect [1 2 3] [2 3 4])                 ;; => [2 3]

;; set/difference - 差集合
(set/difference [1 2 3] [2])                    ;; => [1 3]

;; set/symmetric-difference - 対称差
(set/symmetric-difference [1 2 3] [2 3 4])      ;; => [1 4]

;; set/subset? - 部分集合判定
(set/subset? [1 2] [1 2 3])                     ;; => true

;; set/superset? - 上位集合判定
(set/superset? [1 2 3] [1 2])                   ;; => true

;; set/disjoint? - 互いに素判定
(set/disjoint? [1 2] [3 4])                     ;; => true
```

---

## 高階関数

### map - 各要素に関数を適用

```qi
;; Vector入力→Vector返却
(map inc [1 2 3])                    ;; => [2 3 4]
(map str [1 2 3])                    ;; => ["1" "2" "3"]

;; List入力→List返却
(map inc (list 1 2 3))               ;; => (2 3 4)

;; パイプラインで使う
([1 2 3] |> (map (fn [x] (* x x))))  ;; => [1 4 9]
```

### filter - 条件を満たす要素のみ抽出

```qi
;; Vector入力→Vector返却
(filter even? [1 2 3 4 5])                      ;; => [2 4]
(filter (fn [x] (> x 10)) [5 15 3 20])          ;; => [15 20]

;; List入力→List返却
(filter even? (list 1 2 3 4 5))                 ;; => (2 4)

;; パイプラインで使う
([1 2 3 4 5] |> (filter odd?))                  ;; => [1 3 5]
```

### reduce - 畳み込み

```qi
(reduce + 0 [1 2 3 4])        ;; => 10
(reduce * 1 [2 3 4])          ;; => 24

;; パイプラインで使う
([1 2 3 4 5] |> (reduce + 0)) ;; => 15
```

### each - 各要素に関数を適用（副作用用）

`map`と異なり、戻り値を収集せず`nil`を返します。副作用（println、ファイル書き込みなど）を目的とする場合に使用します。

```qi
;; 基本的な使い方
(each println [1 2 3])
;; 出力:
;; 1
;; 2
;; 3
;; => nil

;; Vector、List両方に対応
(each println (list "a" "b" "c"))
;; 出力:
;; a
;; b
;; c
;; => nil

;; ラムダ式と組み合わせ
(each (fn [x] (println f"値: {x}")) [10 20 30])
;; 出力:
;; 値: 10
;; 値: 20
;; 値: 30
;; => nil

;; パイプラインで使う
(def lines ["  hello  " "  world  " "  test  "])
(lines
 |> (map str/trim)
 |> (map str/upper)
 |> (each println))
;; 各行を大文字に変換して出力

;; when と組み合わせた条件付き処理
(def data ["item1" "" "item2" "item3"])
(data
 |> (each (fn [item]
            (when (> (len item) 0)
              (println f"処理: {item}")))))

;; 統計情報の集計
(def data2 [1 2 3 4 5])
(def count (atom 0))
(data2
 |> (each (fn [item]
            (when (> item 2)
              (swap! count inc)))))
```

**map との使い分け**:
- `map`: 戻り値が必要な場合（データ変換）
- `each`: 戻り値が不要な場合（副作用のみ）

```qi
;; map - 変換結果を返す
(map inc [1 2 3])  ;; => [2 3 4]

;; each - 副作用のみ、nilを返す
(each println [1 2 3])  ;; => nil（ただし各要素が出力される）
```

### find - 条件を満たす最初の要素

```qi
(find (fn [x] (> x 5)) [1 7 3])     ;; => 7
(find even? [1 3 4 5])              ;; => 4
```

### 述語（全体チェック）

```qi
;; list/every? - 全要素が条件を満たすか
(list/every? (fn [x] (> x 0)) [1 2 3])   ;; => true
(list/every? even? [2 4 6])              ;; => true

;; list/some? - いずれかの要素が条件を満たすか
(list/some? (fn [x] (> x 5)) [1 7 3])    ;; => true
(list/some? even? [1 3 5])               ;; => false
```

**注:** `some?`（1引数）はCore述語関数として、nilでないかをチェックする述語です（→ 基本構文参照）。

---

## ソート・グループ化

### ソート

```qi
;; sort - 昇順ソート（型を維持）
(sort [3 1 4 1 5])                             ;; => [1 1 3 4 5]
(sort ["zebra" "apple" "banana"])              ;; => ["apple" "banana" "zebra"]
(sort (list 3 1 4 1 5))                        ;; => (1 1 3 4 5)

;; list/sort-by - キー指定ソート
(list/sort-by (fn [u] (get u :age))
  [{:name "Bob" :age 25} {:name "Alice" :age 30}])
;; => ({:name "Bob" :age 25} {:name "Alice" :age 30})
```

### グループ化

```qi
;; list/group-by - キー関数でグループ化
(list/group-by even? [1 2 3 4 5 6])
;; => {true [2 4 6], false [1 3 5]}

;; list/partition - 述語で2分割
(list/partition even? [1 2 3 4])
;; => [[2 4] [1 3]]
```

### 頻度・カウント

```qi
;; list/frequencies - 出現頻度を集計
(list/frequencies [1 2 2 3 3 3])
;; => {1 1, 2 2, 3 3}

;; list/count-by - 述語でカウント
(list/count-by even? [1 2 3 4])
;; => {true 2, false 2}
```

---

## 実用例

### データ分析パイプライン

```qi
;; 重複排除してソート
([5 2 8 2 9 1 3 8 4]
 |> distinct
 |> sort)
;; => (1 2 3 4 5 8 9)

;; グループ化して集計
(list/group-by (fn [n] (% n 3)) [1 2 3 4 5 6 7 8 9])
;; => {0 (3 6 9), 1 (1 4 7), 2 (2 5 8)}
```

### ユーザー検索

```qi
(def users [{:name "Alice" :age 30} {:name "Bob" :age 25}])

;; ユーザーを名前で探す
(find (fn [u] (= (get u :name) "Bob")) users)
;; => {:name "Bob" :age 25}

;; 全員成人か確認
(list/every? (fn [u] (>= (get u :age) 20)) users)
;; => true

;; パイプラインで検索
(users
 |> (filter (fn [u] (>= (get u :age) 25)))
 |> (map (fn [u] (get u :name))))
;; => ("Alice" "Bob")
```
