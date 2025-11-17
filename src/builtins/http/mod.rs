//! HTTPクライアントモジュール
//!
//! HTTP通信関数を提供:
//! - get/post/put/delete/patch/head/options: 各HTTPメソッド
//! - request: 詳細なリクエスト設定
//! - get-async/post-async: 非同期版
//! - get-stream/post-stream/request-stream: ストリーミング版
//!
//! このモジュールは `http-client` feature でコンパイルされます。

use crate::i18n::{fmt_msg, MsgKey};
use crate::value::{Stream, Value};
use crossbeam_channel::bounded;
use flate2::write::GzEncoder;
use flate2::Compression;
use parking_lot::RwLock;
use reqwest::blocking::Client;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

mod helpers;
mod simple;
mod detailed;
mod request;
mod async_ops;
mod stream;
mod core;

pub use simple::*;
pub use detailed::*;
pub use request::*;
pub use async_ops::*;
pub use stream::*;

// ========================================
// 関数登録テーブル
// ========================================

/// 登録すべき関数のリスト
/// @qi-doc:category net/http
/// @qi-doc:functions get, post, put, delete, patch, head, options, request, get!, post!, put!, delete!, patch!, head!, options!, get-stream, post-stream, request-stream
pub const FUNCTIONS: super::NativeFunctions = &[
    // シンプル版（bodyのみ返す）
    ("http/get", native_get),
    ("http/post", native_post),
    ("http/put", native_put),
    ("http/delete", native_delete),
    ("http/patch", native_patch),
    ("http/head", native_head),
    ("http/options", native_options),
    // 詳細版（!付き、Map形式で詳細情報を返す）
    ("http/get!", native_get_bang),
    ("http/post!", native_post_bang),
    ("http/put!", native_put_bang),
    ("http/delete!", native_delete_bang),
    ("http/patch!", native_patch_bang),
    ("http/head!", native_head_bang),
    ("http/options!", native_options_bang),
    // 詳細制御版（元からの仕様）
    ("http/request", native_request),
    // 非同期版
    ("http/get-async", native_get_async),
    ("http/post-async", native_post_async),
    // ストリーミング版
    ("http/get-stream", native_get_stream),
    ("http/post-stream", native_post_stream),
    ("http/request-stream", native_request_stream),
];
