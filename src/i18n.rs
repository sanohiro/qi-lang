/// 国際化メッセージ管理
///
/// 言語設定の優先順位:
/// 1. QI_LANG 環境変数（Qi専用の設定）
/// 2. LANG 環境変数（システムのロケール設定）
/// 3. デフォルト: en
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ja,
}

impl Lang {
    /// 環境変数から言語を取得
    /// 優先順位: QI_LANG > LANG > デフォルト(en)
    pub fn from_env() -> Self {
        // QI_LANGが設定されていればそれを優先
        if let Ok(lang) = std::env::var("QI_LANG") {
            return Self::parse(&lang);
        }

        // LANGから言語コードを取得（ja_JP.UTF-8 -> ja）
        if let Ok(lang) = std::env::var("LANG") {
            let lang_code = lang.split('_').next().unwrap_or("");
            return Self::parse(lang_code);
        }

        // デフォルトは英語
        Lang::En
    }

    /// 言語コードをパース
    fn parse(code: &str) -> Self {
        match code {
            "ja" | "ja_JP" => Lang::Ja,
            "en" | "en_US" | "en_GB" => Lang::En,
            _ => Lang::En, // 未対応言語は英語にフォールバック
        }
    }
}

/// エラーメッセージキー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsgKey {
    // パーサーエラー
    UnexpectedToken,
    UnexpectedEof,
    ExpectedToken,
    DefNeedsSymbol,
    LetNeedsSymbol,
    FnNeedsParam,
    VarargNeedsName,
    MapKeyNeedsKeyword,
    UnexpectedPattern,

    // レキサーエラー
    UnexpectedChar,
    UnclosedString,

    // 評価器エラー
    UndefinedVar,
    NotAFunction,
    ArgCountMismatch,
    DivisionByZero,

    // 組み込み関数エラー
    IntegerOnly,
    CollectionOnly,
    StringOrCollectionOnly,
    ListOrVectorOnly,
    KeyMustBeKeyword,

    // match関連
    NoMatchingPattern,

    // 引数エラー
    NeedAtLeastNArgs,
    NeedExactlyNArgs,
    Need2Or3Args,

    // 型エラー
    CannotQuote,
}

/// UIメッセージキー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiMsg {
    // REPL
    ReplWelcome,
    ReplPressCtrlC,
    ReplGoodbye,
    ReplLoading,
    ReplLoaded,

    // ヘルプ
    HelpTitle,
    HelpUsage,
    HelpOptions,
    HelpExamples,
    HelpEnvVars,

    // オプション説明
    OptExecute,
    OptLoad,
    OptHelp,
    OptVersion,

    // エラー
    ErrorFailedToRead,
    ErrorRequiresArg,
    ErrorRequiresFile,
    ErrorUnknownOption,
    ErrorUseHelp,
    ErrorInput,
    ErrorParse,
    ErrorLexer,
    ErrorRuntime,
}

/// メッセージマネージャー
pub struct Messages {
    lang: Lang,
    messages: HashMap<(Lang, MsgKey), &'static str>,
    ui_messages: HashMap<(Lang, UiMsg), &'static str>,
}

impl Messages {
    pub fn new(lang: Lang) -> Self {
        let mut messages = HashMap::new();

        // 英語メッセージ
        messages.insert((Lang::En, MsgKey::UnexpectedToken), "unexpected token: {0}");
        messages.insert((Lang::En, MsgKey::UnexpectedEof), "unexpected end of file");
        messages.insert((Lang::En, MsgKey::ExpectedToken), "expected {0}, got {1}");
        messages.insert((Lang::En, MsgKey::DefNeedsSymbol), "'def' requires a symbol");
        messages.insert((Lang::En, MsgKey::LetNeedsSymbol), "'let' binding requires a symbol");
        messages.insert((Lang::En, MsgKey::FnNeedsParam), "function parameter must be a symbol");
        messages.insert((Lang::En, MsgKey::VarargNeedsName), "'&' requires a variable name");
        messages.insert((Lang::En, MsgKey::MapKeyNeedsKeyword), "map pattern key must be a keyword");
        messages.insert((Lang::En, MsgKey::UnexpectedPattern), "unexpected pattern: {0}");
        messages.insert((Lang::En, MsgKey::UnexpectedChar), "unexpected character: {0}");
        messages.insert((Lang::En, MsgKey::UnclosedString), "unclosed string");
        messages.insert((Lang::En, MsgKey::UndefinedVar), "undefined variable: {0}");
        messages.insert((Lang::En, MsgKey::NotAFunction), "not a function: {0}");
        messages.insert((Lang::En, MsgKey::ArgCountMismatch), "argument count mismatch: expected {0}, got {1}");
        messages.insert((Lang::En, MsgKey::DivisionByZero), "division by zero");
        messages.insert((Lang::En, MsgKey::IntegerOnly), "{0} accepts integers only");
        messages.insert((Lang::En, MsgKey::CollectionOnly), "{0} accepts collections only");
        messages.insert((Lang::En, MsgKey::StringOrCollectionOnly), "{0} accepts strings or collections only");
        messages.insert((Lang::En, MsgKey::ListOrVectorOnly), "{0} accepts lists or vectors only");
        messages.insert((Lang::En, MsgKey::KeyMustBeKeyword), "map key must be a string or keyword");
        messages.insert((Lang::En, MsgKey::NoMatchingPattern), "no matching pattern");
        messages.insert((Lang::En, MsgKey::NeedAtLeastNArgs), "{0} requires at least {1} argument(s)");
        messages.insert((Lang::En, MsgKey::NeedExactlyNArgs), "{0} requires exactly {1} argument(s)");
        messages.insert((Lang::En, MsgKey::Need2Or3Args), "{0} requires 2 or 3 arguments");
        messages.insert((Lang::En, MsgKey::CannotQuote), "cannot quote expression: {0}");

        // 日本語メッセージ
        messages.insert((Lang::Ja, MsgKey::UnexpectedToken), "予期しないトークン: {0}");
        messages.insert((Lang::Ja, MsgKey::UnexpectedEof), "予期しないEOF");
        messages.insert((Lang::Ja, MsgKey::ExpectedToken), "期待: {0}, 実際: {1}");
        messages.insert((Lang::Ja, MsgKey::DefNeedsSymbol), "defの後にはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::LetNeedsSymbol), "letの束縛にはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::FnNeedsParam), "パラメータリストにはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::VarargNeedsName), "&の後には変数名が必要です");
        messages.insert((Lang::Ja, MsgKey::MapKeyNeedsKeyword), "マップパターンのキーはキーワードである必要があります");
        messages.insert((Lang::Ja, MsgKey::UnexpectedPattern), "予期しないパターン: {0}");
        messages.insert((Lang::Ja, MsgKey::UnexpectedChar), "予期しない文字: {0}");
        messages.insert((Lang::Ja, MsgKey::UnclosedString), "文字列が閉じられていません");
        messages.insert((Lang::Ja, MsgKey::UndefinedVar), "未定義の変数: {0}");
        messages.insert((Lang::Ja, MsgKey::NotAFunction), "関数ではありません: {0}");
        messages.insert((Lang::Ja, MsgKey::ArgCountMismatch), "引数の数が一致しません: 期待 {0}, 実際 {1}");
        messages.insert((Lang::Ja, MsgKey::DivisionByZero), "ゼロ除算エラー");
        messages.insert((Lang::Ja, MsgKey::IntegerOnly), "{0}は整数のみ受け付けます");
        messages.insert((Lang::Ja, MsgKey::CollectionOnly), "{0}はコレクションのみ受け付けます");
        messages.insert((Lang::Ja, MsgKey::StringOrCollectionOnly), "{0}は文字列またはコレクションのみ受け付けます");
        messages.insert((Lang::Ja, MsgKey::ListOrVectorOnly), "{0}はリストまたはベクタのみ受け付けます");
        messages.insert((Lang::Ja, MsgKey::KeyMustBeKeyword), "マップのキーは文字列またはキーワードが必要です");
        messages.insert((Lang::Ja, MsgKey::NoMatchingPattern), "どのパターンにもマッチしませんでした");
        messages.insert((Lang::Ja, MsgKey::NeedAtLeastNArgs), "{0}には少なくとも{1}個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::NeedExactlyNArgs), "{0}には{1}個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need2Or3Args), "{0}には2または3個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::CannotQuote), "quoteできない式: {0}");

        // UIメッセージ
        let mut ui_messages = HashMap::new();

        // 英語UI
        ui_messages.insert((Lang::En, UiMsg::ReplWelcome), "Qi REPL v{0}");
        ui_messages.insert((Lang::En, UiMsg::ReplPressCtrlC), "Press Ctrl+C to exit");
        ui_messages.insert((Lang::En, UiMsg::ReplGoodbye), "Goodbye!");
        ui_messages.insert((Lang::En, UiMsg::ReplLoading), "Loading {0}...");
        ui_messages.insert((Lang::En, UiMsg::ReplLoaded), "Loaded.");
        ui_messages.insert((Lang::En, UiMsg::HelpTitle), "Qi - A Lisp that flows");
        ui_messages.insert((Lang::En, UiMsg::HelpUsage), "USAGE:");
        ui_messages.insert((Lang::En, UiMsg::HelpOptions), "OPTIONS:");
        ui_messages.insert((Lang::En, UiMsg::HelpExamples), "EXAMPLES:");
        ui_messages.insert((Lang::En, UiMsg::HelpEnvVars), "ENVIRONMENT VARIABLES:");
        ui_messages.insert((Lang::En, UiMsg::OptExecute), "Execute code string and exit");
        ui_messages.insert((Lang::En, UiMsg::OptLoad), "Load file and start REPL");
        ui_messages.insert((Lang::En, UiMsg::OptHelp), "Print help information");
        ui_messages.insert((Lang::En, UiMsg::OptVersion), "Print version information");
        ui_messages.insert((Lang::En, UiMsg::ErrorFailedToRead), "Failed to read file");
        ui_messages.insert((Lang::En, UiMsg::ErrorRequiresArg), "{0} requires an argument");
        ui_messages.insert((Lang::En, UiMsg::ErrorRequiresFile), "{0} requires a file path");
        ui_messages.insert((Lang::En, UiMsg::ErrorUnknownOption), "Unknown option: {0}");
        ui_messages.insert((Lang::En, UiMsg::ErrorUseHelp), "Use --help for usage information");
        ui_messages.insert((Lang::En, UiMsg::ErrorInput), "Input error");
        ui_messages.insert((Lang::En, UiMsg::ErrorParse), "Parse error");
        ui_messages.insert((Lang::En, UiMsg::ErrorLexer), "Lexer error");
        ui_messages.insert((Lang::En, UiMsg::ErrorRuntime), "Error");

        // 日本語UI
        ui_messages.insert((Lang::Ja, UiMsg::ReplWelcome), "Qi REPL v{0}");
        ui_messages.insert((Lang::Ja, UiMsg::ReplPressCtrlC), "終了するには Ctrl+C を押してください");
        ui_messages.insert((Lang::Ja, UiMsg::ReplGoodbye), "さようなら！");
        ui_messages.insert((Lang::Ja, UiMsg::ReplLoading), "{0} を読み込んでいます...");
        ui_messages.insert((Lang::Ja, UiMsg::ReplLoaded), "読み込み完了");
        ui_messages.insert((Lang::Ja, UiMsg::HelpTitle), "Qi - 流れるLisp");
        ui_messages.insert((Lang::Ja, UiMsg::HelpUsage), "使い方:");
        ui_messages.insert((Lang::Ja, UiMsg::HelpOptions), "オプション:");
        ui_messages.insert((Lang::Ja, UiMsg::HelpExamples), "例:");
        ui_messages.insert((Lang::Ja, UiMsg::HelpEnvVars), "環境変数:");
        ui_messages.insert((Lang::Ja, UiMsg::OptExecute), "コード文字列を実行して終了");
        ui_messages.insert((Lang::Ja, UiMsg::OptLoad), "ファイルを読み込んでREPLを起動");
        ui_messages.insert((Lang::Ja, UiMsg::OptHelp), "ヘルプ情報を表示");
        ui_messages.insert((Lang::Ja, UiMsg::OptVersion), "バージョン情報を表示");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorFailedToRead), "ファイルの読み込みに失敗しました");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorRequiresArg), "{0} には引数が必要です");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorRequiresFile), "{0} にはファイルパスが必要です");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorUnknownOption), "不明なオプション: {0}");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorUseHelp), "使い方は --help で確認してください");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorInput), "入力エラー");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorParse), "パースエラー");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorLexer), "レキサーエラー");
        ui_messages.insert((Lang::Ja, UiMsg::ErrorRuntime), "エラー");

        Messages { lang, messages, ui_messages }
    }

    /// メッセージを取得
    pub fn get(&self, key: MsgKey) -> &'static str {
        self.messages
            .get(&(self.lang, key))
            .copied()
            .unwrap_or("unknown error")
    }

    /// フォーマット付きメッセージを取得
    pub fn fmt(&self, key: MsgKey, args: &[&str]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }

    /// UIメッセージを取得
    pub fn get_ui(&self, key: UiMsg) -> &'static str {
        self.ui_messages
            .get(&(self.lang, key))
            .copied()
            .unwrap_or("unknown ui message")
    }

    /// フォーマット付きUIメッセージを取得
    pub fn fmt_ui(&self, key: UiMsg, args: &[&str]) -> String {
        let template = self.get_ui(key);
        let mut result = template.to_string();
        for (i, arg) in args.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), arg);
        }
        result
    }
}

/// グローバルなメッセージマネージャー
static MESSAGES: OnceLock<Messages> = OnceLock::new();

/// メッセージマネージャーを初期化
pub fn init() {
    MESSAGES.get_or_init(|| Messages::new(Lang::from_env()));
}

/// メッセージを取得
pub fn msg(key: MsgKey) -> &'static str {
    MESSAGES
        .get()
        .map(|m| m.get(key))
        .unwrap_or("message system not initialized")
}

/// フォーマット付きメッセージを取得
pub fn fmt_msg(key: MsgKey, args: &[&str]) -> String {
    MESSAGES
        .get()
        .map(|m| m.fmt(key, args))
        .unwrap_or_else(|| "message system not initialized".to_string())
}

/// UIメッセージを取得
pub fn ui_msg(key: UiMsg) -> &'static str {
    MESSAGES
        .get()
        .map(|m| m.get_ui(key))
        .unwrap_or("message system not initialized")
}

/// フォーマット付きUIメッセージを取得
pub fn fmt_ui_msg(key: UiMsg, args: &[&str]) -> String {
    MESSAGES
        .get()
        .map(|m| m.fmt_ui(key, args))
        .unwrap_or_else(|| "message system not initialized".to_string())
}
