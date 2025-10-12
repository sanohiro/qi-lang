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
    RestNeedsVar,       // ...rest の後に変数名が必要

    // レキサーエラー
    UnexpectedChar,
    UnclosedString,

    // 評価器エラー
    UndefinedVar,
    UndefinedVarWithSuggestions, // 未定義変数（サジェスト付き）
    NotAFunction,
    TypeMismatch,       // 型エラー（期待と実際）
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
    Need0Or1Args,       // {0}には0または1個の引数が必要
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
    FStringCodeParseError,  // f-string: コードのパースエラー: {0}

    // マクロエラー
    MacVarargNeedsSymbol,
    VariadicMacroNeedsParams,
    MacArgCountMismatch,        // mac {0}: 引数の数が一致しません（期待: {1}, 実際: {2}）
    MacVariadicArgCountMismatch, // mac {0}: 引数の数が不足しています（最低: {1}, 実際: {2}）

    // quasiquote エラー
    UnquoteOutsideQuasiquote,
    UnquoteSpliceOutsideQuasiquote,
    UnquoteSpliceNeedsListOrVector,

    // loop/recur エラー
    RecurNotFound,
    RecurArgCountMismatch,      // recur: 引数の数が一致しません（期待: {0}, 実際: {1}）

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
    SystemTimeError,     // {0}: system time error: {1}
    JsonParseError,      // {0}: {1}
    JsonStringifyError2, // {0}: {1}
    CannotParseAsInt,    // {0}: cannot parse '{1}' as integer
    CannotConvertToInt,  // {0}: cannot convert {1} to integer
    CannotParseAsFloat,  // {0}: cannot parse '{1}' as float
    CannotConvertToFloat, // {0}: cannot convert {1} to float
    CannotConvertToJson, // Cannot convert {0} to JSON
    InvalidRegex,        // {0}: invalid regex: {1}

    // 警告
    RedefineBuiltin,     // warning: redefining builtin function: {0} ({1})
    RedefineFunction,    // warning: redefining function: {0}
    RedefineVariable,    // warning: redefining variable: {0}

    // CSV エラー
    FileReadError,       // {0}: file read error: {1}
    CsvCannotSerialize,  // csv/stringify: cannot serialize {0}
    CsvRecordMustBeList, // csv/stringify: each record must be a list

    // データ構造エラー
    MustBeQueue,         // {0}: {1} must be a queue
    MustBeStack,         // {0}: {1} must be a stack
    IsEmpty,             // {0}: {1} is empty

    // テストエラー
    TestsFailed,         // Some tests failed
    AssertExpectedException, // Assertion failed: expected exception but none was thrown

    // パスエラー
    AllPathsMustBeStrings, // {0}: all paths must be strings

    // サーバーエラー
    JsonStringifyError,  // Failed to stringify JSON
    RequestMustHave,     // {0}: request must have {1}
    RequestMustBe,       // {0}: request must be {1}
    InvalidFilePath,     // {0}: invalid file path (contains ..)
    FourthArgMustBe,     // {0}'s fourth argument must be {1}
    Need3Args,           // {0} requires 3 arguments
    Need1Or3Args,        // {0}: requires 1 or 3 arguments

    // 環境変数エラー
    ValueMustBeStringNumberBool, // {0}: value must be a string, number, or boolean

    // I/Oエラー
    BothArgsMustBeStrings, // {0}: both arguments must be strings
    UnsupportedEncoding,   // Unsupported encoding: {0}
    KeywordRequiresValue,  // Keyword :{0} requires a value
    ExpectedKeywordArg,    // Expected keyword argument, got {0}
    FileAlreadyExists,     // {0}: file already exists
    InvalidIfExistsOption, // Invalid :if-exists option: {0}

    // HTTPエラー
    HttpClientError,       // HTTP client error: {0}
    HttpCompressionError,  // Compression error: {0}
    HttpStreamClientError, // http stream: client creation error: {0}
    HttpStreamRequestFailed, // http stream: request failed: {0}
    HttpStreamReadBytesFailed, // http stream: failed to read bytes: {0}
    HttpStreamReadBodyFailed,  // http stream: failed to read body: {0}
    HttpRequestUrlRequired,    // http/request: :url is required
    HttpUnsupportedMethod,     // Unsupported HTTP method: {0}
    HttpStreamError,           // http stream: HTTP {0}

    // I/Oエラー（詳細）
    IoFileError,              // {0}: {1}
    IoFailedToDecodeUtf8,     // {0}: failed to decode as UTF-8 (invalid byte sequence)
    IoFailedToCreateDir,      // {0}: failed to create directory: {1}
    IoFailedToOpenForAppend,  // {0}: failed to open for append: {1}
    IoFailedToAppend,         // {0}: failed to append: {1}
    IoFailedToWrite,          // {0}: failed to write: {1}
    FileStreamFailedToOpen,   // file-stream: failed to open '{0}': {1}
    WriteStreamFailedToCreate,// write-stream: failed to create {0}: {1}
    WriteStreamFailedToWrite, // write-stream: failed to write to {0}: {1}
    IoListDirInvalidPattern,  // io/list-dir: invalid pattern '{0}': {1}
    IoListDirFailedToRead,    // io/list-dir: failed to read entry: {0}
    IoCreateDirFailed,        // io/create-dir: failed to create '{0}': {1}
    IoDeleteFileFailed,       // io/delete-file: failed to delete '{0}': {1}
    IoDeleteDirFailed,        // io/delete-dir: failed to delete '{0}': {1}
    IoCopyFileFailed,         // io/copy-file: failed to copy '{0}' to '{1}': {2}
    IoMoveFileFailed,         // io/move-file: failed to move '{0}' to '{1}': {2}
    IoGetMetadataFailed,      // io/file-info: failed to get metadata for '{0}': {1}

    // サーバーエラー（詳細）
    ServerFailedToReadBody,   // Failed to read request body: {0}
    ServerFailedToDecompressGzip, // Failed to decompress gzip body: {0}
    ServerFailedToBuildResponse,  // Failed to build response: {0}
    ServerStaticFileMetadataFailed, // server/static-file: failed to read file metadata: {0}
    ServerHandlerMustReturnMap,     // Handler must return a map, got: {0}
    ServerHandlerMustBeFunction,    // Handler must be a function or router, got: {0}
    ServerHandlerError,             // Handler error: {0}
    ServerFileTooLarge,             // File too large: {0} bytes (max: {1} bytes / {2} MB). Path: {3}
    ServerFailedToReadFile,         // Failed to read file: {0}
    ServerStaticFileTooLarge,       // server/static-file: file too large: {0} bytes (max: {1} bytes / {2} MB). Consider using streaming in the future.
    ServerStaticFileFailedToRead,   // server/static-file: failed to read file: {0}
    ServerStaticDirNotDirectory,    // server/static-dir: {0} is not a directory

    // SQLiteエラー
    SqliteFailedToOpen,       // Failed to open SQLite database: {0}
    SqliteFailedToSetTimeout, // Failed to set timeout: {0}
    SqliteFailedToGetColumnName, // Failed to get column name: {0}
    SqliteFailedToPrepare,    // Failed to prepare statement: {0}
    SqliteFailedToExecuteQuery, // Failed to execute query: {0}
    SqliteFailedToExecuteStatement, // Failed to execute statement: {0}
    SqliteFailedToBeginTransaction, // Failed to begin transaction: {0}
    SqliteFailedToCommitTransaction, // Failed to commit transaction: {0}
    SqliteFailedToRollbackTransaction, // Failed to rollback transaction: {0}

    // 環境変数エラー（詳細）
    EnvLoadDotenvFailedToRead, // env/load-dotenv: failed to read file '{0}': {1}
    EnvLoadDotenvInvalidFormat, // env/load-dotenv: invalid format at line {0}: '{1}'

    // CSVエラー（詳細）
    CsvWriteFileStringifyFailed, // csv/write-file: stringify failed
    CsvWriteFileFailedToWrite,   // csv/write-file: failed to write '{0}': {1}

    // ログエラー
    LogSetLevelInvalidLevel,     // log/set-level: invalid level '{0}' (valid: debug, info, warn, error)
    LogSetFormatInvalidFormat,   // log/set-format: invalid format '{0}' (valid: text, json)

    // 時刻エラー（詳細）
    TimeParseFailedToParse,      // time/parse: failed to parse '{0}' with format '{1}'

    // ZIPエラー
    ZipPathDoesNotExist,         // {0}: path '{1}' does not exist

    // データベースエラー
    DbUnsupportedUrl,            // Unsupported database URL: {0}. Supported: sqlite:
    DbNeed2To4Args,              // {0} requires 2-4 arguments, got {1}
    DbExpectedConnection,        // Expected DbConnection, got: {0}
    DbConnectionNotFound,        // Connection not found: {0}
    DbExpectedTransaction,       // Expected DbTransaction, got: {0}
    DbTransactionNotFound,       // Transaction not found: {0}
    DbExpectedConnectionOrTransaction,  // Expected DbConnection or DbTransaction, got: {0}

    // I/Oエラー（追加）
    IoFailedToDecodeAs,          // {0}: failed to decode as {1} (invalid byte sequence)
    IoCouldNotDetectEncoding,    // {0}: could not detect encoding (tried UTF-8, UTF-16, Japanese, Chinese, Korean, European encodings)
    IoAppendFileFailedToWrite,   // append-file: failed to write {0}: {1}
    IoAppendFileFailedToOpen,    // append-file: failed to open {0}: {1}
    IoReadLinesFailedToRead,     // read-lines: failed to read {0}: {1}

    // Featureエラー
    FeatureDisabled,             // {0} support is disabled. Build with feature '{1}': {2}
    DbUnsupportedDriver,         // Unsupported database driver: {0}
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
    ReplTypeHelp,
    ReplAvailableCommands,
    ReplNoVariables,
    ReplDefinedVariables,
    ReplNoFunctions,
    ReplDefinedFunctions,
    ReplNoBuiltinsMatching,
    ReplBuiltinsMatching,
    ReplBuiltinFunctions,
    ReplBuiltinTotal,
    ReplBuiltinTip,
    ReplEnvCleared,
    ReplLoadUsage,
    ReplNoFileLoaded,
    ReplUnknownCommand,
    ReplTypeHelpForCommands,

    // REPLコマンドヘルプ
    ReplCommandHelp,
    ReplCommandVars,
    ReplCommandFuncs,
    ReplCommandBuiltins,
    ReplCommandClear,
    ReplCommandLoad,
    ReplCommandReload,
    ReplCommandQuit,

    // テスト
    TestNoTests,
    TestResults,
    TestResultsSeparator,
    TestSummary,
    TestAssertEqFailed,
    TestAssertTruthyFailed,
    TestAssertFalsyFailed,

    // プロファイラー
    ProfileNoData,
    ProfileUseStart,
    ProfileReport,
    ProfileTableHeader,
    ProfileTotalTime,

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
        messages.insert((Lang::En, MsgKey::RestNeedsVar), "'...' requires a variable name");
        messages.insert((Lang::En, MsgKey::UnexpectedChar), "unexpected character: {0}");
        messages.insert((Lang::En, MsgKey::UnclosedString), "unclosed string");

        // 英語メッセージ - 評価器
        messages.insert((Lang::En, MsgKey::UndefinedVar), "undefined variable: {0}");
        messages.insert((Lang::En, MsgKey::UndefinedVarWithSuggestions), "undefined variable: {0}\n      Did you mean: {1}?");
        messages.insert((Lang::En, MsgKey::NotAFunction), "not a function: {0}");
        messages.insert((Lang::En, MsgKey::TypeMismatch), "type error: expected {0}, got {1} ({2})");
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
        messages.insert((Lang::En, MsgKey::Need0Or1Args), "{0} requires 0 or 1 argument");
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
        messages.insert((Lang::En, MsgKey::FStringCodeParseError), "f-string: code parse error: {0}");

        // 英語メッセージ - マクロ
        messages.insert((Lang::En, MsgKey::MacVarargNeedsSymbol), "mac: '&' requires a symbol");
        messages.insert((Lang::En, MsgKey::VariadicMacroNeedsParams), "variadic macro requires parameters");
        messages.insert((Lang::En, MsgKey::MacArgCountMismatch), "mac {0}: argument count mismatch (expected {1}, got {2})");
        messages.insert((Lang::En, MsgKey::MacVariadicArgCountMismatch), "mac {0}: insufficient arguments (minimum {1}, got {2})");

        // 英語メッセージ - quasiquote
        messages.insert((Lang::En, MsgKey::UnquoteOutsideQuasiquote), "unquote: can only be used inside quasiquote");
        messages.insert((Lang::En, MsgKey::UnquoteSpliceOutsideQuasiquote), "unquote-splice: can only be used inside quasiquote");
        messages.insert((Lang::En, MsgKey::UnquoteSpliceNeedsListOrVector), "unquote-splice: requires a list or vector");

        // 英語メッセージ - loop/recur
        messages.insert((Lang::En, MsgKey::RecurNotFound), "recur not found");
        messages.insert((Lang::En, MsgKey::RecurArgCountMismatch), "recur: argument count mismatch (expected {0}, got {1})");

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
        messages.insert((Lang::En, MsgKey::SystemTimeError), "{0}: system time error: {1}");
        messages.insert((Lang::En, MsgKey::JsonParseError), "{0}: {1}");
        messages.insert((Lang::En, MsgKey::JsonStringifyError2), "{0}: {1}");
        messages.insert((Lang::En, MsgKey::CannotParseAsInt), "{0}: cannot parse '{1}' as integer");
        messages.insert((Lang::En, MsgKey::CannotConvertToInt), "{0}: cannot convert {1} to integer");
        messages.insert((Lang::En, MsgKey::CannotParseAsFloat), "{0}: cannot parse '{1}' as float");
        messages.insert((Lang::En, MsgKey::CannotConvertToFloat), "{0}: cannot convert {1} to float");
        messages.insert((Lang::En, MsgKey::CannotConvertToJson), "Cannot convert {0} to JSON");
        messages.insert((Lang::En, MsgKey::InvalidRegex), "{0}: invalid regex: {1}");

        // 英語メッセージ - 警告
        messages.insert((Lang::En, MsgKey::RedefineBuiltin), "warning: redefining builtin function '{0}' ({1})");
        messages.insert((Lang::En, MsgKey::RedefineFunction), "warning: redefining function '{0}'");
        messages.insert((Lang::En, MsgKey::RedefineVariable), "warning: redefining variable '{0}'");

        // 英語メッセージ - CSV
        messages.insert((Lang::En, MsgKey::FileReadError), "{0}: file read error: {1}");
        messages.insert((Lang::En, MsgKey::CsvCannotSerialize), "csv/stringify: cannot serialize {0}");
        messages.insert((Lang::En, MsgKey::CsvRecordMustBeList), "csv/stringify: each record must be a list");

        // 英語メッセージ - データ構造
        messages.insert((Lang::En, MsgKey::MustBeQueue), "{0}: {1} must be a queue");
        messages.insert((Lang::En, MsgKey::MustBeStack), "{0}: {1} must be a stack");
        messages.insert((Lang::En, MsgKey::IsEmpty), "{0}: {1} is empty");

        // 英語メッセージ - テスト
        messages.insert((Lang::En, MsgKey::TestsFailed), "Some tests failed");
        messages.insert((Lang::En, MsgKey::AssertExpectedException), "Assertion failed: expected exception but none was thrown");

        // 英語メッセージ - パス
        messages.insert((Lang::En, MsgKey::AllPathsMustBeStrings), "{0}: all paths must be strings");

        // 英語メッセージ - サーバー
        messages.insert((Lang::En, MsgKey::JsonStringifyError), "Failed to stringify JSON");
        messages.insert((Lang::En, MsgKey::RequestMustHave), "{0}: request must have {1}");
        messages.insert((Lang::En, MsgKey::RequestMustBe), "{0}: request must be {1}");
        messages.insert((Lang::En, MsgKey::InvalidFilePath), "{0}: invalid file path (contains ..)");
        messages.insert((Lang::En, MsgKey::FourthArgMustBe), "{0}'s fourth argument must be {1}");
        messages.insert((Lang::En, MsgKey::Need3Args), "{0} requires 3 arguments");
        messages.insert((Lang::En, MsgKey::Need1Or3Args), "{0}: requires 1 or 3 arguments");

        // 英語メッセージ - 環境変数
        messages.insert((Lang::En, MsgKey::ValueMustBeStringNumberBool), "{0}: value must be a string, number, or boolean");

        // 英語メッセージ - I/O
        messages.insert((Lang::En, MsgKey::BothArgsMustBeStrings), "{0}: both arguments must be strings");
        messages.insert((Lang::En, MsgKey::UnsupportedEncoding), "Unsupported encoding: {0}");
        messages.insert((Lang::En, MsgKey::KeywordRequiresValue), "Keyword :{0} requires a value");
        messages.insert((Lang::En, MsgKey::ExpectedKeywordArg), "Expected keyword argument, got {0}");
        messages.insert((Lang::En, MsgKey::FileAlreadyExists), "{0}: file already exists");
        messages.insert((Lang::En, MsgKey::InvalidIfExistsOption), "Invalid :if-exists option: {0}");

        // 英語メッセージ - HTTP
        messages.insert((Lang::En, MsgKey::HttpClientError), "HTTP client error: {0}");
        messages.insert((Lang::En, MsgKey::HttpCompressionError), "Compression error: {0}");
        messages.insert((Lang::En, MsgKey::HttpStreamClientError), "http stream: client creation error: {0}");
        messages.insert((Lang::En, MsgKey::HttpStreamRequestFailed), "http stream: request failed: {0}");
        messages.insert((Lang::En, MsgKey::HttpStreamReadBytesFailed), "http stream: failed to read bytes: {0}");
        messages.insert((Lang::En, MsgKey::HttpStreamReadBodyFailed), "http stream: failed to read body: {0}");
        messages.insert((Lang::En, MsgKey::HttpRequestUrlRequired), "http/request: :url is required");
        messages.insert((Lang::En, MsgKey::HttpUnsupportedMethod), "Unsupported HTTP method: {0}");
        messages.insert((Lang::En, MsgKey::HttpStreamError), "http stream: HTTP {0}");

        // 英語メッセージ - I/O（詳細）
        messages.insert((Lang::En, MsgKey::IoFileError), "{0}: {1}");
        messages.insert((Lang::En, MsgKey::IoFailedToDecodeUtf8), "{0}: failed to decode as UTF-8 (invalid byte sequence)");
        messages.insert((Lang::En, MsgKey::IoFailedToCreateDir), "{0}: failed to create directory: {1}");
        messages.insert((Lang::En, MsgKey::IoFailedToOpenForAppend), "{0}: failed to open for append: {1}");
        messages.insert((Lang::En, MsgKey::IoFailedToAppend), "{0}: failed to append: {1}");
        messages.insert((Lang::En, MsgKey::IoFailedToWrite), "{0}: failed to write: {1}");
        messages.insert((Lang::En, MsgKey::FileStreamFailedToOpen), "file-stream: failed to open '{0}': {1}");
        messages.insert((Lang::En, MsgKey::WriteStreamFailedToCreate), "write-stream: failed to create {0}: {1}");
        messages.insert((Lang::En, MsgKey::WriteStreamFailedToWrite), "write-stream: failed to write to {0}: {1}");
        messages.insert((Lang::En, MsgKey::IoListDirInvalidPattern), "io/list-dir: invalid pattern '{0}': {1}");
        messages.insert((Lang::En, MsgKey::IoListDirFailedToRead), "io/list-dir: failed to read entry: {0}");
        messages.insert((Lang::En, MsgKey::IoCreateDirFailed), "io/create-dir: failed to create '{0}': {1}");
        messages.insert((Lang::En, MsgKey::IoDeleteFileFailed), "io/delete-file: failed to delete '{0}': {1}");
        messages.insert((Lang::En, MsgKey::IoDeleteDirFailed), "io/delete-dir: failed to delete '{0}': {1}");
        messages.insert((Lang::En, MsgKey::IoCopyFileFailed), "io/copy-file: failed to copy '{0}' to '{1}': {2}");
        messages.insert((Lang::En, MsgKey::IoMoveFileFailed), "io/move-file: failed to move '{0}' to '{1}': {2}");
        messages.insert((Lang::En, MsgKey::IoGetMetadataFailed), "io/file-info: failed to get metadata for '{0}': {1}");

        // 英語メッセージ - サーバー（詳細）
        messages.insert((Lang::En, MsgKey::ServerFailedToReadBody), "Failed to read request body: {0}");
        messages.insert((Lang::En, MsgKey::ServerFailedToDecompressGzip), "Failed to decompress gzip body: {0}");
        messages.insert((Lang::En, MsgKey::ServerFailedToBuildResponse), "Failed to build response: {0}");
        messages.insert((Lang::En, MsgKey::ServerStaticFileMetadataFailed), "server/static-file: failed to read file metadata: {0}");
        messages.insert((Lang::En, MsgKey::ServerHandlerMustReturnMap), "Handler must return a map, got: {0}");
        messages.insert((Lang::En, MsgKey::ServerHandlerMustBeFunction), "Handler must be a function or router, got: {0}");
        messages.insert((Lang::En, MsgKey::ServerHandlerError), "Handler error: {0}");
        messages.insert((Lang::En, MsgKey::ServerFileTooLarge), "File too large: {0} bytes (max: {1} bytes / {2} MB). Path: {3}");
        messages.insert((Lang::En, MsgKey::ServerFailedToReadFile), "Failed to read file: {0}");
        messages.insert((Lang::En, MsgKey::ServerStaticFileTooLarge), "server/static-file: file too large: {0} bytes (max: {1} bytes / {2} MB). Consider using streaming in the future.");
        messages.insert((Lang::En, MsgKey::ServerStaticFileFailedToRead), "server/static-file: failed to read file: {0}");
        messages.insert((Lang::En, MsgKey::ServerStaticDirNotDirectory), "server/static-dir: {0} is not a directory");

        // 英語メッセージ - SQLite
        messages.insert((Lang::En, MsgKey::SqliteFailedToOpen), "Failed to open SQLite database: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToSetTimeout), "Failed to set timeout: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToGetColumnName), "Failed to get column name: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToPrepare), "Failed to prepare statement: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToExecuteQuery), "Failed to execute query: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToExecuteStatement), "Failed to execute statement: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToBeginTransaction), "Failed to begin transaction: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToCommitTransaction), "Failed to commit transaction: {0}");
        messages.insert((Lang::En, MsgKey::SqliteFailedToRollbackTransaction), "Failed to rollback transaction: {0}");

        // 英語メッセージ - 環境変数（詳細）
        messages.insert((Lang::En, MsgKey::EnvLoadDotenvFailedToRead), "env/load-dotenv: failed to read file '{0}': {1}");
        messages.insert((Lang::En, MsgKey::EnvLoadDotenvInvalidFormat), "env/load-dotenv: invalid format at line {0}: '{1}'");

        // 英語メッセージ - CSV（詳細）
        messages.insert((Lang::En, MsgKey::CsvWriteFileStringifyFailed), "csv/write-file: stringify failed");
        messages.insert((Lang::En, MsgKey::CsvWriteFileFailedToWrite), "csv/write-file: failed to write '{0}': {1}");

        // 英語メッセージ - ログ
        messages.insert((Lang::En, MsgKey::LogSetLevelInvalidLevel), "log/set-level: invalid level '{0}' (valid: debug, info, warn, error)");
        messages.insert((Lang::En, MsgKey::LogSetFormatInvalidFormat), "log/set-format: invalid format '{0}' (valid: text, json)");

        // 英語メッセージ - 時刻（詳細）
        messages.insert((Lang::En, MsgKey::TimeParseFailedToParse), "time/parse: failed to parse '{0}' with format '{1}'");

        // 英語メッセージ - ZIP
        messages.insert((Lang::En, MsgKey::ZipPathDoesNotExist), "{0}: path '{1}' does not exist");

        // 英語メッセージ - データベース
        messages.insert((Lang::En, MsgKey::DbUnsupportedUrl), "Unsupported database URL: {0}. Supported: sqlite:");
        messages.insert((Lang::En, MsgKey::DbNeed2To4Args), "{0} requires 2-4 arguments, got {1}");
        messages.insert((Lang::En, MsgKey::DbExpectedConnection), "Expected DbConnection, got: {0}");
        messages.insert((Lang::En, MsgKey::DbConnectionNotFound), "Connection not found: {0}");
        messages.insert((Lang::En, MsgKey::DbExpectedTransaction), "Expected DbTransaction, got: {0}");
        messages.insert((Lang::En, MsgKey::DbTransactionNotFound), "Transaction not found: {0}");
        messages.insert((Lang::En, MsgKey::DbExpectedConnectionOrTransaction), "Expected DbConnection or DbTransaction, got: {0}");

        // 英語メッセージ - I/O（追加）
        messages.insert((Lang::En, MsgKey::IoFailedToDecodeAs), "{0}: failed to decode as {1} (invalid byte sequence)");
        messages.insert((Lang::En, MsgKey::IoCouldNotDetectEncoding), "{0}: could not detect encoding (tried UTF-8, UTF-16, Japanese, Chinese, Korean, European encodings)");
        messages.insert((Lang::En, MsgKey::IoAppendFileFailedToWrite), "append-file: failed to write {0}: {1}");
        messages.insert((Lang::En, MsgKey::IoAppendFileFailedToOpen), "append-file: failed to open {0}: {1}");
        messages.insert((Lang::En, MsgKey::IoReadLinesFailedToRead), "read-lines: failed to read {0}: {1}");

        // 英語メッセージ - Feature
        messages.insert((Lang::En, MsgKey::FeatureDisabled), "{0} support is disabled. Build with feature '{1}': {2}");
        messages.insert((Lang::En, MsgKey::DbUnsupportedDriver), "Unsupported database driver: {0}");

        // 日本語メッセージ - パーサー/レキサー
        messages.insert((Lang::Ja, MsgKey::UnexpectedToken), "予期しないトークン: {0}");
        messages.insert((Lang::Ja, MsgKey::UnexpectedEof), "予期しないEOF");
        messages.insert((Lang::Ja, MsgKey::ExpectedToken), "期待: {0}, 実際: {1}");
        messages.insert((Lang::Ja, MsgKey::NeedsSymbol), "{0}にはシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::VarargNeedsName), "&の後には変数名が必要です");
        messages.insert((Lang::Ja, MsgKey::UnexpectedPattern), "予期しないパターン: {0}");
        messages.insert((Lang::Ja, MsgKey::RestNeedsVar), "...の後には変数名が必要です");
        messages.insert((Lang::Ja, MsgKey::UnexpectedChar), "予期しない文字: {0}");
        messages.insert((Lang::Ja, MsgKey::UnclosedString), "文字列が閉じられていません");

        // 日本語メッセージ - 評価器
        messages.insert((Lang::Ja, MsgKey::UndefinedVar), "未定義の変数: {0}");
        messages.insert((Lang::Ja, MsgKey::UndefinedVarWithSuggestions), "未定義の変数: {0}\n      もしかして: {1}?");
        messages.insert((Lang::Ja, MsgKey::NotAFunction), "関数ではありません: {0}");
        messages.insert((Lang::Ja, MsgKey::TypeMismatch), "型エラー: 期待={0}, 実際={1} ({2})");
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
        messages.insert((Lang::Ja, MsgKey::Need0Or1Args), "{0}には0または1個の引数が必要です");
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
        messages.insert((Lang::Ja, MsgKey::FStringCodeParseError), "f-string: コードのパースエラー: {0}");

        // 日本語メッセージ - マクロ
        messages.insert((Lang::Ja, MsgKey::MacVarargNeedsSymbol), "mac: &の後にシンボルが必要です");
        messages.insert((Lang::Ja, MsgKey::VariadicMacroNeedsParams), "可変長マクロはパラメータが必要です");
        messages.insert((Lang::Ja, MsgKey::MacArgCountMismatch), "mac {0}: 引数の数が一致しません（期待: {1}, 実際: {2}）");
        messages.insert((Lang::Ja, MsgKey::MacVariadicArgCountMismatch), "mac {0}: 引数の数が不足しています（最低: {1}, 実際: {2}）");

        // 日本語メッセージ - quasiquote
        messages.insert((Lang::Ja, MsgKey::UnquoteOutsideQuasiquote), "unquote: quasiquote外では使用できません");
        messages.insert((Lang::Ja, MsgKey::UnquoteSpliceOutsideQuasiquote), "unquote-splice: quasiquote外では使用できません");
        messages.insert((Lang::Ja, MsgKey::UnquoteSpliceNeedsListOrVector), "unquote-splice: リストまたはベクタが必要です");

        // 日本語メッセージ - loop/recur
        messages.insert((Lang::Ja, MsgKey::RecurNotFound), "recurが見つかりません");
        messages.insert((Lang::Ja, MsgKey::RecurArgCountMismatch), "recur: 引数の数が一致しません（期待: {0}, 実際: {1}）");

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
        messages.insert((Lang::Ja, MsgKey::SystemTimeError), "{0}: システム時刻エラー: {1}");
        messages.insert((Lang::Ja, MsgKey::JsonParseError), "{0}: {1}");
        messages.insert((Lang::Ja, MsgKey::JsonStringifyError2), "{0}: {1}");
        messages.insert((Lang::Ja, MsgKey::CannotParseAsInt), "{0}: '{1}' を整数としてパースできません");
        messages.insert((Lang::Ja, MsgKey::CannotConvertToInt), "{0}: {1} を整数に変換できません");
        messages.insert((Lang::Ja, MsgKey::CannotParseAsFloat), "{0}: '{1}' を浮動小数点数としてパースできません");
        messages.insert((Lang::Ja, MsgKey::CannotConvertToFloat), "{0}: {1} を浮動小数点数に変換できません");
        messages.insert((Lang::Ja, MsgKey::CannotConvertToJson), "{0} をJSONに変換できません");
        messages.insert((Lang::Ja, MsgKey::InvalidRegex), "{0}: 不正な正規表現: {1}");

        // 日本語メッセージ - 警告
        messages.insert((Lang::Ja, MsgKey::RedefineBuiltin), "警告: ビルトイン関数を再定義しています: '{0}' ({1})");
        messages.insert((Lang::Ja, MsgKey::RedefineFunction), "警告: 関数を再定義しています: '{0}'");
        messages.insert((Lang::Ja, MsgKey::RedefineVariable), "警告: 変数を再定義しています: '{0}'");

        // 日本語メッセージ - CSV
        messages.insert((Lang::Ja, MsgKey::FileReadError), "{0}: ファイル読み込みエラー: {1}");
        messages.insert((Lang::Ja, MsgKey::CsvCannotSerialize), "csv/stringify: シリアライズできません: {0}");
        messages.insert((Lang::Ja, MsgKey::CsvRecordMustBeList), "csv/stringify: 各レコードはリストである必要があります");

        // 日本語メッセージ - データ構造
        messages.insert((Lang::Ja, MsgKey::MustBeQueue), "{0}: {1}はキューである必要があります");
        messages.insert((Lang::Ja, MsgKey::MustBeStack), "{0}: {1}はスタックである必要があります");
        messages.insert((Lang::Ja, MsgKey::IsEmpty), "{0}: {1}は空です");

        // 日本語メッセージ - テスト
        messages.insert((Lang::Ja, MsgKey::TestsFailed), "いくつかのテストが失敗しました");
        messages.insert((Lang::Ja, MsgKey::AssertExpectedException), "アサーション失敗: 例外が期待されましたが発生しませんでした");

        // 日本語メッセージ - パス
        messages.insert((Lang::Ja, MsgKey::AllPathsMustBeStrings), "{0}: すべてのパスは文字列である必要があります");

        // 日本語メッセージ - サーバー
        messages.insert((Lang::Ja, MsgKey::JsonStringifyError), "JSON文字列化に失敗しました");
        messages.insert((Lang::Ja, MsgKey::RequestMustHave), "{0}: リクエストには{1}が必要です");
        messages.insert((Lang::Ja, MsgKey::RequestMustBe), "{0}: リクエストは{1}である必要があります");
        messages.insert((Lang::Ja, MsgKey::InvalidFilePath), "{0}: 不正なファイルパスです (.. が含まれています)");
        messages.insert((Lang::Ja, MsgKey::FourthArgMustBe), "{0}の第4引数は{1}が必要です");
        messages.insert((Lang::Ja, MsgKey::Need3Args), "{0}には3つの引数が必要です");
        messages.insert((Lang::Ja, MsgKey::Need1Or3Args), "{0}: 1または3個の引数が必要です");

        // 日本語メッセージ - 環境変数
        messages.insert((Lang::Ja, MsgKey::ValueMustBeStringNumberBool), "{0}: 値は文字列、数値、または真偽値である必要があります");

        // 日本語メッセージ - I/O
        messages.insert((Lang::Ja, MsgKey::BothArgsMustBeStrings), "{0}: 両方の引数は文字列である必要があります");
        messages.insert((Lang::Ja, MsgKey::UnsupportedEncoding), "サポートされていないエンコーディングです: {0}");
        messages.insert((Lang::Ja, MsgKey::KeywordRequiresValue), "キーワード :{0} には値が必要です");
        messages.insert((Lang::Ja, MsgKey::ExpectedKeywordArg), "キーワード引数が必要です。実際: {0}");
        messages.insert((Lang::Ja, MsgKey::FileAlreadyExists), "{0}: ファイルが既に存在します");
        messages.insert((Lang::Ja, MsgKey::InvalidIfExistsOption), "不正な :if-exists オプション: {0}");

        // 日本語メッセージ - HTTP
        messages.insert((Lang::Ja, MsgKey::HttpClientError), "HTTPクライアントエラー: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpCompressionError), "圧縮エラー: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpStreamClientError), "http stream: クライアント作成エラー: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpStreamRequestFailed), "http stream: リクエスト失敗: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpStreamReadBytesFailed), "http stream: バイト読み込み失敗: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpStreamReadBodyFailed), "http stream: ボディ読み込み失敗: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpRequestUrlRequired), "http/request: :url は必須です");
        messages.insert((Lang::Ja, MsgKey::HttpUnsupportedMethod), "未サポートのHTTPメソッド: {0}");
        messages.insert((Lang::Ja, MsgKey::HttpStreamError), "http stream: HTTP {0}");

        // 日本語メッセージ - I/O（詳細）
        messages.insert((Lang::Ja, MsgKey::IoFileError), "{0}: {1}");
        messages.insert((Lang::Ja, MsgKey::IoFailedToDecodeUtf8), "{0}: UTF-8としてデコードできませんでした (無効なバイト列)");
        messages.insert((Lang::Ja, MsgKey::IoFailedToCreateDir), "{0}: ディレクトリの作成に失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoFailedToOpenForAppend), "{0}: 追記用のオープンに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoFailedToAppend), "{0}: 追記に失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoFailedToWrite), "{0}: 書き込みに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::FileStreamFailedToOpen), "file-stream: '{0}' のオープンに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::WriteStreamFailedToCreate), "write-stream: {0} の作成に失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::WriteStreamFailedToWrite), "write-stream: {0} への書き込みに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoListDirInvalidPattern), "io/list-dir: 不正なパターン '{0}': {1}");
        messages.insert((Lang::Ja, MsgKey::IoListDirFailedToRead), "io/list-dir: エントリの読み込みに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::IoCreateDirFailed), "io/create-dir: '{0}' の作成に失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoDeleteFileFailed), "io/delete-file: '{0}' の削除に失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoDeleteDirFailed), "io/delete-dir: '{0}' の削除に失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoCopyFileFailed), "io/copy-file: '{0}' から '{1}' へのコピーに失敗しました: {2}");
        messages.insert((Lang::Ja, MsgKey::IoMoveFileFailed), "io/move-file: '{0}' から '{1}' への移動に失敗しました: {2}");
        messages.insert((Lang::Ja, MsgKey::IoGetMetadataFailed), "io/file-info: '{0}' のメタデータ取得に失敗しました: {1}");

        // 日本語メッセージ - サーバー（詳細）
        messages.insert((Lang::Ja, MsgKey::ServerFailedToReadBody), "リクエストボディの読み込みに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerFailedToDecompressGzip), "gzipボディの解凍に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerFailedToBuildResponse), "レスポンスの構築に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerStaticFileMetadataFailed), "server/static-file: ファイルメタデータの読み込みに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerHandlerMustReturnMap), "ハンドラーはマップを返す必要があります。実際: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerHandlerMustBeFunction), "ハンドラーは関数またはルーターである必要があります。実際: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerHandlerError), "ハンドラーエラー: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerFileTooLarge), "ファイルが大きすぎます: {0} バイト (最大: {1} バイト / {2} MB)。パス: {3}");
        messages.insert((Lang::Ja, MsgKey::ServerFailedToReadFile), "ファイルの読み込みに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerStaticFileTooLarge), "server/static-file: ファイルが大きすぎます: {0} バイト (最大: {1} バイト / {2} MB)。今後ストリーミングの使用を検討してください。");
        messages.insert((Lang::Ja, MsgKey::ServerStaticFileFailedToRead), "server/static-file: ファイルの読み込みに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::ServerStaticDirNotDirectory), "server/static-dir: {0} はディレクトリではありません");

        // 日本語メッセージ - SQLite
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToOpen), "SQLiteデータベースのオープンに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToSetTimeout), "タイムアウトの設定に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToGetColumnName), "カラム名の取得に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToPrepare), "ステートメントの準備に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToExecuteQuery), "クエリの実行に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToExecuteStatement), "ステートメントの実行に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToBeginTransaction), "トランザクションの開始に失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToCommitTransaction), "トランザクションのコミットに失敗しました: {0}");
        messages.insert((Lang::Ja, MsgKey::SqliteFailedToRollbackTransaction), "トランザクションのロールバックに失敗しました: {0}");

        // 日本語メッセージ - 環境変数（詳細）
        messages.insert((Lang::Ja, MsgKey::EnvLoadDotenvFailedToRead), "env/load-dotenv: ファイル '{0}' の読み込みに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::EnvLoadDotenvInvalidFormat), "env/load-dotenv: {0}行目のフォーマットが不正です: '{1}'");

        // 日本語メッセージ - CSV（詳細）
        messages.insert((Lang::Ja, MsgKey::CsvWriteFileStringifyFailed), "csv/write-file: 文字列化に失敗しました");
        messages.insert((Lang::Ja, MsgKey::CsvWriteFileFailedToWrite), "csv/write-file: '{0}' への書き込みに失敗しました: {1}");

        // 日本語メッセージ - ログ
        messages.insert((Lang::Ja, MsgKey::LogSetLevelInvalidLevel), "log/set-level: 不正なレベル '{0}' です (有効な値: debug, info, warn, error)");
        messages.insert((Lang::Ja, MsgKey::LogSetFormatInvalidFormat), "log/set-format: 不正なフォーマット '{0}' です (有効な値: text, json)");

        // 日本語メッセージ - 時刻（詳細）
        messages.insert((Lang::Ja, MsgKey::TimeParseFailedToParse), "time/parse: '{0}' をフォーマット '{1}' でパースできませんでした");

        // 日本語メッセージ - ZIP
        messages.insert((Lang::Ja, MsgKey::ZipPathDoesNotExist), "{0}: パス '{1}' が存在しません");

        // 日本語メッセージ - データベース
        messages.insert((Lang::Ja, MsgKey::DbUnsupportedUrl), "サポートされていないデータベースURL: {0} (対応: sqlite:)");
        messages.insert((Lang::Ja, MsgKey::DbNeed2To4Args), "{0}には2〜4個の引数が必要です。実際: {1}個");
        messages.insert((Lang::Ja, MsgKey::DbExpectedConnection), "DbConnectionが期待されましたが、実際: {0}");
        messages.insert((Lang::Ja, MsgKey::DbConnectionNotFound), "接続が見つかりません: {0}");
        messages.insert((Lang::Ja, MsgKey::DbExpectedTransaction), "DbTransactionが期待されましたが、実際: {0}");
        messages.insert((Lang::Ja, MsgKey::DbTransactionNotFound), "トランザクションが見つかりません: {0}");
        messages.insert((Lang::Ja, MsgKey::DbExpectedConnectionOrTransaction), "DbConnectionまたはDbTransactionが期待されましたが、実際: {0}");

        // 日本語メッセージ - I/O（追加）
        messages.insert((Lang::Ja, MsgKey::IoFailedToDecodeAs), "{0}: {1}としてデコードできませんでした (無効なバイト列)");
        messages.insert((Lang::Ja, MsgKey::IoCouldNotDetectEncoding), "{0}: エンコーディングを検出できませんでした (UTF-8, UTF-16, 日本語, 中国語, 韓国語, ヨーロッパ言語のエンコーディングを試行)");
        messages.insert((Lang::Ja, MsgKey::IoAppendFileFailedToWrite), "append-file: {0} への書き込みに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoAppendFileFailedToOpen), "append-file: {0} のオープンに失敗しました: {1}");
        messages.insert((Lang::Ja, MsgKey::IoReadLinesFailedToRead), "read-lines: {0} の読み込みに失敗しました: {1}");

        // 日本語メッセージ - Feature
        messages.insert((Lang::Ja, MsgKey::FeatureDisabled), "{0} サポートは無効化されています。feature '{1}' を有効にしてビルドしてください: {2}");
        messages.insert((Lang::Ja, MsgKey::DbUnsupportedDriver), "サポートされていないデータベースドライバー: {0}");

        // UIメッセージ
        let mut ui_messages = HashMap::new();

        // 英語UI - REPL基本
        ui_messages.insert((Lang::En, UiMsg::ReplWelcome), "Qi REPL v{0}");
        ui_messages.insert((Lang::En, UiMsg::ReplPressCtrlC), "Press Ctrl+C to exit");
        ui_messages.insert((Lang::En, UiMsg::ReplGoodbye), "Goodbye!");
        ui_messages.insert((Lang::En, UiMsg::ReplLoading), "Loading {0}...");
        ui_messages.insert((Lang::En, UiMsg::ReplLoaded), "Loaded.");
        ui_messages.insert((Lang::En, UiMsg::ReplTypeHelp), "Type :help for REPL commands");
        ui_messages.insert((Lang::En, UiMsg::ReplAvailableCommands), "Available REPL commands:");
        ui_messages.insert((Lang::En, UiMsg::ReplNoVariables), "No variables defined");
        ui_messages.insert((Lang::En, UiMsg::ReplDefinedVariables), "Defined variables:");
        ui_messages.insert((Lang::En, UiMsg::ReplNoFunctions), "No user-defined functions");
        ui_messages.insert((Lang::En, UiMsg::ReplDefinedFunctions), "User-defined functions:");
        ui_messages.insert((Lang::En, UiMsg::ReplNoBuiltinsMatching), "No builtin functions matching '{0}'");
        ui_messages.insert((Lang::En, UiMsg::ReplBuiltinsMatching), "Builtin functions matching '{0}':");
        ui_messages.insert((Lang::En, UiMsg::ReplBuiltinFunctions), "Builtin functions:");
        ui_messages.insert((Lang::En, UiMsg::ReplBuiltinTotal), "Total: {0} functions");
        ui_messages.insert((Lang::En, UiMsg::ReplBuiltinTip), "Tip: Use ':builtins <pattern>' to filter (e.g., ':builtins str')");
        ui_messages.insert((Lang::En, UiMsg::ReplEnvCleared), "Environment cleared");
        ui_messages.insert((Lang::En, UiMsg::ReplLoadUsage), "Usage: :load <file>");
        ui_messages.insert((Lang::En, UiMsg::ReplNoFileLoaded), "No file has been loaded yet");
        ui_messages.insert((Lang::En, UiMsg::ReplUnknownCommand), "Unknown command: {0}");
        ui_messages.insert((Lang::En, UiMsg::ReplTypeHelpForCommands), "Type :help for available commands");

        // 英語UI - REPLコマンドヘルプ
        ui_messages.insert((Lang::En, UiMsg::ReplCommandHelp), ":help              - Show this help");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandVars), ":vars              - List all defined variables");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandFuncs), ":funcs             - List all defined functions");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandBuiltins), ":builtins [filter] - List all builtin functions (optional: filter by pattern)");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandClear), ":clear             - Clear environment");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandLoad), ":load <file>       - Load a file");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandReload), ":reload            - Reload the last loaded file");
        ui_messages.insert((Lang::En, UiMsg::ReplCommandQuit), ":quit              - Exit REPL");

        // 英語UI - テスト
        ui_messages.insert((Lang::En, UiMsg::TestNoTests), "No tests to run");
        ui_messages.insert((Lang::En, UiMsg::TestResults), "Test Results:");
        ui_messages.insert((Lang::En, UiMsg::TestResultsSeparator), "=============");
        ui_messages.insert((Lang::En, UiMsg::TestSummary), "{0} tests, {1} passed, {2} failed");
        ui_messages.insert((Lang::En, UiMsg::TestAssertEqFailed), "Assertion failed:\n  Expected: {0}\n  Actual:   {1}");
        ui_messages.insert((Lang::En, UiMsg::TestAssertTruthyFailed), "Assertion failed: expected truthy value, got {0}");
        ui_messages.insert((Lang::En, UiMsg::TestAssertFalsyFailed), "Assertion failed: expected falsy value, got {0}");

        // 英語UI - プロファイラー
        ui_messages.insert((Lang::En, UiMsg::ProfileNoData), "No profile data available.");
        ui_messages.insert((Lang::En, UiMsg::ProfileUseStart), "Use (profile/start) to begin profiling.");
        ui_messages.insert((Lang::En, UiMsg::ProfileReport), "=== Profile Report ===");
        ui_messages.insert((Lang::En, UiMsg::ProfileTableHeader), "Function                                     Calls      Total (ms)       Avg (μs)");
        ui_messages.insert((Lang::En, UiMsg::ProfileTotalTime), "Total time: {0} ms");

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

        // 日本語UI - REPL基本
        ui_messages.insert((Lang::Ja, UiMsg::ReplWelcome), "Qi REPL v{0}");
        ui_messages.insert((Lang::Ja, UiMsg::ReplPressCtrlC), "終了するには Ctrl+C を押してください");
        ui_messages.insert((Lang::Ja, UiMsg::ReplGoodbye), "さようなら！");
        ui_messages.insert((Lang::Ja, UiMsg::ReplLoading), "{0} を読み込んでいます...");
        ui_messages.insert((Lang::Ja, UiMsg::ReplLoaded), "読み込み完了");
        ui_messages.insert((Lang::Ja, UiMsg::ReplTypeHelp), "REPLコマンドは :help で確認できます");
        ui_messages.insert((Lang::Ja, UiMsg::ReplAvailableCommands), "利用可能なREPLコマンド:");
        ui_messages.insert((Lang::Ja, UiMsg::ReplNoVariables), "変数は定義されていません");
        ui_messages.insert((Lang::Ja, UiMsg::ReplDefinedVariables), "定義済み変数:");
        ui_messages.insert((Lang::Ja, UiMsg::ReplNoFunctions), "ユーザー定義関数はありません");
        ui_messages.insert((Lang::Ja, UiMsg::ReplDefinedFunctions), "ユーザー定義関数:");
        ui_messages.insert((Lang::Ja, UiMsg::ReplNoBuiltinsMatching), "'{0}' に一致する組み込み関数はありません");
        ui_messages.insert((Lang::Ja, UiMsg::ReplBuiltinsMatching), "'{0}' に一致する組み込み関数:");
        ui_messages.insert((Lang::Ja, UiMsg::ReplBuiltinFunctions), "組み込み関数:");
        ui_messages.insert((Lang::Ja, UiMsg::ReplBuiltinTotal), "合計: {0} 個");
        ui_messages.insert((Lang::Ja, UiMsg::ReplBuiltinTip), "ヒント: ':builtins <パターン>' でフィルタできます (例: ':builtins str')");
        ui_messages.insert((Lang::Ja, UiMsg::ReplEnvCleared), "環境をクリアしました");
        ui_messages.insert((Lang::Ja, UiMsg::ReplLoadUsage), "使い方: :load <file>");
        ui_messages.insert((Lang::Ja, UiMsg::ReplNoFileLoaded), "まだファイルが読み込まれていません");
        ui_messages.insert((Lang::Ja, UiMsg::ReplUnknownCommand), "不明なコマンド: {0}");
        ui_messages.insert((Lang::Ja, UiMsg::ReplTypeHelpForCommands), "利用可能なコマンドは :help で確認してください");

        // 日本語UI - REPLコマンドヘルプ
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandHelp), ":help              - このヘルプを表示");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandVars), ":vars              - 定義済み変数を一覧表示");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandFuncs), ":funcs             - 定義済み関数を一覧表示");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandBuiltins), ":builtins [filter] - 組み込み関数を一覧表示 (オプション: パターンでフィルタ)");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandClear), ":clear             - 環境をクリア");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandLoad), ":load <file>       - ファイルを読み込む");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandReload), ":reload            - 最後に読み込んだファイルを再読み込み");
        ui_messages.insert((Lang::Ja, UiMsg::ReplCommandQuit), ":quit              - REPLを終了");

        // 日本語UI - テスト
        ui_messages.insert((Lang::Ja, UiMsg::TestNoTests), "実行するテストがありません");
        ui_messages.insert((Lang::Ja, UiMsg::TestResults), "テスト結果:");
        ui_messages.insert((Lang::Ja, UiMsg::TestResultsSeparator), "===========");
        ui_messages.insert((Lang::Ja, UiMsg::TestSummary), "{0} テスト, {1} 成功, {2} 失敗");
        ui_messages.insert((Lang::Ja, UiMsg::TestAssertEqFailed), "アサーション失敗:\n  期待値: {0}\n  実際値: {1}");
        ui_messages.insert((Lang::Ja, UiMsg::TestAssertTruthyFailed), "アサーション失敗: 真の値が期待されましたが、実際は {0} でした");
        ui_messages.insert((Lang::Ja, UiMsg::TestAssertFalsyFailed), "アサーション失敗: 偽の値が期待されましたが、実際は {0} でした");

        // 日本語UI - プロファイラー
        ui_messages.insert((Lang::Ja, UiMsg::ProfileNoData), "プロファイルデータがありません。");
        ui_messages.insert((Lang::Ja, UiMsg::ProfileUseStart), "(profile/start) でプロファイリングを開始してください。");
        ui_messages.insert((Lang::Ja, UiMsg::ProfileReport), "=== プロファイルレポート ===");
        ui_messages.insert((Lang::Ja, UiMsg::ProfileTableHeader), "関数                                         呼出回数    合計時間(ms)     平均(μs)");
        ui_messages.insert((Lang::Ja, UiMsg::ProfileTotalTime), "合計時間: {0} ms");

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
