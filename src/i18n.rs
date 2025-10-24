/// 国際化メッセージ管理
///
/// 言語設定の優先順位:
/// 1. QI_LANG 環境変数（Qi専用の設定）
/// 2. LANG 環境変数（システムのロケール設定）
/// 3. デフォルト: en
use std::collections::HashMap;
use std::sync::LazyLock;

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
    FloatKeyNotAllowed,   // Floatはマップのキーとして使用できません
    InvalidMapKey,        // 無効なマップキー型: {0}
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

    // JWT エラー
    NeedNArgs,        // {0} requires {1} arguments
    InvalidAlgorithm, // {0}: invalid algorithm: {1} (supported: {2})
    InvalidFloat,     // {0}: invalid float value
    InvalidNumber,    // {0}: invalid number value

    // パスワードハッシュエラー
    PasswordHashError, // {0}: password hash error: {1}

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

    // データベース汎用エラー（PostgreSQL/MySQL/SQLite共通）
    DbFailedToConnect,             // Failed to connect to database: {0}
    DbFailedToExecuteQuery,        // Failed to execute query: {0}
    DbFailedToExecuteStatement,    // Failed to execute statement: {0}
    DbFailedToBeginTransaction,    // Failed to begin transaction: {0}
    DbFailedToCommitTransaction,   // Failed to commit transaction: {0}
    DbFailedToRollbackTransaction, // Failed to rollback transaction: {0}
    DbUnsupportedUrl,              // Unsupported database URL: {0}

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
    DbNeed2To4Args,                    // {0} requires 2-4 arguments, got {1}
    DbNeed1To3Args,                    // {0} requires 1-3 arguments, got {1}
    DbExpectedConnection,              // Expected DbConnection, got: {0}
    DbConnectionNotFound,              // Connection not found: {0}
    DbExpectedTransaction,             // Expected DbTransaction, got: {0}
    DbTransactionNotFound,             // Transaction not found: {0}
    DbExpectedConnectionOrTransaction, // Expected DbConnection or DbTransaction, got: {0}
    DbExpectedPool,                    // Expected DbPool, got: {0}
    DbPoolNotFound,                    // Pool not found: {0}
    DbInvalidPoolSize,                 // {0}: invalid pool size, expected {1}

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

    // DAPデバッガーエラー
    DapEmptyExpression,       // Empty expression
    DapEvaluationError,       // Evaluation error: {0}
    DapParseError,            // Parse error: {0}
    DapNoEnvironment,         // No environment available (not stopped at breakpoint)
    DapDebuggerNotAvailable,  // Debugger not available
    DapServerError,           // DAP server error: {0}
    DapServerNotEnabled,      // Error: DAP server is not enabled. Build with --features dap-server
    InternalError,            // Internal error: {0}

    // プロジェクト管理エラー
    QiTomlFailedToRead,          // Failed to read qi.toml: {0}
    QiTomlFailedToParse,         // Failed to parse qi.toml: {0}
    QiTomlFailedToSerialize,     // Failed to serialize qi.toml: {0}
    QiTomlFailedToWrite,         // Failed to write qi.toml: {0}
    FailedToGetCurrentDir,       // Failed to get current directory: {0}
    DirectoryAlreadyExists,      // Directory '{}' already exists
    FailedToCreateDirectory,     // Failed to create directory: {0}
    FailedToCreateSrcDir,        // Failed to create src/ directory: {0}
    FailedToCreateExamplesDir,   // Failed to create examples/ directory: {0}
    FailedToCreateTestsDir,      // Failed to create tests/ directory: {0}
    FailedToCreateMainQi,        // Failed to create main.qi: {0}
    FailedToCreateLibQi,         // Failed to create src/lib.qi: {0}
    FailedToCreateExampleQi,     // Failed to create examples/example.qi: {0}
    FailedToCreateTestQi,        // Failed to create tests/test.qi: {0}
    TemplateNotFound,            // Template '{}' not found
    FailedToReadDirectory,       // Failed to read directory: {0}
    FailedToReadFile,            // Failed to read file: {0}
    FailedToWriteFile,           // Failed to write file: {0}
    TemplateTomlFailedToRead,    // Failed to read template.toml: {0}
    TemplateTomlFailedToParse,   // Failed to parse template.toml: {0}

    // 評価器エラー
    TypeErrorVectorPattern,      // Type error: cannot pass {0} to vector pattern
    ArgErrorVectorPatternMinimum, // Argument error: vector pattern expects at least {0} elements, but got {1}
    ArgErrorVectorPattern,       // Argument error: vector pattern expects {0} elements, but got {1}
    TypeErrorMapPattern,         // Type error: cannot pass {0} to map pattern
    KeyErrorMapMissing,          // Key error: map does not have key :{0}

    // データベース・KVSエラー
    ConnectionError,             // Connection error: {0}
    ConnectionNotFound,          // Connection not found: {0}
    EvalError,                   // eval: {0}
    FailedToCreateRuntime,       // Failed to create runtime: {0}
    FailedToExecuteColumnsQuery, // Failed to execute columns query: {0}
    FailedToExecuteForeignKeysQuery, // Failed to execute foreign keys query: {0}
    FailedToExecuteIndexColumnsQuery, // Failed to execute index columns query: {0}
    FailedToExecuteIndexesQuery, // Failed to execute indexes query: {0}
    FailedToExecuteTablesQuery,  // Failed to execute tables query: {0}
    FailedToGetColumnName,       // Failed to get column name: {0}
    FailedToGetColumnValue,      // Failed to get column value: {0}
    FailedToGetDatabaseVersion,  // Failed to get database version: {0}
    FailedToPrepareStatement,    // Failed to prepare statement: {0}
    FailedToQueryColumns,        // Failed to query columns: {0}
    FailedToQueryForeignKeys,    // Failed to query foreign keys: {0}
    FailedToQueryIndexColumns,   // Failed to query index columns: {0}
    FailedToQueryIndexes,        // Failed to query indexes: {0}
    FailedToQueryTables,         // Failed to query tables: {0}
    FailedToReadFileMetadata,    // Failed to read file metadata: {0}
    InvalidIsolationLevel,       // Invalid isolation level: {0}
    UnsupportedKvsUrl,           // Unsupported KVS URL: {0}
    UnsupportedUrl,              // Unsupported URL: {0}
    RedisSupportNotEnabled,      // Redis support not enabled (feature 'kvs-redis' required)
    InvalidConnection,           // Invalid connection (expected KvsConnection:xxx)
    QiTomlAlreadyExists,         // qi.toml already exists
    PatternErrorNotAllowed,      // Pattern error: this pattern cannot be used in function parameters or let bindings (only in match)
    UnexpectedResponse,          // Unexpected response
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
    ReplCommandDoc,
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
    OptQuiet,
    OptHelp,
    OptVersion,
    OptNew,
    OptTemplate,
    OptDap,

    // ヘルプ例
    ExampleStartRepl,
    ExampleRunScript,
    ExampleExecuteCode,
    ExampleStdin,
    ExampleLoadFile,
    ExampleNewProject,
    ExampleNewHttpServer,
    ExampleTemplateList,

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

    // DAP
    DapStdinWaiting,      // ⏸️  標準入力を待っています
    DapStdinInstructions, // デバッグコンソールで .stdin <text> と入力してください
    DapStdinSent,         // ✓ 入力を送信しました: {0}

    // プロジェクト作成
    ProjectNewNeedName,         // エラー: プロジェクト名を指定してください
    ProjectNewUsage,            // 使い方: qi new <project-name> [--template <template>]
    ProjectNewUnknownOption,    // エラー: 不明なオプション: {0}
    ProjectNewError,            // エラー: {0}
    TemplateNeedSubcommand,     // エラー: サブコマンドを指定してください
    TemplateUsage,              // 使い方: qi template <list|info>
    TemplateNeedName,           // エラー: テンプレート名を指定してください
    TemplateInfoUsage,          // 使い方: qi template info <name>
    TemplateUnknownSubcommand,  // エラー: 不明なサブコマンド: {0}

    // REPL
    ReplDocUsage,    // Usage: :doc <name>
    ReplDocNotFound, // No such function or variable: {0}
    ReplDocParameters, // Parameters:
    ReplDocExamples,   // Examples:
    ReplDocNoDoc,      // (no documentation available)
}

// ========================================
// メッセージマップ定義（可読性重視）
// ========================================

/// 英語エラーメッセージ
static EN_MSGS: LazyLock<HashMap<MsgKey, &'static str>> = LazyLock::new(|| {
    use MsgKey::*;
    HashMap::from([
        // パーサーエラー
        (UnexpectedToken, "unexpected token: {0}"),
        (UnexpectedEof, "unexpected end of file (parenthesis, bracket, or brace may not be closed)"),
        (ExpectedToken, "expected {0}, got {1}"),
        (NeedsSymbol, "{0} requires a symbol"),
        (VarargNeedsName, "'&' requires a variable name"),
        (UnexpectedPattern, "unexpected pattern: {0}"),
        (RestNeedsVar, "'...' requires a variable name"),
        // レキサーエラー
        (UnexpectedChar, "unexpected character: {0}"),
        (UnclosedString, "unclosed string"),
        (NumberLiteralInvalid, "invalid number literal: {0}"),
        (
            EmptyKeyword,
            "empty keyword: ':' must be followed by an identifier",
        ),
        // 評価器エラー
        (UndefinedVar, "undefined variable: {0}"),
        (
            UndefinedVarWithSuggestions,
            "undefined variable: {0}\n      Did you mean: {1}?",
        ),
        (NotAFunction, "not a function: {0}"),
        (TypeMismatch, "type error: expected {0}, got {1} ({2})"),
        (ArgCountMismatch, "argument count mismatch: expected {0}, got {1}"),
        (DivisionByZero, "division by zero"),
        (
            ExportOnlyInModule,
            "export can only be used inside a module definition",
        ),
        (CannotQuote, "cannot quote: {0}"),
        (NoMatchingPattern, "no matching pattern"),
        // モジュールエラー
        (ModuleNeedsName, "module requires a module name"),
        (ExportNeedsSymbols, "export requires symbols"),
        (UseNeedsModuleName, "use requires a module name"),
        (ExpectedSymbolInOnlyList, "expected symbol in :only list"),
        (AsNeedsAlias, ":as requires an alias name"),
        (UseNeedsMode, "use requires :only, :as, or :all"),
        (SymbolNotFound, "symbol {0} not found (module: {1})"),
        (ModuleNotFound, "module {0} not found ({0}.qi)"),
        (
            SymbolNotExported,
            "symbol {0} is not exported from module {1}",
        ),
        (UseAsNotImplemented, ":as mode is not implemented yet"),
        // 引数エラー
        (NeedAtLeastNArgs, "{0} requires at least {1} argument(s)"),
        (NeedExactlyNArgs, "{0} requires exactly {1} argument(s)"),
        (Need2Or3Args, "{0} requires 2 or 3 arguments"),
        (Need1Or2Args, "{0} requires 1 or 2 arguments"),
        (Need0Or1Args, "{0} requires 0 or 1 argument"),
        (Need2Args, "{0} requires 2 arguments"),
        (Need1Arg, "{0} requires 1 argument"),
        (Need0Args, "{0} requires no arguments"),
        // 型エラー
        (TypeOnly, "{0} accepts {1} only"),
        (TypeOnlyWithDebug, "{0} accepts {1} only: {2}"),
        (ArgMustBeType, "{0}: argument must be {1}"),
        (FirstArgMustBe, "{0}'s first argument must be {1}"),
        (SecondArgMustBe, "{0}'s second argument must be {1}"),
        (ThirdArgMustBe, "{0}'s third argument must be {1}"),
        (KeyMustBeKeyword, "key must be a string or keyword"),
        (FloatKeyNotAllowed, "float cannot be used as a map key"),
        (InvalidMapKey, "invalid map key type: {0}"),
        (KeyNotFound, "key not found: {0}"),
        (MustBePositive, "{0}: {1} must be positive"),
        (MustBeNonNegative, "{0}: {1} must be non-negative"),
        (MustBeInteger, "{0}: {1} must be an integer"),
        (MustBeString, "{0}: {1} must be a string"),
        (MinMustBeLessThanMax, "{0}: min must be less than max"),
        (MustBeListOrVector, "{0}: {1} must be a list or vector"),
        (MustBePromise, "{0}: {1} must be a promise (channel)"),
        (MustBeScope, "{0}: {1} must be a scope"),
        (MustNotBeEmpty, "{0}: {1} must not be empty"),
        (FuncMustReturnType, "{0}: function must return {1}"),
        (MustBeMap, "{0}: {1} must be a map"),
        // 特殊な引数エラー
        (SplitTwoStrings, "split requires two strings"),
        (JoinStringAndList, "join requires a string and a list"),
        (
            AssocMapAndKeyValues,
            "assoc requires a map and one or more key-value pairs",
        ),
        (
            DissocMapAndKeys,
            "dissoc requires a map and one or more keys",
        ),
        (
            VariadicFnNeedsOneParam,
            "variadic function requires exactly one parameter",
        ),
        // f-string エラー
        (FStringUnclosedBrace, "f-string: unclosed {"),
        (FStringUnclosed, "f-string: unclosed string"),
        (FStringCannotBeQuoted, "f-string cannot be quoted"),
        (FStringCodeParseError, "f-string: code parse error: {0}"),
        // マクロエラー
        (MacVarargNeedsSymbol, "mac: '&' requires a symbol"),
        (VariadicMacroNeedsParams, "variadic macro requires parameters"),
        (
            MacArgCountMismatch,
            "mac {0}: argument count mismatch (expected {1}, got {2})",
        ),
        (
            MacVariadicArgCountMismatch,
            "mac {0}: insufficient arguments (minimum {1}, got {2})",
        ),
        // quasiquote エラー
        (
            UnquoteOutsideQuasiquote,
            "unquote: can only be used inside quasiquote",
        ),
        (
            UnquoteSpliceOutsideQuasiquote,
            "unquote-splice: can only be used inside quasiquote",
        ),
        (
            UnquoteSpliceNeedsListOrVector,
            "unquote-splice: requires a list or vector",
        ),
        // loop/recur エラー
        (RecurNotFound, "recur not found"),
        (
            RecurArgCountMismatch,
            "recur: argument count mismatch (expected {0}, got {1})",
        ),
        // 内部変換エラー
        (ValueCannotBeConverted, "value cannot be converted"),
        // モジュールロード詳細エラー
        (CircularDependency, "circular dependency detected: {0}"),
        (
            ModuleParserInitError,
            "module {0} parser initialization error: {1}",
        ),
        (ModuleParseError, "module {0} parse error: {1}"),
        (ModuleMustExport, "module {0} must contain export"),
        // その他の特殊エラー
        (AsNeedsVarName, ":as requires a variable name"),
        (NeedNArgsDesc, "{0} requires {1} argument(s): {2}"),
        (SelectNeedsList, "{0} requires a list"),
        (SelectNeedsAtLeastOne, "{0} requires at least one case"),
        (
            SelectTimeoutCase,
            "{0}: :timeout case must have 3 elements: [:timeout ms handler]",
        ),
        (SelectOnlyOneTimeout, "{0} can only have one :timeout case"),
        (
            SelectChannelCase,
            "{0}: channel case must have 2 elements: [channel handler]",
        ),
        (
            SelectCaseMustStart,
            "{0}: case must start with a channel or :timeout",
        ),
        (
            SelectCaseMustBe,
            "{0}: case must be a list [channel handler] or [:timeout ms handler]",
        ),
        (AllElementsMustBe, "{0}: all elements must be {1}"),
        // 並行処理エラー
        (ChannelClosed, "{0}: channel is closed"),
        (ExpectedKeyword, "{0}: expected {1} keyword"),
        (PromiseFailed, "promise failed"),
        (NotAPromise, "not a promise"),
        (UnexpectedError, "{0}: unexpected error"),
        (
            RecvArgs,
            "{0}: requires 1 or 3 arguments: ({0} ch) or ({0} ch :timeout ms)",
        ),
        (
            TimeoutMustBeMs,
            "{0}: timeout must be an integer (milliseconds)",
        ),
        // その他のエラー
        (UnsupportedNumberType, "unsupported number type"),
        (
            RailwayRequiresOkError,
            "|>? requires {:ok/:error} map",
        ),
        (InvalidTimestamp, "{0}: invalid timestamp"),
        (InvalidDateFormat, "{0}: invalid date format: {1}"),
        (
            InvalidPercentile,
            "{0}: percentile must be between 0 and 100",
        ),
        (SystemTimeError, "{0}: system time error: {1}"),
        (JsonParseError, "{0}: {1}"),
        (JsonStringifyError2, "{0}: {1}"),
        (CannotParseAsInt, "{0}: cannot parse '{1}' as integer"),
        (CannotConvertToInt, "{0}: cannot convert {1} to integer"),
        (CannotParseAsFloat, "{0}: cannot parse '{1}' as float"),
        (CannotConvertToFloat, "{0}: cannot convert {1} to float"),
        (CannotConvertToJson, "Cannot convert {0} to JSON"),
        (InvalidRegex, "{0}: invalid regex: {1}"),
        // JWT エラー
        (NeedNArgs, "{0} requires {1} arguments"),
        (InvalidAlgorithm, "{0}: invalid algorithm '{1}' (supported: {2})"),
        (InvalidFloat, "{0}: invalid float value"),
        (InvalidNumber, "{0}: invalid number value"),
        // パスワードハッシュエラー
        (PasswordHashError, "{0}: password hash error: {1}"),
        // 警告
        (
            RedefineBuiltin,
            "warning: redefining builtin function '{0}' ({1})",
        ),
        (RedefineFunction, "warning: redefining function '{0}'"),
        (RedefineVariable, "warning: redefining variable '{0}'"),
        // CSV エラー
        (FileReadError, "{0}: file read error: {1}"),
        (CsvCannotSerialize, "csv/stringify: cannot serialize {0}"),
        (
            CsvRecordMustBeList,
            "csv/stringify: each record must be a list",
        ),
        (CsvParseNeed1Or3Args, "csv/parse requires 1 or 3 arguments"),
        (
            CsvDelimiterMustBeSingleChar,
            "csv/parse: delimiter must be a single character",
        ),
        (
            CsvInvalidDelimiterArg,
            "csv/parse: invalid delimiter argument (use :delimiter \"char\")",
        ),
        // コマンド実行エラー
        (CmdEmptyCommand, "Command cannot be empty"),
        (
            CmdFirstArgMustBeString,
            "First element of command list must be a string",
        ),
        (
            CmdArgsMustBeStrings,
            "All command arguments must be strings",
        ),
        (
            CmdInvalidArgument,
            "Invalid command argument: expected string or list",
        ),
        (CmdExecutionFailed, "Command execution failed: {0}"),
        (CmdWriteFailed, "Failed to write to command stdin: {0}"),
        (CmdWaitFailed, "Failed to wait for command: {0}"),
        (CmdInvalidProcessHandle, "Invalid process handle"),
        (CmdProcessNotFound, "Process not found (PID: {0})"),
        (CmdStdinClosed, "stdin is already closed"),
        (CmdStdoutClosed, "stdout is already closed"),
        (CmdReadFailed, "Failed to read: {0}"),
        (
            MustBePositiveInteger,
            "{0}: {1} must be a positive integer",
        ),
        // データ構造エラー
        (MustBeQueue, "{0}: {1} must be a queue"),
        (MustBeStack, "{0}: {1} must be a stack"),
        (IsEmpty, "{0}: {1} is empty"),
        // テストエラー
        (TestsFailed, "Some tests failed"),
        (
            AssertExpectedException,
            "Assertion failed: expected exception but none was thrown",
        ),
        // パスエラー
        (AllPathsMustBeStrings, "{0}: all paths must be strings"),
        // サーバーエラー
        (JsonStringifyError, "Failed to stringify JSON"),
        (RequestMustHave, "{0}: request must have {1}"),
        (RequestMustBe, "{0}: request must be {1}"),
        (
            InvalidFilePath,
            "{0}: invalid file path (contains ..)",
        ),
        (FourthArgMustBe, "{0}'s fourth argument must be {1}"),
        (Need3Args, "{0} requires 3 arguments"),
        (Need1Or3Args, "{0}: requires 1 or 3 arguments"),
        // 環境変数エラー
        (
            ValueMustBeStringNumberBool,
            "{0}: value must be a string, number, or boolean",
        ),
        // I/Oエラー
        (
            BothArgsMustBeStrings,
            "{0}: both arguments must be strings",
        ),
        (UnsupportedEncoding, "Unsupported encoding: {0}"),
        (KeywordRequiresValue, "Keyword :{0} requires a value"),
        (ExpectedKeywordArg, "Expected keyword argument, got {0}"),
        (FileAlreadyExists, "{0}: file already exists"),
        (InvalidIfExistsOption, "Invalid :if-exists option: {0}"),
        // HTTPエラー
        (HttpClientError, "HTTP client error: {0}"),
        (HttpCompressionError, "Compression error: {0}"),
        (
            HttpStreamClientError,
            "http stream: client creation error: {0}",
        ),
        (HttpStreamRequestFailed, "http stream: request failed: {0}"),
        (
            HttpStreamReadBytesFailed,
            "http stream: failed to read bytes: {0}",
        ),
        (
            HttpStreamReadBodyFailed,
            "http stream: failed to read body: {0}",
        ),
        (HttpRequestUrlRequired, "http/request: :url is required"),
        (HttpUnsupportedMethod, "Unsupported HTTP method: {0}"),
        (HttpStreamError, "http stream: HTTP {0}"),
        // I/Oエラー（詳細）
        (IoFileError, "{0}: {1}"),
        (
            IoFailedToDecodeUtf8,
            "{0}: failed to decode as UTF-8 (invalid byte sequence)",
        ),
        (
            IoFailedToCreateDir,
            "{0}: failed to create directory: {1}",
        ),
        (
            IoFailedToOpenForAppend,
            "{0}: failed to open for append: {1}",
        ),
        (IoFailedToAppend, "{0}: failed to append: {1}"),
        (IoFailedToWrite, "{0}: failed to write: {1}"),
        (
            FileStreamFailedToOpen,
            "file-stream: failed to open '{0}': {1}",
        ),
        (
            WriteStreamFailedToCreate,
            "write-stream: failed to create {0}: {1}",
        ),
        (
            WriteStreamFailedToWrite,
            "write-stream: failed to write to {0}: {1}",
        ),
        (
            IoListDirInvalidPattern,
            "io/list-dir: invalid pattern '{0}': {1}",
        ),
        (
            IoListDirFailedToRead,
            "io/list-dir: failed to read entry: {0}",
        ),
        (
            IoCreateDirFailed,
            "io/create-dir: failed to create '{0}': {1}",
        ),
        (
            IoDeleteFileFailed,
            "io/delete-file: failed to delete '{0}': {1}",
        ),
        (
            IoDeleteDirFailed,
            "io/delete-dir: failed to delete '{0}': {1}",
        ),
        (
            IoCopyFileFailed,
            "io/copy-file: failed to copy '{0}' to '{1}': {2}",
        ),
        (
            IoMoveFileFailed,
            "io/move-file: failed to move '{0}' to '{1}': {2}",
        ),
        (
            IoGetMetadataFailed,
            "io/file-info: failed to get metadata for '{0}': {1}",
        ),
        // サーバーエラー（詳細）
        (
            ServerFailedToReadBody,
            "Failed to read request body: {0}",
        ),
        (
            ServerFailedToDecompressGzip,
            "Failed to decompress gzip body: {0}",
        ),
        (
            ServerFailedToBuildResponse,
            "Failed to build response: {0}",
        ),
        (
            ServerStaticFileMetadataFailed,
            "server/static-file: failed to read file metadata: {0}",
        ),
        (
            ServerHandlerMustReturnMap,
            "Handler must return a map, got: {0}",
        ),
        (
            ServerHandlerMustBeFunction,
            "Handler must be a function or router, got: {0}",
        ),
        (ServerHandlerError, "Handler error: {0}"),
        (
            ServerFileTooLarge,
            "File too large: {0} bytes (max: {1} bytes / {2} MB). Path: {3}",
        ),
        (ServerFailedToReadFile, "Failed to read file: {0}"),
        (
            ServerStaticFileTooLarge,
            "server/static-file: file too large: {0} bytes (max: {1} bytes / {2} MB). Consider using streaming in the future.",
        ),
        (
            ServerStaticFileFailedToRead,
            "server/static-file: failed to read file: {0}",
        ),
        (
            ServerStaticDirNotDirectory,
            "server/static-dir: {0} is not a directory",
        ),
        // データベース汎用エラー（PostgreSQL/MySQL/SQLite共通）
        (DbFailedToConnect, "Failed to connect to database: {0}"),
        (DbFailedToExecuteQuery, "Failed to execute query: {0}"),
        (DbFailedToExecuteStatement, "Failed to execute statement: {0}"),
        (DbFailedToBeginTransaction, "Failed to begin transaction: {0}"),
        (DbFailedToCommitTransaction, "Failed to commit transaction: {0}"),
        (DbFailedToRollbackTransaction, "Failed to rollback transaction: {0}"),
        (DbUnsupportedUrl, "Unsupported database URL: {0}"),
        // SQLiteエラー
        (
            SqliteFailedToOpen,
            "Failed to open SQLite database: {0}",
        ),
        (SqliteFailedToSetTimeout, "Failed to set timeout: {0}"),
        (
            SqliteFailedToGetColumnName,
            "Failed to get column name: {0}",
        ),
        (SqliteFailedToPrepare, "Failed to prepare statement: {0}"),
        (
            SqliteFailedToExecuteQuery,
            "Failed to execute query: {0}",
        ),
        (
            SqliteFailedToExecuteStatement,
            "Failed to execute statement: {0}",
        ),
        (
            SqliteFailedToBeginTransaction,
            "Failed to begin transaction: {0}",
        ),
        (
            SqliteFailedToCommitTransaction,
            "Failed to commit transaction: {0}",
        ),
        (
            SqliteFailedToRollbackTransaction,
            "Failed to rollback transaction: {0}",
        ),
        // 環境変数エラー（詳細）
        (
            EnvLoadDotenvFailedToRead,
            "env/load-dotenv: failed to read file '{0}': {1}",
        ),
        (
            EnvLoadDotenvInvalidFormat,
            "env/load-dotenv: invalid format at line {0}: '{1}'",
        ),
        // CSVエラー（詳細）
        (
            CsvWriteFileStringifyFailed,
            "csv/write-file: stringify failed",
        ),
        (
            CsvWriteFileFailedToWrite,
            "csv/write-file: failed to write '{0}': {1}",
        ),
        // ログエラー
        (
            LogSetLevelInvalidLevel,
            "log/set-level: invalid level '{0}' (valid: debug, info, warn, error)",
        ),
        (
            LogSetFormatInvalidFormat,
            "log/set-format: invalid format '{0}' (valid: text, json)",
        ),
        // 時刻エラー（詳細）
        (
            TimeParseFailedToParse,
            "time/parse: failed to parse '{0}' with format '{1}'",
        ),
        // ZIPエラー
        (ZipPathDoesNotExist, "{0}: path '{1}' does not exist"),
        // データベースエラー
        (
            DbUnsupportedUrl,
            "Unsupported database URL: {0}. Supported: sqlite:",
        ),
        (DbNeed2To4Args, "{0} requires 2-4 arguments, got {1}"),
        (DbNeed1To3Args, "{0} requires 1-3 arguments, got {1}"),
        (DbExpectedConnection, "Expected DbConnection, got: {0}"),
        (DbConnectionNotFound, "Connection not found: {0}"),
        (DbExpectedTransaction, "Expected DbTransaction, got: {0}"),
        (DbTransactionNotFound, "Transaction not found: {0}"),
        (
            DbExpectedConnectionOrTransaction,
            "Expected DbConnection or DbTransaction, got: {0}",
        ),
        (DbExpectedPool, "Expected DbPool, got: {0}"),
        (DbPoolNotFound, "Pool not found: {0}"),
        (DbInvalidPoolSize, "{0}: invalid pool size, expected {1}"),
        // I/Oエラー（追加）
        (
            IoFailedToDecodeAs,
            "{0}: failed to decode as {1} (invalid byte sequence)",
        ),
        (IoCouldNotDetectEncoding, "{0}: could not detect encoding (tried UTF-8, UTF-16, Japanese, Chinese, Korean, European encodings)"),
        (
            IoAppendFileFailedToWrite,
            "append-file: failed to write {0}: {1}",
        ),
        (
            IoAppendFileFailedToOpen,
            "append-file: failed to open {0}: {1}",
        ),
        (
            IoReadLinesFailedToRead,
            "read-lines: failed to read {0}: {1}",
        ),
        // Featureエラー
        (
            FeatureDisabled,
            "{0} support is disabled. Build with feature '{1}': {2}",
        ),
        (DbUnsupportedDriver, "Unsupported database driver: {0}"),
        // Markdownエラー
        (
            MdHeaderInvalidLevel,
            "markdown/header: level must be 1-6, got: {0}",
        ),
        (MdTableEmpty, "markdown/table: table must not be empty"),
        (MdTableRowMustBeList, "markdown/table: row {0} must be a list"),
        (
            MdTableColumnMismatch,
            "markdown/table: row {0} has {1} columns, expected {2}",
        ),
        // DAPデバッガーエラー
        (DapEmptyExpression, "Empty expression"),
        (DapEvaluationError, "Evaluation error: {0}"),
        (DapParseError, "Parse error: {0}"),
        (DapNoEnvironment, "No environment available (not stopped at breakpoint)"),
        (DapDebuggerNotAvailable, "Debugger not available"),
        (DapServerError, "DAP server error: {0}"),
        (DapServerNotEnabled, "Error: DAP server is not enabled. Build with --features dap-server"),
        (InternalError, "Internal error: {0}"),
        // プロジェクト管理エラー
        (QiTomlFailedToRead, "Failed to read qi.toml: {0}"),
        (QiTomlFailedToParse, "Failed to parse qi.toml: {0}"),
        (QiTomlFailedToSerialize, "Failed to serialize qi.toml: {0}"),
        (QiTomlFailedToWrite, "Failed to write qi.toml: {0}"),
        (FailedToGetCurrentDir, "Failed to get current directory: {0}"),
        (DirectoryAlreadyExists, "Directory '{0}' already exists"),
        (FailedToCreateDirectory, "Failed to create directory: {0}"),
        (FailedToCreateSrcDir, "Failed to create src/ directory: {0}"),
        (FailedToCreateExamplesDir, "Failed to create examples/ directory: {0}"),
        (FailedToCreateTestsDir, "Failed to create tests/ directory: {0}"),
        (FailedToCreateMainQi, "Failed to create main.qi: {0}"),
        (FailedToCreateLibQi, "Failed to create src/lib.qi: {0}"),
        (FailedToCreateExampleQi, "Failed to create examples/example.qi: {0}"),
        (FailedToCreateTestQi, "Failed to create tests/test.qi: {0}"),
        (TemplateNotFound, "Template '{0}' not found"),
        (FailedToReadDirectory, "Failed to read directory: {0}"),
        (FailedToReadFile, "Failed to read file: {0}"),
        (FailedToWriteFile, "Failed to write file: {0}"),
        (TemplateTomlFailedToRead, "Failed to read template.toml: {0}"),
        (TemplateTomlFailedToParse, "Failed to parse template.toml: {0}"),
        // 評価器エラー
        (TypeErrorVectorPattern, "Type error: cannot pass {0} to vector pattern"),
        (ArgErrorVectorPatternMinimum, "Argument error: vector pattern expects at least {0} elements, but got {1}"),
        (ArgErrorVectorPattern, "Argument error: vector pattern expects {0} elements, but got {1}"),
        (TypeErrorMapPattern, "Type error: cannot pass {0} to map pattern"),
        (KeyErrorMapMissing, "Key error: map does not have key :{0}"),
        // データベース・KVSエラー
        (ConnectionError, "Connection error: {0}"),
        (ConnectionNotFound, "Connection not found: {0}"),
        (EvalError, "eval: {0}"),
        (FailedToCreateRuntime, "Failed to create runtime: {0}"),
        (FailedToExecuteColumnsQuery, "Failed to execute columns query: {0}"),
        (FailedToExecuteForeignKeysQuery, "Failed to execute foreign keys query: {0}"),
        (FailedToExecuteIndexColumnsQuery, "Failed to execute index columns query: {0}"),
        (FailedToExecuteIndexesQuery, "Failed to execute indexes query: {0}"),
        (FailedToExecuteTablesQuery, "Failed to execute tables query: {0}"),
        (FailedToGetColumnName, "Failed to get column name: {0}"),
        (FailedToGetColumnValue, "Failed to get column value: {0}"),
        (FailedToGetDatabaseVersion, "Failed to get database version: {0}"),
        (FailedToPrepareStatement, "Failed to prepare statement: {0}"),
        (FailedToQueryColumns, "Failed to query columns: {0}"),
        (FailedToQueryForeignKeys, "Failed to query foreign keys: {0}"),
        (FailedToQueryIndexColumns, "Failed to query index columns: {0}"),
        (FailedToQueryIndexes, "Failed to query indexes: {0}"),
        (FailedToQueryTables, "Failed to query tables: {0}"),
        (FailedToReadFileMetadata, "Failed to read file metadata: {0}"),
        (InvalidIsolationLevel, "Invalid isolation level: {0}"),
        (UnsupportedKvsUrl, "Unsupported KVS URL: {0}"),
        (UnsupportedUrl, "Unsupported URL: {0}"),
        (RedisSupportNotEnabled, "Redis support not enabled (feature 'kvs-redis' required)"),
        (InvalidConnection, "Invalid connection (expected KvsConnection:xxx)"),
        (QiTomlAlreadyExists, "qi.toml already exists"),
        (PatternErrorNotAllowed, "Pattern error: this pattern cannot be used in function parameters or let bindings (only in match)"),
        (UnexpectedResponse, "Unexpected response"),
    ])
});

/// 日本語エラーメッセージ（enにフォールバック可能）
static JA_MSGS: LazyLock<HashMap<MsgKey, &'static str>> = LazyLock::new(|| {
    use MsgKey::*;
    HashMap::from([
        // パーサーエラー
        (UnexpectedToken, "予期しないトークン: {0}"),
        (UnexpectedEof, "ファイルが予期せず終了しました（括弧が閉じられていない可能性があります）"),
        (ExpectedToken, "期待: {0}, 実際: {1}"),
        (NeedsSymbol, "{0}にはシンボルが必要です"),
        (VarargNeedsName, "&の後には変数名が必要です"),
        (UnexpectedPattern, "予期しないパターン: {0}"),
        (RestNeedsVar, "...の後には変数名が必要です"),
        // レキサーエラー
        (UnexpectedChar, "予期しない文字: {0}"),
        (UnclosedString, "文字列が閉じられていません"),
        (NumberLiteralInvalid, "不正な数値リテラル: {0}"),
        (
            EmptyKeyword,
            "空のキーワード: ':' の後には識別子が必要です",
        ),
        // 評価器エラー
        (UndefinedVar, "未定義の変数: {0}"),
        (
            UndefinedVarWithSuggestions,
            "未定義の変数: {0}\n      もしかして: {1}?",
        ),
        (NotAFunction, "関数ではありません: {0}"),
        (TypeMismatch, "型エラー: 期待={0}, 実際={1} ({2})"),
        (
            ArgCountMismatch,
            "引数の数が一致しません: 期待 {0}, 実際 {1}",
        ),
        (DivisionByZero, "ゼロ除算エラー"),
        (
            ExportOnlyInModule,
            "exportはmodule定義の中でのみ使用できます",
        ),
        (CannotQuote, "quoteできません: {0}"),
        (NoMatchingPattern, "どのパターンにもマッチしませんでした"),
        // モジュールエラー
        (ModuleNeedsName, "moduleにはモジュール名が必要です"),
        (ExportNeedsSymbols, "exportにはシンボルが必要です"),
        (UseNeedsModuleName, "useにはモジュール名が必要です"),
        (
            ExpectedSymbolInOnlyList,
            ":onlyリストにはシンボルが必要です",
        ),
        (AsNeedsAlias, ":asにはエイリアス名が必要です"),
        (UseNeedsMode, "useには:onlyまたは:asが必要です"),
        (
            SymbolNotFound,
            "シンボル{0}が見つかりません（モジュール: {1}）",
        ),
        (
            ModuleNotFound,
            "モジュール{0}が見つかりません（{0}.qi）",
        ),
        (
            SymbolNotExported,
            "シンボル{0}はモジュール{1}からエクスポートされていません",
        ),
        (
            UseAsNotImplemented,
            ":asモードはまだ実装されていません",
        ),
        // 引数エラー
        (
            NeedAtLeastNArgs,
            "{0}には少なくとも{1}個の引数が必要です",
        ),
        (NeedExactlyNArgs, "{0}には{1}個の引数が必要です"),
        (Need2Or3Args, "{0}には2または3個の引数が必要です"),
        (Need1Or2Args, "{0}には1または2個の引数が必要です"),
        (Need0Or1Args, "{0}には0または1個の引数が必要です"),
        (Need2Args, "{0}には2つの引数が必要です"),
        (Need1Arg, "{0}には1つの引数が必要です"),
        (Need0Args, "{0}には引数は不要です"),
        // 型エラー
        (TypeOnly, "{0}は{1}のみ受け付けます"),
        (TypeOnlyWithDebug, "{0}は{1}のみ受け付けます: {2}"),
        (ArgMustBeType, "{0}: 引数は{1}である必要があります"),
        (FirstArgMustBe, "{0}の第1引数は{1}が必要です"),
        (SecondArgMustBe, "{0}の第2引数は{1}が必要です"),
        (ThirdArgMustBe, "{0}の第3引数は{1}が必要です"),
        (
            KeyMustBeKeyword,
            "キーは文字列またはキーワードが必要です",
        ),
        (FloatKeyNotAllowed, "Floatはマップのキーとして使用できません"),
        (InvalidMapKey, "無効なマップキー型: {0}"),
        (KeyNotFound, "キーが見つかりません: {0}"),
        (MustBePositive, "{0}: {1}は正の数である必要があります"),
        (
            MustBeNonNegative,
            "{0}: {1}は非負の数である必要があります",
        ),
        (MustBeInteger, "{0}: {1}は整数である必要があります"),
        (MustBeString, "{0}: {1}は文字列である必要があります"),
        (
            MinMustBeLessThanMax,
            "{0}: minはmaxより小さい必要があります",
        ),
        (
            MustBeListOrVector,
            "{0}: {1}はリストまたはベクタである必要があります",
        ),
        (
            MustBePromise,
            "{0}: {1}はプロミス（チャネル）である必要があります",
        ),
        (MustBeScope, "{0}: {1}はスコープである必要があります"),
        (MustNotBeEmpty, "{0}: {1}は空であってはいけません"),
        (
            FuncMustReturnType,
            "{0}: 関数は{1}を返す必要があります",
        ),
        (MustBeMap, "{0}: {1}はマップである必要があります"),
        // 特殊な引数エラー
        (SplitTwoStrings, "splitは2つの文字列が必要です"),
        (JoinStringAndList, "joinは文字列とリストが必要です"),
        (
            AssocMapAndKeyValues,
            "assocはマップと1つ以上のキー・値のペアが必要です",
        ),
        (
            DissocMapAndKeys,
            "dissocはマップと1つ以上のキーが必要です",
        ),
        (
            VariadicFnNeedsOneParam,
            "可変長引数関数にはパラメータが1つだけ必要です",
        ),
        // f-string エラー
        (FStringUnclosedBrace, "f-string: 閉じられていない { があります"),
        (FStringUnclosed, "f-string: 閉じられていない文字列です"),
        (FStringCannotBeQuoted, "f-string はquoteできません"),
        (FStringCodeParseError, "f-string: コードのパースエラー: {0}"),
        // マクロエラー
        (MacVarargNeedsSymbol, "mac: &の後にシンボルが必要です"),
        (
            VariadicMacroNeedsParams,
            "可変長マクロはパラメータが必要です",
        ),
        (
            MacArgCountMismatch,
            "mac {0}: 引数の数が一致しません（期待: {1}, 実際: {2}）",
        ),
        (
            MacVariadicArgCountMismatch,
            "mac {0}: 引数の数が不足しています（最低: {1}, 実際: {2}）",
        ),
        // quasiquote エラー
        (
            UnquoteOutsideQuasiquote,
            "unquote: quasiquote外では使用できません",
        ),
        (
            UnquoteSpliceOutsideQuasiquote,
            "unquote-splice: quasiquote外では使用できません",
        ),
        (
            UnquoteSpliceNeedsListOrVector,
            "unquote-splice: リストまたはベクタが必要です",
        ),
        // loop/recur エラー
        (RecurNotFound, "recurが見つかりません"),
        (
            RecurArgCountMismatch,
            "recur: 引数の数が一致しません（期待: {0}, 実際: {1}）",
        ),
        // 内部変換エラー
        (ValueCannotBeConverted, "この値は変換できません"),
        // モジュールロード詳細エラー
        (CircularDependency, "循環参照を検出しました: {0}"),
        (
            ModuleParserInitError,
            "モジュール{0}のパーサー初期化エラー: {1}",
        ),
        (ModuleParseError, "モジュール{0}のパースエラー: {1}"),
        (
            ModuleMustExport,
            "モジュール{0}はexportを含む必要があります",
        ),
        // その他の特殊エラー
        (AsNeedsVarName, ":asには変数名が必要です"),
        (NeedNArgsDesc, "{0}には{1}個の引数が必要です: {2}"),
        (SelectNeedsList, "{0}にはリストが必要です"),
        (
            SelectNeedsAtLeastOne,
            "{0}には少なくとも1つのケースが必要です",
        ),
        (
            SelectTimeoutCase,
            "{0}: :timeoutケースは3要素が必要です: [:timeout ms handler]",
        ),
        (
            SelectOnlyOneTimeout,
            "{0}には:timeoutケースは1つだけです",
        ),
        (
            SelectChannelCase,
            "{0}: チャネルケースは2要素が必要です: [channel handler]",
        ),
        (
            SelectCaseMustStart,
            "{0}: ケースはチャネルまたは:timeoutで始まる必要があります",
        ),
        (SelectCaseMustBe, "{0}: ケースはリストである必要があります [channel handler] or [:timeout ms handler]"),
        (AllElementsMustBe, "{0}: 全ての要素は{1}である必要があります"),
        // 並行処理エラー
        (ChannelClosed, "{0}: チャネルは閉じられています"),
        (ExpectedKeyword, "{0}: {1}キーワードが必要です"),
        (PromiseFailed, "プロミスが失敗しました"),
        (NotAPromise, "プロミスではありません"),
        (UnexpectedError, "{0}: 予期しないエラー"),
        (RecvArgs, "{0}: 1または3個の引数が必要です: ({0} ch) or ({0} ch :timeout ms)"),
        (TimeoutMustBeMs, "{0}: タイムアウトは整数（ミリ秒）である必要があります"),
        // その他のエラー
        (UnsupportedNumberType, "サポートされていない数値型です"),
        (RailwayRequiresOkError, "|>? には {:ok/:error} マップが必要です"),
        (InvalidTimestamp, "{0}: 不正なタイムスタンプです"),
        (InvalidDateFormat, "{0}: 不正な日付フォーマット: {1}"),
        (InvalidPercentile, "{0}: パーセンタイルは0から100の間である必要があります"),
        (SystemTimeError, "{0}: システム時刻エラー: {1}"),
        (JsonParseError, "{0}: {1}"),
        (JsonStringifyError2, "{0}: {1}"),
        (CannotParseAsInt, "{0}: '{1}'を整数としてパースできません"),
        (CannotConvertToInt, "{0}: {1}を整数に変換できません"),
        (CannotParseAsFloat, "{0}: '{1}'を浮動小数点数としてパースできません"),
        (CannotConvertToFloat, "{0}: {1}を浮動小数点数に変換できません"),
        (CannotConvertToJson, "{0}をJSONに変換できません"),
        (InvalidRegex, "{0}: 不正な正規表現: {1}"),
        // JWT エラー
        (NeedNArgs, "{0}には{1}個の引数が必要です"),
        (InvalidAlgorithm, "{0}: 不正なアルゴリズム'{1}'（サポート: {2}）"),
        (InvalidFloat, "{0}: 不正な浮動小数点数値です"),
        (InvalidNumber, "{0}: 不正な数値です"),
        // パスワードハッシュエラー
        (PasswordHashError, "{0}: パスワードハッシュエラー: {1}"),
        // 警告
        (RedefineBuiltin, "警告: ビルトイン関数'{0}'を再定義しています ({1})"),
        (RedefineFunction, "警告: 関数'{0}'を再定義しています"),
        (RedefineVariable, "警告: 変数'{0}'を再定義しています"),
        // CSV エラー
        (FileReadError, "{0}: ファイル読み込みエラー: {1}"),
        (CsvCannotSerialize, "csv/stringify: {0}をシリアライズできません"),
        (CsvRecordMustBeList, "csv/stringify: 各レコードはリストである必要があります"),
        (CsvParseNeed1Or3Args, "csv/parseには1または3個の引数が必要です"),
        (CsvDelimiterMustBeSingleChar, "csv/parse: デリミタは1文字である必要があります"),
        (CsvInvalidDelimiterArg, "csv/parse: 不正なデリミタ引数です (:delimiter \"char\" を使用してください)"),
        // コマンド実行エラー
        (CmdEmptyCommand, "コマンドを空にすることはできません"),
        (CmdFirstArgMustBeString, "コマンドリストの最初の要素は文字列である必要があります"),
        (CmdArgsMustBeStrings, "すべてのコマンド引数は文字列である必要があります"),
        (CmdInvalidArgument, "不正なコマンド引数: 文字列またはリストが必要です"),
        (CmdExecutionFailed, "コマンド実行失敗: {0}"),
        (CmdWriteFailed, "コマンド標準入力への書き込み失敗: {0}"),
        (CmdWaitFailed, "コマンドの待機失敗: {0}"),
        (CmdInvalidProcessHandle, "不正なプロセスハンドルです"),
        (CmdProcessNotFound, "プロセスが見つかりません (PID: {0})"),
        (CmdStdinClosed, "標準入力は既に閉じられています"),
        (CmdStdoutClosed, "標準出力は既に閉じられています"),
        (CmdReadFailed, "読み込み失敗: {0}"),
        (MustBePositiveInteger, "{0}: {1}は正の整数である必要があります"),
        // データ構造エラー
        (MustBeQueue, "{0}: {1}はキューである必要があります"),
        (MustBeStack, "{0}: {1}はスタックである必要があります"),
        (IsEmpty, "{0}: {1}は空です"),
        // テストエラー
        (TestsFailed, "一部のテストが失敗しました"),
        (AssertExpectedException, "アサーション失敗: 例外が期待されましたがスローされませんでした"),
        // パスエラー
        (AllPathsMustBeStrings, "{0}: すべてのパスは文字列である必要があります"),
        // サーバーエラー
        (JsonStringifyError, "JSON文字列化失敗"),
        (RequestMustHave, "{0}: リクエストには{1}が必要です"),
        (RequestMustBe, "{0}: リクエストは{1}である必要があります"),
        (InvalidFilePath, "{0}: 不正なファイルパス (..を含んでいます)"),
        (FourthArgMustBe, "{0}の第4引数は{1}が必要です"),
        (Need3Args, "{0}には3個の引数が必要です"),
        (Need1Or3Args, "{0}: 1または3個の引数が必要です"),
        // 環境変数エラー
        (ValueMustBeStringNumberBool, "{0}: 値は文字列、数値、または真偽値である必要があります"),
        // I/Oエラー
        (BothArgsMustBeStrings, "{0}: 両方の引数は文字列である必要があります"),
        (UnsupportedEncoding, "サポートされていないエンコーディング: {0}"),
        (KeywordRequiresValue, "キーワード:{0}には値が必要です"),
        (ExpectedKeywordArg, "キーワード引数が必要です、実際: {0}"),
        (FileAlreadyExists, "{0}: ファイルは既に存在します"),
        (InvalidIfExistsOption, "不正な:if-existsオプション: {0}"),
        // HTTPエラー
        (HttpClientError, "HTTPクライアントエラー: {0}"),
        (HttpCompressionError, "圧縮エラー: {0}"),
        (HttpStreamClientError, "http stream: クライアント作成エラー: {0}"),
        (HttpStreamRequestFailed, "http stream: リクエスト失敗: {0}"),
        (HttpStreamReadBytesFailed, "http stream: バイト読み込み失敗: {0}"),
        (HttpStreamReadBodyFailed, "http stream: ボディ読み込み失敗: {0}"),
        (HttpRequestUrlRequired, "http/request: :urlが必要です"),
        (HttpUnsupportedMethod, "サポートされていないHTTPメソッド: {0}"),
        (HttpStreamError, "http stream: HTTP {0}"),
        // I/Oエラー（詳細）
        (IoFileError, "{0}: {1}"),
        (IoFailedToDecodeUtf8, "{0}: UTF-8としてデコード失敗 (不正なバイト列)"),
        (IoFailedToCreateDir, "{0}: ディレクトリ作成失敗: {1}"),
        (IoFailedToOpenForAppend, "{0}: 追記用オープン失敗: {1}"),
        (IoFailedToAppend, "{0}: 追記失敗: {1}"),
        (IoFailedToWrite, "{0}: 書き込み失敗: {1}"),
        (FileStreamFailedToOpen, "file-stream: '{0}'のオープン失敗: {1}"),
        (WriteStreamFailedToCreate, "write-stream: {0}の作成失敗: {1}"),
        (WriteStreamFailedToWrite, "write-stream: {0}への書き込み失敗: {1}"),
        (IoListDirInvalidPattern, "io/list-dir: 不正なパターン'{0}': {1}"),
        (IoListDirFailedToRead, "io/list-dir: エントリ読み込み失敗: {0}"),
        (IoCreateDirFailed, "io/create-dir: '{0}'の作成失敗: {1}"),
        (IoDeleteFileFailed, "io/delete-file: '{0}'の削除失敗: {1}"),
        (IoDeleteDirFailed, "io/delete-dir: '{0}'の削除失敗: {1}"),
        (IoCopyFileFailed, "io/copy-file: '{0}'から'{1}'へのコピー失敗: {2}"),
        (IoMoveFileFailed, "io/move-file: '{0}'から'{1}'への移動失敗: {2}"),
        (IoGetMetadataFailed, "io/file-info: '{0}'のメタデータ取得失敗: {1}"),
        // サーバーエラー（詳細）
        (ServerFailedToReadBody, "リクエストボディの読み込み失敗: {0}"),
        (ServerFailedToDecompressGzip, "gzipボディの解凍失敗: {0}"),
        (ServerFailedToBuildResponse, "レスポンス構築失敗: {0}"),
        (ServerStaticFileMetadataFailed, "server/static-file: ファイルメタデータ読み込み失敗: {0}"),
        (ServerHandlerMustReturnMap, "ハンドラはマップを返す必要があります、実際: {0}"),
        (ServerHandlerMustBeFunction, "ハンドラは関数またはルーターである必要があります、実際: {0}"),
        (ServerHandlerError, "ハンドラエラー: {0}"),
        (ServerFileTooLarge, "ファイルが大きすぎます: {0}バイト (最大: {1}バイト / {2}MB)。パス: {3}"),
        (ServerFailedToReadFile, "ファイル読み込み失敗: {0}"),
        (ServerStaticFileTooLarge, "server/static-file: ファイルが大きすぎます: {0}バイト (最大: {1}バイト / {2}MB)。将来的にストリーミングの使用を検討してください。"),
        (ServerStaticFileFailedToRead, "server/static-file: ファイル読み込み失敗: {0}"),
        (ServerStaticDirNotDirectory, "server/static-dir: {0}はディレクトリではありません"),
        // データベース汎用エラー（PostgreSQL/MySQL/SQLite共通）
        (DbFailedToConnect, "データベース接続失敗: {0}"),
        (DbFailedToExecuteQuery, "クエリ実行失敗: {0}"),
        (DbFailedToExecuteStatement, "ステートメント実行失敗: {0}"),
        (DbFailedToBeginTransaction, "トランザクション開始失敗: {0}"),
        (DbFailedToCommitTransaction, "トランザクションコミット失敗: {0}"),
        (DbFailedToRollbackTransaction, "トランザクションロールバック失敗: {0}"),
        (DbUnsupportedUrl, "サポートされていないデータベースURL: {0}"),
        // SQLiteエラー
        (SqliteFailedToOpen, "SQLiteデータベースのオープン失敗: {0}"),
        (SqliteFailedToSetTimeout, "タイムアウト設定失敗: {0}"),
        (SqliteFailedToGetColumnName, "カラム名取得失敗: {0}"),
        (SqliteFailedToPrepare, "ステートメント準備失敗: {0}"),
        (SqliteFailedToExecuteQuery, "クエリ実行失敗: {0}"),
        (SqliteFailedToExecuteStatement, "ステートメント実行失敗: {0}"),
        (SqliteFailedToBeginTransaction, "トランザクション開始失敗: {0}"),
        (SqliteFailedToCommitTransaction, "トランザクションコミット失敗: {0}"),
        (SqliteFailedToRollbackTransaction, "トランザクションロールバック失敗: {0}"),
        // 環境変数エラー（詳細）
        (EnvLoadDotenvFailedToRead, "env/load-dotenv: ファイル'{0}'の読み込み失敗: {1}"),
        (EnvLoadDotenvInvalidFormat, "env/load-dotenv: {0}行目の不正なフォーマット: '{1}'"),
        // CSVエラー（詳細）
        (CsvWriteFileStringifyFailed, "csv/write-file: 文字列化失敗"),
        (CsvWriteFileFailedToWrite, "csv/write-file: '{0}'の書き込み失敗: {1}"),
        // ログエラー
        (LogSetLevelInvalidLevel, "log/set-level: 不正なレベル'{0}' (有効: debug, info, warn, error)"),
        (LogSetFormatInvalidFormat, "log/set-format: 不正なフォーマット'{0}' (有効: text, json)"),
        // 時刻エラー（詳細）
        (TimeParseFailedToParse, "time/parse: '{0}'をフォーマット'{1}'でパース失敗"),
        // ZIPエラー
        (ZipPathDoesNotExist, "{0}: パス'{1}'は存在しません"),
        // データベースエラー
        (DbUnsupportedUrl, "サポートされていないデータベースURL: {0}。サポート: sqlite:"),
        (DbNeed2To4Args, "{0}には2-4個の引数が必要です、実際: {1}"),
        (DbNeed1To3Args, "{0}には1-3個の引数が必要です、実際: {1}"),
        (DbExpectedConnection, "DbConnectionが必要です、実際: {0}"),
        (DbConnectionNotFound, "接続が見つかりません: {0}"),
        (DbExpectedTransaction, "DbTransactionが必要です、実際: {0}"),
        (DbTransactionNotFound, "トランザクションが見つかりません: {0}"),
        (DbExpectedConnectionOrTransaction, "DbConnectionまたはDbTransactionが必要です、実際: {0}"),
        (DbExpectedPool, "DbPoolが必要です、実際: {0}"),
        (DbPoolNotFound, "プールが見つかりません: {0}"),
        (DbInvalidPoolSize, "{0}: 不正なプールサイズです、期待: {1}"),
        // I/Oエラー（追加）
        (IoFailedToDecodeAs, "{0}: {1}としてデコード失敗 (不正なバイト列)"),
        (IoCouldNotDetectEncoding, "{0}: エンコーディングを検出できませんでした (UTF-8、UTF-16、日本語、中国語、韓国語、ヨーロッパのエンコーディングを試行)"),
        (IoAppendFileFailedToWrite, "append-file: {0}の書き込み失敗: {1}"),
        (IoAppendFileFailedToOpen, "append-file: {0}のオープン失敗: {1}"),
        (IoReadLinesFailedToRead, "read-lines: {0}の読み込み失敗: {1}"),
        // Featureエラー
        (FeatureDisabled, "{0}サポートは無効化されています。feature '{1}'でビルドしてください: {2}"),
        (DbUnsupportedDriver, "サポートされていないデータベースドライバ: {0}"),
        // Markdownエラー
        (MdHeaderInvalidLevel, "markdown/header: レベルは1-6である必要があります、実際: {0}"),
        (MdTableEmpty, "markdown/table: テーブルは空であってはいけません"),
        (MdTableRowMustBeList, "markdown/table: 行{0}はリストである必要があります"),
        (MdTableColumnMismatch, "markdown/table: 行{0}は{1}列ですが、{2}列が期待されています"),
        // DAPデバッガーエラー
        (DapEmptyExpression, "式が空です"),
        (DapEvaluationError, "評価エラー: {0}"),
        (DapParseError, "パースエラー: {0}"),
        (DapNoEnvironment, "環境が利用できません（ブレークポイントで停止していません）"),
        (DapDebuggerNotAvailable, "デバッガーが利用できません"),
        (DapServerError, "DAPサーバーエラー: {0}"),
        (DapServerNotEnabled, "エラー: DAPサーバーが有効化されていません。--features dap-serverでビルドしてください"),
        (InternalError, "内部エラー: {0}"),
        // プロジェクト管理エラー
        (QiTomlFailedToRead, "qi.tomlの読み込みに失敗: {0}"),
        (QiTomlFailedToParse, "qi.tomlのパースに失敗: {0}"),
        (QiTomlFailedToSerialize, "qi.tomlのシリアライズに失敗: {0}"),
        (QiTomlFailedToWrite, "qi.tomlの書き込みに失敗: {0}"),
        (FailedToGetCurrentDir, "カレントディレクトリの取得に失敗: {0}"),
        (DirectoryAlreadyExists, "ディレクトリ '{0}' は既に存在します"),
        (FailedToCreateDirectory, "ディレクトリの作成に失敗: {0}"),
        (FailedToCreateSrcDir, "src/ディレクトリの作成に失敗: {0}"),
        (FailedToCreateExamplesDir, "examples/ディレクトリの作成に失敗: {0}"),
        (FailedToCreateTestsDir, "tests/ディレクトリの作成に失敗: {0}"),
        (FailedToCreateMainQi, "main.qiの作成に失敗: {0}"),
        (FailedToCreateLibQi, "src/lib.qiの作成に失敗: {0}"),
        (FailedToCreateExampleQi, "examples/example.qiの作成に失敗: {0}"),
        (FailedToCreateTestQi, "tests/test.qiの作成に失敗: {0}"),
        (TemplateNotFound, "テンプレート '{0}' が見つかりません"),
        (FailedToReadDirectory, "ディレクトリの読み込みに失敗: {0}"),
        (FailedToReadFile, "ファイルの読み込みに失敗: {0}"),
        (FailedToWriteFile, "ファイルの書き込みに失敗: {0}"),
        (TemplateTomlFailedToRead, "template.tomlの読み込みに失敗: {0}"),
        (TemplateTomlFailedToParse, "template.tomlのパースに失敗: {0}"),
        // 評価器エラー
        (TypeErrorVectorPattern, "型エラー: ベクタパターンに対して{0}を渡すことはできません"),
        (ArgErrorVectorPatternMinimum, "引数エラー: ベクタパターンは最低{0}個の要素を期待しましたが、{1}個が渡されました"),
        (ArgErrorVectorPattern, "引数エラー: ベクタパターンは{0}個の要素を期待しましたが、{1}個が渡されました"),
        (TypeErrorMapPattern, "型エラー: マップパターンに対して{0}を渡すことはできません"),
        (KeyErrorMapMissing, "キーエラー: マップにキー :{0}が存在しません"),
        // データベース・KVSエラー
        (ConnectionError, "接続エラー: {0}"),
        (ConnectionNotFound, "接続が見つかりません: {0}"),
        (EvalError, "評価: {0}"),
        (FailedToCreateRuntime, "ランタイムの作成に失敗: {0}"),
        (FailedToExecuteColumnsQuery, "カラムクエリの実行に失敗: {0}"),
        (FailedToExecuteForeignKeysQuery, "外部キークエリの実行に失敗: {0}"),
        (FailedToExecuteIndexColumnsQuery, "インデックスカラムクエリの実行に失敗: {0}"),
        (FailedToExecuteIndexesQuery, "インデックスクエリの実行に失敗: {0}"),
        (FailedToExecuteTablesQuery, "テーブルクエリの実行に失敗: {0}"),
        (FailedToGetColumnName, "カラム名の取得に失敗: {0}"),
        (FailedToGetColumnValue, "カラム値の取得に失敗: {0}"),
        (FailedToGetDatabaseVersion, "データベースバージョンの取得に失敗: {0}"),
        (FailedToPrepareStatement, "ステートメントの準備に失敗: {0}"),
        (FailedToQueryColumns, "カラムのクエリに失敗: {0}"),
        (FailedToQueryForeignKeys, "外部キーのクエリに失敗: {0}"),
        (FailedToQueryIndexColumns, "インデックスカラムのクエリに失敗: {0}"),
        (FailedToQueryIndexes, "インデックスのクエリに失敗: {0}"),
        (FailedToQueryTables, "テーブルのクエリに失敗: {0}"),
        (FailedToReadFileMetadata, "ファイルメタデータの読み込みに失敗: {0}"),
        (InvalidIsolationLevel, "無効なアイソレーションレベル: {0}"),
        (UnsupportedKvsUrl, "サポートされていないKVS URL: {0}"),
        (UnsupportedUrl, "サポートされていないURL: {0}"),
        (RedisSupportNotEnabled, "Redisサポートが有効化されていません（feature 'kvs-redis'が必要です）"),
        (InvalidConnection, "無効な接続（KvsConnection:xxxが期待されます）"),
        (QiTomlAlreadyExists, "qi.tomlが既に存在します"),
        (PatternErrorNotAllowed, "パターンエラー: このパターンは関数パラメータやlet束縛では使用できません（matchでのみ使用可能）"),
        (UnexpectedResponse, "予期しないレスポンス"),
    ])
});

/// 英語UIメッセージ
static EN_UI_MSGS: LazyLock<HashMap<UiMsg, &'static str>> = LazyLock::new(|| {
    use UiMsg::*;
    HashMap::from([
        // REPL
        (ReplWelcome, "Qi REPL v{0}"),
        (ReplPressCtrlC, "Press Ctrl+C to exit"),
        (ReplGoodbye, "Goodbye!"),
        (ReplLoading, "Loading {0}..."),
        (ReplLoaded, "Loaded {0}"),
        (ReplTypeHelp, "Type :help for help"),
        (ReplAvailableCommands, "Available commands:"),
        (ReplNoVariables, "No variables defined"),
        (ReplDefinedVariables, "Defined variables:"),
        (ReplNoFunctions, "No functions defined"),
        (ReplDefinedFunctions, "Defined functions:"),
        (
            ReplNoBuiltinsMatching,
            "No builtin functions matching \"{0}\"",
        ),
        (ReplBuiltinsMatching, "Builtin functions matching \"{0}\":"),
        (ReplBuiltinFunctions, "Builtin functions:"),
        (ReplBuiltinTotal, "Total: {0} functions"),
        (ReplBuiltinTip, "Tip: Use :builtins <pattern> to search"),
        (ReplEnvCleared, "Environment cleared"),
        (ReplLoadUsage, "Usage: :load <filename>"),
        (ReplNoFileLoaded, "No file loaded yet"),
        (ReplUnknownCommand, "Unknown command: {0}"),
        (ReplTypeHelpForCommands, "Type :help for available commands"),
        // REPLコマンドヘルプ
        (ReplCommandHelp, "  :help              Show this help"),
        (ReplCommandDoc, "  :doc <name>        Show documentation for a function"),
        (
            ReplCommandVars,
            "  :vars              List defined variables",
        ),
        (
            ReplCommandFuncs,
            "  :funcs             List defined functions",
        ),
        (
            ReplCommandBuiltins,
            "  :builtins [pat]    List builtin functions (optional pattern)",
        ),
        (ReplCommandClear, "  :clear             Clear environment"),
        (
            ReplCommandLoad,
            "  :load <file>       Load and execute a file",
        ),
        (
            ReplCommandReload,
            "  :reload            Reload last loaded file",
        ),
        (ReplCommandQuit, "  :quit, :q          Exit REPL"),
        // テスト
        (TestNoTests, "No tests found"),
        (TestResults, "Test Results:"),
        (
            TestResultsSeparator,
            "----------------------------------------",
        ),
        (TestSummary, "Summary: {0} passed, {1} failed, {2} total"),
        (
            TestAssertEqFailed,
            "Assertion failed: {0}\n  Expected: {1}\n  Actual:   {2}",
        ),
        (
            TestAssertTruthyFailed,
            "Assertion failed: {0}\n  Expected truthy value, got: {1}",
        ),
        (
            TestAssertFalsyFailed,
            "Assertion failed: {0}\n  Expected falsy value, got: {1}",
        ),
        // プロファイラー
        (
            ProfileNoData,
            "No profiling data available. Use (profile/start) to begin profiling.",
        ),
        (ProfileUseStart, "Use (profile/start) to begin profiling"),
        (ProfileReport, "Profiling Report:"),
        (ProfileTableHeader, "{0:<30} {1:>10} {2:>12} {3:>12}"),
        (ProfileTotalTime, "Total execution time: {0}"),
        // ヘルプ
        (HelpTitle, "Qi Programming Language"),
        (HelpUsage, "Usage: qi [options] [file]"),
        (HelpOptions, "Options:"),
        (HelpExamples, "Examples:"),
        (HelpEnvVars, "Environment Variables:"),
        // オプション説明
        (OptExecute, "  -e <code>          Execute code directly"),
        (OptStdin, "  -s, --stdin        Read code from stdin"),
        (
            OptLoad,
            "  -l <file>          Load file before starting REPL",
        ),
        (
            OptQuiet,
            "  -q, --quiet        Start REPL in quiet mode (no startup messages)",
        ),
        (OptHelp, "  -h, --help         Show this help message"),
        (OptVersion, "  -v, --version      Show version information"),
        (OptNew, "    new <name> [-t <template>]  Create a new Qi project"),
        (OptTemplate, "    template <list|info>        Template management"),
        (OptDap, "    --dap                       Start Debug Adapter Protocol server"),
        // ヘルプ例
        (ExampleStartRepl, "  qi                 Start REPL"),
        (ExampleRunScript, "  qi script.qi       Run script"),
        (ExampleExecuteCode, "  qi -e '(+ 1 2)'    Execute code"),
        (ExampleStdin, "  echo '(+ 1 2)' | qi -s"),
        (
            ExampleLoadFile,
            "  qi -l prelude.qi   Load file and start REPL",
        ),
        (ExampleNewProject, "    qi new my-project        Create a new project"),
        (ExampleNewHttpServer, "    qi new myapi -t http-server  Create an HTTP server project"),
        (ExampleTemplateList, "    qi template list         List available templates"),
        // 環境変数説明
        (EnvLangQi, "  QI_LANG            Language (ja, en)"),
        (
            EnvLangSystem,
            "  LANG               System language (fallback)",
        ),
        // バージョン
        (VersionString, "Qi v{0}"),
        // エラー
        (ErrorFailedToRead, "Failed to read file"),
        (ErrorFailedToReadStdin, "Failed to read from stdin"),
        (ErrorRequiresArg, "Option {0} requires an argument"),
        (ErrorRequiresFile, "{0} requires a file path"),
        (ErrorUnknownOption, "Unknown option: {0}"),
        (ErrorUseHelp, "Use --help for usage information"),
        (ErrorInput, "Input error"),
        (ErrorParse, "Parse error"),
        (ErrorLexer, "Lexer error"),
        (ErrorRuntime, "Runtime error"),
        // DAP
        (DapStdinWaiting, "\n⏸️  Waiting for standard input"),
        (DapStdinInstructions, "   Enter '.stdin <text>' in the debug console\n"),
        (DapStdinSent, "✓ Sent to stdin: {0}"),
        // Project creation
        (ProjectNewNeedName, "Error: Please specify a project name"),
        (ProjectNewUsage, "Usage: qi new <project-name> [--template <template>]"),
        (ProjectNewUnknownOption, "Error: Unknown option: {0}"),
        (ProjectNewError, "Error: {0}"),
        (TemplateNeedSubcommand, "Error: Please specify a subcommand"),
        (TemplateUsage, "Usage: qi template <list|info>"),
        (TemplateNeedName, "Error: Please specify a template name"),
        (TemplateInfoUsage, "Usage: qi template info <name>"),
        (TemplateUnknownSubcommand, "Error: Unknown subcommand: {0}"),
        // REPL documentation
        (ReplDocUsage, "Usage: :doc <name>"),
        (ReplDocNotFound, "No such function or variable: {0}"),
        (ReplDocParameters, "\nParameters:"),
        (ReplDocExamples, "\nExamples:"),
        (ReplDocNoDoc, "(no documentation available)"),
    ])
});

/// 日本語UIメッセージ
static JA_UI_MSGS: LazyLock<HashMap<UiMsg, &'static str>> = LazyLock::new(|| {
    use UiMsg::*;
    HashMap::from([
        // REPL
        (ReplWelcome, "Qi REPL v{0}"),
        (ReplPressCtrlC, "終了するには Ctrl+C を押してください"),
        (ReplGoodbye, "さようなら！"),
        (ReplLoading, "{0}を読み込んでいます..."),
        (ReplLoaded, "{0}を読み込みました"),
        (
            ReplTypeHelp,
            "ヘルプを表示するには :help と入力してください",
        ),
        (ReplAvailableCommands, "利用可能なコマンド:"),
        (ReplNoVariables, "定義されている変数はありません"),
        (ReplDefinedVariables, "定義されている変数:"),
        (ReplNoFunctions, "定義されている関数はありません"),
        (ReplDefinedFunctions, "定義されている関数:"),
        (
            ReplNoBuiltinsMatching,
            "\"{0}\"に一致するビルトイン関数はありません",
        ),
        (ReplBuiltinsMatching, "\"{0}\"に一致するビルトイン関数:"),
        (ReplBuiltinFunctions, "ビルトイン関数:"),
        (ReplBuiltinTotal, "合計: {0}個の関数"),
        (
            ReplBuiltinTip,
            "ヒント: :builtins <パターン> で検索できます",
        ),
        (ReplEnvCleared, "環境をクリアしました"),
        (ReplLoadUsage, "使い方: :load <ファイル名>"),
        (ReplNoFileLoaded, "まだファイルが読み込まれていません"),
        (ReplUnknownCommand, "不明なコマンド: {0}"),
        (
            ReplTypeHelpForCommands,
            "利用可能なコマンドを表示するには :help と入力してください",
        ),
        // REPLコマンドヘルプ
        (ReplCommandHelp, "  :help              このヘルプを表示"),
        (ReplCommandDoc, "  :doc <name>        関数のドキュメントを表示"),
        (
            ReplCommandVars,
            "  :vars              定義されている変数を一覧表示",
        ),
        (
            ReplCommandFuncs,
            "  :funcs             定義されている関数を一覧表示",
        ),
        (
            ReplCommandBuiltins,
            "  :builtins [pat]    ビルトイン関数を一覧表示（パターン指定可能）",
        ),
        (ReplCommandClear, "  :clear             環境をクリア"),
        (
            ReplCommandLoad,
            "  :load <file>       ファイルを読み込んで実行",
        ),
        (
            ReplCommandReload,
            "  :reload            最後に読み込んだファイルを再読み込み",
        ),
        (ReplCommandQuit, "  :quit, :q          REPLを終了"),
        // テスト
        (TestNoTests, "テストが見つかりません"),
        (TestResults, "テスト結果:"),
        (
            TestResultsSeparator,
            "----------------------------------------",
        ),
        (TestSummary, "要約: {0}件成功, {1}件失敗, 合計{2}件"),
        (
            TestAssertEqFailed,
            "アサーション失敗: {0}\n  期待値: {1}\n  実際値: {2}",
        ),
        (
            TestAssertTruthyFailed,
            "アサーション失敗: {0}\n  真と評価される値が期待されましたが、実際: {1}",
        ),
        (
            TestAssertFalsyFailed,
            "アサーション失敗: {0}\n  偽と評価される値が期待されましたが、実際: {1}",
        ),
        // プロファイラー
        (
            ProfileNoData,
            "プロファイリングデータがありません。(profile/start)で計測を開始してください。",
        ),
        (ProfileUseStart, "(profile/start)で計測を開始してください"),
        (ProfileReport, "プロファイリングレポート:"),
        (ProfileTableHeader, "{0:<30} {1:>10} {2:>12} {3:>12}"),
        (ProfileTotalTime, "合計実行時間: {0}"),
        // ヘルプ
        (HelpTitle, "Qiプログラミング言語"),
        (HelpUsage, "使い方: qi [オプション] [ファイル]"),
        (HelpOptions, "オプション:"),
        (HelpExamples, "例:"),
        (HelpEnvVars, "環境変数:"),
        // オプション説明
        (OptExecute, "  -e <コード>        コードを直接実行"),
        (
            OptStdin,
            "  -s, --stdin        標準入力からコードを読み込む",
        ),
        (
            OptLoad,
            "  -l <ファイル>      REPL起動前にファイルを読み込む",
        ),
        (
            OptQuiet,
            "  -q, --quiet        quietモードでREPL起動（起動メッセージなし）",
        ),
        (OptHelp, "  -h, --help         このヘルプメッセージを表示"),
        (OptVersion, "  -v, --version      バージョン情報を表示"),
        (OptNew, "    new <name> [-t <template>]  新しいQiプロジェクトを作成"),
        (OptTemplate, "    template <list|info>        テンプレート管理"),
        (OptDap, "    --dap                       Debug Adapter Protocolサーバーを起動"),
        // ヘルプ例
        (ExampleStartRepl, "  qi                 REPLを起動"),
        (ExampleRunScript, "  qi script.qi       スクリプトを実行"),
        (ExampleExecuteCode, "  qi -e '(+ 1 2)'    コードを実行"),
        (ExampleStdin, "  echo '(+ 1 2)' | qi -s"),
        (
            ExampleLoadFile,
            "  qi -l prelude.qi   ファイルを読み込んでREPL起動",
        ),
        (ExampleNewProject, "    qi new my-project        新しいプロジェクトを作成"),
        (ExampleNewHttpServer, "    qi new myapi -t http-server  HTTPサーバープロジェクトを作成"),
        (ExampleTemplateList, "    qi template list         利用可能なテンプレート一覧"),
        // 環境変数説明
        (EnvLangQi, "  QI_LANG            言語 (ja, en)"),
        (
            EnvLangSystem,
            "  LANG               システム言語（フォールバック）",
        ),
        // バージョン
        (VersionString, "Qi v{0}"),
        // エラー
        (ErrorFailedToRead, "ファイルの読み込み失敗"),
        (ErrorFailedToReadStdin, "標準入力からの読み込み失敗"),
        (ErrorRequiresArg, "オプション{0}には引数が必要です"),
        (ErrorRequiresFile, "{0}にはファイルパスが必要です"),
        (ErrorUnknownOption, "不明なオプション: {0}"),
        (
            ErrorUseHelp,
            "使い方を表示するには --help を使用してください",
        ),
        (ErrorInput, "入力エラー"),
        (ErrorParse, "パースエラー"),
        (ErrorLexer, "字句解析エラー"),
        (ErrorRuntime, "実行時エラー"),
        // DAP
        (DapStdinWaiting, "\n⏸️  標準入力を待っています"),
        (DapStdinInstructions, "   デバッグコンソールで .stdin <text> と入力してください\n"),
        (DapStdinSent, "✓ 入力を送信しました: {0}"),
        // プロジェクト作成
        (ProjectNewNeedName, "エラー: プロジェクト名を指定してください"),
        (ProjectNewUsage, "使い方: qi new <project-name> [--template <template>]"),
        (ProjectNewUnknownOption, "エラー: 不明なオプション: {0}"),
        (ProjectNewError, "エラー: {0}"),
        (TemplateNeedSubcommand, "エラー: サブコマンドを指定してください"),
        (TemplateUsage, "使い方: qi template <list|info>"),
        (TemplateNeedName, "エラー: テンプレート名を指定してください"),
        (TemplateInfoUsage, "使い方: qi template info <name>"),
        (TemplateUnknownSubcommand, "エラー: 不明なサブコマンド: {0}"),
        // REPLドキュメント
        (ReplDocUsage, "使い方: :doc <name>"),
        (ReplDocNotFound, "関数または変数が見つかりません: {0}"),
        (ReplDocParameters, "\nパラメータ:"),
        (ReplDocExamples, "\n使用例:"),
        (ReplDocNoDoc, "(ドキュメントがありません)"),
    ])
});

/// メッセージマネージャー（HashMap検索、enフォールバック）
pub struct Messages {
    lang: Lang,
}

impl Messages {
    /// 言語設定でMessagesインスタンスを作成
    pub fn new(lang: Lang) -> Self {
        Self { lang }
    }

    /// メッセージを取得（jaになければenにフォールバック）
    pub fn get(&self, key: MsgKey) -> &'static str {
        match self.lang {
            Lang::En => EN_MSGS.get(&key).unwrap_or(&"[missing message]"),
            Lang::Ja => JA_MSGS
                .get(&key)
                .or_else(|| EN_MSGS.get(&key))
                .unwrap_or(&"[missing message]"),
        }
    }

    /// UIメッセージを取得（en以外で対象LANGになければenにフォールバック）
    pub fn ui(&self, key: UiMsg) -> &'static str {
        match self.lang {
            Lang::En => EN_UI_MSGS.get(&key).unwrap_or(&"[missing message]"),
            Lang::Ja => JA_UI_MSGS
                .get(&key)
                .or_else(|| EN_UI_MSGS.get(&key))
                .unwrap_or(&"[missing message]"),
        }
    }

    /// メッセージをフォーマット（プレースホルダー {0}, {1}, ... を置換）
    /// 一度の走査でO(n)で処理
    pub fn fmt(&self, key: MsgKey, args: &[&str]) -> String {
        let template = self.get(key);

        // 予想サイズを確保（テンプレート + 引数の合計長）
        let estimated_size = template.len() + args.iter().map(|s| s.len()).sum::<usize>();
        let mut result = String::with_capacity(estimated_size);

        let mut chars = template.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // プレースホルダーの可能性
                let mut digits = String::new();
                let mut temp_chars = chars.clone();

                // 数字を収集
                while let Some(d) = temp_chars.next() {
                    if d.is_ascii_digit() {
                        digits.push(d);
                    } else if d == '}' && !digits.is_empty() {
                        // 正常なプレースホルダー
                        if let Ok(index) = digits.parse::<usize>() {
                            if let Some(arg) = args.get(index) {
                                result.push_str(arg);
                                chars = temp_chars;
                                break;
                            }
                        }
                        // パース失敗時はそのまま出力
                        result.push(ch);
                        break;
                    } else {
                        // プレースホルダーではない
                        result.push(ch);
                        break;
                    }
                }

                if digits.is_empty() {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// UIメッセージをフォーマット
    /// 一度の走査でO(n)で処理
    pub fn fmt_ui(&self, key: UiMsg, args: &[&str]) -> String {
        let template = self.ui(key);

        // 予想サイズを確保（テンプレート + 引数の合計長）
        let estimated_size = template.len() + args.iter().map(|s| s.len()).sum::<usize>();
        let mut result = String::with_capacity(estimated_size);

        let mut chars = template.chars();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                // プレースホルダーの可能性
                let mut digits = String::new();
                let mut temp_chars = chars.clone();

                // 数字を収集
                while let Some(d) = temp_chars.next() {
                    if d.is_ascii_digit() {
                        digits.push(d);
                    } else if d == '}' && !digits.is_empty() {
                        // 正常なプレースホルダー
                        if let Ok(index) = digits.parse::<usize>() {
                            if let Some(arg) = args.get(index) {
                                result.push_str(arg);
                                chars = temp_chars;
                                break;
                            }
                        }
                        // パース失敗時はそのまま出力
                        result.push(ch);
                        break;
                    } else {
                        // プレースホルダーではない
                        result.push(ch);
                        break;
                    }
                }

                if digits.is_empty() {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }
}

// ========================================
// グローバルインスタンス
// ========================================

use std::sync::OnceLock;
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
