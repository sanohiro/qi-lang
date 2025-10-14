import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
  console.log('Qi Language Extension is now active');

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
    vscode.window.showInformationMessage('Qi debugger is not yet implemented');
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
