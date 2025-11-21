//! セルフアップグレード機能
//!
//! GitHub Releasesから最新版をダウンロードして自動的にアップグレードします。

use crate::i18n::{fmt_msg, MsgKey};
use std::env;
use std::fs;
use std::path::PathBuf;

const GITHUB_REPO: &str = "sanohiro/qi-lang";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 最新リリース情報
#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

/// リリースアセット
#[derive(serde::Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// バージョン文字列を比較可能な形式に変換
fn parse_version(version: &str) -> Vec<u32> {
    version
        .trim_start_matches('v')
        .split('-')
        .next()
        .unwrap_or("")
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// バージョンを比較（新しい方がtrue）
fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for (c, l) in current_parts.iter().zip(latest_parts.iter()) {
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

/// 現在のプラットフォームに対応するアセット名のパターンを取得
/// バージョン番号を含むアセット名（例: qi-v0.1.3-linux-x86_64.tar.gz）を検索するためのパターン
fn get_platform_asset_pattern() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("macos", "aarch64") => "darwin-arm64.tar.gz".to_string(),
        ("macos", "x86_64") => "darwin-x86_64.tar.gz".to_string(),
        ("linux", "x86_64") => "linux-x86_64.tar.gz".to_string(),
        ("linux", "aarch64") => "linux-aarch64.tar.gz".to_string(),
        ("windows", "x86_64") => "windows-x86_64.zip".to_string(),
        _ => {
            eprintln!("{}", fmt_msg(MsgKey::UnsupportedPlatform, &[os, arch]));
            std::process::exit(1);
        }
    }
}

/// GitHub APIから最新リリース情報を取得
#[cfg(feature = "http-client")]
fn fetch_latest_release() -> Result<Release, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("qi-lang")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    response
        .json::<Release>()
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

/// バイナリをダウンロード
#[cfg(feature = "http-client")]
fn download_binary(url: &str) -> Result<Vec<u8>, String> {
    println!("{}", fmt_msg(MsgKey::DownloadingBinary, &[]));

    let client = reqwest::blocking::Client::builder()
        .user_agent("qi-lang")
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Download failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download error: {}", response.status()));
    }

    response
        .bytes()
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read response: {}", e))
}

/// tar.gzアーカイブを展開してqiディレクトリを取得
#[cfg(feature = "http-client")]
fn extract_qi_directory_from_targz(archive_data: &[u8]) -> Result<PathBuf, String> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    // 一時ディレクトリを作成
    let temp_dir = std::env::temp_dir().join(format!("qi-upgrade-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // gzip解凍してtar展開
    let tar_decoder = GzDecoder::new(archive_data);
    let mut archive = Archive::new(tar_decoder);
    archive
        .unpack(&temp_dir)
        .map_err(|e| format!("Failed to unpack archive: {}", e))?;

    // アーカイブ内のqiディレクトリを検索（qi-vX.X.X-platform/qi/ のような構造を想定）
    for entry in
        fs::read_dir(&temp_dir).map_err(|e| format!("Failed to read temp directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            let qi_dir = path.join("qi");
            if qi_dir.is_dir() && qi_dir.join("qi").exists() {
                return Ok(qi_dir);
            }
        }
    }

    Err("qi directory not found in archive".to_string())
}

/// 現在の実行ファイルパスを取得
fn get_current_exe() -> Result<PathBuf, String> {
    env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))
}

/// qiディレクトリを置き換え
fn replace_qi_directory(new_qi_dir: &std::path::Path) -> Result<(), String> {
    let current_exe = get_current_exe()?;

    // 現在のバイナリの親ディレクトリを取得（これがqiディレクトリであるべき）
    let qi_install_dir = current_exe
        .parent()
        .ok_or_else(|| "Failed to get parent directory of current executable".to_string())?;

    // qiディレクトリであることを確認（stdディレクトリが存在するか）
    let std_dir = qi_install_dir.join("std");
    if !std_dir.exists() {
        return Err(format!(
            "Current installation directory does not look like a qi directory: {}",
            qi_install_dir.display()
        ));
    }

    // 古いディレクトリをバックアップ
    let backup_dir = qi_install_dir.with_extension("old");
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }

    // 現在のqiディレクトリを.oldにリネーム
    fs::rename(qi_install_dir, &backup_dir)
        .map_err(|e| format!("Failed to backup current qi directory: {}", e))?;

    // 新しいqiディレクトリをコピー
    copy_directory(new_qi_dir, qi_install_dir)?;

    // 実行権限を設定（Unix系）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let new_binary = qi_install_dir.join("qi");
        let mut perms = fs::metadata(&new_binary)
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&new_binary, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

/// ディレクトリを再帰的にコピー
fn copy_directory(src: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;

    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy {}: {}", src_path.display(), e))?;
        }
    }

    Ok(())
}

/// アップグレードを実行
#[cfg(feature = "http-client")]
pub fn upgrade() -> Result<(), String> {
    println!("{}", fmt_msg(MsgKey::CheckingForUpdates, &[]));

    // 最新リリースを取得
    let release = fetch_latest_release()?;
    let latest_version = release.tag_name.trim_start_matches('v');

    println!("{}", fmt_msg(MsgKey::CurrentVersion, &[CURRENT_VERSION]));
    println!("{}", fmt_msg(MsgKey::LatestVersion, &[latest_version]));

    // バージョン比較
    if !is_newer_version(CURRENT_VERSION, latest_version) {
        println!("{}", fmt_msg(MsgKey::AlreadyLatest, &[]));
        return Ok(());
    }

    println!(
        "{}",
        fmt_msg(MsgKey::NewVersionAvailable, &[latest_version])
    );

    // プラットフォームに対応するアセットを検索（部分一致）
    let asset_pattern = get_platform_asset_pattern();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(&asset_pattern))
        .ok_or_else(|| format!("No binary found for platform pattern: {}", asset_pattern))?;

    // アーカイブをダウンロード
    let archive_data = download_binary(&asset.browser_download_url)?;

    // tar.gzから展開（Windowsの場合はzip対応が必要だが、今はLinux/macOSのみ対応）
    println!("{}", fmt_msg(MsgKey::ExtractingBinary, &[]));
    let qi_dir = extract_qi_directory_from_targz(&archive_data)?;

    // qiディレクトリを置き換え
    println!("{}", fmt_msg(MsgKey::InstallingUpdate, &[]));
    replace_qi_directory(&qi_dir)?;

    // 一時ディレクトリをクリーンアップ
    if let Some(temp_parent) = qi_dir.parent() {
        if let Some(temp_root) = temp_parent.parent() {
            let _ = fs::remove_dir_all(temp_root);
        }
    }

    println!("{}", fmt_msg(MsgKey::UpgradeSuccess, &[latest_version]));
    println!("{}", fmt_msg(MsgKey::RestartRequired, &[]));

    Ok(())
}

/// http-client機能が無効な場合のダミー実装
#[cfg(not(feature = "http-client"))]
pub fn upgrade() -> Result<(), String> {
    Err(fmt_msg(MsgKey::HttpClientNotEnabled, &[]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("0.1.0"), vec![0, 1, 0]);
        assert_eq!(parse_version("v1.2.3"), vec![1, 2, 3]);
        assert_eq!(parse_version("2.0.0-beta"), vec![2, 0, 0]);
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.1.0", "0.1.1"));
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(!is_newer_version("0.2.0", "0.1.9"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
    }
}
