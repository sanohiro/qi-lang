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
    FileNotFound, // file not found: {0} ({1})

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
    SetOperationError, // {0}: set operation error

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

    // Table エラー
    TableInvalidFormat,         // table: invalid format (expected {0})
    TableColumnNotFound,        // table: column '{0}' not found
    TableNoHeaders,             // table: no headers (cannot access by column name)
    TableColumnIndexOutOfRange, // table: column index {0} out of range
    TableColumnSelectorInvalid, // table: column selector must be {0}
    TableSelectNeedsList,       // table/select: column selectors must be {0}
    TableOrderByInvalidOrder,   // table/order-by: order must be {0}
    TableTakeNegative,          // table/take: n must be non-negative (got {0})
    TableTakeNotInteger,        // table/take: n must be an integer
    TableDropNegative,          // table/drop: n must be non-negative (got {0})
    TableDropNotInteger,        // table/drop: n must be an integer

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
    HttpErrorStatus,           // HTTP error {0}

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
    ServerFailedToCreateRuntime, // Failed to create Tokio runtime: {0}

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
    IoEncodingNotSupportedInMinimalBuild, // Encoding '{0}' is not supported in minimal build. Only UTF-8 is available. Enable 'encoding-extended' feature for more encodings.

    // Featureエラー
    FeatureDisabled,     // {0} support is disabled. Build with feature '{1}': {2}
    DbUnsupportedDriver, // Unsupported database driver: {0}

    // Markdownエラー
    MdHeaderInvalidLevel,  // markdown/header: level must be 1-6, got: {0}
    MdTableEmpty,          // markdown/table: table must not be empty
    MdTableRowMustBeList,  // markdown/table: row {0} must be a list
    MdTableColumnMismatch, // markdown/table: row {0} has {1} columns, expected {2}

    // DAPデバッガーエラー
    DapEmptyExpression,      // Empty expression
    DapEvaluationError,      // Evaluation error: {0}
    DapParseError,           // Parse error: {0}
    DapNoEnvironment,        // No environment available (not stopped at breakpoint)
    DapDebuggerNotAvailable, // Debugger not available
    DapServerError,          // DAP server error: {0}
    DapServerNotEnabled,     // Error: DAP server is not enabled. Build with --features dap-server
    InternalError,           // Internal error: {0}

    // プロジェクト管理エラー
    QiTomlFailedToRead,        // Failed to read qi.toml: {0}
    QiTomlFailedToParse,       // Failed to parse qi.toml: {0}
    QiTomlFailedToSerialize,   // Failed to serialize qi.toml: {0}
    QiTomlFailedToWrite,       // Failed to write qi.toml: {0}
    FailedToGetCurrentDir,     // Failed to get current directory: {0}
    DirectoryAlreadyExists,    // Directory '{}' already exists
    FailedToCreateDirectory,   // Failed to create directory: {0}
    TemplateNotFound,          // Template '{}' not found
    FailedToReadDirectory,     // Failed to read directory: {0}
    FailedToReadFile,          // Failed to read file: {0}
    FailedToWriteFile,         // Failed to write file: {0}
    TemplateTomlFailedToRead,  // Failed to read template.toml: {0}
    TemplateTomlFailedToParse, // Failed to parse template.toml: {0}

    // 評価器エラー
    TypeErrorVectorPattern, // Type error: cannot pass {0} to vector pattern
    ArgErrorVectorPatternMinimum, // Argument error: vector pattern expects at least {0} elements, but got {1}
    ArgErrorVectorPattern, // Argument error: vector pattern expects {0} elements, but got {1}
    TypeErrorMapPattern,   // Type error: cannot pass {0} to map pattern
    KeyErrorMapMissing,    // Key error: map does not have key :{0}

    // データベース・KVSエラー
    ConnectionError,                  // Connection error: {0}
    ConnectionNotFound,               // Connection not found: {0}
    EvalError,                        // eval: {0}
    FailedToCreateRuntime,            // Failed to create runtime: {0}
    FailedToExecuteColumnsQuery,      // Failed to execute columns query: {0}
    FailedToExecuteForeignKeysQuery,  // Failed to execute foreign keys query: {0}
    FailedToExecuteIndexColumnsQuery, // Failed to execute index columns query: {0}
    FailedToExecuteIndexesQuery,      // Failed to execute indexes query: {0}
    FailedToExecuteTablesQuery,       // Failed to execute tables query: {0}
    FailedToGetColumnName,            // Failed to get column name: {0}
    FailedToGetColumnValue,           // Failed to get column value: {0}
    FailedToGetDatabaseVersion,       // Failed to get database version: {0}
    FailedToPrepareStatement,         // Failed to prepare statement: {0}
    FailedToQueryColumns,             // Failed to query columns: {0}
    FailedToQueryForeignKeys,         // Failed to query foreign keys: {0}
    FailedToQueryIndexColumns,        // Failed to query index columns: {0}
    FailedToQueryIndexes,             // Failed to query indexes: {0}
    FailedToQueryTables,              // Failed to query tables: {0}
    FailedToReadFileMetadata,         // Failed to read file metadata: {0}
    InvalidIsolationLevel,            // Invalid isolation level: {0}
    UnsupportedKvsUrl,                // Unsupported KVS URL: {0}
    UnsupportedUrl,                   // Unsupported URL: {0}
    RedisSupportNotEnabled,           // Redis support not enabled (feature 'kvs-redis' required)
    InvalidConnection,                // Invalid connection (expected KvsConnection:xxx)
    QiTomlAlreadyExists,              // qi.toml already exists
    PatternErrorNotAllowed, // Pattern error: this pattern cannot be used in function parameters or let bindings (only in match)
    UnexpectedResponse,     // Unexpected response

    // Upgrade
    CheckingForUpdates,   // Checking for updates...
    CurrentVersion,       // Current version: {0}
    LatestVersion,        // Latest version: {0}
    AlreadyLatest,        // Already using the latest version
    NewVersionAvailable,  // New version available: {0}
    DownloadingBinary,    // Downloading binary...
    InstallingUpdate,     // Installing update...
    UpgradeSuccess,       // Successfully upgraded to version {0}
    RestartRequired,      // Please restart qi to use the new version
    UnsupportedPlatform,  // Unsupported platform: {0}-{1}
    HttpClientNotEnabled, // HTTP client not enabled (feature 'http-client' required)
}
