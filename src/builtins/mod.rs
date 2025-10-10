//! 組み込み関数モジュール
//!
//! このモジュールは組み込み関数を機能別に整理しています:
//! - arithmetic: 算術演算（+, -, *, /, %, abs, min, max, inc, dec, sum）
//! - comparison: 比較演算（=, !=, <, >, <=, >=）
//! - list: リスト操作（first, rest, nth, cons, conj, take, drop, concat, flatten, range）
//! - string: 文字列操作（str, split, join, upper, lower, trim）
//! - map: マップ操作（get, keys, vals, assoc, dissoc）
//! - predicates: 述語関数（empty?, nil?, list?, vector?, map?, string?, integer?, float?, keyword?）
//! - logic: 論理演算（not）
//! - hof: 高階関数（map, filter, reduce）
//! - error: エラー処理（error）

pub mod arithmetic;
pub mod comparison;
pub mod error;
pub mod hof;
pub mod list;
pub mod logic;
pub mod map;
pub mod predicates;
pub mod string;

use crate::eval::Evaluator;
use crate::value::{Env, NativeFunc, Value};
use std::cell::RefCell;
use std::rc::Rc;

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
pub fn register_all(env: &Rc<RefCell<Env>>) {
    register_native!(env.borrow_mut(),
        // 算術演算
        "+" => arithmetic::native_add,
        "-" => arithmetic::native_sub,
        "*" => arithmetic::native_mul,
        "/" => arithmetic::native_div,
        "%" => arithmetic::native_mod,
        "abs" => arithmetic::native_abs,
        "min" => arithmetic::native_min,
        "max" => arithmetic::native_max,
        "inc" => arithmetic::native_inc,
        "dec" => arithmetic::native_dec,
        "sum" => arithmetic::native_sum,

        // 比較演算
        "=" => comparison::native_eq,
        "!=" => comparison::native_ne,
        "<" => comparison::native_lt,
        ">" => comparison::native_gt,
        "<=" => comparison::native_le,
        ">=" => comparison::native_ge,

        // リスト操作
        "first" => list::native_first,
        "rest" => list::native_rest,
        "len" => list::native_len,
        "count" => list::native_count,
        "nth" => list::native_nth,
        "reverse" => list::native_reverse,
        "cons" => list::native_cons,
        "conj" => list::native_conj,
        "take" => list::native_take,
        "drop" => list::native_drop,
        "concat" => list::native_concat,
        "flatten" => list::native_flatten,
        "range" => list::native_range,
        "last" => list::native_last,
        "zip" => list::native_zip,

        // 文字列操作
        "str" => string::native_str,
        "split" => string::native_split,
        "join" => string::native_join,
        "upper" => string::native_upper,
        "lower" => string::native_lower,
        "trim" => string::native_trim,
        "contains?" => string::native_contains,
        "starts-with?" => string::native_starts_with,
        "ends-with?" => string::native_ends_with,
        "index-of" => string::native_index_of,
        "last-index-of" => string::native_last_index_of,
        "slice" => string::native_slice,
        "take-str" => string::native_take_str,
        "drop-str" => string::native_drop_str,
        "sub-before" => string::native_sub_before,
        "sub-after" => string::native_sub_after,
        "replace" => string::native_replace,
        "replace-first" => string::native_replace_first,
        "lines" => string::native_lines,
        "words" => string::native_words,
        "capitalize" => string::native_capitalize,
        "trim-left" => string::native_trim_left,
        "trim-right" => string::native_trim_right,
        "repeat" => string::native_repeat,
        "chars-count" => string::native_chars_count,
        "bytes-count" => string::native_bytes_count,
        "digit?" => string::native_digit_p,
        "alpha?" => string::native_alpha_p,
        "alnum?" => string::native_alnum_p,
        "space?" => string::native_space_p,
        "lower?" => string::native_lower_p,
        "upper?" => string::native_upper_p,
        "pad-left" => string::native_pad_left,
        "pad-right" => string::native_pad_right,
        "pad" => string::native_pad,
        "squish" => string::native_squish,
        "expand-tabs" => string::native_expand_tabs,
        "title" => string::native_title,
        "reverse" => string::native_reverse,
        "chars" => string::native_chars,
        "snake" => string::native_snake,
        "camel" => string::native_camel,
        "kebab" => string::native_kebab,
        "pascal" => string::native_pascal,
        "split-camel" => string::native_split_camel,
        "align-left" => string::native_align_left,
        "align-center" => string::native_align_center,
        "align-right" => string::native_align_right,
        "truncate" => string::native_truncate,
        "trunc-words" => string::native_trunc_words,
        "splice" => string::native_splice,
        "numeric?" => string::native_numeric_p,
        "integer?" => string::native_integer_p,
        "blank?" => string::native_blank_p,
        "ascii?" => string::native_ascii_p,
        "indent" => string::native_indent,
        "wrap" => string::native_wrap,
        "parse-int" => string::native_parse_int,
        "parse-float" => string::native_parse_float,
        "slugify" => string::native_slugify,
        "word-count" => string::native_word_count,
        "to-base64" => string::native_to_base64,
        "from-base64" => string::native_from_base64,
        "url-encode" => string::native_url_encode,
        "url-decode" => string::native_url_decode,
        "html-escape" => string::native_html_escape,
        "html-unescape" => string::native_html_unescape,
        "hash" => string::native_hash,
        "uuid" => string::native_uuid,

        // マップ操作
        "get" => map::native_get,
        "keys" => map::native_keys,
        "vals" => map::native_vals,
        "assoc" => map::native_assoc,
        "dissoc" => map::native_dissoc,
        "merge" => map::native_merge,
        "select-keys" => map::native_select_keys,

        // 述語関数
        "empty?" => predicates::native_empty,
        "nil?" => predicates::native_nil,
        "list?" => predicates::native_list_q,
        "vector?" => predicates::native_vector_q,
        "map?" => predicates::native_map_q,
        "string?" => predicates::native_string_q,
        "integer?" => predicates::native_integer_q,
        "float?" => predicates::native_float_q,
        "keyword?" => predicates::native_keyword_q,

        // 論理演算
        "not" => logic::native_not,

        // エラー処理
        "error" => error::native_error,
    );
}

/// 高階関数を登録（Evaluatorへの参照が必要なため別扱い）
pub fn map(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    hof::native_map(args, evaluator)
}

pub fn filter(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    hof::native_filter(args, evaluator)
}

pub fn reduce(args: &[Value], evaluator: &mut Evaluator) -> Result<Value, String> {
    hof::native_reduce(args, evaluator)
}
