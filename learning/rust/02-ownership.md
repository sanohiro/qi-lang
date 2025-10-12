# 所有権システム

Rustの最も独特で重要な概念が**所有権（Ownership）**です。これによりガベージコレクションなしでメモリ安全を実現します。

## メモリ管理の比較

### C/C++ - 手動管理

```c
char* str = malloc(100);  // 明示的に確保
// ... 使う ...
free(str);                // 明示的に解放（忘れるとリーク！）
```

**問題点:**
- メモリリーク（解放忘れ）
- ダングリングポインタ（解放済みメモリへのアクセス）
- ダブルフリー（2回解放）

### Java/Python - GC

```java
String str = new String("hello");
// ... 使う ...
// GCが自動で解放（いつかは不明）
```

**問題点:**
- 実行時オーバーヘッド
- 一時停止（GCポーズ）
- メモリ使用量の増加

### Rust - 所有権システム

```rust
let s = String::from("hello");
// スコープを抜けると自動で解放（確定的）
```

**利点:**
- コンパイル時にチェック（実行時オーバーヘッドなし）
- 確定的な解放（いつ解放されるか明確）
- データ競合の防止

## 所有権の3つのルール

1. **各値には所有者がいる**
2. **所有者は常に1つだけ**
3. **所有者がスコープを抜けると値は破棄される**

### ルール1: 各値には所有者がいる

```rust
let s = String::from("hello");
// s が文字列 "hello" の所有者
```

### ルール2: 所有者は常に1つだけ

```rust
let s1 = String::from("hello");
let s2 = s1;  // 所有権がs1からs2にムーブ

// println!("{}", s1);  // エラー！s1 はもう無効
println!("{}", s2);  // OK
```

### ルール3: スコープを抜けると破棄

```rust
{
    let s = String::from("hello");
    // s を使う
}  // ここで s がドロップ（メモリ解放）

// println!("{}", s);  // エラー！s はスコープ外
```

## ムーブ（Move）

所有権の移動を**ムーブ**と呼びます。

### 基本的なムーブ

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 から s2 へムーブ

// s1 は無効、s2 のみ有効
```

**メモリのイメージ:**
```
before:
s1 → [ptr, len, cap] → heap["hello"]

after move:
s1 (無効)
s2 → [ptr, len, cap] → heap["hello"]
```

### 関数呼び出しでのムーブ

```rust
fn take_ownership(s: String) {
    println!("{}", s);
}  // s がドロップされる

let s = String::from("hello");
take_ownership(s);  // s がムーブされる

// println!("{}", s);  // エラー！s は無効
```

### qi-langでの例

```rust
// src/eval.rs
Expr::Def(name, value) => {
    let val = self.eval_with_env(value, env.clone())?;
    env.write().set(name.clone(), val.clone());
    //                ^^^^^^^^^^^^  ^^^^^^^^^^^
    //                nameとvalの所有権を維持するためにクローン
    Ok(val)
}
```

## コピー（Copy）

一部の型は**コピー**されます（ムーブではなく）。

### Copyトレイトを持つ型

```rust
let x = 5;
let y = x;  // コピー（xは依然として有効）

println!("x = {}, y = {}", x, y);  // OK
```

**Copy型:**
- 整数型: `i32`, `i64`, `u32`, `usize`等
- 浮動小数点型: `f32`, `f64`
- ブール型: `bool`
- 文字型: `char`
- タプル（すべての要素がCopy型の場合）

**非Copy型:**
- `String`
- `Vec<T>`
- `Box<T>`
- その他のヒープ割り当て型

### qi-langでの例

```rust
// src/lexer.rs
fn advance(&mut self) {
    self.pos += 1;  // usize はCopy型 → ムーブではなくコピー
    self.column += 1;
}
```

## 借用（Borrowing）

所有権を移動せずに、**参照**を渡すことを**借用**と呼びます。

### 不変借用（Immutable Borrow）

```rust
fn calculate_length(s: &String) -> usize {
    s.len()  // 読み取りのみ
}  // s の参照がスコープを抜けるが、所有権はないので何もしない

let s1 = String::from("hello");
let len = calculate_length(&s1);  // s1 を借用
println!("{} の長さは {}", s1, len);  // s1 は依然として有効
```

**`&`**: 参照（借用）を作る

### 可変借用（Mutable Borrow）

```rust
fn append_world(s: &mut String) {
    s.push_str(", world");  // 変更可能
}

let mut s = String::from("hello");
append_world(&mut s);  // 可変借用
println!("{}", s);  // "hello, world"
```

**`&mut`**: 可変参照

### 借用のルール

1. **不変借用は複数同時に可能**
2. **可変借用は1つだけ**
3. **不変借用と可変借用は同時に不可**

```rust
let mut s = String::from("hello");

// OK: 複数の不変借用
let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2);

// OK: 可変借用（1つだけ）
let r3 = &mut s;
r3.push_str("!");

// エラー: 不変借用と可変借用の同時使用
let r4 = &s;
let r5 = &mut s;  // エラー！
println!("{} {}", r4, r5);
```

### qi-langでの例

```rust
// src/value.rs
impl Env {
    // 不変借用
    pub fn get(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    // 可変借用
    pub fn set(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }
}
```

## ライフタイム（Lifetime）

参照の**有効期間**をコンパイラに教えます。

### 基本的なライフタイム

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

let string1 = String::from("long string");
let string2 = String::from("short");

let result = longest(&string1, &string2);
println!("{}", result);
```

**`'a`**: ライフタイムパラメータ
- `x`, `y`, 戻り値はすべて同じライフタイムを持つ
- 戻り値は`x`または`y`の短い方のライフタイムを持つ

### 構造体のライフタイム

```rust
struct Parser<'a> {
    tokens: &'a [Token],  // トークン列への参照
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }
}
```

**意味:**
- `Parser`インスタンスは`tokens`より長く生きられない
- `Parser`がある限り、`tokens`は有効

### ライフタイムの省略

多くの場合、ライフタイムは省略できます（コンパイラが推論）：

```rust
// 明示的
fn first<'a>(s: &'a str) -> &'a str {
    &s[0..1]
}

// 省略可能
fn first(s: &str) -> &str {
    &s[0..1]
}
```

### qi-langでの例

実際には、qi-langでは`Arc`や`Clone`を使うことで、ライフタイムをあまり気にしなくて良いようにしています：

```rust
// src/eval.rs
pub struct Evaluator {
    global_env: Arc<RwLock<Env>>,  // 所有権の共有
    // ...
}

fn eval_with_env(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
    // env は Arc なので、ライフタイムを気にしなくて良い
}
```

## Clone と Copy

### Clone

明示的なコピー：

```rust
let s1 = String::from("hello");
let s2 = s1.clone();  // ディープコピー

println!("{} {}", s1, s2);  // 両方とも有効
```

### qi-langでの Clone の使用

```rust
// src/parser.rs
Some(Token::String(s)) => {
    let s = s.clone();  // String をクローン
    self.advance();
    Ok(Expr::String(s))
}
```

**理由:**
- `Token`は借用として取得（`&Token`）
- 所有権を得るには`clone()`が必要

## 実践例: qi-langのEvaluator

### 所有権とムーブ

```rust
Expr::Let { bindings, body } => {
    let mut new_env = Env::with_parent(env.clone());

    for (name, expr) in bindings {
        let value = self.eval_with_env(expr, Arc::new(RwLock::new(new_env.clone())))?;
        new_env.set(name.clone(), value);
    }

    self.eval_with_env(body, Arc::new(RwLock::new(new_env)))
}
```

**ポイント:**
- `env.clone()`: `Arc`のクローン（参照カウントの増加、安価）
- `name.clone()`: `String`のクローン（ディープコピー）
- `new_env`は関数の最後でドロップ

### 借用とメソッドチェーン

```rust
// src/lexer.rs
fn read_string(&mut self) -> Result<String, String> {
    self.advance();  // 可変借用
    let mut result = String::new();

    while let Some(ch) = self.current() {  // 不変借用
        if ch == '"' {
            self.advance();  // 可変借用
            return Ok(result);
        }
        result.push(ch);
        self.advance();
    }

    Err("Unclosed string".to_string())
}
```

**ポイント:**
- 可変借用と不変借用が交互に出現
- 各ステップで前の借用が終わっているのでOK

## データ競合の防止

Rustの所有権システムは**データ競合**を防ぎます。

### 悪い例（コンパイルエラー）

```rust
let mut v = vec![1, 2, 3];

let first = &v[0];  // 不変借用

v.push(4);  // エラー！可変借用（不変借用と同時使用不可）

println!("{}", first);
```

**理由:**
- `v.push(4)`でベクタが再割り当てされる可能性
- `first`が無効なメモリを指すかもしれない
- Rustはこれをコンパイル時に防ぐ！

### 正しい例

```rust
let mut v = vec![1, 2, 3];

{
    let first = &v[0];
    println!("{}", first);
}  // first の借用が終わる

v.push(4);  // OK
```

## まとめ

所有権システムの重要なポイント：

1. **所有権**: 各値には唯一の所有者がいる
2. **ムーブ**: 所有権の移動（元の変数は無効）
3. **借用**: 所有権を移動せずに参照
   - 不変借用（`&T`）: 複数同時OK
   - 可変借用（`&mut T`）: 1つだけ
4. **ライフタイム**: 参照の有効期間
5. **データ競合の防止**: コンパイル時にチェック

これらにより、Rustは**メモリ安全**と**スレッド安全**を実現します。

## 次のステップ

次は[エラーハンドリング](./03-result-option.md)を学びます。ResultとOptionをより深く理解しましょう。
