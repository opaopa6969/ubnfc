package org.ubnfc.java;

import java.io.IOException;
import java.nio.file.Path;
import org.ubnfc.java.emit.JavaEmitter;
import org.ubnfc.java.ir.IrReader;

/** Grammar IR to standalone JDK 21 Java source files. */
public final class Main {
    private Main() {}
    public static void main(String[] args) throws IOException {
        if (args.length == 0 || args.length == 1 && args[0].equals("--help")) {
            System.out.println("usage: ubnfc-java --ir <grammar.ir.json> --out <dir> [--package x.y]"); return;
        }
        String ir = null, out = null, pkg = null;
        for (int i = 0; i < args.length; i += 2) {
            if (i + 1 == args.length) throw new IllegalArgumentException("Missing value for " + args[i]);
            switch (args[i]) {
                case "--ir" -> { if (ir != null) throw new IllegalArgumentException("Duplicate --ir"); ir = args[i + 1]; }
                case "--out" -> { if (out != null) throw new IllegalArgumentException("Duplicate --out"); out = args[i + 1]; }
                case "--package" -> { if (pkg != null) throw new IllegalArgumentException("Duplicate --package"); pkg = args[i + 1]; }
                default -> throw new IllegalArgumentException("Unknown option: " + args[i]);
            }
        }
        if (ir == null || out == null) throw new IllegalArgumentException("--ir and --out are required");
        new JavaEmitter(IrReader.read(Path.of(ir)), pkg).write(Path.of(out));
    }
}
