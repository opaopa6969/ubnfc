package org.ubnfc.java;

/** ubnfc-java: Grammar IR (JSON) を読み、Java パーサソースを生成する。JSON は自前の最小パーサで読む（依存ゼロ）。 */
public final class Main {
  private Main() {}

  public static void main(String[] args) {
    if (args.length == 0 || args[0].equals("--help")) {
      System.out.print("usage: ubnfc-java --ir <grammar.ir.json> --out <dir>\n");
      return;
    }
    System.err.println("ubnfc-java: not implemented yet");
    System.exit(2);
  }
}
