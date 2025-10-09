# Qi言語実装チュートリアル

**Qi（チー）** は、シンプルで高速なモダンLisp系言語です。このチュートリアルでは、Qiを実装しながら以下の3つを同時に学びます：

1. **Rust** - システムプログラミング言語の基礎
2. **プログラミング言語実装** - レキサー、パーサー、評価器の仕組み
3. **Qi言語** - Lisp系言語の文法と関数型プログラミング

## このチュートリアルの特徴

- 📚 **段階的な学習**: 簡単な機能から始めて、徐々に高度な機能を追加
- 💡 **実践的**: 実際に動くコードを書きながら学ぶ
- 🔍 **詳細な解説**: なぜそう実装するのか、どう動くのかを説明
- 🎯 **3つの視点**: Rust、言語実装、Qi言語の3方向から理解

## 対象読者

- Rustを学び始めた方
- プログラミング言語の作り方に興味がある方
- Lisp系言語を学びたい方

## 目次

1. [Phase 1: 基礎理解（完了済み）](#phase-1-基礎理解) - レキサー、パーサー、評価器
2. [Phase 2: match式の実装（完了済み）](#phase-2-match式の実装) - パターンマッチング
3. [Phase 3: パイプライン演算子（完了済み）](#phase-3-パイプライン演算子) - 糖衣構文
4. [Phase 3.5: Rustのマクロでコードを簡潔に（完了済み）](#phase-35-rustのマクロでコードを簡潔に--完了) - リファクタリング
5. [Phase 4: より多くの組み込み関数](#phase-4-より多くの組み込み関数)
6. [Phase 5: マクロシステム](#phase-5-マクロシステム)
7. [Phase 6: モジュールシステム](#phase-6-モジュールシステム)
8. [Phase 7: Cranelift統合](#phase-7-cranelift統合)

---

## Phase 1: 基礎理解（完了済み）

このフェーズでは、プログラミング言語の基本的な構造を実装しました：
- **レキサー**: 文字列をトークンに分解
- **パーサー**: トークンをASTに変換
- **評価器**: ASTを実行して結果を得る

### 実装した機能

- ✅ 基本データ型（整数、文字列、bool、nil、シンボル、キーワード）
- ✅ コレクション（リスト、ベクタ、マップ）
- ✅ 特殊形式（def、fn、let、if、do）
- ✅ 関数呼び出しとクロージャ
- ✅ 基本的な組み込み関数（+、-、*、/、=、<、>など）

### 学んだこと

#### Rustの概念

1. **所有権とライフタイム**
   ```rust
   // String は所有権を持つ
   let s = String::from("hello");
   // s がスコープを抜けると自動的にメモリが解放される
   ```

2. **Rc (Reference Counted)**
   ```rust
   use std::rc::Rc;

   // 複数の所有者を持つデータ
   let data = Rc::new(5);
   let data2 = data.clone();  // 参照カウントが増える
   ```

3. **RefCell (内部可変性)**
   ```rust
   use std::cell::RefCell;

   // 不変参照の中で可変的に変更できる
   let data = RefCell::new(5);
   *data.borrow_mut() = 10;
   ```

4. **enum と match**
   ```rust
   enum Value {
       Integer(i64),
       String(String),
   }

   match value {
       Value::Integer(n) => println!("数値: {}", n),
       Value::String(s) => println!("文字列: {}", s),
   }
   ```

#### プログラミング言語実装の基礎

1. **レキサー（字句解析）**: テキスト → トークン列
   ```
   "(+ 1 2)" → [LParen, Symbol("+"), Integer(1), Integer(2), RParen]
   ```

2. **パーサー（構文解析）**: トークン列 → AST
   ```
   [LParen, Symbol("+"), ...] → Call { func: Symbol("+"), args: [1, 2] }
   ```

3. **評価器**: AST → 実行結果
   ```
   Call { func: "+", args: [1, 2] } → Value::Integer(3)
   ```

4. **環境（Environment）**: 変数の束縛を管理
   ```rust
   env.set("x".to_string(), Value::Integer(42));
   env.get("x") // Some(Value::Integer(42))
   ```

#### Qi言語の特徴

- **Lisp-1**: 変数と関数が同じ名前空間
- **特殊形式**: `def`, `fn`, `let`, `if`, `do`, `match`
- **演算子**: `|>` (パイプライン)
- **クロージャ**: 関数が環境をキャプチャ
- **nil/bool**: 明確に区別（条件式では両方falsy）

#### Qi言語の基本的な使い方

```lisp
; 変数の定義
(def x 42)
(def name "Alice")

; 関数の定義
(def add (fn [a b] (+ a b)))
(def greet (fn [name] (str "Hello, " name "!")))

; 関数呼び出し
(add 10 20)           ; 30
(greet "Bob")         ; "Hello, Bob!"

; let で局所変数
(let [x 10 y 20]
  (+ x y))            ; 30

; if で条件分岐
(if (> x 10)
  "big"
  "small")

; match でパターンマッチング
(match x
  0 -> "zero"
  n -> (str "value: " n))

; パイプラインでデータフロー
(10 |> (+ 5) |> (* 2))  ; 30
```

---

## Phase 2: match式の実装 ✅ 完了

### Qi言語でのmatch式

Qi言語では、`match`を使って値に応じた処理を分岐できます：

```lisp
; 数値の分類
(match x
  0 -> "zero"
  1 -> "one"
  n -> (str "other: " n))

; nil/boolの区別（重要！）
(match result
  nil -> "見つからない"
  false -> "明示的にfalse"
  true -> "成功"
  v -> (str "値: " v))

; マップから値を取り出す
(match user
  {:name n :age a} -> (str n "さんは" a "歳")
  _ -> "不明")

; ガード条件で細かく制御
(match x
  n when (> n 0) -> "正の数"
  n when (< n 0) -> "負の数"
  _ -> "ゼロ")
```

### 実装済みの機能

- ✅ 値のマッチング（整数、文字列、bool、nil、キーワード）
- ✅ 変数バインディング
- ✅ ワイルドカード（`_`）
- ✅ ベクタパターンマッチ `[x y z]`
- ✅ マップパターンマッチ `{:name n :age a}`
- ✅ ガード条件 `n when (> n 0) -> "positive"`

### 学習内容

#### Rustで学ぶこと

1. **複雑なenumのパターンマッチ**
   - 複数のバリアントを持つenumの設計
   - 再帰的なデータ構造（`Pattern`の中に`Pattern`）

2. **Box の使い方**（再帰的なデータ構造）
   - なぜBoxが必要か（サイズが確定しない型）
   - Boxの使い分け

3. **Vec の操作**
   - 可変ベクタの構築
   - イテレータとの組み合わせ

4. **HashMap の使用**（パターンマッチ時のバインディング管理）
   - キー・バリューの挿入と検索
   - 一時的なバインディング収集

#### 言語実装で学ぶこと

1. **パターンマッチングの理論**
   - 線形マッチング（上から順に試す）
   - パターンの優先順位

2. **バインディングの扱い**
   - 変数のキャプチャ
   - スコープの管理

3. **ガード条件の実装**
   - パターンマッチ後の追加チェック
   - バインディングされた変数の利用

### ステップ1: ASTにMatchを追加（実装済み）

`src/value.rs` の `Expr` に新しいバリアントを追加しました:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ... 既存のバリアント ...

    // match式
    Match {
        expr: Box<Expr>,           // マッチ対象の式
        arms: Vec<MatchArm>,       // マッチの腕
    },
}

/// matchのアーム（パターン -> 結果）
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,  // when句（オプション）
    pub body: Box<Expr>,
}

/// パターン
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard,                      // _
    Nil,                           // nil
    Bool(bool),                    // true, false
    Integer(i64),                  // 整数リテラル
    Float(f64),                    // 浮動小数点リテラル
    String(String),                // 文字列リテラル
    Keyword(String),               // キーワードリテラル
    Var(String),                   // 変数（バインディング）
    List(Vec<Pattern>, Option<Box<Pattern>>), // リストパターン（固定部、可変部）
    Vector(Vec<Pattern>),          // ベクタパターン
    Map(Vec<(String, Pattern)>),   // マップパターン
}
```

**Rustポイント**: `Box<T>` は再帰的なデータ構造を作るために必須です。

```rust
// これはコンパイルエラー（サイズが無限大）
// struct Node {
//     next: Node  // NG!
// }

// これはOK（ポインタなのでサイズが確定）
struct Node {
    next: Box<Node>  // OK!
}
```

**実装のポイント**:
- nil/bool/整数/文字列などのリテラル値を直接パターンで表現
- `Var(String)` で変数バインディングを実現
- マップパターンではキーをStringで保持し、値をPatternで再帰的に表現

### ステップ2: パーサーにmatchを追加（実装済み）

まず、レキサーに `->` トークンと `when` キーワードを追加:

```rust
// src/lexer.rs
pub enum Token {
    // ... 既存のトークン ...
    Arrow,  // ->
    When,   // when
}

// -> のパース
Some('-') if self.peek(1) == Some('>') => {
    self.advance();
    self.advance();
    return Ok(Token::Arrow);
}

// when のパース（キーワードとして）
match result.as_str() {
    "when" => Token::When,
    // ...
}
```

次に、`src/parser.rs` に `parse_match` メソッドを追加:

```rust
fn parse_match(&mut self) -> Result<Expr, String> {
    self.advance(); // 'match'をスキップ

    // マッチ対象の式
    let expr = Box::new(self.parse_expr()?);

    // マッチの腕を集める
    let mut arms = Vec::new();

    while self.current() != Some(&Token::RParen) {
        // パターンをパース
        let pattern = self.parse_pattern()?;

        // ガード条件のチェック
        let guard = if self.current() == Some(&Token::When) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        // '->'をパース
        self.expect(Token::Arrow)?;

        // 本体をパース
        let body = Box::new(self.parse_expr()?);

        arms.push(MatchArm {
            pattern,
            guard,
            body,
        });
    }

    self.expect(Token::RParen)?;

    Ok(Expr::Match { expr, arms })
}

fn parse_pattern(&mut self) -> Result<Pattern, String> {
    match self.current() {
        Some(Token::Symbol(s)) if s == "_" => {
            self.advance();
            Ok(Pattern::Wildcard)
        }
        Some(Token::Nil) => {
            self.advance();
            Ok(Pattern::Nil)
        }
        Some(Token::True) => {
            self.advance();
            Ok(Pattern::Bool(true))
        }
        Some(Token::False) => {
            self.advance();
            Ok(Pattern::Bool(false))
        }
        Some(Token::Integer(n)) => {
            let n = *n;
            self.advance();
            Ok(Pattern::Integer(n))
        }
        Some(Token::Symbol(s)) => {
            let s = s.clone();
            self.advance();
            Ok(Pattern::Var(s))  // 変数バインディング
        }
        Some(Token::LBracket) => self.parse_vector_pattern(),
        Some(Token::LBrace) => self.parse_map_pattern(),
        _ => Err("無効なパターンです".to_string()),
    }
}

fn parse_vector_pattern(&mut self) -> Result<Pattern, String> {
    self.expect(Token::LBracket)?;
    let mut patterns = Vec::new();
    while self.current() != Some(&Token::RBracket) {
        patterns.push(self.parse_pattern()?);
    }
    self.expect(Token::RBracket)?;
    Ok(Pattern::Vector(patterns))
}

fn parse_map_pattern(&mut self) -> Result<Pattern, String> {
    self.expect(Token::LBrace)?;
    let mut pairs = Vec::new();
    while self.current() != Some(&Token::RBrace) {
        let key = match self.current() {
            Some(Token::Keyword(k)) => k.clone(),
            _ => return Err("マップパターンのキーはキーワードが必要です".to_string()),
        };
        self.advance();
        let pattern = self.parse_pattern()?;
        pairs.push((key, pattern));
    }
    self.expect(Token::RBrace)?;
    Ok(Pattern::Map(pairs))
}
```

**Rustポイント**: `Vec::new()` と `Vec::push()` でベクタを構築します。

```rust
let mut items = Vec::new();
items.push(1);
items.push(2);
// items = [1, 2]
```

**実装のポイント**:
- `->` を専用のトークンとして扱うため、2文字の先読みが必要
- `when` はキーワードとして認識
- ベクタパターンとマップパターンは再帰的にパース

### ステップ3: 評価器にmatchを追加（実装済み）

`src/eval.rs` に評価ロジックを追加しました:

```rust
fn eval_with_env(&mut self, expr: &Expr, env: Rc<RefCell<Env>>) -> Result<Value, String> {
    match expr {
        // ... 既存の処理 ...

        Expr::Match { expr, arms } => {
            let value = self.eval_with_env(expr, env.clone())?;
            self.eval_match(&value, arms, env)
        }
    }
}

fn eval_match(
    &mut self,
    value: &Value,
    arms: &[MatchArm],
    env: Rc<RefCell<Env>>,
) -> Result<Value, String> {
    for arm in arms {
        let mut bindings = HashMap::new();
        if self.match_pattern(&arm.pattern, value, &mut bindings)? {
            // ガード条件のチェック
            if let Some(guard) = &arm.guard {
                let mut guard_env = Env::with_parent(env.clone());
                for (name, val) in &bindings {
                    guard_env.set(name.clone(), val.clone());
                }
                let guard_val = self.eval_with_env(guard, Rc::new(RefCell::new(guard_env)))?;
                if !is_truthy(&guard_val) {
                    continue;
                }
            }

            // マッチ成功：バインディングを環境に追加して本体を評価
            let mut match_env = Env::with_parent(env.clone());
            for (name, val) in bindings {
                match_env.set(name, val);
            }
            return self.eval_with_env(&arm.body, Rc::new(RefCell::new(match_env)));
        }
    }
    Err("どのパターンにもマッチしませんでした".to_string())
}

fn match_pattern(
    &self,
    pattern: &Pattern,
    value: &Value,
    bindings: &mut HashMap<String, Value>,
) -> Result<bool, String> {
    match pattern {
        Pattern::Wildcard => Ok(true),
        Pattern::Nil => Ok(matches!(value, Value::Nil)),
        Pattern::Bool(b) => Ok(matches!(value, Value::Bool(vb) if vb == b)),
        Pattern::Integer(n) => Ok(matches!(value, Value::Integer(vn) if vn == n)),
        Pattern::String(s) => Ok(matches!(value, Value::String(vs) if vs == s)),
        Pattern::Keyword(k) => Ok(matches!(value, Value::Keyword(vk) if vk == k)),
        Pattern::Var(name) => {
            bindings.insert(name.clone(), value.clone());
            Ok(true)
        }
        Pattern::Vector(patterns) => {
            if let Value::Vector(values) = value {
                if patterns.len() != values.len() {
                    return Ok(false);
                }
                for (pat, val) in patterns.iter().zip(values.iter()) {
                    if !self.match_pattern(pat, val, bindings)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Pattern::Map(pattern_pairs) => {
            if let Value::Map(map) = value {
                for (key, pat) in pattern_pairs {
                    if let Some(val) = map.get(key) {
                        if !self.match_pattern(pat, val, bindings)? {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
                Ok(true)
            } else {
                Ok(false)
            }
        }
        // ... その他のパターン ...
    }
}
```

**Rustポイント**: `HashMap` を使って変数のバインディングを管理します。

```rust
use std::collections::HashMap;

let mut bindings = HashMap::new();
bindings.insert("x".to_string(), Value::Integer(42));

if let Some(value) = bindings.get("x") {
    println!("x = {:?}", value);
}
```

**実装のポイント**:
- `HashMap<String, Value>` でパターンマッチ時のバインディングを収集
- ガード条件は独自の環境で評価（バインディングを含む）
- パターンマッチは再帰的に行い、失敗したら即座に `Ok(false)` を返す
- 全てのパターンがマッチしたら、バインディングを環境に追加して本体を評価

### ステップ4: テストを書く（実装済み）

`src/eval.rs` の `#[cfg(test)]` に追加しました:

```rust
#[test]
fn test_match_literal() {
    assert_eq!(
        eval_str("(match 0 0 -> 42 1 -> 99)").unwrap(),
        Value::Integer(42)
    );
}

#[test]
fn test_match_var() {
    assert_eq!(
        eval_str("(match 10 n -> (+ n 5))").unwrap(),
        Value::Integer(15)
    );
}

#[test]
fn test_match_wildcard() {
    assert_eq!(
        eval_str("(match 42 0 -> 1 1 -> 2 _ -> 99)").unwrap(),
        Value::Integer(99)
    );
}

#[test]
fn test_match_nil_bool() {
    // nil/boolの区別
    assert_eq!(
        eval_str("(match nil nil -> 1 false -> 2 _ -> 3)").unwrap(),
        Value::Integer(1)
    );
}

#[test]
fn test_match_vector() {
    assert_eq!(
        eval_str("(match [1 2] [x y] -> (+ x y))").unwrap(),
        Value::Integer(3)
    );
}

#[test]
fn test_match_guard() {
    assert_eq!(
        eval_str("(match 5 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
        Value::Integer(1)
    );
}
```

### 動作確認

`examples/match_test.qi` を実行:

```bash
$ cargo run examples/match_test.qi
"zero"
"nil"
"false"
52
6
"positive"
"negative"
"zero"
```

全てのテストが通り、match式が正常に動作しています！

### 学んだこと

1. **Rustの概念**:
   - `HashMap` を使った動的なデータ構造管理
   - `matches!` マクロでパターンマッチの簡潔な記述
   - 可変参照 `&mut` を使ったバインディング収集

2. **言語実装のテクニック**:
   - パターンマッチングアルゴリズム（線形マッチング）
   - ガード条件の評価タイミング
   - バインディング環境の階層的な管理

3. **Qi言語の特徴**:
   - nil/bool の明確な区別（SPEC.mdに従った実装）
   - 変数バインディングによる柔軟なパターン
   - ガード条件による条件付きマッチ

---

## Phase 3: パイプライン演算子 ✅ 完了

### Qi言語でのパイプライン

Qi言語では、`|>` を使ってデータの流れを左から右に記述できます：

```lisp
; ネストした関数呼び出しは読みにくい
(double (inc 10))  ; 22

; パイプラインなら流れが分かりやすい！
(10 |> inc |> double)  ; 22

; 複数の処理を連鎖
(def data [1 2 3 4 5])
(data
  |> (map square)      ; 各要素を二乗
  |> (filter even?)    ; 偶数だけ残す
  |> (reduce +))       ; 合計を計算

; 引数付き関数にも使える
(10 |> (+ 5))         ; 15 (+ 5 10) と同じ
(1 |> (+ 2) |> (* 3)) ; 9  (* 3 (+ 2 1)) と同じ
```

**なぜパイプラインが便利？**
- データの変換の流れが一目で分かる
- 関数型プログラミングが書きやすい
- ネストが深くならない

### 実装済みの機能

- ✅ `|>` トークンの追加
- ✅ パイプライン式のパース
- ✅ 関数呼び出しへの変換（糖衣構文）
- ✅ 引数付き関数へのパイプライン対応

### 学習内容

#### Rustで学ぶこと

1. **演算子のパース**
   - 2文字トークンの認識
   - 先読み処理

2. **AST変換**
   - パース時のAST書き換え
   - 所有権の移動

3. **可変データ構造の操作**
   - `mut`を使ったベクタの更新
   - `match`での分解と再構築

#### 言語実装で学ぶこと

1. **糖衣構文（syntax sugar）**
   - コンパイル時の構文変換
   - ユーザーフレンドリーな構文の提供

2. **AST変換の技法**
   - パーサーレベルでの最適化
   - 評価器の変更なしで機能追加

3. **中置演算子の実装**
   - Lisp系言語での中置演算子
   - 左結合の実現

### ステップ1: トークンに|>を追加（実装済み）

`src/lexer.rs` にPipeトークンを追加しました:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ... 既存のトークン ...
    Pipe,  // |>
}

// next_token() の中に追加
Some('|') if self.peek(1) == Some('>') => {
    self.advance(); // |
    self.advance(); // >
    return Ok(Token::Pipe);
}
```

**実装のポイント**:
- `->` トークンと同様に2文字の先読みが必要
- `-` より前にチェックしないと、`-` が数値のマイナス記号と誤認される

### ステップ2: パーサーで|>を処理（実装済み）

パイプラインはリスト内で処理されます。`parse_list` を修正：

```rust
fn parse_list(&mut self) -> Result<Expr, String> {
    self.expect(Token::LParen)?;

    // 空リストや特殊形式のチェック...

    // 通常のリスト
    let first_expr = self.parse_primary()?;  // パイプラインを含まない

    // パイプラインのチェック
    if self.current() == Some(&Token::Pipe) {
        let mut expr = first_expr;
        while self.current() == Some(&Token::Pipe) {
            self.advance();
            let right = self.parse_primary()?;

            // x |> f を (f x) に変換
            // x |> (f a b) を (f a b x) に変換
            expr = match right {
                Expr::Call { func, mut args } => {
                    args.push(expr);
                    Expr::Call { func, args }
                }
                _ => Expr::Call {
                    func: Box::new(right),
                    args: vec![expr],
                },
            };
        }
        self.expect(Token::RParen)?;
        return Ok(expr);
    }

    // 通常の関数呼び出し...
}
```

**言語実装のポイント**:
- パイプラインは「糖衣構文」
- パーサーで通常の関数呼び出しに変換すれば、評価器の変更は不要

**変換例**:
```
(10 |> inc)              →  (inc 10)
(1 |> inc |> inc)        →  (inc (inc 1))
(10 |> (+ 5))            →  (+ 5 10)  = 15
(1 |> (+ 2) |> (* 3))    →  (* 3 (+ 2 1))  = 9
```

### 動作確認

`examples/pipe_test.qi` を実行:

```bash
$ cargo run examples/pipe_test.qi
11
3
15
9
100
```

全てのテストが通り、パイプラインが正常に動作しています！

### 学んだこと

1. **Rustの概念**:
   - `mut` を使った可変ベクタの操作
   - `match` による列挙型の分岐とパターンマッチング
   - 所有権の移動と `mut` パラメータ

2. **言語実装のテクニック**:
   - 糖衣構文（syntax sugar）の実装
   - AST変換による機能追加
   - 中置演算子の左結合パース

3. **Qi言語の特徴**:
   - パイプラインによる読みやすいデータフロー
   - 関数型プログラミングの促進
   - Lisp構文での中置演算子の実現

---

## Phase 3.5: Rustのマクロでコードを簡潔に ✅ 完了

### 目標

Rustの`macro_rules!`を使って、重複したコードを削減し、保守性を向上させる。

### Rustで学ぶこと

1. **宣言的マクロ（`macro_rules!`）**
   - パターンマッチングベースのコード生成
   - コンパイル時のコード展開
   - 反復処理の自動化

2. **マクロの使い分け**
   - 関数では実現できないケース
   - ボイラープレートの削減

3. **メタプログラミング**
   - コードを書くコード
   - DRY原則の徹底

### 実装した内容

#### 問題: 重複した関数登録コード

eval.rsでは、各組み込み関数を環境に登録する際、以下のような重複したコードが280行以上ありました：

```rust
// 各関数につき6行が必要
env.set(
    "+".to_string(),
    Value::NativeFunc(NativeFunc {
        name: "+".to_string(),
        func: native_add,
    }),
);

env.set(
    "-".to_string(),
    Value::NativeFunc(NativeFunc {
        name: "-".to_string(),
        func: native_sub,
    }),
);

// これが40個以上続く...
```

**なぜこれが問題か？**
- 新しい関数を追加するたびに6行書く必要がある
- タイプミスの可能性
- 関数名が2箇所に登場（DRY原則違反）
- コードが長く読みづらい

#### 解決策1: `register_native!` マクロ

関数登録を1行で書けるマクロを作成：

```rust
/// 組み込み関数を登録するマクロ
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
```

**マクロの仕組み**:
- `$env:expr` - 環境変数を受け取る
- `$($name:expr => $func:expr),*` - `名前 => 関数` のペアを0個以上受け取る
- `$(,)?` - 末尾のカンマを許可（オプション）
- `$(...)*` - パターンを繰り返し展開

**使用例**:
```rust
register_native!(env,
    // 算術演算
    "+" => native_add,
    "-" => native_sub,
    "*" => native_mul,
    "/" => native_div,
    "%" => native_mod,

    // 比較演算
    "=" => native_eq,
    "<" => native_lt,
    ">" => native_gt,

    // ... 他の関数も1行ずつ
);
```

**効果**:
- 280行 → 40行に削減（85%減！）
- 新しい関数は1行追加するだけ
- カテゴリごとにコメントで整理
- タイプミスのリスク減少

#### 解決策2: `check_args!` マクロ

各組み込み関数の引数チェックも重複していました：

```rust
// 各関数で同じパターンが繰り返される
fn native_nth(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["nth", "2"]));
    }
    // 実際の処理...
}

fn native_count(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["count", "1"]));
    }
    // 実際の処理...
}
```

**引数チェック用マクロ**:
```rust
/// 引数の数をチェックするマクロ
macro_rules! check_args {
    // 正確にN個の引数が必要
    ($args:expr, $expected:expr, $func_name:expr) => {
        if $args.len() != $expected {
            return Err(fmt_msg(
                MsgKey::NeedExactlyNArgs,
                &[$func_name, &$expected.to_string()],
            ));
        }
    };

    // 最小〜最大個の引数が必要
    ($args:expr, $min:expr, $max:expr, $func_name:expr) => {
        if $args.len() < $min || $args.len() > $max {
            return Err(format!(
                "{}には{}〜{}個の引数が必要です",
                $func_name, $min, $max
            ));
        }
    };
}
```

**使用例**:
```rust
fn native_nth(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "nth");  // たった1行！
    // 実際の処理...
}

fn native_count(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "count");  // たった1行！
    // 実際の処理...
}

fn native_abs(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "abs");
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err("absは数値のみ受け付けます".to_string()),
    }
}
```

**効果**:
- 各関数で3行 → 1行に削減
- エラーメッセージが統一される
- 可変長引数にも対応（2つ目のパターン）

### Rustのマクロ基礎

#### 宣言的マクロ vs 手続き的マクロ

**宣言的マクロ (`macro_rules!`)**:
- パターンマッチングでコードを生成
- 実装が簡単
- 今回使用したもの

**手続き的マクロ**:
- Rustコードでマクロを実装
- より複雑な処理が可能
- derive, attribute, function-like の3種類

#### パターンマッチングの構文

```rust
macro_rules! my_macro {
    // パターン1: 引数なし
    () => {
        println!("引数なし");
    };

    // パターン2: 式1つ
    ($x:expr) => {
        println!("値: {}", $x);
    };

    // パターン3: 繰り返し
    ($($x:expr),*) => {
        $(
            println!("値: {}", $x);
        )*
    };
}

// 使用例
my_macro!();           // 引数なし
my_macro!(42);         // 値: 42
my_macro!(1, 2, 3);    // 値: 1 \n 値: 2 \n 値: 3
```

**パターンの種類**:
- `expr` - 式
- `ident` - 識別子
- `ty` - 型
- `pat` - パターン
- `stmt` - 文
- `block` - ブロック

**繰り返しの記号**:
- `*` - 0回以上
- `+` - 1回以上
- `?` - 0回または1回

#### 実践例: vec! マクロ

Rustの標準ライブラリの`vec!`マクロを理解しましょう：

```rust
// 簡易版の実装
macro_rules! vec {
    // 空のベクタ
    () => {
        Vec::new()
    };

    // 要素のリスト
    ($($x:expr),* $(,)?) => {
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
let v1 = vec![];
let v2 = vec![1, 2, 3];
let v3 = vec![1, 2, 3,];  // 末尾カンマもOK
```

### マクロの利点と注意点

**利点**:
1. **ボイラープレート削減** - 繰り返しコードを大幅に減らせる
2. **型安全** - コンパイル時にチェックされる
3. **ゼロコスト抽象化** - ランタイムオーバーヘッドなし
4. **柔軟性** - 関数では不可能な構文を実現

**注意点**:
1. **エラーメッセージが分かりにくい** - マクロ展開時のエラーは読みづらい
2. **デバッグが難しい** - `cargo expand`で展開結果を確認
3. **使いすぎない** - できるだけ関数を使う
4. **ドキュメント必須** - マクロの動作を明確に説明

### マクロのデバッグ

**cargo expandで展開結果を見る**:
```bash
# cargo-expandをインストール
cargo install cargo-expand

# マクロの展開結果を表示
cargo expand
```

**展開結果の例**:
```rust
// 元のコード
register_native!(env,
    "+" => native_add,
    "-" => native_sub,
);

// 展開後
env.set(
    "+".to_string(),
    Value::NativeFunc(NativeFunc {
        name: "+".to_string(),
        func: native_add,
    }),
);
env.set(
    "-".to_string(),
    Value::NativeFunc(NativeFunc {
        name: "-".to_string(),
        func: native_sub,
    }),
);
```

### 学んだこと

1. **Rustのマクロ**:
   - `macro_rules!` による宣言的マクロ
   - パターンマッチングでのコード生成
   - 繰り返しパターン `$(...)*`
   - オプショナルパターン `$(...)?`

2. **リファクタリング技法**:
   - 重複コードの特定
   - マクロによる共通化
   - コードの可読性向上

3. **実装の学習**:
   - ボイラープレートの削減方法
   - メタプログラミングの活用
   - 保守性の高いコード設計

### 練習問題

1. **println! マクロの実装**: 独自の出力マクロを作ってみよう
2. **HashMap初期化マクロ**: `hashmap!{ "key" => value }` のようなマクロを実装
3. **テスト生成マクロ**: 似たようなテストを一括生成するマクロ

---

## Phase 4: より多くの組み込み関数

### 目標

実用的なプログラムを書けるように、組み込み関数を充実させる。

### Rustのクレート（crate）とは

Qiの実装では、いくつかの外部機能を**クレート**を使って実装しています。

**クレートの基本**:
- Rustのパッケージ管理システムの単位
- `Cargo.toml`で依存関係を指定
- [crates.io](https://crates.io/)から取得できる

**Pure Rust vs 外部ライブラリ**:

Qiでは、以下の方針でクレートを選択しています：

✅ **採用**: Pure Rustクレート（C/C++依存なし）
- ビルドが簡単
- クロスプラットフォーム
- 静的リンク

❌ **避ける**: 動的リンクライブラリ（libssl等）が必要なもの
- ビルド環境に依存
- 配布が複雑

**Qiで使用しているクレート**:

```toml
[dependencies]
base64 = "0.21"          # Base64エンコード/デコード
urlencoding = "2.1"      # URLエンコード/デコード
html-escape = "0.2"      # HTMLエスケープ処理
sha2 = "0.10"            # SHA-256ハッシュ生成
uuid = "1.6"             # UUID生成
```

これらはすべてPure Rustで実装されており、外部ライブラリへの依存がありません。

**実装例**:

```rust
// src/builtins/string.rs
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use uuid::Uuid;

pub fn native_to_base64(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::String(s) => {
            let encoded = general_purpose::STANDARD.encode(s);
            Ok(Value::String(encoded))
        }
        _ => Err("to-base64: 文字列が必要です".to_string()),
    }
}

pub fn native_hash(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::String(s) => {
            let mut hasher = Sha256::new();
            hasher.update(s.as_bytes());
            let hash = format!("{:x}", hasher.finalize());
            Ok(Value::String(hash))
        }
        _ => Err("hash: 文字列が必要です".to_string()),
    }
}

pub fn native_uuid(args: &[Value]) -> Result<Value, String> {
    let uuid = Uuid::new_v4();
    Ok(Value::String(uuid.to_string()))
}
```

**Qi言語での使用例**:

```lisp
;; Base64エンコード/デコード
(to-base64 "hello")        ;; => "aGVsbG8="
(from-base64 "aGVsbG8=")   ;; => "hello"

;; URLエンコード/デコード
(url-encode "hello world")  ;; => "hello%20world"
(url-decode "hello%20world") ;; => "hello world"

;; HTMLエスケープ
(html-escape "<div>test</div>")  ;; => "&lt;div&gt;test&lt;/div&gt;"

;; ハッシュ生成
(hash "hello")  ;; => "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"

;; UUID生成
(uuid)  ;; => "550e8400-e29b-41d4-a716-446655440000" (毎回異なる)
```

### 実装する関数

#### リスト操作

```rust
// map: リストの各要素に関数を適用
fn native_map(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("map には2つの引数が必要です".to_string());
    }

    let func = &args[0];
    let list = &args[1];

    match list {
        Value::List(items) => {
            let mut result = Vec::new();
            for item in items {
                // TODO: 関数を呼び出す方法が必要
                // これは評価器を渡す必要がある
            }
            Ok(Value::List(result))
        }
        _ => Err("map の第2引数はリストが必要です".to_string()),
    }
}
```

**問題**: ネイティブ関数から `Evaluator` にアクセスできない！

**解決策**: 組み込み関数を特別扱いせず、評価器の中で直接実装する。

```rust
// eval.rs の中で
Expr::Call { func, args } => {
    let func_val = self.eval_with_env(func, env.clone())?;

    // 特別な組み込み関数を先にチェック
    if let Value::Symbol(name) = &func_val {
        match name.as_str() {
            "map" => return self.builtin_map(args, env),
            "filter" => return self.builtin_filter(args, env),
            "reduce" => return self.builtin_reduce(args, env),
            _ => {}
        }
    }

    // 通常の関数呼び出し
    // ...
}

fn builtin_map(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("map には2つの引数が必要です".to_string());
    }

    let func = self.eval_with_env(&args[0], env.clone())?;
    let list = self.eval_with_env(&args[1], env.clone())?;

    match list {
        Value::List(items) => {
            let mut result = Vec::new();
            for item in items {
                // 関数を各要素に適用
                let val = self.apply_function(&func, &[item])?;
                result.push(val);
            }
            Ok(Value::List(result))
        }
        _ => Err("map の第2引数はリストが必要です".to_string()),
    }
}

fn apply_function(&mut self, func: &Value, args: &[Value]) -> Result<Value, String> {
    match func {
        Value::Function(f) => {
            let parent_env = Rc::new(RefCell::new(f.env.clone()));
            let mut new_env = Env::with_parent(parent_env);

            for (param, arg) in f.params.iter().zip(args.iter()) {
                new_env.set(param.clone(), arg.clone());
            }

            self.eval_with_env(&f.body, Rc::new(RefCell::new(new_env)))
        }
        Value::NativeFunc(nf) => (nf.func)(args),
        _ => Err("関数ではありません".to_string()),
    }
}
```

### 実装する関数リスト

```rust
// リスト操作
map, filter, reduce
take, drop, take-while, drop-while
concat, flatten
zip, zip-with

// 文字列操作（strモジュール用）
str-len, str-concat
str-split, str-join
str-upper, str-lower

// 数値操作
min, max, abs
floor, ceil, round

// 述語関数
even?, odd?
nil?, list?, number?, string?
```

### 練習問題

1. **filter の実装**: リストから条件を満たす要素だけを抽出
2. **reduce の実装**: リストを1つの値に畳み込む
3. **高階関数の組み合わせ**: `(map square (filter even? [1 2 3 4 5]))`

---

#### 文字列処理関数（実装済み）

Qiでは実用的な文字列処理を重視しており、以下のような関数群が実装されています。

##### 1. 検索系 - フィルタやバリデーションで頻用
```lisp
(contains? "hello world" "world")  ;; true - 部分一致判定
(starts-with? "hello" "he")        ;; true - URLやファイル拡張子チェック
(ends-with? "hello" "lo")          ;; true - 拡張子やプロトコル判定
(index-of "hello world" "world")   ;; 6 - 部分文字列の開始位置
(last-index-of "hello hello" "hello")  ;; 6 - ログ解析やタグ抽出
```

##### 2. 部分取得系 - サブ文字列の抽出
```lisp
(slice "hello world" 0 5)          ;; "hello" - インデックス範囲で抽出
(take-str "hello" 3)               ;; "hel" - ログプレビューや短縮表示
(drop-str "hello" 2)               ;; "llo" - プレフィックス除去
(sub-before "user@example.com" "@")  ;; "user" - キーやパスの抽出
(sub-after "user@example.com" "@")   ;; "example.com" - 拡張子やクエリ抽出
```

##### 3. 分割・結合 - CSVやテキスト処理
```lisp
(split "a,b,c" ",")    ;; ["a" "b" "c"] - 区切り文字で分割
(lines "hello\nworld") ;; ["hello" "world"] - テキスト処理やスクレイピング
(words "hello world")  ;; ["hello" "world"] - NLPやキーワード抽出
(join ", " ["a" "b"])  ;; "a, b" - パイプライン終端でフォーマット整形
```

##### 4. 置換 - フォーマット変換やクレンジング
```lisp
(replace "hello hello" "hello" "hi")      ;; "hi hi" - 全て置換
(replace-first "hello hello" "hello" "hi") ;; "hi hello" - 最初のみ置換
```

##### 5. 変換・正規化 - 入力値クリーニング
```lisp
(upper "hello")        ;; "HELLO" - UIやデータフォーマット統一
(lower "HELLO")        ;; "hello" - 比較の前処理、正規化
(capitalize "hello")   ;; "Hello" - 人名やタイトル整形
(trim "  hello  ")     ;; "hello" - 入力値クリーニング
(trim-left "  hello")  ;; "hello" - インデント調整
(trim-right "hello  ") ;; "hello" - フォーマット調整
(squish "  hello   world  \n")  ;; "hello world" - 連続空白を1つに（超重要）
(expand-tabs "\thello")         ;; "    hello" - タブをスペースに変換
```

##### 6. 繰り返し・フォーマット - CLI/UI整形
```lisp
(repeat "-" 80)              ;; 80個の"-" - 区切り線生成
(pad-left "Total" 20)        ;; "               Total" - 整列やコード生成
(pad-right "Name" 20)        ;; "Name               " - 表やログ整形
(pad "hi" 10)                ;; "    hi    " - 中央揃え
(pad "hi" 10 "*")            ;; "****hi****" - カスタム文字で詰める
```

##### 7. 判定（バリデーション） - 入力検証
```lisp
(digit? "12345")   ;; true - 数字のみ
(alpha? "hello")   ;; true - アルファベットのみ
(alnum? "hello123") ;; true - 英数字のみ
(space? "  \n\t")  ;; true - 空白文字のみ
(lower? "abc")     ;; true - 小文字のみ
(upper? "ABC")     ;; true - 大文字のみ
```

##### 8. Unicode対応 - 国際化対応
```lisp
(chars-count "hello")      ;; 5 - Unicode文字数（見た目の文字数）
(bytes-count "hello")      ;; 5 - バイト数（保存時や通信時の容量制御）
(chars-count "こんにちは")   ;; 5
(bytes-count "こんにちは")   ;; 15
```

**実装のポイント**:
- Rustの`str`型のメソッド（`contains`, `starts_with`, `split_whitespace`等）を活用
- Unicode文字数は`chars().count()`、バイト数は`len()`を使用
- パディング系は文字数ベースで動作（Unicode対応）

## Phase 5: マクロシステム

### 目標

コンパイル時にコードを変換するマクロシステムを実装する。

```lisp
;; whenマクロの定義
(mac when (test & body)
  `(if ,test (do ,@body)))

;; 使用例
(when (> x 10)
  (print "big")
  (print "number"))

;; 展開後:
;; (if (> x 10) (do (print "big") (print "number")))
```

### 学習内容

#### Rustで学ぶこと

1. **マクロの概念**
2. **quasiquote/unquote の実装**

#### 言語実装で学ぶこと

1. **マクロ展開の仕組み**
2. **衛生的マクロ（hygienic macros）**
3. **uvar による変数衝突回避**

### ステップ1: quasiquote/unquoteの実装

まず、ASTに新しいノードを追加:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ... 既存 ...
    Quote(Box<Expr>),
    Quasiquote(Box<Expr>),
    Unquote(Box<Expr>),
    UnquoteSplice(Box<Expr>),
}
```

レキサーにバッククォートとカンマを追加:

```rust
Some('`') => {
    self.advance();
    return Ok(Token::Backquote);
}
Some(',') if self.peek(1) == Some('@') => {
    self.advance();
    self.advance();
    return Ok(Token::UnquoteSplice);
}
Some(',') => {
    self.advance();
    return Ok(Token::Unquote);
}
```

### ステップ2: マクロの定義

```rust
// value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // ... 既存 ...
    Macro(Rc<Macro>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub params: Vec<String>,
    pub body: Expr,
    pub env: Env,
}

// expr.rs
pub enum Expr {
    // ... 既存 ...
    Mac {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
}
```

### ステップ3: マクロ展開器

```rust
struct MacroExpander {
    macros: HashMap<String, Rc<Macro>>,
}

impl MacroExpander {
    fn expand(&mut self, expr: &Expr) -> Result<Expr, String> {
        match expr {
            Expr::Call { func, args } => {
                if let Expr::Symbol(name) = func.as_ref() {
                    // マクロかチェック
                    if let Some(mac) = self.macros.get(name) {
                        // マクロを展開
                        return self.expand_macro(mac, args);
                    }
                }

                // 通常の式として再帰的に展開
                let func = Box::new(self.expand(func)?);
                let args = args.iter()
                    .map(|a| self.expand(a))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Expr::Call { func, args })
            }

            // 他の式も再帰的に展開
            _ => {
                // ... 実装 ...
            }
        }
    }

    fn expand_macro(&mut self, mac: &Macro, args: &[Expr]) -> Result<Expr, String> {
        // 1. マクロのパラメータに引数をバインド
        let mut env = mac.env.clone();
        for (param, arg) in mac.params.iter().zip(args.iter()) {
            env.set(param.clone(), /* Exprを保存 */);
        }

        // 2. マクロの本体を評価（これがquasiquoteの処理）
        let expanded = self.eval_quasiquote(&mac.body, &env)?;

        // 3. 展開結果を再度展開（ネストしたマクロに対応）
        self.expand(&expanded)
    }

    fn eval_quasiquote(&self, expr: &Expr, env: &Env) -> Result<Expr, String> {
        match expr {
            Expr::Unquote(e) => {
                // ,expr は env から値を取得
                self.eval_expr(e, env)
            }
            Expr::List(items) => {
                let mut result = Vec::new();
                for item in items {
                    match item {
                        Expr::UnquoteSplice(e) => {
                            // ,@expr はリストを展開して挿入
                            let list = self.eval_expr(e, env)?;
                            if let Expr::List(items) = list {
                                result.extend(items);
                            }
                        }
                        _ => {
                            result.push(self.eval_quasiquote(item, env)?);
                        }
                    }
                }
                Ok(Expr::List(result))
            }
            _ => Ok(expr.clone()),
        }
    }
}
```

### ステップ4: uvarの実装

変数名の衝突を避けるため、ユニークな変数を生成:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static UVAR_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn uvar() -> String {
    let id = UVAR_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("#:uvar-{}", id)
}

// 使用例
fn native_uvar(_args: &[Value]) -> Result<Value, String> {
    Ok(Value::Symbol(uvar()))
}
```

**Rustポイント**: `AtomicUsize` はスレッドセーフなカウンター。

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn get_id() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}
```

### 練習問題

1. **whenマクロの実装**: `(mac when (test & body) ...)`
2. **orマクロの実装**: 短絡評価を実現
3. **aifマクロの実装**: anaphoric if（itが使える）

---

## Phase 6: モジュールシステム

### 目標

コードを複数ファイルに分割し、再利用可能にする。

```lisp
;; math.qi
(module math)

(def square (fn [x] (* x x)))
(def cube (fn [x] (* x x x)))

(export square cube)

;; main.qi
(use math :only [square])

(print (square 5))  ; 25
```

### 学習内容

#### Rustで学ぶこと

1. **ファイルシステムの操作**
2. **HashMap の使い方**

#### 言語実装で学ぶこと

1. **モジュール解決**
2. **名前空間の管理**
3. **循環参照の検出**

### ステップ1: モジュールの定義

```rust
// value.rs
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub exports: HashMap<String, Value>,
}

// eval.rs
pub struct Evaluator {
    global_env: Rc<RefCell<Env>>,
    modules: HashMap<String, Rc<Module>>,  // 追加
    current_module: Option<String>,        // 追加
}
```

### ステップ2: useの実装

```rust
Expr::Use { module, imports } => {
    // モジュールをロード
    let module = self.load_module(&module)?;

    match imports {
        ImportSpec::Only(names) => {
            // 指定された関数のみインポート
            for name in names {
                if let Some(value) = module.exports.get(name) {
                    env.borrow_mut().set(name.clone(), value.clone());
                } else {
                    return Err(format!("{}はモジュール{}にありません", name, module.name));
                }
            }
        }
        ImportSpec::All => {
            // 全てインポート
            for (name, value) in &module.exports {
                env.borrow_mut().set(name.clone(), value.clone());
            }
        }
        ImportSpec::As(alias) => {
            // エイリアスとしてインポート
            // TODO: 名前空間付きアクセス (alias/function) を実装
        }
    }

    Ok(Value::Nil)
}

fn load_module(&mut self, name: &str) -> Result<Rc<Module>, String> {
    // キャッシュをチェック
    if let Some(module) = self.modules.get(name) {
        return Ok(module.clone());
    }

    // ファイルを探す
    let path = format!("{}.qi", name);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("モジュール{}が見つかりません: {}", name, e))?;

    // パース
    let mut parser = Parser::new(&content)?;
    let exprs = parser.parse_all()?;

    // 新しい環境で評価
    let module_env = Rc::new(RefCell::new(Env::new()));
    let mut exports = HashMap::new();

    for expr in exprs {
        match expr {
            Expr::Module(name) => {
                self.current_module = Some(name);
            }
            Expr::Export(names) => {
                // エクスポートする名前を記録
                for name in names {
                    if let Some(value) = module_env.borrow().get(&name) {
                        exports.insert(name, value);
                    }
                }
            }
            _ => {
                self.eval_with_env(&expr, module_env.clone())?;
            }
        }
    }

    let module = Rc::new(Module {
        name: name.to_string(),
        exports,
    });

    self.modules.insert(name.to_string(), module.clone());

    Ok(module)
}
```

### ステップ3: 標準ライブラリの実装

```lisp
;; stdlib/str.qi
(module str)

(def upper (fn [s]
  ;; TODO: Rustのネイティブ関数を呼ぶ
  ))

(def lower (fn [s]
  ;; TODO: 実装
  ))

(def split (fn [s sep]
  ;; TODO: 実装
  ))

(export upper lower split)
```

ネイティブ関数として実装する場合:

```rust
// stdlib.rs
pub fn register_stdlib(evaluator: &mut Evaluator) {
    let mut str_module = Module {
        name: "str".to_string(),
        exports: HashMap::new(),
    };

    str_module.exports.insert(
        "upper".to_string(),
        Value::NativeFunc(NativeFunc {
            name: "str/upper".to_string(),
            func: |args| {
                if args.len() != 1 {
                    return Err("upperには1つの引数が必要です".to_string());
                }
                match &args[0] {
                    Value::String(s) => Ok(Value::String(s.to_uppercase())),
                    _ => Err("upperは文字列が必要です".to_string()),
                }
            },
        }),
    );

    evaluator.modules.insert("str".to_string(), Rc::new(str_module));
}
```

### 練習問題

1. **循環参照の検出**: A → B → A のような循環を検出
2. **プライベート関数**: エクスポートされない関数を実装
3. **標準ライブラリ**: str, math, io モジュールを作成

---

## Phase 7: Cranelift統合

### 目標

インタプリタからJITコンパイラに移行し、実行速度を大幅に向上させる。

### 学習内容

#### Rustで学ぶこと

1. **unsafeコードの扱い**
2. **FFI (Foreign Function Interface)**
3. **ポインタとメモリ管理**

#### 言語実装で学ぶこと

1. **JITコンパイルの仕組み**
2. **中間表現（IR）**
3. **最適化技法**

### ステップ1: Craneliftのセットアップ

```toml
# Cargo.toml
[dependencies]
cranelift = "0.100"
cranelift-jit = "0.100"
cranelift-module = "0.100"
cranelift-native = "0.100"
```

```rust
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, Linkage};

pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
}

impl JITCompiler {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();

        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);

        JITCompiler {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }
}
```

### ステップ2: 簡単な関数のコンパイル

```rust
impl JITCompiler {
    // (fn [x] (+ x 1)) をコンパイル
    pub fn compile_add_one(&mut self) -> Result<*const u8, String> {
        // 関数のシグネチャを定義
        self.ctx.func.signature.params.push(AbiParam::new(types::I64));
        self.ctx.func.signature.returns.push(AbiParam::new(types::I64));

        // 関数ビルダーを作成
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // エントリーブロックを作成
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // 引数を取得
        let x = builder.block_params(entry_block)[0];

        // 1を足す
        let one = builder.ins().iconst(types::I64, 1);
        let result = builder.ins().iadd(x, one);

        // 結果を返す
        builder.ins().return_(&[result]);

        // 関数を確定
        builder.finalize();

        // 関数をコンパイル
        let id = self.module
            .declare_function("add_one", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| format!("関数宣言エラー: {}", e))?;

        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| format!("関数定義エラー: {}", e))?;

        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions();

        // 関数ポインタを取得
        let code = self.module.get_finalized_function(id);

        Ok(code)
    }

    // 関数を呼び出す
    pub fn call_add_one(&self, ptr: *const u8, arg: i64) -> i64 {
        let func: extern "C" fn(i64) -> i64 = unsafe {
            std::mem::transmute(ptr)
        };
        func(arg)
    }
}
```

**使用例**:

```rust
let mut jit = JITCompiler::new();
let func_ptr = jit.compile_add_one().unwrap();
let result = jit.call_add_one(func_ptr, 41);
assert_eq!(result, 42);
```

### ステップ3: 式のコンパイル

```rust
impl JITCompiler {
    fn compile_expr(&mut self, expr: &Expr, builder: &mut FunctionBuilder) -> Result<Value, String> {
        match expr {
            Expr::Integer(n) => {
                // 整数定数
                let val = builder.ins().iconst(types::I64, *n);
                Ok(val)
            }
            Expr::Symbol(name) => {
                // 変数の読み込み
                // TODO: 変数のマッピングが必要
                Err("変数はまだ未実装".to_string())
            }
            Expr::Call { func, args } => {
                if let Expr::Symbol(op) = func.as_ref() {
                    match op.as_str() {
                        "+" => {
                            let lhs = self.compile_expr(&args[0], builder)?;
                            let rhs = self.compile_expr(&args[1], builder)?;
                            let result = builder.ins().iadd(lhs, rhs);
                            Ok(result)
                        }
                        "-" => {
                            let lhs = self.compile_expr(&args[0], builder)?;
                            let rhs = self.compile_expr(&args[1], builder)?;
                            let result = builder.ins().isub(lhs, rhs);
                            Ok(result)
                        }
                        "*" => {
                            let lhs = self.compile_expr(&args[0], builder)?;
                            let rhs = self.compile_expr(&args[1], builder)?;
                            let result = builder.ins().imul(lhs, rhs);
                            Ok(result)
                        }
                        _ => Err(format!("未知の演算子: {}", op)),
                    }
                } else {
                    Err("関数呼び出しはまだ未実装".to_string())
                }
            }
            _ => Err("未実装の式です".to_string()),
        }
    }
}
```

### ステップ4: 動的型の扱い

Qi言語は動的型なので、タグ付きポインタを使います:

```rust
// 64ビット値の下位3ビットをタグとして使用
const TAG_MASK: u64 = 0b111;
const TAG_INT: u64 = 0b000;
const TAG_PTR: u64 = 0b001;

// 整数を encode
fn encode_int(n: i64) -> u64 {
    ((n as u64) << 3) | TAG_INT
}

// 整数を decode
fn decode_int(val: u64) -> i64 {
    (val >> 3) as i64
}

// ポインタを encode
fn encode_ptr(ptr: *mut u8) -> u64 {
    (ptr as u64) | TAG_PTR
}
```

Craneliftでの実装:

```rust
fn compile_integer(&mut self, n: i64, builder: &mut FunctionBuilder) -> Value {
    // (n << 3) | TAG_INT
    let shifted = builder.ins().iconst(types::I64, n << 3);
    let tag = builder.ins().iconst(types::I64, TAG_INT as i64);
    builder.ins().bor(shifted, tag)
}

fn compile_add(&mut self, lhs: Value, rhs: Value, builder: &mut FunctionBuilder) -> Value {
    // タグをチェック（両方とも整数か？）
    // ...

    // タグを除去
    let shift = builder.ins().iconst(types::I64, 3);
    let lhs_int = builder.ins().ushr(lhs, shift);
    let rhs_int = builder.ins().ushr(rhs, shift);

    // 加算
    let result = builder.ins().iadd(lhs_int, rhs_int);

    // タグを付ける
    let shifted = builder.ins().ishl(result, shift);
    let tag = builder.ins().iconst(types::I64, TAG_INT as i64);
    builder.ins().bor(shifted, tag)
}
```

### ステップ5: ガベージコレクション

動的にメモリを確保するため、GCが必要です。

**簡易版: 参照カウント**

```rust
struct GcValue {
    data: Value,
    ref_count: AtomicUsize,
}

impl GcValue {
    fn new(value: Value) -> *mut Self {
        let gc = Box::new(GcValue {
            data: value,
            ref_count: AtomicUsize::new(1),
        });
        Box::into_raw(gc)
    }

    fn inc_ref(ptr: *mut Self) {
        unsafe {
            (*ptr).ref_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn dec_ref(ptr: *mut Self) {
        unsafe {
            let old = (*ptr).ref_count.fetch_sub(1, Ordering::SeqCst);
            if old == 1 {
                // 参照カウントが0になったので解放
                let _ = Box::from_raw(ptr);
            }
        }
    }
}
```

**本格版: マーク&スイープ**

```rust
struct Heap {
    objects: Vec<*mut GcValue>,
    threshold: usize,
}

impl Heap {
    fn gc(&mut self, roots: &[*mut GcValue]) {
        // 1. マーク: ルートから到達可能なオブジェクトをマーク
        for root in roots {
            self.mark(*root);
        }

        // 2. スイープ: マークされていないオブジェクトを解放
        self.objects.retain(|obj| {
            unsafe {
                if (*obj).marked {
                    (*obj).marked = false;
                    true  // 保持
                } else {
                    let _ = Box::from_raw(*obj);
                    false  // 解放
                }
            }
        });
    }

    fn mark(&self, ptr: *mut GcValue) {
        unsafe {
            if (*ptr).marked {
                return;  // 既にマーク済み
            }
            (*ptr).marked = true;

            // 子オブジェクトも再帰的にマーク
            match &(*ptr).data {
                Value::List(items) => {
                    for item in items {
                        if let Value::Pointer(child) = item {
                            self.mark(*child);
                        }
                    }
                }
                // 他の型も同様に
                _ => {}
            }
        }
    }
}
```

### 練習問題

1. **ベンチマーク**: インタプリタとJITの速度を比較
2. **最適化**: 定数畳み込み（constant folding）を実装
3. **デバッグ情報**: コンパイル後のIRを表示する機能を追加

---

## 補足: Rustの重要概念

### 所有権（Ownership）

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1の所有権がs2に移動
// println!("{}", s1);  // エラー: s1はもう使えない
println!("{}", s2);  // OK
```

### 借用（Borrowing）

```rust
fn print_length(s: &String) {  // 借用（不変参照）
    println!("length: {}", s.len());
}

let s = String::from("hello");
print_length(&s);  // 所有権は移動しない
println!("{}", s);  // まだ使える
```

### 可変借用

```rust
fn append(s: &mut String) {  // 可変借用
    s.push_str(" world");
}

let mut s = String::from("hello");
append(&mut s);
println!("{}", s);  // "hello world"
```

### ライフタイム

```rust
// 'a はライフタイムパラメータ
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### トレイト

```rust
trait Printable {
    fn print(&self);
}

impl Printable for i32 {
    fn print(&self) {
        println!("integer: {}", self);
    }
}

fn print_it<T: Printable>(x: T) {
    x.print();
}
```

---

## デバッグのヒント

### 1. println!デバッグ

```rust
println!("value = {:?}", value);  // Debug出力
println!("value = {:#?}", value);  // 整形されたDebug出力
```

### 2. dbg!マクロ

```rust
let x = dbg!(some_expression());
// [src/main.rs:10] some_expression() = 42
```

### 3. cargo test -- --nocapture

```rust
#[test]
fn test_something() {
    println!("デバッグ情報");  // 通常は表示されない
    assert_eq!(1, 1);
}

// 実行: cargo test -- --nocapture
```

### 4. Rust Analyzerの使用

VSCodeで rust-analyzer をインストールすると:
- 型推論の表示
- エラーのインライン表示
- 補完機能

---

## まとめ

このチュートリアルを通じて、以下のことを学びました:

### Rust
- 所有権、借用、ライフタイム
- Rc, RefCell による共有可変性
- enum, match, Option, Result
- トレイト、ジェネリクス
- unsafe, FFI

### プログラミング言語実装
- レキサー、パーサー、評価器
- AST（抽象構文木）
- 環境（Environment）とスコープ
- クロージャの実装
- パターンマッチング
- マクロシステム
- モジュールシステム
- JITコンパイル

### Qi言語
- Lisp系の文法
- 関数型プログラミング
- 特殊形式とマクロ
- パイプライン演算子
- モジュールシステム

次のステップとして、以下に挑戦してみてください:
1. より高度な最適化（インライン展開、デッドコード削除）
2. LSP（Language Server Protocol）の実装
3. パッケージマネージャーの実装
4. 並行・並列処理のサポート

Happy hacking!
