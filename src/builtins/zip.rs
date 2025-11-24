//! ZIP圧縮・解凍・gzip関数
//!
//! このモジュールは `util-zip` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::map_i18n_err;
use crate::value::Value;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

/// create - ZIPファイルを作成
/// 引数: (zip-path files...) - ZIP出力パス、追加するファイル/ディレクトリのリスト
/// 例: (zip/create "backup.zip" "data.txt" "config.json")
///     (zip/create "archive.zip" "logs/")
pub fn native_zip_create(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["zip/create", "2"]));
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["zip/create", "a string"])),
    };

    // ZIPファイル作成
    let file = map_i18n_err!(
        File::create(zip_path),
        MsgKey::ZipCreateFileFailed,
        "zip/create",
        zip_path
    )?;
    let mut zip = ZipWriter::new(file);

    // 各ファイルを追加
    for arg in &args[1..] {
        let path_str = match arg {
            Value::String(s) => s,
            _ => return Err(fmt_msg(MsgKey::AllPathsMustBeStrings, &["zip/create"])),
        };

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(fmt_msg(
                MsgKey::ZipPathDoesNotExist,
                &["zip/create", path_str],
            ));
        }

        if path.is_file() {
            add_file_to_zip(&mut zip, path, path)?;
        } else if path.is_dir() {
            add_dir_to_zip(&mut zip, path, path)?;
        }
    }

    map_i18n_err!(zip.finish(), MsgKey::ZipFinishFailed, "zip/create")?;

    Ok(Value::String(zip_path.clone()))
}

/// ZIPにファイルを追加
fn add_file_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    file_path: &Path,
    base_path: &Path,
) -> Result<(), String> {
    let name = file_path
        .strip_prefix(base_path.parent().unwrap_or(Path::new("")))
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    // ファイルのメタデータを取得してパーミッションを保持
    let metadata = map_i18n_err!(
        fs::metadata(file_path),
        MsgKey::ZipReadFileFailed,
        "zip/create",
        &file_path.display().to_string()
    )?;

    #[cfg(unix)]
    let permissions = {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o777
    };
    #[cfg(not(unix))]
    let permissions = if metadata.permissions().readonly() {
        0o444 // r--r--r--
    } else {
        0o644 // rw-r--r--
    };

    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(permissions);

    map_i18n_err!(
        zip.start_file(name, options),
        MsgKey::ZipStartFileFailed,
        "zip/create"
    )?;

    let f = map_i18n_err!(
        File::open(file_path),
        MsgKey::ZipOpenFileFailed,
        "zip/create",
        &file_path.display().to_string()
    )?;

    // Zip bomb攻撃対策: ファイルサイズ制限（500MB）
    const MAX_FILE_SIZE: usize = 500 * 1024 * 1024; // 500MB
    let mut buffer = Vec::new();
    let mut limited_reader = f.take(MAX_FILE_SIZE as u64);
    map_i18n_err!(
        limited_reader.read_to_end(&mut buffer),
        MsgKey::ZipReadFileFailed,
        "zip/create",
        &file_path.display().to_string()
    )?;

    if buffer.len() >= MAX_FILE_SIZE {
        return Err(fmt_msg(
            MsgKey::ZipFileTooLarge,
            &["zip/create", &file_path.display().to_string(), "500"],
        ));
    }

    map_i18n_err!(zip.write_all(&buffer), MsgKey::ZipWriteFailed, "zip/create")?;

    Ok(())
}

/// ZIPにディレクトリを再帰的に追加
fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    dir_path: &Path,
    base_path: &Path,
) -> Result<(), String> {
    // ディレクトリエントリを追加（空ディレクトリも保存されるように）
    if let Ok(relative_path) = dir_path.strip_prefix(base_path) {
        if !relative_path.as_os_str().is_empty() {
            let dir_name = relative_path.to_string_lossy().to_string() + "/";

            // ディレクトリのパーミッションを取得
            #[cfg(unix)]
            let permissions = {
                use std::os::unix::fs::PermissionsExt;
                match fs::metadata(dir_path) {
                    Ok(metadata) => metadata.permissions().mode() & 0o777,
                    Err(_) => 0o755, // デフォルト
                }
            };
            #[cfg(not(unix))]
            let permissions = 0o755;

            let options = FileOptions::<()>::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(permissions);

            map_i18n_err!(
                zip.add_directory(&dir_name, options),
                MsgKey::ZipStartFileFailed,
                "zip/create"
            )?;
        }
    }

    let entries = map_i18n_err!(
        fs::read_dir(dir_path),
        MsgKey::ZipReadDirFailed,
        "zip/create",
        &dir_path.display().to_string()
    )?;

    for entry in entries {
        let entry = map_i18n_err!(entry, MsgKey::ZipReadEntryFailed, "zip/create", "?")?;
        let path = entry.path();

        if path.is_file() {
            add_file_to_zip(zip, &path, base_path)?;
        } else if path.is_dir() {
            add_dir_to_zip(zip, &path, base_path)?;
        }
    }

    Ok(())
}

/// extract - ZIPファイルを解凍
/// 引数: (zip-path [dest-dir]) - ZIP入力パス、展開先ディレクトリ（デフォルト: カレント）
/// 例: (zip/extract "backup.zip")
///     (zip/extract "archive.zip" "output/")
pub fn native_zip_extract(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["zip/extract"]));
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::FirstArgMustBe,
                &["zip/extract", "a string"],
            ))
        }
    };

    let dest_dir = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => PathBuf::from(s),
            _ => {
                return Err(fmt_msg(
                    MsgKey::SecondArgMustBe,
                    &["zip/extract", "a string"],
                ))
            }
        }
    } else {
        PathBuf::from(".")
    };

    // ZIPファイルを開く
    let file = map_i18n_err!(
        File::open(zip_path),
        MsgKey::ZipOpenFileFailed,
        "zip/extract",
        zip_path
    )?;
    let mut archive = map_i18n_err!(
        ZipArchive::new(file),
        MsgKey::ZipReadFileFailed,
        "zip/extract",
        zip_path
    )?;

    // 展開先ディレクトリを作成
    map_i18n_err!(
        fs::create_dir_all(&dest_dir),
        MsgKey::ZipCreateDirFailed,
        "zip/extract",
        &dest_dir.display().to_string()
    )?;

    // 各ファイルを展開
    for i in 0..archive.len() {
        let mut file = map_i18n_err!(
            archive.by_index(i),
            MsgKey::ZipReadEntryFailed,
            "zip/extract",
            &i.to_string()
        )?;

        // enclosed_name()を使用してパストラバーサル攻撃を防ぐ
        let safe_path = file
            .enclosed_name()
            .ok_or_else(|| fmt_msg(MsgKey::ZipUnsafePath, &["zip/extract", file.name()]))?;

        let outpath = dest_dir.join(safe_path);

        if file.is_dir() {
            map_i18n_err!(
                fs::create_dir_all(&outpath),
                MsgKey::ZipCreateDirFailed,
                "zip/extract",
                &outpath.display().to_string()
            )?;
        } else {
            if let Some(parent) = outpath.parent() {
                map_i18n_err!(
                    fs::create_dir_all(parent),
                    MsgKey::ZipCreateParentDirFailed,
                    "zip/extract"
                )?;
            }
            let mut outfile = map_i18n_err!(
                File::create(&outpath),
                MsgKey::ZipCreateDirFailed,
                "zip/extract",
                &outpath.display().to_string()
            )?;
            map_i18n_err!(
                std::io::copy(&mut file, &mut outfile),
                MsgKey::ZipExtractFailed,
                "zip/extract"
            )?;
        }
    }

    Ok(Value::String(dest_dir.to_string_lossy().to_string()))
}

/// list - ZIP内容一覧を取得
/// 引数: (zip-path) - ZIP入力パス
/// 例: (zip/list "backup.zip")
pub fn native_zip_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err(fmt_msg(MsgKey::NeedExactlyNArgs, &["zip/list", "1"]));
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["zip/list", "a string"])),
    };

    let file = map_i18n_err!(
        File::open(zip_path),
        MsgKey::ZipOpenFileFailed,
        "zip/list",
        zip_path
    )?;
    let mut archive = map_i18n_err!(
        ZipArchive::new(file),
        MsgKey::ZipReadFileFailed,
        "zip/list",
        zip_path
    )?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let file = map_i18n_err!(
            archive.by_index(i),
            MsgKey::ZipReadEntryFailed,
            "zip/list",
            &i.to_string()
        )?;

        let mut info = crate::new_hashmap();
        info.insert(
            crate::value::MapKey::String("name".to_string()),
            Value::String(file.name().to_string()),
        );
        info.insert(
            crate::value::MapKey::String("size".to_string()),
            Value::Integer(file.size() as i64),
        );
        info.insert(
            crate::value::MapKey::String("compressed-size".to_string()),
            Value::Integer(file.compressed_size() as i64),
        );
        info.insert(
            crate::value::MapKey::String("is-dir".to_string()),
            Value::Bool(file.is_dir()),
        );

        entries.push(Value::Map(info));
    }

    Ok(Value::List(entries.into()))
}

/// add - 既存ZIPにファイルを追加
/// 引数: (zip-path files...) - ZIP入力パス、追加するファイルのリスト
/// 例: (zip/add "backup.zip" "new-file.txt")
pub fn native_zip_add(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err(fmt_msg(MsgKey::NeedAtLeastNArgs, &["zip/add", "2"]));
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["zip/add", "a string"])),
    };

    // 既存ZIPを読み込み
    let zip_file = map_i18n_err!(
        File::open(zip_path),
        MsgKey::ZipOpenFileFailed,
        "zip/add",
        zip_path
    )?;
    let mut archive = map_i18n_err!(
        ZipArchive::new(zip_file),
        MsgKey::ZipReadFileFailed,
        "zip/add",
        zip_path
    )?;

    // 一時ファイルに書き込み
    let mut temp_path = PathBuf::from(zip_path);
    temp_path.as_mut_os_string().push(".tmp");
    let temp_file = map_i18n_err!(
        File::create(&temp_path),
        MsgKey::ZipCreateTempFailed,
        "zip/add"
    )?;
    let mut zip = ZipWriter::new(temp_file);

    // 既存ファイルをコピー（パーミッション保持）
    for i in 0..archive.len() {
        let file = map_i18n_err!(
            archive.by_index(i),
            MsgKey::ZipReadEntryFailed,
            "zip/add",
            &i.to_string()
        )?;

        // 既存ファイルのパーミッションを保持
        let permissions = file.unix_mode().unwrap_or(0o644);
        let options = FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(permissions);

        // ファイル名を先に取得（take()で所有権が移動する前）
        let file_name = file.name().to_string();

        map_i18n_err!(
            zip.start_file(&file_name, options),
            MsgKey::ZipStartFileFailed,
            "zip/add"
        )?;

        // Zip bomb攻撃対策: ファイルサイズ制限（500MB）
        const MAX_FILE_SIZE: usize = 500 * 1024 * 1024; // 500MB
        let mut buffer = Vec::new();
        let mut limited_reader = file.take(MAX_FILE_SIZE as u64);
        map_i18n_err!(
            limited_reader.read_to_end(&mut buffer),
            MsgKey::ZipReadFileFailed,
            "zip/add",
            "?"
        )?;

        if buffer.len() >= MAX_FILE_SIZE {
            return Err(fmt_msg(
                MsgKey::ZipFileTooLarge,
                &["zip/add", &file_name, "500"],
            ));
        }

        map_i18n_err!(zip.write_all(&buffer), MsgKey::ZipWriteFailed, "zip/add")?;
    }

    // 新しいファイルを追加
    for arg in &args[1..] {
        let path_str = match arg {
            Value::String(s) => s,
            _ => return Err(fmt_msg(MsgKey::AllPathsMustBeStrings, &["zip/add"])),
        };

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(fmt_msg(MsgKey::ZipPathDoesNotExist, &["zip/add", path_str]));
        }

        if path.is_file() {
            add_file_to_zip(&mut zip, path, path)?;
        } else if path.is_dir() {
            add_dir_to_zip(&mut zip, path, path)?;
        }
    }

    map_i18n_err!(zip.finish(), MsgKey::ZipFinishFailed, "zip/add")?;

    // 元のファイルを置き換え
    map_i18n_err!(
        fs::remove_file(zip_path),
        MsgKey::ZipRemoveOriginalFailed,
        "zip/add"
    )?;
    map_i18n_err!(
        fs::rename(&temp_path, zip_path),
        MsgKey::ZipRenameTempFailed,
        "zip/add"
    )?;

    Ok(Value::String(zip_path.clone()))
}

/// gzip - ファイルをgzip圧縮
/// 引数: (input-path [output-path]) - 入力ファイル、出力ファイル（省略時: 入力.gz）
/// 例: (zip/gzip "data.txt")                ;; => "data.txt.gz"
///     (zip/gzip "data.txt" "backup.gz")    ;; => "backup.gz"
pub fn native_gzip(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["zip/gzip"]));
    }

    let input_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["zip/gzip", "a string"])),
    };

    let output_path = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => PathBuf::from(s),
            _ => return Err(fmt_msg(MsgKey::SecondArgMustBe, &["zip/gzip", "a string"])),
        }
    } else {
        let mut path = PathBuf::from(input_path);
        path.as_mut_os_string().push(".gz");
        path
    };

    // 入力ファイルを読み込み
    let mut input = map_i18n_err!(
        File::open(input_path),
        MsgKey::ZipOpenFileFailed,
        "zip/gzip",
        input_path
    )?;

    // 出力ファイルを作成
    let output = map_i18n_err!(
        File::create(&output_path),
        MsgKey::ZipCreateDirFailed,
        "zip/gzip",
        &output_path.display().to_string()
    )?;

    // gzip圧縮
    let mut encoder = GzEncoder::new(output, Compression::default());
    map_i18n_err!(
        std::io::copy(&mut input, &mut encoder),
        MsgKey::ZipCompressFailed,
        "zip/gzip"
    )?;

    map_i18n_err!(encoder.finish(), MsgKey::ZipFinishFailed, "zip/gzip")?;

    Ok(Value::String(output_path.to_string_lossy().to_string()))
}

/// gunzip - gzipファイルを解凍
/// 引数: (input-path [output-path]) - 入力.gzファイル、出力ファイル（省略時: .gz除去）
/// 例: (zip/gunzip "data.txt.gz")             ;; => "data.txt"
///     (zip/gunzip "backup.gz" "data.txt")    ;; => "data.txt"
pub fn native_gunzip(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err(fmt_msg(MsgKey::Need1Or2Args, &["zip/gunzip"]));
    }

    let input_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err(fmt_msg(MsgKey::FirstArgMustBe, &["zip/gunzip", "a string"])),
    };

    let output_path = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => PathBuf::from(s),
            _ => {
                return Err(fmt_msg(
                    MsgKey::SecondArgMustBe,
                    &["zip/gunzip", "a string"],
                ))
            }
        }
    } else {
        // .gz拡張子を除去
        let path = PathBuf::from(input_path);
        if input_path.ends_with(".gz") {
            path.with_extension("")
        } else {
            let mut p = path;
            p.as_mut_os_string().push(".decompressed");
            p
        }
    };

    // 入力ファイルを読み込み
    let input = map_i18n_err!(
        File::open(input_path),
        MsgKey::ZipOpenFileFailed,
        "zip/gunzip",
        input_path
    )?;

    // gzip解凍
    let mut decoder = GzDecoder::new(input);

    // 出力ファイルに書き込み
    let mut output = map_i18n_err!(
        File::create(&output_path),
        MsgKey::ZipCreateDirFailed,
        "zip/gunzip",
        &output_path.display().to_string()
    )?;

    map_i18n_err!(
        std::io::copy(&mut decoder, &mut output),
        MsgKey::ZipDecompressFailed,
        "zip/gunzip"
    )?;

    Ok(Value::String(output_path.to_string_lossy().to_string()))
}

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category zip
/// @qi-doc:functions create, extract, list, gzip, gunzip
pub const FUNCTIONS: super::NativeFunctions = &[
    ("zip/create", native_zip_create),
    ("zip/extract", native_zip_extract),
    ("zip/list", native_zip_list),
    ("zip/add", native_zip_add),
    ("zip/gzip", native_gzip),
    ("zip/gunzip", native_gunzip),
];
