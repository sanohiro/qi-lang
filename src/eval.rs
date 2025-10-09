use crate::value::{Env, Expr, Function, MatchArm, NativeFunc, Pattern, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Evaluator {
    global_env: Rc<RefCell<Env>>,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = Env::new();

        // 基本的な組み込み関数を登録
        env.set(
            "+".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "+".to_string(),
                func: native_add,
            }),
        );
        env.set(
            "-".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "-".to_string(),
                func: native_sub,
            }),
        );
        env.set(
            "*".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "*".to_string(),
                func: native_mul,
            }),
        );
        env.set(
            "/".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "/".to_string(),
                func: native_div,
            }),
        );
        env.set(
            "=".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "=".to_string(),
                func: native_eq,
            }),
        );
        env.set(
            "<".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "<".to_string(),
                func: native_lt,
            }),
        );
        env.set(
            ">".to_string(),
            Value::NativeFunc(NativeFunc {
                name: ">".to_string(),
                func: native_gt,
            }),
        );
        env.set(
            "list".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "list".to_string(),
                func: native_list,
            }),
        );
        env.set(
            "first".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "first".to_string(),
                func: native_first,
            }),
        );
        env.set(
            "rest".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "rest".to_string(),
                func: native_rest,
            }),
        );
        env.set(
            "len".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "len".to_string(),
                func: native_len,
            }),
        );
        env.set(
            "print".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "print".to_string(),
                func: native_print,
            }),
        );

        Evaluator {
            global_env: Rc::new(RefCell::new(env)),
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        self.eval_with_env(expr, self.global_env.clone())
    }

    fn eval_with_env(&mut self, expr: &Expr, env: Rc<RefCell<Env>>) -> Result<Value, String> {
        match expr {
            Expr::Nil => Ok(Value::Nil),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),

            Expr::Symbol(name) => env
                .borrow()
                .get(name)
                .ok_or_else(|| format!("未定義の変数: {}", name)),

            Expr::List(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                Ok(Value::List(values?))
            }

            Expr::Vector(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                Ok(Value::Vector(values?))
            }

            Expr::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match self.eval_with_env(k, env.clone())? {
                        Value::Keyword(k) => k,
                        Value::String(s) => s,
                        Value::Symbol(s) => s,
                        _ => return Err("マップのキーは文字列またはキーワードが必要です".to_string()),
                    };
                    let value = self.eval_with_env(v, env.clone())?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
            }

            Expr::Def(name, value) => {
                let val = self.eval_with_env(value, env.clone())?;
                // グローバル環境に定義
                self.global_env.borrow_mut().set(name.clone(), val.clone());
                Ok(val)
            }

            Expr::Fn {
                params,
                body,
                is_variadic,
            } => Ok(Value::Function(Rc::new(Function {
                params: params.clone(),
                body: (**body).clone(),
                env: env.borrow().clone(),
                is_variadic: *is_variadic,
            }))),

            Expr::Let { bindings, body } => {
                let mut new_env = Env::with_parent(env.clone());
                for (name, expr) in bindings {
                    let value = self.eval_with_env(expr, Rc::new(RefCell::new(new_env.clone())))?;
                    new_env.set(name.clone(), value);
                }
                self.eval_with_env(body, Rc::new(RefCell::new(new_env)))
            }

            Expr::If {
                test,
                then,
                otherwise,
            } => {
                let test_val = self.eval_with_env(test, env.clone())?;
                if is_truthy(&test_val) {
                    self.eval_with_env(then, env)
                } else if let Some(otherwise) = otherwise {
                    self.eval_with_env(otherwise, env)
                } else {
                    Ok(Value::Nil)
                }
            }

            Expr::Do(exprs) => {
                let mut result = Value::Nil;
                for expr in exprs {
                    result = self.eval_with_env(expr, env.clone())?;
                }
                Ok(result)
            }

            Expr::Match { expr, arms } => {
                let value = self.eval_with_env(expr, env.clone())?;
                self.eval_match(&value, arms, env)
            }

            Expr::Call { func, args } => {
                let func_val = self.eval_with_env(func, env.clone())?;
                let arg_vals: Result<Vec<_>, _> = args
                    .iter()
                    .map(|e| self.eval_with_env(e, env.clone()))
                    .collect();
                let arg_vals = arg_vals?;

                match func_val {
                    Value::NativeFunc(nf) => (nf.func)(&arg_vals),
                    Value::Function(f) => {
                        let parent_env = Rc::new(RefCell::new(f.env.clone()));
                        let mut new_env = Env::with_parent(parent_env);

                        if f.is_variadic {
                            // 可変長引数の処理
                            if f.params.len() != 1 {
                                return Err("可変長引数関数はパラメータが1つである必要があります".to_string());
                            }
                            new_env.set(f.params[0].clone(), Value::List(arg_vals));
                        } else {
                            // 通常の引数
                            if f.params.len() != arg_vals.len() {
                                return Err(format!(
                                    "引数の数が一致しません: 期待 {}, 実際 {}",
                                    f.params.len(),
                                    arg_vals.len()
                                ));
                            }
                            for (param, arg) in f.params.iter().zip(arg_vals.iter()) {
                                new_env.set(param.clone(), arg.clone());
                            }
                        }

                        self.eval_with_env(&f.body, Rc::new(RefCell::new(new_env)))
                    }
                    _ => Err(format!("関数ではありません: {:?}", func_val)),
                }
            }
        }
    }

    fn eval_match(
        &mut self,
        value: &Value,
        arms: &[MatchArm],
        env: Rc<RefCell<Env>>,
    ) -> Result<Value, String> {
        for arm in arms {
            let mut bindings = HashMap::new();
            if self.match_pattern(&arm.pattern, value, &mut bindings)? {
                // ガード条件のチェック
                if let Some(guard) = &arm.guard {
                    let mut guard_env = Env::with_parent(env.clone());
                    for (name, val) in &bindings {
                        guard_env.set(name.clone(), val.clone());
                    }
                    let guard_val = self.eval_with_env(guard, Rc::new(RefCell::new(guard_env)))?;
                    if !is_truthy(&guard_val) {
                        continue;
                    }
                }

                // マッチ成功：バインディングを環境に追加して本体を評価
                let mut match_env = Env::with_parent(env.clone());
                for (name, val) in bindings {
                    match_env.set(name, val);
                }
                return self.eval_with_env(&arm.body, Rc::new(RefCell::new(match_env)));
            }
        }
        Err("どのパターンにもマッチしませんでした".to_string())
    }

    fn match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
        bindings: &mut HashMap<String, Value>,
    ) -> Result<bool, String> {
        match pattern {
            Pattern::Wildcard => Ok(true),
            Pattern::Nil => Ok(matches!(value, Value::Nil)),
            Pattern::Bool(b) => Ok(matches!(value, Value::Bool(vb) if vb == b)),
            Pattern::Integer(n) => Ok(matches!(value, Value::Integer(vn) if vn == n)),
            Pattern::Float(f) => Ok(matches!(value, Value::Float(vf) if (vf - f).abs() < f64::EPSILON)),
            Pattern::String(s) => Ok(matches!(value, Value::String(vs) if vs == s)),
            Pattern::Keyword(k) => Ok(matches!(value, Value::Keyword(vk) if vk == k)),
            Pattern::Var(name) => {
                bindings.insert(name.clone(), value.clone());
                Ok(true)
            }
            Pattern::Vector(patterns) => {
                if let Value::Vector(values) = value {
                    if patterns.len() != values.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if !self.match_pattern(pat, val, bindings)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::List(patterns, _rest) => {
                if let Value::List(values) = value {
                    if patterns.len() > values.len() {
                        return Ok(false);
                    }
                    for (pat, val) in patterns.iter().zip(values.iter()) {
                        if !self.match_pattern(pat, val, bindings)? {
                            return Ok(false);
                        }
                    }
                    // TODO: ...restのサポート
                    if patterns.len() != values.len() {
                        return Ok(false);
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Pattern::Map(pattern_pairs) => {
                if let Value::Map(map) = value {
                    for (key, pat) in pattern_pairs {
                        if let Some(val) = map.get(key) {
                            if !self.match_pattern(pat, val, bindings)? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}

fn is_truthy(val: &Value) -> bool {
    !matches!(val, Value::Nil | Value::Bool(false))
}

// ネイティブ関数の実装

fn native_add(args: &[Value]) -> Result<Value, String> {
    let mut sum = 0i64;
    for arg in args {
        match arg {
            Value::Integer(n) => sum += n,
            _ => return Err(format!("+ は整数のみ受け付けます: {:?}", arg)),
        }
    }
    Ok(Value::Integer(sum))
}

fn native_sub(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("- には少なくとも1つの引数が必要です".to_string());
    }
    match args[0] {
        Value::Integer(first) => {
            if args.len() == 1 {
                Ok(Value::Integer(-first))
            } else {
                let mut result = first;
                for arg in &args[1..] {
                    match arg {
                        Value::Integer(n) => result -= n,
                        _ => return Err(format!("- は整数のみ受け付けます: {:?}", arg)),
                    }
                }
                Ok(Value::Integer(result))
            }
        }
        _ => Err(format!("- は整数のみ受け付けます: {:?}", args[0])),
    }
}

fn native_mul(args: &[Value]) -> Result<Value, String> {
    let mut product = 1i64;
    for arg in args {
        match arg {
            Value::Integer(n) => product *= n,
            _ => return Err(format!("* は整数のみ受け付けます: {:?}", arg)),
        }
    }
    Ok(Value::Integer(product))
}

fn native_div(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("/ には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                Err("ゼロ除算エラー".to_string())
            } else {
                Ok(Value::Integer(a / b))
            }
        }
        _ => Err("/ は整数のみ受け付けます".to_string()),
    }
}

fn native_eq(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("= には2つの引数が必要です".to_string());
    }
    Ok(Value::Bool(args[0] == args[1]))
}

fn native_lt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("< には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a < b)),
        _ => Err("< は整数のみ受け付けます".to_string()),
    }
}

fn native_gt(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("> には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a > b)),
        _ => Err("> は整数のみ受け付けます".to_string()),
    }
}

fn native_list(args: &[Value]) -> Result<Value, String> {
    Ok(Value::List(args.to_vec()))
}

fn native_first(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("first には1つの引数が必要です".to_string());
    }
    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            Ok(items.first().cloned().unwrap_or(Value::Nil))
        }
        _ => Err("first はリストまたはベクタのみ受け付けます".to_string()),
    }
}

fn native_rest(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("rest には1つの引数が必要です".to_string());
    }
    match &args[0] {
        Value::List(items) => {
            if items.is_empty() {
                Ok(Value::List(vec![]))
            } else {
                Ok(Value::List(items[1..].to_vec()))
            }
        }
        Value::Vector(items) => {
            if items.is_empty() {
                Ok(Value::Vector(vec![]))
            } else {
                Ok(Value::Vector(items[1..].to_vec()))
            }
        }
        _ => Err("rest はリストまたはベクタのみ受け付けます".to_string()),
    }
}

fn native_len(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("len には1つの引数が必要です".to_string());
    }
    match &args[0] {
        Value::List(items) | Value::Vector(items) => Ok(Value::Integer(items.len() as i64)),
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Map(m) => Ok(Value::Integer(m.len() as i64)),
        _ => Err("len はコレクションまたは文字列のみ受け付けます".to_string()),
    }
}

fn native_print(args: &[Value]) -> Result<Value, String> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn eval_str(input: &str) -> Result<Value, String> {
        let mut parser = Parser::new(input)?;
        let exprs = parser.parse_all()?;
        let mut evaluator = Evaluator::new();
        let mut result = Value::Nil;
        for expr in exprs {
            result = evaluator.eval(&expr)?;
        }
        Ok(result)
    }

    #[test]
    fn test_integers() {
        assert_eq!(eval_str("42").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_add() {
        assert_eq!(eval_str("(+ 1 2 3)").unwrap(), Value::Integer(6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(eval_str("(- 10 3)").unwrap(), Value::Integer(7));
    }

    #[test]
    fn test_mul() {
        assert_eq!(eval_str("(* 2 3 4)").unwrap(), Value::Integer(24));
    }

    #[test]
    fn test_nested() {
        assert_eq!(eval_str("(+ (* 2 3) (- 10 5))").unwrap(), Value::Integer(11));
    }

    #[test]
    fn test_if() {
        assert_eq!(eval_str("(if true 1 2)").unwrap(), Value::Integer(1));
        assert_eq!(eval_str("(if false 1 2)").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_fn() {
        assert_eq!(
            eval_str("((fn [x] (+ x 1)) 5)").unwrap(),
            Value::Integer(6)
        );
    }

    #[test]
    fn test_let() {
        assert_eq!(
            eval_str("(let [x 10 y 20] (+ x y))").unwrap(),
            Value::Integer(30)
        );
    }

    #[test]
    fn test_match_literal() {
        // 値のマッチ
        assert_eq!(
            eval_str("(match 0 0 -> 42 1 -> 99)").unwrap(),
            Value::Integer(42)
        );
        assert_eq!(
            eval_str("(match 1 0 -> 42 1 -> 99)").unwrap(),
            Value::Integer(99)
        );
    }

    #[test]
    fn test_match_var() {
        // 変数のバインディング
        assert_eq!(
            eval_str("(match 10 n -> (+ n 5))").unwrap(),
            Value::Integer(15)
        );
    }

    #[test]
    fn test_match_wildcard() {
        // ワイルドカード
        assert_eq!(
            eval_str("(match 42 0 -> 1 1 -> 2 _ -> 99)").unwrap(),
            Value::Integer(99)
        );
    }

    #[test]
    fn test_match_nil_bool() {
        // nil/boolの区別
        assert_eq!(
            eval_str("(match nil nil -> 1 false -> 2 _ -> 3)").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            eval_str("(match false nil -> 1 false -> 2 _ -> 3)").unwrap(),
            Value::Integer(2)
        );
        assert_eq!(
            eval_str("(match true nil -> 1 false -> 2 _ -> 3)").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_match_vector() {
        // ベクタのマッチ
        assert_eq!(
            eval_str("(match [1 2] [x y] -> (+ x y))").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_match_guard() {
        // ガード条件
        assert_eq!(
            eval_str("(match 5 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            eval_str("(match -5 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
            Value::Integer(-1)
        );
        assert_eq!(
            eval_str("(match 0 n when (> n 0) -> 1 n when (< n 0) -> -1 _ -> 0)").unwrap(),
            Value::Integer(0)
        );
    }

    #[test]
    fn test_pipe_simple() {
        // 単純なパイプライン: (10 |> inc) は (inc 10) と同じ
        assert_eq!(
            eval_str("(def inc (fn [x] (+ x 1))) (10 |> inc)").unwrap(),
            Value::Integer(11)
        );
    }

    #[test]
    fn test_pipe_chain() {
        // パイプラインのチェーン: (1 |> inc |> inc) は 3
        assert_eq!(
            eval_str("(def inc (fn [x] (+ x 1))) (1 |> inc |> inc)").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_pipe_with_args() {
        // 引数ありの関数: (10 |> (+ 5)) は (+ 5 10) = 15
        assert_eq!(
            eval_str("(10 |> (+ 5))").unwrap(),
            Value::Integer(15)
        );
    }

    #[test]
    fn test_pipe_complex() {
        // 複雑なパイプライン: (1 |> (+ 2) |> (* 3)) は ((* 3 (+ 2 1))) = 9
        assert_eq!(
            eval_str("(1 |> (+ 2) |> (* 3))").unwrap(),
            Value::Integer(9)
        );
    }
}
