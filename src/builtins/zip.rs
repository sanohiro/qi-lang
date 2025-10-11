//! ZIP圧縮・解凍・gzip関数

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
        return Err("zip/create: at least 2 arguments required (zip-path files...)".to_string());
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("zip/create: zip-path must be a string".to_string()),
    };

    // ZIPファイル作成
    let file = File::create(zip_path)
        .map_err(|e| format!("zip/create: failed to create zip file '{}': {}", zip_path, e))?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 各ファイルを追加
    for arg in &args[1..] {
        let path_str = match arg {
            Value::String(s) => s,
            _ => return Err("zip/create: all file paths must be strings".to_string()),
        };

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("zip/create: path '{}' does not exist", path_str));
        }

        if path.is_file() {
            add_file_to_zip(&mut zip, path, path, &options)?;
        } else if path.is_dir() {
            add_dir_to_zip(&mut zip, path, path, &options)?;
        }
    }

    zip.finish()
        .map_err(|e| format!("zip/create: failed to finish zip: {}", e))?;

    Ok(Value::String(zip_path.clone()))
}

/// ZIPにファイルを追加
fn add_file_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    file_path: &Path,
    base_path: &Path,
    options: &FileOptions<()>,
) -> Result<(), String> {
    let name = file_path
        .strip_prefix(base_path.parent().unwrap_or(Path::new("")))
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    zip.start_file(name, *options)
        .map_err(|e| format!("zip/create: failed to start file: {}", e))?;

    let mut f = File::open(file_path)
        .map_err(|e| format!("zip/create: failed to open file '{}': {}", file_path.display(), e))?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .map_err(|e| format!("zip/create: failed to read file '{}': {}", file_path.display(), e))?;

    zip.write_all(&buffer)
        .map_err(|e| format!("zip/create: failed to write file to zip: {}", e))?;

    Ok(())
}

/// ZIPにディレクトリを再帰的に追加
fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    dir_path: &Path,
    base_path: &Path,
    options: &FileOptions<()>,
) -> Result<(), String> {
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("zip/create: failed to read directory '{}': {}", dir_path.display(), e))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| format!("zip/create: failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            add_file_to_zip(zip, &path, base_path, options)?;
        } else if path.is_dir() {
            add_dir_to_zip(zip, &path, base_path, options)?;
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
        return Err("zip/extract: 1 or 2 arguments required (zip-path [dest-dir])".to_string());
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("zip/extract: zip-path must be a string".to_string()),
    };

    let dest_dir = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => PathBuf::from(s),
            _ => return Err("zip/extract: dest-dir must be a string".to_string()),
        }
    } else {
        PathBuf::from(".")
    };

    // ZIPファイルを開く
    let file = File::open(zip_path)
        .map_err(|e| format!("zip/extract: failed to open zip file '{}': {}", zip_path, e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("zip/extract: failed to read zip file '{}': {}", zip_path, e))?;

    // 展開先ディレクトリを作成
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("zip/extract: failed to create destination directory: {}", e))?;

    // 各ファイルを展開
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("zip/extract: failed to read entry {}: {}", i, e))?;

        let outpath = dest_dir.join(file.name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("zip/extract: failed to create directory '{}': {}", outpath.display(), e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("zip/extract: failed to create parent directory: {}", e))?;
            }
            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("zip/extract: failed to create file '{}': {}", outpath.display(), e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("zip/extract: failed to extract file: {}", e))?;
        }
    }

    Ok(Value::String(dest_dir.to_string_lossy().to_string()))
}

/// list - ZIP内容一覧を取得
/// 引数: (zip-path) - ZIP入力パス
/// 例: (zip/list "backup.zip")
pub fn native_zip_list(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 {
        return Err("zip/list: exactly 1 argument required (zip-path)".to_string());
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("zip/list: zip-path must be a string".to_string()),
    };

    let file = File::open(zip_path)
        .map_err(|e| format!("zip/list: failed to open zip file '{}': {}", zip_path, e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("zip/list: failed to read zip file '{}': {}", zip_path, e))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i)
            .map_err(|e| format!("zip/list: failed to read entry {}: {}", i, e))?;

        let mut info = std::collections::HashMap::new();
        info.insert("name".to_string(), Value::String(file.name().to_string()));
        info.insert("size".to_string(), Value::Integer(file.size() as i64));
        info.insert("compressed-size".to_string(), Value::Integer(file.compressed_size() as i64));
        info.insert("is-dir".to_string(), Value::Bool(file.is_dir()));

        entries.push(Value::Map(info));
    }

    Ok(Value::List(entries))
}

/// add - 既存ZIPにファイルを追加
/// 引数: (zip-path files...) - ZIP入力パス、追加するファイルのリスト
/// 例: (zip/add "backup.zip" "new-file.txt")
pub fn native_zip_add(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("zip/add: at least 2 arguments required (zip-path files...)".to_string());
    }

    let zip_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("zip/add: zip-path must be a string".to_string()),
    };

    // 既存ZIPを読み込み
    let zip_file = File::open(zip_path)
        .map_err(|e| format!("zip/add: failed to open zip file '{}': {}", zip_path, e))?;
    let mut archive = ZipArchive::new(zip_file)
        .map_err(|e| format!("zip/add: failed to read zip file '{}': {}", zip_path, e))?;

    // 一時ファイルに書き込み
    let temp_path = format!("{}.tmp", zip_path);
    let temp_file = File::create(&temp_path)
        .map_err(|e| format!("zip/add: failed to create temporary file: {}", e))?;
    let mut zip = ZipWriter::new(temp_file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // 既存ファイルをコピー
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("zip/add: failed to read entry {}: {}", i, e))?;

        zip.start_file(file.name(), options)
            .map_err(|e| format!("zip/add: failed to start file: {}", e))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| format!("zip/add: failed to read file: {}", e))?;
        zip.write_all(&buffer)
            .map_err(|e| format!("zip/add: failed to write file: {}", e))?;
    }

    // 新しいファイルを追加
    for arg in &args[1..] {
        let path_str = match arg {
            Value::String(s) => s,
            _ => return Err("zip/add: all file paths must be strings".to_string()),
        };

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(format!("zip/add: path '{}' does not exist", path_str));
        }

        if path.is_file() {
            add_file_to_zip(&mut zip, path, path, &options)?;
        } else if path.is_dir() {
            add_dir_to_zip(&mut zip, path, path, &options)?;
        }
    }

    zip.finish()
        .map_err(|e| format!("zip/add: failed to finish zip: {}", e))?;

    // 元のファイルを置き換え
    fs::remove_file(zip_path)
        .map_err(|e| format!("zip/add: failed to remove original zip: {}", e))?;
    fs::rename(&temp_path, zip_path)
        .map_err(|e| format!("zip/add: failed to rename temporary file: {}", e))?;

    Ok(Value::String(zip_path.clone()))
}

/// gzip - ファイルをgzip圧縮
/// 引数: (input-path [output-path]) - 入力ファイル、出力ファイル（省略時: 入力.gz）
/// 例: (zip/gzip "data.txt")                ;; => "data.txt.gz"
///     (zip/gzip "data.txt" "backup.gz")    ;; => "backup.gz"
pub fn native_gzip(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("zip/gzip: 1 or 2 arguments required (input-path [output-path])".to_string());
    }

    let input_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("zip/gzip: input-path must be a string".to_string()),
    };

    let output_path = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("zip/gzip: output-path must be a string".to_string()),
        }
    } else {
        format!("{}.gz", input_path)
    };

    // 入力ファイルを読み込み
    let mut input = File::open(input_path)
        .map_err(|e| format!("zip/gzip: failed to open input file '{}': {}", input_path, e))?;

    // 出力ファイルを作成
    let output = File::create(&output_path)
        .map_err(|e| format!("zip/gzip: failed to create output file '{}': {}", output_path, e))?;

    // gzip圧縮
    let mut encoder = GzEncoder::new(output, Compression::default());
    std::io::copy(&mut input, &mut encoder)
        .map_err(|e| format!("zip/gzip: failed to compress file: {}", e))?;

    encoder.finish()
        .map_err(|e| format!("zip/gzip: failed to finish compression: {}", e))?;

    Ok(Value::String(output_path))
}

/// gunzip - gzipファイルを解凍
/// 引数: (input-path [output-path]) - 入力.gzファイル、出力ファイル（省略時: .gz除去）
/// 例: (zip/gunzip "data.txt.gz")             ;; => "data.txt"
///     (zip/gunzip "backup.gz" "data.txt")    ;; => "data.txt"
pub fn native_gunzip(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() || args.len() > 2 {
        return Err("zip/gunzip: 1 or 2 arguments required (input-path [output-path])".to_string());
    }

    let input_path = match &args[0] {
        Value::String(s) => s,
        _ => return Err("zip/gunzip: input-path must be a string".to_string()),
    };

    let output_path = if args.len() == 2 {
        match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err("zip/gunzip: output-path must be a string".to_string()),
        }
    } else {
        // .gz拡張子を除去
        if input_path.ends_with(".gz") {
            input_path[..input_path.len() - 3].to_string()
        } else {
            format!("{}.decompressed", input_path)
        }
    };

    // 入力ファイルを読み込み
    let input = File::open(input_path)
        .map_err(|e| format!("zip/gunzip: failed to open input file '{}': {}", input_path, e))?;

    // gzip解凍
    let mut decoder = GzDecoder::new(input);

    // 出力ファイルに書き込み
    let mut output = File::create(&output_path)
        .map_err(|e| format!("zip/gunzip: failed to create output file '{}': {}", output_path, e))?;

    std::io::copy(&mut decoder, &mut output)
        .map_err(|e| format!("zip/gunzip: failed to decompress file: {}", e))?;

    Ok(Value::String(output_path))
}
