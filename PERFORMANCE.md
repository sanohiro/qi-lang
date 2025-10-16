# パフォーマンスガイド

## 概要

このドキュメントでは、Qi言語インタプリタのパフォーマンス最適化の取り組みと、実装時に適用すべきベストプラクティスを記録します。

Phase 1-21の最適化により、**40-50%のパフォーマンス改善**を達成しました。

## ベンチマーク結果

### 最適化前後の比較

主要な最適化フェーズでの改善率：

- **Phase 1-10**: 初期最適化（Arc<RwLock<>>の導入、クローン削減）
  - 平均改善率: 約30%

- **Phase 11-21**: 高度な最適化（イテレータ活用、不要な割り当て削減）
  - 平均改善率: 約15-20%の追加改善

- **総合**: Phase 1から21までで40-50%のパフォーマンス向上

### ベンチマーク実行方法

```bash
# 基本ベンチマーク
cargo bench

# 特定のベンチマークのみ実行
cargo bench --bench <ベンチマーク名>

# リリースビルドでの実行時間測定
time cargo run --release -- examples/benchmark.qi
```

## ベストプラクティス

### 1. Arc<RwLock<T>>の効果的な活用

**推奨**:
```rust
// 環境を共有する場合はArc<RwLock<>>を使用
let env = Arc::new(RwLock::new(Env::new()));
let env_clone = Arc::clone(&env);
```

**効果**: スレッドセーフな共有と最小限のクローンでメモリ効率向上

### 2. 不要なクローンの削減

**推奨**:
```rust
// 参照を活用
fn process_value(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.as_str().to_string()),
        _ => Err("Not a string".to_string())
    }
}
```

**避けるべき**:
```rust
// 不要なクローン
fn process_value(value: Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),  // 余分なクローン
        _ => Err("Not a string".to_string())
    }
}
```

### 3. イテレータの活用

**推奨**:
```rust
// イテレータで直接処理
let sum: i64 = values
    .iter()
    .filter_map(|v| match v {
        Value::Integer(n) => Some(n),
        _ => None,
    })
    .sum();
```

**避けるべき**:
```rust
// 中間ベクタを作る
let mut integers = Vec::new();
for v in values {
    if let Value::Integer(n) = v {
        integers.push(n);
    }
}
let sum: i64 = integers.iter().sum();
```

### 4. 文字列の効率的な構築

**推奨**:
```rust
// format!の代わりにpush_strを使う（ループ内で）
let mut result = String::with_capacity(estimated_size);
for item in items {
    result.push_str(&item.to_string());
    result.push('\n');
}
```

**避けるべき**:
```rust
// ループ内でformat!を繰り返す
let mut result = String::new();
for item in items {
    result = format!("{}{}\n", result, item);  // 毎回新しい文字列を割り当て
}
```

### 5. im::Vectorの活用

**推奨**:
```rust
// 永続データ構造を活用
use im::Vector;

let v1: Vector<Value> = Vector::new();
let v2 = v1.push_back(value);  // 構造共有により効率的
```

**効果**: 関数型プログラミングスタイルで効率的なデータ共有

### 6. LazyLockによる遅延初期化

**推奨**:
```rust
use std::sync::LazyLock;

static GLOBAL_CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    RwLock::new(Config::default())
});
```

**効果**: 初回アクセス時のみ初期化、スレッドセーフな遅延評価

### 7. 型変換の最適化

**推奨**:
```rust
// 型推論を活用し、不要な.into()を避ける
let map: im::HashMap<String, Value> = HashMap::new();
Ok(Value::Map(map))  // .into()不要
```

**避けるべき**:
```rust
// 不要な型変換
let map: im::HashMap<String, Value> = HashMap::new();
Ok(Value::Map(map.into()))  // 冗長
```

## アンチパターン

### ❌ 1. 過度なクローン

**悪い例**:
```rust
fn bad_clone(env: &Arc<RwLock<Env>>, value: &Value) -> Value {
    let env_copy = env.clone();  // Arc自体をクローン（不要）
    let value_copy = value.clone();  // Valueをクローン（不要）
    value_copy
}
```

**良い例**:
```rust
fn good_reference(env: &Arc<RwLock<Env>>, value: &Value) -> Value {
    // 必要な箇所でのみクローン
    value.clone()
}
```

**理由**: Arcのクローンは安価だが、Valueのクローンは高コスト。必要最小限に留める。

### ❌ 2. String連結でのformat!乱用

**悪い例**:
```rust
let mut result = String::new();
for i in 0..1000 {
    result = format!("{}{}", result, i);  // O(n²)の複雑度
}
```

**良い例**:
```rust
let mut result = String::with_capacity(4000);
for i in 0..1000 {
    result.push_str(&i.to_string());  // O(n)の複雑度
}
```

**理由**: `format!`は毎回新しい文字列を割り当てるため、ループ内では非効率。

### ❌ 3. 中間ベクタの不要な作成

**悪い例**:
```rust
fn bad_filter(values: &[Value]) -> Vec<i64> {
    let mut temp = Vec::new();
    for v in values {
        if let Value::Integer(n) = v {
            temp.push(*n);
        }
    }
    temp.into_iter().filter(|n| n > &0).collect()
}
```

**良い例**:
```rust
fn good_filter(values: &[Value]) -> Vec<i64> {
    values
        .iter()
        .filter_map(|v| match v {
            Value::Integer(n) if *n > 0 => Some(*n),
            _ => None,
        })
        .collect()
}
```

**理由**: イテレータチェーンで一度に処理することでメモリ割り当てを削減。

### ❌ 4. ロックの長期保持

**悪い例**:
```rust
fn bad_lock(env: &Arc<RwLock<Env>>) -> Result<Value, String> {
    let env_read = env.read();  // ロック取得
    // ... 長時間の処理 ...
    expensive_computation();  // ロックを保持したまま重い処理
    Ok(env_read.get("key").cloned().unwrap_or(Value::Nil))
}  // ここでようやくロック解放
```

**良い例**:
```rust
fn good_lock(env: &Arc<RwLock<Env>>) -> Result<Value, String> {
    let value = {
        let env_read = env.read();  // スコープを限定
        env_read.get("key").cloned().unwrap_or(Value::Nil)
    };  // ここでロック解放
    // ロック解放後に重い処理
    expensive_computation();
    Ok(value)
}
```

**理由**: ロックを長時間保持すると並行性が低下。最小限のスコープで解放する。

### ❌ 5. 不要な.into()や型変換

**悪い例**:
```rust
fn bad_conversion(items: im::Vector<Value>) -> Value {
    Value::List(items.into())  // 既にim::Vectorなのに.into()
}
```

**良い例**:
```rust
fn good_conversion(items: im::Vector<Value>) -> Value {
    Value::List(items)  // 型推論に任せる
}
```

**理由**: Rustの型推論は強力。明示的な変換は冗長で、パフォーマンスに影響する可能性も。

### ❌ 6. or_insert_with(Constructor::new)

**悪い例**:
```rust
map.entry(key)
    .or_insert_with(Vec::new)  // Defaultトレイトがあるのに
    .push(value);
```

**良い例**:
```rust
map.entry(key)
    .or_default()  // より簡潔で意図が明確
    .push(value);
```

**理由**: `or_default()`の方が簡潔で、Defaultトレイトの実装を活用できる。

## プロファイリング方法

### cargo-flameによるフレームグラフ生成

```bash
# インストール
cargo install flamegraph

# フレームグラフ生成
cargo flamegraph --bin qi

# SVGファイルが生成される（flamegraph.svg）
```

### cargo-benchによるベンチマーク

```bash
# ベンチマーク実行
cargo bench

# 結果の比較
cargo bench -- --save-baseline before
# ... コード変更 ...
cargo bench -- --baseline before
```

### perf（Linux）によるプロファイリング

```bash
# プロファイルの記録
perf record --call-graph dwarf cargo run --release -- script.qi

# レポート表示
perf report
```

### Instruments（macOS）によるプロファイリング

```bash
# Time Profilerでプロファイル
instruments -t "Time Profiler" cargo run --release -- script.qi
```

## 継続的な改善

### パフォーマンス測定の自動化

- CI/CDパイプラインにベンチマークを組み込む
- リグレッション検出の仕組みを導入
- 定期的なプロファイリングの実施

### メモリプロファイリング

```bash
# valgrindによるメモリリーク検出（Linux）
valgrind --leak-check=full cargo run --release -- script.qi

# heaptrackによるメモリ使用量分析
heaptrack cargo run --release -- script.qi
```

## 参考資料

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [cargo-bench](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)
