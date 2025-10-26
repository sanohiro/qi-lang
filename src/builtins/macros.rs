//! ビルトイン関数用のユーティリティマクロ
//!
//! このモジュールは、引数チェックやエラーハンドリングなど、
//! ビルトイン関数で繰り返し使用されるパターンを統一するマクロを提供します。

/// 引数の個数をチェックするマクロ
///
/// # 使用例
///
/// ```rust
/// // 引数が正確に2個必要な場合
/// check_args!(args, 2, "split")?;
///
/// // 引数が1個必要な場合
/// check_args!(args, 1, "sqrt")?;
///
/// // 引数が最低1個必要な場合
/// check_args!(args, 1.., "print")?;
/// ```
#[macro_export]
macro_rules! check_args {
    // 引数が正確に0個
    ($args:expr, 0, $name:expr) => {
        if !$args.is_empty() {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need0Args,
                &[$name],
            ));
        }
    };

    // 引数が正確に1個
    ($args:expr, 1, $name:expr) => {
        if $args.len() != 1 {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need1Arg,
                &[$name],
            ));
        }
    };

    // 引数が正確に2個
    ($args:expr, 2, $name:expr) => {
        if $args.len() != 2 {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need2Args,
                &[$name],
            ));
        }
    };

    // 引数が正確にN個（汎用）
    ($args:expr, $n:expr, $name:expr) => {
        if $args.len() != $n {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::NeedExactlyNArgs,
                &[$name, &$n.to_string()],
            ));
        }
    };
}

/// ネイティブ関数を登録するマクロ（既存のものを改善）
///
/// # 使用例
///
/// ```rust
/// register_native!(env.write(),
///     "string/split" => string::native_split,
///     "string/join" => string::native_join,
/// );
/// ```
#[macro_export]
macro_rules! register_native {
    ($env:expr, $($name:expr => $func:path),+ $(,)?) => {
        $(
            $env.define(
                $name.to_string(),
                $crate::value::Value::NativeFunction($func),
            );
        )+
    };
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    #[test]
    fn test_check_args_exact() {
        fn test_func(args: &[Value]) -> Result<Value, String> {
            check_args!(args, 2, "test")?;
            Ok(Value::Nil)
        }

        let args = vec![Value::Integer(1), Value::Integer(2)];
        assert!(test_func(&args).is_ok());

        let args = vec![Value::Integer(1)];
        assert!(test_func(&args).is_err());
    }

    #[test]
    fn test_check_args_zero() {
        fn test_func(args: &[Value]) -> Result<Value, String> {
            check_args!(args, 0, "test")?;
            Ok(Value::Nil)
        }

        let args = vec![];
        assert!(test_func(&args).is_ok());

        let args = vec![Value::Integer(1)];
        assert!(test_func(&args).is_err());
    }
}
