import * as path from 'path';
import * as vscode from 'vscode';
import { LanguageClient, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration('ubnfcStax');
  const java = config.get<string>('java.path') ?? 'java';
  const configured = config.get<string>('server.jarPath');
  const jar = configured || context.asAbsolutePath(path.join('server-dist', 'ubnfc-stax-lsp-server.jar'));
  const args = [...(config.get<string[]>('server.jvmArgs') ?? []), '-jar', jar];
  const serverOptions: ServerOptions = {
    run: { command: java, args, transport: TransportKind.stdio },
    debug: { command: java, args, transport: TransportKind.stdio }
  };
  client = new LanguageClient('ubnfcStax', 'stax (ubnfc)', serverOptions, {
    documentSelector: [{ scheme: 'file', language: 'stax' }],
    outputChannelName: 'stax (ubnfc)'
  });
  context.subscriptions.push({ dispose: () => void client?.stop() });
  void client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
