//! テストフレームワーク

use crate::check_args;
use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, fmt_ui_msg, ui_msg, MsgKey, UiMsg};
use crate::value::Value;
use parking_lot::Mutex;
use std::sync::OnceLock;

/// テスト結果
#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    passed: bool,
    message: Option<String>,
}

/// グローバルテストレジストリ
fn test_registry() -> &'static Mutex<Vec<TestResult>> {
    static REGISTRY: OnceLock<Mutex<Vec<TestResult>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// test/assert-eq - 2つの値が等しいことをアサート
pub fn native_assert_eq(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "test/assert-eq");

    let expected = &args[0];
    let actual = &args[1];

    if expected == actual {
        Ok(Value::Bool(true))
    } else {
        Err(fmt_ui_msg(
            UiMsg::TestAssertEqFailed,
            &[&expected.to_string(), &actual.to_string()],
        ))
    }
}

/// test/assert - 値が真であることをアサート
pub fn native_assert(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "test/assert");

    if args[0].is_truthy() {
        Ok(Value::Bool(true))
    } else {
        Err(fmt_ui_msg(
            UiMsg::TestAssertTruthyFailed,
            &[&args[0].to_string()],
        ))
    }
}

/// test/assert-not - 値が偽であることをアサート
pub fn native_assert_not(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "test/assert-not");

    if !args[0].is_truthy() {
        Ok(Value::Bool(true))
    } else {
        Err(fmt_ui_msg(
            UiMsg::TestAssertFalsyFailed,
            &[&args[0].to_string()],
        ))
    }
}

/// test/assert-throws - 式が例外を投げることをアサート
pub fn native_assert_throws(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    check_args!(args, 1, "test/assert-throws");

    // 関数を実行
    match &args[0] {
        Value::Function(func) => {
            let result = evaluator.eval(&func.body);
            if result.is_err() {
                Ok(Value::Bool(true))
            } else {
                Err(fmt_msg(MsgKey::AssertExpectedException, &[]))
            }
        }
        _ => Err(fmt_msg(
            MsgKey::ArgMustBeType,
            &["test/assert-throws", "a function"],
        )),
    }
}

/// test/run - テストを実行して結果を記録
pub fn native_test_run(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    check_args!(args, 2, "test/run");

    let name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["test/run", "a string"])),
    };

    let body = &args[1];

    // テストを実行
    let result = match body {
        Value::Function(func) => {
            let res = evaluator.eval(&func.body);
            match res {
                Ok(_) => TestResult {
                    name: name.clone(),
                    passed: true,
                    message: None,
                },
                Err(e) => TestResult {
                    name: name.clone(),
                    passed: false,
                    message: Some(e),
                },
            }
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::SecondArgMustBe,
                &["test/run", "a function"],
            ));
        }
    };

    // 結果を記録
    test_registry().lock().push(result.clone());

    Ok(Value::Bool(result.passed))
}

/// test/run-all - 全テストを実行して結果を表示
pub fn native_run_all(_args: &[Value]) -> Result<Value, String> {
    let registry = test_registry().lock();

    if registry.is_empty() {
        println!("{}", ui_msg(UiMsg::TestNoTests));
        return Ok(Value::Integer(0));
    }

    let total = registry.len();
    let passed = registry.iter().filter(|t| t.passed).count();
    let failed = total - passed;

    println!("\n{}", ui_msg(UiMsg::TestResults));
    println!("{}", ui_msg(UiMsg::TestResultsSeparator));

    for test in registry.iter() {
        if test.passed {
            println!("  ✓ {}", test.name);
        } else {
            println!("  ✗ {}", test.name);
            if let Some(msg) = &test.message {
                println!("    {}", msg);
            }
        }
    }

    println!(
        "\n{}",
        fmt_ui_msg(
            UiMsg::TestSummary,
            &[&total.to_string(), &passed.to_string(), &failed.to_string()],
        )
    );

    if failed > 0 {
        Err(fmt_msg(MsgKey::TestsFailed, &[]))
    } else {
        Ok(Value::Integer(passed as i64))
    }
}

/// test/clear - テスト結果をクリア
pub fn native_test_clear(_args: &[Value]) -> Result<Value, String> {
    test_registry().lock().clear();
    Ok(Value::Nil)
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category test
/// @qi-doc:functions assert-eq, assert-ne, assert-true, assert-false, assert-nil, run, assert-throws, summary, clear
///
/// 注意: test/run, test/assert-throwsはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("test/assert-eq", native_assert_eq),
    ("test/assert", native_assert),
    ("test/assert-not", native_assert_not),
    ("test/run-all", native_run_all),
    ("test/clear", native_test_clear),
];
