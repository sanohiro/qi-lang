use super::*;

/// list-dir - ディレクトリ内のファイル一覧を取得
///
/// 指定されたディレクトリ内のファイルとディレクトリを列挙します。
/// グロブパターンでフィルタリングできます。
///
/// # 引数
/// - `path: string` - 列挙するディレクトリのパス
/// - `:pattern string` - グロブパターン（オプション、デフォルト: *）
/// - `:recursive bool` - サブディレクトリも検索するか（オプション、デフォルト: false）
///
/// # 戻り値
/// `list` - マッチしたファイルパスの配列
///
/// # 使用例
/// ```qi
/// (io/list-dir "./src" :pattern "*.rs")
/// (io/list-dir "./src" :recursive true)
/// ```
pub fn native_list_dir(args: &[Value]) -> Result<Value, String> {
    // 可変引数（1 + keyword args）のため、最小1つの引数が必要
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["io/list-dir", "1"]));
    }

    let dir_path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["io/list-dir", "a string"],
            ))
        }
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // パターンオプション
    let pattern = opts.get("pattern").and_then(|v| match v {
        Value::String(s) => Some(s.as_str()),
        _ => None,
    });

    // recursiveオプション
    let recursive = opts
        .get("recursive")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);

    // グロブパターンの構築
    let glob_pattern = if let Some(pat) = pattern {
        if recursive {
            format!("{}/**/{}", dir_path, pat)
        } else {
            format!("{}/{}", dir_path, pat)
        }
    } else if recursive {
        format!("{}/**/*", dir_path)
    } else {
        format!("{}/*", dir_path)
    };

    // グロブでファイル一覧を取得
    let entries: Result<Vec<Value>, String> = glob::glob(&glob_pattern)
        .map_err(|e| {
            fmt_msg(
                MsgKey::IoListDirInvalidPattern,
                &[&glob_pattern, &e.to_string()],
            )
        })?
        .map(|entry| {
            entry
                .map(|path| Value::String(path.to_string_lossy().to_string()))
                .map_err(|e| fmt_msg(MsgKey::IoListDirFailedToRead, &[&e.to_string()]))
        })
        .collect();

    Ok(Value::List(entries?.into()))
}

/// create-dir - ディレクトリを作成
/// 引数: (path) または (path :parents true)
/// オプション:
///   :parents - 親ディレクトリも作成するか（デフォルト: true）
pub fn native_create_dir(args: &[Value]) -> Result<Value, String> {
    // 可変引数（1 + keyword args）のため、最小1つの引数が必要
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["io/create-dir", "1"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["io/create-dir", "a string"],
            ))
        }
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // parentsオプション（デフォルトtrue）
    let parents = opts
        .get("parents")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(true);

    if parents {
        fs::create_dir_all(path)
            .map_err(|e| fmt_msg(MsgKey::IoCreateDirFailed, &[path, &e.to_string()]))?;
    } else {
        fs::create_dir(path)
            .map_err(|e| fmt_msg(MsgKey::IoCreateDirFailed, &[path, &e.to_string()]))?;
    }

    Ok(Value::Nil)
}

/// delete-file - ファイルを削除
/// 引数: (path)
pub fn native_delete_file(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "io/delete-file");

    match &args[0] {
        Value::String(path) => {
            fs::remove_file(path)
                .map_err(|e| fmt_msg(MsgKey::IoDeleteFileFailed, &[path, &e.to_string()]))?;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(
            MsgKey::ArgMustBeType,
            &["io/delete-file", "a string"],
        )),
    }
}

/// delete-dir - ディレクトリを削除
/// 引数: (path) または (path :recursive true)
/// オプション:
///   :recursive - 中身ごと削除するか（デフォルト: false）
pub fn native_delete_dir(args: &[Value]) -> Result<Value, String> {
    // 可変引数（1 + keyword args）のため、最小1つの引数が必要
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["io/delete-dir", "1"]));
    }

    let path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["io/delete-dir", "a string"],
            ))
        }
    };

    // キーワード引数を解析
    let opts = if args.len() > 1 {
        parse_keyword_args(args, 1)?
    } else {
        std::collections::HashMap::new()
    };

    // recursiveオプション
    let recursive = opts
        .get("recursive")
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false);

    if recursive {
        fs::remove_dir_all(path)
            .map_err(|e| fmt_msg(MsgKey::IoDeleteDirFailed, &[path, &e.to_string()]))?;
    } else {
        fs::remove_dir(path)
            .map_err(|e| fmt_msg(MsgKey::IoDeleteDirFailed, &[path, &e.to_string()]))?;
    }

    Ok(Value::Nil)
}

/// copy-file - ファイルをコピー
/// 引数: (src, dst)
pub fn native_copy_file(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "io/copy-file");

    match (&args[0], &args[1]) {
        (Value::String(src), Value::String(dst)) => {
            fs::copy(src, dst)
                .map_err(|e| fmt_msg(MsgKey::IoCopyFileFailed, &[src, dst, &e.to_string()]))?;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::BothArgsMustBeStrings, &["io/copy-file"])),
    }
}

/// move-file - ファイルを移動（名前変更）
/// 引数: (src, dst)
pub fn native_move_file(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 2, "io/move-file");

    match (&args[0], &args[1]) {
        (Value::String(src), Value::String(dst)) => {
            fs::rename(src, dst)
                .map_err(|e| fmt_msg(MsgKey::IoMoveFileFailed, &[src, dst, &e.to_string()]))?;
            Ok(Value::Nil)
        }
        _ => Err(fmt_msg(MsgKey::BothArgsMustBeStrings, &["io/move-file"])),
    }
}

/// file-info - ファイル/ディレクトリのメタデータを取得
/// 引数: (path)
/// 戻り値: {:size 1024 :modified "2024-01-01" :is-dir false :is-file true}
pub fn native_file_info(args: &[Value]) -> Result<Value, String> {
    use std::time::UNIX_EPOCH;

    check_args!(args, 1, "io/file-info");

    match &args[0] {
        Value::String(path) => {
            let metadata = fs::metadata(path)
                .map_err(|e| fmt_msg(MsgKey::IoGetMetadataFailed, &[path, &e.to_string()]))?;

            let mut info = crate::new_hashmap();

            // サイズ（u64→i64オーバーフローチェック）
            let size = metadata.len();
            if size > i64::MAX as u64 {
                return Err(fmt_msg(
                    MsgKey::ValueTooLargeForI64,
                    &["io/metadata", &size.to_string()],
                ));
            }
            info.insert(
                crate::value::MapKey::String("size".to_string()),
                Value::Integer(size as i64),
            );

            // ファイルタイプ
            info.insert(
                crate::value::MapKey::String("is-dir".to_string()),
                Value::Bool(metadata.is_dir()),
            );
            info.insert(
                crate::value::MapKey::String("is-file".to_string()),
                Value::Bool(metadata.is_file()),
            );

            // 更新日時（u64→i64オーバーフローチェック）
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    let secs = duration.as_secs();
                    if secs > i64::MAX as u64 {
                        return Err(fmt_msg(
                            MsgKey::ValueTooLargeForI64,
                            &["io/metadata timestamp", &secs.to_string()],
                        ));
                    }
                    info.insert(
                        crate::value::MapKey::String("modified".to_string()),
                        Value::Integer(secs as i64),
                    );
                }
            }

            Ok(Value::Map(info))
        }
        _ => Err(fmt_msg(
            MsgKey::ArgMustBeType,
            &["io/file-info", "a string"],
        )),
    }
}

/// is-file? - ファイルかどうか判定
/// 引数: (path)
pub fn native_is_file(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "io/is-file?");

    match &args[0] {
        Value::String(path) => {
            let path_obj = Path::new(path);
            Ok(Value::Bool(path_obj.is_file()))
        }
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &["io/is-file?", "a string"])),
    }
}

/// is-dir? - ディレクトリかどうか判定
/// 引数: (path)
pub fn native_is_dir(args: &[Value]) -> Result<Value, String> {
    check_args!(args, 1, "io/is-dir?");

    match &args[0] {
        Value::String(path) => {
            let path_obj = Path::new(path);
            Ok(Value::Bool(path_obj.is_dir()))
        }
        _ => Err(fmt_msg(MsgKey::ArgMustBeType, &["io/is-dir?", "a string"])),
    }
}

// ========================================
// 標準入力関数
// ========================================
