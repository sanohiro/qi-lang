# Qi 実装戦略

## 概要

QiはRustで実装し、Cranelift JITコンパイラを使用して高速な実行を実現する。

## アーキテクチャ

### コンパイラパイプライン

```
ソースコード (.qi)
    ↓
[1] レキサー (Lexer)
    ↓
トークン列
    ↓
[2] パーサー (Parser)
    ↓
AST (抽象構文木)
    ↓
[3] マクロ展開 (Macro Expansion)
    ↓
展開済みAST
    ↓
[4] 意味解析 (Semantic Analysis)
    ↓
型付きAST / IR
    ↓
[5] Cranelift IR生成
    ↓
Cranelift IR
    ↓
[6] JITコンパイル & 実行
    ↓
ネイティブコード実行
```

### 実装言語

**Rust** を選択する理由:
- メモリ安全性
- 高速な実行
- Craneliftとの親和性（同じくRust製）
- パターンマッチング、enum、traitなど言語機能が豊富
- エコシステムが充実（pest, logos などパーサーライブラリ）

## フェーズ1: レキサー・パーサー

### レキサー

トークン化を行う。

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    // リテラル
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    Keyword(String),

    // 特殊
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,

    // 演算子
    Pipe,  // |>

    // その他
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplice,
}
```

### パーサー

S式をASTに変換。

```rust
#[derive(Debug, Clone)]
enum Expr {
    // リテラル
    Nil,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Symbol(String),
    Keyword(String),

    // コレクション
    List(Vec<Expr>),
    Vector(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),

    // 特殊形式
    Def(String, Box<Expr>),
    Fn(Vec<String>, Box<Expr>),
    Let(Vec<(String, Expr)>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Match(Box<Expr>, Vec<MatchArm>),
    Do(Vec<Expr>),
    Try(Box<Expr>),
    Defer(Box<Expr>),

    // パイプライン
    Pipeline(Box<Expr>, Box<Expr>),
}
```

## フェーズ2: マクロ展開

### マクロシステム

```rust
struct MacroEnv {
    macros: HashMap<String, Macro>,
    uvar_counter: AtomicUsize,
}

impl MacroEnv {
    fn expand(&mut self, expr: Expr) -> Expr {
        // マクロを再帰的に展開
    }

    fn new_uvar(&mut self) -> Expr {
        // 一意な変数を生成
    }
}
```

## フェーズ3: 意味解析

### 変数解決・スコープチェック

```rust
struct SemanticAnalyzer {
    scopes: Vec<HashMap<String, VarInfo>>,
    errors: Vec<SemanticError>,
}

impl SemanticAnalyzer {
    fn analyze(&mut self, expr: Expr) -> Result<TypedExpr, Vec<SemanticError>> {
        // スコープチェック
        // 未定義変数の検出
        // 型推論（オプション）
    }
}
```

## フェーズ4: Cranelift IR生成

### Craneliftバックエンド

```rust
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Module, Linkage};

struct Compiler {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
}

impl Compiler {
    fn compile_expr(&mut self, expr: &TypedExpr, builder: &mut FunctionBuilder) -> Value {
        match expr {
            TypedExpr::Integer(n) => {
                builder.ins().iconst(types::I64, *n)
            }
            TypedExpr::Add(a, b) => {
                let lhs = self.compile_expr(a, builder);
                let rhs = self.compile_expr(b, builder);
                builder.ins().iadd(lhs, rhs)
            }
            // ...
        }
    }
}
```

### 値の表現（タグ付きポインタ）

Lispの動的型を効率的に表現:

```rust
// 64ビットのタグ付きポインタ
// 下位3ビットをタグとして使用
#[repr(transparent)]
struct Value(u64);

impl Value {
    const TAG_MASK: u64 = 0b111;
    const TAG_INT: u64 = 0b000;      // 整数（即値、61ビット）
    const TAG_BOOL: u64 = 0b001;     // bool/nil
    const TAG_FLOAT: u64 = 0b010;    // float（ボックス化）
    const TAG_STRING: u64 = 0b011;   // 文字列（ポインタ）
    const TAG_CONS: u64 = 0b100;     // cons cell（ポインタ）
    const TAG_VECTOR: u64 = 0b101;   // ベクタ（ポインタ）
    const TAG_MAP: u64 = 0b110;      // マップ（ポインタ）
    const TAG_FUNC: u64 = 0b111;     // 関数（ポインタ）
}
```

## フェーズ5: ランタイム

### メモリ管理

**ガベージコレクタ**:
- 初期実装: 参照カウント（Arc/Rc）
- 将来: 世代別GC（generational GC）

```rust
struct Runtime {
    heap: Heap,
    gc: GarbageCollector,
    globals: HashMap<String, Value>,
}

struct Heap {
    allocator: BumpAllocator,
    objects: Vec<HeapObject>,
}
```

### 組み込み関数

```rust
// Rust側で実装し、Craneliftから呼び出し可能にする
#[no_mangle]
pub extern "C" fn qi_add(a: Value, b: Value) -> Value {
    // ...
}

#[no_mangle]
pub extern "C" fn qi_map(f: Value, list: Value) -> Value {
    // ...
}
```

## フェーズ6: 並行処理

### チャネル

Go風のチャネルをRustで実装:

```rust
use crossbeam::channel::{unbounded, Sender, Receiver};

struct Channel<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}
```

### go構文

軽量スレッドまたはtokio利用:

```rust
// オプション1: OS スレッド
std::thread::spawn(|| {
    // ...
});

// オプション2: async/await (tokio)
tokio::spawn(async {
    // ...
});
```

## フェーズ7: モジュールシステム

### モジュールローダー

```rust
struct ModuleLoader {
    loaded: HashMap<String, Module>,
    search_paths: Vec<PathBuf>,
}

impl ModuleLoader {
    fn load(&mut self, name: &str) -> Result<&Module, LoadError> {
        // .qi ファイルを探してコンパイル
    }
}
```

## 技術選定まとめ

| 領域 | 技術 | 理由 |
|------|------|------|
| 実装言語 | Rust | 安全性、速度、Cranelift親和性 |
| JIT | Cranelift | 高速、組み込み可能、Rust製 |
| パーサー | 手書き or pest | シンプルなS式パーサー |
| GC | 参照カウント → 世代別GC | 段階的実装 |
| 並行 | crossbeam or tokio | チャネル、並行実行 |
| 文字列 | String (UTF-8) | Unicode対応 |
| 数値 | i64, f64 | 十分な精度 |

## 開発ロードマップ

### Phase 1: MVP (Minimum Viable Product)
- [ ] レキサー・パーサー
- [ ] 基本的な特殊形式（def, fn, let, if, do）
- [ ] 整数演算
- [ ] 関数呼び出し
- [ ] Cranelift JIT統合
- [ ] REPL

### Phase 2: コア機能
- [ ] match式
- [ ] パイプライン演算子
- [ ] 文字列操作
- [ ] リスト・マップ・ベクタ
- [ ] 組み込み関数（map, filter, reduce）
- [ ] try/defer
- [ ] エラー処理

### Phase 3: マクロシステム
- [ ] quasiquote/unquote
- [ ] マクロ定義
- [ ] uvar実装
- [ ] マクロ展開

### Phase 4: モジュールシステム
- [ ] use/export
- [ ] 標準モジュール (str, csv, regex)
- [ ] パッケージマネージャー

### Phase 5: 並行・並列
- [ ] チャネル
- [ ] go構文
- [ ] pmap
- [ ] アトム

### Phase 6: 最適化
- [ ] 末尾呼び出し最適化
- [ ] インライン展開
- [ ] 世代別GC
- [ ] プロファイラ

### Phase 7: ツール
- [ ] デバッガ
- [ ] LSP (Language Server Protocol)
- [ ] パッケージマネージャー
- [ ] ドキュメント生成

## 参考実装

- **Cranelift**: https://github.com/bytecodealliance/wasmtime/tree/main/cranelift
- **Rust Lisp実装例**:
  - Ketos: https://github.com/murarth/ketos
  - Risp: https://github.com/stopachka/risp
- **Go言語のチャネル**: crossbeam-channel

## ベンチマーク目標

- 起動時間: < 10ms
- Hello World実行: < 50ms
- フィボナッチ(30): < 100ms
- メモリ使用量（アイドル時）: < 10MB

## パフォーマンス戦略

1. **JITコンパイル**: Craneliftで高速なネイティブコード生成
2. **タグ付きポインタ**: メモリ効率的な値表現
3. **末尾呼び出し最適化**: 再帰的コードの効率化
4. **インライン展開**: 小さな関数の最適化
5. **並列実行**: pmapによるマルチコア活用

## 開発環境

```bash
# Rustツールチェイン
rustup update stable

# 依存ライブラリ
cargo add cranelift
cargo add cranelift-jit
cargo add cranelift-module
cargo add crossbeam

# 開発ツール
cargo install cargo-watch
cargo install flamegraph
```

## テスト戦略

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parser() {
        // パーサーのユニットテスト
    }

    #[test]
    fn test_compiler() {
        // コンパイラの統合テスト
    }

    #[test]
    fn test_runtime() {
        // ランタイムのテスト
    }
}
```

## 次のステップ

1. プロジェクト初期化: `cargo init --bin`
2. レキサー実装
3. パーサー実装
4. 簡単なCranelift統合（整数演算のみ）
5. REPL作成

実装を開始する準備ができました。
