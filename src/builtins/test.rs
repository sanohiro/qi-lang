//! テストフレームワーク

use crate::eval::Evaluator;
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
    if args.len() != 2 {
        return Err("test/assert-eq: requires 2 arguments (expected actual)".to_string());
    }

    let expected = &args[0];
    let actual = &args[1];

    if expected == actual {
        Ok(Value::Bool(true))
    } else {
        Err(format!(
            "Assertion failed:\n  Expected: {}\n  Actual:   {}",
            expected, actual
        ))
    }
}

/// test/assert - 値が真であることをアサート
pub fn native_assert(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("test/assert: requires 1 argument".to_string());
    }

    if args[0].is_truthy() {
        Ok(Value::Bool(true))
    } else {
        Err(format!("Assertion failed: expected truthy value, got {}", args[0]))
    }
}

/// test/assert-not - 値が偽であることをアサート
pub fn native_assert_not(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("test/assert-not: requires 1 argument".to_string());
    }

    if !args[0].is_truthy() {
        Ok(Value::Bool(true))
    } else {
        Err(format!("Assertion failed: expected falsy value, got {}", args[0]))
    }
}

/// test/assert-throws - 式が例外を投げることをアサート
pub fn native_assert_throws(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("test/assert-throws: requires 1 argument (function)".to_string());
    }

    // 関数を実行
    match &args[0] {
        Value::Function(func) => {
            let result = evaluator.eval(&func.body);
            if result.is_err() {
                Ok(Value::Bool(true))
            } else {
                Err("Assertion failed: expected exception but none was thrown".to_string())
            }
        }
        _ => Err("test/assert-throws: argument must be a function".to_string()),
    }
}

/// test/run - テストを実行して結果を記録
pub fn native_test_run(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("test/run: requires 2 arguments (name body)".to_string());
    }

    let name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("test/run: first argument must be a string".to_string()),
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
            return Err("test/run: second argument must be a function".to_string());
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
        println!("No tests to run");
        return Ok(Value::Integer(0));
    }

    let total = registry.len();
    let passed = registry.iter().filter(|t| t.passed).count();
    let failed = total - passed;

    println!("\nTest Results:");
    println!("=============");

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

    println!("\n{} tests, {} passed, {} failed", total, passed, failed);

    if failed > 0 {
        Err("Some tests failed".to_string())
    } else {
        Ok(Value::Integer(passed as i64))
    }
}

/// test/clear - テスト結果をクリア
pub fn native_test_clear(_args: &[Value]) -> Result<Value, String> {
    test_registry().lock().clear();
    Ok(Value::Nil)
}
