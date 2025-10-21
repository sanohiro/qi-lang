//! 組み込み関数モジュール
//!
//! Qi-langは**2層モジュール設計**を採用しています：
//!
//! ## Core（88個）- グローバル名前空間、自動インポート
//! - 特殊形式・演算子（11個）: def, fn, let, do, if, match, try, defer, |>, ||>, |>?
//! - リスト操作（29個）: first, rest, last, nth, take, drop, map, filter, reduce, pmap, tap, etc.
//! - マップ操作（9個）: get, keys, vals, assoc, dissoc, merge, get-in, update-in, update
//! - 数値・比較（17個）: +, -, *, /, %, inc, dec, abs, min, max, sum, =, !=, <, >, <=, >=
//! - 文字列（3個）: str, split, join
//! - 述語・型判定（22個）: nil?, list?, vector?, map?, string?, integer?, float?, etc.
//! - 並行処理（5個）: go, chan, send!, recv!, close!
//! - 論理・I/O（4個）: not, print, println, error (※ and, or は特殊形式)
//! - 状態管理（4個）: atom, deref, swap!, reset!
//! - メタプログラミング（4個）: eval, uvar, variable, macro?
//! - 型変換（3個）: to-int, to-float, to-string
//! - 日時（3個）: now, timestamp, sleep
//! - デバッグ（2個）: time, inspect
//!
//! ## 専門モジュール - `module/function` 形式で使用
//! - list: 高度なリスト操作（18個）
//! - map: 高度なマップ操作（5個）
//! - fn: 高階関数（3個）
//! - set: 集合演算（7個）
//! - math: 数学関数（10個）
//! - time: 日時処理（25個）
//! - stats: 統計関数（6個）
//! - io: ファイルI/O（19個）
//! - path: パス操作（9個）
//! - env: 環境変数（4個）
//! - log: 構造化ログ（6個）
//! - async: 並行処理（高度）（16個）
//! - pipeline: パイプライン処理（5個）
//! - stream: ストリーム処理（11個）
//! - str: 文字列操作（62個）
//! - json: JSON処理（3個）
//! - yaml: YAML処理（3個）
//! - csv: CSV処理（5個）
//! - markdown: Markdown生成・解析（11個）
//! - http: HTTP通信（22個）
//! - db: データベース（17個）

// Lazy初期化サポート
pub mod lazy_init;

// Coreモジュール
pub mod core_collections;
pub mod core_functions;
pub mod core_io_logic;
pub mod core_numeric;
pub mod core_predicates;
pub mod core_state_meta;
pub mod core_string;
pub mod core_util;

// 専門モジュール
pub mod hof;

#[cfg(feature = "http-client")]
pub mod http;

pub mod io;

#[cfg(feature = "format-json")]
pub mod json;
pub mod list;
pub mod map;
pub mod math;
pub mod path;
#[cfg(feature = "format-yaml")]
pub mod yaml;

#[cfg(feature = "std-set")]
pub mod set;

#[cfg(feature = "std-stats")]
pub mod stats;

pub mod stream;
pub mod string;

#[cfg(feature = "std-time")]
pub mod time;

pub mod concurrency;
pub mod csv;
pub mod env;
pub mod flow;
pub mod log;
pub mod util;

#[cfg(feature = "util-zip")]
pub mod zip;

pub mod args;

#[cfg(feature = "io-temp")]
pub mod temp;

#[cfg(feature = "cmd-exec")]
pub mod cmd;

#[cfg(feature = "format-markdown")]
pub mod markdown;

pub mod ds;
pub mod profile;
pub mod test;

#[cfg(feature = "db-sqlite")]
pub mod db;

#[cfg(feature = "db-sqlite")]
pub mod sqlite;

#[cfg(feature = "db-postgres")]
pub mod postgres;

#[cfg(feature = "db-mysql")]
pub mod mysql;

#[cfg(feature = "kvs-redis")]
pub mod redis;

#[cfg(feature = "kvs-redis")]
pub mod kvs;

#[cfg(feature = "http-server")]
pub mod server;

#[cfg(feature = "auth-jwt")]
pub mod jwt;

#[cfg(feature = "auth-password")]
pub mod password;

use crate::eval::Evaluator;
use crate::value::{Env, NativeFunc, Value};
use parking_lot::RwLock;
use std::sync::Arc;

/// ネイティブ関数のシグネチャ（Evaluator不要）
pub type NativeFn = fn(&[Value]) -> Result<Value, String>;

/// Evaluator必要な関数のシグネチャ
pub type NativeEvalFn = fn(&[Value], &Evaluator) -> Result<Value, String>;

/// FUNCTIONS配列の型（Evaluator不要な関数用）
pub type NativeFunctions = &'static [(&'static str, NativeFn)];

/// Evaluator必要な関数の配列型（将来の拡張用）
#[allow(dead_code)]
pub type NativeEvalFunctions = &'static [(&'static str, NativeEvalFn)];

/// FUNCTIONS配列から関数を登録するヘルパー
fn register_functions(env: &mut Env, functions: NativeFunctions) {
    for (name, func) in functions {
        env.set(
            name.to_string(),
            Value::NativeFunc(NativeFunc { name, func: *func }),
        );
    }
}

/// Coreモジュール一覧（配列にまとめて保守性向上）
const CORE_MODULES: &[NativeFunctions] = &[
    core_numeric::FUNCTIONS,
    core_collections::FUNCTIONS,
    core_predicates::FUNCTIONS,
    core_string::FUNCTIONS,
    core_util::FUNCTIONS,
    core_io_logic::FUNCTIONS,
    core_functions::FUNCTIONS,
    core_state_meta::FUNCTIONS,
];

/// 標準専門モジュール一覧（feature-gatedでないもの）
const STANDARD_MODULES: &[NativeFunctions] = &[
    math::FUNCTIONS,
    csv::FUNCTIONS,
    path::FUNCTIONS,
    env::FUNCTIONS,
    log::FUNCTIONS,
    args::FUNCTIONS,
    test::FUNCTIONS,
    profile::FUNCTIONS,
    ds::FUNCTIONS,
    io::FUNCTIONS,
    string::FUNCTIONS,
    list::FUNCTIONS,
    map::FUNCTIONS,
    hof::FUNCTIONS,
    util::FUNCTIONS,
    stream::FUNCTIONS,
    concurrency::FUNCTIONS,
];

/// すべての組み込み関数を環境に登録
pub fn register_all(env: &Arc<RwLock<Env>>) {
    let mut env_write = env.write();

    // ========================================
    // Coreモジュール関数の登録（FUNCTIONS配列を使用）
    // ========================================
    for funcs in CORE_MODULES {
        register_functions(&mut env_write, funcs);
    }

    // ========================================
    // 標準専門モジュール（Evaluator不要）
    // ========================================
    for funcs in STANDARD_MODULES {
        register_functions(&mut env_write, funcs);
    }

    // Feature-gated専門モジュール（Evaluator不要）
    #[cfg(feature = "std-math")]
    register_functions(&mut env_write, math::FUNCTIONS_STD_MATH);

    #[cfg(feature = "string-encoding")]
    register_functions(&mut env_write, string::FUNCTIONS_STRING_ENCODING);

    #[cfg(feature = "string-crypto")]
    register_functions(&mut env_write, string::FUNCTIONS_STRING_CRYPTO);

    #[cfg(feature = "std-set")]
    register_functions(&mut env_write, set::FUNCTIONS);

    #[cfg(feature = "std-time")]
    register_functions(&mut env_write, time::FUNCTIONS);

    // Feature-gated専門モジュール（Evaluator不要、その他）
    #[cfg(feature = "format-markdown")]
    register_functions(&mut env_write, markdown::FUNCTIONS);

    #[cfg(feature = "format-json")]
    register_functions(&mut env_write, json::FUNCTIONS);

    #[cfg(feature = "format-yaml")]
    register_functions(&mut env_write, yaml::FUNCTIONS);

    #[cfg(feature = "http-client")]
    register_functions(&mut env_write, http::FUNCTIONS);

    #[cfg(feature = "http-server")]
    register_functions(&mut env_write, server::FUNCTIONS);

    #[cfg(feature = "std-stats")]
    register_functions(&mut env_write, stats::FUNCTIONS);

    #[cfg(feature = "util-zip")]
    register_functions(&mut env_write, zip::FUNCTIONS);

    #[cfg(feature = "io-temp")]
    register_functions(&mut env_write, temp::FUNCTIONS);

    #[cfg(feature = "cmd-exec")]
    register_functions(&mut env_write, cmd::FUNCTIONS);

    #[cfg(feature = "db-sqlite")]
    register_functions(&mut env_write, db::FUNCTIONS);

    #[cfg(feature = "db-postgres")]
    register_functions(&mut env_write, postgres::FUNCTIONS);

    #[cfg(feature = "db-mysql")]
    register_functions(&mut env_write, mysql::FUNCTIONS);

    // KVS統一インターフェース（kvs/*）のみ公開
    // redis::FUNCTIONSは内部実装用で、統一インターフェースから外れたRedis固有機能が必要な場合のみ追加
    #[cfg(feature = "kvs-redis")]
    register_functions(&mut env_write, kvs::FUNCTIONS);

    #[cfg(feature = "auth-jwt")]
    register_functions(&mut env_write, jwt::FUNCTIONS);

    #[cfg(feature = "auth-password")]
    register_functions(&mut env_write, password::FUNCTIONS);
}

// ========================================
// Evaluatorが必要な関数（mod.rsでラップ）
// ========================================
//
// 注: これらの関数はeval.rsから直接呼ばれるため、公開ラッパーとして定義されています。
//     将来的にはNativeEvalFunctions配列にまとめることで、新規追加時の抜け漏れを防ぐことができます。

/// map - リストの各要素に関数を適用
pub fn map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_map(args, evaluator)
}

/// filter - リストから条件を満たす要素のみ抽出
pub fn filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_filter(args, evaluator)
}

/// reduce - リストを畳み込み
pub fn reduce(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_reduce(args, evaluator)
}

/// pmap - 並列map
pub fn pmap(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_pmap(args, evaluator)
}

/// each - コレクションの各要素に関数を適用（副作用用）
pub fn each(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_each(args, evaluator)
}

/// comp - 関数合成
pub fn comp(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    core_functions::native_comp(args, evaluator)
}

/// apply - リストを引数として関数適用
pub fn apply(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    core_functions::native_apply(args, evaluator)
}

/// take-while - 条件を満たす間要素を取得
pub fn take_while(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_take_while(args, evaluator)
}

/// drop-while - 条件を満たす間要素をスキップ
pub fn drop_while(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_drop_while(args, evaluator)
}

/// find - リストから条件を満たす最初の要素を検索
pub fn find(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_find(args, evaluator)
}

/// every - すべての要素が条件を満たすか判定
pub fn every(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_every(args, evaluator)
}

/// some - いずれかの要素が条件を満たすか判定
pub fn some(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_some(args, evaluator)
}

/// update-in - ネストしたマップの値を更新
pub fn update_in(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_update_in(args, evaluator)
}

/// update - マップの値を関数で更新
pub fn update(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_update(args, evaluator)
}

/// swap! - アトムの値を関数で更新
pub fn swap(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    core_state_meta::native_swap(args, evaluator)
}

/// eval - 式を評価
pub fn eval(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    core_state_meta::native_eval(args, evaluator)
}

/// go/run - goroutine風の非同期実行
pub fn run(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_run(args, evaluator)
}

/// time - 関数実行時間を計測
pub fn time(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    util::native_time(args, evaluator)
}

/// tap - 副作用タップ（パイプライン内で括弧1つで使用可能）
pub fn tap(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_tap_direct(args, evaluator)
}

/// Railway Pipeline用の内部関数
pub fn railway_pipe(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    util::native_railway_pipe(args, evaluator)
}

// 以下、その他のEvaluator必要な関数

pub fn partition(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_partition(args, evaluator)
}

pub fn group_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_group_by(args, evaluator)
}

pub fn update_keys(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    map::native_update_keys(args, evaluator)
}

pub fn update_vals(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    map::native_update_vals(args, evaluator)
}

pub fn partition_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_partition_by(args, evaluator)
}

pub fn keep(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_keep(args, evaluator)
}

pub fn sort_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_sort_by(args, evaluator)
}

pub fn count_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_count_by(args, evaluator)
}

pub fn max_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_max_by(args, evaluator)
}

pub fn min_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_min_by(args, evaluator)
}

pub fn sum_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_sum_by(args, evaluator)
}

pub fn find_index(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_find_index(args, evaluator)
}

pub fn pfilter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_pfilter(args, evaluator)
}

pub fn preduce(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_preduce(args, evaluator)
}

pub fn map_lines(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    hof::native_map_lines(args, evaluator)
}

pub fn pipeline(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_pipeline(args, evaluator)
}

pub fn pipeline_map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_pipeline_map(args, evaluator)
}

pub fn pipeline_filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_pipeline_filter(args, evaluator)
}

pub fn then(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_then(args, evaluator)
}

pub fn catch(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_catch(args, evaluator)
}

pub fn select(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_select(args, evaluator)
}

pub fn with_scope(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_with_scope(args, evaluator)
}

pub fn scope_go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_scope_go(args, evaluator)
}

pub fn parallel_do(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    concurrency::native_parallel_do(args, evaluator)
}

pub fn iterate(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    stream::native_iterate(args, evaluator)
}

pub fn stream_map(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    stream::native_stream_map(args, evaluator)
}

pub fn stream_filter(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    stream::native_stream_filter(args, evaluator)
}

/// branch - 条件分岐（パイプライン用）
pub fn branch(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    flow::native_branch(args, evaluator)
}

/// test/run - テストを実行して結果を記録
pub fn test_run(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    test::native_test_run(args, evaluator)
}

/// test/assert-throws - 式が例外を投げることをアサート
pub fn test_assert_throws(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    test::native_assert_throws(args, evaluator)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Evaluator不要な関数が register_all で登録されているかチェック
    ///
    /// 注: map, filter, reduce, pmap, comp, apply, swap!, eval, go, time, tap, _railway-pipe,
    /// branch, test/run, test/assert-throws などの Evaluator 必要な関数は
    /// eval.rs:418-475 で特別処理されるため、register_all には登録されません。
    #[test]
    fn test_evaluator_independent_functions_registered() {
        let env = Arc::new(RwLock::new(Env::new()));
        register_all(&env);

        // Evaluator不要な基本関数が登録されているかチェック
        let basic_functions = vec![
            // Core関数
            "+",
            "-",
            "*",
            "/",
            "first",
            "rest",
            "last",
            "nth",
            "str",
            "split",
            "join",
            "print",
            "println",
            "atom",
            "deref",
            "reset!",
            // 専門モジュール関数
            "io/read-file",
            "io/write-file",
            "list/frequencies",
            "test/assert-eq",
            "test/assert",
            "inspect",
        ];

        let env_read = env.read();
        for func_name in basic_functions {
            assert!(
                env_read.get(func_name).is_some(),
                "Basic function '{}' is not registered in register_all!",
                func_name
            );
        }
    }

    /// HTTP関数が登録されているかチェック（feature-gated）
    #[test]
    #[cfg(feature = "http-client")]
    fn test_async_http_registered() {
        let env = Arc::new(RwLock::new(Env::new()));
        register_all(&env);

        let http_functions = vec![
            "http/get",
            "http/post",
            "http/put",
            "http/delete",
            "http/get-async",
            "http/post-async",
        ];

        let env_read = env.read();
        for func_name in http_functions {
            assert!(
                env_read.get(func_name).is_some(),
                "HTTP function '{}' is not registered in register_all!",
                func_name
            );
        }
    }

    /// モジュール別の登録数チェック（基準値）
    #[test]
    fn test_function_counts() {
        let env = Arc::new(RwLock::new(Env::new()));
        register_all(&env);

        let env_read = env.read();
        let all_bindings: Vec<_> = env_read.bindings().collect();

        // 最低限の関数数をチェック（新規追加で増えるのはOK、減るのはNG）
        assert!(
            all_bindings.len() >= 300,
            "Too few functions registered: {} (expected >= 300)",
            all_bindings.len()
        );
    }
}
