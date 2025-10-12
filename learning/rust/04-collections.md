# コレクション

Rustの標準的なコレクション型について学びます。

## Vec<T> - 可変長配列

最も基本的なコレクション型です。

### 作成

```rust
// 空のベクタ
let mut v: Vec<i32> = Vec::new();

// vec! マクロ
let v = vec![1, 2, 3, 4, 5];

// 初期容量を指定
let mut v = Vec::with_capacity(10);
```

### 要素の追加・削除

```rust
let mut v = vec![1, 2, 3];

// 末尾に追加
v.push(4);      // [1, 2, 3, 4]

// 末尾から削除
let last = v.pop();  // Some(4)

// 挿入
v.insert(1, 10);  // [1, 10, 2, 3]

// 削除
v.remove(1);      // [1, 2, 3]

// クリア
v.clear();        // []
```

### アクセス

```rust
let v = vec![1, 2, 3, 4, 5];

// インデックスでアクセス（パニックの可能性）
let third = v[2];  // 3

// get() - Option を返す（安全）
let third = v.get(2);  // Some(&3)
let tenth = v.get(10); // None

// 先頭と末尾
let first = v.first();  // Some(&1)
let last = v.last();    // Some(&5)
```

### qi-langでの使用例

```rust
// src/value.rs
pub enum Value {
    List(Vec<Value>),
    Vector(Vec<Value>),
    // ...
}

// src/lexer.rs
pub struct Lexer {
    input: Vec<char>,  // 文字の配列
    pos: usize,
}
```

## HashMap<K, V> - ハッシュマップ

キーと値のペアを格納します。

### 作成

```rust
use std::collections::HashMap;

// 空のマップ
let mut map = HashMap::new();

// 要素を追加
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);
```

### アクセス

```rust
let mut map = HashMap::new();
map.insert("key", "value");

// get() - Option を返す
let val = map.get("key");      // Some(&"value")
let val = map.get("missing");  // None

// [] でアクセス（パニックの可能性）
// let val = map["key"];  // "value"
```

### 更新

```rust
let mut map = HashMap::new();

// 挿入（上書き）
map.insert("key", 1);
map.insert("key", 2);  // 上書き

// entry API - 存在しない場合のみ挿入
map.entry("key").or_insert(3);  // 既に存在するので挿入されない
map.entry("new").or_insert(10); // 挿入される

// 値の更新
*map.entry("key").or_insert(0) += 1;
```

### qi-langでの使用例

```rust
// src/value.rs
pub struct Env {
    bindings: HashMap<String, Value>,  // 変数名 → 値
    parent: Option<Arc<RwLock<Env>>>,
}

impl Env {
    pub fn get(&self, name: &str) -> Option<Value> {
        self.bindings.get(name).cloned()
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }
}
```

## イテレータ

Rustのイテレータは**遅延評価**で強力です。

### 基本的な使い方

```rust
let v = vec![1, 2, 3, 4, 5];

// for ループ
for n in &v {
    println!("{}", n);
}

// iter() - 不変参照のイテレータ
for n in v.iter() {
    println!("{}", n);
}

// iter_mut() - 可変参照のイテレータ
let mut v = vec![1, 2, 3];
for n in v.iter_mut() {
    *n *= 2;  // 各要素を2倍
}

// into_iter() - 所有権を移動
for n in v.into_iter() {
    println!("{}", n);
}
// v はもう使えない
```

### map - 変換

```rust
let v = vec![1, 2, 3, 4, 5];

// 各要素を2倍
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
// => [2, 4, 6, 8, 10]
```

### filter - フィルタリング

```rust
let v = vec![1, 2, 3, 4, 5, 6];

// 偶数のみ
let evens: Vec<i32> = v.iter()
    .filter(|&x| x % 2 == 0)
    .cloned()
    .collect();
// => [2, 4, 6]
```

### fold / reduce - 畳み込み

```rust
let v = vec![1, 2, 3, 4, 5];

// 合計
let sum = v.iter().fold(0, |acc, x| acc + x);
// => 15

// reduce (Rust 1.51+)
let sum = v.iter().copied().reduce(|acc, x| acc + x);
// => Some(15)
```

### チェーン

```rust
let v = vec![1, 2, 3, 4, 5, 6];

let result: Vec<i32> = v.iter()
    .filter(|&x| x % 2 == 0)  // 偶数のみ
    .map(|x| x * x)            // 二乗
    .collect();
// => [4, 16, 36]
```

### qi-langでの使用例

```rust
// src/eval.rs
Expr::Vector(items) => {
    let values: Result<Vec<_>, _> = items
        .iter()
        .map(|e| self.eval_with_env(e, env.clone()))
        .collect();
    Ok(Value::Vector(values?))
}
```

### その他の便利なイテレータメソッド

```rust
let v = vec![1, 2, 3, 4, 5];

// take - 最初のn個
let first_three: Vec<i32> = v.iter().take(3).cloned().collect();
// => [1, 2, 3]

// skip - 最初のn個をスキップ
let rest: Vec<i32> = v.iter().skip(2).cloned().collect();
// => [3, 4, 5]

// enumerate - インデックス付き
for (i, val) in v.iter().enumerate() {
    println!("v[{}] = {}", i, val);
}

// zip - 2つを結合
let a = vec![1, 2, 3];
let b = vec!["a", "b", "c"];
let pairs: Vec<_> = a.iter().zip(b.iter()).collect();
// => [(1, "a"), (2, "b"), (3, "c")]

// find - 条件に合う最初の要素
let first_even = v.iter().find(|&&x| x % 2 == 0);
// => Some(&2)

// any - いずれかが条件を満たす
let has_even = v.iter().any(|&x| x % 2 == 0);
// => true

// all - すべてが条件を満たす
let all_positive = v.iter().all(|&x| x > 0);
// => true
```

## String と &str

文字列型について理解しましょう。

### String - 所有権を持つ文字列

```rust
// 作成
let s = String::from("hello");
let s = "hello".to_string();

// 追加
let mut s = String::from("hello");
s.push_str(", world");  // "hello, world"
s.push('!');            // "hello, world!"

// 連結
let s1 = String::from("hello");
let s2 = String::from("world");
let s3 = s1 + " " + &s2;  // s1 はムーブされる

// format! マクロ
let s = format!("{} {}", "hello", "world");
```

### &str - 文字列スライス

```rust
let s = String::from("hello world");

// スライス
let hello = &s[0..5];    // "hello"
let world = &s[6..11];   // "world"
let hello = &s[..5];     // "hello"
let world = &s[6..];     // "world"
```

### 変換

```rust
// String → &str
let s = String::from("hello");
let slice: &str = &s;

// &str → String
let slice = "hello";
let s = slice.to_string();
let s = String::from(slice);
```

### qi-langでの使用例

```rust
// src/lexer.rs
pub struct Lexer {
    input: Vec<char>,  // String を chars() で Vec<char> に変換
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }
}
```

## HashSet<T> - 集合

重複を許さないコレクションです。

### 基本的な使い方

```rust
use std::collections::HashSet;

let mut set = HashSet::new();

// 追加
set.insert(1);
set.insert(2);
set.insert(2);  // 重複は無視される

// 存在確認
let has_one = set.contains(&1);  // true

// 削除
set.remove(&1);

// サイズ
let size = set.len();
```

### 集合演算

```rust
let a: HashSet<_> = [1, 2, 3].iter().cloned().collect();
let b: HashSet<_> = [2, 3, 4].iter().cloned().collect();

// 和集合
let union: HashSet<_> = a.union(&b).cloned().collect();
// => {1, 2, 3, 4}

// 積集合
let intersection: HashSet<_> = a.intersection(&b).cloned().collect();
// => {2, 3}

// 差集合
let difference: HashSet<_> = a.difference(&b).cloned().collect();
// => {1}
```

## collect() - イテレータからコレクションへ

`collect()`は非常に汎用的です。

```rust
// Vec
let v: Vec<i32> = (0..5).collect();
// => [0, 1, 2, 3, 4]

// HashSet
use std::collections::HashSet;
let set: HashSet<i32> = (0..5).collect();

// HashMap
use std::collections::HashMap;
let pairs = vec![("a", 1), ("b", 2), ("c", 3)];
let map: HashMap<_, _> = pairs.into_iter().collect();

// String (文字のイテレータから)
let chars = vec!['h', 'e', 'l', 'l', 'o'];
let s: String = chars.into_iter().collect();
// => "hello"

// Result<Vec<_>, _>
let results = vec![Ok(1), Ok(2), Ok(3)];
let values: Result<Vec<i32>, String> = results.into_iter().collect();
// => Ok([1, 2, 3])

let results = vec![Ok(1), Err("error"), Ok(3)];
let values: Result<Vec<i32>, &str> = results.into_iter().collect();
// => Err("error")
```

## qi-langでの実践例

### トークン列の構築

```rust
// src/lexer.rs
pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    while self.current().is_some() {
        self.skip_whitespace();
        self.skip_comment();

        if self.current().is_none() {
            break;
        }

        tokens.push(self.next_token()?);
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}
```

### 環境のバインディング収集

```rust
// src/value.rs
impl Env {
    pub fn bindings(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.bindings.iter()
    }
}

// 使用例
let names: Vec<String> = env.bindings()
    .map(|(name, _)| name.clone())
    .collect();
```

### エクスポートシンボルの収集

```rust
// src/eval.rs
Expr::Export(symbols) => {
    let mut exports = HashMap::new();
    for symbol in symbols {
        if let Some(value) = env.read().get(symbol) {
            exports.insert(symbol.clone(), value);
        }
    }
    Ok(Value::Nil)
}
```

## パフォーマンスのヒント

### 容量の事前確保

```rust
// 悪い例 - 何度も再割り当て
let mut v = Vec::new();
for i in 0..1000 {
    v.push(i);  // 容量が足りなくなるたびに再割り当て
}

// 良い例 - 事前に容量を確保
let mut v = Vec::with_capacity(1000);
for i in 0..1000 {
    v.push(i);  // 再割り当てなし
}
```

### イテレータの活用

```rust
// 悪い例 - 中間ベクタを作成
let v = vec![1, 2, 3, 4, 5];
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
let evens: Vec<i32> = doubled.iter().filter(|&x| x % 2 == 0).cloned().collect();

// 良い例 - イテレータをチェーン
let evens: Vec<i32> = v.iter()
    .map(|x| x * 2)
    .filter(|&x| x % 2 == 0)
    .collect();
```

### clone の最小化

```rust
// 悪い例 - 不必要なクローン
let v = vec![1, 2, 3];
for i in v.clone() {  // 全体をクローン
    println!("{}", i);
}

// 良い例 - 借用
for i in &v {
    println!("{}", i);
}
```

## まとめ

Rustのコレクション型：

1. **Vec<T>**: 可変長配列
   - `push()`, `pop()`, `insert()`, `remove()`
   - インデックスアクセスと`get()`

2. **HashMap<K, V>**: キー・バリュー
   - `insert()`, `get()`, `entry()`
   - 変数の束縛管理に使用

3. **イテレータ**: 遅延評価
   - `map()`, `filter()`, `fold()`, `collect()`
   - チェーンで複雑な処理

4. **String / &str**: 文字列
   - 所有権 vs 借用
   - UTF-8エンコーディング

5. **HashSet<T>**: 集合
   - 重複なし
   - 集合演算

これらを使いこなすことで、効率的なデータ処理が可能になります。

## 次のステップ

次は[並行処理](./05-concurrency.md)を学びます。Arc、RwLock、スレッドについて理解しましょう。
