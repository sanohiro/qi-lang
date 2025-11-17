//! Promise/非同期操作

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Value};
use crossbeam_channel::bounded;
use std::sync::Arc;
/// Promiseチャネルを作成（capacity=1のboundedチャネル）
pub(super) fn create_promise_channel() -> Arc<Channel> {
    let (sender, receiver) = bounded(1);
    Arc::new(Channel { sender, receiver })
}

/// スレッドを起動してPromiseを返す汎用ヘルパー
///
/// # Promise風の非同期実行
///
/// ## 設計
/// - capacity=1のboundedチャネルを作成（単一の結果を保持）
/// - スレッドを起動してクロージャ内でsenderに結果を送信
/// - チャネルを即座に返し、呼び出し側はreceiverで結果を待機可能
///
/// ## なぜsender.clone()が必要か
/// - `channel`はArc<Channel>で共有されるため、senderを別途cloneして
///   スレッドに移動させる必要がある（channelの所有権は返す）
/// - スレッドクロージャは`move`なので、senderの所有権を移動
///
/// ## クロージャ内でチャネルのsenderに結果を送信する処理を実行する
pub(super) fn spawn_promise<F>(f: F) -> Value
where
    F: FnOnce(crossbeam_channel::Sender<Value>) + Send + 'static,
{
    let channel = create_promise_channel();
    let sender = channel.sender.clone();

    std::thread::spawn(move || {
        f(sender);
    });

    Value::Channel(channel)
}
pub fn native_await(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["await"]));
    }

    match &args[0] {
        Value::Channel(ch) => ch
            .receiver
            .recv()
            .map_err(|_| fmt_msg(MsgKey::ChannelClosed, &["await"])),
        _ => Err(fmt_msg(MsgKey::MustBePromise, &["await", "argument"])),
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
/// ```qi
/// (def p (go/run (fn [] 10)))
/// (def p2 (go/then p (fn [x] (* x 2))))
/// (go/await p2)  ;; => 20
/// ```
pub fn native_then(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["then", "2", "(promise, fn)"],
        ));
    }

    let promise = match &args[0] {
        Value::Channel(ch) => ch.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBePromise,
                &["then (1st arg)", "first argument"],
            ))
        }
    };

    let f = args[1].clone();
    let eval = evaluator.clone();

    Ok(spawn_promise(move |sender| {
        if let Ok(value) = promise.receiver.recv() {
            let result = eval.apply_function(&f, &[value]);
            // エラー情報を保持（Railway Oriented Programming）
            let value = match result {
                Ok(v) => v,
                Err(e) => Value::error(e),
            };
            let _ = sender.send(value);
        }
    }))
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
/// ```qi
/// (def p (go/run (fn [] (error "oops"))))
/// (def p2 (go/catch p (fn [e] (println "Error:" e))))
/// ```
pub fn native_catch(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["catch", "2", "(promise, handler)"],
        ));
    }

    let promise = match &args[0] {
        Value::Channel(ch) => ch.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBePromise,
                &["catch (1st arg)", "first argument"],
            ))
        }
    };

    let handler = args[1].clone();
    let eval = evaluator.clone();

    Ok(spawn_promise(move |sender| match promise.receiver.recv() {
        Ok(value) => {
            let _ = sender.send(value);
        }
        Err(_) => {
            let result =
                eval.apply_function(&handler, &[Value::String("channel closed".to_string())]);
            // エラー情報を保持（Railway Oriented Programming）
            let value = match result {
                Ok(v) => v,
                Err(e) => Value::error(e),
            };
            let _ = sender.send(value);
        }
    }))
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
/// ```qi
/// (def promises [(go/run (fn [] 1)) (go/run (fn [] 2)) (go/run (fn [] 3))])
/// (def result (go/await (go/all promises)))  ;; => [1 2 3]
/// ```
pub fn native_all(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["all", "1", "(list of promises)"],
        ));
    }

    let promises = match &args[0] {
        Value::List(ps) | Value::Vector(ps) => ps.clone(),
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["all", "argument"])),
    };

    Ok(spawn_promise(move |sender| {
        let mut results = Vec::new();
        for promise in promises {
            if let Value::Channel(ch) = promise {
                match ch.receiver.recv() {
                    Ok(value) => results.push(value),
                    Err(_) => {
                        let _ = sender.send(Value::String(fmt_msg(MsgKey::PromiseFailed, &[])));
                        return;
                    }
                }
            } else {
                let _ = sender.send(Value::String(fmt_msg(MsgKey::NotAPromise, &[])));
                return;
            }
        }
        let _ = sender.send(Value::List(results.into()));
    }))
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
/// ```qi
/// (def promises [(go/run (fn [] (sleep 100) 1)) (go/run (fn [] 2))])
/// (def result (go/await (go/race promises)))  ;; => 2 (最速)
/// ```
pub fn native_race(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["race", "1", "(list of promises)"],
        ));
    }

    let promises = match &args[0] {
        Value::List(ps) | Value::Vector(ps) => ps,
        _ => return Err(fmt_msg(MsgKey::MustBeListOrVector, &["race", "argument"])),
    };

    let result_channel = create_promise_channel();
    let sender = result_channel.sender.clone();

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
            return Err(fmt_msg(
                MsgKey::AllElementsMustBe,
                &["race", "promises (channels)"],
            ));
        }
    }

    Ok(Value::Channel(result_channel))
}
