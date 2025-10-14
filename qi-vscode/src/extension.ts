import * as vscode from 'vscode';
import * as cp from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

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
  const dir = path.dirname(document.uri.fsPath);
  const tmpFileName = `.qi-format-${Date.now()}-${Math.random()
    .toString(16)
    .slice(2)}${path.extname(document.uri.fsPath) || '.qi'}`;
  const tempPath = path.join(dir, tmpFileName);

  try {
    fs.writeFileSync(tempPath, text, 'utf8');

    const result = cp.execFileSync(qiPath, ['fmt', '--check', tempPath], {
      encoding: 'utf8',
      timeout: 10000,
      cwd: dir
    });

    // フォーマット結果で置換
    const fullRange = new vscode.Range(
      document.positionAt(0),
      document.positionAt(text.length)
    );

    return [vscode.TextEdit.replace(fullRange, result)];
  } catch (error) {
    const err = error as { stderr?: unknown; message?: string };
    const stderr = (err.stderr ?? '').toString();
    const message = stderr.trim() || err.message || 'Unknown error';
    vscode.window.showErrorMessage(`Failed to format: ${message}`);
    return [];
  } finally {
    try {
      fs.rmSync(tempPath, { force: true });
    } catch (rmError) {
      console.debug('Failed to remove qi temp file', rmError);
    }
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

export function deactivate(): void {
  console.log('Qi Language Extension deactivated');
}
