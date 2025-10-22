# バリデーション機能設計書

## 概要

Qiのスキーマベースバリデーションシステム。マップで定義したスキーマに基づいてデータを検証する。

## 設計方針

- **シンプルさ**: 実用的な80%のユースケースをカバー
- **i18n対応**: エラーメッセージは多言語対応
- **拡張性**: カスタムバリデータで複雑なケースに対応
- **日本語ネイティブ**: 日本語文字種チェックを標準サポート

## 基本的な使い方

```qi
;; スキーマ定義
(def user-schema
  {:type :map
   :fields {:name {:type :string :required true :min-length 1 :max-length 50}
            :age {:type :number :min 0 :max 150}
            :email {:type :string :pattern "^[^@]+@[^@]+\\.[^@]+$"}}})

;; バリデーション実行
(validate user-schema {:name "太郎" :age 25 :email "taro@example.com"})
;=> {:ok {:name "太郎" :age 25 :email "taro@example.com"}}

(validate user-schema {:age "invalid"})
;=> {:error {:field :age :code :type-mismatch :message "数値である必要があります"}}
```

## スキーマ定義

### 基本型チェック

```qi
{:type :string}    ;; 文字列
{:type :number}    ;; 数値
{:type :integer}   ;; 整数
{:type :boolean}   ;; 真偽値
{:type :map}       ;; マップ
{:type :vector}    ;; ベクター
{:type :list}      ;; リスト
{:type :keyword}   ;; キーワード
{:type :symbol}    ;; シンボル
{:type :nil}       ;; nil
{:type :any}       ;; 任意の型（型チェックなし）
```

### 必須フィールド

```qi
{:required true}   ;; 必須
{:required false}  ;; オプション（デフォルト）
```

### 数値バリデーション

```qi
{:type :number
 :min 0           ;; 最小値
 :max 100         ;; 最大値
 :positive true   ;; 正の数
 :negative true   ;; 負の数
 :integer true}   ;; 整数であること
```

### 文字列バリデーション

```qi
{:type :string
 :min-length 1          ;; 最小文字数
 :max-length 100        ;; 最大文字数
 :pattern "^[a-z]+$"    ;; 正規表現パターン
 :trim true}            ;; トリム後に検証（デフォルト: false）
```

### 文字種チェック

```qi
{:type :string
 :char-class :alphanumeric}  ;; 英数字のみ

;; プリセット
:alphabetic    ;; 英字（a-z, A-Z）
:numeric       ;; 数字（0-9）
:alphanumeric  ;; 英数字
:ascii         ;; ASCII文字のみ
:hiragana      ;; ひらがな（U+3040-U+309F）
:katakana      ;; カタカナ（U+30A0-U+30FF）
:kanji         ;; 漢字（U+4E00-U+9FFF）
:zenkaku       ;; 全角文字
:hankaku       ;; 半角文字

;; 複数指定（いずれかを含む）
:char-class [:hiragana :katakana :kanji]

;; 含有チェック
:contains :uppercase   ;; 大文字を含む
:contains :lowercase   ;; 小文字を含む
:contains :digit       ;; 数字を含む
:contains :symbol      ;; 記号を含む

;; カスタム文字セット
:char-set "あいうえおアイウエオ"  ;; 指定文字のみ許可
```

### Unicode範囲指定

```qi
{:type :string
 :unicode-range [0x3040 0x309F]}  ;; ひらがな範囲
```

### 日付バリデーション

```qi
{:type :string
 :date-format "yyyy-MM-dd"     ;; 日付フォーマット
 :min-date "2020-01-01"        ;; 最小日付
 :max-date "2030-12-31"        ;; 最大日付
 :after "2020-01-01"           ;; 指定日付より後
 :before "2030-12-31"          ;; 指定日付より前
 :within-days 30}              ;; 今日から±30日以内
```

### コレクションバリデーション

```qi
{:type :vector
 :min-items 1      ;; 最小要素数
 :max-items 10     ;; 最大要素数
 :item-schema {:type :string}}  ;; 各要素のスキーマ

{:type :map
 :fields {:name {:type :string :required true}
          :age {:type :number}}}  ;; フィールド定義
```

### カスタムバリデータ

```qi
{:type :string
 :validator (fn [value]
              (if (starts-with? value "test-")
                {:ok value}
                {:error "test- で始まる必要があります"}))}

;; 複数バリデータ
{:validators [(fn [v] ...) (fn [v] ...)]}
```

### カスタムメッセージ

```qi
{:type :string
 :min-length 5
 :message "5文字以上入力してください"}

;; メッセージ関数
{:type :string
 :min-length 5
 :message-fn (fn [field value error]
               (str field "は" (:min-length error) "文字以上必要です"))}
```

### i18n対応

```qi
;; 環境変数 QI_LANG で言語切り替え
;; エラーメッセージは自動的に対応言語で返される

;; QI_LANG=ja
;=> {:error {:field :age :code :type-mismatch :message "数値である必要があります"}}

;; QI_LANG=en
;=> {:error {:field :age :code :type-mismatch :message "must be a number"}}
```

## 提供関数

### validate
データをスキーマで検証する。

```qi
(validate schema data)
;=> {:ok data} または {:error {:field ... :code ... :message ...}}
```

### validate-field
単一フィールドを検証する。

```qi
(validate-field field-schema value)
;=> {:ok value} または {:error {:code ... :message ...}}
```

### validate!
検証失敗時にエラーを投げる。

```qi
(validate! schema data)
;=> data（成功） または エラー発生
```

## エラーコード

- `:type-mismatch` - 型が一致しない
- `:required` - 必須フィールドが不足
- `:min-length` - 最小文字数未満
- `:max-length` - 最大文字数超過
- `:min-value` - 最小値未満
- `:max-value` - 最大値超過
- `:pattern` - パターン不一致
- `:char-class` - 文字種不一致
- `:date-format` - 日付フォーマット不正
- `:date-range` - 日付範囲外
- `:min-items` - 最小要素数未満
- `:max-items` - 最大要素数超過
- `:custom` - カスタムバリデータエラー

## 実装詳細

### ファイル構成

- `src/builtins/validation.rs` - バリデーション関数実装
- `src/i18n.rs` - バリデーションエラーメッセージ追加

### 依存関係

- `regex` - 正規表現マッチング（既存）
- `chrono` - 日付処理（feature: validation-date で追加）

### feature flag

```toml
[features]
default = ["validation", ...]
validation = []
validation-date = ["dep:chrono", "validation"]
```

## 使用例

### ユーザー登録フォーム

```qi
(def signup-schema
  {:type :map
   :fields {:username {:type :string
                       :required true
                       :min-length 3
                       :max-length 20
                       :pattern "^[a-zA-Z0-9_-]+$"
                       :char-class :alphanumeric}
            :password {:type :string
                       :required true
                       :min-length 8
                       :contains [:uppercase :lowercase :digit :symbol]}
            :email {:type :string
                    :required true
                    :pattern "^[^@]+@[^@]+\\.[^@]+$"}
            :age {:type :integer
                  :min 13
                  :max 150}}})

(validate signup-schema
  {:username "user123"
   :password "Pass123!"
   :email "user@example.com"
   :age 25})
```

### 日本語名前バリデーション

```qi
(def japanese-name-schema
  {:type :map
   :fields {:name {:type :string
                   :required true
                   :char-class [:hiragana :katakana :kanji]
                   :min-length 1
                   :max-length 50}
            :furigana {:type :string
                       :required true
                       :char-class :hiragana
                       :min-length 1
                       :max-length 50}}})

(validate japanese-name-schema
  {:name "山田太郎"
   :furigana "やまだたろう"})
```

### イベント日付バリデーション

```qi
(def event-schema
  {:type :map
   :fields {:title {:type :string :required true}
            :date {:type :string
                   :required true
                   :date-format "yyyy-MM-dd"
                   :after "2025-01-01"
                   :before "2026-12-31"}}})

(validate event-schema
  {:title "忘年会"
   :date "2025-12-20"})
```

## 実装フェーズ

### Phase 1: 基本実装（最優先）
- [x] 設計書作成
- [ ] 基本型チェック（:type）
- [ ] 必須チェック（:required）
- [ ] 数値範囲（:min, :max）
- [ ] 文字列長（:min-length, :max-length）
- [ ] パターンマッチ（:pattern）
- [ ] i18nメッセージ

### Phase 2: 文字種チェック
- [ ] 基本文字種（:alphabetic, :numeric, :alphanumeric, :ascii）
- [ ] 日本語文字種（:hiragana, :katakana, :kanji, :zenkaku, :hankaku）
- [ ] 含有チェック（:contains）
- [ ] Unicode範囲（:unicode-range）

### Phase 3: 日付バリデーション（feature flag）
- [ ] 日付フォーマット（:date-format）
- [ ] 日付範囲（:min-date, :max-date, :after, :before）
- [ ] 相対日付（:within-days）

### Phase 4: 拡張機能
- [ ] カスタムバリデータ（:validator, :validators）
- [ ] カスタムメッセージ（:message, :message-fn）
- [ ] ネストしたマップ/ベクターバリデーション
- [ ] validate! 関数

## テスト計画

- 基本型チェックのテスト
- 必須/オプションフィールドのテスト
- 数値範囲のテスト
- 文字列長のテスト
- パターンマッチのテスト
- 文字種チェックのテスト（ASCII + 日本語）
- 日付バリデーションのテスト
- カスタムバリデータのテスト
- i18nメッセージのテスト（日本語/英語）
- エラーケースのテスト
