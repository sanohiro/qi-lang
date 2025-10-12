# 並行処理

Rustの並行処理とスレッドセーフなデータ共有について学びます。

## Rustの並行性の特徴

Rustは**所有権システム**により、コンパイル時にデータ競合を防ぎます。

```rust
// これはコンパイルエラー！
let mut data = vec![1, 2, 3];

std::thread::spawn(|| {
    data.push(4);  // エラー！data の所有権がない
});

data.push(5);  // エラー！data はムーブされている
```

Rustのコンパイラが教えてくれます：
> "closure may outlive the current function"
> "data may be modified concurrently"

## スレッドの基本

### スレッドの作成

```rust
use std::thread;

// スレッドを生成
let handle = thread::spawn(|| {
    println!("Hello from thread!");
});

// スレッドの終了を待つ
handle.join().unwrap();
```

### 値を返すスレッド

```rust
let handle = thread::spawn(|| {
    // 何か計算
    42
});

let result = handle.join().unwrap();
println!("Result: {}", result);  // 42
```

### qi-langでの使用例

```rust
// src/builtins/core_concurrency.rs
pub fn native_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    let func = args[0].clone();
    let evaluator = evaluator.clone();

    let handle = std::thread::spawn(move || {
        evaluator.apply_function(&func, &[])
    });

    Ok(Value::Handle(Arc::new(RwLock::new(Some(handle)))))
}
```

## Arc<T> - 原子参照カウント

複数のスレッドで所有権を共有します。

### 基本的な使い方

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3, 4, 5]);

let mut handles = vec![];

for i in 0..3 {
    let data = Arc::clone(&data);  // 参照カウントを増やす

    let handle = thread::spawn(move || {
        println!("Thread {}: {:?}", i, data);
    });

    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}
```

**`Arc`の特徴:**
- **A**tomic **R**eference **C**ounted（原子参照カウント）
- スレッドセーフな`Rc`（参照カウント）
- 読み取り専用の共有

### qi-langでの使用例

```rust
// src/eval.rs
pub struct Evaluator {
    global_env: Arc<RwLock<Env>>,  // 複数スレッドで共有
    defer_stack: Arc<RwLock<Vec<Vec<Expr>>>>,
    modules: Arc<RwLock<HashMap<String, Arc<Module>>>>,
    // ...
}
```

## Mutex<T> - 排他制御

可変データへの排他アクセスを提供します。

### 基本的な使い方

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);

    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });  // ロックはここで自動的に解放される

    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", *counter.lock().unwrap());  // 10
```

**`Mutex`の特徴:**
- **Mut**ual **Ex**clusion（相互排他）
- 一度に1つのスレッドだけがアクセス可能
- `lock()`でロックを取得、スコープを抜けると自動解放

### デッドロックの危険性

```rust
let data = Arc::new(Mutex::new(0));
let data2 = Arc::clone(&data);

let handle = thread::spawn(move || {
    let _lock1 = data2.lock().unwrap();
    // 長い処理...
});

// デッドロックの可能性
let _lock2 = data.lock().unwrap();
handle.join().unwrap();
```

## RwLock<T> - 読み書きロック

読み取りと書き込みを分離したロックです。

### 基本的な使い方

```rust
use std::sync::{Arc, RwLock};
use std::thread;

let data = Arc::new(RwLock::new(vec![1, 2, 3]));
let mut handles = vec![];

// 複数の読み取りスレッド
for i in 0..5 {
    let data = Arc::clone(&data);

    let handle = thread::spawn(move || {
        let vec = data.read().unwrap();
        println!("Reader {}: {:?}", i, *vec);
    });

    handles.push(handle);
}

// 1つの書き込みスレッド
let data_writer = Arc::clone(&data);
let handle = thread::spawn(move || {
    let mut vec = data_writer.write().unwrap();
    vec.push(4);
});
handles.push(handle);

for handle in handles {
    handle.join().unwrap();
}
```

**`RwLock`の特徴:**
- 複数の読み取り、または1つの書き込み
- 読み取りが多い場合に効率的
- `read()`: 読み取りロック（共有）
- `write()`: 書き込みロック（排他）

### qi-langでの使用例

```rust
// src/eval.rs
fn eval_with_env(&self, expr: &Expr, env: Arc<RwLock<Env>>) -> Result<Value, String> {
    match expr {
        Expr::Symbol(name) => {
            // 読み取りロック
            env.read().get(name).ok_or_else(|| {
                format!("Undefined variable: {}", name)
            })
        }
        Expr::Def(name, value) => {
            let val = self.eval_with_env(value, env.clone())?;
            // 書き込みロック
            env.write().set(name.clone(), val.clone());
            Ok(val)
        }
        // ...
    }
}
```

## parking_lot - より効率的なロック

qi-langでは`parking_lot`クレートを使用しています。

```rust
use parking_lot::RwLock;  // std の RwLock より高速

let data = Arc::new(RwLock::new(0));

// 読み取り
let value = data.read();
println!("{}", *value);

// 書き込み
let mut value = data.write();
*value += 1;
```

**`parking_lot`の利点:**
- より高速
- より小さいメモリフットプリント
- `unwrap()`不要（Poisoning なし）
- デッドロック検出機能

## チャネル - メッセージパッシング

スレッド間でデータを送受信します。

### 標準ライブラリのチャネル

```rust
use std::sync::mpsc;
use std::thread;

// チャネルを作成
let (tx, rx) = mpsc::channel();

// 送信スレッド
thread::spawn(move || {
    tx.send("hello").unwrap();
});

// 受信
let message = rx.recv().unwrap();
println!("{}", message);  // "hello"
```

### crossbeam チャネル

qi-langでは`crossbeam`クレートを使用しています。

```rust
use crossbeam::channel;

// 無制限チャネル
let (tx, rx) = channel::unbounded();

// 有制限チャネル
let (tx, rx) = channel::bounded(10);

// 送信
tx.send(42).unwrap();

// 受信
let value = rx.recv().unwrap();

// try_recv - ブロックしない
match rx.try_recv() {
    Ok(value) => println!("Received: {}", value),
    Err(_) => println!("No message"),
}
```

### qi-langでの使用例

```rust
// src/builtins/core_concurrency.rs
pub fn native_chan(args: &[Value]) -> Result<Value, String> {
    let capacity = if args.is_empty() { 0 } else { /* ... */ };

    let (tx, rx) = if capacity == 0 {
        crossbeam::channel::unbounded()
    } else {
        crossbeam::channel::bounded(capacity)
    };

    Ok(Value::Channel(Arc::new(RwLock::new((tx, rx)))))
}

pub fn native_send(args: &[Value]) -> Result<Value, String> {
    match &args[0] {
        Value::Channel(ch) => {
            let (tx, _) = &*ch.read();
            tx.send(args[1].clone()).map_err(|e| format!("{}", e))?;
            Ok(Value::Nil)
        }
        _ => Err("send! expects a channel".to_string()),
    }
}
```

## データ競合の防止

Rustの型システムがデータ競合を防ぎます。

### 例1: 可変参照の共有を防ぐ

```rust
let mut data = vec![1, 2, 3];

// コンパイルエラー！
let handle = thread::spawn(|| {
    data.push(4);  // &mut を要求するが、所有権がない
});

data.push(5);
```

### 例2: Arc<Mutex<T>> で解決

```rust
let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let data2 = Arc::clone(&data);

let handle = thread::spawn(move || {
    data2.lock().unwrap().push(4);  // OK
});

data.lock().unwrap().push(5);  // OK
handle.join().unwrap();
```

### 例3: Send と Sync トレイト

```rust
// Send: スレッド間で所有権を移動できる
// Sync: スレッド間で参照を共有できる

// Rc<T> は Send でも Sync でもない
use std::rc::Rc;
let data = Rc::new(5);
// thread::spawn(move || { println!("{}", data); });  // エラー！

// Arc<T> は Send + Sync
use std::sync::Arc;
let data = Arc::new(5);
thread::spawn(move || { println!("{}", data); });  // OK
```

## 並行処理のパターン

### パターン1: 並列マップ

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn parallel_map<T, F>(data: Vec<T>, f: F) -> Vec<T>
where
    T: Send + 'static,
    F: Fn(T) -> T + Send + Sync + 'static,
{
    let f = Arc::new(f);
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for item in data {
        let f = Arc::clone(&f);
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let result = f(item);
            results.lock().unwrap().push(result);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}
```

### パターン2: ワーカースレッドプール

```rust
use crossbeam::channel;
use std::thread;

struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: channel::Sender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = channel::unbounded();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let receiver = Arc::clone(&receiver);

            let handle = thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv();
                match job {
                    Ok(job) => job(),
                    Err(_) => break,
                }
            });

            workers.push(handle);
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}
```

## qi-langの並行処理実装

### goroutine風の非同期実行

```rust
// (go (fn [] (expensive-computation)))
pub fn native_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("go requires exactly 1 argument".to_string());
    }

    let func = args[0].clone();
    let evaluator = evaluator.clone();

    let handle = std::thread::spawn(move || {
        evaluator.apply_function(&func, &[])
    });

    Ok(Value::Handle(Arc::new(RwLock::new(Some(handle)))))
}
```

### 並列map

```rust
// src/eval.rs
fn eval_pmap(&self, args: &[Expr], env: Arc<RwLock<Env>>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("pmap requires 2 arguments".to_string());
    }

    let func = self.eval_with_env(&args[0], env.clone())?;
    let coll = self.eval_with_env(&args[1], env.clone())?;

    match coll {
        Value::List(items) | Value::Vector(items) => {
            let results = Arc::new(RwLock::new(Vec::new()));
            let mut handles = vec![];

            for item in items {
                let func = func.clone();
                let evaluator = self.clone();
                let results = Arc::clone(&results);

                let handle = std::thread::spawn(move || {
                    let result = evaluator.apply_function(&func, &[item])?;
                    results.write().push(result);
                    Ok::<(), String>(())
                });

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap()?;
            }

            let results = Arc::try_unwrap(results)
                .unwrap()
                .into_inner();

            Ok(Value::Vector(results))
        }
        _ => Err("pmap expects a collection".to_string()),
    }
}
```

## ベストプラクティス

### 1. 適切な同期プリミティブを選ぶ

- **読み取りが多い**: `RwLock`
- **書き込みが多い**: `Mutex`
- **メッセージパッシング**: チャネル

### 2. ロックの範囲を最小化

```rust
// 悪い例
let data = data.lock().unwrap();
expensive_computation();  // ロックを持ったまま
println!("{}", *data);

// 良い例
let value = {
    let data = data.lock().unwrap();
    *data  // すぐにコピー
};  // ロック解放
expensive_computation();
println!("{}", value);
```

### 3. デッドロックを避ける

- ロックの順序を統一
- ロックの保持時間を短く
- 複数ロックを同時に取得しない

### 4. Cloneを最小化

```rust
// Arc::clone() は参照カウントの増加のみ（安価）
let data2 = Arc::clone(&data);

// Value のクローンは実際のデータをコピー（高価）
let value2 = value.clone();
```

## まとめ

Rustの並行処理：

1. **スレッド**: `std::thread::spawn()`
2. **Arc<T>**: 所有権の共有（読み取り専用）
3. **Mutex<T>**: 排他アクセス（読み書き）
4. **RwLock<T>**: 読み書き分離ロック
5. **チャネル**: メッセージパッシング
6. **Send / Sync**: スレッド安全性の保証

これらにより、Rustは**コンパイル時にデータ競合を防ぐ**並行処理を実現します。

## 次のステップ

次は[マクロ](./06-macros.md)を学びます。Rustのマクロシステムについて理解しましょう。
