//! セルフアップグレード機能
//!
//! GitHub Releasesから最新版をダウンロードして自動的にアップグレードします。

use crate::i18n::{fmt_msg, MsgKey};
use std::env;
use std::fs;
use std::io::Write;
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

/// 現在のプラットフォームに対応するアセット名を取得
fn get_platform_asset_name() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("macos", "aarch64") => "qi-aarch64-apple-darwin".to_string(),
        ("macos", "x86_64") => "qi-x86_64-apple-darwin".to_string(),
        ("linux", "x86_64") => "qi-x86_64-unknown-linux-gnu".to_string(),
        ("linux", "aarch64") => "qi-aarch64-unknown-linux-gnu".to_string(),
        ("windows", "x86_64") => "qi-x86_64-pc-windows-msvc.exe".to_string(),
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

/// 現在の実行ファイルパスを取得
fn get_current_exe() -> Result<PathBuf, String> {
    env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))
}

/// バイナリを置き換え
fn replace_binary(new_binary: &[u8]) -> Result<(), String> {
    let current_exe = get_current_exe()?;

    // 一時ファイルに書き込み
    let temp_path = current_exe.with_extension("tmp");

    {
        let mut file = fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        file.write_all(new_binary)
            .map_err(|e| format!("Failed to write binary: {}", e))?;
    }

    // 実行権限を設定（Unix系）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    // 古いバイナリをバックアップ
    let backup_path = current_exe.with_extension("old");
    if backup_path.exists() {
        fs::remove_file(&backup_path).map_err(|e| format!("Failed to remove old backup: {}", e))?;
    }

    fs::rename(&current_exe, &backup_path)
        .map_err(|e| format!("Failed to backup current binary: {}", e))?;

    // 新しいバイナリを配置
    fs::rename(&temp_path, &current_exe)
        .map_err(|e| format!("Failed to install new binary: {}", e))?;

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

    // プラットフォームに対応するアセットを検索
    let asset_name = get_platform_asset_name();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| format!("No binary found for platform: {}", asset_name))?;

    // バイナリをダウンロード
    let binary_data = download_binary(&asset.browser_download_url)?;

    // バイナリを置き換え
    println!("{}", fmt_msg(MsgKey::InstallingUpdate, &[]));
    replace_binary(&binary_data)?;

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
