# Result と Option

RustのエラーハンドリングとNull安全性について学びます。

## Option型

Rustには**null**がありません。代わりに`Option<T>`型を使います。

### 定義

```rust
pub enum Option<T> {
    Some(T),
    None,
}
```

### 基本的な使い方

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
    None => println!("終端に達しました"),
}
```

### Optionの便利メソッド

#### unwrap - 値を取り出す（危険）

```rust
let x = Some(5);
let value = x.unwrap();  // 5

let y: Option<i32> = None;
// let value = y.unwrap();  // パニック！
```

**注意**: `unwrap()`は`None`の場合パニックします。本番コードでは避けましょう。

#### unwrap_or - デフォルト値を指定

```rust
let x = Some(5);
let value = x.unwrap_or(0);  // 5

let y: Option<i32> = None;
let value = y.unwrap_or(0);  // 0
```

#### unwrap_or_else - デフォルト値を計算

```rust
let x: Option<i32> = None;
let value = x.unwrap_or_else(|| {
    println!("デフォルト値を計算中...");
    42
});
```

#### map - 値を変換

```rust
let x = Some("hello");
let upper = x.map(|s| s.to_uppercase());
// => Some("HELLO")

let y: Option<&str> = None;
let upper = y.map(|s| s.to_uppercase());
// => None
```

#### and_then - フラットマップ

```rust
fn parse_int(s: &str) -> Option<i32> {
    s.parse().ok()
}

let x = Some("42");
let result = x.and_then(parse_int);
// => Some(42)

let y = Some("hello");
let result = y.and_then(parse_int);
// => None
```

#### or - 代替のOptionを試す

```rust
let x: Option<i32> = None;
let y = Some(100);
let result = x.or(y);
// => Some(100)
```

#### filter - 条件でフィルタ

```rust
let x = Some(5);
let result = x.filter(|&n| n > 3);
// => Some(5)

let y = Some(2);
let result = y.filter(|&n| n > 3);
// => None
```

### qi-langでの使用例

```rust
// src/value.rs
impl Env {
    pub fn get(&self, name: &str) -> Option<Value> {
        self.bindings
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.read().get(name)))
    }
}
```

**解説:**
1. `self.bindings.get(name)` → 現在の環境から検索
2. `.cloned()` → 参照を値に変換
3. `.or_else(...)` → 見つからなければ親環境で検索

## Result型

エラーハンドリングに使います。

### 定義

```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

### 基本的な使い方

```rust
// src/lexer.rs
fn read_string(&mut self) -> Result<String, String> {
    self.advance();  // 先頭の "
    let mut result = String::new();

    while let Some(ch) = self.current() {
        if ch == '"' {
            self.advance();
            return Ok(result);
        }
        result.push(ch);
        self.advance();
    }

    Err("Unclosed string".to_string())
}
```

### Resultの便利メソッド

#### unwrap - 値を取り出す（危険）

```rust
let x: Result<i32, &str> = Ok(5);
let value = x.unwrap();  // 5

let y: Result<i32, &str> = Err("エラー");
// let value = y.unwrap();  // パニック！
```

#### expect - パニック時のメッセージを指定

```rust
let x: Result<i32, &str> = Err("エラー");
// let value = x.expect("値の取得に失敗");  // パニック: "値の取得に失敗: エラー"
```

#### unwrap_or / unwrap_or_else

```rust
let x: Result<i32, &str> = Err("エラー");
let value = x.unwrap_or(0);  // 0

let y: Result<i32, &str> = Err("エラー");
let value = y.unwrap_or_else(|e| {
    eprintln!("エラー: {}", e);
    -1
});
```

#### map - 成功時の値を変換

```rust
let x: Result<i32, &str> = Ok(5);
let result = x.map(|n| n * 2);
// => Ok(10)

let y: Result<i32, &str> = Err("エラー");
let result = y.map(|n| n * 2);
// => Err("エラー")
```

#### map_err - エラーを変換

```rust
let x: Result<i32, &str> = Err("エラー");
let result = x.map_err(|e| format!("Failed: {}", e));
// => Err("Failed: エラー")
```

#### and_then - フラットマップ

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

let x = Ok(10);
let result = x.and_then(|n| divide(n, 2));
// => Ok(5)

let y = Ok(10);
let result = y.and_then(|n| divide(n, 0));
// => Err("Division by zero")
```

## ? 演算子

エラーを上位に伝搬する最も重要な機能です。

### 基本的な使い方

```rust
// 手動でのエラー処理
fn parse_list_manual(&mut self) -> Result<Expr, String> {
    match self.expect(Token::LParen) {
        Ok(()) => {},
        Err(e) => return Err(e),
    }

    let first = match self.parse_primary() {
        Ok(expr) => expr,
        Err(e) => return Err(e),
    };

    match self.expect(Token::RParen) {
        Ok(()) => {},
        Err(e) => return Err(e),
    }

    Ok(Expr::List(vec![first]))
}

// ? 演算子を使った簡潔な書き方
fn parse_list(&mut self) -> Result<Expr, String> {
    self.expect(Token::LParen)?;
    let first = self.parse_primary()?;
    self.expect(Token::RParen)?;

    Ok(Expr::List(vec![first]))
}
```

### ? 演算子の動作

```rust
// expr? は以下と同等
match expr {
    Ok(val) => val,
    Err(e) => return Err(e.into()),  // From トレイトで変換
}
```

### qi-langでの使用例

```rust
// src/parser.rs
fn parse_if(&mut self) -> Result<Expr, String> {
    self.advance();  // 'if'をスキップ

    let test = Box::new(self.parse_expr()?);
    let then = Box::new(self.parse_expr()?);

    let otherwise = if self.current() != Some(&Token::RParen) {
        Some(Box::new(self.parse_expr()?))
    } else {
        None
    };

    self.expect(Token::RParen)?;

    Ok(Expr::If { test, then, otherwise })
}
```

**ポイント:**
- 各`?`でエラーが発生したら、即座に関数から返る
- エラーが発生しなければ、値を取り出して処理を続ける

### エラー型の変換

```rust
use std::fs;
use std::io;

fn read_file(path: &str) -> Result<String, String> {
    // io::Error を String に変換
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(content)
}
```

## イテレータとResult/Option

### collect でエラーを集約

```rust
// src/eval.rs
Expr::List(items) => {
    let values: Result<Vec<_>, _> = items
        .iter()
        .map(|e| self.eval_with_env(e, env.clone()))
        .collect();
    Ok(Value::List(values?))
}
```

**動作:**
- 各要素を評価
- 1つでもエラーなら全体が`Err`
- すべて成功なら`Ok(Vec<Value>)`

### filter_map で Some のみ抽出

```rust
let numbers = vec![Some(1), None, Some(2), None, Some(3)];
let valid: Vec<i32> = numbers.into_iter().filter_map(|x| x).collect();
// => [1, 2, 3]
```

### flatten で入れ子を平坦化

```rust
let nested = vec![Some(vec![1, 2]), None, Some(vec![3, 4])];
let flat: Vec<i32> = nested.into_iter().flatten().flatten().collect();
// => [1, 2, 3, 4]
```

## エラーハンドリングのベストプラクティス

### 1. パニックを避ける

```rust
// 悪い例
let value = result.unwrap();  // パニックの可能性

// 良い例
let value = result?;  // エラーを上位に伝搬
let value = result.unwrap_or_default();  // デフォルト値
```

### 2. 具体的なエラーメッセージ

```rust
// 悪い例
Err("Error".to_string())

// 良い例
Err(format!("Failed to parse token at line {}, column {}", line, col))
```

### 3. エラーの伝搬

```rust
// 悪い例 - エラーを握りつぶす
fn process() {
    let _ = do_something();  // エラーを無視
}

// 良い例 - エラーを伝搬
fn process() -> Result<(), String> {
    do_something()?;
    Ok(())
}
```

### 4. カスタムエラー型

```rust
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    UnexpectedEof,
    InvalidNumber(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedToken(tok) => write!(f, "Unexpected token: {}", tok),
            ParseError::UnexpectedEof => write!(f, "Unexpected end of file"),
            ParseError::InvalidNumber(num) => write!(f, "Invalid number: {}", num),
        }
    }
}
```

## qi-langでの実践例

### エラーの伝搬チェーン

```rust
// src/main.rs
fn run_file(path: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;

    let mut parser = Parser::new(&content)?;
    let exprs = parser.parse_all()?;

    let evaluator = Evaluator::new();
    for expr in exprs {
        evaluator.eval(&expr)?;
    }

    Ok(())
}

// 使用側
if let Err(e) = run_file("script.qi") {
    eprintln!("Error: {}", e);
    std::process::exit(1);
}
```

### 複雑なエラー処理

```rust
// src/eval.rs
fn eval_with_env(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
    match expr {
        Expr::Symbol(name) => {
            env.read().get(name).ok_or_else(|| {
                // 類似した変数名を検索
                let suggestions = find_similar_names(&env.read(), name, 3, 3);
                if suggestions.is_empty() {
                    format!("Undefined variable: {}", name)
                } else {
                    format!("Undefined variable: {}. Did you mean: {}?",
                            name, suggestions.join(", "))
                }
            })
        }
        // ...
    }
}
```

## まとめ

Result と Option を使ったエラーハンドリング：

1. **Option**: nullの代替
   - `Some(T)` / `None`
   - null参照エラーを防ぐ

2. **Result**: エラーハンドリング
   - `Ok(T)` / `Err(E)`
   - 例外機構の代替

3. **? 演算子**: エラー伝搬
   - 簡潔な記述
   - 早期リターン

4. **ベストプラクティス**:
   - パニックを避ける
   - 具体的なエラーメッセージ
   - エラーを適切に伝搬

これらにより、Rustは**型安全なエラーハンドリング**を実現します。

## 次のステップ

次は[コレクション](./04-collections.md)を学びます。Vec、HashMap、イテレータなどの使い方を理解しましょう。
