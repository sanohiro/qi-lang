//! Flow制御関数
//!
//! パイプラインの分岐・合流をサポートする関数

use crate::eval::Evaluator;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

/// branch - 条件分岐
///
/// 使い方:
/// ```lisp
/// (data |> (branch
///   [condition1 handler1]
///   [condition2 handler2]
///   [:else default-handler]))
/// ```
///
/// 各分岐は [condition handler] のベクタ
/// - condition が関数なら、データを渡して評価
/// - condition が :else なら、常に true
/// - 最初に true になった handler を実行
pub fn native_branch(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["branch", "1"]));
    }

    // パイプライン対応: 最後の引数がデータ値
    let value = &args[args.len() - 1];

    // 分岐がない場合は、そのまま値を返す
    if args.len() == 1 {
        return Ok(value.clone());
    }

    // 各分岐を評価（最後の引数を除く）
    for branch_arg in &args[..args.len() - 1] {
        match branch_arg {
            Value::Vector(parts) if parts.len() == 2 => {
                let condition = &parts[0];
                let handler = &parts[1];

                // 条件を評価
                let is_match = match condition {
                    // :else は常に true
                    Value::Keyword(k) if k == "else" => true,

                    // 関数の場合、値を渡して評価
                    Value::Function(_) | Value::NativeFunc(_) => {
                        let cond_result = evaluator.apply_function(condition, std::slice::from_ref(value))?;
                        is_truthy(&cond_result)
                    }

                    // その他はエラー
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::TypeOnly,
                            &["branch condition", "function or :else"],
                        ))
                    }
                };

                // マッチしたら handler を実行
                if is_match {
                    return evaluator.apply_function(handler, std::slice::from_ref(value));
                }
            }
            _ => {
                return Err(fmt_msg(
                    MsgKey::TypeOnly,
                    &["branch", "vectors of [condition handler]"],
                ))
            }
        }
    }

    // どの条件にもマッチしなかった場合は、元の値を返す
    Ok(value.clone())
}

/// 値が truthy かどうか判定
fn is_truthy(value: &Value) -> bool {
    !matches!(value, Value::Nil | Value::Bool(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval_str(s: &str) -> Result<Value, String> {
        crate::i18n::init(); // i18nシステムを初期化
        let evaluator = Evaluator::new();
        let mut parser = Parser::new(s)?;
        let exprs = parser.parse_all()?;
        let mut result = Value::Nil;
        for expr in exprs {
            result = evaluator.eval(&expr)?;
        }
        Ok(result)
    }

    #[test]
    fn test_branch_basic() {
        let code = r#"
        (defn positive? [x] (> x 0))
        (defn negative? [x] (< x 0))

        (10 |> (branch
          [positive? (fn [x] "positive")]
          [negative? (fn [x] "negative")]
          [:else (fn [x] "zero")]))
        "#;

        assert_eq!(eval_str(code).unwrap(), Value::String("positive".to_string()));
    }

    #[test]
    fn test_branch_negative() {
        let code = r#"
        (defn positive? [x] (> x 0))
        (defn negative? [x] (< x 0))

        (-5 |> (branch
          [positive? (fn [x] "positive")]
          [negative? (fn [x] "negative")]
          [:else (fn [x] "zero")]))
        "#;

        assert_eq!(eval_str(code).unwrap(), Value::String("negative".to_string()));
    }

    #[test]
    fn test_branch_else() {
        let code = r#"
        (defn positive? [x] (> x 0))
        (defn negative? [x] (< x 0))

        (0 |> (branch
          [positive? (fn [x] "positive")]
          [negative? (fn [x] "negative")]
          [:else (fn [x] "zero")]))
        "#;

        assert_eq!(eval_str(code).unwrap(), Value::String("zero".to_string()));
    }

    #[test]
    fn test_branch_with_pipeline() {
        let code = r#"
        (defn even? [x] (= (% x 2) 0))
        (defn odd? [x] (!= (% x 2) 0))

        (10 |> (branch
          [even? (fn [x] (x |> inc |> (* 2)))]
          [odd? (fn [x] (x |> dec |> (* 3)))]
          [:else (fn [x] 0)]))
        "#;

        // 10 is even -> inc -> 11 -> * 2 -> 22
        assert_eq!(eval_str(code).unwrap(), Value::Integer(22));
    }

    #[test]
    fn test_branch_first_match() {
        let code = r#"
        (defn always-true [x] true)

        (5 |> (branch
          [always-true (fn [x] "first")]
          [always-true (fn [x] "second")]
          [:else (fn [x] "third")]))
        "#;

        // 最初にマッチした分岐が実行される
        assert_eq!(eval_str(code).unwrap(), Value::String("first".to_string()));
    }
}
