import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration('klar');
  const lspPath = config.get<string>('lsp.path', 'klar-lsp');

  const serverOptions: ServerOptions = {
    run: { command: lspPath, transport: TransportKind.stdio },
    debug: { command: lspPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'klar' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.klar'),
    },
  };

  client = new LanguageClient(
    'klar-lsp',
    'Klar Language Server',
    serverOptions,
    clientOptions
  );

  client.start();

  // Register format on save
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider('klar', {
      provideDocumentFormattingEdits(
        document: vscode.TextDocument
      ): vscode.TextEdit[] {
        // Will call klar fmt via the LSP in a future version
        return [];
      },
    })
  );

  vscode.window.showInformationMessage('Klar Language Server activated');
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
