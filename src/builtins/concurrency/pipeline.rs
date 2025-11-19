//! タスク実行・パイプライン・Select操作

use super::promise::spawn_promise;
use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Value};
use crossbeam_channel::{bounded, unbounded};
use std::sync::Arc;
pub fn native_run(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["go/run"]));
    }

    let expr = args[0].clone();
    let eval = evaluator.clone();

    Ok(spawn_promise(move |sender| {
        let result = match expr {
            Value::Function(ref _f) => eval.apply_function(&expr, &[]),
            _ => Ok(expr.clone()),
        };
        // エラー情報を保持（Railway Oriented Programming）
        let value = match result {
            Ok(v) => v,
            Err(e) => Value::error(e),
        };
        let _ = sender.send(value);
    }))
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
/// ```qi
/// (def in-ch (go/chan))
/// (def out-chs (go/fan-out in-ch 3))  ;; 3つに分岐
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
        Value::Integer(_) => return Err(fmt_msg(MsgKey::MustBePositive, &["fan-out", "n"])),
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["fan-out", "n"])),
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

    Ok(Value::List(out_channels.into()))
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
/// ```qi
/// (def chs [(go/chan) (chan) (chan)])
/// (def merged (go/fan-in chs))
/// ```
pub fn native_fan_in(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["fan-in"]));
    }

    let channels = match &args[0] {
        Value::List(chs) | Value::Vector(chs) => chs,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["fan-in", "argument"])),
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
            return Err(fmt_msg(MsgKey::AllElementsMustBe, &["fan-in", "channels"]));
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
/// ```qi
/// (def in-ch (go/chan))
/// (def out-ch (go/pipeline 4 (fn [x] (* x x)) in-ch))
/// ```
pub fn native_pipeline(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["pipeline", "3", "(n, xf, in-ch)"],
        ));
    }

    let n = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err(fmt_msg(MsgKey::MustBePositive, &["pipeline", "n"])),
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["pipeline", "n"])),
    };

    let xf = args[1].clone();

    let in_channel = match &args[2] {
        Value::Channel(ch) => ch.clone(),
        _ => return Err(fmt_msg(MsgKey::ThirdArgMustBe, &["pipeline", "a channel"])),
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
/// ```qi
/// (go/pipeline-map 4 (fn [x] (* x x)) [1 2 3 4 5])
/// ```
pub fn native_pipeline_map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["pipeline-map", "3", "(n, f, coll)"],
        ));
    }

    let n = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => return Err(fmt_msg(MsgKey::MustBePositive, &["pipeline-map", "n"])),
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["pipeline-map", "n"])),
    };

    let f = args[1].clone();

    let items = match &args[2] {
        Value::List(items) | Value::Vector(items) => items.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["pipeline-map (3rd arg)", "third argument"],
            ))
        }
    };

    // 入出力チャネルを作成
    let (in_sender, in_receiver) = unbounded();
    let (out_sender, out_receiver) = unbounded();

    let in_channel = Arc::new(Channel {
        sender: in_sender.clone(),
        receiver: in_receiver,
    });

    // 並列mapの実装（順序保持 + Arc最適化）
    //
    // Arc最適化:
    //   - 関数fとevaluatorを事前にArcでラップ
    //   - n個のワーカー起動時にArc::cloneのみ（参照カウンタのインクリメント）
    //   - 各ワーカーでのcloneコストを削減（特にevaluatorは大きい）
    //
    // 順序保持:
    //   - 各要素を [idx, value] の形式で送信
    //   - ワーカーは [idx, result] の形式で返す
    //   - 受信側でインデックス順にソートして元の順序を復元
    let f = Arc::new(f);
    let evaluator = Arc::new(evaluator.clone());

    // ワーカーを起動
    for _ in 0..n {
        let in_receiver = in_channel.receiver.clone();
        let out_sender = out_sender.clone();
        let f = Arc::clone(&f);
        let evaluator = Arc::clone(&evaluator);

        std::thread::spawn(move || {
            while let Ok(msg) = in_receiver.recv() {
                // [idx, value] の形式でメッセージを受信
                if let Value::Vector(vec) = msg {
                    if vec.len() == 2 {
                        if let Value::Integer(idx) = vec[0] {
                            match evaluator.apply_function(&f, &[vec[1].clone()]) {
                                Ok(result) => {
                                    // [idx, result] の形式で結果を返す
                                    if out_sender
                                        .send(Value::Vector(
                                            vec![Value::Integer(idx), result].into(),
                                        ))
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

    // 入力を送信（インデックス付き）
    for (i, item) in items.iter().enumerate() {
        let _ = in_sender.send(Value::Vector(
            vec![Value::Integer(i as i64), item.clone()].into(),
        ));
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

    Ok(Value::List(results.into()))
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
/// ```qi
/// (go/pipeline-filter 4 even? [1 2 3 4 5 6])
/// ```
pub fn native_pipeline_filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["pipeline-filter", "3", "(n, pred, coll)"],
        ));
    }

    let n = match &args[0] {
        Value::Integer(n) if *n > 0 => *n as usize,
        Value::Integer(_) => {
            return Err(fmt_msg(MsgKey::MustBePositive, &["pipeline-filter", "n"]))
        }
        _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["pipeline-filter", "n"])),
    };

    let pred = args[1].clone();

    let items = match &args[2] {
        Value::List(items) | Value::Vector(items) => items.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeListOrVector,
                &["pipeline-filter (3rd arg)", "third argument"],
            ))
        }
    };

    // 入出力チャネルを作成
    let (in_sender, in_receiver) = unbounded();
    let (out_sender, out_receiver) = unbounded();

    let in_channel = Arc::new(Channel {
        sender: in_sender.clone(),
        receiver: in_receiver,
    });

    // Arcで共有することでワーカー起動時のcloneコストを削減
    let pred = Arc::new(pred);
    let evaluator = Arc::new(evaluator.clone());

    // ワーカーを起動
    for _ in 0..n {
        let in_receiver = in_channel.receiver.clone();
        let out_sender = out_sender.clone();
        let pred = Arc::clone(&pred);
        let evaluator = Arc::clone(&evaluator);

        std::thread::spawn(move || {
            while let Ok(msg) = in_receiver.recv() {
                // [idx, value] の形式
                if let Value::Vector(vec) = msg {
                    if vec.len() == 2 {
                        if let Value::Integer(idx) = vec[0] {
                            match evaluator.apply_function(&pred, &[vec[1].clone()]) {
                                Ok(result) if result.is_truthy() => {
                                    // マッチした場合は送信
                                    if out_sender
                                        .send(Value::Vector(
                                            vec![Value::Integer(idx), vec[1].clone()].into(),
                                        ))
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                                Ok(_) => {
                                    // マッチしなかった場合はnilを送信
                                    if out_sender
                                        .send(Value::Vector(
                                            vec![Value::Integer(idx), Value::Nil].into(),
                                        ))
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
        let _ = in_sender.send(Value::Vector(
            vec![Value::Integer(i as i64), item.clone()].into(),
        ));
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

    Ok(Value::List(results.into()))
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
/// ```qi
/// (select!
///   [[ch1 (fn [v] (println "ch1:" v))]
///    [ch2 (fn [v] (println "ch2:" v))]
///    [:timeout 1000 (fn [] (println "timeout"))]])
/// ```
///
/// # select!の実装（crossbeam-channel::Selectを使用）
///
/// ## ケース解析
/// - 通常ケース: `[channel handler]` → チャネルから受信したらハンドラを実行
/// - タイムアウトケース: `[:timeout ms handler]` → 指定時間後にハンドラを実行
///
/// ## Selectの仕組み
/// 1. `sel.recv()`で各チャネルを登録
/// 2. タイムアウトがあれば別スレッドでスリープ後にダミーチャネルに送信
/// 3. `sel.select()`で最初に準備ができたチャネルを選択
/// 4. `oper.index()`で選ばれたチャネルのインデックスを取得
///    - `0..channels.len()`: 通常ケース
///    - `channels.len()`: タイムアウトケース
///
/// ## タイムアウトの実装
/// - 別スレッドで指定時間スリープ後、ダミーチャネルに送信
/// - Selectに登録されるインデックスは`channels.len()`（最後）
/// - タイムアウトケースは1つのみ許可
pub fn native_select(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["select!", "1", "(a list of cases)"],
        ));
    }

    let cases = match &args[0] {
        Value::List(items) | Value::Vector(items) => items,
        _ => return Err(fmt_msg(MsgKey::SelectNeedsList, &["select!"])),
    };

    if cases.is_empty() {
        return Err(fmt_msg(MsgKey::SelectNeedsAtLeastOne, &["select!"]));
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
                    Value::Keyword(k) if &**k == "timeout" => {
                        // タイムアウトケース: [:timeout ms handler]
                        if parts.len() != 3 {
                            return Err(fmt_msg(MsgKey::SelectTimeoutCase, &["select!"]));
                        }
                        if timeout_handler.is_some() {
                            return Err(fmt_msg(MsgKey::SelectOnlyOneTimeout, &["select!"]));
                        }
                        match &parts[1] {
                            Value::Integer(ms) if *ms >= 0 => {
                                timeout_ms = Some(*ms);
                                timeout_handler = Some(parts[2].clone());
                            }
                            Value::Integer(_) => {
                                return Err(fmt_msg(
                                    MsgKey::MustBeNonNegative,
                                    &["select!", "timeout"],
                                ))
                            }
                            _ => return Err(fmt_msg(MsgKey::TimeoutMustBeMs, &["select!"])),
                        }
                    }
                    Value::Channel(ch) => {
                        // 通常のチャネルケース: [channel handler]
                        if parts.len() != 2 {
                            return Err(fmt_msg(MsgKey::SelectChannelCase, &["select!"]));
                        }
                        channels.push(ch.clone());
                        handlers.push(parts[1].clone());
                    }
                    _ => return Err(fmt_msg(MsgKey::SelectCaseMustStart, &["select!"])),
                }
            }
            _ => return Err(fmt_msg(MsgKey::SelectCaseMustBe, &["select!"])),
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

    // 選択を実行（ブロッキング）
    let oper = sel.select();
    let index = oper.index();

    // インデックスでどのケースが選択されたかを判定
    // - 0..channels.len(): 通常のチャネルケース
    // - channels.len(): タイムアウトケース
    if let Some(ref rx) = timeout_receiver {
        if index == channels.len() {
            // タイムアウトケースが選択された
            let _ = oper.recv(rx); // 操作を完了させる（ダミー受信）
            if let Some(handler) = timeout_handler {
                return evaluator.apply_function(&handler, &[]);
            }
            return Ok(Value::Nil);
        }
    }

    // 通常のチャネルケースが選択された
    if index < channels.len() {
        let value = oper
            .recv(&channels[index].receiver)
            .map_err(|_| fmt_msg(MsgKey::ChannelClosed, &["select!"]))?;
        return evaluator.apply_function(&handlers[index], &[value]);
    }

    Err(fmt_msg(MsgKey::UnexpectedError, &["select!"]))
}
