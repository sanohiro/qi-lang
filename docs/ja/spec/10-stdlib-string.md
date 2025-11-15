# 標準ライブラリ - 文字列操作（str/）

**60以上の文字列操作関数**

すべての関数は `str/` モジュールに属します。

---

## 検索

```qi
;; str/contains? - 部分文字列を含むか判定
(str/contains? "hello world" "world")     ;; => true

;; str/starts-with? - 前方一致判定
(str/starts-with? "hello" "he")           ;; => true

;; str/ends-with? - 後方一致判定
(str/ends-with? "hello" "lo")             ;; => true

;; str/index-of - 最初の出現位置（見つからない場合はnil）
(str/index-of "hello world" "world")      ;; => 6
(str/index-of "hello" "xyz")              ;; => nil

;; str/last-index-of - 最後の出現位置（見つからない場合はnil）
(str/last-index-of "hello hello" "hello") ;; => 6
(str/last-index-of "hello" "xyz")         ;; => nil
```

---

## 基本変換

```qi
;; str/upper - 大文字化
(str/upper "hello")                       ;; => "HELLO"

;; str/lower - 小文字化
(str/lower "HELLO")                       ;; => "hello"

;; str/capitalize - 先頭のみ大文字
(str/capitalize "hello")                  ;; => "Hello"

;; str/title - タイトルケース
(str/title "hello world")                 ;; => "Hello World"

;; str/trim - 前後の空白を削除
(str/trim "  hello  ")                    ;; => "hello"

;; str/trim-left - 左側の空白を削除
(str/trim-left "  hello  ")               ;; => "hello  "

;; str/trim-right - 右側の空白を削除
(str/trim-right "  hello  ")              ;; => "  hello"

;; str/repeat - 文字列を繰り返し
(str/repeat "-" 80)                       ;; => "----..." (80個)
(str/repeat "ab" 3)                       ;; => "ababab"

;; str/reverse - 文字列を逆順に
(str/reverse "hello")                     ;; => "olleh"
```

---

## ケース変換

```qi
;; str/snake - スネークケースに変換
(str/snake "userName")                    ;; => "user_name"

;; str/camel - キャメルケースに変換
(str/camel "user_name")                   ;; => "userName"

;; str/kebab - ケバブケースに変換
(str/kebab "userName")                    ;; => "user-name"

;; str/pascal - パスカルケースに変換
(str/pascal "user_name")                  ;; => "UserName"

;; str/split-camel - キャメルケースを分割
(str/split-camel "userName")              ;; => ["user" "Name"]
```

---

## 分割・結合

```qi
;; str/split - 文字列を分割
(str/split "a,b,c" ",")                   ;; => ["a" "b" "c"]

;; str/lines - 行に分割
(str/lines "hello\nworld")                ;; => ["hello" "world"]

;; str/words - 単語に分割
(str/words "hello world")                 ;; => ["hello" "world"]

;; str/chars - 文字に分割
(str/chars "hello")                       ;; => ["h" "e" "l" "l" "o"]
```

---

## 置換

```qi
;; str/replace - 全て置換
(str/replace "hello world" "world" "qi")  ;; => "hello qi"

;; str/replace-first - 最初の1つのみ置換
(str/replace-first "aa bb aa" "aa" "cc")  ;; => "cc bb aa"

;; str/splice - 範囲を置換
(str/splice "hello world" 6 11 "universe") ;; => "hello universe"
```

---

## 部分文字列

```qi
;; str/slice - 範囲を取得
(str/slice "hello world" 0 5)             ;; => "hello"

;; str/take-str - 先頭n文字を取得（パイプライン最適化）
(str/take-str 3 "hello")                  ;; => "hel"
("hello" |> (str/take-str 3))             ;; => "hel"

;; str/drop-str - 先頭n文字を削除（パイプライン最適化）
(str/drop-str 2 "hello")                  ;; => "llo"
("hello" |> (str/drop-str 2))             ;; => "llo"

;; str/sub-before - 区切り文字より前を取得
(str/sub-before "user@example.com" "@")   ;; => "user"

;; str/sub-after - 区切り文字より後を取得
(str/sub-after "user@example.com" "@")    ;; => "example.com"
```

---

## 整形・配置

```qi
;; str/pad-left - 左詰め
(str/pad-left "Total" 20)                 ;; => "               Total"

;; str/pad-right - 右詰め
(str/pad-right "Name" 20)                 ;; => "Name               "

;; str/pad - 中央揃え
(str/pad "hi" 10)                         ;; => "    hi    "
(str/pad "hi" 10 "*")                     ;; => "****hi****"

;; str/truncate - 長さを制限
(str/truncate "hello world" 8)            ;; => "hello..."

;; str/trunc-words - 単語数を制限
(str/trunc-words "hello world from qi" 2) ;; => "hello world..."

;; str/indent - インデント追加
(str/indent "hello\nworld" 2)             ;; => "  hello\n  world"

;; str/wrap - 指定幅で折り返し
(str/wrap "hello world from qi" 10)       ;; => "hello\nworld from\nqi"
```

---

## 正規化

```qi
;; str/squish - 連続空白を1つに（前後trim込み）
(str/squish "  hello   world  \n")        ;; => "hello world"

;; str/expand-tabs - タブをスペースに変換
(str/expand-tabs "\thello\tworld")        ;; => "    hello    world"
```

---

## 判定（バリデーション）

```qi
;; str/digit? - 数字のみか判定
(str/digit? "12345")                      ;; => true

;; str/alpha? - アルファベットのみか判定
(str/alpha? "hello")                      ;; => true

;; str/alnum? - 英数字のみか判定
(str/alnum? "hello123")                   ;; => true

;; str/space? - 空白文字のみか判定
(str/space? "  \n\t")                     ;; => true

;; str/numeric? - 数値表現か判定
(str/numeric? "123.45")                   ;; => true

;; str/integer? - 整数表現か判定
(str/integer? "123")                      ;; => true

;; str/blank? - 空白または空文字列か判定
(str/blank? "  \n")                       ;; => true

;; str/ascii? - ASCII文字のみか判定
(str/ascii? "hello")                      ;; => true

;; str/lower? - 全て小文字か判定
(str/lower? "hello")                      ;; => true

;; str/upper? - 全て大文字か判定
(str/upper? "HELLO")                      ;; => true
```

---

## URL/Web

```qi
;; str/slugify - URL/ファイル名用に変換
(str/slugify "Hello World! 2024")         ;; => "hello-world-2024"
(str/slugify "Café résumé")               ;; => "cafe-resume"

;; str/url-encode - URLエンコード
(str/url-encode "hello world")            ;; => "hello%20world"

;; str/url-decode - URLデコード
(str/url-decode "hello%20world")          ;; => "hello world"

;; str/html-encode - HTMLエンコード
(str/html-encode "<div>test</div>")       ;; => "&lt;div&gt;test&lt;/div&gt;"

;; str/html-decode - HTMLデコード
(str/html-decode "&lt;div&gt;test&lt;/div&gt;") ;; => "<div>test</div>"
```

---

## エンコード

```qi
;; str/to-base64 - Base64エンコード
(str/to-base64 "hello")                   ;; => "aGVsbG8="

;; str/from-base64 - Base64デコード
(str/from-base64 "aGVsbG8=")              ;; => "hello"
```

---

## パース

```qi
;; str/parse-int - 整数にパース
(str/parse-int "123")                     ;; => 123

;; str/parse-float - 浮動小数点数にパース
(str/parse-float "3.14")                  ;; => 3.14
```

---

## Unicode

```qi
;; str/chars-count - Unicode文字数
(str/chars-count "👨‍👩‍👧‍👦")                ;; => 1

;; str/bytes-count - バイト数
(str/bytes-count "👨‍👩‍👧‍👦")                ;; => 25
```

---

## 生成

```qi
;; str/hash - ハッシュ値生成
(str/hash "hello")                        ;; => "2cf24dba5fb0a30e..."
(str/hash "hello" :sha256)                ;; SHA-256 (デフォルト)

;; str/uuid - UUID生成
(str/uuid)                                ;; => "550e8400-e29b-41d4-a716-446655440000"
```

---

## NLP

```qi
;; str/word-count - 単語数をカウント
(str/word-count "hello world")            ;; => 2
```

---

## フォーマット

```qi
;; str/format - プレースホルダー置換
(str/format "Hello, {}!" "World")         ;; => "Hello, World!"
(str/format "{} + {} = {}" 1 2 3)         ;; => "1 + 2 = 3"

;; str/format-decimal - 小数点桁数指定
(str/format-decimal 3.14159 2)            ;; => "3.14"
(3.14159 |> (str/format-decimal _ 2))     ;; パイプラインで使用

;; str/format-comma - 3桁カンマ区切り
(str/format-comma 1234567)                ;; => "1,234,567"
(1234567 |> str/format-comma)             ;; パイプラインで使用

;; str/format-percent - パーセント表示
(str/format-percent 0.1234)               ;; => "12%"
(str/format-percent 0.1234 2)             ;; => "12.34%"
(0.856 |> (str/format-percent _ 1))       ;; => "85.6%"
```

---

## 実用例

### URL処理

```qi
;; URLパラメータの生成
(def params [["user" "alice"] ["page" "1"] ["sort" "name"]])

(params
 |> (map (fn [[k v]] (str k "=" (str/url-encode v))))
 |> (join "&" _))
;; => "user=alice&page=1&sort=name"
```

### テキスト整形

```qi
;; マークダウンのコード整形
(defn format-code [code lang]
  f"```{lang}\n{(str/trim code)}\n```")

(format-code "  def x = 42  " "qi")
;; => "```qi\ndef x = 42\n```"
```

### バリデーション

```qi
;; メールアドレスの簡易チェック
(defn valid-email? [email]
  (and
    (str/contains? email "@")
    (str/contains? (str/sub-after email "@") ".")
    (not (str/blank? (str/sub-before email "@")))))

(valid-email? "user@example.com")  ;; => true
(valid-email? "invalid")           ;; => false
```

### データ変換パイプライン

```qi
;; CSVヘッダーの正規化
(def headers ["User Name" "E-Mail" "Created At"])

(headers
 |> (map str/lower)
 |> (map str/squish)
 |> (map (fn [s] (str/replace s " " "_"))))
;; => ["user_name" "e-mail" "created_at"]
```
