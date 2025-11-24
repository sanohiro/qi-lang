use super::msg_key::MsgKey;
use super::msg_key::MsgKey::*;
use super::ui_msg::UiMsg;
use std::collections::HashMap;
use std::sync::LazyLock;

/// 英語エラーメッセージ
pub static EN_MSGS: LazyLock<HashMap<MsgKey, &'static str>> = LazyLock::new(|| {
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
        (IntegerOverflow, "integer overflow in {0} operation"),
        (IntegerUnderflow, "integer underflow in {0} operation"),
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
        (FileNotFound, "file not found: {0} ({1})"),
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
        (IntegerOutOfRange, "{0}: {1} is out of range: {2} (range: {3} to {4})"),
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
        (
            InfiniteLoopDetected,
            "infinite loop detected (iterations: {0})",
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
        (SetOperationError, "{0}"),
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
        (FloatOutOfI64Range, "{0}: float value {1} is out of i64 range (-9223372036854775808 to 9223372036854775807)"),
        (FloatIsNanOrInfinity, "{0}: cannot convert NaN or Infinity to integer"),
        (CannotConvertToJson, "Cannot convert {0} to JSON"),
        (InvalidRegex, "{0}: invalid regex: {1}"),
        (IndexOutOfBounds, "{0}: index {1} is out of bounds (length: {2})"),
        (PathIndexOutOfBounds, "{0}: path index {1} is out of bounds (path length: {2})"),
        (InvalidEnvVarName, "{0}: invalid environment variable name: {1}"),
        (UnsignedIntTooLarge, "{0}: unsigned integer {1} exceeds {2}"),
        (ValueTooLargeForI64, "{0}: value {1} is too large for i64"),
        (IntegerTooLargeForUsize, "{0}: integer {1} is too large for usize"),
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
        (
            CmdDangerousCharacters,
            "Command contains dangerous shell metacharacters: {0}. Use array format [\"cmd\" \"arg1\" \"arg2\"] to avoid shell injection.",
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
        (HttpErrorStatus, "HTTP error {0}"),
        (HttpFailedToReadBody, "Failed to read response body: {0}"),
        (HttpResponseTooLarge, "HTTP response too large (max {0} MB)"),
        (HttpInvalidHeader, "Invalid HTTP header '{0}': contains newline characters"),
        (HttpJsonSerializationError, "JSON serialization error: {0}"),
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
            ServerBodyTooLarge,
            "Request body too large: {0} bytes (max: {1} bytes)",
        ),
        (
            ServerFailedToDecompressGzip,
            "Failed to decompress gzip body: {0}",
        ),
        (
            ServerInvalidStatusCode,
            "Invalid HTTP status code: {0} (must be 100-599)",
        ),
        (
            ServerInvalidPortNumber,
            "Invalid port number: {0} (must be 0-65535)",
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
        (
            ServerFailedToCreateRuntime,
            "Failed to create Tokio runtime: {0}",
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
        // Tableエラー
        (
            TableInvalidFormat,
            "table: invalid format (expected {0})",
        ),
        (TableColumnNotFound, "table: column '{0}' not found"),
        (
            TableNoHeaders,
            "table: no headers (cannot access by column name)",
        ),
        (
            TableColumnIndexOutOfRange,
            "table: column index {0} out of range",
        ),
        (
            TableColumnSelectorInvalid,
            "table: column selector must be {0}",
        ),
        (
            TableSelectNeedsList,
            "table/select: column selectors must be {0}",
        ),
        (
            TableOrderByInvalidOrder,
            "table/order-by: order must be {0}",
        ),
        (
            TableTakeNegative,
            "table/take: n must be non-negative (got {0})",
        ),
        (TableTakeNotInteger, "table/take: n must be an integer"),
        (
            TableDropNegative,
            "table/drop: n must be non-negative (got {0})",
        ),
        (TableDropNotInteger, "table/drop: n must be an integer"),
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
        (
            ZipCreateFileFailed,
            "{0}: failed to create zip file '{1}': {2}",
        ),
        (ZipFinishFailed, "{0}: failed to finish: {1}"),
        (ZipStartFileFailed, "{0}: failed to start file: {1}"),
        (ZipOpenFileFailed, "{0}: failed to open '{1}': {2}"),
        (ZipReadFileFailed, "{0}: failed to read '{1}': {2}"),
        (ZipFileTooLarge, "{0}: file '{1}' too large (max {2} MB)"),
        (ZipWriteFailed, "{0}: failed to write: {1}"),
        (
            ZipReadDirFailed,
            "{0}: failed to read directory '{1}': {2}",
        ),
        (ZipReadEntryFailed, "{0}: failed to read entry {1}: {2}"),
        (
            ZipCreateDirFailed,
            "{0}: failed to create directory '{1}': {2}",
        ),
        (
            ZipCreateParentDirFailed,
            "{0}: failed to create parent directory: {1}",
        ),
        (ZipExtractFailed, "{0}: failed to extract: {1}"),
        (ZipUnsafePath, "{0}: unsafe path detected (path traversal): {1}"),
        (
            ZipCreateTempFailed,
            "{0}: failed to create temporary file: {1}",
        ),
        (
            ZipRemoveOriginalFailed,
            "{0}: failed to remove original: {1}",
        ),
        (
            ZipRenameTempFailed,
            "{0}: failed to rename temporary file: {1}",
        ),
        (ZipCompressFailed, "{0}: failed to compress: {1}"),
        (ZipDecompressFailed, "{0}: failed to decompress: {1}"),
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
        (
            DbPooledConnectionCannotClose,
            "Connection {0} is from a pool. Use db/pool-release instead of db/close.",
        ),
        (
            DbInvalidTimeout,
            "Invalid timeout value: {0} (must be non-negative)",
        ),
        (
            DbInvalidLimit,
            "Invalid limit value: {0} (must be non-negative)",
        ),
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
        (
            IoEncodingNotSupportedInMinimalBuild,
            "Encoding '{0}' is not supported in minimal build. Only UTF-8 is available. Enable 'encoding-extended' feature for more encodings.",
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
        (TemplateNotFound, "Template '{0}' not found"),
        (FailedToReadDirectory, "Failed to read directory: {0}"),
        (FailedToReadFile, "Failed to read file: {0}"),
        (FailedToWriteFile, "Failed to write file: {0}"),
        (FailedToCopyFile, "Failed to copy file: {0}"),
        (FailedToSetPermissions, "Failed to set file permissions: {0}"),
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

        // Upgrade
        (CheckingForUpdates, "Checking for updates..."),
        (CurrentVersion, "Current version: {0}"),
        (LatestVersion, "Latest version: {0}"),
        (AlreadyLatest, "Already using the latest version"),
        (NewVersionAvailable, "New version available: {0}"),
        (DownloadingBinary, "Downloading binary..."),
        (ExtractingBinary, "Extracting binary..."),
        (InstallingUpdate, "Installing update..."),
        (UpgradeSuccess, "Successfully upgraded to version {0}"),
        (RestartRequired, "Please restart qi to use the new version"),
        (UnsupportedPlatform, "Unsupported platform: {0}-{1}"),
        (HttpClientNotEnabled, "HTTP client not enabled (feature 'http-client' required)"),
        (UpgradeFetchFailed, "Failed to fetch release info: {0}"),
        (UpgradeGitHubApiError, "GitHub API error: {0}"),
        (UpgradeJsonParseFailed, "Failed to parse JSON: {0}"),
        (UpgradeDownloadFailed, "Download failed: {0}"),
        (UpgradeDownloadError, "Download error: {0}"),
        (UpgradeReadResponseFailed, "Failed to read response: {0}"),
        (UpgradeCreateTempDirFailed, "Failed to create temp directory: {0}"),
        (UpgradeUnpackArchiveFailed, "Failed to unpack archive: {0}"),
        (UpgradeReadTempDirFailed, "Failed to read temp directory: {0}"),
        (UpgradeReadEntryFailed, "Failed to read entry: {0}"),
        (UpgradeQiDirNotFound, "qi directory not found in archive"),
        (UpgradeGetExePathFailed, "Failed to get current exe path: {0}"),
        (UpgradeGetParentDirFailed, "Failed to get parent directory of current executable"),
        (UpgradeNotQiDirectory, "Current installation directory does not look like a qi directory: {0}"),
        (UpgradeRemoveBackupFailed, "Failed to remove old backup: {0}"),
        (UpgradeBackupFailed, "Failed to backup current qi directory: {0}"),
        (UpgradeGetMetadataFailed, "Failed to get metadata: {0}"),
        (UpgradeSetPermissionsFailed, "Failed to set permissions: {0}"),
        (UpgradeCreateDirFailed, "Failed to create directory {0}: {1}"),
        (UpgradeReadDirFailed, "Failed to read directory {0}: {1}"),
        (UpgradeCopyFileFailed, "Failed to copy {0}: {1}"),
        (UpgradeNoBinaryForPlatform, "No binary found for platform pattern: {0}"),

        // WebSocketエラー
        (WsFailedToSend, "Failed to send WebSocket message: {0}"),
        (WsConnectionClosed, "WebSocket connection is closed"),
        (WsUnexpectedFrame, "Received unexpected frame"),
        (WsFailedToClose, "Failed to close WebSocket connection: {0}"),
        (
            WsConnectionAlreadyClosed,
            "WebSocket connection is already closed",
        ),
        (
            WsFailedToConnect,
            "Failed to connect to WebSocket server: {0}",
        ),
        (WsConnectionNotFound, "WebSocket connection not found"),

        // HTTP詳細エラー
        (HttpUnexpectedErrorFormat, "Unexpected error format"),
        (HttpMissingStatus, "Missing status in response"),
        (HttpMissingBody, "Missing body in response"),
        (HttpErrorWithBody, "HTTP error {0}: {1}"),
        (HttpUnexpectedResponse, "Unexpected response format"),

        // 静的ファイルエラー
        (StaticFileInvalidPath, "Invalid path: {0}"),
        (StaticFileNotFound, "File not found: {0}"),
        (
            StaticFileMetadataFailed,
            "Failed to read file metadata: {0}",
        ),
        (StaticFileInvalidUrlEncoding, "Invalid URL encoding"),
        (StaticFileInvalidPathTraversal, "Invalid path"),
        (StaticFileInvalidBaseDir, "Invalid base directory"),
        (StaticFileInvalidFileName, "Invalid file name"),
        (StaticFileBaseDirNotExist, "Base directory does not exist"),
        (StaticFilePathTraversal, "Path traversal detected"),
        (
            StaticFileFailedToGetCwd,
            "Failed to get current directory: {0}",
        ),
        (StaticFileInvalidFilePath, "Invalid file path: {0}"),
        (StaticFileInvalidEncoding, "Invalid file path encoding"),
        (
            StaticFileFailedToCanonicalize,
            "Failed to canonicalize directory: {0}",
        ),

        // DAPサーバーエラー
        (DapUnknownCommand, "Unknown command: {0}"),
        (DapNoProgramSpecified, "No program specified"),
        (DapInvalidLaunchArgs, "Invalid launch arguments"),
        (DapInvalidArguments, "Invalid arguments"),
        (DapMissingExpression, "Missing 'expression' argument"),
        (DapFailedToWriteStdin, "Failed to write to stdin: {0}"),
        (DapFailedToReadFile, "Failed to read file: {0}"),
        (DapFailedToRedirectStdio, "Failed to redirect stdio: {0}"),
        (DapParserInitError, "Parser error: {0}"),
        (DapRuntimeError, "Runtime error: {0}"),
        (DapTaskJoinError, "Task join error: {0}"),
        // Binary data errors
        (ByteOutOfRange, "Byte at index {0} out of range (0-255): {1}"),
        (BytesMustBeIntegers, "All elements must be integers, got {1} at index {0}"),
    ])
});

/// 英語UIメッセージ
pub static EN_UI_MSGS: LazyLock<HashMap<UiMsg, &'static str>> = LazyLock::new(|| {
    use UiMsg::*;
    HashMap::from([
        // REPL
        (ReplWelcome, "Qi REPL v{0}"),
        (
            ReplPressCtrlC,
            "Press Ctrl+D to exit, Ctrl+C to cancel input",
        ),
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
        (
            ReplCommandDoc,
            "  :doc <name>        Show documentation for a function",
        ),
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
        (
            OptNew,
            "    new <name> [-t <template>]  Create a new Qi project",
        ),
        (
            OptTemplate,
            "    template <list|info>        Template management",
        ),
        (
            OptDap,
            "    --dap                       Start Debug Adapter Protocol server",
        ),
        // ヘルプ例
        (ExampleStartRepl, "  qi                 Start REPL"),
        (ExampleRunScript, "  qi script.qi       Run script"),
        (ExampleExecuteCode, "  qi -e '(+ 1 2)'    Execute code"),
        (ExampleStdin, "  echo '(+ 1 2)' | qi -s"),
        (
            ExampleLoadFile,
            "  qi -l prelude.qi   Load file and start REPL",
        ),
        (
            ExampleNewProject,
            "    qi new my-project        Create a new project",
        ),
        (
            ExampleNewHttpServer,
            "    qi new myapi -t http-server  Create an HTTP server project",
        ),
        (
            ExampleTemplateList,
            "    qi template list         List available templates",
        ),
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
        (
            DapStdinInstructions,
            "   Enter '.stdin <text>' in the debug console\n",
        ),
        (DapStdinSent, "✓ Sent to stdin: {0}"),
        // Project creation
        (ProjectNewNeedName, "Error: Please specify a project name"),
        (
            ProjectNewUsage,
            "Usage: qi new <project-name> [--template <template>]",
        ),
        (ProjectNewUnknownOption, "Error: Unknown option: {0}"),
        (ProjectNewError, "Error: {0}"),
        (ProjectCreated, "\nNew Qi project created: {0}"),
        (ProjectNextSteps, "\nNext steps:"),
        (ProjectCreating, "Creating new Qi project\n"),
        (PromptProjectName, "Project name"),
        (PromptVersion, "Version"),
        (PromptDescription, "Description"),
        (PromptAuthor, "Author"),
        (PromptLicense, "License"),
        (PromptOptional, "(optional)"),
        (TemplateNeedSubcommand, "Error: Please specify a subcommand"),
        (TemplateUsage, "Usage: qi template <list|info>"),
        (TemplateNeedName, "Error: Please specify a template name"),
        (TemplateInfoUsage, "Usage: qi template info <name>"),
        (TemplateUnknownSubcommand, "Error: Unknown subcommand: {0}"),
        (TemplateNoTemplates, "No templates available"),
        (TemplateAvailable, "Available templates:"),
        (TemplateNoInfo, "(no information)"),
        (TemplateInfoTemplate, "Template: {0}"),
        (TemplateInfoDescription, "Description: {0}"),
        (TemplateInfoAuthor, "Author: {0}"),
        (TemplateInfoVersion, "Version: {0}"),
        (TemplateInfoRequired, "Required features: {0}"),
        (TemplateInfoLocation, "Location: {0}"),
        // REPL documentation
        (ReplDocUsage, "Usage: :doc <name>"),
        (ReplDocNotFound, "No such function or variable: {0}"),
        (ReplDocParameters, "\nParameters:"),
        (ReplDocExamples, "\nExamples:"),
        (ReplDocNoDoc, "(no documentation available)"),
    ])
});
