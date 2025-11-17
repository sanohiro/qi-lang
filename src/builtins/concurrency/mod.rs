//! 並行処理関数（go/chan）

pub mod channel;
pub mod pipeline;
pub mod promise;
pub mod scope;

pub use channel::*;
pub use pipeline::*;
pub use promise::*;
pub use scope::*;

/// 登録すべき関数のリスト（Evaluator不要な関数のみ）
/// @qi-doc:category go
/// @qi-doc:functions chan, send!, recv!, close!, chan-closed?, then, catch, go, pipeline, pipeline-map, pipeline-filter, select!, atom, swap!, reset!, deref, scope, scope-go, with-scope, parallel-do
///
/// 注意: then, catch, go, pipeline, pipeline-map, pipeline-filter, select!, scope-go, with-scope, parallel-do
/// はEvaluatorが必要なため、mod.rsで別途登録されます
pub const FUNCTIONS: super::NativeFunctions = &[
    ("go/chan", native_chan),
    ("go/send!", native_send),
    ("go/recv!", native_recv),
    ("go/try-recv!", native_try_recv),
    ("go/close!", native_close),
    ("go/await", native_await),
    ("go/all", native_all),
    ("go/race", native_race),
    ("go/fan-out", native_fan_out),
    ("go/fan-in", native_fan_in),
    ("go/make-scope", native_make_scope),
    ("go/cancel!", native_cancel),
    ("go/cancelled?", native_cancelled_q),
];
