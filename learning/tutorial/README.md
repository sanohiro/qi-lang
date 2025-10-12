# Qi言語チュートリアル

Qi言語の基本から応用までを学びます。プログラミング初心者やLisp初心者でも理解できるように、段階的に解説します。

## Qiとは

Qiは**モダンなLisp系言語**です：

- **シンプルな構文**: S式（括弧で囲まれた式）
- **関数型プログラミング**: イミュータブルなデータ構造
- **並行処理**: goroutine風の軽量スレッド
- **豊富な標準ライブラリ**: HTTP、DB、ファイルI/O等
- **パイプライン演算子**: データ処理が直感的

## 学習パス

1. **[基本文法](./01-basics.md)** - 式、データ型、変数、関数
2. **[リストとコレクション](./02-collections.md)** - リスト、ベクタ、マップ
3. **[制御構造](./03-control-flow.md)** - if、match、loop
4. **[関数型プログラミング](./04-functional.md)** - map、filter、reduce
5. **[並行処理](./05-concurrency.md)** - go、チャネル、並列処理
6. **[実践的なプログラミング](./06-practical.md)** - ファイル、HTTP、データベース

## Qiの特徴

### S式（Symbolic Expression）

すべてのコードは**括弧で囲まれた式**です：

```lisp
(+ 1 2 3)           ; 式
(println "Hello")   ; 関数呼び出し
(def x 42)          ; 変数定義
```

### パイプライン演算子

データの変換を直感的に書けます：

```lisp
; 従来の書き方
(filter even? (map inc [1 2 3 4 5]))

; パイプライン
[1 2 3 4 5]
|> (map inc)
|> (filter even?)
```

### 並行処理

簡単に並行処理ができます：

```lisp
; goroutine風
(go (fn [] (expensive-task)))

; 並列map
(pmap process-item items)
```

## 開発環境のセットアップ

### インストール

```bash
# リポジトリをクローン
git clone https://github.com/your-org/qi-lang.git
cd qi-lang

# ビルド
cargo build --release

# パスを通す
export PATH="$PWD/target/release:$PATH"
```

### REPL（対話環境）

```bash
$ qi
qi> (+ 1 2 3)
6
qi> (def name "Alice")
"Alice"
qi> (println (str "Hello, " name))
Hello, Alice
nil
qi> (exit)
```

### スクリプトファイル

```lisp
; hello.qi
(println "Hello, World!")
```

```bash
$ qi hello.qi
Hello, World!
```

## はじめてのプログラム

### Hello, World!

```lisp
(println "Hello, World!")
```

実行：
```bash
$ qi -e '(println "Hello, World!")'
Hello, World!
```

### 簡単な計算

```lisp
; 四則演算
(+ 1 2 3)        ; => 6
(- 10 3)         ; => 7
(* 2 3 4)        ; => 24
(/ 10 2)         ; => 5

; 比較
(= 1 1)          ; => true
(> 5 3)          ; => true
(<= 2 2)         ; => true
```

### 変数と関数

```lisp
; 変数定義
(def x 10)
(def name "Alice")

; 関数定義
(defn add [a b]
  (+ a b))

; 使用
(add 3 4)        ; => 7
```

## Lispの考え方

### コードはデータ

Lispでは**コードもデータ**です：

```lisp
; これは式（データ）
(+ 1 2)

; クォートすると評価されない
'(+ 1 2)         ; => (+ 1 2)

; 評価すると計算される
(eval '(+ 1 2))  ; => 3
```

### すべてが式

Lispでは**すべてが値を返す式**です：

```lisp
; if は式
(def result (if (> x 10) "big" "small"))

; 関数定義も式
(def my-fn (fn [x] (* x 2)))
```

### 前置記法

演算子が最初に来ます：

```lisp
; 前置記法
(+ 1 2 3)        ; 1 + 2 + 3

; 他の言語の中置記法
; 1 + 2 + 3
```

**利点:**
- 可変長引数が自然
- 演算子の優先順位がない
- 一貫性のある構文

## コメント

```lisp
; これは行コメント
(+ 1 2)  ; 式の後ろにもコメント

(def x 10)  ; 変数を定義
```

## 次のステップ

準備ができたら、[基本文法](./01-basics.md)から始めましょう！

Qi言語を楽しく学んでいきましょう。
