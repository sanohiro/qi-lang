//! アプリケーション全体で使用される定数
//!
//! ハードコードされた文字列を定数化し、`to_string()` の呼び出しを削減します。

use crate::value::MapKey;
use std::sync::Arc;

/// DAPプロトコル関連の定数
pub mod dap {
    pub const MSG_TYPE_RESPONSE: &str = "response";
    pub const MSG_TYPE_REQUEST: &str = "request";
    pub const MSG_TYPE_EVENT: &str = "event";

    pub const COMMAND_INITIALIZE: &str = "initialize";
    pub const COMMAND_LAUNCH: &str = "launch";
    pub const COMMAND_ATTACH: &str = "attach";
    pub const COMMAND_DISCONNECT: &str = "disconnect";
    pub const COMMAND_TERMINATE: &str = "terminate";
    pub const COMMAND_SET_BREAKPOINTS: &str = "setBreakpoints";
    pub const COMMAND_SET_EXCEPTION_BREAKPOINTS: &str = "setExceptionBreakpoints";
    pub const COMMAND_CONTINUE: &str = "continue";
    pub const COMMAND_NEXT: &str = "next";
    pub const COMMAND_STEP_IN: &str = "stepIn";
    pub const COMMAND_STEP_OUT: &str = "stepOut";
    pub const COMMAND_PAUSE: &str = "pause";
    pub const COMMAND_STACK_TRACE: &str = "stackTrace";
    pub const COMMAND_SCOPES: &str = "scopes";
    pub const COMMAND_VARIABLES: &str = "variables";
    pub const COMMAND_EVALUATE: &str = "evaluate";
    pub const COMMAND_THREADS: &str = "threads";
    pub const COMMAND_CONFIGURATION_DONE: &str = "configurationDone";

    pub const EVENT_INITIALIZED: &str = "initialized";
    pub const EVENT_STOPPED: &str = "stopped";
    pub const EVENT_CONTINUED: &str = "continued";
    pub const EVENT_EXITED: &str = "exited";
    pub const EVENT_TERMINATED: &str = "terminated";
    pub const EVENT_THREAD: &str = "thread";
    pub const EVENT_OUTPUT: &str = "output";
    pub const EVENT_BREAKPOINT: &str = "breakpoint";

    pub const REASON_BREAKPOINT: &str = "breakpoint";
    pub const REASON_STEP: &str = "step";
    pub const REASON_PAUSE: &str = "pause";
    pub const REASON_EXCEPTION: &str = "exception";
    pub const REASON_ENTRY: &str = "entry";
}

/// HTTPサーバー関連の定数
pub mod server {
    pub const MIDDLEWARE_BASIC_AUTH: &str = "basic-auth";
    pub const MIDDLEWARE_CORS: &str = "cors";
    pub const MIDDLEWARE_LOGGER: &str = "logger";
    pub const MIDDLEWARE_STATIC: &str = "static";

    pub const KEY_STATIC_DIR: &str = "__static_dir__";
    pub const KEY_MIDDLEWARE: &str = "__middleware__";
    pub const KEY_HANDLER: &str = "__handler__";
    pub const KEY_USERS: &str = "__users__";
    pub const KEY_REALM: &str = "__realm__";
    pub const KEY_ORIGINS: &str = "__origins__";
    pub const KEY_METHODS: &str = "__methods__";
    pub const KEY_HEADERS: &str = "__headers__";
    pub const KEY_CREDENTIALS: &str = "__credentials__";

    pub const HEADER_AUTHORIZATION: &str = "authorization";
    pub const HEADER_CONTENT_TYPE: &str = "content-type";
    pub const HEADER_ACCEPT: &str = "accept";
    pub const HEADER_ORIGIN: &str = "origin";
    pub const HEADER_ACCESS_CONTROL_ALLOW_ORIGIN: &str = "access-control-allow-origin";
    pub const HEADER_ACCESS_CONTROL_ALLOW_METHODS: &str = "access-control-allow-methods";
    pub const HEADER_ACCESS_CONTROL_ALLOW_HEADERS: &str = "access-control-allow-headers";
    pub const HEADER_ACCESS_CONTROL_ALLOW_CREDENTIALS: &str = "access-control-allow-credentials";
    pub const HEADER_WWW_AUTHENTICATE: &str = "www-authenticate";

    pub const METHOD_GET: &str = "GET";
    pub const METHOD_POST: &str = "POST";
    pub const METHOD_PUT: &str = "PUT";
    pub const METHOD_DELETE: &str = "DELETE";
    pub const METHOD_PATCH: &str = "PATCH";
    pub const METHOD_HEAD: &str = "HEAD";
    pub const METHOD_OPTIONS: &str = "OPTIONS";

    pub const CONTENT_TYPE_JSON: &str = "application/json";
    pub const CONTENT_TYPE_HTML: &str = "text/html";
    pub const CONTENT_TYPE_TEXT: &str = "text/plain";
    pub const CONTENT_TYPE_CSS: &str = "text/css";
    pub const CONTENT_TYPE_JS: &str = "application/javascript";

    pub const STATUS_OK: i64 = 200;
    pub const STATUS_CREATED: i64 = 201;
    pub const STATUS_NO_CONTENT: i64 = 204;
    pub const STATUS_BAD_REQUEST: i64 = 400;
    pub const STATUS_UNAUTHORIZED: i64 = 401;
    pub const STATUS_FORBIDDEN: i64 = 403;
    pub const STATUS_NOT_FOUND: i64 = 404;
    pub const STATUS_METHOD_NOT_ALLOWED: i64 = 405;
    pub const STATUS_INTERNAL_SERVER_ERROR: i64 = 500;
}

/// 特殊形式のキーワード
pub mod special_forms {
    pub const DEF: &str = "def";
    pub const DEFN: &str = "defn";
    pub const DEFN_PRIVATE: &str = "defn-";
    pub const LET: &str = "let";
    pub const IF: &str = "if";
    pub const DO: &str = "do";
    pub const FN: &str = "fn";
    pub const QUOTE: &str = "quote";
    pub const QUASIQUOTE: &str = "quasiquote";
    pub const UNQUOTE: &str = "unquote";
    pub const UNQUOTE_SPLICING: &str = "unquote-splicing";
    pub const TRY: &str = "try";
    pub const CATCH: &str = "catch";
    pub const FINALLY: &str = "finally";
    pub const THROW: &str = "throw";
    pub const MATCH: &str = "match";
    pub const LOOP: &str = "loop";
    pub const RECUR: &str = "recur";
    pub const DEFER: &str = "defer";
}

/// よく使われるキーワード
pub mod keywords {
    use super::*;

    pub const OK: &str = "ok";
    pub const ERROR: &str = "error";
    pub const STATUS: &str = "status";
    pub const BODY: &str = "body";
    pub const HEADERS: &str = "headers";
    pub const METHOD: &str = "method";
    pub const PATH: &str = "path";
    pub const QUERY: &str = "query";
    pub const PARAMS: &str = "params";
    pub const MESSAGE: &str = "message";
    pub const CODE: &str = "code";
    pub const DATA: &str = "data";

    // マップキー用（:プレフィックス付き文字列、後方互換性）
    pub const ERROR_KEY: &str = ":error";
    pub const OK_KEY: &str = ":ok";
    pub const STATUS_KEY: &str = ":status";
    pub const BODY_KEY: &str = ":body";
    pub const HEADERS_KEY: &str = ":headers";
    pub const METHOD_KEY: &str = ":method";
    pub const PATH_KEY: &str = ":path";
    pub const QUERY_KEY: &str = ":query";
    pub const PARAMS_KEY: &str = ":params";
    pub const MESSAGE_KEY: &str = ":message";
    pub const CODE_KEY: &str = ":code";
    pub const DATA_KEY: &str = ":data";

    // MapKey定数（型安全）
    pub fn error_mapkey() -> MapKey { MapKey::Keyword(Arc::from(ERROR)) }
    pub fn ok_mapkey() -> MapKey { MapKey::Keyword(Arc::from(OK)) }
    pub fn status_mapkey() -> MapKey { MapKey::Keyword(Arc::from(STATUS)) }
    pub fn body_mapkey() -> MapKey { MapKey::Keyword(Arc::from(BODY)) }
    pub fn headers_mapkey() -> MapKey { MapKey::Keyword(Arc::from(HEADERS)) }
    pub fn method_mapkey() -> MapKey { MapKey::Keyword(Arc::from(METHOD)) }
    pub fn path_mapkey() -> MapKey { MapKey::Keyword(Arc::from(PATH)) }
    pub fn query_mapkey() -> MapKey { MapKey::Keyword(Arc::from(QUERY)) }
    pub fn params_mapkey() -> MapKey { MapKey::Keyword(Arc::from(PARAMS)) }
    pub fn message_mapkey() -> MapKey { MapKey::Keyword(Arc::from(MESSAGE)) }
    pub fn code_mapkey() -> MapKey { MapKey::Keyword(Arc::from(CODE)) }
    pub fn data_mapkey() -> MapKey { MapKey::Keyword(Arc::from(DATA)) }
}

/// ファイルI/O関連の定数
pub mod io {
    pub const ENCODING_UTF8: &str = "utf-8";
    pub const ENCODING_SHIFT_JIS: &str = "shift_jis";
    pub const ENCODING_EUC_JP: &str = "euc-jp";
    pub const ENCODING_ISO_2022_JP: &str = "iso-2022-jp";

    pub const MODE_READ: &str = "r";
    pub const MODE_WRITE: &str = "w";
    pub const MODE_APPEND: &str = "a";
    pub const MODE_READ_WRITE: &str = "r+";
}

/// コレクション操作の定数
pub mod collections {
    pub const EMPTY_VEC_CAPACITY: usize = 0;
    pub const DEFAULT_VEC_CAPACITY: usize = 16;
    pub const LARGE_VEC_CAPACITY: usize = 256;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dap_constants() {
        assert_eq!(dap::MSG_TYPE_RESPONSE, "response");
        assert_eq!(dap::COMMAND_INITIALIZE, "initialize");
        assert_eq!(dap::EVENT_INITIALIZED, "initialized");
    }

    #[test]
    fn test_server_constants() {
        assert_eq!(server::MIDDLEWARE_BASIC_AUTH, "basic-auth");
        assert_eq!(server::METHOD_GET, "GET");
        assert_eq!(server::STATUS_OK, 200);
    }

    #[test]
    fn test_special_forms() {
        assert_eq!(special_forms::DEF, "def");
        assert_eq!(special_forms::DEFN, "defn");
        assert_eq!(special_forms::IF, "if");
    }
}
