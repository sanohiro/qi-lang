use super::*;

pub fn native_file_stream(args: &[Value]) -> Result<Value, String> {
    // 可変引数（1 or 2）のため、最小1つの引数が必要
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["file-stream"]));
    }

    let path = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err(fmt_msg(MsgKey::ArgMustBeType, &["file-stream", "a string"])),
    };

    // 第2引数でバイナリモード判定
    let is_bytes = args.len() >= 2 && matches!(&args[1], Value::Keyword(k) if k == "bytes");

    if is_bytes {
        create_file_byte_stream(&path)
    } else {
        create_file_line_stream(&path)
    }
}

/// ファイルを行ごとに読み込むストリーム（テキストモード）
fn create_file_line_stream(path: &str) -> Result<Value, String> {
    let file = File::open(path)
        .map_err(|e| fmt_msg(MsgKey::FileStreamFailedToOpen, &[path, &e.to_string()]))?;

    let reader = Arc::new(RwLock::new(BufReader::new(file)));

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut reader = reader.write();
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => None, // EOF
                Ok(_) => {
                    // 末尾の改行を削除
                    if line.ends_with('\n') {
                        line.pop();
                        if line.ends_with('\r') {
                            line.pop();
                        }
                    }
                    Some(Value::String(line))
                }
                Err(_) => None,
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// ファイルをバイトごとに読み込むストリーム（バイナリモード）
fn create_file_byte_stream(path: &str) -> Result<Value, String> {
    use std::io::Read;

    let file = File::open(path)
        .map_err(|e| fmt_msg(MsgKey::FileStreamFailedToOpen, &[path, &e.to_string()]))?;

    let reader = Arc::new(RwLock::new(BufReader::new(file)));
    const CHUNK_SIZE: usize = 4096; // 4KB chunks

    let stream = Stream {
        next_fn: Box::new(move || {
            let mut reader = reader.write();
            let mut buffer = vec![0u8; CHUNK_SIZE];
            match reader.read(&mut buffer) {
                Ok(0) => None, // EOF
                Ok(n) => {
                    buffer.truncate(n);
                    // バイト配列をIntegerのVectorに変換
                    let bytes: Vec<Value> =
                        buffer.iter().map(|&b| Value::Integer(b as i64)).collect();
                    Some(Value::Vector(bytes.into()))
                }
                Err(_) => None,
            }
        }),
    };

    Ok(Value::Stream(Arc::new(RwLock::new(stream))))
}

/// write-stream - ストリームをファイルに書き込み
/// 引数: (stream, path) - パイプライン対応
/// 使い方: (stream |> (io/write-stream "output.txt"))
/// ストリームの各要素を文字列に変換して改行付きで書き込む
pub fn native_write_stream(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "write-stream");

    let stream = match &args[0] {
        Value::Stream(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::ArgMustBeType,
                &["write-stream (1st arg)", "a stream"],
            ))
        }
    };

    let path = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["write-stream (2nd arg)", "string"],
            ))
        }
    };

    // ファイルを開く
    let mut file = fs::File::create(path)
        .map_err(|e| fmt_msg(MsgKey::WriteStreamFailedToCreate, &[path, &e.to_string()]))?;

    // ストリームの各要素をファイルに書き込み
    let mut count = 0;
    loop {
        let next_val = {
            let s = stream.read();
            (s.next_fn)()
        };

        match next_val {
            Some(val) => {
                let line = match &val {
                    Value::String(s) => s.clone(),
                    Value::Integer(n) => n.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Nil => String::from("nil"),
                    _ => format!("{:?}", val),
                };

                writeln!(file, "{}", line).map_err(|e| {
                    fmt_msg(MsgKey::WriteStreamFailedToWrite, &[path, &e.to_string()])
                })?;
                count += 1;
            }
            None => break,
        }
    }

    Ok(Value::Integer(count))
}

// ============================================
// ファイルシステム操作
// ============================================
