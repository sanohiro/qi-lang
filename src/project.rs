//! プロジェクト管理（qi.toml）
//!
//! Qiプロジェクトのメタデータと依存関係を管理します。

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
            .map_err(|e| format!("qi.tomlの読み込みに失敗: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("qi.tomlのパースに失敗: {}", e))
    }

    /// カレントディレクトリからqi.tomlを探す
    pub fn find_and_load() -> Result<Self, String> {
        let current = std::env::current_dir()
            .map_err(|e| format!("カレントディレクトリの取得に失敗: {}", e))?;

        Self::load(current.join("qi.toml"))
    }

    /// qi.tomlを書き込む
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("qi.tomlのシリアライズに失敗: {}", e))?;

        fs::write(path.as_ref(), content).map_err(|e| format!("qi.tomlの書き込みに失敗: {}", e))
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
        return Err(format!("ディレクトリ '{}' は既に存在します", project_name));
    }
    fs::create_dir_all(&project_dir).map_err(|e| format!("ディレクトリの作成に失敗: {}", e))?;

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

    // テンプレートからプロジェクトを生成（qi.tomlを含む）
    copy_template(&template_dir, &project_dir, &vars)?;

    println!(
        "\n新しいQiプロジェクトが作成されました: {}",
        project_dir.display()
    );
    println!("\n次のステップ:");
    println!("  cd {}", project_dir.display());
    println!("  qi main.qi");

    Ok(())
}

/// 互換性のため残す（deprecated）
#[deprecated(note = "Use new_project instead")]
pub fn init_project(project_name: Option<String>, non_interactive: bool) -> Result<(), String> {
    let project_dir = if let Some(name) = &project_name {
        PathBuf::from(name)
    } else {
        std::env::current_dir().map_err(|e| format!("カレントディレクトリの取得に失敗: {}", e))?
    };

    // ディレクトリが存在しない場合は作成
    if !project_dir.exists() {
        fs::create_dir_all(&project_dir).map_err(|e| format!("ディレクトリの作成に失敗: {}", e))?;
    }

    // qi.tomlが既に存在するかチェック
    let qi_toml_path = project_dir.join("qi.toml");
    if qi_toml_path.exists() {
        return Err("qi.tomlが既に存在します".to_string());
    }

    // プロジェクトメタデータを取得
    let metadata = if non_interactive {
        create_default_metadata(&project_dir)
    } else {
        prompt_metadata(&project_dir)?
    };

    // QiProjectを作成
    let project = QiProject {
        project: metadata,
        dependencies: HashMap::new(),
        features: {
            let mut features = HashMap::new();
            features.insert("default".to_string(), vec![]);
            features
        },
    };

    // qi.tomlを保存
    project.save(&qi_toml_path)?;
    println!("✓ qi.tomlを作成しました");

    // ディレクトリ構造を作成
    create_project_structure(&project_dir)?;

    println!(
        "\n新しいQiプロジェクトが作成されました: {}",
        project_dir.display()
    );
    println!("\n次のステップ:");
    println!("  cd {}", project_dir.display());
    println!("  qi main.qi");

    Ok(())
}

/// デフォルトのメタデータを作成
fn create_default_metadata(project_dir: &Path) -> ProjectMetadata {
    let name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("qi-project")
        .to_string();

    ProjectMetadata {
        name,
        version: "0.1.0".to_string(),
        authors: None,
        description: None,
        license: Some("MIT".to_string()),
        qi_version: Some("0.1.0".to_string()),
    }
}

/// 対話的にメタデータを入力
fn prompt_metadata(project_dir: &Path) -> Result<ProjectMetadata, String> {
    let default_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("qi-project");

    println!("新しいQiプロジェクトを作成します\n");

    let name = prompt_with_default("プロジェクト名", default_name)?;
    let version = prompt_with_default("バージョン", "0.1.0")?;
    let description = prompt_optional("説明")?;
    let author = prompt_optional("著者名")?;
    let license = prompt_with_default("ライセンス", "MIT")?;

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
    print!("{} (省略可): ", label);
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

/// プロジェクト構造を作成
fn create_project_structure(project_dir: &Path) -> Result<(), String> {
    // ディレクトリ作成
    fs::create_dir_all(project_dir.join("src"))
        .map_err(|e| format!("src/ディレクトリの作成に失敗: {}", e))?;
    fs::create_dir_all(project_dir.join("examples"))
        .map_err(|e| format!("examples/ディレクトリの作成に失敗: {}", e))?;
    fs::create_dir_all(project_dir.join("tests"))
        .map_err(|e| format!("tests/ディレクトリの作成に失敗: {}", e))?;

    println!("✓ ディレクトリ構造を作成しました");

    // main.qiを作成
    let main_qi = r#";; main.qi - エントリーポイント

(println "Hello, Qi!")

;; 挨拶関数
(defn greet [name]
  (str "こんにちは、" name "さん！"))

;; 関数を使用
(println (greet "World"))

;; パイプライン例
(println ([1 2 3 4 5]
          |> (map (fn [x] (* x x)))
          |> (filter (fn [x] (> x 10)))))
"#;
    fs::write(project_dir.join("main.qi"), main_qi)
        .map_err(|e| format!("main.qiの作成に失敗: {}", e))?;
    println!("✓ main.qiを作成しました");

    // src/lib.qiを作成
    let lib_qi = r#";; lib.qi - ライブラリコード

(defn greet [name]
  (str "こんにちは、" name "さん！"))

(defn add [a b]
  (+ a b))
"#;
    fs::write(project_dir.join("src/lib.qi"), lib_qi)
        .map_err(|e| format!("src/lib.qiの作成に失敗: {}", e))?;
    println!("✓ src/lib.qiを作成しました");

    // examples/example.qiを作成
    let example_qi = r#";; example.qi - サンプルコード

(println "=== Qiプロジェクトのサンプル ===")

;; 関数定義
(defn greet [name]
  (str "こんにちは、" name "さん！"))

(defn add [a b]
  (+ a b))

;; 挨拶
(println (greet "Qi"))

;; 計算
(println f"2 + 3 = {(add 2 3)}")

;; パイプライン
(println "パイプライン例:")
(println ([1 2 3 4 5]
          |> (map (fn [x] (* x x)))
          |> (filter (fn [x] (> x 10)))))

;; 並列処理
(println "\n並列処理例:")
(println ([1 2 3 4 5 6 7 8 9 10]
          ||> (fn [x] (* x x))))
"#;
    fs::write(project_dir.join("examples/example.qi"), example_qi)
        .map_err(|e| format!("examples/example.qiの作成に失敗: {}", e))?;
    println!("✓ examples/example.qiを作成しました");

    // tests/test.qiを作成
    let test_qi = r#";; test.qi - テストコード

(println "=== テスト実行 ===")

;; テスト対象の関数
(defn greet [name]
  (str "こんにちは、" name "さん！"))

(defn add [a b]
  (+ a b))

;; greet関数のテスト
(def result (greet "Test"))
(if (= result "こんにちは、Testさん！")
  (println "✓ greet関数のテスト: 成功")
  (println "✗ greet関数のテスト: 失敗"))

;; add関数のテスト
(def add-result (add 2 3))
(if (= add-result 5)
  (println "✓ add関数のテスト: 成功")
  (println "✗ add関数のテスト: 失敗"))

;; パイプライン処理のテスト
(def pipeline-result
  ([1 2 3 4 5]
   |> (map (fn [x] (* x 2)))
   |> (reduce + 0)))

(if (= pipeline-result 30)
  (println "✓ パイプライン処理のテスト: 成功")
  (println "✗ パイプライン処理のテスト: 失敗"))

(println "\nテスト完了")
"#;
    fs::write(project_dir.join("tests/test.qi"), test_qi)
        .map_err(|e| format!("tests/test.qiの作成に失敗: {}", e))?;
    println!("✓ tests/test.qiを作成しました");

    Ok(())
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

    Err(format!("テンプレート '{}' が見つかりません", name))
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
    for entry in fs::read_dir(src).map_err(|e| format!("ディレクトリの読み込みに失敗: {}", e))?
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
                .map_err(|e| format!("ディレクトリの作成に失敗: {}", e))?;
            copy_dir_recursive(&src_path, &dest_path, vars)?;
        } else if file_type.is_file() {
            // ファイルを読み込んで変数置換
            let content = fs::read_to_string(&src_path)
                .map_err(|e| format!("ファイルの読み込みに失敗: {}", e))?;
            let rendered = render_template(&content, vars);
            fs::write(&dest_path, rendered)
                .map_err(|e| format!("ファイルの書き込みに失敗: {}", e))?;
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
        println!("利用可能なテンプレートがありません");
        return Ok(());
    }

    println!("利用可能なテンプレート:");
    for template in templates {
        if let Ok(info) = load_template_info(&template) {
            println!(
                "  {:16} - {}",
                info.template.name, info.template.description
            );
        } else {
            println!("  {:16} - (情報なし)", template);
        }
    }

    Ok(())
}

/// テンプレート情報を表示
pub fn show_template_info(name: &str) -> Result<(), String> {
    let template_dir = find_template(name)?;
    let info = load_template_info(name)?;

    println!("Template: {}", info.template.name);
    println!("Description: {}", info.template.description);
    if !info.template.author.is_empty() {
        println!("Author: {}", info.template.author);
    }
    if !info.template.version.is_empty() {
        println!("Version: {}", info.template.version);
    }
    if !info.features.required.is_empty() {
        println!("Required features: {}", info.features.required.join(", "));
    }
    println!("Location: {}", template_dir.display());

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
    let info_path = template_dir.join("template.toml");

    let content = fs::read_to_string(&info_path)
        .map_err(|e| format!("template.tomlの読み込みに失敗: {}", e))?;

    toml::from_str(&content).map_err(|e| format!("template.tomlのパースに失敗: {}", e))
}
