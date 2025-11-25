//! ビルトイン関数用のユーティリティマクロ
//!
//! このモジュールは、引数チェックやエラーハンドリングなど、
//! ビルトイン関数で繰り返し使用されるパターンを統一するマクロを提供します。

/// 引数の個数をチェックするマクロ
///
/// # 使用例
///
/// ```
/// use qi_lang::value::Value;
/// use qi_lang::check_args;
///
/// fn example_func(args: &[Value]) -> Result<(), String> {
///     // 引数が正確に2個必要な場合
///     check_args!(args, 2, "example");
///     Ok(())
/// }
///
/// let args = vec![Value::Integer(1), Value::Integer(2)];
/// assert!(example_func(&args).is_ok());
///
/// let args = vec![Value::Integer(1)];
/// assert!(example_func(&args).is_err());
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
/// ```
/// // このマクロは主にビルトイン関数の登録に使用されます。
/// // Envモジュールは内部APIのため、ここでは簡略化した例を示します。
/// use qi_lang::value::Value;
///
/// fn example_func(_args: &[Value]) -> Result<Value, String> {
///     Ok(Value::Integer(42))
/// }
///
/// // 実際の使用例（内部のみ）:
/// // register_native!(env.write(),
/// //     "string/split" => string::native_split,
/// //     "string/join" => string::native_join,
/// // );
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

/// グローバルマップから値を安全に取得してクローン
///
/// グローバルな `Mutex<HashMap>` や `parking_lot::Mutex<HashMap>` から
/// 値を取得する際の定型コードを削減します。
///
/// # 引数
/// - `$map`: グローバルマップ（例: `CONNECTIONS`, `POOLS`, `TRANSACTIONS`）
/// - `$key`: 検索キー（`&str` または `&String`）
/// - `$err_key`: エラーメッセージキー（`MsgKey::XXX`）
///
/// # 戻り値
/// - `Ok(T)`: マップから取得した値（クローン）
/// - `Err(String)`: キーが見つからない場合のエラーメッセージ
///
/// # 使用例
/// ```rust,ignore
/// // データベース接続取得
/// let conn = with_global!(CONNECTIONS, &conn_id, MsgKey::DbConnectionNotFound)?;
///
/// // トランザクション取得
/// let tx = with_global!(TRANSACTIONS, &tx_id, MsgKey::DbTransactionNotFound)?;
///
/// // プール取得
/// let pool = with_global!(POOLS, &pool_id, MsgKey::DbPoolNotFound)?;
///
/// // KVS接続取得
/// let driver = with_global!(CONNECTIONS, &conn_id, MsgKey::ConnectionNotFound)?;
/// ```
///
/// # 生成されるコード
/// ```rust,ignore
/// {
///     let lock = CONNECTIONS.lock();
///     lock.get(&conn_id)
///         .ok_or_else(|| fmt_msg(MsgKey::DbConnectionNotFound, &[&conn_id]))?
///         .clone()
/// }
/// ```
#[macro_export]
macro_rules! with_global {
    ($map:expr, $key:expr, $err_key:expr) => {{
        let lock = $map.lock();
        lock.get($key)
            .ok_or_else(|| $crate::i18n::fmt_msg($err_key, &[$key]))?
            .clone()
    }};
}

/// エラーをi18nメッセージに変換
///
/// `Result<T, E>` の `E` を `String` (i18nメッセージ) に変換します。
/// `map_err()` の冗長なボイラープレートを削減します。
///
/// # 引数
/// - `$result`: `Result<T, E>` 型の式
/// - `$msg_key`: エラーメッセージキー（`MsgKey::XXX`）
/// - `$arg`: メッセージパラメータ（0個以上）
///
/// # 使用例
/// ```rust,ignore
/// // ファイルオープン
/// let file = map_i18n_err!(File::create(path), MsgKey::FileNotFound, path)?;
///
/// // HTTP接続
/// let resp = map_i18n_err!(
///     reqwest::blocking::get(url),
///     MsgKey::HttpRequestFailed,
///     "GET",
///     url
/// )?;
/// ```
///
/// # 生成されるコード
/// ```rust,ignore
/// File::create(path).map_err(|e| {
///     fmt_msg(MsgKey::FileNotFound, &[path, &e.to_string()])
/// })
/// ```
#[macro_export]
macro_rules! map_i18n_err {
    ($result:expr, $msg_key:expr $(, $arg:expr)*) => {
        $result.map_err(|e| {
            $crate::i18n::fmt_msg($msg_key, &[$($arg,)* &e.to_string()])
        })
    };
}

/// エラーを `DbError` 型に変換
///
/// データベース操作のエラーを `DbError` でラップし、i18nメッセージに変換します。
///
/// # 引数
/// - `$result`: `Result<T, E>` 型の式
/// - `$msg_key`: エラーメッセージキー（`MsgKey::XXX`）
/// - `$arg`: メッセージパラメータ（0個以上）
///
/// # 使用例
/// ```rust,ignore
/// // データベース接続
/// let conn = map_db_err!(rusqlite::Connection::open(path), MsgKey::DbFailedToConnect)?;
///
/// // クエリ実行
/// let rows = map_db_err!(
///     conn.query(sql, params),
///     MsgKey::DbFailedToExecuteQuery
/// )?;
/// ```
///
/// # 生成されるコード
/// ```rust,ignore
/// rusqlite::Connection::open(path).map_err(|e| {
///     DbError::new(fmt_msg(MsgKey::DbFailedToConnect, &[&e.to_string()]))
/// })
/// ```
#[macro_export]
macro_rules! map_db_err {
    ($result:expr, $msg_key:expr $(, $arg:expr)*) => {
        $result.map_err(|e| {
            $crate::builtins::db::types::DbError::new(
                $crate::i18n::fmt_msg($msg_key, &[$($arg,)* &e.to_string()])
            )
        })
    };
}

/// 引数1個チェック + 文字列型チェック + 抽出を1行で実行
///
/// 最も頻繁に使用されるパターン（引数1個かつ文字列）を1マクロで処理。
///
/// # 使用例
/// ```rust,ignore
/// // Before (6行)
/// if args.len() != 1 {
///     return Err(fmt_msg(MsgKey::Need1Arg, &["upper"]));
/// }
/// match &args[0] {
///     Value::String(s) => Ok(Value::String(s.to_uppercase())),
///     _ => Err(fmt_msg(MsgKey::TypeOnly, &["upper", "strings"])),
/// }
///
/// // After (2行)
/// let s = require_string!(args, "upper");
/// Ok(Value::String(s.to_uppercase()))
/// ```
#[macro_export]
macro_rules! require_string {
    ($args:expr, $func:expr) => {{
        if $args.len() != 1 {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need1Arg,
                &[$func],
            ));
        }
        match &$args[0] {
            $crate::value::Value::String(s) => s.as_str(),
            _ => {
                return Err($crate::i18n::fmt_msg(
                    $crate::i18n::MsgKey::TypeOnly,
                    &[$func, "strings"],
                ))
            }
        }
    }};
}

/// 引数1個チェック + 整数型チェック + 抽出を1行で実行
///
/// 整数引数1個を取る関数で使用。
///
/// # 使用例
/// ```rust,ignore
/// let n = require_int!(args, "time/from-unix");
/// ```
#[macro_export]
macro_rules! require_int {
    ($args:expr, $func:expr) => {{
        if $args.len() != 1 {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need1Arg,
                &[$func],
            ));
        }
        match &$args[0] {
            $crate::value::Value::Integer(n) => *n,
            _ => {
                return Err($crate::i18n::fmt_msg(
                    $crate::i18n::MsgKey::TypeOnly,
                    &[$func, "integers"],
                ))
            }
        }
    }};
}

/// 引数1個チェック + 数値型（Integer/Float）チェック + 抽出を1行で実行
///
/// IntegerまたはFloatを受け入れ、f64として返す。
///
/// # 使用例
/// ```rust,ignore
/// let n = require_number!(args, "math/sqrt");
/// ```
#[macro_export]
macro_rules! require_number {
    ($args:expr, $func:expr) => {{
        if $args.len() != 1 {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need1Arg,
                &[$func],
            ));
        }
        match &$args[0] {
            $crate::value::Value::Float(f) => *f,
            $crate::value::Value::Integer(i) => *i as f64,
            _ => {
                return Err($crate::i18n::fmt_msg(
                    $crate::i18n::MsgKey::TypeOnly,
                    &[$func, "numbers"],
                ))
            }
        }
    }};
}

/// 引数2個チェック + 両方文字列型チェック + 抽出を1行で実行
///
/// 2つの文字列引数を取る関数で使用。
///
/// # 使用例
/// ```rust,ignore
/// let (s1, s2) = require_2_strings!(args, "string/concat");
/// ```
#[macro_export]
macro_rules! require_2_strings {
    ($args:expr, $func:expr) => {{
        if $args.len() != 2 {
            return Err($crate::i18n::fmt_msg(
                $crate::i18n::MsgKey::Need2Args,
                &[$func],
            ));
        }
        let s1 = match &$args[0] {
            $crate::value::Value::String(s) => s.as_str(),
            _ => {
                return Err($crate::i18n::fmt_msg(
                    $crate::i18n::MsgKey::FirstArgMustBe,
                    &[$func, "a string"],
                ))
            }
        };
        let s2 = match &$args[1] {
            $crate::value::Value::String(s) => s.as_str(),
            _ => {
                return Err($crate::i18n::fmt_msg(
                    $crate::i18n::MsgKey::SecondArgMustBe,
                    &[$func, "a string"],
                ))
            }
        };
        (s1, s2)
    }};
}

#[cfg(test)]
mod tests {
    use crate::value::Value;

    #[test]
    fn test_check_args_exact() {
        fn test_func(args: &[Value]) -> Result<Value, String> {
            check_args!(args, 2, "test");
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
            check_args!(args, 0, "test");
            Ok(Value::Nil)
        }

        let args = vec![];
        assert!(test_func(&args).is_ok());

        let args = vec![Value::Integer(1)];
        assert!(test_func(&args).is_err());
    }
}
