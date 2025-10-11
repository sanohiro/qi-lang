# Qiドキュメントシステム仕様

## 概要

Qiは柔軟なドキュメントシステムを提供します：
- **多言語対応**（i18n）：現在は日本語（ja）と英語（en）
- **ハイブリッド型**：ソースコード内記述と外部ファイルの両方に対応
- **3つの記述レベル**：シンプルな文字列から詳細な構造化まで
- **遅延読み込み**：メモリ効率を重視し、必要時のみロード

## 記述方法

### 1. 文字列形式（シンプル）

最もシンプルな形式。関数の簡単な説明のみ。

```lisp
(def greet
  "指定された名前で挨拶する"
  (fn [name]
    (str "Hello, " name "!")))
```

多言語対応：
```lisp
(def greet
  {:en "Greets with the given name"
   :ja "指定された名前で挨拶する"}
  (fn [name]
    (str "Hello, " name "!")))

### 2. 構造化形式（詳細）

詳細なドキュメントを記述する場合。

```lisp
(def greet
  {:desc "指定された名前で挨拶する"
   :params [{:name "name" :type "string" :desc "挨拶する相手の名前"}]
   :returns {:type "string" :desc "挨拶メッセージ"}
   :examples [
     "(greet \"Alice\") ;=> \"Hello, Alice!\""
     "(greet \"Bob\")   ;=> \"Hello, Bob!\""
   ]}
  (fn [name]
    (str "Hello, " name "!")))
```

多言語対応：
```lisp
(def greet
  {:en {:desc "Greets with the given name"
        :params [{:name "name" :type "string" :desc "Name to greet"}]
        :returns {:type "string" :desc "Greeting message"}
        :examples ["(greet \"Alice\") ;=> \"Hello, Alice!\""]}
   :ja {:desc "指定された名前で挨拶する"
        :params [{:name "name" :type "string" :desc "挨拶する相手の名前"}]
        :returns {:type "string" :desc "挨拶メッセージ"}
        :examples ["(greet \"Alice\") ;=> \"Hello, Alice!\""]}}
  (fn [name]
    (str "Hello, " name "!")))
```

### 3. 外部参照形式（大規模）

複雑な関数や大量のドキュメントを外部ファイルで管理。

```lisp
(def complex-function
  :see-ja "docs/ja/complex-function.qi"
  :see-en "docs/en/complex-function.qi"
  (fn [x y z]
    ;; 実装
    ))
```

## 外部ファイル形式

### ディレクトリ構造

```
project/
  main.qi              # メインソース
  docs/
    ja/
      my-module.qi     # モジュール単位のドキュメント
    en/
      my-module.qi
```

標準ライブラリ：
```
qi (バイナリ)
std/
  ja/
    io-module.qi
    list-module.qi
    time-module.qi
  en/
    io-module.qi
    list-module.qi
    time-module.qi
```

### 外部ファイル内の記述

外部ファイル内でも文字列・構造化どちらでも記述可能。

**文字列形式**：
```lisp
{
  greet "指定された名前で挨拶する"
  farewell "指定された名前で別れの挨拶をする"
}
```

**構造化形式**：
```lisp
{
  greet {
    :desc "指定された名前で挨拶する"
    :params [{:name "name" :type "string" :desc "挨拶する相手の名前"}]
    :returns {:type "string" :desc "挨拶メッセージ"}
    :examples [
      "(greet \"Alice\") ;=> \"Hello, Alice!\""
    ]
  }

  farewell {
    :desc "指定された名前で別れの挨拶をする"
    :params [{:name "name" :type "string" :desc "別れる相手の名前"}]
    :returns {:type "string" :desc "別れのメッセージ"}
  }
}
```

**混在も可能**：
```lisp
{
  greet "指定された名前で挨拶する"  ; シンプルな関数は文字列

  complex-function {                  ; 複雑な関数は構造化
    :desc "複雑な処理を実行する"
    :params [...]
    :examples [...]
  }
}
```

## 言語フォールバック

ドキュメント表示時の優先順位：

1. **現在の言語**（環境変数 `QI_LANG`）
2. **英語**（`en`）
3. **表示なし**

例：
```bash
export QI_LANG=ja
qi repl
> :doc greet   # 日本語で表示

export QI_LANG=fr
qi repl
> :doc greet   # フランス語が未定義なら英語で表示
```

## ドキュメント読み込み優先順位

### REPLでの `:doc` コマンド（インタラクティブ検索）

1. **カレントディレクトリ**のドキュメント（ユーザー定義優先）
2. **qiバイナリディレクトリ**の `std/` 配下（標準ライブラリ）

```lisp
qi> :doc read
[fzf風のインタラクティブ検索]
3. my-read-config      設定ファイルを読み込む（ユーザー定義）
1. io/read-file        ファイルの内容を読み込む
2. io/read-bytes       バイナリファイルを読み込む

> _

[選択後]
doc: my-read-config
説明: プロジェクトの設定ファイル（config.json）を読み込んでパースします
引数: なし
戻り値: 設定のマップ
例:
  (my-read-config)
  ;=> {:host "localhost" :port 8080}
```

### qiコマンド（一括出力）

デフォルトはユーザードキュメントのみ出力。

```bash
# ユーザードキュメントをmarkdown一括出力
qi --gen-doc > my-functions.md

# 標準ライブラリ含めて全部出力
qi --gen-doc --include-std > all-functions.md
```

## 実装方針

### メモリ効率

- **遅延読み込み**：`:doc` コマンド実行時のみロード
- **ファイルシステムベース**：バイナリに埋め込まない
- **必要最小限**：検索時は関数名+短い説明のみ、詳細は選択後

### パフォーマンス

- ドキュメント参照は頻繁な操作ではないため、速度よりメモリ効率を優先
- キャッシュは実装しない（シンプルさ重視）

## 設計思想

### データ駆動

- ドキュメントもQiのデータ構造（文字列、マップ）で記述
- マクロではなく、データとして扱う

### フロー指向

- パイプライン処理との親和性を保つ
- ドキュメント取得もデータフローの一部として設計可能

### 段階的詳細化

- シンプルな文字列から始められる
- 必要に応じて構造化、外部ファイルへと拡張
- 強制しない、選択肢を提供する

### 明確な分離

- **Rustレイヤー**：`i18n.rs`（エラーメッセージの内部i18n）
- **Qiレイヤー**：本ドキュメントシステム（ユーザー向けi18n）

これらは異なる目的を持ち、混同しないこと。

## 今後の拡張

- `:doc` コマンドのインタラクティブ検索実装
- `qi --gen-doc` コマンドの実装
- より多くの言語サポート（中国語、韓国語など）
- ドキュメントのバリデーション機能
- 型情報との統合
