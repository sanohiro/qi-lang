//! 数学関数

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::require_number;
use crate::value::Value;

#[cfg(feature = "std-math")]
use rand::Rng;

/// pow - べき乗
pub fn native_pow(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "pow");

    let base_f = match &args[0] {
        Value::Integer(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pow (base)", "numbers"])),
    };
    let exp_f = match &args[1] {
        Value::Integer(i) => *i as f64,
        Value::Float(f) => *f,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["pow (exponent)", "numbers"])),
    };

    // 両方が整数で指数が非負の場合は整数で返す
    if let (Value::Integer(base), Value::Integer(exp)) = (&args[0], &args[1]) {
        if *exp >= 0 {
            if let Some(result) = base.checked_pow(*exp as u32) {
                return Ok(Value::Integer(result));
            }
        }
    }

    Ok(Value::Float(base_f.powf(exp_f)))
}

/// sqrt - 平方根
pub fn native_sqrt(args: &[Value]) -> Result<Value, String> {
    let n = require_number!(args, "sqrt");
    Ok(Value::Float(n.sqrt()))
}

/// round - 四捨五入
pub fn native_round(args: &[Value]) -> Result<Value, String> {
    let n = require_number!(args, "round");

    let rounded = n.round();
    if rounded.is_nan() || rounded.is_infinite() {
        return Err(fmt_msg(MsgKey::FloatIsNanOrInfinity, &["round"]));
    }
    if rounded < i64::MIN as f64 || rounded > i64::MAX as f64 {
        return Err(fmt_msg(
            MsgKey::FloatOutOfI64Range,
            &["round", &rounded.to_string()],
        ));
    }
    Ok(Value::Integer(rounded as i64))
}

/// floor - 切り捨て
pub fn native_floor(args: &[Value]) -> Result<Value, String> {
    let n = require_number!(args, "floor");

    let floored = n.floor();
    if floored.is_nan() || floored.is_infinite() {
        return Err(fmt_msg(MsgKey::FloatIsNanOrInfinity, &["floor"]));
    }
    if floored < i64::MIN as f64 || floored > i64::MAX as f64 {
        return Err(fmt_msg(
            MsgKey::FloatOutOfI64Range,
            &["floor", &floored.to_string()],
        ));
    }
    Ok(Value::Integer(floored as i64))
}

/// ceil - 切り上げ
pub fn native_ceil(args: &[Value]) -> Result<Value, String> {
    let n = require_number!(args, "ceil");

    let ceiled = n.ceil();
    if ceiled.is_nan() || ceiled.is_infinite() {
        return Err(fmt_msg(MsgKey::FloatIsNanOrInfinity, &["ceil"]));
    }
    if ceiled < i64::MIN as f64 || ceiled > i64::MAX as f64 {
        return Err(fmt_msg(
            MsgKey::FloatOutOfI64Range,
            &["ceil", &ceiled.to_string()],
        ));
    }
    Ok(Value::Integer(ceiled as i64))
}

/// clamp - 値を範囲内に制限
pub fn native_clamp(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 3, "clamp");

    match (&args[0], &args[1], &args[2]) {
        (Value::Integer(val), Value::Integer(min), Value::Integer(max)) => {
            Ok(Value::Integer((*val).clamp(*min, *max)))
        }
        _ => {
            // floatの場合も対応
            let val = match &args[0] {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["clamp (value)", "numbers"])),
            };
            let min = match &args[1] {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["clamp (min)", "numbers"])),
            };
            let max = match &args[2] {
                Value::Integer(i) => *i as f64,
                Value::Float(f) => *f,
                _ => return Err(fmt_msg(MsgKey::TypeOnly, &["clamp (max)", "numbers"])),
            };
            Ok(Value::Float(val.clamp(min, max)))
        }
    }
}

/// rand - 0.0以上1.0未満の乱数を生成
///
/// cryptographicallyセキュアな乱数生成器を使用して、
/// 0.0以上1.0未満の浮動小数点数を生成します。
///
/// # 引数
/// なし
///
/// # 戻り値
/// `float` - 0.0 <= n < 1.0 の乱数
///
/// # 使用例
/// ```qi
/// (math/rand)  ;=> 0.7234...
/// (math/rand |> (* 100) |> math/floor)  ;=> 0～99の整数
/// ```
///
/// # 必須feature
/// `std-math`
#[cfg(feature = "std-math")]
pub fn native_rand(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 0, "rand");

    let mut rng = rand::rng();
    Ok(Value::Float(rng.random::<f64>()))
}

/// rand-int - 0以上n未満の整数乱数を生成
///
/// 0以上指定された値未満の整数乱数を生成します。
/// 上限値は1以上である必要があります。
///
/// # 引数
/// - `n: integer` - 上限値（n > 0）
///
/// # 戻り値
/// `integer` - 0 <= result < n の整数乱数
///
/// # 使用例
/// ```qi
/// (math/rand-int 10)     ;=> 0～9の乱数
/// (math/rand-int 100)    ;=> 0～99の乱数
/// (stream/range 10 |> map (fn [_] (math/rand-int 6)))  ;=> サイコロ10回
/// ```
///
/// # 必須feature
/// `std-math`
#[cfg(feature = "std-math")]
pub fn native_rand_int(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "rand-int");

    match &args[0] {
        Value::Integer(n) => {
            if *n <= 0 {
                return Err(fmt_msg(
                    MsgKey::MustBePositive,
                    &["rand-int", "upper bound"],
                ));
            }
            let mut rng = rand::rng();
            Ok(Value::Integer(rng.random_range(0..*n)))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["rand-int", "integers"])),
    }
}

/// random-range - min以上max未満の整数乱数を生成
///
/// 指定された範囲[min, max)内の整数乱数を生成します。
/// minはmaxより小さい必要があります。
///
/// # 引数
/// - `min: integer` - 最小値（含む）
/// - `max: integer` - 最大値（含まない）
///
/// # 戻り値
/// `integer` - min <= result < max の整数乱数
///
/// # 使用例
/// ```qi
/// (math/random-range 1 10)    ;=> 1～9の乱数
/// (math/random-range -10 10)  ;=> -10～9の乱数
/// (stream/range 5 |> map (fn [_] (math/random-range 100 200)))
/// ```
///
/// # 必須feature
/// `std-math`
#[cfg(feature = "std-math")]
pub fn native_random_range(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "random-range");

    match (&args[0], &args[1]) {
        (Value::Integer(min), Value::Integer(max)) => {
            if min >= max {
                return Err(fmt_msg(MsgKey::MinMustBeLessThanMax, &["random-range"]));
            }
            let mut rng = rand::rng();
            Ok(Value::Integer(rng.random_range(*min..*max)))
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["random-range", "integers"])),
    }
}

/// shuffle - リスト/ベクトルをランダムにシャッフル
///
/// Fisher-Yatesアルゴリズムを使用して、リストまたはベクトルの要素を
/// ランダムにシャッフルします。元の値は変更されず、シャッフルされた
/// コピーが返されます。
///
/// # 引数
/// - `coll: list | vector` - シャッフルするコレクション
///
/// # 戻り値
/// `list | vector` - シャッフルされたコレクション（元の型を保持）
///
/// # 使用例
/// ```qi
/// (math/shuffle [1 2 3 4 5])     ;=> [3 1 4 5 2] （例）
/// (math/shuffle '(a b c d e))    ;=> (c a d e b) （例）
///
/// ;; ゲームのカード処理
/// (def cards ["A" "2" "3" "4" "5" "6" "7" "8" "9" "10" "J" "Q" "K"])
/// (math/shuffle cards)            ;=> シャッフルされたカード
/// ```
///
/// # 必須feature
/// `std-math`
#[cfg(feature = "std-math")]
pub fn native_shuffle(args: &[Value]) -> Result<Value, String> {
    use rand::seq::SliceRandom;

    check_args!(args, 1, "shuffle");

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            let mut shuffled: Vec<_> = items.iter().cloned().collect();
            let mut rng = rand::rng();
            shuffled.shuffle(&mut rng);

            match &args[0] {
                Value::List(_) => Ok(Value::List(shuffled.into())),
                Value::Vector(_) => Ok(Value::Vector(shuffled.into())),
                _ => Err(fmt_msg(
                    MsgKey::InternalError,
                    &["shuffle: unexpected collection type"],
                )),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["shuffle", "lists or vectors"])),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（feature-gatedでない関数のみ）
/// @qi-doc:category math
/// @qi-doc:functions pow, sqrt, round, floor, ceil, clamp
/// @qi-doc:note rand, rand-int, random-range, shuffleは "std-math" featureが必要なため別途登録
pub const FUNCTIONS: super::NativeFunctions = &[
    ("math/pow", native_pow),
    ("math/sqrt", native_sqrt),
    ("math/round", native_round),
    ("math/floor", native_floor),
    ("math/ceil", native_ceil),
    ("math/clamp", native_clamp),
];

/// Feature-gated関数のリスト (std-math feature)
#[cfg(feature = "std-math")]
pub const FUNCTIONS_STD_MATH: super::NativeFunctions = &[
    ("math/rand", native_rand),
    ("math/rand-int", native_rand_int),
    ("math/random-range", native_random_range),
    ("math/shuffle", native_shuffle),
];
