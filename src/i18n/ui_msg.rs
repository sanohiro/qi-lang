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
    ProjectNewNeedName,        // エラー: プロジェクト名を指定してください
    ProjectNewUsage,           // 使い方: qi new <project-name> [--template <template>]
    ProjectNewUnknownOption,   // エラー: 不明なオプション: {0}
    ProjectNewError,           // エラー: {0}
    ProjectCreated,            // 新しいQiプロジェクトが作成されました: {0}
    ProjectNextSteps,          // 次のステップ:
    ProjectCreating,           // 新しいQiプロジェクトを作成します
    PromptProjectName,         // プロジェクト名
    PromptVersion,             // バージョン
    PromptDescription,         // 説明
    PromptAuthor,              // 著者名
    PromptLicense,             // ライセンス
    PromptOptional,            // (省略可)
    TemplateNeedSubcommand,    // エラー: サブコマンドを指定してください
    TemplateUsage,             // 使い方: qi template <list|info>
    TemplateNeedName,          // エラー: テンプレート名を指定してください
    TemplateInfoUsage,         // 使い方: qi template info <name>
    TemplateUnknownSubcommand, // エラー: 不明なサブコマンド: {0}
    TemplateNoTemplates,       // 利用可能なテンプレートがありません
    TemplateAvailable,         // 利用可能なテンプレート:
    TemplateNoInfo,            // (情報なし)
    TemplateInfoTemplate,      // Template: {0}
    TemplateInfoDescription,   // Description: {0}
    TemplateInfoAuthor,        // Author: {0}
    TemplateInfoVersion,       // Version: {0}
    TemplateInfoRequired,      // Required features: {0}
    TemplateInfoLocation,      // Location: {0}

    // REPL
    ReplDocUsage,      // Usage: :doc <name>
    ReplDocNotFound,   // No such function or variable: {0}
    ReplDocParameters, // Parameters:
    ReplDocExamples,   // Examples:
    ReplDocNoDoc,      // (no documentation available)
}
