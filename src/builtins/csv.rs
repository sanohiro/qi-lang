//! CSV処理関数
//! RFC 4180 準拠の CSV パーサー・シリアライザー

use crate::check_args;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use parking_lot::RwLock;
use std::sync::Arc;

/// csv/parse - CSV文字列をパースして Vec<Vec<String>> に変換
/// 引数: (text [:delimiter delim])
/// 例: (csv/parse text :delimiter "\t")  ;; TSV
pub fn native_csv_parse(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 3 {
        return Err(fmt_msg(MsgKey::CsvParseNeed1Or3Args, &[]));
    }

    let csv_str = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["csv/parse", "string"])),
    };

    // オプショナル delimiter引数
    let delimiter = if args.len() == 3 {
        // :delimiter "\t" 形式
        match (&args[1], &args[2]) {
            (Value::Keyword(k), Value::String(d)) if &**k == "delimiter" => {
                if d.chars().count() != 1 {
                    return Err(fmt_msg(MsgKey::CsvDelimiterMustBeSingleChar, &[]));
                }
                // count() == 1 をチェック済みのため安全
                #[allow(clippy::expect_used)]
                d.chars()
                    .next()
                    .expect("delimiter must have exactly one character")
            }
            _ => return Err(fmt_msg(MsgKey::CsvInvalidDelimiterArg, &[])),
        }
    } else {
        ',' // デフォルト
    };

    let records = parse_csv_with_delimiter(csv_str, delimiter)?;

    // Vec<Vec<String>> を Value::List(Vec<Value::List(Vec<Value::String>)>) に変換
    let result = records
        .into_iter()
        .map(|record| Value::List(record.into_iter().map(Value::String).collect()))
        .collect();

    Ok(Value::List(result))
}

/// csv/stringify - Vec<Vec<Value>> を CSV文字列に変換
pub fn native_csv_stringify(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "csv/stringify");

    let records = match &args[0] {
        Value::List(records) | Value::Vector(records) => records,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["csv/stringify", "list or vector"],
            ))
        }
    };

    // Value::List を Vec<Vec<String>> に変換
    let mut string_records = Vec::with_capacity(records.len());
    for record in records {
        match record {
            Value::List(fields) | Value::Vector(fields) => {
                let mut string_fields = Vec::with_capacity(fields.len());
                for field in fields {
                    let s = match field {
                        Value::String(s) => s.clone(),
                        Value::Integer(n) => n.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Nil => String::new(),
                        _ => {
                            return Err(fmt_msg(
                                MsgKey::CsvCannotSerialize,
                                &[&format!("{:?}", field)],
                            ))
                        }
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
    check_args!(args, 1, "csv/read-file");

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
        .map(|record| Value::List(record.into_iter().map(Value::String).collect()))
        .collect();

    Ok(Value::List(result))
}

/// csv/read-stream - CSV ファイルをストリームとして読み込み
///
/// **真のストリーミング実装**: 行ごとにメモリに読み込み、
/// 大きなファイルでもメモリ使用量を抑えます。
pub fn native_csv_read_stream(args: &[Value]) -> Result<Value, String> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    check_args!(args, 1, "csv/read-stream");

    let path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::TypeOnly, &["csv/read-stream", "string"])),
    };

    // ファイルをバッファリングして開く（ストリーミング）
    let file =
        File::open(&path).map_err(|e| fmt_msg(MsgKey::FileReadError, &[&path, &e.to_string()]))?;
    let reader = BufReader::new(file);
    let lines = Arc::new(parking_lot::Mutex::new(reader.lines()));

    // ストリームを作成（行ごとに処理）
    let stream = Stream {
        next_fn: Box::new(move || {
            // 次の行を読み取り、CSVとしてパース
            match lines.lock().next() {
                Some(Ok(line)) => {
                    // 1行をCSVとしてパース
                    match parse_csv_with_delimiter(&line, ',') {
                        Ok(mut records) if !records.is_empty() => {
                            // 最初のレコードを取得（1行につき1レコード）
                            let record = records.remove(0);
                            Some(Value::List(record.into_iter().map(Value::String).collect()))
                        }
                        Ok(_) => None,  // 空行
                        Err(_) => None, // パースエラーはスキップ（エラーを返すとストリーム全体が停止）
                    }
                }
                Some(Err(_)) => None, // I/Oエラーはスキップ
                None => None,         // EOF
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// csv/write-file - データをCSV形式でファイルに書き込み
/// 引数: (data, path) - パイプライン対応
/// 使い方: (data |> (csv/write-file "output.csv"))
/// csv/stringify + io/write-file の便利関数
pub fn native_csv_write_file(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "csv/write-file");

    // データをCSV文字列に変換
    let csv_str = native_csv_stringify(&[args[0].clone()])?;

    // ファイルに書き込み
    let path = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["csv/write-file (2nd arg)", "string"],
            ))
        }
    };

    std::fs::write(
        path,
        match csv_str {
            Value::String(s) => s,
            _ => return Err(fmt_msg(MsgKey::CsvWriteFileStringifyFailed, &[])),
        },
    )
    .map_err(|e| fmt_msg(MsgKey::CsvWriteFileFailedToWrite, &[path, &e.to_string()]))?;

    Ok(Value::Nil)
}

/// CSV文字列をパースする（RFC 4180準拠）
fn parse_csv(input: &str) -> Result<Vec<Vec<String>>, String> {
    parse_csv_with_delimiter(input, ',')
}

/// CSV文字列を指定されたdelimiterでパースする
fn parse_csv_with_delimiter(input: &str, delimiter: char) -> Result<Vec<Vec<String>>, String> {
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
            ch if ch == delimiter && !in_quotes => {
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
    stringify_csv_with_delimiter(records, ',')
}

/// Vec<Vec<String>> を指定されたdelimiterでCSV文字列に変換
fn stringify_csv_with_delimiter(records: &[Vec<String>], delimiter: char) -> String {
    let mut result = String::new();

    for (i, record) in records.iter().enumerate() {
        for (j, field) in record.iter().enumerate() {
            // フィールドにdelimiter、ダブルクォート、改行が含まれている場合はクォートで囲む
            let needs_quoting = field.contains(delimiter)
                || field.contains('"')
                || field.contains('\n')
                || field.contains('\r');

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
                result.push(delimiter);
            }
        }

        // レコード区切り
        if i < records.len() - 1 {
            result.push('\n');
        }
    }

    result
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category data/csv
/// @qi-doc:functions parse, stringify, read-file, write-file, read-stream
pub const FUNCTIONS: super::NativeFunctions = &[
    ("csv/parse", native_csv_parse),
    ("csv/stringify", native_csv_stringify),
    ("csv/read-file", native_csv_read_file),
    ("csv/write-file", native_csv_write_file),
    ("csv/read-stream", native_csv_read_stream),
];

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
        let data = Value::List(
            vec![
                Value::List(
                    vec![
                        Value::String("a".to_string()),
                        Value::String("b".to_string()),
                        Value::String("c".to_string()),
                    ]
                    .into(),
                ),
                Value::List(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into()),
            ]
            .into(),
        );

        let result = native_csv_stringify(&[data]).unwrap();
        assert_eq!(result, Value::String("a,b,c\n1,2,3".to_string()));
    }

    #[test]
    fn test_csv_stringify_quoted_fields() {
        let data = Value::List(
            vec![Value::List(
                vec![
                    Value::String("a,b".to_string()),
                    Value::String("c\"d".to_string()),
                    Value::String("e\nf".to_string()),
                ]
                .into(),
            )]
            .into(),
        );

        let result = native_csv_stringify(&[data]).unwrap();
        assert_eq!(
            result,
            Value::String("\"a,b\",\"c\"\"d\",\"e\nf\"".to_string())
        );
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
