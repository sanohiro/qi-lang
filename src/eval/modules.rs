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
    /// 1. 絶対パス・相対パスの場合はそのまま使用
    /// 2. プロジェクトローカル: `./qi_packages/{name}/mod.qi`
    /// 3. グローバルキャッシュ: `~/.qi/packages/{name}/{version}/mod.qi`（repl featureが有効な場合）
    pub(super) fn resolve_module_path(&self, name: &str) -> Result<Vec<String>, String> {
        let mut paths = Vec::new();

        // 絶対パスまたは相対パスの場合はそのまま使用
        if name.starts_with("./") || name.starts_with("../") || name.starts_with("/") {
            paths.push(format!("{}.qi", name));
            return Ok(paths);
        }

        // 1. プロジェクトローカル: ./qi_packages/{name}/mod.qi
        paths.push(format!("./qi_packages/{}/mod.qi", name));

        // 2. グローバルキャッシュ: ~/.qi/packages/{name}/{version}/mod.qi
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
