//! 数学関数

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

#[cfg(feature = "std-math")]
use rand::Rng;

/// pow - べき乗
pub fn native_pow(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["pow", "2", "(base, exponent)"],
        ));
    }

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
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["sqrt"]));
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
        Value::Float(n) => Ok(Value::Float(n.sqrt())),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["sqrt", "numbers"])),
    }
}

/// round - 四捨五入
pub fn native_round(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["round"]));
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(n) => Ok(Value::Integer(n.round() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["round", "numbers"])),
    }
}

/// floor - 切り捨て
pub fn native_floor(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["floor"]));
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(n) => Ok(Value::Integer(n.floor() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["floor", "numbers"])),
    }
}

/// ceil - 切り上げ
pub fn native_ceil(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["ceil"]));
    }

    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(n) => Ok(Value::Integer(n.ceil() as i64)),
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["ceil", "numbers"])),
    }
}

/// clamp - 値を範囲内に制限
pub fn native_clamp(args: &[Value]) -> Result<Value, String> {
    if args.len() != 3 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["clamp", "3", "(value, min, max)"],
        ));
    }

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

/// rand - 0.0以上1.0未満の乱数
#[cfg(feature = "std-math")]
pub fn native_rand(args: &[Value]) -> Result<Value, String> {
    if !args.is_empty() {
        return Err(fmt_msg(MsgKey::Need0Args, &["rand"]));
    }

    let mut rng = rand::rng();
    Ok(Value::Float(rng.random::<f64>()))
}

/// rand-int - 0以上n未満の整数乱数
#[cfg(feature = "std-math")]
pub fn native_rand_int(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["rand-int", "1", "(upper bound)"],
        ));
    }

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

/// random-range - min以上max未満の整数乱数
#[cfg(feature = "std-math")]
pub fn native_random_range(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["random-range", "2", "(min, max)"],
        ));
    }

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

/// shuffle - リストをシャッフル
#[cfg(feature = "std-math")]
pub fn native_shuffle(args: &[Value]) -> Result<Value, String> {
    use rand::seq::SliceRandom;

    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["shuffle"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            let mut shuffled = items.clone();
            let mut rng = rand::rng();
            shuffled.shuffle(&mut rng);

            match &args[0] {
                Value::List(_) => Ok(Value::List(shuffled)),
                Value::Vector(_) => Ok(Value::Vector(shuffled)),
                _ => unreachable!(),
            }
        }
        _ => Err(fmt_msg(MsgKey::TypeOnly, &["shuffle", "lists or vectors"])),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（feature-gatedでない関数のみ）
///
/// 注意: rand, rand-int, random-range, shuffleは "std-math" featureが必要なため別途登録
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
