//! 文字列インターン
//!
//! シンボルとキーワードの重複文字列を削減するため、
//! グローバルなインターンテーブルで文字列を管理する。
//!
//! インターンされた文字列は Arc<str> で管理され、
//! clone() 時のコピーコストを削減する。

use dashmap::DashMap;
use std::sync::{Arc, LazyLock};

/// グローバルなシンボルインターンテーブル（頻出シンボルを事前登録）
static SYMBOL_INTERN: LazyLock<DashMap<String, Arc<str>>> = LazyLock::new(|| {
    let map = DashMap::new();
    // 頻出シンボルを事前インターン（評価器で頻繁に使われるもの）
    for s in [
        // 基本演算子
        "print",
        "println",
        "list",
        "vector",
        "map",
        "filter",
        "reduce",
        "get",
        "assoc",
        "+",
        "-",
        "*",
        "/",
        "=",
        "<",
        ">",
        "<=",
        ">=",
        "not=",
        // 型判定
        "number?",
        "fn?",
        "string?",
        "list?",
        "vector?",
        "map?",
        "nil?",
        "empty?",
        // リスト操作
        "first",
        "rest",
        "cons",
        "concat",
        "take",
        "drop",
        "count",
        "nth",
        // 高階関数
        "apply",
        "partial",
        "comp",
        "complement",
        "juxt",
        // 制御構造
        "if",
        "do",
        "let",
        "fn",
        "def",
        "defn",
        "loop",
        "recur",
        // 並行処理
        "go",
        "chan",
        "<!",
        ">!",
        "close!",
        "thread",
    ] {
        map.insert(s.to_string(), Arc::from(s));
    }
    map
});

/// グローバルなキーワードインターンテーブル（頻出キーワードを事前登録）
static KEYWORD_INTERN: LazyLock<DashMap<String, Arc<str>>> = LazyLock::new(|| {
    let map = DashMap::new();
    // 頻出キーワードを事前インターン（Tryやレスポンスでよく使われるもの）
    for k in [
        // エラー・結果
        "ok",
        "error",
        "status",
        "message",
        // HTTP/API
        "body",
        "headers",
        "method",
        "path",
        "query",
        "params",
        "data",
        "response",
        "request",
        // データ構造の共通フィールド
        "name",
        "value",
        "id",
        "type",
        "title",
        "description",
        // 日時・状態
        "created",
        "updated",
        "timestamp",
        "date",
        "time",
        // データベース
        "table",
        "column",
        "row",
        "where",
        "limit",
        "offset",
        // ユーザー・認証
        "user",
        "email",
        "password",
        "token",
    ] {
        map.insert(k.to_string(), Arc::from(k));
    }
    map
});

/// シンボル文字列をインターンする
///
/// 既に同じ文字列がインターンテーブルにあれば、それを返す。
/// なければ新規にインターンして返す。
pub fn intern_symbol(s: &str) -> Arc<str> {
    // 高速パス: 既存エントリをチェック（String 確保なし）
    if let Some(entry) = SYMBOL_INTERN.get(s) {
        return entry.value().clone();
    }
    // スローパス: なければ insert（このときのみ String 確保）
    SYMBOL_INTERN
        .entry(s.to_string())
        .or_insert_with(|| Arc::from(s))
        .clone()
}

/// キーワード文字列をインターンする
///
/// 既に同じ文字列がインターンテーブルにあれば、それを返す。
/// なければ新規にインターンして返す。
pub fn intern_keyword(s: &str) -> Arc<str> {
    // 高速パス: 既存エントリをチェック（String 確保なし）
    if let Some(entry) = KEYWORD_INTERN.get(s) {
        return entry.value().clone();
    }
    // スローパス: なければ insert（このときのみ String 確保）
    KEYWORD_INTERN
        .entry(s.to_string())
        .or_insert_with(|| Arc::from(s))
        .clone()
}

/// インターンテーブルの統計情報を取得（デバッグ用）
#[allow(dead_code)]
pub fn intern_stats() -> (usize, usize) {
    (SYMBOL_INTERN.len(), KEYWORD_INTERN.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_symbol() {
        let s1 = intern_symbol("foo");
        let s2 = intern_symbol("foo");
        let s3 = intern_symbol("bar");

        // 同じ文字列は同じArcを返す
        assert!(Arc::ptr_eq(&s1, &s2));
        // 異なる文字列は異なるArcを返す
        assert!(!Arc::ptr_eq(&s1, &s3));
    }

    #[test]
    fn test_intern_keyword() {
        let k1 = intern_keyword("name");
        let k2 = intern_keyword("name");
        let k3 = intern_keyword("value");

        // 同じ文字列は同じArcを返す
        assert!(Arc::ptr_eq(&k1, &k2));
        // 異なる文字列は異なるArcを返す
        assert!(!Arc::ptr_eq(&k1, &k3));
    }

    #[test]
    fn test_symbol_keyword_separate() {
        let s = intern_symbol("test");
        let k = intern_keyword("test");

        // シンボルとキーワードは別のテーブルなので、異なるArc
        assert!(!Arc::ptr_eq(&s, &k));
    }
}
