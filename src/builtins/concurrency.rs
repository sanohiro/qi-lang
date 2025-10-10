//! 並行処理関数（go/chan）

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Value};
use crossbeam_channel::{bounded, unbounded};
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
///
/// 戻り値:
/// - 受信した値（チャネルがクローズされていればnil）
///
/// 例:
/// ```lisp
/// (def ch (chan))
/// (recv! ch)  ;; ブロックして待つ
/// ```
pub fn native_recv(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["recv!"]));
    }

    match &args[0] {
        Value::Channel(ch) => ch
            .receiver
            .recv()
            .map_err(|_| "recv!: channel is closed".to_string()),
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
