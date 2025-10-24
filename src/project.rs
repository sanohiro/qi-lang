//! プロジェクト管理（qi.toml）
//!
//! Qiプロジェクトのメタデータと依存関係を管理します。

use crate::i18n::{fmt_msg, fmt_ui_msg, ui_msg, Lang, MsgKey, UiMsg};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Qiプロジェクト設定（qi.toml）
#[derive(Debug, Deserialize, Serialize)]
pub struct QiProject {
    pub project: ProjectMetadata,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub features: HashMap<String, Vec<String>>,
}

/// プロジェクトメタデータ
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(rename = "qi-version", skip_serializing_if = "Option::is_none")]
    pub qi_version: Option<String>,
}

/// 依存関係の種類
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    /// バージョン指定: "0.1.0"
    Version(String),
    /// 詳細指定: { path = "...", git = "...", version = "..." }
    Detailed(DependencyDetail),
}

/// 依存関係の詳細
#[derive(Debug, Deserialize, Serialize)]
pub struct DependencyDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

impl QiProject {
    /// qi.tomlを読み込む
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| fmt_msg(MsgKey::QiTomlFailedToRead, &[&e.to_string()]))?;

        toml::from_str(&content).map_err(|e| fmt_msg(MsgKey::QiTomlFailedToParse, &[&e.to_string()]))
    }

    /// カレントディレクトリからqi.tomlを探す
    pub fn find_and_load() -> Result<Self, String> {
        let current = std::env::current_dir()
            .map_err(|e| fmt_msg(MsgKey::FailedToGetCurrentDir, &[&e.to_string()]))?;

        Self::load(current.join("qi.toml"))
    }

    /// qi.tomlを書き込む
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| fmt_msg(MsgKey::QiTomlFailedToSerialize, &[&e.to_string()]))?;

        fs::write(path.as_ref(), content).map_err(|e| fmt_msg(MsgKey::QiTomlFailedToWrite, &[&e.to_string()]))
    }
}

/// qi newコマンドの実装（テンプレート対応）
pub fn new_project(project_name: String, template: Option<String>) -> Result<(), String> {
    let template_name = template.unwrap_or_else(|| "basic".to_string());

    // テンプレートを探す
    let template_dir = find_template(&template_name)?;

    // プロジェクトディレクトリ作成
    let project_dir = PathBuf::from(&project_name);
    if project_dir.exists() {
        return Err(fmt_msg(MsgKey::DirectoryAlreadyExists, &[&project_name]));
    }
    fs::create_dir_all(&project_dir).map_err(|e| fmt_msg(MsgKey::FailedToCreateDirectory, &[&e.to_string()]))?;

    // メタデータを取得
    let metadata = prompt_metadata(&project_dir)?;

    // テンプレート変数
    let mut vars = HashMap::new();
    vars.insert("project_name".to_string(), metadata.name.clone());
    vars.insert("version".to_string(), metadata.version.clone());
    // 空の値も含めてすべての変数を追加（条件分岐の処理のため）
    vars.insert(
        "author".to_string(),
        metadata
            .authors
            .as_ref()
            .map(|a| a.join(", "))
            .unwrap_or_default(),
    );
    vars.insert(
        "description".to_string(),
        metadata.description.as_ref().cloned().unwrap_or_default(),
    );
    vars.insert(
        "license".to_string(),
        metadata.license.as_ref().cloned().unwrap_or_default(),
    );

    // 言語に応じたテンプレートディレクトリを選択
    let lang = Lang::from_env();
    let lang_template_dir = select_lang_template(&template_dir, lang);

    // テンプレートからプロジェクトを生成（qi.tomlを含む）
    copy_template(&lang_template_dir, &project_dir, &vars)?;

    println!("{}", fmt_ui_msg(UiMsg::ProjectCreated, &[&project_dir.display().to_string()]));
    println!("{}", ui_msg(UiMsg::ProjectNextSteps));
    println!("  cd {}", project_dir.display());
    println!("  qi main.qi");

    Ok(())
}

/// 対話的にメタデータを入力
fn prompt_metadata(project_dir: &Path) -> Result<ProjectMetadata, String> {
    let default_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("qi-project");

    println!("{}", ui_msg(UiMsg::ProjectCreating));

    let name = prompt_with_default(&ui_msg(UiMsg::PromptProjectName), default_name)?;
    let version = prompt_with_default(&ui_msg(UiMsg::PromptVersion), "0.1.0")?;
    let description = prompt_optional(&ui_msg(UiMsg::PromptDescription))?;
    let author = prompt_optional(&ui_msg(UiMsg::PromptAuthor))?;
    let license = prompt_with_default(&ui_msg(UiMsg::PromptLicense), "MIT")?;

    Ok(ProjectMetadata {
        name,
        version,
        authors: author.map(|a| vec![a]),
        description,
        license: Some(license),
        qi_version: Some("0.1.0".to_string()),
    })
}

/// デフォルト値付きでプロンプト表示
fn prompt_with_default(label: &str, default: &str) -> Result<String, String> {
    print!("{} [{}]: ", label, default);
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;

    let trimmed = input.trim();
    Ok(if trimmed.is_empty() {
        default.to_string()
    } else {
        trimmed.to_string()
    })
}

/// オプショナルな値をプロンプト表示
fn prompt_optional(label: &str) -> Result<Option<String>, String> {
    print!("{} {}: ", label, ui_msg(UiMsg::PromptOptional));
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;

    let trimmed = input.trim();
    Ok(if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    })
}

// ====================================================================
// テンプレートシステム
// ====================================================================

/// テンプレートを検索
fn find_template(name: &str) -> Result<PathBuf, String> {
    // 1. カレントディレクトリ
    let current_templates = PathBuf::from(".qi/templates").join(name);
    if current_templates.exists() {
        return Ok(current_templates);
    }

    // 2. ホームディレクトリ
    #[cfg(feature = "repl")]
    if let Some(home) = dirs::home_dir() {
        let home_templates = home.join(".qi/templates").join(name);
        if home_templates.exists() {
            return Ok(home_templates);
        }
    }

    // 3. qiバイナリの隣（std/templates/）
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let std_templates = exe_dir.join("std/templates").join(name);
            if std_templates.exists() {
                return Ok(std_templates);
            }
        }
    }

    // 4. ソースからのビルド時（開発用）
    let dev_templates = PathBuf::from("std/templates").join(name);
    if dev_templates.exists() {
        return Ok(dev_templates);
    }

    Err(fmt_msg(MsgKey::TemplateNotFound, &[name]))
}

/// 言語に応じたテンプレートディレクトリを選択
fn select_lang_template(template_dir: &Path, lang: Lang) -> PathBuf {
    let lang_dir = template_dir.join(lang.as_str());
    if lang_dir.exists() {
        lang_dir
    } else {
        // フォールバック: 英語版
        template_dir.join("en")
    }
}

/// テンプレートをコピー＆変数置換
fn copy_template(
    template_dir: &Path,
    dest_dir: &Path,
    vars: &HashMap<String, String>,
) -> Result<(), String> {
    // テンプレートディレクトリを再帰的にコピー
    copy_dir_recursive(template_dir, dest_dir, vars)
}

/// ディレクトリを再帰的にコピー
fn copy_dir_recursive(
    src: &Path,
    dest: &Path,
    vars: &HashMap<String, String>,
) -> Result<(), String> {
    for entry in fs::read_dir(src).map_err(|e| fmt_msg(MsgKey::FailedToReadDirectory, &[&e.to_string()]))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let src_path = entry.path();
        let file_name = entry.file_name();

        // template.tomlはスキップ
        if file_name == "template.toml" {
            continue;
        }

        // .templateサフィックスを除去
        let dest_file_name = if let Some(name) = file_name.to_str() {
            if name.ends_with(".template") {
                name.strip_suffix(".template").unwrap()
            } else {
                name
            }
        } else {
            continue;
        };

        let dest_path = dest.join(dest_file_name);

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)
                .map_err(|e| fmt_msg(MsgKey::FailedToCreateDirectory, &[&e.to_string()]))?;
            copy_dir_recursive(&src_path, &dest_path, vars)?;
        } else if file_type.is_file() {
            // ファイルを読み込んで変数置換
            let content = fs::read_to_string(&src_path)
                .map_err(|e| fmt_msg(MsgKey::FailedToReadFile, &[&e.to_string()]))?;
            let rendered = render_template(&content, vars);
            fs::write(&dest_path, rendered)
                .map_err(|e| fmt_msg(MsgKey::FailedToWriteFile, &[&e.to_string()]))?;
        }
    }
    Ok(())
}

/// テンプレート変数を置換（シンプル実装）
fn render_template(content: &str, vars: &HashMap<String, String>) -> String {
    let mut result = content.to_string();

    // {{ variable }} を置換
    for (key, value) in vars {
        let placeholder = format!("{{{{ {} }}}}", key);
        result = result.replace(&placeholder, value);
    }

    // 条件分岐の処理（簡易版）
    // {{ #if var }}...{{ /if }} のような行を削除または展開
    let lines: Vec<&str> = result.lines().collect();
    let mut output_lines = Vec::new();
    let mut skip_mode = false;

    for line in lines {
        // 同じ行に {{ #if }} と {{ /if }} がある場合の処理
        if line.contains("{{ #if") && line.contains("{{ /if }}") {
            let mut processed_line = line.to_string();
            let mut include_line = true;

            // すべての条件分岐を処理
            for (k, v) in vars.iter() {
                let if_pattern = format!("{{{{ #if {} }}}}", k);
                if processed_line.contains(&if_pattern) {
                    if !v.is_empty() {
                        // 変数が存在する場合、条件文を削除して内容を残す
                        processed_line = processed_line.replace(&if_pattern, "");
                        processed_line = processed_line.replace("{{ /if }}", "");
                    } else {
                        // 変数が存在しない場合、行全体を削除
                        include_line = false;
                        break;
                    }
                }
            }

            if include_line {
                output_lines.push(processed_line);
            }
        } else if line.contains("{{ #if") {
            // 複数行にまたがる条件分岐の開始
            let var_exists = vars
                .iter()
                .any(|(k, v)| line.contains(&format!("#if {}", k)) && !v.is_empty());
            if !var_exists {
                skip_mode = true;
            }
        } else if line.contains("{{ /if }}") {
            // 複数行にまたがる条件分岐の終了
            skip_mode = false;
        } else if !skip_mode {
            output_lines.push(line.to_string());
        }
    }

    // 最後の改行を保持
    let mut result_str = output_lines.join("\n");
    if content.ends_with('\n') {
        result_str.push('\n');
    }
    result_str
}

/// テンプレート情報を取得
#[derive(Debug, Deserialize)]
struct TemplateInfo {
    template: TemplateMetadata,
    #[serde(default)]
    features: FeatureInfo,
}

#[derive(Debug, Deserialize)]
struct TemplateMetadata {
    name: String,
    description: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    version: String,
}

#[derive(Debug, Deserialize, Default)]
struct FeatureInfo {
    #[serde(default)]
    required: Vec<String>,
}

/// テンプレート一覧を取得
pub fn list_templates() -> Result<(), String> {
    let templates = find_all_templates()?;

    if templates.is_empty() {
        println!("{}", ui_msg(UiMsg::TemplateNoTemplates));
        return Ok(());
    }

    println!("{}", ui_msg(UiMsg::TemplateAvailable));
    for template in templates {
        if let Ok(info) = load_template_info(&template) {
            println!(
                "  {:16} - {}",
                info.template.name, info.template.description
            );
        } else {
            println!("  {:16} - {}", template, ui_msg(UiMsg::TemplateNoInfo));
        }
    }

    Ok(())
}

/// テンプレート情報を表示
pub fn show_template_info(name: &str) -> Result<(), String> {
    let template_dir = find_template(name)?;
    let info = load_template_info(name)?;

    println!("{}", fmt_ui_msg(UiMsg::TemplateInfoTemplate, &[&info.template.name]));
    println!("{}", fmt_ui_msg(UiMsg::TemplateInfoDescription, &[&info.template.description]));
    if !info.template.author.is_empty() {
        println!("{}", fmt_ui_msg(UiMsg::TemplateInfoAuthor, &[&info.template.author]));
    }
    if !info.template.version.is_empty() {
        println!("{}", fmt_ui_msg(UiMsg::TemplateInfoVersion, &[&info.template.version]));
    }
    if !info.features.required.is_empty() {
        println!("{}", fmt_ui_msg(UiMsg::TemplateInfoRequired, &[&info.features.required.join(", ")]));
    }
    println!("{}", fmt_ui_msg(UiMsg::TemplateInfoLocation, &[&template_dir.display().to_string()]));

    Ok(())
}

/// すべてのテンプレートを検索
fn find_all_templates() -> Result<Vec<String>, String> {
    let mut templates = Vec::new();

    // std/templates/ を探す
    let std_templates = PathBuf::from("std/templates");
    if std_templates.exists() {
        if let Ok(entries) = fs::read_dir(&std_templates) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Some(name) = entry.file_name().to_str() {
                        templates.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(templates)
}

/// テンプレート情報を読み込む
fn load_template_info(name: &str) -> Result<TemplateInfo, String> {
    let template_dir = find_template(name)?;

    // 言語に応じたtemplate.tomlを読み込む
    let lang = Lang::from_env();
    let lang_template_dir = select_lang_template(&template_dir, lang);
    let info_path = lang_template_dir.join("template.toml");

    let content = fs::read_to_string(&info_path)
        .map_err(|e| fmt_msg(MsgKey::TemplateTomlFailedToRead, &[&e.to_string()]))?;

    toml::from_str(&content).map_err(|e| fmt_msg(MsgKey::TemplateTomlFailedToParse, &[&e.to_string()]))
}
