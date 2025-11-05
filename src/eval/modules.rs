//! モジュール管理
//!
//! use、export、module、load_module等のモジュールシステム機能を提供します。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Env, Module, Value};
use parking_lot::RwLock;
use std::sync::Arc;

use super::helpers::qerr;
use super::Evaluator;

impl Evaluator {
    /// useモジュールの評価
    ///
    /// モジュールをロードし、指定されたインポートモードに応じて環境に追加します。
    ///
    /// # インポートモード
    /// - `UseMode::All` - 全ての公開シンボルをインポート
    /// - `UseMode::Only(names)` - 指定された関数のみインポート
    /// - `UseMode::As(alias)` - エイリアス機能（alias/name形式で全ての公開関数をインポート）
    pub(super) fn eval_use(
        &self,
        module_name: &str,
        mode: &crate::value::UseMode,
        env: Arc<RwLock<Env>>,
    ) -> Result<Value, String> {
        use crate::value::UseMode;

        // モジュールをロード
        let module = self.load_module(module_name)?;

        // インポートモードに応じて環境に追加
        match mode {
            UseMode::Only(names) => {
                // 指定された関数のみインポート
                for name in names {
                    if module.is_exported(name) {
                        if let Some(value) = module.env.read().get(name) {
                            env.write().set(name.clone(), value);
                        } else {
                            return Err(qerr(MsgKey::SymbolNotFound, &[name, module_name]));
                        }
                    } else {
                        return Err(qerr(MsgKey::SymbolNotExported, &[name, module_name]));
                    }
                }
            }
            UseMode::All => {
                // 全ての公開シンボルをインポート（デッドロック回避のため先に収集）
                let bindings: Vec<(String, Value)> = {
                    let env_guard = module.env.read();
                    let all_bindings: Vec<_> = env_guard
                        .all_bindings()
                        .map(|(name, binding)| (name.clone(), binding.clone()))
                        .collect();
                    std::mem::drop(env_guard); // 明示的にロックを解放

                    // exportリストに基づいてフィルタ
                    all_bindings
                        .into_iter()
                        .filter(|(name, binding)| {
                            match &module.exports {
                                None => !binding.is_private,       // exportなし = privateでなければ公開
                                Some(list) => list.contains(name), // exportあり = リストに含まれていれば公開
                            }
                        })
                        .map(|(name, binding)| (name, binding.value))
                        .collect()
                };

                for (name, value) in bindings {
                    env.write().set(name, value);
                }
            }
            UseMode::As(alias) => {
                // エイリアス機能: alias/name という形式で全ての公開関数をインポート（デッドロック回避のため先に収集）
                let bindings: Vec<(String, Value)> = {
                    let env_guard = module.env.read();
                    let all_bindings: Vec<_> = env_guard
                        .all_bindings()
                        .map(|(name, binding)| (name.clone(), binding.clone()))
                        .collect();
                    std::mem::drop(env_guard); // 明示的にロックを解放

                    // exportリストに基づいてフィルタ
                    all_bindings
                        .into_iter()
                        .filter(|(name, binding)| {
                            match &module.exports {
                                None => !binding.is_private,       // exportなし = privateでなければ公開
                                Some(list) => list.contains(name), // exportあり = リストに含まれていれば公開
                            }
                        })
                        .map(|(name, binding)| (name, binding.value))
                        .collect()
                };

                for (name, value) in bindings {
                    let aliased_name = format!("{}/{}", alias, name);
                    env.write().set(aliased_name, value);
                }
            }
        }

        Ok(Value::Nil)
    }

    /// パッケージ検索パスを解決
    ///
    /// モジュール名から実際のファイルパスを解決します。
    ///
    /// # 検索順序
    /// 1. 絶対パス・相対パスの場合は現在のソースファイルのディレクトリを基準に解決
    /// 2. 標準ライブラリ（カレントディレクトリ基準）: `./std/{name}.qi`（サブディレクトリ対応）
    /// 3. 標準ライブラリ（Qi実行ファイル基準）: `{qi_exe_dir}/std/{name}.qi`（サブディレクトリ対応）
    /// 4. プロジェクトローカル: `./qi_packages/{name}/mod.qi`
    /// 5. グローバルキャッシュ: `~/.qi/packages/{name}/{version}/mod.qi`（repl featureが有効な場合）
    pub(super) fn resolve_module_path(&self, name: &str) -> Result<Vec<String>, String> {
        let mut paths = Vec::new();

        // 絶対パスまたは相対パスの場合
        if name.starts_with("./") || name.starts_with("../") || name.starts_with("/") {
            // 現在のソースファイルのディレクトリを基準に相対パスを解決
            let base_dir = if let Some(source_name) = self.source_name.read().as_ref() {
                std::path::Path::new(source_name)
                    .parent()
                    .map(|p| p.to_path_buf())
            } else {
                None
            };

            let resolved_path = if let Some(base) = base_dir {
                // ソースファイルのディレクトリを基準に相対パスを解決
                base.join(name).with_extension("qi").display().to_string()
            } else {
                // ソース名がない場合はカレントディレクトリを基準にする
                format!("{}.qi", name)
            };

            paths.push(resolved_path);
            return Ok(paths);
        }

        // 標準ライブラリのパス解決（std/配下のサブディレクトリに対応）
        // 例: "std/lib/table" -> "./std/lib/table.qi" と "{qi_exe_dir}/std/lib/table.qi"
        if let Some(relative_path) = name.strip_prefix("std/") {
            // 1. カレントディレクトリの./std/配下を検索
            paths.push(format!("./std/{}.qi", relative_path));

            // 2. Qi実行ファイルと同じディレクトリのstd/配下を検索
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let std_path = exe_dir
                        .join("std")
                        .join(format!("{}.qi", relative_path))
                        .to_string_lossy()
                        .to_string();
                    paths.push(std_path);
                }
            }
        } else {
            // stdで始まらない場合は従来の検索パスを使用

            // 1. 標準ライブラリトップレベル: ./std/{name}.qi
            paths.push(format!("./std/{}.qi", name));

            // 2. 標準ライブラリトップレベル（Qi実行ファイル基準）
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let std_path = exe_dir
                        .join("std")
                        .join(format!("{}.qi", name))
                        .to_string_lossy()
                        .to_string();
                    paths.push(std_path);
                }
            }

            // 3. プロジェクトローカル: ./qi_packages/{name}/mod.qi
            paths.push(format!("./qi_packages/{}/mod.qi", name));

            // 4. グローバルキャッシュ: ~/.qi/packages/{name}/{version}/mod.qi
            #[cfg(feature = "repl")]
            {
                if let Some(home) = dirs::home_dir() {
                    let packages_dir = home.join(".qi").join("packages").join(name);

                    // バージョンディレクトリを探す（最新版を使用）
                    if let Ok(entries) = std::fs::read_dir(&packages_dir) {
                        let mut versions: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().is_dir())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect();

                        // セマンティックバージョニングでソート（簡易版）
                        versions.sort_by(|a, b| {
                            let a_parts: Vec<u32> =
                                a.split('.').filter_map(|s| s.parse().ok()).collect();
                            let b_parts: Vec<u32> =
                                b.split('.').filter_map(|s| s.parse().ok()).collect();
                            b_parts.cmp(&a_parts) // 降順（新しい順）
                        });

                        // 最新バージョンのmod.qiを追加
                        if let Some(latest) = versions.first() {
                            paths.push(
                                packages_dir
                                    .join(latest)
                                    .join("mod.qi")
                                    .to_string_lossy()
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }

        Ok(paths)
    }

    /// モジュールファイルをロード
    ///
    /// モジュールをロードしてキャッシュに保存します。
    /// 既にロード済みの場合はキャッシュから返します。
    /// 循環参照の検出も行います。
    pub(super) fn load_module(&self, name: &str) -> Result<Arc<Module>, String> {
        // 既にロード済みならキャッシュから返す
        if let Some(module) = self.modules.read().get(name) {
            return Ok(module.clone());
        }

        // 循環参照チェック
        {
            let loading = self.loading_modules.read();
            if loading.contains(&name.to_string()) {
                return Err(fmt_msg(
                    MsgKey::CircularDependency,
                    &[&loading.join(" -> ")],
                ));
            }
        }

        // ロード中のモジュールリストに追加
        self.loading_modules.write().push(name.to_string());

        // パッケージ検索パスを解決
        let paths = self.resolve_module_path(name)?;

        let mut content = None;
        let mut found_path = None;
        for path in &paths {
            if let Ok(c) = std::fs::read_to_string(path) {
                content = Some(c);
                found_path = Some(path.clone());
                break;
            }
        }

        let content = content.ok_or_else(|| qerr(MsgKey::ModuleNotFound, &[name]))?;

        // デバッグ: ロードしたパスを表示（開発時のみ）
        if std::env::var("QI_DEBUG").is_ok() {
            eprintln!(
                "[DEBUG] Loaded module '{}' from: {}",
                name,
                found_path.as_deref().unwrap_or_default()
            );
        }

        // パースして評価
        let mut parser = crate::parser::Parser::new(&content)
            .map_err(|e| qerr(MsgKey::ModuleParserInitError, &[name, &e]))?;

        let exprs = parser
            .parse_all()
            .map_err(|e| qerr(MsgKey::ModuleParseError, &[name, &e]))?;

        // 新しい環境で評価
        let module_env = Arc::new(RwLock::new(Env::new()));

        // グローバル環境から組み込み関数をコピー
        let bindings: Vec<_> = self
            .global_env
            .read()
            .bindings()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in bindings {
            module_env.write().set(key, value);
        }

        // 現在のモジュール名をクリア（評価前の状態に戻す）
        let prev_module = self.current_module.read().clone();

        // 式を順次評価
        for expr in exprs {
            self.eval_with_env(&expr, module_env.clone())?;
        }

        // ロード中リストから削除
        self.loading_modules.write().pop();

        // モジュールが登録されているか確認、なければデフォルトで全公開モジュールを作成
        let module = {
            let modules_guard = self.modules.read();
            let existing = modules_guard.get(name).cloned();
            std::mem::drop(modules_guard); // 明示的にロックを解放

            if let Some(m) = existing {
                m
            } else {
                // exportがない場合は全公開モジュールとして登録
                let module_name = self.current_module.read().clone().unwrap_or_else(|| {
                    // モジュール名が設定されていない場合はファイル名から取得
                    std::path::Path::new(name)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(name)
                        .to_string()
                });

                let module = Arc::new(Module {
                    name: module_name.clone(),
                    file_path: found_path,
                    env: module_env.clone(),
                    exports: None, // None = 全公開（defn-以外）
                });

                self.modules
                    .write()
                    .insert(name.to_string(), module.clone());
                module
            }
        };

        // 現在のモジュール名を元に戻す
        *self.current_module.write() = prev_module;

        Ok(module)
    }
}
