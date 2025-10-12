# Rustで学ぶプログラミング言語実装

このディレクトリでは、Qi言語の実装を通じて**Rustプログラミング**を学びます。

Rust初心者がqi-langのコードベースを理解できるように、実際の実装例を使ってRustの基本概念を解説します。

## なぜRustを学ぶのか？

Rustは**安全性**と**パフォーマンス**を両立する言語です：

- **メモリ安全性**: ガベージコレクションなしでメモリ安全を保証
- **並行性**: データ競合を型システムで防ぐ
- **ゼロコスト抽象化**: 高レベルの抽象化でもパフォーマンスを犠牲にしない
- **パターンマッチ**: 強力な構造化データの分解

これらの特性により、Rustは**言語処理系**や**システムプログラミング**に最適です。

## 学習パス

Qi言語の実装を読みながら、以下の順序でRustを学びましょう：

1. **[基本文法](./01-basics.md)** - struct, enum, match, if let
2. **[所有権システム](./02-ownership.md)** - 所有権、借用、ライフタイム
3. **[エラーハンドリング](./03-result-option.md)** - Result, Option, ? 演算子
4. **[コレクション](./04-collections.md)** - Vec, HashMap, イテレータ
5. **[並行処理](./05-concurrency.md)** - Arc, RwLock, スレッド
6. **[マクロ](./06-macros.md)** - 宣言的マクロと手続き的マクロ

## このドキュメントの特徴

### 1. 実際のコードを使う

理論だけでなく、**qi-langの実装コード**を具体例として使います：

```rust
// src/value.rs から抜粋
pub enum Value {
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    // ...
}
```

このコードを通じて：
- `enum`の定義方法
- バリアント（列挙子）の書き方
- ジェネリック型（`Vec<Value>`）の使い方

を学びます。

### 2. 段階的な学習

簡単な概念から始めて、徐々に高度な概念へ：

```
基本文法 → 所有権 → エラーハンドリング → コレクション → 並行処理 → マクロ
```

### 3. 実践的な例

言語処理系の実装という**実用的なコンテキスト**で学びます：

- なぜこの設計にしたのか？
- 他の選択肢は？
- トレードオフは？

## 前提知識

以下の知識があると理解しやすいです：

- **プログラミング経験**: 他の言語（Python, JavaScript, Java等）でのプログラミング経験
- **基本的なデータ構造**: リスト、ハッシュマップ、木構造
- **再帰の理解**: 再帰的な関数とデータ構造

Rustは初めてでも大丈夫です！このドキュメントで基礎から学べます。

## Rustの特徴

### 所有権システム

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 は無効になる（ムーブ）
// println!("{}", s1);  // エラー！s1 はもう使えない
println!("{}", s2);  // OK
```

Rustは**所有権**という独自のメモリ管理システムを持ちます。これによりガベージコレクションなしでメモリ安全を実現します。

### 強力な型システム

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

エラーは**値として扱われ**、型システムで追跡されます。例外機構はありません。

### パターンマッチ

```rust
match value {
    Value::Integer(n) => println!("整数: {}", n),
    Value::String(s) => println!("文字列: {}", s),
    _ => println!("その他"),
}
```

構造化データを**分解**して、各ケースに対応する処理を書けます。

### トレイトによる多相性

```rust
trait Display {
    fn fmt(&self) -> String;
}

impl Display for Value {
    fn fmt(&self) -> String {
        // 実装...
    }
}
```

トレイトは**インターフェース**のような役割を果たします。

## 開発環境

### Rustのインストール

```bash
# rustup（Rustツールチェーンインストーラ）をインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# インストール確認
rustc --version
cargo --version
```

### qi-langのビルド

```bash
cd ~/Projects/qi-lang

# デバッグビルド
cargo build

# リリースビルド（最適化あり）
cargo build --release

# 実行
cargo run
```

### エディタ

Rust開発には**rust-analyzer**の使用を推奨します：

- VSCode: Rust Analyzer拡張機能
- Vim/Neovim: rust-analyzer LSP
- その他: 公式サイト参照

## 学習の進め方

1. **まず概要を読む**: 各章の最初に概要があります
2. **コードを読む**: qi-langの実装コードを読んでみましょう
3. **実験する**: コードを変更して挙動を確認
4. **写経する**: 理解したコードを自分で書いてみる

## 参考資料

- [The Rust Programming Language](https://doc.rust-lang.org/book/) - 公式の入門書
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/) - サンプルコードで学ぶ
- [Rustlings](https://github.com/rust-lang/rustlings/) - 小さな演習問題集

## 次のステップ

準備ができたら、[基本文法](./01-basics.md)から始めましょう！

Qi言語の実装を通じて、Rustの世界を楽しく学びましょう。
