//! チャネル基本操作

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Value};
use crossbeam_channel::{bounded, unbounded};
use std::sync::Arc;
pub fn native_chan(args: &[Value]) -> Result<Value, String> {
    let capacity = if args.is_empty() {
        None
    } else if args.len() == 1 {
        match &args[0] {
            Value::Integer(n) if *n >= 0 => Some(*n as usize),
            Value::Integer(_) => {
                return Err(fmt_msg(MsgKey::MustBeNonNegative, &["chan", "capacity"]))
            }
            _ => return Err(fmt_msg(MsgKey::MustBeInteger, &["chan", "capacity"])),
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
/// ```qi
/// (def ch (go/chan))
/// (go/send! ch 42)
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
                .map_err(|_| fmt_msg(MsgKey::ChannelClosed, &["send!"]))?;
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
/// ```qi
/// (def ch (go/chan))
/// (go/recv! ch)                    ;; ブロックして待つ
/// (go/recv! ch :timeout 1000)      ;; 最大1秒待つ
/// ```
pub fn native_recv(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 3 {
        return Err(fmt_msg(MsgKey::RecvArgs, &["recv!"]));
    }

    match &args[0] {
        Value::Channel(ch) => {
            // タイムアウト指定があるか確認
            if args.len() == 3 {
                // :timeout キーワードを確認
                match &args[1] {
                    Value::Keyword(k) if &**k == "timeout" => {
                        // タイムアウト値（ミリ秒）を取得
                        match &args[2] {
                            Value::Integer(ms) if *ms >= 0 => {
                                let timeout = std::time::Duration::from_millis(*ms as u64);
                                Ok(ch.receiver.recv_timeout(timeout).unwrap_or(Value::Nil))
                            }
                            Value::Integer(_) => {
                                Err(fmt_msg(MsgKey::MustBeNonNegative, &["recv!", "timeout"]))
                            }
                            _ => Err(fmt_msg(MsgKey::TimeoutMustBeMs, &["recv!"])),
                        }
                    }
                    _ => Err(fmt_msg(MsgKey::ExpectedKeyword, &["recv!", ":timeout"])),
                }
            } else {
                // タイムアウトなし（通常のブロッキング受信）
                ch.receiver
                    .recv()
                    .map_err(|_| fmt_msg(MsgKey::ChannelClosed, &["recv!"]))
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
/// ```qi
/// (def ch (go/chan))
/// (go/try-recv! ch)  ;; すぐに返る（nilまたは値）
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
/// ```qi
/// (def ch (go/chan))
/// (go/close! ch)
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
