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
            "<=".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "<=".to_string(),
                func: native_le,
            }),
        );
        env.set(
            ">=".to_string(),
            Value::NativeFunc(NativeFunc {
                name: ">=".to_string(),
                func: native_ge,
            }),
        );
        env.set(
            "!=".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "!=".to_string(),
                func: native_ne,
            }),
        );
        env.set(
            "%".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "%".to_string(),
                func: native_mod,
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
        env.set(
            "cons".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "cons".to_string(),
                func: native_cons,
            }),
        );
        env.set(
            "conj".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "conj".to_string(),
                func: native_conj,
            }),
        );
        env.set(
            "empty?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "empty?".to_string(),
                func: native_empty,
            }),
        );
        env.set(
            "nil?".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "nil?".to_string(),
                func: native_nil,
            }),
        );
        env.set(
            "str".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "str".to_string(),
                func: native_str,
            }),
        );
        env.set(
            "not".to_string(),
            Value::NativeFunc(NativeFunc {
                name: "not".to_string(),
                func: native_not,
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
                // 高階関数と論理演算子、quoteの特別処理
                if let Expr::Symbol(name) = func.as_ref() {
                    match name.as_str() {
                        "map" => return self.eval_map(args, env),
                        "filter" => return self.eval_filter(args, env),
                        "reduce" => return self.eval_reduce(args, env),
                        "and" => return self.eval_and(args, env),
                        "or" => return self.eval_or(args, env),
                        "quote" => return self.eval_quote(args),
                        _ => {}
                    }
                }

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

    /// map関数の実装: (map f coll)
    fn eval_map(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(format!("mapは2つの引数が必要です: 実際 {}", args.len()));
        }

        let func = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;

        match coll {
            Value::List(items) | Value::Vector(items) => {
                let mut results = Vec::new();
                for item in items {
                    let result = self.apply_func(&func, vec![item])?;
                    results.push(result);
                }
                Ok(Value::List(results))
            }
            _ => Err("mapの第2引数はリストまたはベクタである必要があります".to_string()),
        }
    }

    /// filter関数の実装: (filter pred coll)
    fn eval_filter(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        if args.len() != 2 {
            return Err(format!("filterは2つの引数が必要です: 実際 {}", args.len()));
        }

        let pred = self.eval_with_env(&args[0], env.clone())?;
        let coll = self.eval_with_env(&args[1], env.clone())?;

        match coll {
            Value::List(items) | Value::Vector(items) => {
                let mut results = Vec::new();
                for item in items {
                    let test = self.apply_func(&pred, vec![item.clone()])?;
                    if is_truthy(&test) {
                        results.push(item);
                    }
                }
                Ok(Value::List(results))
            }
            _ => Err("filterの第2引数はリストまたはベクタである必要があります".to_string()),
        }
    }

    /// reduce関数の実装: (reduce f init coll) または (reduce f coll)
    fn eval_reduce(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        if args.len() < 2 || args.len() > 3 {
            return Err(format!("reduceは2または3つの引数が必要です: 実際 {}", args.len()));
        }

        let func = self.eval_with_env(&args[0], env.clone())?;

        let (mut acc, items) = if args.len() == 3 {
            let init = self.eval_with_env(&args[1], env.clone())?;
            let coll = self.eval_with_env(&args[2], env.clone())?;
            match coll {
                Value::List(items) | Value::Vector(items) => (init, items),
                _ => return Err("reduceの第3引数はリストまたはベクタである必要があります".to_string()),
            }
        } else {
            let coll = self.eval_with_env(&args[1], env.clone())?;
            match coll {
                Value::List(mut items) | Value::Vector(mut items) => {
                    if items.is_empty() {
                        return Err("reduceの引数が2つの場合、コレクションは空であってはいけません".to_string());
                    }
                    let init = items.remove(0);
                    (init, items)
                }
                _ => return Err("reduceの第2引数はリストまたはベクタである必要があります".to_string()),
            }
        };

        for item in items {
            acc = self.apply_func(&func, vec![acc, item])?;
        }

        Ok(acc)
    }

    /// 関数を適用するヘルパー
    fn apply_func(&mut self, func: &Value, args: Vec<Value>) -> Result<Value, String> {
        match func {
            Value::NativeFunc(nf) => (nf.func)(&args),
            Value::Function(f) => {
                let parent_env = Rc::new(RefCell::new(f.env.clone()));
                let mut new_env = Env::with_parent(parent_env);

                if f.is_variadic {
                    if f.params.len() != 1 {
                        return Err("可変長引数関数はパラメータが1つである必要があります".to_string());
                    }
                    new_env.set(f.params[0].clone(), Value::List(args));
                } else {
                    if f.params.len() != args.len() {
                        return Err(format!(
                            "引数の数が一致しません: 期待 {}, 実際 {}",
                            f.params.len(),
                            args.len()
                        ));
                    }
                    for (param, arg) in f.params.iter().zip(args.iter()) {
                        new_env.set(param.clone(), arg.clone());
                    }
                }

                self.eval_with_env(&f.body, Rc::new(RefCell::new(new_env)))
            }
            _ => Err(format!("関数ではありません: {:?}", func)),
        }
    }

    /// and論理演算子（短絡評価）
    fn eval_and(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Ok(Value::Bool(true));
        }
        let mut last = Value::Bool(true);
        for arg in args {
            last = self.eval_with_env(arg, env.clone())?;
            if !is_truthy(&last) {
                return Ok(last);
            }
        }
        Ok(last)
    }

    /// or論理演算子（短絡評価）
    fn eval_or(&mut self, args: &[Expr], env: Rc<RefCell<Env>>) -> Result<Value, String> {
        if args.is_empty() {
            return Ok(Value::Nil);
        }
        for arg in args {
            let val = self.eval_with_env(arg, env.clone())?;
            if is_truthy(&val) {
                return Ok(val);
            }
        }
        Ok(Value::Nil)
    }

    /// quote - 式を評価せずにそのまま返す
    fn eval_quote(&self, args: &[Expr]) -> Result<Value, String> {
        if args.len() != 1 {
            return Err(format!("quoteは1つの引数が必要です: 実際 {}", args.len()));
        }
        self.expr_to_value(&args[0])
    }

    /// ExprをValueに変換（評価せずに）
    fn expr_to_value(&self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Nil => Ok(Value::Nil),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Symbol(s) => Ok(Value::Symbol(s.clone())),
            Expr::Keyword(k) => Ok(Value::Keyword(k.clone())),
            Expr::List(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(Value::List(values?))
            }
            Expr::Vector(items) => {
                let values: Result<Vec<_>, _> = items
                    .iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(Value::Vector(values?))
            }
            Expr::Map(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key = match self.expr_to_value(k)? {
                        Value::Keyword(k) => k,
                        Value::String(s) => s,
                        Value::Symbol(s) => s,
                        _ => return Err("マップのキーは文字列またはキーワードが必要です".to_string()),
                    };
                    let value = self.expr_to_value(v)?;
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
            }
            // 特殊形式やCallは評価せずにリストとして返す
            Expr::Call { func, args } => {
                let mut items = vec![self.expr_to_value(func)?];
                for arg in args {
                    items.push(self.expr_to_value(arg)?);
                }
                Ok(Value::List(items))
            }
            _ => Err(format!("quoteできない式: {:?}", expr)),
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

fn native_le(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("<= には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a <= b)),
        _ => Err("<= は整数のみ受け付けます".to_string()),
    }
}

fn native_ge(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(">= には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(a >= b)),
        _ => Err(">= は整数のみ受け付けます".to_string()),
    }
}

fn native_ne(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("!= には2つの引数が必要です".to_string());
    }
    Ok(Value::Bool(args[0] != args[1]))
}

fn native_mod(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("% には2つの引数が必要です".to_string());
    }
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => {
            if *b == 0 {
                Err("ゼロ除算エラー".to_string())
            } else {
                Ok(Value::Integer(a % b))
            }
        }
        _ => Err("% は整数のみ受け付けます".to_string()),
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

/// cons - リストの先頭に要素を追加
fn native_cons(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("consには2つの引数が必要です".to_string());
    }
    match &args[1] {
        Value::List(items) => {
            let mut new_items = vec![args[0].clone()];
            new_items.extend(items.clone());
            Ok(Value::List(new_items))
        }
        Value::Nil => Ok(Value::List(vec![args[0].clone()])),
        _ => Err("consの第2引数はリストである必要があります".to_string()),
    }
}

/// conj - コレクションに要素を追加
fn native_conj(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("conjには少なくとも2つの引数が必要です".to_string());
    }
    match &args[0] {
        Value::List(items) => {
            let mut new_items = items.clone();
            for arg in &args[1..] {
                new_items.insert(0, arg.clone());
            }
            Ok(Value::List(new_items))
        }
        Value::Vector(items) => {
            let mut new_items = items.clone();
            for arg in &args[1..] {
                new_items.push(arg.clone());
            }
            Ok(Value::Vector(new_items))
        }
        _ => Err("conjの第1引数はリストまたはベクタである必要があります".to_string()),
    }
}

/// empty? - コレクションが空かどうか
fn native_empty(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("empty?には1つの引数が必要です".to_string());
    }
    match &args[0] {
        Value::List(items) | Value::Vector(items) => Ok(Value::Bool(items.is_empty())),
        Value::Map(m) => Ok(Value::Bool(m.is_empty())),
        Value::String(s) => Ok(Value::Bool(s.is_empty())),
        Value::Nil => Ok(Value::Bool(true)),
        _ => Err("empty?はコレクションまたは文字列のみ受け付けます".to_string()),
    }
}

/// nil? - nilかどうか
fn native_nil(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("nil?には1つの引数が必要です".to_string());
    }
    Ok(Value::Bool(matches!(&args[0], Value::Nil)))
}

/// str - 文字列結合
fn native_str(args: &[Value]) -> Result<Value, String> {
    let mut result = String::new();
    for arg in args {
        match arg {
            Value::String(s) => result.push_str(s),
            _ => result.push_str(&format!("{}", arg)),
        }
    }
    Ok(Value::String(result))
}

/// not - 論理否定
fn native_not(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("notには1つの引数が必要です".to_string());
    }
    Ok(Value::Bool(!is_truthy(&args[0])))
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

    #[test]
    fn test_map() {
        // mapのテスト
        assert_eq!(
            eval_str("(map (fn [x] (* x 2)) [1 2 3])").unwrap(),
            Value::List(vec![Value::Integer(2), Value::Integer(4), Value::Integer(6)])
        );
    }

    #[test]
    fn test_filter() {
        // filterのテスト
        assert_eq!(
            eval_str("(filter (fn [x] (> x 2)) [1 2 3 4 5])").unwrap(),
            Value::List(vec![Value::Integer(3), Value::Integer(4), Value::Integer(5)])
        );
    }

    #[test]
    fn test_reduce() {
        // reduceのテスト（初期値あり）
        assert_eq!(
            eval_str("(reduce + 0 [1 2 3 4])").unwrap(),
            Value::Integer(10)
        );
        // reduceのテスト（初期値なし）
        assert_eq!(
            eval_str("(reduce + [1 2 3 4])").unwrap(),
            Value::Integer(10)
        );
    }

    #[test]
    fn test_cons() {
        // consのテスト
        assert_eq!(
            eval_str("(cons 1 (list 2 3))").unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)])
        );
        assert_eq!(
            eval_str("(cons 1 nil)").unwrap(),
            Value::List(vec![Value::Integer(1)])
        );
    }

    #[test]
    fn test_conj() {
        // conjのテスト
        assert_eq!(
            eval_str("(conj [1 2] 3 4)").unwrap(),
            Value::Vector(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3), Value::Integer(4)])
        );
        assert_eq!(
            eval_str("(conj (list 1 2) 3)").unwrap(),
            Value::List(vec![Value::Integer(3), Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_empty() {
        // empty?のテスト
        assert_eq!(eval_str("(empty? [])").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(empty? [1])").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(empty? nil)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_nil_q() {
        // nil?のテスト
        assert_eq!(eval_str("(nil? nil)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(nil? false)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(nil? 0)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_str() {
        // strのテスト
        assert_eq!(
            eval_str("(str \"hello\" \" \" \"world\")").unwrap(),
            Value::String("hello world".to_string())
        );
        assert_eq!(
            eval_str("(str \"count: \" 42)").unwrap(),
            Value::String("count: 42".to_string())
        );
    }

    #[test]
    fn test_and() {
        // andのテスト（短絡評価）
        assert_eq!(eval_str("(and true true)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(and true false)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(and false true)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(and 1 2 3)").unwrap(), Value::Integer(3));
        assert_eq!(eval_str("(and 1 nil 3)").unwrap(), Value::Nil);
    }

    #[test]
    fn test_or() {
        // orのテスト（短絡評価）
        assert_eq!(eval_str("(or false false)").unwrap(), Value::Nil);
        assert_eq!(eval_str("(or false true)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(or true false)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(or nil 2 3)").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_not() {
        // notのテスト
        assert_eq!(eval_str("(not true)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(not false)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(not nil)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(not 42)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_pipeline_with_builtins() {
        // パイプラインと新しい組み込み関数の組み合わせ
        assert_eq!(
            eval_str("[1 2 3 4 5] |> (filter (fn [x] (> x 2))) |> (map (fn [x] (* x 2)))").unwrap(),
            Value::List(vec![Value::Integer(6), Value::Integer(8), Value::Integer(10)])
        );
    }

    #[test]
    fn test_mod() {
        // %（剰余）のテスト
        assert_eq!(eval_str("(% 10 3)").unwrap(), Value::Integer(1));
        assert_eq!(eval_str("(% 15 4)").unwrap(), Value::Integer(3));
        assert_eq!(eval_str("(% 8 2)").unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_le() {
        // <=のテスト
        assert_eq!(eval_str("(<= 5 10)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(<= 10 10)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(<= 15 10)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_ge() {
        // >=のテスト
        assert_eq!(eval_str("(>= 10 5)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(>= 10 10)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(>= 5 10)").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_ne() {
        // !=のテスト
        assert_eq!(eval_str("(!= 1 2)").unwrap(), Value::Bool(true));
        assert_eq!(eval_str("(!= 1 1)").unwrap(), Value::Bool(false));
        assert_eq!(eval_str("(!= nil false)").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_quote() {
        // quoteのテスト
        assert_eq!(
            eval_str("(quote x)").unwrap(),
            Value::Symbol("x".to_string())
        );
        assert_eq!(
            eval_str("'x").unwrap(),
            Value::Symbol("x".to_string())
        );
        assert_eq!(
            eval_str("'(1 2 3)").unwrap(),
            Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)])
        );
        assert_eq!(
            eval_str("'(+ 1 2)").unwrap(),
            Value::List(vec![Value::Symbol("+".to_string()), Value::Integer(1), Value::Integer(2)])
        );
    }

    #[test]
    fn test_even_with_mod() {
        // %を使った偶数判定
        assert_eq!(
            eval_str("(def even? (fn [x] (= (% x 2) 0))) (even? 4)").unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            eval_str("(def even? (fn [x] (= (% x 2) 0))) (even? 5)").unwrap(),
            Value::Bool(false)
        );
    }
}
