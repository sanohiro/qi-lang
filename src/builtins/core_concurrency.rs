//! Core並行処理関数
//!
//! 並行処理基本（5個）: go, chan, send!, recv!, close!

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Value};
use crossbeam_channel::{bounded, unbounded};
use std::sync::Arc;

/// chan - チャネルを作成
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
                    Value::Keyword(k) if k == "timeout" => {
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

/// close! - チャネルをクローズ
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
pub fn native_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["go"]));
    }

    // 結果を返すチャネルを作成
    let (sender, receiver) = bounded(1);
    let result_channel = Arc::new(Channel {
        sender: sender.clone(),
        receiver,
    });

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
        let _ = sender.send(result.unwrap_or_else(Value::String));
    });

    Ok(Value::Channel(result_channel))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
///
/// 注意: goはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("chan", native_chan),
    ("send!", native_send),
    ("recv!", native_recv),
    ("close!", native_close),
];
