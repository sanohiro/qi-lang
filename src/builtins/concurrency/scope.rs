//! スコープ・キャンセル・並列実行

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Channel, Scope, Value};
use crossbeam_channel::unbounded;
use parking_lot::RwLock;
use std::sync::Arc;
/// キャンセル可能なスコープを作成
///
/// 引数: なし
///
/// 戻り値:
/// - 新しいスコープオブジェクト
///
/// 例:
/// ```qi
/// (def ctx (go/make-scope))
/// (go/scope-go ctx (fn [] (println "task")))
/// (go/cancel! ctx)  ;; スコープをキャンセル
/// ```
pub fn native_make_scope(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["make-scope"]));
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
/// ```qi
/// (def ctx (go/make-scope))
/// (go/scope-go ctx (fn [] (println "running")))
/// ```
pub fn native_scope_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["scope-go", "2", "(scope, function)"],
        ));
    }

    let _scope = match &args[0] {
        Value::Scope(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeScope,
                &["scope-go (1st arg)", "first argument"],
            ))
        }
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
        // エラー情報を保持（Railway Oriented Programming）
        let value = match result {
            Ok(v) => v,
            Err(e) => Value::error(e),
        };
        let _ = sender.send(value);
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
/// ```qi
/// (def ctx (go/make-scope))
/// (go/cancel! ctx)
/// ```
pub fn native_cancel(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["cancel!"]));
    }

    match &args[0] {
        Value::Scope(s) => {
            *s.cancelled.write() = true;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::MustBeScope, &["cancel!", "argument"])),
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
/// ```qi
/// (def ctx (go/make-scope))
/// (go/cancelled? ctx)  ;; => false
/// (go/cancel! ctx)
/// (go/cancelled? ctx)  ;; => true
/// ```
pub fn native_cancelled_q(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["cancelled?"]));
    }

    match &args[0] {
        Value::Scope(s) => {
            let is_cancelled = *s.cancelled.read();
            Ok(Value::Bool(is_cancelled))
        }
        _ => Err(fmt_msg(MsgKey::MustBeScope, &["cancelled?", "argument"])),
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
/// ```qi
/// (go/with-scope (fn [ctx]
///   (go/scope-go ctx (fn [] (println "task 1")))
///   (go/scope-go ctx (fn [] (println "task 2")))
///   (sleep 100)))
/// ;; 関数終了時に自動的にキャンセルされる
/// ```
pub fn native_with_scope(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["with-scope", "1", "(function)"],
        ));
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
/// ```qi
/// (go/parallel-do
///   (http/get "url1")
///   (http/get "url2")
///   (http/get "url3"))
/// ;; => [result1 result2 result3]
/// ```
pub fn native_parallel_do(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.is_empty() {
        return Ok(Value::Vector(vec![].into()));
    }

    // 各式を関数としてラップされていることを期待
    // （eval.rsで事前に評価される前の式を受け取る）
    // 実際にはValueとして評価済みのものが来るので、
    // 関数を受け取る形式にする

    // Arcで共有することでcloneコストを削減
    let evaluator = Arc::new(evaluator.clone());

    // 結果を格納するチャネルのベクタ
    let channels: Vec<_> = args
        .iter()
        .map(|func| {
            let (sender, receiver) = unbounded();
            let ch = Arc::new(Channel {
                sender: sender.clone(),
                receiver: receiver.clone(),
            });

            let func = Arc::new(func.clone());
            let evaluator = Arc::clone(&evaluator);

            // 各タスクを並列実行
            std::thread::spawn(move || {
                let result = evaluator.apply_function(&func, &[]);
                // エラー情報を保持（Railway Oriented Programming）
                let value = match result {
                    Ok(v) => v,
                    Err(e) => Value::error(e),
                };
                let _ = sender.send(value);
            });

            ch
        })
        .collect();

    // 全ての結果を収集
    let results: Result<Vec<_>, _> = channels
        .iter()
        .map(|ch| {
            ch.receiver
                .recv()
                .map_err(|_| fmt_msg(MsgKey::ChannelClosed, &["parallel-do"]))
        })
        .collect();

    Ok(Value::Vector(results?.into()))
}
