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
            System.out.println("usage: ubnfc-java --ir <grammar.ir.json> --out <dir> [--package x.y] [--emit-driver [true|false]] [--no-emit-driver]"); return;
        }
        String ir = null, out = null, pkg = null;
        boolean emitDriver = true, driverSeen = false;
        for (int i = 0; i < args.length; i++) {
            String option = args[i];
            if (option.equals("--emit-driver") || option.equals("--no-emit-driver")) {
                if (driverSeen) throw new IllegalArgumentException("Duplicate driver option");
                driverSeen = true; emitDriver = option.equals("--emit-driver");
                if (emitDriver && i + 1 < args.length && !args[i + 1].startsWith("--")) {
                    String value = args[++i];
                    if (!value.equals("true") && !value.equals("false"))
                        throw new IllegalArgumentException("--emit-driver requires true or false");
                    emitDriver = Boolean.parseBoolean(value);
                }
                continue;
            }
            if (i + 1 == args.length) throw new IllegalArgumentException("Missing value for " + option);
            String value = args[++i];
            switch (option) {
                case "--ir" -> { if (ir != null) throw new IllegalArgumentException("Duplicate --ir"); ir = value; }
                case "--out" -> { if (out != null) throw new IllegalArgumentException("Duplicate --out"); out = value; }
                case "--package" -> { if (pkg != null) throw new IllegalArgumentException("Duplicate --package"); pkg = value; }
                default -> throw new IllegalArgumentException("Unknown option: " + option);
            }
        }
        if (ir == null || out == null) throw new IllegalArgumentException("--ir and --out are required");
        new JavaEmitter(IrReader.read(Path.of(ir)), pkg, emitDriver).write(Path.of(out));
    }
}
