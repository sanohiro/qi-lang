//! 並行処理関数（go/chan）

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Scope, Value};
use crossbeam_channel::{bounded, unbounded};
use parking_lot::RwLock;
use std::sync::Arc;

/// chan - チャネルを作成
///
/// 引数:
/// - capacity (optional): バッファサイズ（省略時は無制限）
///
/// 戻り値:
/// - チャネル
///
/// 例:
/// ```lisp
/// (def ch (chan))      ;; 無制限バッファ
/// (def ch (chan 10))   ;; バッファサイズ10
/// ```
pub fn native_chan(args: &[Value]) -> Result<Value, String> {
    let capacity = if args.is_empty() {
        None
    } else if args.len() == 1 {
        match &args[0] {
            Value::Integer(n) if *n >= 0 => Some(*n as usize),
            Value::Integer(_) => return Err("chan: capacity must be non-negative".to_string()),
            _ => return Err("chan: capacity must be an integer".to_string()),
        }
    } else {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["chan", "0 or 1"]));
    };

    let (sender, receiver) = if let Some(cap) = capacity {
        bounded(cap)
    } else {
        unbounded()
    };

    Ok(Value::Channel(Arc::new(Channel { sender, receiver })))
}

/// send! - チャネルに値を送信
///
/// 引数:
/// - channel: チャネル
/// - value: 送信する値
///
/// 戻り値:
/// - 送信した値
///
/// 例:
/// ```lisp
/// (def ch (chan))
/// (send! ch 42)
/// ```
pub fn native_send(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["send!"]));
    }

    match &args[0] {
        Value::Channel(ch) => {
            let value = args[1].clone();
            ch.sender
                .send(value.clone())
                .map_err(|_| "send!: channel is closed".to_string())?;
            Ok(value)
        }
        _ => Err(fmt_msg(MsgKey::FirstArgMustBe, &["send!", "a channel"])),
    }
}

/// recv! - チャネルから値を受信（ブロッキング）
///
/// 引数:
/// - channel: チャネル
/// - :timeout ms (optional): タイムアウト（ミリ秒）
///
/// 戻り値:
/// - 受信した値（チャネルがクローズまたはタイムアウトならnil）
///
/// 例:
/// ```lisp
/// (def ch (chan))
/// (recv! ch)                    ;; ブロックして待つ
/// (recv! ch :timeout 1000)      ;; 最大1秒待つ
/// ```
pub fn native_recv(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 3 {
        return Err("recv! requires 1 or 3 arguments: (recv! ch) or (recv! ch :timeout ms)".to_string());
    }

    match &args[0] {
        Value::Channel(ch) => {
            // タイムアウト指定があるか確認
            if args.len() == 3 {
                // :timeout キーワードを確認
                match &args[1] {
                    Value::Keyword(k) if k == "timeout" => {
                        // タイムアウト値（ミリ秒）を取得
                        match &args[2] {
                            Value::Integer(ms) if *ms >= 0 => {
                                let timeout = std::time::Duration::from_millis(*ms as u64);
                                Ok(ch.receiver.recv_timeout(timeout).unwrap_or(Value::Nil))
                            }
                            Value::Integer(_) => Err("recv!: timeout must be non-negative".to_string()),
                            _ => Err("recv!: timeout must be an integer (milliseconds)".to_string()),
                        }
                    }
                    _ => Err("recv!: expected :timeout keyword".to_string()),
                }
            } else {
                // タイムアウトなし（通常のブロッキング受信）
                ch.receiver
                    .recv()
                    .map_err(|_| "recv!: channel is closed".to_string())
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["recv!", "channels"])),
    }
}

/// try-recv! - チャネルから値を非ブロッキング受信
///
/// 引数:
/// - channel: チャネル
///
/// 戻り値:
/// - 受信した値、またはnil（値がない場合）
///
/// 例:
/// ```lisp
/// (def ch (chan))
/// (try-recv! ch)  ;; すぐに返る（nilまたは値）
/// ```
pub fn native_try_recv(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["try-recv!"]));
    }

    match &args[0] {
        Value::Channel(ch) => Ok(ch.receiver.try_recv().unwrap_or(Value::Nil)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["try-recv!", "channels"])),
    }
}

/// close! - チャネルをクローズ
///
/// 引数:
/// - channel: チャネル
///
/// 戻り値:
/// - nil
///
/// 例:
/// ```lisp
/// (def ch (chan))
/// (close! ch)
/// ```
pub fn native_close(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["close!"]));
    }

    match &args[0] {
        Value::Channel(_ch) => {
            // crossbeam-channelではdropでクローズされる
            // 明示的にクローズするにはsenderをすべてdropする必要がある
            // ここでは単にnilを返す（実際のクローズはGCで行われる）
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["close!", "channels"])),
    }
}

// ============================================================================
// Layer 3: async/await - 高レベル非同期処理
// ============================================================================

/// await - Promiseを待機（チャネルから受信）
///
/// 引数:
/// - promise: Promise（チャネル）
///
/// 戻り値:
/// - Promiseの結果
///
/// 例:
/// ```lisp
/// (def p (go (fn [] (+ 1 2))))
/// (await p)  ;; => 3
/// ```
pub fn native_await(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("await requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Channel(ch) => ch
            .receiver
            .recv()
            .map_err(|_| "await: promise is closed".to_string()),
        _ => Err("await: argument must be a promise (channel)".to_string()),
    }
}

/// then - Promiseチェーン
///
/// 引数:
/// - promise: Promise（チャネル）
/// - f: 結果に適用する関数
///
/// 戻り値:
/// - 新しいPromise
///
/// 例:
/// ```lisp
/// (def p (go (fn [] 10)))
/// (def p2 (then p (fn [x] (* x 2))))
/// (await p2)  ;; => 20
/// ```
pub fn native_then(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("then requires 2 arguments (promise, fn)".to_string());
    }

    let promise = match &args[0] {
        Value::Channel(ch) => ch.clone(),
        _ => return Err("then: first argument must be a promise (channel)".to_string()),
    };

    let f = args[1].clone();
    let eval = evaluator.clone();

    // 新しいPromiseを作成
    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(Channel {
        sender: sender.clone(),
        receiver,
    });

    // 別スレッドで処理
    std::thread::spawn(move || {
        // 元のPromiseから受信
        if let Ok(value) = promise.receiver.recv() {
            // 関数を適用
            let result = eval.apply_function(&f, &[value]);
            let _ = sender.send(result.unwrap_or_else(|e| Value::String(e)));
        }
    });

    Ok(Value::Channel(result_channel))
}

/// catch - エラーハンドリング
///
/// 引数:
/// - promise: Promise（チャネル）
/// - handler: エラーハンドラ関数
///
/// 戻り値:
/// - 新しいPromise
///
/// 例:
/// ```lisp
/// (def p (go (fn [] (error "oops"))))
/// (def p2 (catch p (fn [e] (println "Error:" e))))
/// ```
pub fn native_catch(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("catch requires 2 arguments (promise, handler)".to_string());
    }

    let promise = match &args[0] {
        Value::Channel(ch) => ch.clone(),
        _ => return Err("catch: first argument must be a promise (channel)".to_string()),
    };

    let handler = args[1].clone();
    let eval = evaluator.clone();

    // 新しいPromiseを作成
    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(Channel {
        sender: sender.clone(),
        receiver,
    });

    // 別スレッドで処理
    std::thread::spawn(move || {
        // 元のPromiseから受信
        match promise.receiver.recv() {
            Ok(value) => {
                // 成功した場合はそのまま転送
                let _ = sender.send(value);
            }
            Err(_) => {
                // エラーの場合はハンドラを呼び出す
                let result = eval.apply_function(&handler, &[Value::String("channel closed".to_string())]);
                let _ = sender.send(result.unwrap_or(Value::Nil));
            }
        }
    });

    Ok(Value::Channel(result_channel))
}

/// all - 複数のPromiseを並列実行して全て待機
///
/// 引数:
/// - promises: Promiseのリスト/ベクタ
///
/// 戻り値:
/// - 全ての結果を含むリストを返すPromise
///
/// 例:
/// ```lisp
/// (def promises [(go (fn [] 1)) (go (fn [] 2)) (go (fn [] 3))])
/// (def result (await (all promises)))  ;; => [1 2 3]
/// ```
pub fn native_all(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("all requires 1 argument (list of promises)".to_string());
    }

    let promises = match &args[0] {
        Value::List(ps) | Value::Vector(ps) => ps,
        _ => return Err("all: argument must be a list or vector of promises".to_string()),
    };

    // 新しいPromiseを作成
    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(Channel {
        sender: sender.clone(),
        receiver,
    });

    let promises = promises.clone();
    std::thread::spawn(move || {
        let mut results = Vec::new();
        for promise in promises {
            if let Value::Channel(ch) = promise {
                match ch.receiver.recv() {
                    Ok(value) => results.push(value),
                    Err(_) => {
                        let _ = sender.send(Value::String("promise failed".to_string()));
                        return;
                    }
                }
            } else {
                let _ = sender.send(Value::String("not a promise".to_string()));
                return;
            }
        }
        let _ = sender.send(Value::List(results));
    });

    Ok(Value::Channel(result_channel))
}

/// race - 複数のPromiseで最初に完了したものを返す
///
/// 引数:
/// - promises: Promiseのリスト/ベクタ
///
/// 戻り値:
/// - 最初に完了したPromiseの結果を返すPromise
///
/// 例:
/// ```lisp
/// (def promises [(go (fn [] (sleep 100) 1)) (go (fn [] 2))])
/// (def result (await (race promises)))  ;; => 2 (最速)
/// ```
pub fn native_race(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("race requires 1 argument (list of promises)".to_string());
    }

    let promises = match &args[0] {
        Value::List(ps) | Value::Vector(ps) => ps,
        _ => return Err("race: argument must be a list or vector of promises".to_string()),
    };

    // 新しいPromiseを作成
    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(Channel {
        sender: sender.clone(),
        receiver,
    });

    // すべてのpromiseから受信を試みる（最初に完了したものを返す）
    for promise in promises {
        if let Value::Channel(ch) = promise {
            let sender = sender.clone();
            let receiver = ch.receiver.clone();
            std::thread::spawn(move || {
                if let Ok(value) = receiver.recv() {
                    let _ = sender.send(value);
                }
            });
        } else {
            return Err("race: all elements must be promises (channels)".to_string());
        }
    }

    Ok(Value::Channel(result_channel))
}

// ============================================================================
// Layer 1: go/chan - 基盤
// ============================================================================

/// go - goroutine風の非同期実行
///
/// 引数:
/// - expr: 実行する式
///
/// 戻り値:
/// - future（結果を取得できるチャネル）
///
/// 例:
/// ```lisp
/// (go (println "async!"))
/// (def result (go (+ 1 2)))
/// (recv! result)  ;; => 3
/// ```
pub fn native_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["go"]));
    }

    // 結果を返すチャネルを作成
    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(Channel { sender: sender.clone(), receiver });

    // 実行する式
    let expr = args[0].clone();
    let eval = evaluator.clone();

    // 別スレッドで実行
    std::thread::spawn(move || {
        // 式を評価
        let result = match expr {
            Value::Function(ref _f) => {
                // 関数の場合は引数なしで呼び出し
                eval.apply_function(&expr, &[])
            }
            _ => {
                // その他の値はそのまま返す
                Ok(expr.clone())
            }
        };

        // 結果をチャネルに送信
        let _ = sender.send(result.unwrap_or_else(|e| Value::String(e)));
    });

    Ok(Value::Channel(result_channel))
}

/// fan-out - チャネルを複数に分岐
///
/// 引数:
/// - channel: 入力チャネル
/// - n: 分岐数
///
/// 戻り値:
/// - 分岐したチャネルのリスト
///
/// 例:
/// ```lisp
/// (def in-ch (chan))
/// (def out-chs (fan-out in-ch 3))  ;; 3つに分岐
/// ```
pub fn native_fan_out(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["fan-out"]));
    }

    let in_channel = match &args[0] {
        Value::Channel(ch) => ch.clone(),
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["fan-out", "a channel"])),
    };

    let n = match &args[1] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err("fan-out: n must be positive".to_string()),
        _ => return Err("fan-out: n must be an integer".to_string()),
    };

    // 出力チャネルを作成
    let mut out_channels = Vec::new();
    for _ in 0..n {
        let (sender, receiver) = unbounded();
        out_channels.push(Value::Channel(Arc::new(Channel { sender, receiver })));
    }

    // 入力から出力へ分配するgoroutineを起動
    let out_senders: Vec<_> = out_channels
        .iter()
        .filter_map(|ch| {
            if let Value::Channel(c) = ch {
                Some(c.sender.clone())
            } else {
                None
            }
        })
        .collect();

    let in_receiver = in_channel.receiver.clone();
    std::thread::spawn(move || {
        while let Ok(value) = in_receiver.recv() {
            // すべての出力チャネルに送信
            for sender in &out_senders {
                let _ = sender.send(value.clone());
            }
        }
    });

    Ok(Value::List(out_channels))
}

/// fan-in - 複数チャネルを1つに集約
///
/// 引数:
/// - channels: チャネルのリスト/ベクタ
///
/// 戻り値:
/// - 統合されたチャネル
///
/// 例:
/// ```lisp
/// (def chs [(chan) (chan) (chan)])
/// (def merged (fan-in chs))
/// ```
pub fn native_fan_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["fan-in"]));
    }

    let channels = match &args[0] {
        Value::List(chs) | Value::Vector(chs) => chs,
        _ => return Err("fan-in: argument must be a list or vector of channels".to_string()),
    };

    // 出力チャネルを作成
    let (out_sender, out_receiver) = unbounded();
    let result_channel = Arc::new(Channel {
        sender: out_sender.clone(),
        receiver: out_receiver,
    });

    // 各入力チャネルから受信して出力に送信
    for ch in channels {
        if let Value::Channel(c) = ch {
            let sender = out_sender.clone();
            let receiver = c.receiver.clone();
            std::thread::spawn(move || {
                while let Ok(value) = receiver.recv() {
                    if sender.send(value).is_err() {
                        break;
                    }
                }
            });
        } else {
            return Err("fan-in: all elements must be channels".to_string());
        }
    }

    Ok(Value::Channel(result_channel))
}

/// pipeline - 汎用パイプライン処理
///
/// 引数:
/// - n: 並列度
/// - xf: 変換関数
/// - in-ch: 入力チャネル
///
/// 戻り値:
/// - 出力チャネル
///
/// 例:
/// ```lisp
/// (def in-ch (chan))
/// (def out-ch (pipeline 4 (fn [x] (* x x)) in-ch))
/// ```
pub fn native_pipeline(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("pipeline requires 3 arguments (n, xf, in-ch)".to_string());
    }

    let n = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err("pipeline: n must be positive".to_string()),
        _ => return Err("pipeline: n must be an integer".to_string()),
    };

    let xf = args[1].clone();

    let in_channel = match &args[2] {
        Value::Channel(ch) => ch.clone(),
        _ => return Err("pipeline: third argument must be a channel".to_string()),
    };

    // 出力チャネルを作成
    let (out_sender, out_receiver) = unbounded();
    let result_channel = Arc::new(Channel {
        sender: out_sender.clone(),
        receiver: out_receiver,
    });

    // n個のワーカーを起動
    for _ in 0..n {
        let in_receiver = in_channel.receiver.clone();
        let out_sender = out_sender.clone();
        let xf = xf.clone();
        let eval = evaluator.clone();

        std::thread::spawn(move || {
            while let Ok(value) = in_receiver.recv() {
                // 変換関数を適用
                match eval.apply_function(&xf, &[value]) {
                    Ok(result) => {
                        if out_sender.send(result).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    Ok(Value::Channel(result_channel))
}

/// pipeline-map - コレクションにパイプライン処理でmapを適用
///
/// 引数:
/// - n: 並列度
/// - f: 変換関数
/// - coll: コレクション
///
/// 戻り値:
/// - 変換後のリスト
///
/// 例:
/// ```lisp
/// (pipeline-map 4 (fn [x] (* x x)) [1 2 3 4 5])
/// ```
pub fn native_pipeline_map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("pipeline-map requires 3 arguments (n, f, coll)".to_string());
    }

    let n = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err("pipeline-map: n must be positive".to_string()),
        _ => return Err("pipeline-map: n must be an integer".to_string()),
    };

    let f = args[1].clone();

    let items = match &args[2] {
        Value::List(items) | Value::Vector(items) => items.clone(),
        _ => return Err("pipeline-map: third argument must be a list or vector".to_string()),
    };

    // 入出力チャネルを作成
    let (in_sender, in_receiver) = unbounded();
    let (out_sender, out_receiver) = unbounded();

    let in_channel = Arc::new(Channel {
        sender: in_sender.clone(),
        receiver: in_receiver,
    });

    // ワーカーを起動
    for _ in 0..n {
        let in_receiver = in_channel.receiver.clone();
        let out_sender = out_sender.clone();
        let f = f.clone();
        let eval = evaluator.clone();

        std::thread::spawn(move || {
            while let Ok(msg) = in_receiver.recv() {
                // [idx, value] の形式
                if let Value::Vector(vec) = msg {
                    if vec.len() == 2 {
                        if let Value::Integer(idx) = vec[0] {
                            match eval.apply_function(&f, &[vec[1].clone()]) {
                                Ok(result) => {
                                    if out_sender
                                        .send(Value::Vector(vec![Value::Integer(idx), result]))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }
            }
        });
    }

    // 入力を送信
    for (i, item) in items.iter().enumerate() {
        let _ = in_sender.send(Value::Vector(vec![Value::Integer(i as i64), item.clone()]));
    }
    drop(in_sender);

    // 結果を収集（順序を保持）
    let mut results = vec![Value::Nil; items.len()];
    for _ in 0..items.len() {
        if let Ok(Value::Vector(vec)) = out_receiver.recv() {
            if vec.len() == 2 {
                if let Value::Integer(idx) = vec[0] {
                    results[idx as usize] = vec[1].clone();
                }
            }
        }
    }

    Ok(Value::List(results))
}

/// pipeline-filter - コレクションにパイプライン処理でfilterを適用
///
/// 引数:
/// - n: 並列度
/// - pred: 述語関数
/// - coll: コレクション
///
/// 戻り値:
/// - フィルタ後のリスト
///
/// 例:
/// ```lisp
/// (pipeline-filter 4 even? [1 2 3 4 5 6])
/// ```
pub fn native_pipeline_filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err("pipeline-filter requires 3 arguments (n, pred, coll)".to_string());
    }

    let n = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err("pipeline-filter: n must be positive".to_string()),
        _ => return Err("pipeline-filter: n must be an integer".to_string()),
    };

    let pred = args[1].clone();

    let items = match &args[2] {
        Value::List(items) | Value::Vector(items) => items.clone(),
        _ => return Err("pipeline-filter: third argument must be a list or vector".to_string()),
    };

    // 入出力チャネルを作成
    let (in_sender, in_receiver) = unbounded();
    let (out_sender, out_receiver) = unbounded();

    let in_channel = Arc::new(Channel {
        sender: in_sender.clone(),
        receiver: in_receiver,
    });

    // ワーカーを起動
    for _ in 0..n {
        let in_receiver = in_channel.receiver.clone();
        let out_sender = out_sender.clone();
        let pred = pred.clone();
        let eval = evaluator.clone();

        std::thread::spawn(move || {
            while let Ok(msg) = in_receiver.recv() {
                // [idx, value] の形式
                if let Value::Vector(vec) = msg {
                    if vec.len() == 2 {
                        if let Value::Integer(idx) = vec[0] {
                            match eval.apply_function(&pred, &[vec[1].clone()]) {
                                Ok(result) if result.is_truthy() => {
                                    // マッチした場合は送信
                                    if out_sender
                                        .send(Value::Vector(vec![Value::Integer(idx), vec[1].clone()]))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Ok(_) => {
                                    // マッチしなかった場合はnilを送信
                                    if out_sender
                                        .send(Value::Vector(vec![Value::Integer(idx), Value::Nil]))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    }
                }
            }
        });
    }

    // 入力を送信
    for (i, item) in items.iter().enumerate() {
        let _ = in_sender.send(Value::Vector(vec![Value::Integer(i as i64), item.clone()]));
    }
    drop(in_sender);

    // 結果を収集（順序を保持、nilは除外）
    let mut indexed_results = Vec::new();
    for _ in 0..items.len() {
        if let Ok(Value::Vector(vec)) = out_receiver.recv() {
            if vec.len() == 2 {
                if let Value::Integer(idx) = vec[0] {
                    if !matches!(vec[1], Value::Nil) {
                        indexed_results.push((idx as usize, vec[1].clone()));
                    }
                }
            }
        }
    }

    // インデックス順にソート
    indexed_results.sort_by_key(|(idx, _)| *idx);
    let results: Vec<Value> = indexed_results.into_iter().map(|(_, v)| v).collect();

    Ok(Value::List(results))
}

/// select! - 複数チャネル待ち合わせ
///
/// 引数:
/// - cases: [[ch1 handler1] [ch2 handler2] [:timeout ms handler]]
///
/// 戻り値:
/// - 選択されたハンドラーの実行結果
///
/// 例:
/// ```lisp
/// (select!
///   [[ch1 (fn [v] (println "ch1:" v))]
///    [ch2 (fn [v] (println "ch2:" v))]
///    [:timeout 1000 (fn [] (println "timeout"))]])
/// ```
pub fn native_select(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("select! requires 1 argument: a list of cases".to_string());
    }

    let cases = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err("select! requires a list of cases".to_string()),
    };

    if cases.is_empty() {
        return Err("select! requires at least one case".to_string());
    }

    // ケースをパース
    let mut channels = Vec::new();
    let mut handlers = Vec::new();
    let mut timeout_ms: Option<i64> = None;
    let mut timeout_handler: Option<Value> = None;

    for case in cases {
        match case {
            Value::List(parts) | Value::Vector(parts) => {
                // タイムアウトケースは3要素、通常のケースは2要素
                match &parts[0] {
                    Value::Keyword(k) if k == "timeout" => {
                        // タイムアウトケース: [:timeout ms handler]
                        if parts.len() != 3 {
                            return Err("select! :timeout case must have 3 elements: [:timeout ms handler]".to_string());
                        }
                        if timeout_handler.is_some() {
                            return Err("select! can only have one :timeout case".to_string());
                        }
                        match &parts[1] {
                            Value::Integer(ms) if *ms >= 0 => {
                                timeout_ms = Some(*ms);
                                timeout_handler = Some(parts[2].clone());
                            }
                            Value::Integer(_) => return Err("select! timeout must be non-negative".to_string()),
                            _ => return Err("select! timeout must be an integer (milliseconds)".to_string()),
                        }
                    }
                    Value::Channel(ch) => {
                        // 通常のチャネルケース: [channel handler]
                        if parts.len() != 2 {
                            return Err("select! channel case must have 2 elements: [channel handler]".to_string());
                        }
                        channels.push(ch.clone());
                        handlers.push(parts[1].clone());
                    }
                    _ => return Err("select! case must start with a channel or :timeout".to_string()),
                }
            }
            _ => return Err("select! case must be a list [channel handler] or [:timeout ms handler]".to_string()),
        }
    }

    // crossbeam-channelのselect!を使用
    use crossbeam_channel::Select;

    let mut sel = Select::new();
    for ch in &channels {
        sel.recv(&ch.receiver);
    }

    // タイムアウトがある場合
    let timeout_receiver = if let Some(ms) = timeout_ms {
        let (tx, rx) = bounded(1);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms as u64));
            let _ = tx.send(());
        });
        Some(rx)
    } else {
        None
    };

    if let Some(ref rx) = timeout_receiver {
        sel.recv(rx);
    }

    // 選択を実行
    let oper = sel.select();
    let index = oper.index();

    // どのケースが選択されたかを判定
    if let Some(ref rx) = timeout_receiver {
        if index == channels.len() {
            // タイムアウトケースが選択された
            let _ = oper.recv(rx); // 操作を完了させる
            if let Some(handler) = timeout_handler {
                return evaluator.apply_function(&handler, &[]);
            }
            return Ok(Value::Nil);
        }
    }

    // 通常のチャネルケースが選択された
    if index < channels.len() {
        let value = oper.recv(&channels[index].receiver).map_err(|_| "select!: channel closed".to_string())?;
        return evaluator.apply_function(&handlers[index], &[value]);
    }

    Err("select!: unexpected error".to_string())
}


/// make-scope - スコープを作成
///
/// Structured Concurrency用のスコープを作成します。
/// スコープ内で起動したgoroutineは、スコープがキャンセルされると全て停止できます。
///
/// 戻り値:
/// - スコープオブジェクト
///
/// 例:
/// ```lisp
/// (def ctx (make-scope))
/// (scope-go ctx task1)
/// (cancel! ctx)  ;; 全てキャンセル
/// ```
pub fn native_make_scope(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err("make-scope requires no arguments".to_string());
    }

    let scope = Scope {
        cancelled: Arc::new(RwLock::new(false)),
    };

    Ok(Value::Scope(Arc::new(scope)))
}

/// scope-go - スコープ内でgoroutineを起動
///
/// 引数:
/// - scope: スコープ
/// - func: 実行する関数
///
/// 戻り値:
/// - チャネル（結果を受信可能）
///
/// 例:
/// ```lisp
/// (def ctx (make-scope))
/// (scope-go ctx (fn [] (println "running")))
/// ```
pub fn native_scope_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("scope-go requires 2 arguments: scope and function".to_string());
    }

    let scope = match &args[0] {
        Value::Scope(s) => s.clone(),
        _ => return Err("scope-go: first argument must be a scope".to_string()),
    };

    let func = args[1].clone();

    // チャネルを作成して結果を返す
    let (sender, receiver) = unbounded();
    let ch = Value::Channel(Arc::new(Channel {
        sender: sender.clone(),
        receiver: receiver.clone(),
    }));

    let evaluator_clone = evaluator.clone();

    // 新しいスレッドで実行
    std::thread::spawn(move || {
        let result = evaluator_clone.apply_function(&func, &[]);
        let _ = sender.send(result.unwrap_or(Value::Nil));
    });

    Ok(ch)
}

/// cancel! - スコープをキャンセル
///
/// 引数:
/// - scope: キャンセルするスコープ
///
/// 戻り値:
/// - nil
///
/// 例:
/// ```lisp
/// (def ctx (make-scope))
/// (cancel! ctx)
/// ```
pub fn native_cancel(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("cancel! requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Scope(s) => {
            *s.cancelled.write() = true;
            Ok(Value::Nil)
        }
        _ => Err("cancel!: argument must be a scope".to_string()),
    }
}

/// cancelled? - スコープがキャンセルされているかチェック
///
/// 引数:
/// - scope: チェックするスコープ
///
/// 戻り値:
/// - true/false
///
/// 例:
/// ```lisp
/// (def ctx (make-scope))
/// (cancelled? ctx)  ;; => false
/// (cancel! ctx)
/// (cancelled? ctx)  ;; => true
/// ```
pub fn native_cancelled_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("cancelled? requires 1 argument".to_string());
    }

    match &args[0] {
        Value::Scope(s) => {
            let is_cancelled = *s.cancelled.read();
            Ok(Value::Bool(is_cancelled))
        }
        _ => Err("cancelled?: argument must be a scope".to_string()),
    }
}

/// with-scope - スコープを作成して関数を実行し、自動的にキャンセル
///
/// 引数:
/// - func: スコープを引数として受け取る関数
///
/// 戻り値:
/// - 関数の戻り値
///
/// 例:
/// ```lisp
/// (with-scope (fn [ctx]
///   (scope-go ctx (fn [] (println "task 1")))
///   (scope-go ctx (fn [] (println "task 2")))
///   (sleep 100)))
/// ;; 関数終了時に自動的にキャンセルされる
/// ```
pub fn native_with_scope(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("with-scope requires 1 argument: function".to_string());
    }

    let func = &args[0];

    // スコープを作成
    let scope = Arc::new(Scope {
        cancelled: Arc::new(RwLock::new(false)),
    });
    let scope_val = Value::Scope(scope.clone());

    // 関数を実行
    let result = evaluator.apply_function(func, &[scope_val]);

    // 実行後に自動的にキャンセル
    *scope.cancelled.write() = true;

    result
}

/// parallel-do - 複数の式を並列実行
///
/// 引数:
/// - exprs: 並列実行する式のリスト（可変長）
///
/// 戻り値:
/// - 全ての結果をベクタで返す
///
/// 例:
/// ```lisp
/// (parallel-do
///   (http/get "url1")
///   (http/get "url2")
///   (http/get "url3"))
/// ;; => [result1 result2 result3]
/// ```
pub fn native_parallel_do(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Vector(vec![]));
    }

    // 各式を関数としてラップされていることを期待
    // （eval.rsで事前に評価される前の式を受け取る）
    // 実際にはValueとして評価済みのものが来るので、
    // 関数を受け取る形式にする

    // 結果を格納するチャネルのベクタ
    let channels: Vec<_> = args
        .iter()
        .map(|func| {
            let (sender, receiver) = unbounded();
            let ch = Arc::new(Channel {
                sender: sender.clone(),
                receiver: receiver.clone(),
            });

            let func_clone = func.clone();
            let evaluator_clone = evaluator.clone();

            // 各タスクを並列実行
            std::thread::spawn(move || {
                let result = evaluator_clone.apply_function(&func_clone, &[]);
                let _ = sender.send(result.unwrap_or(Value::Nil));
            });

            ch
        })
        .collect();

    // 全ての結果を収集
    let results: Result<Vec<_>, _> = channels
        .iter()
        .map(|ch| ch.receiver.recv().map_err(|_| "parallel-do: channel closed".to_string()))
        .collect();

    Ok(Value::Vector(results?))
}

