# 開発ガイドライン

## コーディング規約

### メモリ管理

#### 循環参照の防止

Qiは`Arc`を多用するため、循環参照に注意が必要です：

```qi
;; ❌ 悪い例: 循環参照を作成
(def a [])
(def b [a])
(vector-set! a 0 b)  ;; a → b → a の循環

;; ✅ 良い例: 循環を避ける
(def a [1 2 3])
(def b (conj a 4))  ;; 新しいベクタを作成
```

**対策**:
- データ構造を変更する前に、循環参照の可能性を確認
- 長時間動作するプログラムでは、定期的に不要な参照をクリア
- `atom`の使用時は特に注意（`swap!`で循環が発生しやすい）

### エラー処理

#### unwrap/expectの使用制限

新しいコードでは、`unwrap()`と`expect()`の使用を避けてください：

```rust
// ❌ 悪い例
let value = map.get(key).unwrap();

// ✅ 良い例1: ok_or_else
let value = map.get(key).ok_or_else(|| 
    fmt_msg(MsgKey::KeyNotFound, &[&key.to_string()])
)?;

// ✅ 良い例2: unwrap_or
let value = map.get(key).cloned().unwrap_or(Value::Nil);

// ✅ 良い例3: if let
if let Some(value) = map.get(key) {
    // use value
} else {
    return Err(fmt_msg(MsgKey::KeyNotFound, &[&key.to_string()]));
}
```

**許容される使用例**:
- テストコード内
- 静的データ（`LazyLock`の初期化等）
- 数学的に証明可能な不変条件（例: `0..len`のループ内で`vec[i]`）

**Clippyチェック**:
```bash
cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used
```

### 整数変換

#### i64→usizeの安全な変換

プラットフォーム依存の変換では`try_from`を使用：

```rust
// ❌ 悪い例: 32bitでオーバーフロー
let n = value as usize;

// ✅ 良い例
let n = usize::try_from(value).map_err(|_| 
    fmt_msg(MsgKey::ValueTooLargeForI64, &["function", &value.to_string()])
)?;
```

#### float→i64の安全な変換

範囲チェックを忘れずに：

```rust
// ❌ 悪い例: UB
let n = f as i64;

// ✅ 良い例
if f.is_nan() || f.is_infinite() {
    return Err(fmt_msg(MsgKey::FloatIsNanOrInfinity, &["function"]));
}
if f < i64::MIN as f64 || f > i64::MAX as f64 {
    return Err(fmt_msg(MsgKey::FloatOutOfI64Range, &["function", &f.to_string()]));
}
let n = f as i64;
```

## セキュリティ

### コマンドインジェクション対策

シェルを経由してコマンドを実行する際は、入力検証を徹底：

```rust
// ❌ 悪い例: シェルインジェクション脆弱性
Command::new("sh").arg("-c").arg(user_input).output()

// ✅ 良い例: 直接実行
Command::new(program).args(&args).output()
```

### DoS対策

外部からのデータを処理する際は、サイズ制限を設定：

```rust
// HTTPレスポンス
const MAX_BODY_SIZE: usize = 100 * 1024 * 1024; // 100MB

// 再帰深度
const MAX_RECURSION_DEPTH: usize = 1000;

// コレクションサイズ
const MAX_COLLECTION_SIZE: usize = 1_000_000;
```

## テスト

### Torture Test

処理系の実装漏れをチェックするため、以下のパターンをテスト：

- すべての型の組み合わせ
- 境界値（0, 1, -1, i64::MAX, i64::MIN）
- 空コレクション
- ネストした構造
- 並行実行

## Clippy設定

推奨リント：

```toml
[lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
```

既存コードとの互換性のため、現時点では警告レベルに設定。
