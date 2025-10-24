import * as vscode from 'vscode';
import { ChildProcess, spawn } from 'child_process';

export function activate(context: vscode.ExtensionContext) {
  console.log('Qi Language Extension is now active');

  // デバッグアダプターファクトリーの登録
  const debugAdapterFactory = new QiDebugAdapterFactory();
  context.subscriptions.push(
    vscode.debug.registerDebugAdapterDescriptorFactory('qi', debugAdapterFactory)
  );

  // デバッグ設定プロバイダーの登録
  const debugConfigProvider = new QiDebugConfigurationProvider();
  context.subscriptions.push(
    vscode.debug.registerDebugConfigurationProvider('qi', debugConfigProvider)
  );

  // コマンドの登録
  const runFileCommand = vscode.commands.registerCommand('qi.runFile', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active Qi file');
      return;
    }

    const filePath = editor.document.uri.fsPath;
    runQiFile(filePath);
  });

  const startReplCommand = vscode.commands.registerCommand('qi.startRepl', () => {
    startRepl();
  });

  const debugFileCommand = vscode.commands.registerCommand('qi.debugFile', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active Qi file');
      return;
    }

    const filePath = editor.document.uri.fsPath;
    vscode.debug.startDebugging(undefined, {
      type: 'qi',
      name: 'Debug Qi File',
      request: 'launch',
      program: filePath
    });
  });

  const showDocsCommand = vscode.commands.registerCommand('qi.showDocs', () => {
    vscode.env.openExternal(vscode.Uri.parse('https://github.com/sanohiro/qi-lang'));
  });

  context.subscriptions.push(
    runFileCommand,
    startReplCommand,
    debugFileCommand,
    showDocsCommand
  );
}

function runQiFile(filePath: string) {
  const config = vscode.workspace.getConfiguration('qi');
  const qiPath = config.get<string>('executablePath', 'qi');

  const terminal = vscode.window.createTerminal('Qi');
  terminal.show();
  terminal.sendText(`${qiPath} "${filePath}"`);
}

function startRepl() {
  const config = vscode.workspace.getConfiguration('qi');
  const qiPath = config.get<string>('executablePath', 'qi');

  const terminal = vscode.window.createTerminal('Qi REPL');
  terminal.show();
  terminal.sendText(qiPath);
}

export function deactivate(): void {
  console.log('Qi Language Extension deactivated');
}

// ========================================
// Qi Debug Adapter Factory
// ========================================

class QiDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
  createDebugAdapterDescriptor(
    session: vscode.DebugSession,
    executable: vscode.DebugAdapterExecutable | undefined
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {

    // session.configurationからqiPathを取得（launch.jsonで指定されたもの）
    // なければ設定から取得
    let qiPath = session.configuration.qiPath;
    if (!qiPath) {
      const config = vscode.workspace.getConfiguration('qi');
      qiPath = config.get<string>('executablePath', 'qi');
    }

    // デバッグアダプターとしてqi --dapを起動
    const args = ['--dap'];

    // 常にログ出力
    console.log(`[Qi Debug] createDebugAdapterDescriptor called`);
    console.log(`[Qi Debug] qiPath: ${qiPath}`);
    console.log(`[Qi Debug] args: ${args.join(' ')}`);
    console.log(`[Qi Debug] session.configuration:`, session.configuration);

    // トレースが有効な場合
    if (session.configuration.trace) {
      console.log(`[Qi Debug] Launching debug adapter: ${qiPath} ${args.join(' ')}`);
    }

    return new vscode.DebugAdapterExecutable(qiPath, args);
  }
}

// ========================================
// Qi Debug Configuration Provider
// ========================================

class QiDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
  /**
   * launch.jsonが存在しない場合の初期設定を提供
   */
  provideDebugConfigurations(
    folder: vscode.WorkspaceFolder | undefined
  ): vscode.ProviderResult<vscode.DebugConfiguration[]> {
    return [
      {
        type: 'qi',
        request: 'launch',
        name: 'Launch Qi Program',
        program: '${file}',
        cwd: '${workspaceFolder}',
        stopOnEntry: false
      }
    ];
  }

  /**
   * デバッグ設定の解決（起動前に呼ばれる）
   */
  resolveDebugConfiguration(
    folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration,
    token?: vscode.CancellationToken
  ): vscode.ProviderResult<vscode.DebugConfiguration> {

    // programが指定されていない場合は現在のファイルを使用
    if (!config.program) {
      const editor = vscode.window.activeTextEditor;
      if (editor && editor.document.languageId === 'qi') {
        config.program = editor.document.uri.fsPath;
      } else {
        vscode.window.showErrorMessage('No Qi file is currently open');
        return undefined;
      }
    }

    // デフォルト値の設定
    if (!config.cwd) {
      config.cwd = folder ? folder.uri.fsPath : process.cwd();
    }

    if (!config.qiPath) {
      const qiConfig = vscode.workspace.getConfiguration('qi');
      config.qiPath = qiConfig.get<string>('executablePath', 'qi');
    }

    return config;
  }
}
