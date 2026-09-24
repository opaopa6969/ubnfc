import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

/** 既定は拡張に同梱した fat jar。設定で差し替えられる。 */
function jarPath(context: vscode.ExtensionContext): string {
  const configured = vscode.workspace.getConfiguration('ubnfcUbnf').get<string>('server.jarPath');
  if (configured && configured.length > 0) {
    return configured;
  }
  return context.asAbsolutePath(path.join('server-dist', 'ubnfc-ubnf-lsp-server.jar'));
}

/**
 * DAP（文法デバッガ）。LSP と同じ fat jar を使い、こちらは -cp + メインクラス指定で起こす
 * （LSP は -jar）。旧スタックの tinycalc-vscode / tinyexpression-p4-lsp-vscode と同じ形。
 *
 * この拡張のデバッガは `scripts/build-vsix.sh --with-dap` で組んだ VSIX でのみ動く
 * （DAP は opt-in の生成物なので、付けずに組んだ jar には DapMain が入っていない）。
 */
class GrammarDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
  constructor(private readonly context: vscode.ExtensionContext) {}

  createDebugAdapterDescriptor(): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    const config = vscode.workspace.getConfiguration('ubnfcUbnf');
    const java = config.get<string>('java.path') ?? 'java';
    const jvmArgs = config.get<string[]>('server.jvmArgs') ?? [];
    return new vscode.DebugAdapterExecutable(java, [
      ...jvmArgs,
      '-cp',
      jarPath(this.context),
      'org.ubnfc.ubnf.lsp.dap.DapMain'
    ]);
  }
}

export function activate(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration('ubnfcUbnf');
  const java = config.get<string>('java.path') ?? 'java';
  const jvmArgs = config.get<string[]>('server.jvmArgs') ?? [];
  const args = [...jvmArgs, '-jar', jarPath(context)];

  // 生成サーバは stdio で JSON-RPC を話す（ServerMain）。
  const serverOptions: ServerOptions = {
    run: { command: java, args, transport: TransportKind.stdio },
    debug: { command: java, args, transport: TransportKind.stdio }
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'ubnf' }],
    outputChannelName: 'UBNF (ubnfc)'
  };

  context.subscriptions.push(vscode.debug.registerDebugAdapterDescriptorFactory(
    'ubnf', new GrammarDebugAdapterFactory(context)));

  client = new LanguageClient('ubnfcUbnf', 'UBNF (ubnfc)', serverOptions, clientOptions);
  context.subscriptions.push({ dispose: () => void client?.stop() });
  void client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
