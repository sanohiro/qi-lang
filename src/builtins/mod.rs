//! 組み込み関数モジュール
//!
//! Qi-langは**2層モジュール設計**を採用しています：
//!
//! ## Core（90個）- グローバル名前空間、自動インポート
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
//! - デバッグ（1個）: time (dbg/time)
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
//! - dbg: デバッグ（2個）
//! - async: 並行処理（高度）（16個）
//! - pipeline: パイプライン処理（5個）
//! - stream: ストリーム処理（11個）
//! - str: 文字列操作（62個）
//! - json: JSON処理（3個）
//! - csv: CSV処理（5個）
//! - http: HTTP通信（11個）

// Coreモジュール
pub mod core_numeric;
pub mod core_collections;
pub mod core_predicates;
pub mod core_string;
pub mod core_util;
pub mod core_io_logic;
pub mod core_functions;
pub mod core_state_meta;
pub mod core_concurrency;

// 専門モジュール
pub mod hof;
pub mod http;
pub mod io;
pub mod json;
pub mod list;
pub mod map;
pub mod math;
pub mod path;
pub mod set;
pub mod stats;
pub mod stream;
pub mod string;
pub mod time;
pub mod concurrency;
pub mod util;
pub mod csv;
pub mod flow;
pub mod env;
pub mod log;
pub mod zip;
pub mod args;
pub mod temp;
pub mod test;

use crate::eval::Evaluator;
use crate::value::{Env, NativeFunc, Value};
use parking_lot::RwLock;
use std::sync::Arc;

/// 組み込み関数を登録するマクロ
macro_rules! register_native {
    ($env:expr, $($name:expr => $func:expr),* $(,)?) => {
        $(
            $env.set(
                $name.to_string(),
                Value::NativeFunc(NativeFunc {
                    name: $name.to_string(),
                    func: $func,
                }),
            );
        )*
    };
}

/// すべての組み込み関数を環境に登録
pub fn register_all(env: &Arc<RwLock<Env>>) {
    register_native!(env.write(),
        // ========================================
        // Core: 数値・比較演算（17個）
        // ========================================
        "+" => core_numeric::native_add,
        "-" => core_numeric::native_sub,
        "*" => core_numeric::native_mul,
        "/" => core_numeric::native_div,
        "%" => core_numeric::native_mod,
        "abs" => core_numeric::native_abs,
        "min" => core_numeric::native_min,
        "max" => core_numeric::native_max,
        "inc" => core_numeric::native_inc,
        "dec" => core_numeric::native_dec,
        "sum" => core_numeric::native_sum,
        "=" => core_numeric::native_eq,
        "!=" => core_numeric::native_ne,
        "<" => core_numeric::native_lt,
        ">" => core_numeric::native_gt,
        "<=" => core_numeric::native_le,
        ">=" => core_numeric::native_ge,

        // ========================================
        // Core: リスト操作（Evaluator不要な基本関数）
        // ========================================
        "first" => core_collections::native_first,
        "rest" => core_collections::native_rest,
        "last" => core_collections::native_last,
        "nth" => core_collections::native_nth,
        "len" => core_collections::native_len,
        "count" => core_collections::native_count,
        "cons" => core_collections::native_cons,
        "conj" => core_collections::native_conj,
        "concat" => core_collections::native_concat,
        "flatten" => core_collections::native_flatten,
        "range" => core_collections::native_range,
        "reverse" => core_collections::native_reverse,
        "take" => core_collections::native_take,
        "drop" => core_collections::native_drop,
        "sort" => core_collections::native_sort,
        "distinct" => core_collections::native_distinct,
        "zip" => core_collections::native_zip,

        // ========================================
        // Core: マップ操作（Evaluator不要）
        // ========================================
        "get" => core_collections::native_get,
        "keys" => core_collections::native_keys,
        "vals" => core_collections::native_vals,
        "assoc" => core_collections::native_assoc,
        "dissoc" => core_collections::native_dissoc,
        "merge" => core_collections::native_merge,
        "get-in" => core_collections::native_get_in,

        // ========================================
        // Core: 述語・型判定（20個）
        // ========================================
        "nil?" => core_predicates::native_nil,
        "list?" => core_predicates::native_list_q,
        "vector?" => core_predicates::native_vector_q,
        "map?" => core_predicates::native_map_q,
        "string?" => core_predicates::native_string_q,
        "integer?" => core_predicates::native_integer_q,
        "float?" => core_predicates::native_float_q,
        "number?" => core_predicates::native_number_q,
        "keyword?" => core_predicates::native_keyword_q,
        "function?" => core_predicates::native_function_q,
        "atom?" => core_predicates::native_atom_q,
        "coll?" => core_predicates::native_coll_q,
        "sequential?" => core_predicates::native_sequential_q,
        "empty?" => core_predicates::native_empty,
        "some?" => core_predicates::native_some_q,
        "true?" => core_predicates::native_true_q,
        "false?" => core_predicates::native_false_q,
        "even?" => core_predicates::native_even_q,
        "odd?" => core_predicates::native_odd_q,
        "positive?" => core_predicates::native_positive_q,
        "negative?" => core_predicates::native_negative_q,
        "zero?" => core_predicates::native_zero_q,

        // ========================================
        // Core: 文字列（3個）
        // ========================================
        "str" => core_string::native_str,
        "split" => core_string::native_split,
        "join" => core_string::native_join,

        // ========================================
        // Core: 型変換・日時（6個）
        // ========================================
        "to-int" => core_util::native_to_int,
        "to-float" => core_util::native_to_float,
        "to-string" => core_util::native_to_string,
        "now" => core_util::native_now,
        "timestamp" => core_util::native_timestamp,
        "sleep" => core_util::native_sleep,

        // ========================================
        // Core: I/O・論理・エラー（4個）
        // ========================================
        "print" => core_io_logic::native_print,
        "println" => core_io_logic::native_println,
        "not" => core_io_logic::native_not,
        "error" => core_io_logic::native_error,

        // ========================================
        // Core: 高階関数（Evaluator不要な関数）
        // ========================================
        "identity" => core_functions::native_identity,
        "constantly" => core_functions::native_constantly,
        "partial" => core_functions::native_partial,

        // ========================================
        // Core: 状態管理（Evaluator不要）
        // ========================================
        "atom" => core_state_meta::native_atom,
        "deref" => core_state_meta::native_deref,
        "reset!" => core_state_meta::native_reset,

        // ========================================
        // Core: メタプログラミング（Evaluator不要）
        // ========================================
        "uvar" => core_state_meta::native_uvar,
        "variable" => core_state_meta::native_variable,
        "macro?" => core_state_meta::native_macro_q,

        // ========================================
        // Core: 並行処理（Evaluator不要）
        // ========================================
        "chan" => core_concurrency::native_chan,
        "send!" => core_concurrency::native_send,
        "recv!" => core_concurrency::native_recv,
        "close!" => core_concurrency::native_close,

        // ========================================
        // 専門モジュール: list（18個）
        // ========================================
        "list/frequencies" => list::native_frequencies,
        "list/interleave" => list::native_interleave,
        "list/take-nth" => list::native_take_nth,
        "list/dedupe" => list::native_dedupe,
        "list/split-at" => list::native_split_at,
        "list/zipmap" => list::native_zipmap,
        "list/chunk" => list::native_chunk,
        "list/drop-last" => list::native_drop_last,

        // ========================================
        // 専門モジュール: map（5個）
        // ========================================
        "map/select-keys" => map::native_select_keys,
        "map/assoc-in" => map::native_assoc_in,
        "map/dissoc-in" => map::native_dissoc_in,

        // ========================================
        // 専門モジュール: fn（3個）
        // ========================================
        "fn/complement" => hof::native_complement,
        "fn/juxt" => hof::native_juxt,
        "fn/tap>" => hof::native_tap,

        // ========================================
        // 専門モジュール: set（7個）
        // ========================================
        "set/union" => set::native_union,
        "set/intersect" => set::native_intersect,
        "set/difference" => set::native_difference,
        "set/subset?" => set::native_subset,
        "set/superset?" => set::native_superset,
        "set/disjoint?" => set::native_disjoint,
        "set/symmetric-difference" => set::native_symmetric_difference,

        // ========================================
        // 専門モジュール: math（10個）
        // ========================================
        "math/pow" => math::native_pow,
        "math/sqrt" => math::native_sqrt,
        "math/round" => math::native_round,
        "math/floor" => math::native_floor,
        "math/ceil" => math::native_ceil,
        "math/clamp" => math::native_clamp,
        "math/rand" => math::native_rand,
        "math/rand-int" => math::native_rand_int,
        "math/random-range" => math::native_random_range,
        "math/shuffle" => math::native_shuffle,

        // ========================================
        // 専門モジュール: time（25個）
        // ========================================
        "time/now-iso" => time::native_now_iso,
        "time/today" => time::native_today,
        "time/from-unix" => time::native_from_unix,
        "time/to-unix" => time::native_to_unix,
        "time/format" => time::native_format,
        "time/parse" => time::native_parse,
        "time/add-days" => time::native_add_days,
        "time/add-hours" => time::native_add_hours,
        "time/add-minutes" => time::native_add_minutes,
        "time/sub-days" => time::native_sub_days,
        "time/sub-hours" => time::native_sub_hours,
        "time/sub-minutes" => time::native_sub_minutes,
        "time/diff-days" => time::native_diff_days,
        "time/diff-hours" => time::native_diff_hours,
        "time/diff-minutes" => time::native_diff_minutes,
        "time/before?" => time::native_before,
        "time/after?" => time::native_after,
        "time/between?" => time::native_between,
        "time/year" => time::native_year,
        "time/month" => time::native_month,
        "time/day" => time::native_day,
        "time/hour" => time::native_hour,
        "time/minute" => time::native_minute,
        "time/second" => time::native_second,
        "time/weekday" => time::native_weekday,

        // ========================================
        // 専門モジュール: stats（6個）
        // ========================================
        "stats/mean" => stats::native_mean,
        "stats/median" => stats::native_median,
        "stats/mode" => stats::native_mode,
        "stats/variance" => stats::native_variance,
        "stats/stddev" => stats::native_stddev,
        "stats/percentile" => stats::native_percentile,

        // ========================================
        // 専門モジュール: csv（5個）
        // ========================================
        "csv/parse" => csv::native_csv_parse,
        "csv/stringify" => csv::native_csv_stringify,
        "csv/read-file" => csv::native_csv_read_file,
        "csv/write-file" => csv::native_csv_write_file,
        "csv/read-stream" => csv::native_csv_read_stream,

        // ========================================
        // 専門モジュール: io（19個）
        // ========================================
        "io/read-file" => io::native_read_file,
        "io/write-file" => io::native_write_file,
        "io/append-file" => io::native_append_file,
        "io/write-stream" => io::native_write_stream,
        "io/read-lines" => io::native_read_lines,
        "io/file-exists?" => io::native_file_exists,
        "io/list-dir" => io::native_list_dir,
        "io/create-dir" => io::native_create_dir,
        "io/delete-file" => io::native_delete_file,
        "io/delete-dir" => io::native_delete_dir,
        "io/copy-file" => io::native_copy_file,
        "io/move-file" => io::native_move_file,
        "io/file-info" => io::native_file_info,
        "io/is-file?" => io::native_is_file,
        "io/is-dir?" => io::native_is_dir,

        // ========================================
        // 専門モジュール: path（9個）
        // ========================================
        "path/join" => path::native_path_join,
        "path/basename" => path::native_path_basename,
        "path/dirname" => path::native_path_dirname,
        "path/extension" => path::native_path_extension,
        "path/stem" => path::native_path_stem,
        "path/absolute" => path::native_path_absolute,
        "path/normalize" => path::native_path_normalize,
        "path/is-absolute?" => path::native_path_is_absolute,
        "path/is-relative?" => path::native_path_is_relative,

        // ========================================
        // 専門モジュール: env（4個）
        // ========================================
        "env/get" => env::native_env_get,
        "env/set" => env::native_env_set,
        "env/all" => env::native_env_all,
        "env/load-dotenv" => env::native_env_load_dotenv,

        // ========================================
        // 専門モジュール: log（6個）
        // ========================================
        "log/debug" => log::native_log_debug,
        "log/info" => log::native_log_info,
        "log/warn" => log::native_log_warn,
        "log/error" => log::native_log_error,
        "log/set-level" => log::native_log_set_level,
        "log/set-format" => log::native_log_set_format,

        // ========================================
        // 専門モジュール: zip（6個）
        // ========================================
        "zip/create" => zip::native_zip_create,
        "zip/extract" => zip::native_zip_extract,
        "zip/list" => zip::native_zip_list,
        "zip/add" => zip::native_zip_add,
        "zip/gzip" => zip::native_gzip,
        "zip/gunzip" => zip::native_gunzip,

        // ========================================
        // 専門モジュール: args（4個）
        // ========================================
        "args/all" => args::native_args_all,
        "args/get" => args::native_args_get,
        "args/parse" => args::native_args_parse,
        "args/count" => args::native_args_count,

        // ========================================
        // 専門モジュール: temp（4個）
        // ========================================
        "io/temp-file" => temp::native_temp_file,
        "io/temp-file-keep" => temp::native_temp_file_keep,
        "io/temp-dir" => temp::native_temp_dir,
        "io/temp-dir-keep" => temp::native_temp_dir_keep,

        // ========================================
        // 専門モジュール: test（4個）
        // ========================================
        "test/assert-eq" => test::native_assert_eq,
        "test/assert" => test::native_assert,
        "test/assert-not" => test::native_assert_not,
        "test/run-all" => test::native_run_all,
        "test/clear" => test::native_test_clear,

        // ========================================
        // 専門モジュール: dbg（2個）
        // ========================================
        "dbg/inspect" => util::native_inspect,

        // ========================================
        // 専門モジュール: json（3個）
        // ========================================
        "json/parse" => json::native_parse,
        "json/stringify" => json::native_stringify,
        "json/pretty" => json::native_pretty,

        // ========================================
        // 専門モジュール: http（11個）
        // ========================================
        "http/get" => http::native_get,
        "http/post" => http::native_post,
        "http/put" => http::native_put,
        "http/delete" => http::native_delete,
        "http/patch" => http::native_patch,
        "http/head" => http::native_head,
        "http/options" => http::native_options,
        "http/request" => http::native_request,

        // ========================================
        // 専門モジュール: stream（11個）
        // ========================================
        "stream/stream" => stream::native_stream,
        "stream/range" => stream::native_range_stream,
        "stream/repeat" => stream::native_repeat,
        "stream/cycle" => stream::native_cycle,
        "stream/take" => stream::native_stream_take,
        "stream/drop" => stream::native_stream_drop,
        "stream/realize" => stream::native_realize,
        "stream/file" => io::native_file_stream,

        // ========================================
        // 専門モジュール: str（62個）
        // ========================================
        "str/upper" => string::native_upper,
        "str/lower" => string::native_lower,
        "str/trim" => string::native_trim,
        "str/contains?" => string::native_contains,
        "str/starts-with?" => string::native_starts_with,
        "str/ends-with?" => string::native_ends_with,
        "str/index-of" => string::native_index_of,
        "str/last-index-of" => string::native_last_index_of,
        "str/slice" => string::native_slice,
        "str/take" => string::native_take_str,
        "str/drop" => string::native_drop_str,
        "str/sub-before" => string::native_sub_before,
        "str/sub-after" => string::native_sub_after,
        "str/replace" => string::native_replace,
        "str/replace-first" => string::native_replace_first,
        "str/lines" => string::native_lines,
        "str/words" => string::native_words,
        "str/capitalize" => string::native_capitalize,
        "str/trim-left" => string::native_trim_left,
        "str/trim-right" => string::native_trim_right,
        "str/repeat" => string::native_repeat,
        "str/chars-count" => string::native_chars_count,
        "str/bytes-count" => string::native_bytes_count,
        "str/digit?" => string::native_digit_p,
        "str/alpha?" => string::native_alpha_p,
        "str/alnum?" => string::native_alnum_p,
        "str/space?" => string::native_space_p,
        "str/lower?" => string::native_lower_p,
        "str/upper?" => string::native_upper_p,
        "str/pad-left" => string::native_pad_left,
        "str/pad-right" => string::native_pad_right,
        "str/pad" => string::native_pad,
        "str/squish" => string::native_squish,
        "str/expand-tabs" => string::native_expand_tabs,
        "str/title" => string::native_title,
        "str/reverse" => string::native_reverse,
        "str/chars" => string::native_chars,
        "str/snake" => string::native_snake,
        "str/camel" => string::native_camel,
        "str/kebab" => string::native_kebab,
        "str/pascal" => string::native_pascal,
        "str/split-camel" => string::native_split_camel,
        "str/truncate" => string::native_truncate,
        "str/trunc-words" => string::native_trunc_words,
        "str/splice" => string::native_splice,
        "str/numeric?" => string::native_numeric_p,
        "str/integer?" => string::native_integer_p,
        "str/blank?" => string::native_blank_p,
        "str/ascii?" => string::native_ascii_p,
        "str/indent" => string::native_indent,
        "str/wrap" => string::native_wrap,
        "str/parse-int" => string::native_parse_int,
        "str/parse-float" => string::native_parse_float,
        "str/slugify" => string::native_slugify,
        "str/word-count" => string::native_word_count,
        "str/to-base64" => string::native_to_base64,
        "str/from-base64" => string::native_from_base64,
        "str/url-encode" => string::native_url_encode,
        "str/url-decode" => string::native_url_decode,
        "str/html-encode" => string::native_html_encode,
        "str/html-decode" => string::native_html_decode,
        "str/hash" => string::native_hash,
        "str/uuid" => string::native_uuid,
        "str/re-find" => string::native_re_find,
        "str/re-matches" => string::native_re_matches,
        "str/re-replace" => string::native_re_replace,
        "str/format" => string::native_format,
        "str/format-decimal" => string::native_format_decimal,
        "str/format-comma" => string::native_format_comma,
        "str/format-percent" => string::native_format_percent,

        // ========================================
        // async/pipeline モジュール（Evaluator不要な関数）
        // ========================================
        "async/try-recv!" => concurrency::native_try_recv,
        "async/make-scope" => concurrency::native_make_scope,
        "async/cancel!" => concurrency::native_cancel,
        "async/cancelled?" => concurrency::native_cancelled_q,
        "async/await" => concurrency::native_await,
        "async/all" => concurrency::native_all,
        "async/race" => concurrency::native_race,
        "pipeline/fan-out" => concurrency::native_fan_out,
        "pipeline/fan-in" => concurrency::native_fan_in,
    );
}

// ========================================
// Evaluatorが必要な関数（mod.rsでラップ）
// ========================================

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

/// go - goroutine風の非同期実行
pub fn go(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    core_concurrency::native_go(args, evaluator)
}

/// time - 関数実行時間を計測（dbg/time）
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

pub fn split_at(args: &[Value], _evaluator: &Evaluator) -> Result<Value, String> {
    list::native_split_at(args)
}

pub fn drop_last(args: &[Value], _evaluator: &Evaluator) -> Result<Value, String> {
    list::native_drop_last(args)
}

pub fn sort_by(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    list::native_sort_by(args, evaluator)
}

pub fn chunk(args: &[Value], _evaluator: &Evaluator) -> Result<Value, String> {
    list::native_chunk(args)
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

pub fn http_get_async(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    http::native_get_async(args, evaluator)
}

pub fn http_post_async(args: &[Value], evaluator: &Evaluator) -> Result<Value, String> {
    http::native_post_async(args, evaluator)
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
