import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';

export function activate(context: vscode.ExtensionContext) {
  console.log('Qi Language Extension is now active');

  // フォーマッタープロバイダーの登録
  const formattingProvider = vscode.languages.registerDocumentFormattingEditProvider('qi', {
    provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
      return formatDocument(document);
    }
  });

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

  const formatDocumentCommand = vscode.commands.registerCommand('qi.formatDocument', () => {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
      vscode.window.showErrorMessage('No active Qi file');
      return;
    }

    vscode.commands.executeCommand('editor.action.formatDocument');
  });

  const showDocsCommand = vscode.commands.registerCommand('qi.showDocs', () => {
    vscode.env.openExternal(vscode.Uri.parse('https://github.com/sanohiro/qi-lang'));
  });

  context.subscriptions.push(
    formattingProvider,
    runFileCommand,
    startReplCommand,
    debugFileCommand,
    formatDocumentCommand,
    showDocsCommand
  );
}

function formatDocument(document: vscode.TextDocument): vscode.TextEdit[] {
  const config = vscode.workspace.getConfiguration('qi');
  const enableFormatting = config.get<boolean>('enableFormatting', true);

  if (!enableFormatting) {
    return [];
  }

  const qiPath = config.get<string>('executablePath', 'qi');
  const text = document.getText();

  try {
    // 一時ファイルに書き込んで qi fmt を実行
    const tempFile = document.uri.fsPath;
    const result = cp.execSync(`${qiPath} fmt --check "${tempFile}"`, {
      encoding: 'utf-8',
      timeout: 10000
    });

    // フォーマット結果で置換
    const fullRange = new vscode.Range(
      document.positionAt(0),
      document.positionAt(text.length)
    );

    return [vscode.TextEdit.replace(fullRange, result)];
  } catch (error: any) {
    vscode.window.showErrorMessage(`Failed to format: ${error.message}`);
    return [];
  }
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

export function deactivate() {}
