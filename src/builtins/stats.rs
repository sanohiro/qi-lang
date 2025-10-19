//! 統計関数
//!
//! このモジュールは `std-stats` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;
use std::collections::HashMap;

/// mean - 平均値
pub fn native_mean(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stats/mean"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Err(fmt_msg(
                    MsgKey::MustNotBeEmpty,
                    &["stats/mean", "collection"],
                ));
            }

            let mut sum = 0.0;
            let mut count = 0;

            for item in items {
                match item {
                    Value::Integer(n) => {
                        sum += *n as f64;
                        count += 1;
                    }
                    Value::Float(f) => {
                        sum += f;
                        count += 1;
                    }
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::AllElementsMustBe,
                            &["stats/mean", "numbers"],
                        ))
                    }
                }
            }

            Ok(Value::Float(sum / count as f64))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["stats/mean", "lists or vectors"],
        )),
    }
}

/// median - 中央値
pub fn native_median(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stats/median"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Err(fmt_msg(
                    MsgKey::MustNotBeEmpty,
                    &["stats/median", "collection"],
                ));
            }

            let mut numbers: Vec<f64> = Vec::new();
            for item in items {
                match item {
                    Value::Integer(n) => numbers.push(*n as f64),
                    Value::Float(f) => numbers.push(*f),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::AllElementsMustBe,
                            &["stats/median", "numbers"],
                        ))
                    }
                }
            }

            numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let len = numbers.len();
            let median = if len % 2 == 0 {
                (numbers[len / 2 - 1] + numbers[len / 2]) / 2.0
            } else {
                numbers[len / 2]
            };

            Ok(Value::Float(median))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["stats/median", "lists or vectors"],
        )),
    }
}

/// mode - 最頻値
pub fn native_mode(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stats/mode"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Err(fmt_msg(
                    MsgKey::MustNotBeEmpty,
                    &["stats/mode", "collection"],
                ));
            }

            let mut freq: HashMap<String, (usize, Value)> = HashMap::new();

            for item in items {
                match item {
                    Value::Integer(_) | Value::Float(_) => {
                        let key = format!("{:?}", item);
                        freq.entry(key)
                            .and_modify(|(count, _)| *count += 1)
                            .or_insert((1, item.clone()));
                    }
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::AllElementsMustBe,
                            &["stats/mode", "numbers"],
                        ))
                    }
                }
            }

            // 最頻値を見つける
            let mut max_count = 0;
            let mut mode_value = &Value::Nil;

            for (count, value) in freq.values() {
                if *count > max_count {
                    max_count = *count;
                    mode_value = value;
                }
            }

            Ok(mode_value.clone())
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["stats/mode", "lists or vectors"],
        )),
    }
}

/// variance - 分散
pub fn native_variance(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["stats/variance"]));
    }

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Err(fmt_msg(
                    MsgKey::MustNotBeEmpty,
                    &["stats/variance", "collection"],
                ));
            }

            let mut numbers: Vec<f64> = Vec::new();
            for item in items {
                match item {
                    Value::Integer(n) => numbers.push(*n as f64),
                    Value::Float(f) => numbers.push(*f),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::AllElementsMustBe,
                            &["stats/variance", "numbers"],
                        ))
                    }
                }
            }

            // 平均を計算
            let sum: f64 = numbers.iter().sum();
            let mean = sum / numbers.len() as f64;

            // 分散を計算
            let variance: f64 =
                numbers.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / numbers.len() as f64;

            Ok(Value::Float(variance))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["stats/variance", "lists or vectors"],
        )),
    }
}

/// stddev - 標準偏差
pub fn native_stddev(args: &[Value]) -> Result<Value, String> {
    match native_variance(args) {
        Ok(Value::Float(variance)) => Ok(Value::Float(variance.sqrt())),
        Ok(_) => unreachable!(),
        Err(e) => Err(e.replace("variance", "stddev")),
    }
}

/// percentile - パーセンタイル
/// 第1引数: コレクション、第2引数: パーセンタイル値(0-100)
pub fn native_percentile(args: &[Value]) -> Result<Value, String> {
    if args.len() != 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["stats/percentile"]));
    }

    let p = match &args[1] {
        Value::Integer(n) => {
            if *n < 0 || *n > 100 {
                return Err(fmt_msg(MsgKey::InvalidPercentile, &["stats/percentile"]));
            }
            *n as f64
        }
        Value::Float(f) => {
            if *f < 0.0 || *f > 100.0 {
                return Err(fmt_msg(MsgKey::InvalidPercentile, &["stats/percentile"]));
            }
            *f
        }
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["stats/percentile (percentile)", "numbers"],
            ))
        }
    };

    match &args[0] {
        Value::List(items) | Value::Vector(items) => {
            if items.is_empty() {
                return Err(fmt_msg(
                    MsgKey::MustNotBeEmpty,
                    &["stats/percentile", "collection"],
                ));
            }

            let mut numbers: Vec<f64> = Vec::new();
            for item in items {
                match item {
                    Value::Integer(n) => numbers.push(*n as f64),
                    Value::Float(f) => numbers.push(*f),
                    _ => {
                        return Err(fmt_msg(
                            MsgKey::AllElementsMustBe,
                            &["stats/percentile", "numbers"],
                        ))
                    }
                }
            }

            numbers.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // 線形補間法でパーセンタイルを計算
            let index = (p / 100.0) * (numbers.len() - 1) as f64;
            let lower = index.floor() as usize;
            let upper = index.ceil() as usize;

            let result = if lower == upper {
                numbers[lower]
            } else {
                let weight = index - lower as f64;
                numbers[lower] * (1.0 - weight) + numbers[upper] * weight
            };

            Ok(Value::Float(result))
        }
        _ => Err(fmt_msg(
            MsgKey::TypeOnly,
            &["stats/percentile", "lists or vectors"],
        )),
    }
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category stats
/// @qi-doc:functions mean, median, mode, stddev, variance, min, max, sum, product, percentile
pub const FUNCTIONS: super::NativeFunctions = &[
    ("stats/mean", native_mean),
    ("stats/median", native_median),
    ("stats/mode", native_mode),
    ("stats/variance", native_variance),
    ("stats/stddev", native_stddev),
    ("stats/percentile", native_percentile),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]
        .into();
        let result = native_mean(&[Value::Vector(data)]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_mean_mixed() {
        let data = vec![Value::Integer(1), Value::Float(2.5), Value::Integer(3)].into();
        let result = native_mean(&[Value::Vector(data)]).unwrap();
        assert!(
            (match result {
                Value::Float(f) => f,
                _ => 0.0,
            } - 2.166666)
                .abs()
                < 0.001
        );
    }

    #[test]
    fn test_median_odd() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]
        .into();
        let result = native_median(&[Value::Vector(data)]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_median_even() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]
        .into();
        let result = native_median(&[Value::Vector(data)]).unwrap();
        assert_eq!(result, Value::Float(2.5));
    }

    #[test]
    fn test_mode() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(3),
            Value::Integer(3),
        ]
        .into();
        let result = native_mode(&[Value::Vector(data)]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_variance() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]
        .into();
        let result = native_variance(&[Value::Vector(data)]).unwrap();
        assert_eq!(result, Value::Float(2.0));
    }

    #[test]
    fn test_stddev() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]
        .into();
        let result = native_stddev(&[Value::Vector(data)]).unwrap();
        let expected = 2.0_f64.sqrt();
        assert!(
            (match result {
                Value::Float(f) => f,
                _ => 0.0,
            } - expected)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn test_percentile_50() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]
        .into();
        let result = native_percentile(&[Value::Vector(data), Value::Integer(50)]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_percentile_95() {
        let data = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]
        .into();
        let result = native_percentile(&[Value::Vector(data), Value::Integer(95)]).unwrap();
        assert_eq!(result, Value::Float(4.8));
    }

    #[test]
    fn test_empty_collection_error() {
        let data = vec![].into();
        let result = native_mean(&[Value::Vector(data)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_numeric_error() {
        let data = vec![Value::Integer(1), Value::String("not a number".to_string())].into();
        let result = native_mean(&[Value::Vector(data)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_percentile_invalid_range() {
        let data = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into();
        let result = native_percentile(&[Value::Vector(data), Value::Integer(101)]);
        assert!(result.is_err());
    }
}
