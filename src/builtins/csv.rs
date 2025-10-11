//! CSV処理関数
//! RFC 4180 準拠の CSV パーサー・シリアライザー

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::sync::Arc;

/// csv/parse - CSV文字列をパースして Vec<Vec<String>> に変換
pub fn native_csv_parse(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["csv/parse"]));
    }

    let csv_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["csv/parse", "string"])),
    };

    let records = parse_csv(csv_str)?;

    // Vec<Vec<String>> を Value::List(Vec<Value::List(Vec<Value::String>)>) に変換
    let result = records
        .into_iter()
        .map(|record| {
            Value::List(
                record
                    .into_iter()
                    .map(Value::String)
                    .collect()
            )
        })
        .collect();

    Ok(Value::List(result))
}

/// csv/stringify - Vec<Vec<Value>> を CSV文字列に変換
pub fn native_csv_stringify(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["csv/stringify"]));
    }

    let records = match &args[0] {
        Value::List(records) | Value::Vector(records) => records,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["csv/stringify", "list or vector"])),
    };

    // Value::List を Vec<Vec<String>> に変換
    let mut string_records = Vec::new();
    for record in records {
        match record {
            Value::List(fields) | Value::Vector(fields) => {
                let mut string_fields = Vec::new();
                for field in fields {
                    let s = match field {
                        Value::String(s) => s.clone(),
                        Value::Integer(n) => n.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Nil => String::new(),
                        _ => return Err(fmt_msg(MsgKey::CsvCannotSerialize, &[&format!("{:?}", field)])),
                    };
                    string_fields.push(s);
                }
                string_records.push(string_fields);
            }
            _ => return Err(fmt_msg(MsgKey::CsvRecordMustBeList, &[])),
        }
    }

    let csv_str = stringify_csv(&string_records);
    Ok(Value::String(csv_str))
}

/// csv/read-file - CSV ファイルを読み込んで Vec<Vec<String>> に変換
pub fn native_csv_read_file(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["csv/read-file"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["csv/read-file", "string"])),
    };

    let content = std::fs::read_to_string(path)
        .map_err(|e| fmt_msg(MsgKey::FileReadError, &[path, &e.to_string()]))?;

    // csv/parse と同じ処理
    let records = parse_csv(&content)?;

    let result = records
        .into_iter()
        .map(|record| {
            Value::List(
                record
                    .into_iter()
                    .map(Value::String)
                    .collect()
            )
        })
        .collect();

    Ok(Value::List(result))
}

/// csv/read-stream - CSV ファイルをストリームとして読み込み
/// 大きなCSVファイルを処理する際に、メモリを節約しながら行ごとに処理できる
pub fn native_csv_read_stream(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::Need1Arg, &["csv/read-stream"]));
    }

    let path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["csv/read-stream", "string"])),
    };

    // ファイルを読み込んでパース
    let content = std::fs::read_to_string(&path)
        .map_err(|e| fmt_msg(MsgKey::FileReadError, &[&path, &e.to_string()]))?;

    let records = parse_csv(&content)?;

    // レコードをイテレータで返すストリームを作成
    let records = Arc::new(RwLock::new(records.into_iter()));

    let stream = Stream {
        next_fn: Box::new(move || {
            records.write().next().map(|record| {
                Value::List(
                    record
                        .into_iter()
                        .map(Value::String)
                        .collect()
                )
            })
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// CSV文字列をパースする（RFC 4180準拠）
fn parse_csv(input: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut current_record = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes => {
                // クォート内でのダブルクォート
                if chars.peek() == Some(&'"') {
                    // "" はエスケープされたクォート
                    current_field.push('"');
                    chars.next(); // 次のクォートを消費
                } else {
                    // クォート終了
                    in_quotes = false;
                }
            }
            '"' if !in_quotes && current_field.is_empty() => {
                // フィールドの開始クォート
                in_quotes = true;
            }
            ',' if !in_quotes => {
                // フィールド区切り
                current_record.push(current_field.clone());
                current_field.clear();
            }
            '\r' if !in_quotes => {
                // CRLFの処理
                if chars.peek() == Some(&'\n') {
                    chars.next(); // LFを消費
                }
                // レコード終了
                current_record.push(current_field.clone());
                current_field.clear();
                if !current_record.is_empty() || !records.is_empty() {
                    records.push(current_record.clone());
                    current_record.clear();
                }
            }
            '\n' if !in_quotes => {
                // レコード終了
                current_record.push(current_field.clone());
                current_field.clear();
                if !current_record.is_empty() || !records.is_empty() {
                    records.push(current_record.clone());
                    current_record.clear();
                }
            }
            _ => {
                // 通常の文字
                current_field.push(ch);
            }
        }
    }

    // 最後のフィールドとレコードを追加
    if !current_field.is_empty() || !current_record.is_empty() || in_quotes {
        current_record.push(current_field);
        if !current_record.is_empty() {
            records.push(current_record);
        }
    }

    Ok(records)
}

/// Vec<Vec<String>> を CSV文字列に変換（RFC 4180準拠）
fn stringify_csv(records: &[Vec<String>]) -> String {
    let mut result = String::new();

    for (i, record) in records.iter().enumerate() {
        for (j, field) in record.iter().enumerate() {
            // フィールドにカンマ、ダブルクォート、改行が含まれている場合はクォートで囲む
            let needs_quoting = field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r');

            if needs_quoting {
                result.push('"');
                // ダブルクォートをエスケープ
                for ch in field.chars() {
                    if ch == '"' {
                        result.push('"');
                        result.push('"');
                    } else {
                        result.push(ch);
                    }
                }
                result.push('"');
            } else {
                result.push_str(field);
            }

            // フィールド区切り
            if j < record.len() - 1 {
                result.push(',');
            }
        }

        // レコード区切り
        if i < records.len() - 1 {
            result.push('\n');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parse_simple() {
        let csv = "a,b,c\n1,2,3";
        let result = native_csv_parse(&[Value::String(csv.to_string())]).unwrap();

        match result {
            Value::List(records) => {
                assert_eq!(records.len(), 2);
                match &records[0] {
                    Value::List(fields) => {
                        assert_eq!(fields.len(), 3);
                        assert_eq!(fields[0], Value::String("a".to_string()));
                        assert_eq!(fields[1], Value::String("b".to_string()));
                        assert_eq!(fields[2], Value::String("c".to_string()));
                    }
                    _ => panic!("Expected List"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_csv_parse_quoted_fields() {
        let csv = "\"a,b\",\"c\"\"d\",e\n1,2,3";
        let result = native_csv_parse(&[Value::String(csv.to_string())]).unwrap();

        match result {
            Value::List(records) => {
                assert_eq!(records.len(), 2);
                match &records[0] {
                    Value::List(fields) => {
                        assert_eq!(fields.len(), 3);
                        assert_eq!(fields[0], Value::String("a,b".to_string()));
                        assert_eq!(fields[1], Value::String("c\"d".to_string()));
                        assert_eq!(fields[2], Value::String("e".to_string()));
                    }
                    _ => panic!("Expected List"),
                }
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_csv_stringify_simple() {
        let data = Value::List(vec![
            Value::List(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string()),
            ]),
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ]),
        ]);

        let result = native_csv_stringify(&[data]).unwrap();
        assert_eq!(result, Value::String("a,b,c\n1,2,3".to_string()));
    }

    #[test]
    fn test_csv_stringify_quoted_fields() {
        let data = Value::List(vec![
            Value::List(vec![
                Value::String("a,b".to_string()),
                Value::String("c\"d".to_string()),
                Value::String("e\nf".to_string()),
            ]),
        ]);

        let result = native_csv_stringify(&[data]).unwrap();
        assert_eq!(result, Value::String("\"a,b\",\"c\"\"d\",\"e\nf\"".to_string()));
    }

    #[test]
    fn test_csv_round_trip() {
        let original = "a,b,c\n\"d,e\",\"f\"\"g\",h";
        let parsed = native_csv_parse(&[Value::String(original.to_string())]).unwrap();
        let stringified = native_csv_stringify(&[parsed]).unwrap();

        // 再度パースして比較
        let reparsed = native_csv_parse(&[stringified]).unwrap();
        let original_parsed = native_csv_parse(&[Value::String(original.to_string())]).unwrap();

        assert_eq!(reparsed, original_parsed);
    }
}
