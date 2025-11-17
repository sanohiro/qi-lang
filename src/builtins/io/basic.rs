use super::*;

pub fn native_read_file(args: &[Value]) -> Result<Value, String> {
    // 可変引数（1 + keyword args）のため、最小1つの引数が必要
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["read-file"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["read-file (1st arg)", "string"],
            ))
        }
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // エンコーディングオプションを取得
    let encoding_keyword = opts
        .get("encoding")
        .and_then(|v| match v {
            Value::Keyword(k) => Some(k.as_str()),
            _ => None,
        })
        .unwrap_or("utf-8");

    // ファイルをバイト列として読み込み
    let bytes =
        fs::read(path).map_err(|e| fmt_msg(MsgKey::IoFileError, &[path, &e.to_string()]))?;

    // エンコーディングに応じてデコード
    #[cfg(feature = "encoding-extended")]
    let content = if encoding_keyword == "auto" {
        auto_detect_encoding(&bytes, path)?
    } else {
        let (bytes_to_decode, _detected_encoding) = strip_bom(&bytes);

        if encoding_keyword == "utf-8" || encoding_keyword == "utf-8-bom" {
            // UTF-8の場合はBOMを自動除去
            String::from_utf8(bytes_to_decode.to_vec())
                .map_err(|_| fmt_msg(MsgKey::IoFailedToDecodeUtf8, &[path]))?
        } else {
            let encoding = resolve_encoding(encoding_keyword)?;
            decode_bytes(bytes_to_decode, encoding, path)?
        }
    };

    #[cfg(not(feature = "encoding-extended"))]
    let content = {
        // encoding-extendedがない場合はUTF-8のみサポート
        if encoding_keyword != "utf-8"
            && encoding_keyword != "utf-8-bom"
            && encoding_keyword != "auto"
        {
            return Err(fmt_msg(
                MsgKey::IoEncodingNotSupportedInMinimalBuild,
                &[encoding_keyword],
            ));
        }
        String::from_utf8(bytes).map_err(|_| fmt_msg(MsgKey::IoFailedToDecodeUtf8, &[path]))?
    };

    Ok(Value::String(content))
}

/// write-file - ファイルに書き込む（上書き）
/// 引数: (content, path) または (content, path :encoding :sjis :if-exists :error :create-dirs true)
/// パイプライン対応: (content |> (io/write-file "output.txt"))
///
/// サポートされるオプション:
///   :encoding :utf-8 (デフォルト)
///   :encoding :utf-8-bom (BOM付きUTF-8)
///   :encoding :sjis (Shift_JIS)
///   :encoding :euc-jp (EUC-JP)
///
///   :if-exists :overwrite (デフォルト、上書き)
///   :if-exists :error (存在したらエラー)
///   :if-exists :skip (存在したらスキップ)
///   :if-exists :append (追記)
///
///   :create-dirs true (ディレクトリを自動作成、デフォルトfalse)
pub fn native_write_file(args: &[Value]) -> Result<Value, String> {
    // 可変引数（2 + keyword args）のため、最小2つの引数が必要
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::Need2Args, &["write-file"]));
    }

    let content = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["write-file (1st arg - content)", "string"],
            ))
        }
    };

    let path = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::TypeOnly,
                &["write-file (2nd arg - path)", "string"],
            ))
        }
    };

    // キーワード引数を解析
    let opts = if args.len() > 2 {
        parse_keyword_args(args, 2)?
    } else {
        std::collections::HashMap::new()
    };

    // エンコーディングオプション
    let encoding_keyword = opts
        .get("encoding")
        .and_then(|v| match v {
            Value::Keyword(k) => Some(k.as_str()),
            _ => None,
        })
        .unwrap_or("utf-8");

    // if-existsオプション
    let if_exists = opts
        .get("if-exists")
        .and_then(|v| match v {
            Value::Keyword(k) => Some(k.as_str()),
            _ => None,
        })
        .unwrap_or("overwrite");

    // create-dirsオプション
    let create_dirs = opts
        .get("create-dirs")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);

    let path_obj = Path::new(path);

    // ディレクトリ自動作成
    if create_dirs {
        if let Some(parent) = path_obj.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    fmt_msg(
                        MsgKey::IoFailedToCreateDir,
                        &[&parent.display().to_string(), &e.to_string()],
                    )
                })?;
            }
        }
    }

    // ファイル存在チェック
    if path_obj.exists() {
        match if_exists {
            "error" => {
                return Err(fmt_msg(MsgKey::FileAlreadyExists, &[path]));
            }
            "skip" => {
                return Ok(Value::Nil);
            }
            "append" => {
                // 追記モード
                #[cfg(feature = "encoding-extended")]
                let bytes = {
                    let add_bom = encoding_keyword == "utf-8-bom";
                    let encoding = resolve_encoding(encoding_keyword)?;
                    encode_string(content, encoding, add_bom)
                };

                #[cfg(not(feature = "encoding-extended"))]
                let bytes = {
                    if encoding_keyword != "utf-8" && encoding_keyword != "utf-8-bom" {
                        return Err(fmt_msg(
                            MsgKey::IoEncodingNotSupportedInMinimalBuild,
                            &[encoding_keyword],
                        ));
                    }
                    content.as_bytes().to_vec()
                };

                let mut file = fs::OpenOptions::new()
                    .append(true)
                    .open(path)
                    .map_err(|e| {
                        fmt_msg(MsgKey::IoFailedToOpenForAppend, &[path, &e.to_string()])
                    })?;

                file.write_all(&bytes)
                    .map_err(|e| fmt_msg(MsgKey::IoFailedToAppend, &[path, &e.to_string()]))?;

                return Ok(Value::Nil);
            }
            "overwrite" => {
                // 上書き（デフォルト）
            }
            _ => {
                return Err(fmt_msg(MsgKey::InvalidIfExistsOption, &[if_exists]));
            }
        }
    }

    // エンコードして書き込み
    #[cfg(feature = "encoding-extended")]
    let bytes = {
        let add_bom = encoding_keyword == "utf-8-bom";
        let encoding = resolve_encoding(encoding_keyword)?;
        encode_string(content, encoding, add_bom)
    };

    #[cfg(not(feature = "encoding-extended"))]
    let bytes = {
        if encoding_keyword != "utf-8" && encoding_keyword != "utf-8-bom" {
            return Err(fmt_msg(
                MsgKey::IoEncodingNotSupportedInMinimalBuild,
                &[encoding_keyword],
            ));
        }
        content.as_bytes().to_vec()
    };

    fs::write(path, bytes)
        .map_err(|e| fmt_msg(MsgKey::IoFailedToWrite, &[path, &e.to_string()]))?;

    Ok(Value::Nil)
}

/// append-file - ファイルに追記
/// 引数: (content, path) - パイプライン対応
/// 使い方: (content |> (io/append-file "log.txt"))
pub fn native_append_file(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "append-file");

    match (&args[0], &args[1]) {
        (Value::String(content), Value::String(path)) => {
            match fs::OpenOptions::new().create(true).append(true).open(path) {
                Ok(mut file) => match file.write_all(content.as_bytes()) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(fmt_msg(
                        MsgKey::IoAppendFileFailedToWrite,
                        &[path, &e.to_string()],
                    )),
                },
                Err(e) => Err(fmt_msg(
                    MsgKey::IoAppendFileFailedToOpen,
                    &[path, &e.to_string()],
                )),
            }
        }
        _ => Err(fmt_msg(
            MsgKey::NeedNArgsDesc,
            &["append-file", "2", "(content: string, path: string)"],
        )),
    }
}

/// read-lines - ファイルを行ごとに読み込み
pub fn native_read_lines(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "read-lines");

    match &args[0] {
        Value::String(path) => match fs::read_to_string(path) {
            Ok(content) => {
                let lines: Vec<Value> = content
                    .lines()
                    .map(|line| Value::String(line.to_string()))
                    .collect();
                Ok(Value::List(lines.into()))
            }
            Err(e) => Err(fmt_msg(
                MsgKey::IoReadLinesFailedToRead,
                &[path, &e.to_string()],
            )),
        },
        _ => Err(fmt_msg(MsgKey::MustBeString, &["read-lines", "argument"])),
    }
}

/// file-exists? - ファイルの存在を確認
pub fn native_file_exists(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "file-exists?");

    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path).exists())),
        _ => Err(fmt_msg(MsgKey::MustBeString, &["file-exists?", "argument"])),
    }
}
