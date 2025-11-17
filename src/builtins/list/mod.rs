//! リスト操作関数
//!
//! 注: 基本的なリスト操作(first, rest, take, drop等)はcore_collections.rsで実装されています。
//! このモジュールには高度なリスト操作のみを含みます。

pub mod aggregation;
pub mod helpers;
pub mod partition;
pub mod predicates;
pub mod transform;

pub use aggregation::*;
pub use partition::*;
pub use predicates::*;
pub use transform::*;

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category list
/// @qi-doc:functions take-while, drop-while, split-at, interleave, frequencies, sort-by, chunk, max-by, min-by, sum-by, find, find-index, every?, some?, zipmap, partition-by, take-nth, keep, dedupe, drop-last
///
/// 注意: take-while, drop-while, sort-by, max-by, min-by, sum-by, find, find-index,
/// every?, some?, partition-by, keepはEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("list/split-at", native_split_at),
    ("list/interleave", native_interleave),
    ("list/frequencies", native_frequencies),
    ("list/chunk", native_chunk),
    ("list/zipmap", native_zipmap),
    ("list/take-nth", native_take_nth),
    ("list/dedupe", native_dedupe),
    ("list/drop-last", native_drop_last),
];
