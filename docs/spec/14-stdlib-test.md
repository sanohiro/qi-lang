# 標準ライブラリ - テストフレームワーク

**シンプルで使いやすいテストフレームワーク**

---

## 概要

Qiのテストフレームワークは、**シンプルさ**と**覚えやすさ**を重視した設計です。

**特徴:**
- ゼロコンフィグ（設定ファイル不要）
- 自動検出（`tests/`ディレクトリを探索）
- Rust/Go風のシンプルな出力
- 特別な構文なし（通常の関数呼び出し）

---

## テスト実行関数

### test/run

テストを実行して結果を記録します。

**構文:**
```qi
(test/run test-name test-fn)
```

**引数:**
- `test-name` (文字列): テストの名前
- `test-fn` (関数): テストを実行する関数（引数なし）

**戻り値:**
- `true`: テスト成功
- `false`: テスト失敗（エラーが発生）

**例:**
```qi
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))))

(test/run "list operations" (fn []
  (test/assert-eq 3 (len [1 2 3]))
  (test/assert-eq 1 (first [1 2 3]))))
```

### test/run-all

記録されたすべてのテスト結果を表示します。

**構文:**
```qi
(test/run-all)
```

**戻り値:**
- 成功したテストの数（整数）
- 失敗したテストがある場合はエラー

**出力例:**
```
テスト結果:
===========
  ✓ addition
  ✓ list operations
  ✗ division by zero
    期待値: {:error ...}
    実際値: 42

3 テスト, 2 成功, 1 失敗
```

### test/clear

テスト結果をクリアします。

**構文:**
```qi
(test/clear)
```

**戻り値:**
- `nil`

**使用例:**
```qi
(test/clear)  ;; テスト結果をリセット
;; 新しいテストを実行
```

---

## アサーション関数

### test/assert-eq

2つの値が等しいことをアサートします。

**構文:**
```qi
(test/assert-eq expected actual)
```

**引数:**
- `expected`: 期待値
- `actual`: 実際の値

**成功条件:**
- `expected` と `actual` が等しい（`=`で比較）

**例:**
```qi
(test/run "equality tests" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq "HELLO" (str/upper "hello"))
  (test/assert-eq [1 2 3] (map inc [0 1 2]))))
```

### test/assert

値が真であることをアサートします。

**構文:**
```qi
(test/assert value)
```

**引数:**
- `value`: チェックする値

**成功条件:**
- `value` が真（`nil`と`false`以外）

**例:**
```qi
(test/run "truthy tests" (fn []
  (test/assert (> 5 3))
  (test/assert (even? 4))
  (test/assert (some? [1 2 3]))          ;; ベクタはnilでない
  (test/assert (list/some? even? [2 4 6]))))
```

### test/assert-not

値が偽であることをアサートします。

**構文:**
```qi
(test/assert-not value)
```

**引数:**
- `value`: チェックする値

**成功条件:**
- `value` が偽（`nil`または`false`）

**例:**
```qi
(test/run "falsy tests" (fn []
  (test/assert-not (< 5 3))
  (test/assert-not (odd? 4))
  (test/assert-not nil)))
```

### test/assert-throws

関数が例外を投げることをアサートします。

**構文:**
```qi
(test/assert-throws test-fn)
```

**引数:**
- `test-fn` (関数): 実行する関数（引数なし）

**成功条件:**
- `test-fn` を実行したときにエラーが発生する

**例:**
```qi
(test/run "exception tests" (fn []
  (test/assert-throws (fn [] (/ 10 0)))
  (test/assert-throws (fn [] (first [])))
  (test/assert-throws (fn [] (get {} :missing)))))
```

---

## CLIコマンド

### qi test

テストファイルを実行します。

**使用方法:**
```bash
# tests/ディレクトリ内のすべての*.qiファイルを実行
qi test

# 特定のテストファイルを実行
qi test tests/core_test.qi

# 複数のファイルを実行
qi test tests/core_test.qi tests/pipeline_test.qi
```

**テストファイルの配置:**
```
プロジェクト/
  tests/
    core_test.qi
    pipeline_test.qi
    http_test.qi
```

**出力:**
```
running 3 test files

テスト結果:
===========
  ✓ addition
  ✓ subtraction
  ✓ multiplication
  ✓ list operations
  ✓ pipeline test

5 テスト, 5 成功, 0 失敗

finished in 0.08s
```

---

## 実用例

### 基本的なテスト

```qi
;; tests/core_test.qi
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))
  (test/assert-eq -1 (+ 1 -2))))

(test/run "subtraction" (fn []
  (test/assert-eq 1 (- 3 2))
  (test/assert-eq -5 (- 0 5))))

(test/run "multiplication" (fn []
  (test/assert-eq 6 (* 2 3))
  (test/assert-eq 0 (* 5 0))))
```

### パイプラインのテスト

```qi
;; tests/pipeline_test.qi
(test/run "basic pipeline" (fn []
  (test/assert-eq (list 2 3 4) ([1 2 3] |> (map inc)))))

(test/run "pipeline with multiple steps" (fn []
  (test/assert-eq 9 ([1 2 3] |> (map inc) |> sum))))

(test/run "filter and map" (fn []
  (test/assert-eq (list 3 5)
    ([1 2 3 4 5 6] |> (filter even?) |> (map inc) |> (take 2)))))
```

### 文字列操作のテスト

```qi
;; tests/string_test.qi
(test/run "string upper/lower" (fn []
  (test/assert-eq "HELLO" (str/upper "hello"))
  (test/assert-eq "world" (str/lower "WORLD"))))

(test/run "string trimming" (fn []
  (test/assert-eq "hello" (str/trim "  hello  "))
  (test/assert-eq "test" (str/trim-start "  test"))
  (test/assert-eq "test" (str/trim-end "test  "))))

(test/run "string splitting" (fn []
  (test/assert-eq (list "a" "b" "c") (str/split "a,b,c" ","))
  (test/assert-eq (list "hello" "world") (str/split "hello world" " "))))
```

### エラーハンドリングのテスト

```qi
;; tests/error_test.qi
(test/run "division by zero" (fn []
  (test/assert-throws (fn [] (/ 10 0)))))

(test/run "empty list operations" (fn []
  (test/assert-throws (fn [] (first [])))
  (test/assert-throws (fn [] (last [])))))

(test/run "map key not found" (fn []
  (test/assert-throws (fn [] (get {} :missing)))))
```

### HTTPのテスト

```qi
;; tests/http_test.qi
(test/run "http/get success" (fn []
  (def result (http/get "https://httpbin.org/get"))
  (test/assert (map? result))
  (test/assert-eq 200 (get result "status"))))

(test/run "railway pipeline with http" (fn []
  (def result
    (match (try
             ("https://httpbin.org/get"
              |> http/get
              |> (fn [resp] (get resp "body"))
              |>? json/parse))
      {:error e} -> {:error e}
      data -> data))

  (match result
    {:error e} -> (test/assert false)  ;; Should not reach here
    data -> (test/assert (map? data)))))
```

### データ変換のテスト

```qi
;; tests/transform_test.qi
(test/run "json round-trip" (fn []
  (def original {"name" "Alice" "age" 30})
  (def result
    (original
     |>? json/stringify
     |>? json/parse))

  (match result
    {:error e} -> (test/assert false)
    data -> (test/assert-eq original data))))

(test/run "map transformations" (fn []
  (def users [{:name "Alice" :age 30} {:name "Bob" :age 25}])
  (def result (map (fn [u] (update u :age inc)) users))

  (test/assert-eq 31 (get (first result) :age))
  (test/assert-eq 26 (get (last result) :age))))
```

---

## ベストプラクティス

### ファイル構成

```
プロジェクト/
  src/
    core.qi
    utils.qi
  tests/
    core_test.qi      # src/core.qiのテスト
    utils_test.qi     # src/utils.qiのテスト
    integration_test.qi  # 統合テスト
```

### テスト命名

```qi
;; ✅ 良い例: 何をテストしているか明確
(test/run "addition with positive numbers" (fn [] ...))
(test/run "string upper converts to uppercase" (fn [] ...))
(test/run "http/get returns 200 for valid URL" (fn [] ...))

;; ❌ 悪い例: 何をテストしているか不明
(test/run "test1" (fn [] ...))
(test/run "check" (fn [] ...))
```

### テストの粒度

```qi
;; ✅ 良い例: 1つのテストで1つの機能をテスト
(test/run "addition" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq 0 (+ 0 0))))

(test/run "subtraction" (fn []
  (test/assert-eq 1 (- 3 2))))

;; ❌ 悪い例: 1つのテストで複数の無関係な機能をテスト
(test/run "math" (fn []
  (test/assert-eq 3 (+ 1 2))
  (test/assert-eq "HELLO" (str/upper "hello"))
  (test/assert-eq [1 2] (take 2 [1 2 3]))))
```

### エラーメッセージ

```qi
;; アサーションが失敗した場合、わかりやすいメッセージが表示される
(test/run "example" (fn []
  (test/assert-eq 5 (+ 2 2))))

;; 出力:
;;   ✗ example
;;     アサーション失敗:
;;   期待値: 5
;;   実際値: 4
```

---

## 実行フロー

1. **テストファイル検出**: `qi test`は`tests/`ディレクトリ内の`*.qi`ファイルを検索
2. **テスト実行**: 各ファイルを順次ロード・実行
3. **結果収集**: `test/run`が呼ばれるたびに結果を記録
4. **レポート表示**: `test/run-all`で全結果を表示
5. **終了コード**:
   - すべて成功: 終了コード0
   - 1つでも失敗: 終了コード1

---

## 制限事項

**現在のバージョンでは未実装:**
- カバレッジ計測
- テストのタグ付け・フィルタリング
- `--watch`モード（ファイル変更時の自動再実行）
- セットアップ/ティアダウン（`before`/`after`）
- テストの並列実行

これらの機能は**将来のバージョンで検討**されますが、基本的なテストには現在の機能で十分です。

---

## 設計思想

Qiのテストフレームワークは、**シンプルさ**を最優先にしています：

1. **ゼロコンフィグ**: 設定ファイル不要。`tests/`にファイルを置くだけ
2. **特別な構文なし**: 通常の関数呼び出しでテストを記述
3. **最小限のAPI**: 覚えるべき関数は5つだけ
4. **明確な出力**: Rust/Go風のシンプルで読みやすい出力

**複雑な機能は後回し**。まずは基本的なテストが書けることが重要です。
