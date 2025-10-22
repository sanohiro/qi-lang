//! Qiエラー処理システム
//!
//! 構造化されたエラー情報を提供し、以下をサポート：
//! - エラーコードによる分類
//! - 詳細な位置情報
//! - ヒントとサジェスト
//! - 複数の出力形式（人間向け/JSON）

use std::fmt;

/// エラーコード
///
/// Rustコンパイラ風の分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 0xxx: 変数・シンボル関連
    E0001, // 未定義の変数
    E0002, // 未定義の関数
    E0003, // 予約語の使用

    // 1xxx: 型エラー
    E0101, // 型の不一致
    E0102, // 期待しない型
    E0103, // 変換不可能な型

    // 2xxx: 引数エラー
    E0201, // 引数の数が一致しない
    E0202, // 引数が多すぎる
    E0203, // 引数が少なすぎる
    E0204, // 可変長引数のエラー

    // 3xxx: データベースエラー
    E0301, // DB接続エラー
    E0302, // クエリエラー
    E0303, // トランザクションエラー
    E0304, // パラメータエラー

    // 4xxx: HTTPエラー
    E0401, // HTTP接続エラー
    E0402, // HTTPリクエストエラー
    E0403, // HTTPレスポンスエラー
    E0404, // 無効なURL

    // 5xxx: I/Oエラー
    E0501, // ファイル読み込みエラー
    E0502, // ファイル書き込みエラー
    E0503, // ディレクトリエラー

    // 6xxx: パースエラー
    E0601, // 構文エラー
    E0602, // 字句解析エラー
    E0603, // 予期しないトークン
    E0604, // 期待されるシンボル
    E0605, // EOF（ファイル終端）エラー
    E0606, // パターンエラー

    // 9xxx: 汎用エラー
    E9999, // 分類されていないエラー
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// ソースコード上の位置情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    pub source_line: Option<String>,
}

impl SourceLocation {
    pub fn new(file: String, line: usize, column: usize) -> Self {
        Self {
            file,
            line,
            column,
            length: 1,
            source_line: None,
        }
    }

    pub fn with_length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn with_source_line(mut self, source_line: String) -> Self {
        self.source_line = Some(source_line);
        self
    }
}

/// 構造化されたエラー情報
#[derive(Debug, Clone)]
pub struct QiError {
    /// エラーコード
    code: ErrorCode,
    /// メインメッセージ（1行）
    message: String,
    /// ソースコード上の位置
    location: Option<SourceLocation>,
    /// 詳細な説明（note）
    notes: Vec<String>,
    /// 解決のヒント（help）
    help: Vec<String>,
    /// サジェスト（もしかして〜？）
    suggestions: Vec<String>,
}

impl QiError {
    /// 新しいエラーを作成
    pub fn new<S: Into<String>>(code: ErrorCode, message: S) -> Self {
        Self {
            code,
            message: message.into(),
            location: None,
            notes: Vec::new(),
            help: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// 位置情報を追加
    pub fn with_location(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// noteを追加
    pub fn with_note<S: Into<String>>(mut self, note: S) -> Self {
        self.notes.push(note.into());
        self
    }

    /// helpを追加
    pub fn with_help<S: Into<String>>(mut self, help: S) -> Self {
        self.help.push(help.into());
        self
    }

    /// suggestionを追加
    pub fn with_suggestion<S: Into<String>>(mut self, suggestion: S) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// シンプルなメッセージのみ取得（API応答用）
    pub fn message(&self) -> &str {
        &self.message
    }

    /// エラーコード + メッセージ（ログ用）
    pub fn short(&self) -> String {
        format!("error[{}]: {}", self.code, self.message)
    }

    /// 完全な詳細情報（CLI/REPL用）
    pub fn full(&self) -> String {
        let mut output = String::new();

        // 1行目：エラーコード + メッセージ
        output.push_str(&format!("error[{}]: {}\n", self.code, self.message));

        // 位置情報
        if let Some(loc) = &self.location {
            output.push_str(&format!("  --> {}:{}:{}\n", loc.file, loc.line, loc.column));

            if let Some(source) = &loc.source_line {
                output.push_str("  |\n");
                output.push_str(&format!("{:3} | {}\n", loc.line, source));

                // エラー位置の強調表示
                let spaces = " ".repeat(loc.column.saturating_sub(1));
                let carets = "^".repeat(loc.length.max(1));
                output.push_str(&format!("  | {}{}\n", spaces, carets));
            }

            output.push_str("  |\n");
        }

        // 付加情報
        for note in &self.notes {
            output.push_str(&format!("  = note: {}\n", note));
        }
        for help_text in &self.help {
            output.push_str(&format!("  = help: {}\n", help_text));
        }
        for suggestion in &self.suggestions {
            output.push_str(&format!("  = suggestion: {}\n", suggestion));
        }

        output
    }

    /// JSON形式で出力（エディタ統合用）
    #[cfg(feature = "format-json")]
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "code": self.code.to_string(),
            "message": self.message,
            "location": self.location.as_ref().map(|loc| json!({
                "file": loc.file,
                "line": loc.line,
                "column": loc.column,
                "length": loc.length,
            })),
            "notes": self.notes,
            "help": self.help,
            "suggestions": self.suggestions,
        })
    }
}

// Display実装（既存のStringエラーと互換性）
impl fmt::Display for QiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full())
    }
}

// QiError -> String（既存コードとの互換性）
impl From<QiError> for String {
    fn from(err: QiError) -> String {
        err.full()
    }
}

// String -> QiError（既存のStringエラーをラップ）
impl From<String> for QiError {
    fn from(msg: String) -> QiError {
        QiError::new(ErrorCode::E9999, msg)
    }
}

impl From<&str> for QiError {
    fn from(msg: &str) -> QiError {
        QiError::new(ErrorCode::E9999, msg.to_string())
    }
}

// std::error::Error実装
impl std::error::Error for QiError {}

// ========================================
// エラー構築ヘルパー関数
// ========================================

impl QiError {
    /// 未定義の変数エラー
    pub fn undefined_var(name: &str) -> Self {
        QiError::new(ErrorCode::E0001, format!("未定義の変数: {}", name))
            .with_help(format!("変数を定義してください: (def {} ...)", name))
    }

    /// 未定義の関数エラー
    pub fn undefined_function(name: &str) -> Self {
        QiError::new(ErrorCode::E0002, format!("未定義の関数: {}", name))
            .with_help(format!("関数を定義してください: (defn {} [...] ...)", name))
    }

    /// 型エラー
    pub fn type_error(expected: &str, actual: &str) -> Self {
        QiError::new(
            ErrorCode::E0101,
            format!("型の不一致: 期待 {}, 実際 {}", expected, actual),
        )
        .with_help(format!("{}型の値を渡してください", expected))
    }

    /// 引数の数エラー
    pub fn arg_count_mismatch(expected: usize, actual: usize) -> Self {
        QiError::new(
            ErrorCode::E0201,
            format!("引数の数が一致しません: 期待 {}, 実際 {}", expected, actual),
        )
    }

    /// 引数が多すぎる
    pub fn too_many_args(max: usize, actual: usize) -> Self {
        QiError::new(
            ErrorCode::E0202,
            format!("引数が多すぎます: 最大 {}, 実際 {}", max, actual),
        )
    }

    /// 引数が少なすぎる
    pub fn too_few_args(min: usize, actual: usize) -> Self {
        QiError::new(
            ErrorCode::E0203,
            format!("引数が少なすぎます: 最小 {}, 実際 {}", min, actual),
        )
    }

    /// データベースエラー
    pub fn database_error(message: String) -> Self {
        QiError::new(ErrorCode::E0302, format!("データベースエラー: {}", message))
    }

    /// HTTPエラー
    pub fn http_error(message: String) -> Self {
        QiError::new(ErrorCode::E0402, format!("HTTPエラー: {}", message))
    }

    /// ファイルI/Oエラー
    pub fn io_error(message: String) -> Self {
        QiError::new(ErrorCode::E0501, format!("I/Oエラー: {}", message))
    }

    /// パースエラー
    pub fn parse_error(message: String) -> Self {
        QiError::new(ErrorCode::E0601, format!("パースエラー: {}", message))
    }

    /// MsgKeyからErrorCodeを推定（parser用）
    pub fn error_code_from_parser_msg(key: &crate::i18n::MsgKey) -> ErrorCode {
        use crate::i18n::MsgKey;
        match key {
            // パースエラー: トークン関連
            MsgKey::UnexpectedToken | MsgKey::ExpectedToken => ErrorCode::E0603,
            MsgKey::UnexpectedEof => ErrorCode::E0605,
            // パースエラー: シンボル関連
            MsgKey::NeedsSymbol
            | MsgKey::VarargNeedsName
            | MsgKey::RestNeedsVar
            | MsgKey::AsNeedsVarName
            | MsgKey::AsNeedsAlias
            | MsgKey::ModuleNeedsName
            | MsgKey::ExportNeedsSymbols
            | MsgKey::UseNeedsModuleName
            | MsgKey::ExpectedSymbolInOnlyList
            | MsgKey::KeyMustBeKeyword
            | MsgKey::MacVarargNeedsSymbol => ErrorCode::E0604,
            // パースエラー: パターン関連
            MsgKey::UnexpectedPattern => ErrorCode::E0606,
            // モジュールエラー
            MsgKey::UseNeedsMode => ErrorCode::E0601,
            // デフォルト: 構文エラー
            _ => ErrorCode::E0601,
        }
    }

    /// MsgKeyからErrorCodeを推定（lexer用）
    pub fn error_code_from_lexer_msg(_key: &crate::i18n::MsgKey) -> ErrorCode {
        // レキサーエラーは全てE0602（字句解析エラー）
        ErrorCode::E0602
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_error() {
        let err = QiError::new(ErrorCode::E0001, "テストエラー");
        assert_eq!(err.message(), "テストエラー");
        assert_eq!(err.short(), "error[E0001]: テストエラー");
    }

    #[test]
    fn test_error_with_location() {
        let loc = SourceLocation::new("test.qi".to_string(), 10, 5)
            .with_length(8)
            .with_source_line("(println test-var)".to_string());

        let err = QiError::new(ErrorCode::E0001, "未定義の変数: test-var")
            .with_location(loc)
            .with_help("変数を定義してください");

        let full = err.full();
        assert!(full.contains("error[E0001]"));
        assert!(full.contains("test.qi:10:5"));
        assert!(full.contains("= help:"));
    }

    #[test]
    fn test_error_with_suggestions() {
        let err = QiError::undefined_var("test-var")
            .with_suggestion("もしかして: test-variable")
            .with_suggestion("もしかして: test-value");

        let full = err.full();
        assert!(full.contains("= suggestion: もしかして: test-variable"));
        assert!(full.contains("= suggestion: もしかして: test-value"));
    }

    #[test]
    fn test_string_conversion() {
        let err = QiError::new(ErrorCode::E0001, "テストエラー");

        // QiError -> String
        let s: String = err.clone().into();
        assert!(s.contains("error[E0001]"));

        // String -> QiError
        let err2: QiError = "シンプルエラー".into();
        assert_eq!(err2.message(), "シンプルエラー");
    }

    #[test]
    fn test_helper_functions() {
        let err1 = QiError::undefined_var("foo");
        assert_eq!(err1.message(), "未定義の変数: foo");

        let err2 = QiError::type_error("number", "string");
        assert!(err2.message().contains("number"));
        assert!(err2.message().contains("string"));

        let err3 = QiError::arg_count_mismatch(2, 1);
        assert!(err3.message().contains("2"));
        assert!(err3.message().contains("1"));
    }

    #[test]
    fn test_display_trait() {
        let err = QiError::new(ErrorCode::E0001, "テストエラー")
            .with_help("これはヒントです");

        let displayed = format!("{}", err);
        assert!(displayed.contains("error[E0001]"));
        assert!(displayed.contains("これはヒントです"));
    }
}
