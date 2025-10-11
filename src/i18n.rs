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
    NeedsSymbol,        // 共通化: Def/Let/Fn等で使用
    VarargNeedsName,
    UnexpectedPattern,

    // レキサーエラー
    UnexpectedChar,
    UnclosedString,

    // 評価器エラー
    UndefinedVar,
    NotAFunction,
    ArgCountMismatch,
    DivisionByZero,
    ExportOnlyInModule,
    CannotQuote,        // 統合: CannotQuoteとCannotQuoteSpecialForm
    NoMatchingPattern,

    // モジュールエラー
    ModuleNeedsName,
    ExportNeedsSymbols,
    UseNeedsModuleName,
    ExpectedSymbolInOnlyList,
    AsNeedsAlias,
    UseNeedsMode,
    SymbolNotFound,
    ModuleNotFound,
    SymbolNotExported,
    UseAsNotImplemented,

    // 引数エラー（汎用）
    NeedAtLeastNArgs,   // {0}には少なくとも{1}個の引数が必要
    NeedExactlyNArgs,   // {0}には{1}個の引数が必要
    Need2Or3Args,       // {0}には2または3個の引数が必要
    Need1Or2Args,       // {0}には1または2個の引数が必要
    Need2Args,          // {0}には2つの引数が必要
    Need1Arg,           // {0}には1つの引数が必要
    Need0Args,          // {0}には引数は不要

    // 型エラー（汎用）
    TypeOnly,           // {0}は{1}のみ受け付けます
    TypeOnlyWithDebug,  // {0}は{1}のみ受け付けます: {2:?}
    ArgMustBeType,      // {0}: 引数は{1}である必要があります
    FirstArgMustBe,     // {0}の第1引数は{1}が必要です
    SecondArgMustBe,    // {0}の第2引数は{1}が必要です
    ThirdArgMustBe,     // {0}の第3引数は{1}が必要です
    KeyMustBeKeyword,   // キーは文字列またはキーワードが必要
    KeyNotFound,        // キーが見つかりません: {0}
    MustBePositive,     // {0}: {1}は正の数である必要があります
    MustBeNonNegative,  // {0}: {1}は非負の数である必要があります
    MustBeInteger,      // {0}: {1}は整数である必要があります
    MustBeString,       // {0}: {1}は文字列である必要があります
    MinMustBeLessThanMax, // {0}: min must be less than max
    MustBeListOrVector, // {0}: {1}はリストまたはベクタである必要があります
    MustBePromise,      // {0}: {1}はプロミス（チャネル）である必要があります
    MustBeScope,        // {0}: {1}はスコープである必要があります
    MustNotBeEmpty,     // {0}: {1}は空であってはいけません
    FuncMustReturnType, // {0}: 関数は{1}を返す必要があります
    MustBeMap,          // {0}: {1}はマップである必要があります

    // 特殊な引数エラー
    SplitTwoStrings,
    JoinStringAndList,
    AssocMapAndKeyValues,
    DissocMapAndKeys,
    VariadicFnNeedsOneParam,

    // f-string エラー
    FStringUnclosedBrace,
    FStringUnclosed,
    FStringCannotBeQuoted,

    // マクロエラー
    MacVarargNeedsSymbol,
    VariadicMacroNeedsParams,

    // quasiquote エラー
    UnquoteOutsideQuasiquote,
    UnquoteSpliceOutsideQuasiquote,
    UnquoteSpliceNeedsListOrVector,

    // loop/recur エラー
    RecurNotFound,

    // 内部変換エラー
    ValueCannotBeConverted,

    // モジュールロード詳細エラー
    CircularDependency,
    ModuleParserInitError,
    ModuleParseError,
    ModuleMustExport,

    // その他の特殊エラー
    AsNeedsVarName,     // :asには変数名が必要です
    NeedNArgsDesc,      // {0}には{1}個の引数が必要です: {2}
    SelectNeedsList,    // {0}にはリストが必要です
    SelectNeedsAtLeastOne, // {0}には少なくとも1つのケースが必要です
    SelectTimeoutCase,  // {0}: :timeoutケースは3要素が必要です: [:timeout ms handler]
    SelectOnlyOneTimeout, // {0}には:timeoutケースは1つだけです
    SelectChannelCase,  // {0}: チャネルケースは2要素が必要です: [channel handler]
    SelectCaseMustStart, // {0}: ケースはチャネルまたは:timeoutで始まる必要があります
    SelectCaseMustBe,   // {0}: ケースはリストである必要があります [channel handler] or [:timeout ms handler]
    AllElementsMustBe,  // {0}: 全ての要素は{1}である必要があります

    // 並行処理エラー
    ChannelClosed,      // {0}: channel is closed
    ExpectedKeyword,    // {0}: expected {1} keyword
    PromiseFailed,      // promise failed
    NotAPromise,        // not a promise
    UnexpectedError,    // {0}: unexpected error
    RecvArgs,           // {0}: requires 1 or 3 arguments: ({0} ch) or ({0} ch :timeout ms)
    TimeoutMustBeMs,    // {0}: timeout must be an integer (milliseconds)

    // その他のエラー
    UnsupportedNumberType, // unsupported number type
    RailwayRequiresOkError, // |>? requires {:ok/:error} map
    InvalidTimestamp,    // invalid timestamp
    InvalidDateFormat,   // invalid date format
    InvalidPercentile,   // invalid percentile (must be 0-100)

    // 警告
    RedefineBuiltin,     // warning: redefining builtin function: {0} ({1})
    RedefineFunction,    // warning: redefining function: {0}
    RedefineVariable,    // warning: redefining variable: {0}
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

    // ヘルプ例
    ExampleStartRepl,
    ExampleRunScript,
    ExampleExecuteCode,
    ExampleLoadFile,

    // 環境変数説明
    EnvLangQi,
    EnvLangSystem,

    // バージョン
    VersionString,

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

        // 英語メッセージ - パーサー/レキサー
        messages.insert((Lang::En, MsgKey::UnexpectedToken), "unexpected token: {0}");
        messages.insert((Lang::En, MsgKey::UnexpectedEof), "unexpected end of file");
        messages.insert((Lang::En, MsgKey::ExpectedToken), "expected {0}, got {1}");
        messages.insert((Lang::En, MsgKey::NeedsSymbol), "{0} requires a symbol");
        messages.insert((Lang::En, MsgKey::VarargNeedsName), "'&' requires a variable name");
        messages.insert((Lang::En, MsgKey::UnexpectedPattern), "unexpected pattern: {0}");
        messages.insert((Lang::En, MsgKey::UnexpectedChar), "unexpected character: {0}");
        messages.insert((Lang::En, MsgKey::UnclosedString), "unclosed string");

        // 英語メッセージ - 評価器
        messages.insert((Lang::En, MsgKey::UndefinedVar), "undefined variable: {0}");
        messages.insert((Lang::En, MsgKey::NotAFunction), "not a function: {0}");
        messages.insert((Lang::En, MsgKey::ArgCountMismatch), "argument count mismatch: expected {0}, got {1}");
        messages.insert((Lang::En, MsgKey::DivisionByZero), "division by zero");
        messages.insert((Lang::En, MsgKey::ExportOnlyInModule), "export can only be used inside a module definition");
        messages.insert((Lang::En, MsgKey::CannotQuote), "cannot quote: {0}");
        messages.insert((Lang::En, MsgKey::NoMatchingPattern), "no matching pattern");

        // 英語メッセージ - モジュール
        messages.insert((Lang::En, MsgKey::ModuleNeedsName), "module requires a module name");
        messages.insert((Lang::En, MsgKey::ExportNeedsSymbols), "export requires symbols");
        messages.insert((Lang::En, MsgKey::UseNeedsModuleName), "use requires a module name");
        messages.insert((Lang::En, MsgKey::ExpectedSymbolInOnlyList), "expected symbol in :only list");
        messages.insert((Lang::En, MsgKey::AsNeedsAlias), ":as requires an alias name");
        messages.insert((Lang::En, MsgKey::UseNeedsMode), "use requires :only, :as, or :all");
        messages.insert((Lang::En, MsgKey::SymbolNotFound), "symbol {0} not found (module: {1})");
        messages.insert((Lang::En, MsgKey::ModuleNotFound), "module {0} not found ({0}.qi)");
        messages.insert((Lang::En, MsgKey::SymbolNotExported), "symbol {0} is not exported from module {1}");
        messages.insert((Lang::En, MsgKey::UseAsNotImplemented), ":as mode is not implemented yet");

        // 英語メッセージ - 引数エラー
        messages.insert((Lang::En, MsgKey::NeedAtLeastNArgs), "{0} requires at least {1} argument(s)");
        messages.insert((Lang::En, MsgKey::NeedExactlyNArgs), "{0} requires exactly {1} argument(s)");
        messages.insert((Lang::En, MsgKey::Need2Or3Args), "{0} requires 2 or 3 arguments");
        messages.insert((Lang::En, MsgKey::Need1Or2Args), "{0} requires 1 or 2 arguments");
        messages.insert((Lang::En, MsgKey::Need2Args), "{0} requires 2 arguments");
        messages.insert((Lang::En, MsgKey::Need1Arg), "{0} requires 1 argument");
        messages.insert((Lang::En, MsgKey::Need0Args), "{0} requires no arguments");

        // 英語メッセージ - 型エラー
        messages.insert((Lang::En, MsgKey::TypeOnly), "{0} accepts {1} only");
        messages.insert((Lang::En, MsgKey::TypeOnlyWithDebug), "{0} accepts {1} only: {2}");
        messages.insert((Lang::En, MsgKey::ArgMustBeType), "{0}: argument must be {1}");
        messages.insert((Lang::En, MsgKey::FirstArgMustBe), "{0}'s first argument must be {1}");
        messages.insert((Lang::En, MsgKey::SecondArgMustBe), "{0}'s second argument must be {1}");
        messages.insert((Lang::En, MsgKey::ThirdArgMustBe), "{0}'s third argument must be {1}");
        messages.insert((Lang::En, MsgKey::KeyMustBeKeyword), "key must be a string or keyword");
        messages.insert((Lang::En, MsgKey::KeyNotFound), "key not found: {0}");
        messages.insert((Lang::En, MsgKey::MustBePositive), "{0}: {1} must be positive");
        messages.insert((Lang::En, MsgKey::MustBeNonNegative), "{0}: {1} must be non-negative");
        messages.insert((Lang::En, MsgKey::MustBeInteger), "{0}: {1} must be an integer");
        messages.insert((Lang::En, MsgKey::MustBeString), "{0}: {1} must be a string");
        messages.insert((Lang::En, MsgKey::MinMustBeLessThanMax), "{0}: min must be less than max");
        messages.insert((Lang::En, MsgKey::MustBeListOrVector), "{0}: {1} must be a list or vector");
        messages.insert((Lang::En, MsgKey::MustBePromise), "{0}: {1} must be a promise (channel)");
        messages.insert((Lang::En, MsgKey::MustBeScope), "{0}: {1} must be a scope");
        messages.insert((Lang::En, MsgKey::MustNotBeEmpty), "{0}: {1} must not be empty");
        messages.insert((Lang::En, MsgKey::FuncMustReturnType), "{0}: function must return {1}");
        messages.insert((Lang::En, MsgKey::MustBeMap), "{0}: {1} must be a map");

        // 英語メッセージ - 特殊な引数エラー
        messages.insert((Lang::En, MsgKey::SplitTwoStrings), "split requires two strings");
        messages.insert((Lang::En, MsgKey::JoinStringAndList), "join requires a string and a list");
        messages.insert((Lang::En, MsgKey::AssocMapAndKeyValues), "assoc requires a map and one or more key-value pairs");
        messages.insert((Lang::En, MsgKey::DissocMapAndKeys), "dissoc requires a map and one or more keys");
        messages.insert((Lang::En, MsgKey::VariadicFnNeedsOneParam), "variadic function requires exactly one parameter");

        // 英語メッセージ - f-string
        messages.insert((Lang::En, MsgKey::FStringUnclosedBrace), "f-string: unclosed {");
        messages.insert((Lang::En, MsgKey::FStringUnclosed), "f-string: unclosed string");
        messages.insert((Lang::En, MsgKey::FStringCannotBeQuoted), "f-string cannot be quoted");

        // 英語メッセージ - マクロ
        messages.insert((Lang::En, MsgKey::MacVarargNeedsSymbol), "mac: '&' requires a symbol");
        messages.insert((Lang::En, MsgKey::VariadicMacroNeedsParams), "variadic macro requires parameters");

        // 英語メッセージ - quasiquote
        messages.insert((Lang::En, MsgKey::UnquoteOutsideQuasiquote), "unquote: can only be used inside quasiquote");
        messages.insert((Lang::En, MsgKey::UnquoteSpliceOutsideQuasiquote), "unquote-splice: can only be used inside quasiquote");
        messages.insert((Lang::En, MsgKey::UnquoteSpliceNeedsListOrVector), "unquote-splice: requires a list or vector");

        // 英語メッセージ - loop/recur
        messages.insert((Lang::En, MsgKey::RecurNotFound), "recur not found");

        // 英語メッセージ - 内部変換
        messages.insert((Lang::En, MsgKey::ValueCannotBeConverted), "value cannot be converted");

        // 英語メッセージ - モジュールロード詳細
        messages.insert((Lang::En, MsgKey::CircularDependency), "circular dependency detected: {0}");
        messages.insert((Lang::En, MsgKey::ModuleParserInitError), "module {0} parser initialization error: {1}");
        messages.insert((Lang::En, MsgKey::ModuleParseError), "module {0} parse error: {1}");
        messages.insert((Lang::En, MsgKey::ModuleMustExport), "module {0} must contain export");

        // 英語メッセージ - その他の特殊エラー
        messages.insert((Lang::En, MsgKey::AsNeedsVarName), ":as requires a variable name");
        messages.insert((Lang::En, MsgKey::NeedNArgsDesc), "{0} requires {1} argument(s): {2}");
        messages.insert((Lang::En, MsgKey::SelectNeedsList), "{0} requires a list");
        messages.insert((Lang::En, MsgKey::SelectNeedsAtLeastOne), "{0} requires at least one case");
        messages.insert((Lang::En, MsgKey::SelectTimeoutCase), "{0}: :timeout case must have 3 elements: [:timeout ms handler]");
        messages.insert((Lang::En, MsgKey::SelectOnlyOneTimeout), "{0} can only have one :timeout case");
        messages.insert((Lang::En, MsgKey::SelectChannelCase), "{0}: channel case must have 2 elements: [channel handler]");
        messages.insert((Lang::En, MsgKey::SelectCaseMustStart), "{0}: case must start with a channel or :timeout");
        messages.insert((Lang::En, MsgKey::SelectCaseMustBe), "{0}: case must be a list [channel handler] or [:timeout ms handler]");
        messages.insert((Lang::En, MsgKey::AllElementsMustBe), "{0}: all elements must be {1}");

        // 英語メッセージ - 並行処理
        messages.insert((Lang::En, MsgKey::ChannelClosed), "{0}: channel is closed");
        messages.insert((Lang::En, MsgKey::ExpectedKeyword), "{0}: expected {1} keyword");
        messages.insert((Lang::En, MsgKey::PromiseFailed), "promise failed");
        messages.insert((Lang::En, MsgKey::NotAPromise), "not a promise");
        messages.insert((Lang::En, MsgKey::UnexpectedError), "{0}: unexpected error");
        messages.insert((Lang::En, MsgKey::RecvArgs), "{0}: requires 1 or 3 arguments: ({0} ch) or ({0} ch :timeout ms)");
        messages.insert((Lang::En, MsgKey::TimeoutMustBeMs), "{0}: timeout must be an integer (milliseconds)");

        // 英語メッセージ - その他
        messages.insert((Lang::En, MsgKey::UnsupportedNumberType), "unsupported number type");
        messages.insert((Lang::En, MsgKey::RailwayRequiresOkError), "|>? requires {:ok/:error} map");
        messages.insert((Lang::En, MsgKey::InvalidTimestamp), "{0}: invalid timestamp");
        messages.insert((Lang::En, MsgKey::InvalidDateFormat), "{0}: invalid date format: {1}");
        messages.insert((Lang::En, MsgKey::InvalidPercentile), "{0}: percentile must be between 0 and 100");

        // 英語メッセージ - 警告
        messages.insert((Lang::En, MsgKey::RedefineBuiltin), "warning: redefining builtin function '{0}' ({1})");
        messages.insert((Lang::En, MsgKey::RedefineFunction), "warning: redefining function '{0}'");
        messages.insert((Lang::En, MsgKey::RedefineVariable), "warning: redefining variable '{0}'");

        // 日本語メッセージ - パーサー/レキサー
        messages.insert((Lang::Ja, MsgKey::UnexpectedToken), "予期しないトークン: {0}");
        messages.insert((Lang::Ja, MsgKey::UnexpectedEof), "予期しないEOF");
        messages.insert((Lang::Ja, MsgKey::ExpectedToken), "期待: {0}, 実際: {1}");
        messages.insert((Lang::Ja, MsgKey::NeedsSymbol), "{0}にはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::VarargNeedsName), "&の後には変数名が必要です");
        messages.insert((Lang::Ja, MsgKey::UnexpectedPattern), "予期しないパターン: {0}");
        messages.insert((Lang::Ja, MsgKey::UnexpectedChar), "予期しない文字: {0}");
        messages.insert((Lang::Ja, MsgKey::UnclosedString), "文字列が閉じられていません");

        // 日本語メッセージ - 評価器
        messages.insert((Lang::Ja, MsgKey::UndefinedVar), "未定義の変数: {0}");
        messages.insert((Lang::Ja, MsgKey::NotAFunction), "関数ではありません: {0}");
        messages.insert((Lang::Ja, MsgKey::ArgCountMismatch), "引数の数が一致しません: 期待 {0}, 実際 {1}");
        messages.insert((Lang::Ja, MsgKey::DivisionByZero), "ゼロ除算エラー");
        messages.insert((Lang::Ja, MsgKey::ExportOnlyInModule), "exportはmodule定義の中でのみ使用できます");
        messages.insert((Lang::Ja, MsgKey::CannotQuote), "quoteできません: {0}");
        messages.insert((Lang::Ja, MsgKey::NoMatchingPattern), "どのパターンにもマッチしませんでした");

        // 日本語メッセージ - モジュール
        messages.insert((Lang::Ja, MsgKey::ModuleNeedsName), "moduleにはモジュール名が必要です");
        messages.insert((Lang::Ja, MsgKey::ExportNeedsSymbols), "exportにはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::UseNeedsModuleName), "useにはモジュール名が必要です");
        messages.insert((Lang::Ja, MsgKey::ExpectedSymbolInOnlyList), ":onlyリストにはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::AsNeedsAlias), ":asにはエイリアス名が必要です");
        messages.insert((Lang::Ja, MsgKey::UseNeedsMode), "useには:onlyまたは:asが必要です");
        messages.insert((Lang::Ja, MsgKey::SymbolNotFound), "シンボル{0}が見つかりません（モジュール: {1}）");
        messages.insert((Lang::Ja, MsgKey::ModuleNotFound), "モジュール{0}が見つかりません（{0}.qi）");
        messages.insert((Lang::Ja, MsgKey::SymbolNotExported), "シンボル{0}はモジュール{1}からエクスポートされていません");
        messages.insert((Lang::Ja, MsgKey::UseAsNotImplemented), ":asモードはまだ実装されていません");

        // 日本語メッセージ - 引数エラー
        messages.insert((Lang::Ja, MsgKey::NeedAtLeastNArgs), "{0}には少なくとも{1}個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::NeedExactlyNArgs), "{0}には{1}個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need2Or3Args), "{0}には2または3個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need1Or2Args), "{0}には1または2個の引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need2Args), "{0}には2つの引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need1Arg), "{0}には1つの引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need0Args), "{0}には引数は不要です");

        // 日本語メッセージ - 型エラー
        messages.insert((Lang::Ja, MsgKey::TypeOnly), "{0}は{1}のみ受け付けます");
        messages.insert((Lang::Ja, MsgKey::TypeOnlyWithDebug), "{0}は{1}のみ受け付けます: {2}");
        messages.insert((Lang::Ja, MsgKey::ArgMustBeType), "{0}: 引数は{1}である必要があります");
        messages.insert((Lang::Ja, MsgKey::FirstArgMustBe), "{0}の第1引数は{1}が必要です");
        messages.insert((Lang::Ja, MsgKey::SecondArgMustBe), "{0}の第2引数は{1}が必要です");
        messages.insert((Lang::Ja, MsgKey::ThirdArgMustBe), "{0}の第3引数は{1}が必要です");
        messages.insert((Lang::Ja, MsgKey::KeyMustBeKeyword), "キーは文字列またはキーワードが必要です");
        messages.insert((Lang::Ja, MsgKey::KeyNotFound), "キーが見つかりません: {0}");
        messages.insert((Lang::Ja, MsgKey::MustBePositive), "{0}: {1}は正の数である必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeNonNegative), "{0}: {1}は非負の数である必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeInteger), "{0}: {1}は整数である必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeString), "{0}: {1}は文字列である必要があります");
        messages.insert((Lang::Ja, MsgKey::MinMustBeLessThanMax), "{0}: minはmaxより小さい必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeListOrVector), "{0}: {1}はリストまたはベクタである必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBePromise), "{0}: {1}はプロミス（チャネル）である必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeScope), "{0}: {1}はスコープである必要があります");
        messages.insert((Lang::Ja, MsgKey::MustNotBeEmpty), "{0}: {1}は空であってはいけません");
        messages.insert((Lang::Ja, MsgKey::FuncMustReturnType), "{0}: 関数は{1}を返す必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeMap), "{0}: {1}はマップである必要があります");

        // 日本語メッセージ - 特殊な引数エラー
        messages.insert((Lang::Ja, MsgKey::SplitTwoStrings), "splitは2つの文字列が必要です");
        messages.insert((Lang::Ja, MsgKey::JoinStringAndList), "joinは文字列とリストが必要です");
        messages.insert((Lang::Ja, MsgKey::AssocMapAndKeyValues), "assocはマップと1つ以上のキー・値のペアが必要です");
        messages.insert((Lang::Ja, MsgKey::DissocMapAndKeys), "dissocはマップと1つ以上のキーが必要です");
        messages.insert((Lang::Ja, MsgKey::VariadicFnNeedsOneParam), "可変長引数関数にはパラメータが1つだけ必要です");

        // 日本語メッセージ - f-string
        messages.insert((Lang::Ja, MsgKey::FStringUnclosedBrace), "f-string: 閉じられていない { があります");
        messages.insert((Lang::Ja, MsgKey::FStringUnclosed), "f-string: 閉じられていない文字列です");
        messages.insert((Lang::Ja, MsgKey::FStringCannotBeQuoted), "f-string はquoteできません");

        // 日本語メッセージ - マクロ
        messages.insert((Lang::Ja, MsgKey::MacVarargNeedsSymbol), "mac: &の後にシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::VariadicMacroNeedsParams), "可変長マクロはパラメータが必要です");

        // 日本語メッセージ - quasiquote
        messages.insert((Lang::Ja, MsgKey::UnquoteOutsideQuasiquote), "unquote: quasiquote外では使用できません");
        messages.insert((Lang::Ja, MsgKey::UnquoteSpliceOutsideQuasiquote), "unquote-splice: quasiquote外では使用できません");
        messages.insert((Lang::Ja, MsgKey::UnquoteSpliceNeedsListOrVector), "unquote-splice: リストまたはベクタが必要です");

        // 日本語メッセージ - loop/recur
        messages.insert((Lang::Ja, MsgKey::RecurNotFound), "recurが見つかりません");

        // 日本語メッセージ - 内部変換
        messages.insert((Lang::Ja, MsgKey::ValueCannotBeConverted), "この値は変換できません");

        // 日本語メッセージ - モジュールロード詳細
        messages.insert((Lang::Ja, MsgKey::CircularDependency), "循環参照を検出しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ModuleParserInitError), "モジュール{0}のパーサー初期化エラー: {1}");
        messages.insert((Lang::Ja, MsgKey::ModuleParseError), "モジュール{0}のパースエラー: {1}");
        messages.insert((Lang::Ja, MsgKey::ModuleMustExport), "モジュール{0}はexportを含む必要があります");

        // 日本語メッセージ - その他の特殊エラー
        messages.insert((Lang::Ja, MsgKey::AsNeedsVarName), ":asには変数名が必要です");
        messages.insert((Lang::Ja, MsgKey::NeedNArgsDesc), "{0}には{1}個の引数が必要です: {2}");
        messages.insert((Lang::Ja, MsgKey::SelectNeedsList), "{0}にはリストが必要です");
        messages.insert((Lang::Ja, MsgKey::SelectNeedsAtLeastOne), "{0}には少なくとも1つのケースが必要です");
        messages.insert((Lang::Ja, MsgKey::SelectTimeoutCase), "{0}: :timeoutケースは3要素が必要です: [:timeout ms handler]");
        messages.insert((Lang::Ja, MsgKey::SelectOnlyOneTimeout), "{0}には:timeoutケースは1つだけです");
        messages.insert((Lang::Ja, MsgKey::SelectChannelCase), "{0}: チャネルケースは2要素が必要です: [channel handler]");
        messages.insert((Lang::Ja, MsgKey::SelectCaseMustStart), "{0}: ケースはチャネルまたは:timeoutで始まる必要があります");
        messages.insert((Lang::Ja, MsgKey::SelectCaseMustBe), "{0}: ケースはリストである必要があります [channel handler] or [:timeout ms handler]");
        messages.insert((Lang::Ja, MsgKey::AllElementsMustBe), "{0}: 全ての要素は{1}である必要があります");

        // 日本語メッセージ - 並行処理
        messages.insert((Lang::Ja, MsgKey::ChannelClosed), "{0}: チャネルがクローズされています");
        messages.insert((Lang::Ja, MsgKey::ExpectedKeyword), "{0}: {1}キーワードが必要です");
        messages.insert((Lang::Ja, MsgKey::PromiseFailed), "プロミスが失敗しました");
        messages.insert((Lang::Ja, MsgKey::NotAPromise), "プロミスではありません");
        messages.insert((Lang::Ja, MsgKey::UnexpectedError), "{0}: 予期しないエラー");
        messages.insert((Lang::Ja, MsgKey::RecvArgs), "{0}: 1または3個の引数が必要です: ({0} ch) または ({0} ch :timeout ms)");
        messages.insert((Lang::Ja, MsgKey::TimeoutMustBeMs), "{0}: タイムアウトは整数（ミリ秒）である必要があります");

        // 日本語メッセージ - その他
        messages.insert((Lang::Ja, MsgKey::UnsupportedNumberType), "サポートされていない数値型です");
        messages.insert((Lang::Ja, MsgKey::RailwayRequiresOkError), "|>? には {:ok/:error} マップが必要です");
        messages.insert((Lang::Ja, MsgKey::InvalidTimestamp), "{0}: 不正なタイムスタンプです");
        messages.insert((Lang::Ja, MsgKey::InvalidDateFormat), "{0}: 不正な日付フォーマットです: {1}");
        messages.insert((Lang::Ja, MsgKey::InvalidPercentile), "{0}: パーセンタイルは0から100の間である必要があります");

        // 日本語メッセージ - 警告
        messages.insert((Lang::Ja, MsgKey::RedefineBuiltin), "警告: ビルトイン関数を再定義しています: '{0}' ({1})");
        messages.insert((Lang::Ja, MsgKey::RedefineFunction), "警告: 関数を再定義しています: '{0}'");
        messages.insert((Lang::Ja, MsgKey::RedefineVariable), "警告: 変数を再定義しています: '{0}'");

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
        ui_messages.insert((Lang::En, UiMsg::ExampleStartRepl), "Start REPL");
        ui_messages.insert((Lang::En, UiMsg::ExampleRunScript), "Run script file");
        ui_messages.insert((Lang::En, UiMsg::ExampleExecuteCode), "Execute code and print result");
        ui_messages.insert((Lang::En, UiMsg::ExampleLoadFile), "Load file and start REPL");
        ui_messages.insert((Lang::En, UiMsg::EnvLangQi), "Set language (ja, en)");
        ui_messages.insert((Lang::En, UiMsg::EnvLangSystem), "System locale (auto-detected)");
        ui_messages.insert((Lang::En, UiMsg::VersionString), "Qi version {0}");
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
        ui_messages.insert((Lang::Ja, UiMsg::ExampleStartRepl), "REPLを起動");
        ui_messages.insert((Lang::Ja, UiMsg::ExampleRunScript), "スクリプトファイルを実行");
        ui_messages.insert((Lang::Ja, UiMsg::ExampleExecuteCode), "コードを実行して結果を表示");
        ui_messages.insert((Lang::Ja, UiMsg::ExampleLoadFile), "ファイルを読み込んでREPLを起動");
        ui_messages.insert((Lang::Ja, UiMsg::EnvLangQi), "言語を設定 (ja, en)");
        ui_messages.insert((Lang::Ja, UiMsg::EnvLangSystem), "システムロケール (自動検出)");
        ui_messages.insert((Lang::Ja, UiMsg::VersionString), "Qi バージョン {0}");
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
