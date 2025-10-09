/// 国際化メッセージ管理
///
/// 言語設定の優先順位:
/// 1. QI_LANG 環境変数（Qi専用の設定）
/// 2. LANG 環境変数（システムのロケール設定）
/// 3. デフォルト: en
use std::collections::HashMap;

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

/// メッセージキー
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

/// メッセージマネージャー
pub struct Messages {
    lang: Lang,
    messages: HashMap<(Lang, MsgKey), &'static str>,
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

        Messages { lang, messages }
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
}

/// グローバルなメッセージマネージャー
static mut MESSAGES: Option<Messages> = None;

/// メッセージマネージャーを初期化
pub fn init() {
    unsafe {
        MESSAGES = Some(Messages::new(Lang::from_env()));
    }
}

/// メッセージを取得
pub fn msg(key: MsgKey) -> &'static str {
    unsafe {
        MESSAGES
            .as_ref()
            .map(|m| m.get(key))
            .unwrap_or("message system not initialized")
    }
}

/// フォーマット付きメッセージを取得
pub fn fmt_msg(key: MsgKey, args: &[&str]) -> String {
    unsafe {
        MESSAGES
            .as_ref()
            .map(|m| m.fmt(key, args))
            .unwrap_or_else(|| "message system not initialized".to_string())
    }
}
