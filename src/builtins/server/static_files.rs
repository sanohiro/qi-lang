//! 静的ファイル配信機能

use super::helpers::kw;
use crate::i18n::{fmt_msg, MsgKey};
use crate::value::Value;

pub(super) fn serve_static_file(dir_path: &str, req: &Value) -> Result<Value, String> {
    let path_key = kw("path");

    let path = match req {
        Value::Map(m) => match m.get(&path_key) {
            Some(Value::String(p)) => p,
            _ => {
                return Err(fmt_msg(
                    MsgKey::RequestMustHave,
                    &["request", ":path string"],
                ))
            }
        },
        _ => return Err(fmt_msg(MsgKey::RequestMustBe, &["request", "a map"])),
    };

    // セキュリティチェック（パストラバーサル攻撃を防止）
    let file_path = validate_safe_path(std::path::Path::new(dir_path), path)
        .map_err(|e| fmt_msg(MsgKey::StaticFileInvalidPath, &[&e]))?;

    // index.htmlの自動配信（ディレクトリの場合）
    let file_path = if file_path.is_dir() {
        file_path.join("index.html")
    } else {
        file_path
    };

    // ファイルの存在確認（メタデータ取得）
    std::fs::metadata(&file_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            return fmt_msg(MsgKey::StaticFileNotFound, &[path]);
        }
        fmt_msg(MsgKey::StaticFileMetadataFailed, &[&e.to_string()])
    })?;

    // ストリーミングレスポンスを生成（:body-file を使用）
    let content_type = get_content_type(file_path.to_str().unwrap_or(""));
    let file_path_str = file_path
        .to_str()
        .ok_or_else(|| fmt_msg(MsgKey::InvalidFilePath, &["serve_static_file"]))?;

    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(200));
    resp.insert(kw("body-file"), Value::String(file_path_str.to_string()));

    let mut headers = crate::new_hashmap();
    headers.insert(
        crate::value::MapKey::String("Content-Type".to_string()),
        Value::String(content_type.to_string()),
    );
    resp.insert(kw("headers"), Value::Map(headers));

    Ok(Value::Map(resp))
}

pub(super) fn get_content_type(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",

        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "webp" => "image/webp",

        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",

        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",

        _ => "application/octet-stream",
    }
}

/// 安全なパス検証（ベースディレクトリ外へのアクセスを防止）
///
/// パストラバーサル攻撃を防ぐため、以下のチェックを実施:
/// - URLデコード後のパス検証
/// - 正規化（canonicalize）によるシンボリックリンク・..の解決
pub(super) fn validate_safe_path(
    base_dir: &std::path::Path,
    requested_path: &str,
) -> Result<std::path::PathBuf, String> {
    // URLデコード（%2e%2e などの攻撃を防ぐ）
    let decoded_path = urlencoding::decode(requested_path)
        .map_err(|_| fmt_msg(MsgKey::StaticFileInvalidUrlEncoding, &[]))?;

    // 相対パスの正規化（先頭の / を除去）
    let requested = std::path::Path::new(decoded_path.trim_start_matches('/'));

    // ベースディレクトリと結合
    let full_path = base_dir.join(requested);

    // 正規化（シンボリックリンク解決、.. 解決）
    let canonical = match full_path.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            // ファイルが存在しない場合、親ディレクトリまで正規化
            let parent = full_path
                .parent()
                .ok_or_else(|| fmt_msg(MsgKey::StaticFileInvalidPathTraversal, &[]))?;
            let parent_canonical = parent
                .canonicalize()
                .map_err(|_| fmt_msg(MsgKey::StaticFileInvalidBaseDir, &[]))?;
            let file_name = full_path
                .file_name()
                .ok_or_else(|| fmt_msg(MsgKey::StaticFileInvalidFileName, &[]))?;
            parent_canonical.join(file_name)
        }
    };

    // ベースディレクトリの正規化
    let base_canonical = base_dir
        .canonicalize()
        .map_err(|_| fmt_msg(MsgKey::StaticFileBaseDirNotExist, &[]))?;

    // ベースディレクトリ外へのアクセスを防止
    if !canonical.starts_with(&base_canonical) {
        return Err(fmt_msg(MsgKey::StaticFilePathTraversal, &[]));
    }

    Ok(canonical)
}

/// server/static-file - 単一ファイルを配信するレスポンスを生成（ストリーミング対応）
pub fn native_server_static_file(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/static-file"]));
    }

    let file_path_str = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeString,
                &["server/static-file", "file path"],
            ))
        }
    };

    // セキュリティチェック（絶対パスの場合はカレントディレクトリから検証）
    let base_dir = std::env::current_dir()
        .map_err(|e| fmt_msg(MsgKey::StaticFileFailedToGetCwd, &[&e.to_string()]))?;
    let file_path = validate_safe_path(&base_dir, file_path_str)
        .map_err(|e| fmt_msg(MsgKey::StaticFileInvalidFilePath, &[&e]))?;

    // ファイルの存在確認
    std::fs::metadata(&file_path)
        .map_err(|e| fmt_msg(MsgKey::ServerStaticFileMetadataFailed, &[&e.to_string()]))?;

    // ストリーミングレスポンスを生成（:body-file を使用）
    let content_type = get_content_type(file_path.to_str().unwrap_or(""));

    let mut resp = crate::new_hashmap();
    resp.insert(kw("status"), Value::Integer(200));
    resp.insert(
        kw("body-file"),
        Value::String(
            file_path
                .to_str()
                .ok_or_else(|| fmt_msg(MsgKey::StaticFileInvalidEncoding, &[]))?
                .to_string(),
        ),
    );

    let mut headers = crate::new_hashmap();
    headers.insert(
        crate::value::MapKey::String("Content-Type".to_string()),
        Value::String(content_type.to_string()),
    );
    resp.insert(kw("headers"), Value::Map(headers));

    Ok(Value::Map(resp))
}

/// server/static-dir - ディレクトリから静的ファイルを配信するハンドラーを生成
pub fn native_server_static_dir(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err(fmt_msg(MsgKey::Need1Arg, &["server/static-dir"]));
    }

    let dir_path_str = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(fmt_msg(
                MsgKey::MustBeString,
                &["server/static-dir", "directory path"],
            ))
        }
    };

    // ディレクトリパスを検証（セキュリティチェック）
    let dir_path = std::path::Path::new(&dir_path_str);

    // ディレクトリの存在チェック
    if !dir_path.is_dir() {
        return Err(fmt_msg(
            MsgKey::ServerStaticDirNotDirectory,
            &[&dir_path_str],
        ));
    }

    // 正規化されたディレクトリパスを取得（シンボリックリンク解決）
    let canonical_dir = dir_path
        .canonicalize()
        .map_err(|e| fmt_msg(MsgKey::StaticFileFailedToCanonicalize, &[&e.to_string()]))?;
    let canonical_dir_str = canonical_dir
        .to_str()
        .ok_or_else(|| fmt_msg(MsgKey::StaticFileInvalidEncoding, &[]))?
        .to_string();

    // 静的ファイルハンドラーマーカー（ミドルウェアと同じパターン）
    // 正規化されたパスを保存（セキュリティ向上）
    let mut metadata = crate::new_hashmap();
    metadata.insert(
        crate::value::MapKey::String("__static_dir__".to_string()),
        Value::String(canonical_dir_str),
    );

    Ok(Value::Map(metadata))
}

// ========================================
// 認証ミドルウェア
// ========================================
