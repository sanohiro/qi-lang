//! 文字列インターン
//!
//! シンボルとキーワードの重複文字列を削減するため、
//! グローバルなインターンテーブルで文字列を管理する。
//!
//! インターンされた文字列は Arc<str> で管理され、
//! clone() 時のコピーコストを削減する。

use dashmap::DashMap;
use std::sync::{Arc, LazyLock};

/// グローバルなシンボルインターンテーブル
static SYMBOL_INTERN: LazyLock<DashMap<String, Arc<str>>> = LazyLock::new(DashMap::new);

/// グローバルなキーワードインターンテーブル
static KEYWORD_INTERN: LazyLock<DashMap<String, Arc<str>>> = LazyLock::new(DashMap::new);

/// シンボル文字列をインターンする
///
/// 既に同じ文字列がインターンテーブルにあれば、それを返す。
/// なければ新規にインターンして返す。
pub fn intern_symbol(s: &str) -> Arc<str> {
    if let Some(interned) = SYMBOL_INTERN.get(s) {
        Arc::clone(interned.value())
    } else {
        let arc_str: Arc<str> = Arc::from(s);
        SYMBOL_INTERN.insert(s.to_string(), Arc::clone(&arc_str));
        arc_str
    }
}

/// キーワード文字列をインターンする
///
/// 既に同じ文字列がインターンテーブルにあれば、それを返す。
/// なければ新規にインターンして返す。
pub fn intern_keyword(s: &str) -> Arc<str> {
    if let Some(interned) = KEYWORD_INTERN.get(s) {
        Arc::clone(interned.value())
    } else {
        let arc_str: Arc<str> = Arc::from(s);
        KEYWORD_INTERN.insert(s.to_string(), Arc::clone(&arc_str));
        arc_str
    }
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
