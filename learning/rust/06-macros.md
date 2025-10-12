# マクロ

Rustのマクロシステムについて学びます。マクロは**コンパイル時にコードを生成**する強力な機能です。

## マクロとは

マクロは**構文の拡張**を可能にします。関数との違い：

| | 関数 | マクロ |
|---|---|---|
| 実行時 | 実行時 | コンパイル時 |
| 引数 | 評価済みの値 | 構文木（AST） |
| 型チェック | 引数の型が固定 | 任意の構文を受け入れ可能 |
| 可変引数 | 制限あり | 柔軟 |

## 宣言的マクロ（macro_rules!）

パターンマッチでコードを生成します。

### 基本的なマクロ

```rust
// vec! マクロの簡易版
macro_rules! my_vec {
    // パターン => 展開
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

// 使用
let v = my_vec![1, 2, 3];
```

**構文:**
- `$( ... )*`: 0回以上の繰り返し
- `$( ... ),*`: カンマ区切りの繰り返し
- `$x:expr`: 式を捕捉する変数

### マクロのパターン

```rust
macro_rules! create_function {
    // ident: 識別子
    ($func_name:ident) => {
        fn $func_name() {
            println!("You called {:?}()", stringify!($func_name));
        }
    };
}

create_function!(foo);
create_function!(bar);

foo();  // "You called foo()"
bar();  // "You called bar()"
```

**キャプチャの種類:**
- `ident`: 識別子（変数名、関数名等）
- `expr`: 式
- `ty`: 型
- `pat`: パターン
- `stmt`: 文
- `block`: ブロック
- `item`: アイテム（関数、struct等）
- `tt`: トークンツリー

### 複数のパターン

```rust
macro_rules! print_type {
    ($val:expr) => {
        println!("{:?}", $val);
    };
    ($val:expr, $msg:expr) => {
        println!("{}: {:?}", $msg, $val);
    };
}

print_type!(42);           // "42"
print_type!(42, "Value");  // "Value: 42"
```

### qi-langでの使用例

```rust
// src/builtins/mod.rs
macro_rules! register_native {
    ($env:expr, $($name:expr => $func:expr),* $(,)?) => {
        $(
            $env.set(
                $name.to_string(),
                Value::NativeFunc(NativeFunc {
                    name: $name.to_string(),
                    func: $func,
                }),
            );
        )*
    };
}

// 使用
register_native!(env.write(),
    "+" => core_numeric::native_add,
    "-" => core_numeric::native_sub,
    "*" => core_numeric::native_mul,
    "/" => core_numeric::native_div,
);
```

**効果:**
- 繰り返しコードを削減
- 一貫性のある登録処理
- 読みやすいAPI

## 標準マクロ

Rustの標準ライブラリには多くの便利なマクロがあります。

### println! / format!

```rust
// println! - 標準出力に書き込む
println!("Hello, world!");
println!("x = {}", x);
println!("x = {}, y = {}", x, y);

// format! - 文字列を生成
let s = format!("x = {}", x);

// eprintln! - 標準エラー出力
eprintln!("Error: {}", msg);
```

### vec!

```rust
let v = vec![1, 2, 3, 4, 5];

// 展開後（概念的に）
let v = {
    let mut temp = Vec::new();
    temp.push(1);
    temp.push(2);
    temp.push(3);
    temp.push(4);
    temp.push(5);
    temp
};
```

### assert! / assert_eq!

```rust
// assert! - 条件をチェック
assert!(x > 0);
assert!(x > 0, "x must be positive, got {}", x);

// assert_eq! - 等価性をチェック
assert_eq!(x, 5);
assert_eq!(x, 5, "x should be 5, got {}", x);

// テストで使用
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}
```

### matches!

```rust
let x = Some(5);

// matches! - パターンマッチをブール値に
if matches!(x, Some(5)) {
    println!("x is Some(5)");
}

// 展開後（概念的に）
if match x {
    Some(5) => true,
    _ => false,
} {
    println!("x is Some(5)");
}
```

## デバッグ用マクロ

### dbg!

```rust
let x = 5;
let y = dbg!(x * 2);  // [src/main.rs:2] x * 2 = 10

// 複雑な式のデバッグ
let result = dbg!(expensive_function());
```

### compile_error!

```rust
#[cfg(not(target_os = "linux"))]
compile_error!("This program only runs on Linux");
```

### unimplemented! / todo!

```rust
fn some_function() {
    unimplemented!("Not yet implemented");
}

fn another_function() {
    todo!("Will implement later");
}
```

## 属性マクロ

`#[...]`の形式で使います。

### derive

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

// Debug, Clone, PartialEq の実装が自動生成される
```

**標準のderive:**
- `Debug`: デバッグ出力
- `Clone`: クローン
- `Copy`: コピー
- `PartialEq`, `Eq`: 等価性比較
- `PartialOrd`, `Ord`: 順序比較
- `Hash`: ハッシュ

### cfg

条件付きコンパイル：

```rust
#[cfg(target_os = "linux")]
fn platform_specific() {
    println!("Linux");
}

#[cfg(target_os = "windows")]
fn platform_specific() {
    println!("Windows");
}

#[cfg(test)]
mod tests {
    // テスト時のみコンパイル
}

#[cfg(feature = "http-client")]
pub mod http;
```

### test

```rust
#[test]
fn test_addition() {
    assert_eq!(2 + 2, 4);
}

#[test]
#[should_panic]
fn test_panic() {
    panic!("This should panic");
}

#[test]
#[ignore]
fn expensive_test() {
    // 通常は実行されない
}
```

## 手続き的マクロ

より強力なマクロです（高度）。

### カスタムderive

```rust
// 使用例
#[derive(MyTrait)]
struct MyStruct {
    field: i32,
}

// 実装（別クレートで）
use proc_macro::TokenStream;

#[proc_macro_derive(MyTrait)]
pub fn my_trait_derive(input: TokenStream) -> TokenStream {
    // 構文木を解析してコードを生成
    // ...
}
```

### 関数風マクロ

```rust
// 使用例
let sql = sql!(SELECT * FROM users WHERE id = ?);

// 実装（別クレートで）
#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    // SQL構文を解析して型安全なコードを生成
    // ...
}
```

### 属性マクロ

```rust
// 使用例
#[route(GET, "/")]
fn index() -> String {
    "Hello".to_string()
}

// 実装（別クレートで）
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    // ルーティング情報を処理
    // ...
}
```

## マクロのデバッグ

### cargo expand

マクロの展開結果を見ることができます：

```bash
cargo install cargo-expand
cargo expand
```

### println!デバッグ

```rust
macro_rules! debug_vec {
    ( $( $x:expr ),* ) => {
        {
            println!("Creating vec with:");
            $(
                println!("  {:?}", $x);
            )*
            vec![$( $x ),*]
        }
    };
}
```

## qi-langでのマクロ活用

### ビルトイン関数登録

```rust
// src/builtins/mod.rs
register_native!(env.write(),
    // Core: 数値・比較演算
    "+" => core_numeric::native_add,
    "-" => core_numeric::native_sub,
    "*" => core_numeric::native_mul,

    // Core: リスト操作
    "first" => core_collections::native_first,
    "rest" => core_collections::native_rest,

    // ... 300個以上の関数
);
```

マクロなしで書くと：

```rust
env.write().set(
    "+".to_string(),
    Value::NativeFunc(NativeFunc {
        name: "+".to_string(),
        func: core_numeric::native_add,
    }),
);
env.write().set(
    "-".to_string(),
    Value::NativeFunc(NativeFunc {
        name: "-".to_string(),
        func: core_numeric::native_sub,
    }),
);
// ... 300回繰り返し
```

### デバッグ出力

```rust
// src/value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Integer(i64),
    // ...
}

// 使用
let value = Value::Integer(42);
println!("{:?}", value);  // "Integer(42)"
```

### テスト

```rust
// src/lexer.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_integer() {
        let mut lexer = Lexer::new("42");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Integer(42), Token::Eof]);
    }

    #[test]
    fn test_tokenize_string() {
        let mut lexer = Lexer::new(r#""hello""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::String("hello".to_string()), Token::Eof]);
    }
}
```

## マクロのベストプラクティス

### 1. マクロ名は`!`で終わる

```rust
// 良い例
vec![1, 2, 3]
println!("Hello")

// 悪い例（関数と区別がつかない）
// vec[1, 2, 3]  // マクロではない
```

### 2. 衛生性（Hygiene）

マクロは変数の衝突を防ぎます：

```rust
macro_rules! safe_macro {
    () => {
        let x = 42;  // マクロ内のxは外側のxと衝突しない
    };
}

let x = 10;
safe_macro!();
println!("{}", x);  // 10（マクロのxとは別）
```

### 3. エラーメッセージ

```rust
macro_rules! create_function {
    ($name:ident) => {
        fn $name() {
            println!("Function {}", stringify!($name));
        }
    };
    () => {
        compile_error!("create_function requires a function name");
    };
}
```

### 4. ドキュメント

```rust
/// ベクタを作成するマクロ
///
/// # 例
///
/// ```
/// let v = my_vec![1, 2, 3];
/// assert_eq!(v, vec![1, 2, 3]);
/// ```
macro_rules! my_vec {
    // ...
}
```

## まとめ

Rustのマクロシステム：

1. **宣言的マクロ** (`macro_rules!`)
   - パターンマッチでコード生成
   - 繰り返しの削減
   - qi-langの`register_native!`

2. **標準マクロ**
   - `println!`, `vec!`, `assert!`等
   - 日常的に使用

3. **属性マクロ**
   - `#[derive]`: 自動実装
   - `#[cfg]`: 条件付きコンパイル
   - `#[test]`: テスト

4. **手続き的マクロ**（高度）
   - より柔軟なコード生成
   - 別クレートで実装

5. **ベストプラクティス**
   - マクロ名に`!`
   - 衛生性を活用
   - 良いエラーメッセージ
   - ドキュメント化

マクロにより、Rustは**メタプログラミング**の能力を持ちます。

## Rustの学習完了

これでRustの基礎を学びました：

1. ✅ [基本文法](./01-basics.md) - struct, enum, match
2. ✅ [所有権](./02-ownership.md) - 所有権、借用、ライフタイム
3. ✅ [Result/Option](./03-result-option.md) - エラーハンドリング
4. ✅ [コレクション](./04-collections.md) - Vec, HashMap, イテレータ
5. ✅ [並行処理](./05-concurrency.md) - Arc, RwLock, スレッド
6. ✅ [マクロ](./06-macros.md) - メタプログラミング

これらの知識を使って、qi-langのコードを読み解き、改良していきましょう！
