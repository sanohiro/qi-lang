/// 国際化メッセージ管理
///
/// 言語設定の優先順位:
/// 1. QI_LANG 環境変数（Qi専用の設定）
/// 2. LANG 環境変数（システムのロケール設定）
/// 3. デフォルト: en
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Lang {
    En = 0,
    Ja = 1,
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
#[repr(usize)]
pub enum MsgKey {
    // パーサーエラー
    UnexpectedToken,
    UnexpectedEof,
    ExpectedToken,
    NeedsSymbol, // 共通化: Def/Let/Fn等で使用
    VarargNeedsName,
    UnexpectedPattern,
    RestNeedsVar, // ...rest の後に変数名が必要

    // レキサーエラー
    UnexpectedChar,
    UnclosedString,
    NumberLiteralInvalid, // invalid number literal: {0}
    EmptyKeyword,         // empty keyword (: must be followed by identifier)

    // 評価器エラー
    UndefinedVar,
    UndefinedVarWithSuggestions, // 未定義変数（サジェスト付き）
    NotAFunction,
    TypeMismatch, // 型エラー（期待と実際）
    ArgCountMismatch,
    DivisionByZero,
    ExportOnlyInModule,
    CannotQuote, // 統合: CannotQuoteとCannotQuoteSpecialForm
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
    NeedAtLeastNArgs, // {0}には少なくとも{1}個の引数が必要
    NeedExactlyNArgs, // {0}には{1}個の引数が必要
    Need2Or3Args,     // {0}には2または3個の引数が必要
    Need1Or2Args,     // {0}には1または2個の引数が必要
    Need0Or1Args,     // {0}には0または1個の引数が必要
    Need2Args,        // {0}には2つの引数が必要
    Need1Arg,         // {0}には1つの引数が必要
    Need0Args,        // {0}には引数は不要

    // 型エラー（汎用）
    TypeOnly,             // {0}は{1}のみ受け付けます
    TypeOnlyWithDebug,    // {0}は{1}のみ受け付けます: {2:?}
    ArgMustBeType,        // {0}: 引数は{1}である必要があります
    FirstArgMustBe,       // {0}の第1引数は{1}が必要です
    SecondArgMustBe,      // {0}の第2引数は{1}が必要です
    ThirdArgMustBe,       // {0}の第3引数は{1}が必要です
    KeyMustBeKeyword,     // キーは文字列またはキーワードが必要
    KeyNotFound,          // キーが見つかりません: {0}
    MustBePositive,       // {0}: {1}は正の数である必要があります
    MustBeNonNegative,    // {0}: {1}は非負の数である必要があります
    MustBeInteger,        // {0}: {1}は整数である必要があります
    MustBeString,         // {0}: {1}は文字列である必要があります
    MinMustBeLessThanMax, // {0}: min must be less than max
    MustBeListOrVector,   // {0}: {1}はリストまたはベクタである必要があります
    MustBePromise,        // {0}: {1}はプロミス（チャネル）である必要があります
    MustBeScope,          // {0}: {1}はスコープである必要があります
    MustNotBeEmpty,       // {0}: {1}は空であってはいけません
    FuncMustReturnType,   // {0}: 関数は{1}を返す必要があります
    MustBeMap,            // {0}: {1}はマップである必要があります

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
    FStringCodeParseError, // f-string: コードのパースエラー: {0}

    // マクロエラー
    MacVarargNeedsSymbol,
    VariadicMacroNeedsParams,
    MacArgCountMismatch, // mac {0}: 引数の数が一致しません（期待: {1}, 実際: {2}）
    MacVariadicArgCountMismatch, // mac {0}: 引数の数が不足しています（最低: {1}, 実際: {2}）

    // quasiquote エラー
    UnquoteOutsideQuasiquote,
    UnquoteSpliceOutsideQuasiquote,
    UnquoteSpliceNeedsListOrVector,

    // loop/recur エラー
    RecurNotFound,
    RecurArgCountMismatch, // recur: 引数の数が一致しません（期待: {0}, 実際: {1}）

    // 内部変換エラー
    ValueCannotBeConverted,

    // モジュールロード詳細エラー
    CircularDependency,
    ModuleParserInitError,
    ModuleParseError,
    ModuleMustExport,

    // その他の特殊エラー
    AsNeedsVarName,        // :asには変数名が必要です
    NeedNArgsDesc,         // {0}には{1}個の引数が必要です: {2}
    SelectNeedsList,       // {0}にはリストが必要です
    SelectNeedsAtLeastOne, // {0}には少なくとも1つのケースが必要です
    SelectTimeoutCase,     // {0}: :timeoutケースは3要素が必要です: [:timeout ms handler]
    SelectOnlyOneTimeout,  // {0}には:timeoutケースは1つだけです
    SelectChannelCase,     // {0}: チャネルケースは2要素が必要です: [channel handler]
    SelectCaseMustStart,   // {0}: ケースはチャネルまたは:timeoutで始まる必要があります
    SelectCaseMustBe, // {0}: ケースはリストである必要があります [channel handler] or [:timeout ms handler]
    AllElementsMustBe, // {0}: 全ての要素は{1}である必要があります

    // 並行処理エラー
    ChannelClosed,   // {0}: channel is closed
    ExpectedKeyword, // {0}: expected {1} keyword
    PromiseFailed,   // promise failed
    NotAPromise,     // not a promise
    UnexpectedError, // {0}: unexpected error
    RecvArgs,        // {0}: requires 1 or 3 arguments: ({0} ch) or ({0} ch :timeout ms)
    TimeoutMustBeMs, // {0}: timeout must be an integer (milliseconds)

    // その他のエラー
    UnsupportedNumberType,  // unsupported number type
    RailwayRequiresOkError, // |>? requires {:ok/:error} map
    InvalidTimestamp,       // invalid timestamp
    InvalidDateFormat,      // invalid date format
    InvalidPercentile,      // invalid percentile (must be 0-100)
    SystemTimeError,        // {0}: system time error: {1}
    JsonParseError,         // {0}: {1}
    JsonStringifyError2,    // {0}: {1}
    CannotParseAsInt,       // {0}: cannot parse '{1}' as integer
    CannotConvertToInt,     // {0}: cannot convert {1} to integer
    CannotParseAsFloat,     // {0}: cannot parse '{1}' as float
    CannotConvertToFloat,   // {0}: cannot convert {1} to float
    CannotConvertToJson,    // Cannot convert {0} to JSON
    InvalidRegex,           // {0}: invalid regex: {1}

    // 警告
    RedefineBuiltin,  // warning: redefining builtin function: {0} ({1})
    RedefineFunction, // warning: redefining function: {0}
    RedefineVariable, // warning: redefining variable: {0}

    // CSV エラー
    FileReadError,                // {0}: file read error: {1}
    CsvCannotSerialize,           // csv/stringify: cannot serialize {0}
    CsvRecordMustBeList,          // csv/stringify: each record must be a list
    CsvParseNeed1Or3Args,         // csv/parse requires 1 or 3 arguments
    CsvDelimiterMustBeSingleChar, // csv/parse: delimiter must be a single character
    CsvInvalidDelimiterArg,       // csv/parse: invalid delimiter argument (use :delimiter "char")

    // コマンド実行エラー
    CmdEmptyCommand,         // Command cannot be empty
    CmdFirstArgMustBeString, // First element of command list must be a string
    CmdArgsMustBeStrings,    // All command arguments must be strings
    CmdInvalidArgument,      // Invalid command argument: expected string or list
    CmdExecutionFailed,      // Command execution failed: {0}
    CmdWriteFailed,          // Failed to write to command stdin: {0}
    CmdWaitFailed,           // Failed to wait for command: {0}
    CmdInvalidProcessHandle, // Invalid process handle
    CmdProcessNotFound,      // Process not found (PID: {0})
    CmdStdinClosed,          // stdin is already closed
    CmdStdoutClosed,         // stdout is already closed
    CmdReadFailed,           // Failed to read: {0}
    MustBePositiveInteger,   // {0}: {1} must be a positive integer

    // データ構造エラー
    MustBeQueue, // {0}: {1} must be a queue
    MustBeStack, // {0}: {1} must be a stack
    IsEmpty,     // {0}: {1} is empty

    // テストエラー
    TestsFailed,             // Some tests failed
    AssertExpectedException, // Assertion failed: expected exception but none was thrown

    // パスエラー
    AllPathsMustBeStrings, // {0}: all paths must be strings

    // サーバーエラー
    JsonStringifyError, // Failed to stringify JSON
    RequestMustHave,    // {0}: request must have {1}
    RequestMustBe,      // {0}: request must be {1}
    InvalidFilePath,    // {0}: invalid file path (contains ..)
    FourthArgMustBe,    // {0}'s fourth argument must be {1}
    Need3Args,          // {0} requires 3 arguments
    Need1Or3Args,       // {0}: requires 1 or 3 arguments

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
    HttpClientError,           // HTTP client error: {0}
    HttpCompressionError,      // Compression error: {0}
    HttpStreamClientError,     // http stream: client creation error: {0}
    HttpStreamRequestFailed,   // http stream: request failed: {0}
    HttpStreamReadBytesFailed, // http stream: failed to read bytes: {0}
    HttpStreamReadBodyFailed,  // http stream: failed to read body: {0}
    HttpRequestUrlRequired,    // http/request: :url is required
    HttpUnsupportedMethod,     // Unsupported HTTP method: {0}
    HttpStreamError,           // http stream: HTTP {0}

    // I/Oエラー（詳細）
    IoFileError,               // {0}: {1}
    IoFailedToDecodeUtf8,      // {0}: failed to decode as UTF-8 (invalid byte sequence)
    IoFailedToCreateDir,       // {0}: failed to create directory: {1}
    IoFailedToOpenForAppend,   // {0}: failed to open for append: {1}
    IoFailedToAppend,          // {0}: failed to append: {1}
    IoFailedToWrite,           // {0}: failed to write: {1}
    FileStreamFailedToOpen,    // file-stream: failed to open '{0}': {1}
    WriteStreamFailedToCreate, // write-stream: failed to create {0}: {1}
    WriteStreamFailedToWrite,  // write-stream: failed to write to {0}: {1}
    IoListDirInvalidPattern,   // io/list-dir: invalid pattern '{0}': {1}
    IoListDirFailedToRead,     // io/list-dir: failed to read entry: {0}
    IoCreateDirFailed,         // io/create-dir: failed to create '{0}': {1}
    IoDeleteFileFailed,        // io/delete-file: failed to delete '{0}': {1}
    IoDeleteDirFailed,         // io/delete-dir: failed to delete '{0}': {1}
    IoCopyFileFailed,          // io/copy-file: failed to copy '{0}' to '{1}': {2}
    IoMoveFileFailed,          // io/move-file: failed to move '{0}' to '{1}': {2}
    IoGetMetadataFailed,       // io/file-info: failed to get metadata for '{0}': {1}

    // サーバーエラー（詳細）
    ServerFailedToReadBody,         // Failed to read request body: {0}
    ServerFailedToDecompressGzip,   // Failed to decompress gzip body: {0}
    ServerFailedToBuildResponse,    // Failed to build response: {0}
    ServerStaticFileMetadataFailed, // server/static-file: failed to read file metadata: {0}
    ServerHandlerMustReturnMap,     // Handler must return a map, got: {0}
    ServerHandlerMustBeFunction,    // Handler must be a function or router, got: {0}
    ServerHandlerError,             // Handler error: {0}
    ServerFileTooLarge, // File too large: {0} bytes (max: {1} bytes / {2} MB). Path: {3}
    ServerFailedToReadFile, // Failed to read file: {0}
    ServerStaticFileTooLarge, // server/static-file: file too large: {0} bytes (max: {1} bytes / {2} MB). Consider using streaming in the future.
    ServerStaticFileFailedToRead, // server/static-file: failed to read file: {0}
    ServerStaticDirNotDirectory, // server/static-dir: {0} is not a directory

    // SQLiteエラー
    SqliteFailedToOpen,                // Failed to open SQLite database: {0}
    SqliteFailedToSetTimeout,          // Failed to set timeout: {0}
    SqliteFailedToGetColumnName,       // Failed to get column name: {0}
    SqliteFailedToPrepare,             // Failed to prepare statement: {0}
    SqliteFailedToExecuteQuery,        // Failed to execute query: {0}
    SqliteFailedToExecuteStatement,    // Failed to execute statement: {0}
    SqliteFailedToBeginTransaction,    // Failed to begin transaction: {0}
    SqliteFailedToCommitTransaction,   // Failed to commit transaction: {0}
    SqliteFailedToRollbackTransaction, // Failed to rollback transaction: {0}

    // 環境変数エラー（詳細）
    EnvLoadDotenvFailedToRead, // env/load-dotenv: failed to read file '{0}': {1}
    EnvLoadDotenvInvalidFormat, // env/load-dotenv: invalid format at line {0}: '{1}'

    // CSVエラー（詳細）
    CsvWriteFileStringifyFailed, // csv/write-file: stringify failed
    CsvWriteFileFailedToWrite,   // csv/write-file: failed to write '{0}': {1}

    // ログエラー
    LogSetLevelInvalidLevel, // log/set-level: invalid level '{0}' (valid: debug, info, warn, error)
    LogSetFormatInvalidFormat, // log/set-format: invalid format '{0}' (valid: text, json)

    // 時刻エラー（詳細）
    TimeParseFailedToParse, // time/parse: failed to parse '{0}' with format '{1}'

    // ZIPエラー
    ZipPathDoesNotExist, // {0}: path '{1}' does not exist

    // データベースエラー
    DbUnsupportedUrl,      // Unsupported database URL: {0}. Supported: sqlite:
    DbNeed2To4Args,        // {0} requires 2-4 arguments, got {1}
    DbNeed1To3Args,        // {0} requires 1-3 arguments, got {1}
    DbExpectedConnection,  // Expected DbConnection, got: {0}
    DbConnectionNotFound,  // Connection not found: {0}
    DbExpectedTransaction, // Expected DbTransaction, got: {0}
    DbTransactionNotFound, // Transaction not found: {0}
    DbExpectedConnectionOrTransaction, // Expected DbConnection or DbTransaction, got: {0}
    DbExpectedPool,        // Expected DbPool, got: {0}
    DbPoolNotFound,        // Pool not found: {0}
    DbInvalidPoolSize,     // {0}: invalid pool size, expected {1}

    // I/Oエラー（追加）
    IoFailedToDecodeAs, // {0}: failed to decode as {1} (invalid byte sequence)
    IoCouldNotDetectEncoding, // {0}: could not detect encoding (tried UTF-8, UTF-16, Japanese, Chinese, Korean, European encodings)
    IoAppendFileFailedToWrite, // append-file: failed to write {0}: {1}
    IoAppendFileFailedToOpen, // append-file: failed to open {0}: {1}
    IoReadLinesFailedToRead,  // read-lines: failed to read {0}: {1}

    // Featureエラー
    FeatureDisabled,     // {0} support is disabled. Build with feature '{1}': {2}
    DbUnsupportedDriver, // Unsupported database driver: {0}

    // Markdownエラー
    MdHeaderInvalidLevel,  // markdown/header: level must be 1-6, got: {0}
    MdTableEmpty,          // markdown/table: table must not be empty
    MdTableRowMustBeList,  // markdown/table: row {0} must be a list
    MdTableColumnMismatch, // markdown/table: row {0} has {1} columns, expected {2}
}

/// UIメッセージキー
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
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
    OptStdin,
    OptLoad,
    OptHelp,
    OptVersion,

    // ヘルプ例
    ExampleStartRepl,
    ExampleRunScript,
    ExampleExecuteCode,
    ExampleStdin,
    ExampleLoadFile,

    // 環境変数説明
    EnvLangQi,
    EnvLangSystem,

    // バージョン
    VersionString,

    // エラー
    ErrorFailedToRead,
    ErrorFailedToReadStdin,
    ErrorRequiresArg,
    ErrorRequiresFile,
    ErrorUnknownOption,
    ErrorUseHelp,
    ErrorInput,
    ErrorParse,
    ErrorLexer,
    ErrorRuntime,
}
// ========================================
// メッセージ配列定義（高速アクセス用）
// ========================================

// ========================================
// MsgKey 配列定義
// ========================================

/// 英語エラーメッセージ配列
static EN_MSGS: [&str; 230] = [
    "unexpected token: {0}", // UnexpectedToken
    "unexpected end of file", // UnexpectedEof
    "expected {0}, got {1}", // ExpectedToken
    "{0} requires a symbol", // NeedsSymbol
    "'&' requires a variable name", // VarargNeedsName
    "unexpected pattern: {0}", // UnexpectedPattern
    "'...' requires a variable name", // RestNeedsVar
    "unexpected character: {0}", // UnexpectedChar
    "unclosed string", // UnclosedString
    "invalid number literal: {0}", // NumberLiteralInvalid
    "empty keyword: ':' must be followed by an identifier", // EmptyKeyword
    "undefined variable: {0}", // UndefinedVar
    "undefined variable: {0}\n      Did you mean: {1}?", // UndefinedVarWithSuggestions
    "not a function: {0}", // NotAFunction
    "type error: expected {0}, got {1} ({2})", // TypeMismatch
    "argument count mismatch: expected {0}, got {1}", // ArgCountMismatch
    "division by zero", // DivisionByZero
    "export can only be used inside a module definition", // ExportOnlyInModule
    "cannot quote: {0}", // CannotQuote
    "no matching pattern", // NoMatchingPattern
    "module requires a module name", // ModuleNeedsName
    "export requires symbols", // ExportNeedsSymbols
    "use requires a module name", // UseNeedsModuleName
    "expected symbol in :only list", // ExpectedSymbolInOnlyList
    ":as requires an alias name", // AsNeedsAlias
    "use requires :only, :as, or :all", // UseNeedsMode
    "symbol {0} not found (module: {1})", // SymbolNotFound
    "module {0} not found ({0}.qi)", // ModuleNotFound
    "symbol {0} is not exported from module {1}", // SymbolNotExported
    ":as mode is not implemented yet", // UseAsNotImplemented
    "{0} requires at least {1} argument(s)", // NeedAtLeastNArgs
    "{0} requires exactly {1} argument(s)", // NeedExactlyNArgs
    "{0} requires 2 or 3 arguments", // Need2Or3Args
    "{0} requires 1 or 2 arguments", // Need1Or2Args
    "{0} requires 0 or 1 argument", // Need0Or1Args
    "{0} requires 2 arguments", // Need2Args
    "{0} requires 1 argument", // Need1Arg
    "{0} requires no arguments", // Need0Args
    "{0} accepts {1} only", // TypeOnly
    "{0} accepts {1} only: {2}", // TypeOnlyWithDebug
    "{0}: argument must be {1}", // ArgMustBeType
    "{0}'s first argument must be {1}", // FirstArgMustBe
    "{0}'s second argument must be {1}", // SecondArgMustBe
    "{0}'s third argument must be {1}", // ThirdArgMustBe
    "key must be a string or keyword", // KeyMustBeKeyword
    "key not found: {0}", // KeyNotFound
    "{0}: {1} must be positive", // MustBePositive
    "{0}: {1} must be non-negative", // MustBeNonNegative
    "{0}: {1} must be an integer", // MustBeInteger
    "{0}: {1} must be a string", // MustBeString
    "{0}: min must be less than max", // MinMustBeLessThanMax
    "{0}: {1} must be a list or vector", // MustBeListOrVector
    "{0}: {1} must be a promise (channel)", // MustBePromise
    "{0}: {1} must be a scope", // MustBeScope
    "{0}: {1} must not be empty", // MustNotBeEmpty
    "{0}: function must return {1}", // FuncMustReturnType
    "{0}: {1} must be a map", // MustBeMap
    "split requires two strings", // SplitTwoStrings
    "join requires a string and a list", // JoinStringAndList
    "assoc requires a map and one or more key-value pairs", // AssocMapAndKeyValues
    "dissoc requires a map and one or more keys", // DissocMapAndKeys
    "variadic function requires exactly one parameter", // VariadicFnNeedsOneParam
    "f-string: unclosed {", // FStringUnclosedBrace
    "f-string: unclosed string", // FStringUnclosed
    "f-string cannot be quoted", // FStringCannotBeQuoted
    "f-string: code parse error: {0}", // FStringCodeParseError
    "mac: '&' requires a symbol", // MacVarargNeedsSymbol
    "variadic macro requires parameters", // VariadicMacroNeedsParams
    "mac {0}: argument count mismatch (expected {1}, got {2})", // MacArgCountMismatch
    "mac {0}: insufficient arguments (minimum {1}, got {2})", // MacVariadicArgCountMismatch
    "unquote: can only be used inside quasiquote", // UnquoteOutsideQuasiquote
    "unquote-splice: can only be used inside quasiquote", // UnquoteSpliceOutsideQuasiquote
    "unquote-splice: requires a list or vector", // UnquoteSpliceNeedsListOrVector
    "recur not found", // RecurNotFound
    "recur: argument count mismatch (expected {0}, got {1})", // RecurArgCountMismatch
    "value cannot be converted", // ValueCannotBeConverted
    "circular dependency detected: {0}", // CircularDependency
    "module {0} parser initialization error: {1}", // ModuleParserInitError
    "module {0} parse error: {1}", // ModuleParseError
    "module {0} must contain export", // ModuleMustExport
    ":as requires a variable name", // AsNeedsVarName
    "{0} requires {1} argument(s): {2}", // NeedNArgsDesc
    "{0} requires a list", // SelectNeedsList
    "{0} requires at least one case", // SelectNeedsAtLeastOne
    "{0}: :timeout case must have 3 elements: [:timeout ms handler]", // SelectTimeoutCase
    "{0} can only have one :timeout case", // SelectOnlyOneTimeout
    "{0}: channel case must have 2 elements: [channel handler]", // SelectChannelCase
    "{0}: case must start with a channel or :timeout", // SelectCaseMustStart
    "{0}: case must be a list [channel handler] or [:timeout ms handler]", // SelectCaseMustBe
    "{0}: all elements must be {1}", // AllElementsMustBe
    "{0}: channel is closed", // ChannelClosed
    "{0}: expected {1} keyword", // ExpectedKeyword
    "promise failed", // PromiseFailed
    "not a promise", // NotAPromise
    "{0}: unexpected error", // UnexpectedError
    "{0}: requires 1 or 3 arguments: ({0} ch) or ({0} ch :timeout ms)", // RecvArgs
    "{0}: timeout must be an integer (milliseconds)", // TimeoutMustBeMs
    "unsupported number type", // UnsupportedNumberType
    "|>? requires {:ok/:error} map", // RailwayRequiresOkError
    "{0}: invalid timestamp", // InvalidTimestamp
    "{0}: invalid date format: {1}", // InvalidDateFormat
    "{0}: percentile must be between 0 and 100", // InvalidPercentile
    "{0}: system time error: {1}", // SystemTimeError
    "{0}: {1}", // JsonParseError
    "{0}: {1}", // JsonStringifyError2
    "{0}: cannot parse '{1}' as integer", // CannotParseAsInt
    "{0}: cannot convert {1} to integer", // CannotConvertToInt
    "{0}: cannot parse '{1}' as float", // CannotParseAsFloat
    "{0}: cannot convert {1} to float", // CannotConvertToFloat
    "Cannot convert {0} to JSON", // CannotConvertToJson
    "{0}: invalid regex: {1}", // InvalidRegex
    "warning: redefining builtin function '{0}' ({1})", // RedefineBuiltin
    "warning: redefining function '{0}'", // RedefineFunction
    "warning: redefining variable '{0}'", // RedefineVariable
    "{0}: file read error: {1}", // FileReadError
    "csv/stringify: cannot serialize {0}", // CsvCannotSerialize
    "csv/stringify: each record must be a list", // CsvRecordMustBeList
    "csv/parse requires 1 or 3 arguments", // CsvParseNeed1Or3Args
    "csv/parse: delimiter must be a single character", // CsvDelimiterMustBeSingleChar
    "csv/parse: invalid delimiter argument (use :delimiter \"char\")", // CsvInvalidDelimiterArg
    "Command cannot be empty", // CmdEmptyCommand
    "First element of command list must be a string", // CmdFirstArgMustBeString
    "All command arguments must be strings", // CmdArgsMustBeStrings
    "Invalid command argument: expected string or list", // CmdInvalidArgument
    "Command execution failed: {0}", // CmdExecutionFailed
    "Failed to write to command stdin: {0}", // CmdWriteFailed
    "Failed to wait for command: {0}", // CmdWaitFailed
    "Invalid process handle", // CmdInvalidProcessHandle
    "Process not found (PID: {0})", // CmdProcessNotFound
    "stdin is already closed", // CmdStdinClosed
    "stdout is already closed", // CmdStdoutClosed
    "Failed to read: {0}", // CmdReadFailed
    "{0}: {1} must be a positive integer", // MustBePositiveInteger
    "{0}: {1} must be a queue", // MustBeQueue
    "{0}: {1} must be a stack", // MustBeStack
    "{0}: {1} is empty", // IsEmpty
    "Some tests failed", // TestsFailed
    "Assertion failed: expected exception but none was thrown", // AssertExpectedException
    "{0}: all paths must be strings", // AllPathsMustBeStrings
    "Failed to stringify JSON", // JsonStringifyError
    "{0}: request must have {1}", // RequestMustHave
    "{0}: request must be {1}", // RequestMustBe
    "{0}: invalid file path (contains ..)", // InvalidFilePath
    "{0}'s fourth argument must be {1}", // FourthArgMustBe
    "{0} requires 3 arguments", // Need3Args
    "{0}: requires 1 or 3 arguments", // Need1Or3Args
    "{0}: value must be a string, number, or boolean", // ValueMustBeStringNumberBool
    "{0}: both arguments must be strings", // BothArgsMustBeStrings
    "Unsupported encoding: {0}", // UnsupportedEncoding
    "Keyword :{0} requires a value", // KeywordRequiresValue
    "Expected keyword argument, got {0}", // ExpectedKeywordArg
    "{0}: file already exists", // FileAlreadyExists
    "Invalid :if-exists option: {0}", // InvalidIfExistsOption
    "HTTP client error: {0}", // HttpClientError
    "Compression error: {0}", // HttpCompressionError
    "http stream: client creation error: {0}", // HttpStreamClientError
    "http stream: request failed: {0}", // HttpStreamRequestFailed
    "http stream: failed to read bytes: {0}", // HttpStreamReadBytesFailed
    "http stream: failed to read body: {0}", // HttpStreamReadBodyFailed
    "http/request: :url is required", // HttpRequestUrlRequired
    "Unsupported HTTP method: {0}", // HttpUnsupportedMethod
    "http stream: HTTP {0}", // HttpStreamError
    "{0}: {1}", // IoFileError
    "{0}: failed to decode as UTF-8 (invalid byte sequence)", // IoFailedToDecodeUtf8
    "{0}: failed to create directory: {1}", // IoFailedToCreateDir
    "{0}: failed to open for append: {1}", // IoFailedToOpenForAppend
    "{0}: failed to append: {1}", // IoFailedToAppend
    "{0}: failed to write: {1}", // IoFailedToWrite
    "file-stream: failed to open '{0}': {1}", // FileStreamFailedToOpen
    "write-stream: failed to create {0}: {1}", // WriteStreamFailedToCreate
    "write-stream: failed to write to {0}: {1}", // WriteStreamFailedToWrite
    "io/list-dir: invalid pattern '{0}': {1}", // IoListDirInvalidPattern
    "io/list-dir: failed to read entry: {0}", // IoListDirFailedToRead
    "io/create-dir: failed to create '{0}': {1}", // IoCreateDirFailed
    "io/delete-file: failed to delete '{0}': {1}", // IoDeleteFileFailed
    "io/delete-dir: failed to delete '{0}': {1}", // IoDeleteDirFailed
    "io/copy-file: failed to copy '{0}' to '{1}': {2}", // IoCopyFileFailed
    "io/move-file: failed to move '{0}' to '{1}': {2}", // IoMoveFileFailed
    "io/file-info: failed to get metadata for '{0}': {1}", // IoGetMetadataFailed
    "Failed to read request body: {0}", // ServerFailedToReadBody
    "Failed to decompress gzip body: {0}", // ServerFailedToDecompressGzip
    "Failed to build response: {0}", // ServerFailedToBuildResponse
    "server/static-file: failed to read file metadata: {0}", // ServerStaticFileMetadataFailed
    "Handler must return a map, got: {0}", // ServerHandlerMustReturnMap
    "Handler must be a function or router, got: {0}", // ServerHandlerMustBeFunction
    "Handler error: {0}", // ServerHandlerError
    "File too large: {0} bytes (max: {1} bytes / {2} MB). Path: {3}", // ServerFileTooLarge
    "Failed to read file: {0}", // ServerFailedToReadFile
    "server/static-file: file too large: {0} bytes (max: {1} bytes / {2} MB). Consider using streaming in the future.", // ServerStaticFileTooLarge
    "server/static-file: failed to read file: {0}", // ServerStaticFileFailedToRead
    "server/static-dir: {0} is not a directory", // ServerStaticDirNotDirectory
    "Failed to open SQLite database: {0}", // SqliteFailedToOpen
    "Failed to set timeout: {0}", // SqliteFailedToSetTimeout
    "Failed to get column name: {0}", // SqliteFailedToGetColumnName
    "Failed to prepare statement: {0}", // SqliteFailedToPrepare
    "Failed to execute query: {0}", // SqliteFailedToExecuteQuery
    "Failed to execute statement: {0}", // SqliteFailedToExecuteStatement
    "Failed to begin transaction: {0}", // SqliteFailedToBeginTransaction
    "Failed to commit transaction: {0}", // SqliteFailedToCommitTransaction
    "Failed to rollback transaction: {0}", // SqliteFailedToRollbackTransaction
    "env/load-dotenv: failed to read file '{0}': {1}", // EnvLoadDotenvFailedToRead
    "env/load-dotenv: invalid format at line {0}: '{1}'", // EnvLoadDotenvInvalidFormat
    "csv/write-file: stringify failed", // CsvWriteFileStringifyFailed
    "csv/write-file: failed to write '{0}': {1}", // CsvWriteFileFailedToWrite
    "log/set-level: invalid level '{0}' (valid: debug, info, warn, error)", // LogSetLevelInvalidLevel
    "log/set-format: invalid format '{0}' (valid: text, json)", // LogSetFormatInvalidFormat
    "time/parse: failed to parse '{0}' with format '{1}'", // TimeParseFailedToParse
    "{0}: path '{1}' does not exist", // ZipPathDoesNotExist
    "Unsupported database URL: {0}. Supported: sqlite:", // DbUnsupportedUrl
    "{0} requires 2-4 arguments, got {1}", // DbNeed2To4Args
    "{0} requires 1-3 arguments, got {1}", // DbNeed1To3Args
    "Expected DbConnection, got: {0}", // DbExpectedConnection
    "Connection not found: {0}", // DbConnectionNotFound
    "Expected DbTransaction, got: {0}", // DbExpectedTransaction
    "Transaction not found: {0}", // DbTransactionNotFound
    "Expected DbConnection or DbTransaction, got: {0}", // DbExpectedConnectionOrTransaction
    "Expected DbPool, got: {0}", // DbExpectedPool
    "Pool not found: {0}", // DbPoolNotFound
    "{0}: invalid pool size, expected {1}", // DbInvalidPoolSize
    "{0}: failed to decode as {1} (invalid byte sequence)", // IoFailedToDecodeAs
    "{0}: could not detect encoding (tried UTF-8, UTF-16, Japanese, Chinese, Korean, European encodings)", // IoCouldNotDetectEncoding
    "append-file: failed to write {0}: {1}", // IoAppendFileFailedToWrite
    "append-file: failed to open {0}: {1}", // IoAppendFileFailedToOpen
    "read-lines: failed to read {0}: {1}", // IoReadLinesFailedToRead
    "{0} support is disabled. Build with feature '{1}': {2}", // FeatureDisabled
    "Unsupported database driver: {0}", // DbUnsupportedDriver
    "markdown/header: level must be 1-6, got: {0}", // MdHeaderInvalidLevel
    "markdown/table: table must not be empty", // MdTableEmpty
    "markdown/table: row {0} must be a list", // MdTableRowMustBeList
    "markdown/table: row {0} has {1} columns, expected {2}", // MdTableColumnMismatch
];

/// 日本語エラーメッセージ配列
static JA_MSGS: [&str; 230] = [
    "予期しないトークン: {0}", // UnexpectedToken
    "予期しないEOF", // UnexpectedEof
    "期待: {0}, 実際: {1}", // ExpectedToken
    "{0}にはシンボルが必要です", // NeedsSymbol
    "&の後には変数名が必要です", // VarargNeedsName
    "予期しないパターン: {0}", // UnexpectedPattern
    "...の後には変数名が必要です", // RestNeedsVar
    "予期しない文字: {0}", // UnexpectedChar
    "文字列が閉じられていません", // UnclosedString
    "不正な数値リテラル: {0}", // NumberLiteralInvalid
    "空のキーワード: ':' の後には識別子が必要です", // EmptyKeyword
    "未定義の変数: {0}", // UndefinedVar
    "未定義の変数: {0}\n      もしかして: {1}?", // UndefinedVarWithSuggestions
    "関数ではありません: {0}", // NotAFunction
    "型エラー: 期待={0}, 実際={1} ({2})", // TypeMismatch
    "引数の数が一致しません: 期待 {0}, 実際 {1}", // ArgCountMismatch
    "ゼロ除算エラー", // DivisionByZero
    "exportはmodule定義の中でのみ使用できます", // ExportOnlyInModule
    "quoteできません: {0}", // CannotQuote
    "どのパターンにもマッチしませんでした", // NoMatchingPattern
    "moduleにはモジュール名が必要です", // ModuleNeedsName
    "exportにはシンボルが必要です", // ExportNeedsSymbols
    "useにはモジュール名が必要です", // UseNeedsModuleName
    ":onlyリストにはシンボルが必要です", // ExpectedSymbolInOnlyList
    ":asにはエイリアス名が必要です", // AsNeedsAlias
    "useには:onlyまたは:asが必要です", // UseNeedsMode
    "シンボル{0}が見つかりません（モジュール: {1}）", // SymbolNotFound
    "モジュール{0}が見つかりません（{0}.qi）", // ModuleNotFound
    "シンボル{0}はモジュール{1}からエクスポートされていません", // SymbolNotExported
    ":asモードはまだ実装されていません", // UseAsNotImplemented
    "{0}には少なくとも{1}個の引数が必要です", // NeedAtLeastNArgs
    "{0}には{1}個の引数が必要です", // NeedExactlyNArgs
    "{0}には2または3個の引数が必要です", // Need2Or3Args
    "{0}には1または2個の引数が必要です", // Need1Or2Args
    "{0}には0または1個の引数が必要です", // Need0Or1Args
    "{0}には2つの引数が必要です", // Need2Args
    "{0}には1つの引数が必要です", // Need1Arg
    "{0}には引数は不要です", // Need0Args
    "{0}は{1}のみ受け付けます", // TypeOnly
    "{0}は{1}のみ受け付けます: {2}", // TypeOnlyWithDebug
    "{0}: 引数は{1}である必要があります", // ArgMustBeType
    "{0}の第1引数は{1}が必要です", // FirstArgMustBe
    "{0}の第2引数は{1}が必要です", // SecondArgMustBe
    "{0}の第3引数は{1}が必要です", // ThirdArgMustBe
    "キーは文字列またはキーワードが必要です", // KeyMustBeKeyword
    "キーが見つかりません: {0}", // KeyNotFound
    "{0}: {1}は正の数である必要があります", // MustBePositive
    "{0}: {1}は非負の数である必要があります", // MustBeNonNegative
    "{0}: {1}は整数である必要があります", // MustBeInteger
    "{0}: {1}は文字列である必要があります", // MustBeString
    "{0}: minはmaxより小さい必要があります", // MinMustBeLessThanMax
    "{0}: {1}はリストまたはベクタである必要があります", // MustBeListOrVector
    "{0}: {1}はプロミス（チャネル）である必要があります", // MustBePromise
    "{0}: {1}はスコープである必要があります", // MustBeScope
    "{0}: {1}は空であってはいけません", // MustNotBeEmpty
    "{0}: 関数は{1}を返す必要があります", // FuncMustReturnType
    "{0}: {1}はマップである必要があります", // MustBeMap
    "splitは2つの文字列が必要です", // SplitTwoStrings
    "joinは文字列とリストが必要です", // JoinStringAndList
    "assocはマップと1つ以上のキー・値のペアが必要です", // AssocMapAndKeyValues
    "dissocはマップと1つ以上のキーが必要です", // DissocMapAndKeys
    "可変長引数関数にはパラメータが1つだけ必要です", // VariadicFnNeedsOneParam
    "f-string: 閉じられていない { があります", // FStringUnclosedBrace
    "f-string: 閉じられていない文字列です", // FStringUnclosed
    "f-string はquoteできません", // FStringCannotBeQuoted
    "f-string: コードのパースエラー: {0}", // FStringCodeParseError
    "mac: &の後にシンボルが必要です", // MacVarargNeedsSymbol
    "可変長マクロはパラメータが必要です", // VariadicMacroNeedsParams
    "mac {0}: 引数の数が一致しません（期待: {1}, 実際: {2}）", // MacArgCountMismatch
    "mac {0}: 引数の数が不足しています（最低: {1}, 実際: {2}）", // MacVariadicArgCountMismatch
    "unquote: quasiquote外では使用できません", // UnquoteOutsideQuasiquote
    "unquote-splice: quasiquote外では使用できません", // UnquoteSpliceOutsideQuasiquote
    "unquote-splice: リストまたはベクタが必要です", // UnquoteSpliceNeedsListOrVector
    "recurが見つかりません", // RecurNotFound
    "recur: 引数の数が一致しません（期待: {0}, 実際: {1}）", // RecurArgCountMismatch
    "この値は変換できません", // ValueCannotBeConverted
    "循環参照を検出しました: {0}", // CircularDependency
    "モジュール{0}のパーサー初期化エラー: {1}", // ModuleParserInitError
    "モジュール{0}のパースエラー: {1}", // ModuleParseError
    "モジュール{0}はexportを含む必要があります", // ModuleMustExport
    ":asには変数名が必要です", // AsNeedsVarName
    "{0}には{1}個の引数が必要です: {2}", // NeedNArgsDesc
    "{0}にはリストが必要です", // SelectNeedsList
    "{0}には少なくとも1つのケースが必要です", // SelectNeedsAtLeastOne
    "{0}: :timeoutケースは3要素が必要です: [:timeout ms handler]", // SelectTimeoutCase
    "{0}には:timeoutケースは1つだけです", // SelectOnlyOneTimeout
    "{0}: チャネルケースは2要素が必要です: [channel handler]", // SelectChannelCase
    "{0}: ケースはチャネルまたは:timeoutで始まる必要があります", // SelectCaseMustStart
    "{0}: ケースはリストである必要があります [channel handler] or [:timeout ms handler]", // SelectCaseMustBe
    "{0}: 全ての要素は{1}である必要があります", // AllElementsMustBe
    "{0}: チャネルがクローズされています", // ChannelClosed
    "{0}: {1}キーワードが必要です", // ExpectedKeyword
    "プロミスが失敗しました", // PromiseFailed
    "プロミスではありません", // NotAPromise
    "{0}: 予期しないエラー", // UnexpectedError
    "{0}: 1または3個の引数が必要です: ({0} ch) または ({0} ch :timeout ms)", // RecvArgs
    "{0}: タイムアウトは整数（ミリ秒）である必要があります", // TimeoutMustBeMs
    "サポートされていない数値型です", // UnsupportedNumberType
    "|>? には {:ok/:error} マップが必要です", // RailwayRequiresOkError
    "{0}: 不正なタイムスタンプです", // InvalidTimestamp
    "{0}: 不正な日付フォーマットです: {1}", // InvalidDateFormat
    "{0}: パーセンタイルは0から100の間である必要があります", // InvalidPercentile
    "{0}: システム時刻エラー: {1}", // SystemTimeError
    "{0}: {1}", // JsonParseError
    "{0}: {1}", // JsonStringifyError2
    "{0}: '{1}' を整数としてパースできません", // CannotParseAsInt
    "{0}: {1} を整数に変換できません", // CannotConvertToInt
    "{0}: '{1}' を浮動小数点数としてパースできません", // CannotParseAsFloat
    "{0}: {1} を浮動小数点数に変換できません", // CannotConvertToFloat
    "{0} をJSONに変換できません", // CannotConvertToJson
    "{0}: 不正な正規表現: {1}", // InvalidRegex
    "警告: ビルトイン関数を再定義しています: '{0}' ({1})", // RedefineBuiltin
    "警告: 関数を再定義しています: '{0}'", // RedefineFunction
    "警告: 変数を再定義しています: '{0}'", // RedefineVariable
    "{0}: ファイル読み込みエラー: {1}", // FileReadError
    "csv/stringify: シリアライズできません: {0}", // CsvCannotSerialize
    "csv/stringify: 各レコードはリストである必要があります", // CsvRecordMustBeList
    "csv/parse: 引数は1個または3個必要です", // CsvParseNeed1Or3Args
    "csv/parse: デリミタは1文字である必要があります", // CsvDelimiterMustBeSingleChar
    "csv/parse: デリミタ引数が不正です (:delimiter \"文字\" を使用)", // CsvInvalidDelimiterArg
    "コマンドが空です", // CmdEmptyCommand
    "コマンドリストの最初の要素は文字列である必要があります", // CmdFirstArgMustBeString
    "全てのコマンド引数は文字列である必要があります", // CmdArgsMustBeStrings
    "無効なコマンド引数: 文字列またはリストが必要です", // CmdInvalidArgument
    "コマンド実行に失敗しました: {0}", // CmdExecutionFailed
    "コマンドの標準入力への書き込みに失敗しました: {0}", // CmdWriteFailed
    "コマンドの待機に失敗しました: {0}", // CmdWaitFailed
    "無効なプロセスハンドルです", // CmdInvalidProcessHandle
    "プロセスが見つかりません (PID: {0})", // CmdProcessNotFound
    "標準入力は既に閉じられています", // CmdStdinClosed
    "標準出力は既に閉じられています", // CmdStdoutClosed
    "読み取りに失敗しました: {0}", // CmdReadFailed
    "{0}: {1}は正の整数である必要があります", // MustBePositiveInteger
    "{0}: {1}はキューである必要があります", // MustBeQueue
    "{0}: {1}はスタックである必要があります", // MustBeStack
    "{0}: {1}は空です", // IsEmpty
    "いくつかのテストが失敗しました", // TestsFailed
    "アサーション失敗: 例外が期待されましたが発生しませんでした", // AssertExpectedException
    "{0}: すべてのパスは文字列である必要があります", // AllPathsMustBeStrings
    "JSON文字列化に失敗しました", // JsonStringifyError
    "{0}: リクエストには{1}が必要です", // RequestMustHave
    "{0}: リクエストは{1}である必要があります", // RequestMustBe
    "{0}: 不正なファイルパスです (.. が含まれています)", // InvalidFilePath
    "{0}の第4引数は{1}が必要です", // FourthArgMustBe
    "{0}には3つの引数が必要です", // Need3Args
    "{0}: 1または3個の引数が必要です", // Need1Or3Args
    "{0}: 値は文字列、数値、または真偽値である必要があります", // ValueMustBeStringNumberBool
    "{0}: 両方の引数は文字列である必要があります", // BothArgsMustBeStrings
    "サポートされていないエンコーディングです: {0}", // UnsupportedEncoding
    "キーワード :{0} には値が必要です", // KeywordRequiresValue
    "キーワード引数が必要です。実際: {0}", // ExpectedKeywordArg
    "{0}: ファイルが既に存在します", // FileAlreadyExists
    "不正な :if-exists オプション: {0}", // InvalidIfExistsOption
    "HTTPクライアントエラー: {0}", // HttpClientError
    "圧縮エラー: {0}", // HttpCompressionError
    "http stream: クライアント作成エラー: {0}", // HttpStreamClientError
    "http stream: リクエスト失敗: {0}", // HttpStreamRequestFailed
    "http stream: バイト読み込み失敗: {0}", // HttpStreamReadBytesFailed
    "http stream: ボディ読み込み失敗: {0}", // HttpStreamReadBodyFailed
    "http/request: :url は必須です", // HttpRequestUrlRequired
    "未サポートのHTTPメソッド: {0}", // HttpUnsupportedMethod
    "http stream: HTTP {0}", // HttpStreamError
    "{0}: {1}", // IoFileError
    "{0}: UTF-8としてデコードできませんでした (無効なバイト列)", // IoFailedToDecodeUtf8
    "{0}: ディレクトリの作成に失敗しました: {1}", // IoFailedToCreateDir
    "{0}: 追記用のオープンに失敗しました: {1}", // IoFailedToOpenForAppend
    "{0}: 追記に失敗しました: {1}", // IoFailedToAppend
    "{0}: 書き込みに失敗しました: {1}", // IoFailedToWrite
    "file-stream: '{0}' のオープンに失敗しました: {1}", // FileStreamFailedToOpen
    "write-stream: {0} の作成に失敗しました: {1}", // WriteStreamFailedToCreate
    "write-stream: {0} への書き込みに失敗しました: {1}", // WriteStreamFailedToWrite
    "io/list-dir: 不正なパターン '{0}': {1}", // IoListDirInvalidPattern
    "io/list-dir: エントリの読み込みに失敗しました: {0}", // IoListDirFailedToRead
    "io/create-dir: '{0}' の作成に失敗しました: {1}", // IoCreateDirFailed
    "io/delete-file: '{0}' の削除に失敗しました: {1}", // IoDeleteFileFailed
    "io/delete-dir: '{0}' の削除に失敗しました: {1}", // IoDeleteDirFailed
    "io/copy-file: '{0}' から '{1}' へのコピーに失敗しました: {2}", // IoCopyFileFailed
    "io/move-file: '{0}' から '{1}' への移動に失敗しました: {2}", // IoMoveFileFailed
    "io/file-info: '{0}' のメタデータ取得に失敗しました: {1}", // IoGetMetadataFailed
    "リクエストボディの読み込みに失敗しました: {0}", // ServerFailedToReadBody
    "gzipボディの解凍に失敗しました: {0}", // ServerFailedToDecompressGzip
    "レスポンスの構築に失敗しました: {0}", // ServerFailedToBuildResponse
    "server/static-file: ファイルメタデータの読み込みに失敗しました: {0}", // ServerStaticFileMetadataFailed
    "ハンドラーはマップを返す必要があります。実際: {0}", // ServerHandlerMustReturnMap
    "ハンドラーは関数またはルーターである必要があります。実際: {0}", // ServerHandlerMustBeFunction
    "ハンドラーエラー: {0}", // ServerHandlerError
    "ファイルが大きすぎます: {0} バイト (最大: {1} バイト / {2} MB)。パス: {3}", // ServerFileTooLarge
    "ファイルの読み込みに失敗しました: {0}", // ServerFailedToReadFile
    "server/static-file: ファイルが大きすぎます: {0} バイト (最大: {1} バイト / {2} MB)。今後ストリーミングの使用を検討してください。", // ServerStaticFileTooLarge
    "server/static-file: ファイルの読み込みに失敗しました: {0}", // ServerStaticFileFailedToRead
    "server/static-dir: {0} はディレクトリではありません", // ServerStaticDirNotDirectory
    "SQLiteデータベースのオープンに失敗しました: {0}", // SqliteFailedToOpen
    "タイムアウトの設定に失敗しました: {0}", // SqliteFailedToSetTimeout
    "カラム名の取得に失敗しました: {0}", // SqliteFailedToGetColumnName
    "ステートメントの準備に失敗しました: {0}", // SqliteFailedToPrepare
    "クエリの実行に失敗しました: {0}", // SqliteFailedToExecuteQuery
    "ステートメントの実行に失敗しました: {0}", // SqliteFailedToExecuteStatement
    "トランザクションの開始に失敗しました: {0}", // SqliteFailedToBeginTransaction
    "トランザクションのコミットに失敗しました: {0}", // SqliteFailedToCommitTransaction
    "トランザクションのロールバックに失敗しました: {0}", // SqliteFailedToRollbackTransaction
    "env/load-dotenv: ファイル '{0}' の読み込みに失敗しました: {1}", // EnvLoadDotenvFailedToRead
    "env/load-dotenv: {0}行目のフォーマットが不正です: '{1}'", // EnvLoadDotenvInvalidFormat
    "csv/write-file: 文字列化に失敗しました", // CsvWriteFileStringifyFailed
    "csv/write-file: '{0}' への書き込みに失敗しました: {1}", // CsvWriteFileFailedToWrite
    "log/set-level: 不正なレベル '{0}' です (有効な値: debug, info, warn, error)", // LogSetLevelInvalidLevel
    "log/set-format: 不正なフォーマット '{0}' です (有効な値: text, json)", // LogSetFormatInvalidFormat
    "time/parse: '{0}' をフォーマット '{1}' でパースできませんでした", // TimeParseFailedToParse
    "{0}: パス '{1}' が存在しません", // ZipPathDoesNotExist
    "サポートされていないデータベースURL: {0} (対応: sqlite:)", // DbUnsupportedUrl
    "{0}には2〜4個の引数が必要です。実際: {1}個", // DbNeed2To4Args
    "{0}は1～3個の引数が必要です。実際: {1}個", // DbNeed1To3Args
    "DbConnectionが期待されましたが、実際: {0}", // DbExpectedConnection
    "接続が見つかりません: {0}", // DbConnectionNotFound
    "DbTransactionが期待されましたが、実際: {0}", // DbExpectedTransaction
    "トランザクションが見つかりません: {0}", // DbTransactionNotFound
    "DbConnectionまたはDbTransactionが期待されましたが、実際: {0}", // DbExpectedConnectionOrTransaction
    "DbPoolが期待されましたが、実際: {0}", // DbExpectedPool
    "プールが見つかりません: {0}", // DbPoolNotFound
    "{0}: 無効なプールサイズです。期待される値: {1}", // DbInvalidPoolSize
    "{0}: {1}としてデコードできませんでした (無効なバイト列)", // IoFailedToDecodeAs
    "{0}: エンコーディングを検出できませんでした (UTF-8, UTF-16, 日本語, 中国語, 韓国語, ヨーロッパ言語のエンコーディングを試行)", // IoCouldNotDetectEncoding
    "append-file: {0} への書き込みに失敗しました: {1}", // IoAppendFileFailedToWrite
    "append-file: {0} のオープンに失敗しました: {1}", // IoAppendFileFailedToOpen
    "read-lines: {0} の読み込みに失敗しました: {1}", // IoReadLinesFailedToRead
    "{0} サポートは無効化されています。feature '{1}' を有効にしてビルドしてください: {2}", // FeatureDisabled
    "サポートされていないデータベースドライバー: {0}", // DbUnsupportedDriver
    "markdown/header: レベルは1-6である必要があります、実際: {0}", // MdHeaderInvalidLevel
    "markdown/table: テーブルは空であってはいけません", // MdTableEmpty
    "markdown/table: 行{0}はリストである必要があります", // MdTableRowMustBeList
    "markdown/table: 行{0}は{1}列ありますが、{2}列が期待されています", // MdTableColumnMismatch
];

// ========================================
// UiMsg 配列定義
// ========================================

/// 英語UIメッセージ配列
static EN_UI_MSGS: [&str; 69] = [
    "Qi REPL v{0}",                                                     // ReplWelcome
    "Press Ctrl+C to exit",                                             // ReplPressCtrlC
    "Goodbye!",                                                         // ReplGoodbye
    "Loading {0}...",                                                   // ReplLoading
    "Loaded.",                                                          // ReplLoaded
    "Type :help for REPL commands",                                     // ReplTypeHelp
    "Available REPL commands:",                                         // ReplAvailableCommands
    "No variables defined",                                             // ReplNoVariables
    "Defined variables:",                                               // ReplDefinedVariables
    "No user-defined functions",                                        // ReplNoFunctions
    "User-defined functions:",                                          // ReplDefinedFunctions
    "No builtin functions matching '{0}'",                              // ReplNoBuiltinsMatching
    "Builtin functions matching '{0}':",                                // ReplBuiltinsMatching
    "Builtin functions:",                                               // ReplBuiltinFunctions
    "Total: {0} functions",                                             // ReplBuiltinTotal
    "Tip: Use ':builtins <pattern>' to filter (e.g., ':builtins str')", // ReplBuiltinTip
    "Environment cleared",                                              // ReplEnvCleared
    "Usage: :load <file>",                                              // ReplLoadUsage
    "No file has been loaded yet",                                      // ReplNoFileLoaded
    "Unknown command: {0}",                                             // ReplUnknownCommand
    "Type :help for available commands",                                // ReplTypeHelpForCommands
    ":help              - Show this help",                              // ReplCommandHelp
    ":vars              - List all defined variables",                  // ReplCommandVars
    ":funcs             - List all defined functions",                  // ReplCommandFuncs
    ":builtins [filter] - List all builtin functions (optional: filter by pattern)", // ReplCommandBuiltins
    ":clear             - Clear environment", // ReplCommandClear
    ":load <file>       - Load a file",       // ReplCommandLoad
    ":reload            - Reload the last loaded file", // ReplCommandReload
    ":quit              - Exit REPL",         // ReplCommandQuit
    "No tests to run",                        // TestNoTests
    "Test Results:",                          // TestResults
    "=============",                          // TestResultsSeparator
    "{0} tests, {1} passed, {2} failed",      // TestSummary
    "Assertion failed:\n  Expected: {0}\n  Actual:   {1}", // TestAssertEqFailed
    "Assertion failed: expected truthy value, got {0}", // TestAssertTruthyFailed
    "Assertion failed: expected falsy value, got {0}", // TestAssertFalsyFailed
    "No profile data available.",             // ProfileNoData
    "Use (profile/start) to begin profiling.", // ProfileUseStart
    "=== Profile Report ===",                 // ProfileReport
    "Function                                     Calls      Total (ms)       Avg (μs)", // ProfileTableHeader
    "Total time: {0} ms",                 // ProfileTotalTime
    "Qi - A Lisp that flows",             // HelpTitle
    "USAGE:",                             // HelpUsage
    "OPTIONS:",                           // HelpOptions
    "EXAMPLES:",                          // HelpExamples
    "ENVIRONMENT VARIABLES:",             // HelpEnvVars
    "Execute code string and exit",       // OptExecute
    "Read and execute script from stdin", // OptStdin
    "Load file and start REPL",           // OptLoad
    "Print help information",             // OptHelp
    "Print version information",          // OptVersion
    "Start REPL",                         // ExampleStartRepl
    "Run script file",                    // ExampleRunScript
    "Execute code and print result",      // ExampleExecuteCode
    "Read from stdin",                    // ExampleStdin
    "Load file and start REPL",           // ExampleLoadFile
    "Set language (ja, en)",              // EnvLangQi
    "System locale (auto-detected)",      // EnvLangSystem
    "Qi version {0}",                     // VersionString
    "Failed to read file",                // ErrorFailedToRead
    "Failed to read from stdin",          // ErrorFailedToReadStdin
    "{0} requires an argument",           // ErrorRequiresArg
    "{0} requires a file path",           // ErrorRequiresFile
    "Unknown option: {0}",                // ErrorUnknownOption
    "Use --help for usage information",   // ErrorUseHelp
    "Input error",                        // ErrorInput
    "Parse error",                        // ErrorParse
    "Lexer error",                        // ErrorLexer
    "Error",                              // ErrorRuntime
];

/// 日本語UIメッセージ配列
static JA_UI_MSGS: [&str; 69] = [
    "Qi REPL v{0}",                             // ReplWelcome
    "終了するには Ctrl+C を押してください",     // ReplPressCtrlC
    "さようなら！",                             // ReplGoodbye
    "{0} を読み込んでいます...",                // ReplLoading
    "読み込み完了",                             // ReplLoaded
    "REPLコマンドは :help で確認できます",      // ReplTypeHelp
    "利用可能なREPLコマンド:",                  // ReplAvailableCommands
    "変数は定義されていません",                 // ReplNoVariables
    "定義済み変数:",                            // ReplDefinedVariables
    "ユーザー定義関数はありません",             // ReplNoFunctions
    "ユーザー定義関数:",                        // ReplDefinedFunctions
    "'{0}' に一致する組み込み関数はありません", // ReplNoBuiltinsMatching
    "'{0}' に一致する組み込み関数:",            // ReplBuiltinsMatching
    "組み込み関数:",                            // ReplBuiltinFunctions
    "合計: {0} 個",                             // ReplBuiltinTotal
    "ヒント: ':builtins <パターン>' でフィルタできます (例: ':builtins str')", // ReplBuiltinTip
    "環境をクリアしました",                     // ReplEnvCleared
    "使い方: :load <file>",                     // ReplLoadUsage
    "まだファイルが読み込まれていません",       // ReplNoFileLoaded
    "不明なコマンド: {0}",                      // ReplUnknownCommand
    "利用可能なコマンドは :help で確認してください", // ReplTypeHelpForCommands
    ":help              - このヘルプを表示",    // ReplCommandHelp
    ":vars              - 定義済み変数を一覧表示", // ReplCommandVars
    ":funcs             - 定義済み関数を一覧表示", // ReplCommandFuncs
    ":builtins [filter] - 組み込み関数を一覧表示 (オプション: パターンでフィルタ)", // ReplCommandBuiltins
    ":clear             - 環境をクリア",       // ReplCommandClear
    ":load <file>       - ファイルを読み込む", // ReplCommandLoad
    ":reload            - 最後に読み込んだファイルを再読み込み", // ReplCommandReload
    ":quit              - REPLを終了",         // ReplCommandQuit
    "実行するテストがありません",              // TestNoTests
    "テスト結果:",                             // TestResults
    "===========",                             // TestResultsSeparator
    "{0} テスト, {1} 成功, {2} 失敗",          // TestSummary
    "アサーション失敗:\n  期待値: {0}\n  実際値: {1}", // TestAssertEqFailed
    "アサーション失敗: 真の値が期待されましたが、実際は {0} でした", // TestAssertTruthyFailed
    "アサーション失敗: 偽の値が期待されましたが、実際は {0} でした", // TestAssertFalsyFailed
    "プロファイルデータがありません。",        // ProfileNoData
    "(profile/start) でプロファイリングを開始してください。", // ProfileUseStart
    "=== プロファイルレポート ===",            // ProfileReport
    "関数                                         呼出回数    合計時間(ms)     平均(μs)", // ProfileTableHeader
    "合計時間: {0} ms",                       // ProfileTotalTime
    "Qi - 流れるLisp",                        // HelpTitle
    "使い方:",                                // HelpUsage
    "オプション:",                            // HelpOptions
    "例:",                                    // HelpExamples
    "環境変数:",                              // HelpEnvVars
    "コード文字列を実行して終了",             // OptExecute
    "標準入力からスクリプトを読み込んで実行", // OptStdin
    "ファイルを読み込んでREPLを起動",         // OptLoad
    "ヘルプ情報を表示",                       // OptHelp
    "バージョン情報を表示",                   // OptVersion
    "REPLを起動",                             // ExampleStartRepl
    "スクリプトファイルを実行",               // ExampleRunScript
    "コードを実行して結果を表示",             // ExampleExecuteCode
    "標準入力から読み込み",                   // ExampleStdin
    "ファイルを読み込んでREPLを起動",         // ExampleLoadFile
    "言語を設定 (ja, en)",                    // EnvLangQi
    "システムロケール (自動検出)",            // EnvLangSystem
    "Qi バージョン {0}",                      // VersionString
    "ファイルの読み込みに失敗しました",       // ErrorFailedToRead
    "標準入力の読み込みに失敗しました",       // ErrorFailedToReadStdin
    "{0} には引数が必要です",                 // ErrorRequiresArg
    "{0} にはファイルパスが必要です",         // ErrorRequiresFile
    "不明なオプション: {0}",                  // ErrorUnknownOption
    "使い方は --help で確認してください",     // ErrorUseHelp
    "入力エラー",                             // ErrorInput
    "パースエラー",                           // ErrorParse
    "レキサーエラー",                         // ErrorLexer
    "エラー",                                 // ErrorRuntime
];

/// メッセージマネージャー（配列ベース、高速アクセス）
pub struct Messages {
    lang: Lang,
}

impl Messages {
    /// 言語設定でMessagesインスタンスを作成
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }

    /// メッセージを取得
    pub fn get(&self, key: MsgKey) -> &'static str {
        match self.lang {
            Lang::En => EN_MSGS[key as usize],
            Lang::Ja => JA_MSGS[key as usize],
        }
    }

    /// UIメッセージを取得
    pub fn ui(&self, key: UiMsg) -> &'static str {
        match self.lang {
            Lang::En => EN_UI_MSGS[key as usize],
            Lang::Ja => JA_UI_MSGS[key as usize],
        }
    }

    /// メッセージをフォーマット（プレースホルダー {0}, {1}, ... を置換）
    pub fn fmt(&self, key: MsgKey, args: &[&str]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();

        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, arg);
        }

        result
    }

    /// UIメッセージをフォーマット
    pub fn fmt_ui(&self, key: UiMsg, args: &[&str]) -> String {
        let template = self.ui(key);
        let mut result = template.to_string();

        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            result = result.replace(&placeholder, arg);
        }

        result
    }
}

// ========================================
// グローバルインスタンス
// ========================================

static MESSAGES: OnceLock<Messages> = OnceLock::new();

/// i18nシステムを初期化
pub fn init() {
    // OnceLockで自動初期化されるため、ここでは何もしない
    // ただし、初期化を強制したい場合は messages() を呼ぶ
    let _ = messages();
}

/// グローバルなメッセージインスタンスを取得
pub fn messages() -> &'static Messages {
    MESSAGES.get_or_init(|| Messages::new(Lang::from_env()))
}

/// メッセージを取得してフォーマット
pub fn fmt_msg(key: MsgKey, args: &[&str]) -> String {
    messages().fmt(key, args)
}

/// UIメッセージを取得してフォーマット
pub fn fmt_ui_msg(key: UiMsg, args: &[&str]) -> String {
    messages().fmt_ui(key, args)
}

/// メッセージを取得
pub fn msg(key: MsgKey) -> &'static str {
    messages().get(key)
}

/// UIメッセージを取得
pub fn ui_msg(key: UiMsg) -> &'static str {
    messages().ui(key)
}
