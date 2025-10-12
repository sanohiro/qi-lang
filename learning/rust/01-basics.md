# Rust基本文法

Rustの基本的な文法を、qi-langの実装を通じて学びます。

## 構造体（Struct）

構造体は**名前付きフィールド**を持つデータ型です。

### 基本的な構造体

```rust
// src/lexer.rs
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}
```

**説明:**
- `pub`: 公開（他のモジュールから使える）
- `struct Lexer`: 構造体名
- `input`, `pos`, `line`, `column`: フィールド（メンバ変数）
- `Vec<char>`: ジェネリック型（後述）
- `usize`: 符号なし整数型

### 構造体のメソッド

```rust
impl Lexer {
    // 関連関数（静的メソッド）
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    // メソッド（インスタンスメソッド）
    fn current(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
        self.column += 1;
    }
}
```

**ポイント:**
- `impl Lexer { ... }`: `Lexer`の実装ブロック
- `new`: 慣習的にコンストラクタ（関連関数）
- `&self`: 不変参照（読み取り専用）
- `&mut self`: 可変参照（書き込み可能）
- `Self`: 型エイリアス（ここでは`Lexer`）

### 使用例

```rust
let mut lexer = Lexer::new("(+ 1 2)");
let ch = lexer.current();  // Some('(')
lexer.advance();           // 次の文字へ
```

## 列挙型（Enum）

列挙型は**複数のバリアント**を持つ型です。Rustの`enum`は非常に強力です。

### 基本的なenum

```rust
// src/lexer.rs
pub enum Token {
    // データなし
    LParen,
    RParen,
    Nil,

    // データ付き
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),

    // タプル型（複数のデータ）
    // 例: FString(Vec<FStringPart>)
}
```

**特徴:**
- 各バリアントは異なる型のデータを持てる
- `Integer(i64)`: i64型の値を持つ
- `String(String)`: String型の値を持つ
- `LParen`: データなし

### より複雑なenum

```rust
// src/value.rs
pub enum Expr {
    // シンプルなバリアント
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),

    // 構造体的なバリアント
    If {
        test: Box<Expr>,
        then: Box<Expr>,
        otherwise: Option<Box<Expr>>,
    },

    Fn {
        params: Vec<String>,
        body: Box<Expr>,
        is_variadic: bool,
    },

    // リスト型のバリアント
    List(Vec<Expr>),
    Vector(Vec<Expr>),
}
```

**ポイント:**
- `Box<Expr>`: ヒープ割り当て（再帰的な構造に必要）
- `Option<Box<Expr>>`: 省略可能なフィールド
- `Vec<String>`: 文字列のベクタ

### enumのサイズ

```rust
// 判別子（どのバリアントか）+ 最大バリアントのサイズ
// 例: Option<T>
enum Option<T> {
    Some(T),  // T のサイズ
    None,     // サイズ0
}
// Option<T> のサイズ ≈ size_of::<T>() + 判別子
```

## パターンマッチ（Match）

`match`式はenumの分解に使います。Rustで最も強力な機能の一つです。

### 基本的なmatch

```rust
// src/parser.rs
fn parse_primary(&mut self) -> Result<Expr, String> {
    match self.current() {
        Some(Token::Integer(n)) => {
            let n = *n;
            self.advance();
            Ok(Expr::Integer(n))
        }
        Some(Token::String(s)) => {
            let s = s.clone();
            self.advance();
            Ok(Expr::String(s))
        }
        Some(Token::LParen) => self.parse_list(),
        None => Err("Unexpected EOF".to_string()),
        _ => Err("Unexpected token".to_string()),
    }
}
```

**ポイント:**
- 各パターンは順番に評価される
- `Some(Token::Integer(n))`: 値を変数`n`に束縛
- `*n`: 参照を外す（デリファレンス）
- `_`: ワイルドカードパターン（何でもマッチ）
- すべてのケースを網羅する必要がある（**網羅性チェック**）

### ガードパターン

```rust
match value {
    Value::Integer(n) if n > 0 => println!("正の整数"),
    Value::Integer(n) if n < 0 => println!("負の整数"),
    Value::Integer(0) => println!("ゼロ"),
    _ => println!("その他"),
}
```

`if`でさらに条件を追加できます。

### 複数パターン

```rust
match token {
    Token::LParen | Token::LBracket | Token::LBrace => {
        println!("開き括弧");
    }
    _ => {}
}
```

`|`で複数パターンをまとめられます。

### ネストしたパターン

```rust
// src/eval.rs
match expr {
    Expr::Call { func, args } => {
        match func.as_ref() {
            Expr::Symbol(name) => {
                // シンボル名による分岐
            }
            _ => {
                // その他の関数
            }
        }
    }
    _ => {}
}
```

パターンマッチはネストできます。

## if let 式

`match`の簡略版。1つのパターンだけをチェックしたい場合に便利です。

```rust
// matchの場合
match self.current() {
    Some(Token::Symbol(s)) => {
        let name = s.clone();
        // 処理...
    }
    _ => {}
}

// if let の場合
if let Some(Token::Symbol(s)) = self.current() {
    let name = s.clone();
    // 処理...
}
```

**使い分け:**
- 複数のケースを扱う → `match`
- 1つのケースだけ関心がある → `if let`

### else if let

```rust
if let Some(Token::Integer(n)) = token {
    println!("整数: {}", n);
} else if let Some(Token::String(s)) = token {
    println!("文字列: {}", s);
} else {
    println!("その他");
}
```

## while let 式

ループでパターンマッチを使います。

```rust
// src/lexer.rs
while let Some(ch) = self.current() {
    if ch.is_numeric() {
        num_str.push(ch);
        self.advance();
    } else {
        break;
    }
}
```

**意味**: `self.current()`が`Some`を返す間ループ

## Option型

Rustには**null**がありません。代わりに`Option`型を使います。

```rust
pub enum Option<T> {
    Some(T),
    None,
}
```

### 使用例

```rust
// src/lexer.rs
fn current(&self) -> Option<char> {
    if self.pos < self.input.len() {
        Some(self.input[self.pos])
    } else {
        None
    }
}

// 使う側
match lexer.current() {
    Some(ch) => println!("文字: {}", ch),
    None => println!("終端"),
}
```

### Optionの便利メソッド

```rust
// unwrap_or: デフォルト値を指定
let ch = lexer.current().unwrap_or(' ');

// map: Some の中身を変換
let upper = Some("hello").map(|s| s.to_uppercase());
// => Some("HELLO")

// and_then: フラットマップ
let result = Some(5).and_then(|n| {
    if n > 0 {
        Some(n * 2)
    } else {
        None
    }
});
// => Some(10)
```

## Result型

エラーハンドリングに使います。

```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 使用例

```rust
// src/parser.rs
fn parse(&mut self) -> Result<Expr, String> {
    match self.current() {
        Some(token) => {
            // 処理...
            Ok(expr)
        }
        None => Err("Unexpected EOF".to_string()),
    }
}

// 使う側
match parser.parse() {
    Ok(expr) => println!("成功: {:?}", expr),
    Err(e) => eprintln!("エラー: {}", e),
}
```

### ? 演算子

エラーを上位に伝搬します。

```rust
fn parse_list(&mut self) -> Result<Expr, String> {
    self.expect(Token::LParen)?;  // エラーなら早期リターン

    let first = self.parse_primary()?;
    let second = self.parse_primary()?;

    self.expect(Token::RParen)?;

    Ok(Expr::List(vec![first, second]))
}
```

**動作:**
- `Ok(value)` → `value`を取り出して続行
- `Err(e)` → 関数から`Err(e)`を返す

## ジェネリック型

型パラメータで汎用的な型を定義します。

```rust
// 標準ライブラリ
pub struct Vec<T> {
    // T 型の要素を持つベクタ
}

pub enum Option<T> {
    Some(T),
    None,
}

pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 使用例

```rust
// Vec<Value>: Value型のベクタ
let values: Vec<Value> = vec![
    Value::Integer(1),
    Value::Integer(2),
    Value::Integer(3),
];

// Option<String>: 文字列かNone
let name: Option<String> = Some("Alice".to_string());

// Result<Value, String>: 成功ならValue、失敗ならString
fn eval() -> Result<Value, String> {
    // ...
}
```

## Box型

ヒープ割り当てに使います。再帰的なデータ構造で必須です。

```rust
// 悪い例（コンパイルエラー）
enum Expr {
    Integer(i64),
    List(Vec<Expr>),  // OK（Vecはポインタを内部で持つ）
    If(Expr, Expr, Expr),  // エラー！無限サイズ
}

// 良い例
enum Expr {
    Integer(i64),
    List(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),  // OK
}
```

**理由:**
- `Expr`のサイズを決定するのに`Expr`のサイズが必要 → 無限再帰
- `Box<Expr>`はポインタサイズ（固定） → OK

### Boxの使い方

```rust
// 作成
let boxed = Box::new(Expr::Integer(42));

// デリファレンス
match *boxed {
    Expr::Integer(n) => println!("{}", n),
    _ => {}
}

// as_ref() で参照を取得
match boxed.as_ref() {
    Expr::Integer(n) => println!("{}", n),
    _ => {}
}
```

## モジュールシステム

Rustはモジュールでコードを整理します。

### モジュール宣言

```rust
// src/lib.rs
pub mod lexer;    // src/lexer.rs を読み込む
pub mod parser;   // src/parser.rs を読み込む
pub mod eval;     // src/eval.rs を読み込む
pub mod value;    // src/value.rs を読み込む
pub mod builtins; // src/builtins/mod.rs を読み込む
```

### use文

```rust
// src/parser.rs
use crate::lexer::{Lexer, Token};
use crate::value::{Expr, Value};

// crate: 現在のクレート（プロジェクト）のルート
// :: パス区切り
```

### 公開範囲

```rust
pub struct Lexer { ... }        // 公開
pub fn new() { ... }            // 公開
fn advance(&mut self) { ... }   // プライベート

pub(crate) fn internal() { ... } // クレート内で公開
```

## デバッグとテスト

### デバッグ出力

```rust
#[derive(Debug)]
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

let lexer = Lexer::new("test");
println!("{:?}", lexer);
// => Lexer { input: ['t', 'e', 's', 't'], pos: 0 }
```

`#[derive(Debug)]`でデバッグ出力を自動生成

### ユニットテスト

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
}
```

```bash
# テスト実行
cargo test
```

## まとめ

Rustの基本文法を学びました：

- **struct**: データの集約
- **enum**: バリアント型（代数的データ型）
- **match**: パターンマッチ
- **Option**: nullの代替
- **Result**: エラーハンドリング
- **Box**: ヒープ割り当て

これらを組み合わせることで、安全で表現力豊かなコードが書けます。

## 次のステップ

次は[所有権システム](./02-ownership.md)を学びます。Rustの最も独特で重要な概念です。
