package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;
import static org.unlaxer.ubnfc.java.IrFixture.obj;
import static org.unlaxer.ubnfc.java.ScanEquivalenceTest.observe;
import static org.unlaxer.ubnfc.java.ScanEquivalenceTest.scanning;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.unlaxer.ubnfc.java.emit.JavaEmitter;
import org.unlaxer.ubnfc.java.ir.IrReader;
import org.unlaxer.ubnfc.java.ir.Json;

class Ll1EquivalenceTest {
    private static final Path JSON = Path.of("../ir/examples/json-v2.ir.json");
    private static CompilerHarness json;

    @BeforeAll static void compile() throws Exception {
        json = new CompilerHarness(IrReader.read(JSON), "ll1.generated.json");
    }

    @AfterAll static void close() throws Exception { if (json != null) json.close(); }

    private static Object enabled(CompilerHarness h, Object options, boolean value) throws Exception {
        return h.options.getMethod("withLl1", boolean.class).invoke(options, value);
    }

    private static Object recognize(CompilerHarness h, boolean full, int depth) throws Exception {
        return h.options.getConstructor(boolean.class, boolean.class, boolean.class, boolean.class, Map.class,
                int.class, boolean.class, boolean.class)
            .newInstance(false, full, false, true, Map.of(), depth, false, false);
    }

    private static void equivalent(CompilerHarness h, String input, Object options) throws Exception {
        assertEquals(observe(h, input, enabled(h, options, false)), observe(h, input, enabled(h, options, true)),
            () -> "認識表の on/off で観測が違う: " + input + " / " + options);
    }

    @Test void jsonRootCoversAllStatesAndRuns() throws Exception {
        var table = json.parser.getDeclaredField("LL1");
        table.setAccessible(true);
        Object[] states = (Object[]) table.get(null);
        assertEquals(IrReader.read(JSON).semantics().expressions().size(), states.length);
        for (Object state : states) assertNotNull(state);
        var machine = json.loader.loadClass(json.pkg + ".internal.Ll1Machine");
        var run = machine.getDeclaredMethod("run", states.getClass(), int.class, char[].class, int.class, int.class, boolean.class);
        run.setAccessible(true);
        char[] input = " [ \"😀\", true, 1e3 ] ".toCharArray();
        assertEquals(input.length, run.invoke(null, states, 0, input, 0, 9999, true));
        assertEquals(input.length, run.invoke(null, states, 0, input, 0, 9999, false));
        assertEquals(-1, run.invoke(null, states, 0, input, 0, 1, false));
    }

    @Test void jsonCorpusAndResourceLimits() throws Exception {
        for (boolean scan : List.of(false, true)) {
            scanning(json, scan);
            Object full = recognize(json, true, 10_000);
            for (String input : ScanEquivalenceTest.seeds("json")) equivalent(json, input, full);
            String deep = "[".repeat(80) + "0" + "]".repeat(80);
            assertTrue(ok(json.parse(deep, enabled(json, full, true))));
            equivalent(json, deep, full);
            Object ast = json.options.getField("DEFAULT").get(null);
            for (String input : List.of("[ \"é😀\", -0.23e+4, null ]", "[1,]", "\"\\u12Q4\"", "01"))
                equivalent(json, input, ast);
            for (int depth : List.of(1, 2, 5, 8, 15, 32, 120)) {
                Object limited = recognize(json, true, depth);
                for (String input : List.of("0", "[1]", "[[[[[[1]]]]]]", deep,
                    "\"😀\"", "{\"a\": [\"\"     ] }", "{\"a\": [\"\"     ] }!")) equivalent(json, input, limited);
            }
        }
    }

    @Test void staleAndUnsupportedTablesUseReferencePath() throws Exception {
        var root = Json.object(Json.read(Files.readString(JSON)));
        var plan = Json.object(root.get("plan"));
        root.put("plan", plan);
        var states = new ArrayList<Object>(Json.array(plan.remove("ll1")));
        assertFalse(parserSource(root).contains("recognizeLl1"));

        var first = Json.object(states.getFirst());
        Object original = first.put("operation", obj("kind", "futureInstruction"));
        states.set(0, first);
        plan.put("ll1", states);
        assertTrue(IrReader.read(Json.write(root)).plan().ll1().isEmpty());
        assertFalse(parserSource(root).contains("recognizeLl1"));

        first.put("operation", obj("kind", "sequence", "targets", List.of(123456)));
        assertFalse(parserSource(root).contains("recognizeLl1"));
        first.put("operation", original);
        var memo = Json.object(plan.get("memo"));
        plan.put("memo", memo);
        var entries = new ArrayList<Object>(Json.array(memo.get("entries")));
        memo.put("entries", entries);
        var rootMemo = Json.object(entries.getFirst());
        rootMemo.put("cacheSuccess", true);
        entries.set(0, rootMemo);
        assertFalse(parserSource(root).contains("recognizeLl1(LL1, 0,"));

        var p4 = IrReader.read(Path.of("../ir/frozen/tinyexpression-p4-f86ce8a5.ir.json"));
        assertFalse(p4.plan().ll1().isEmpty());
        assertTrue(new JavaEmitter(p4, "ll1.generated.p4").sources().values().stream()
            .noneMatch(source -> source.contains("recognizeLl1(LL1,")));
    }

    @Test void reorderedTableUsesExpressionIds() throws Exception {
        var root = Json.object(Json.read(Files.readString(JSON)));
        var plan = Json.object(root.get("plan"));
        root.put("plan", plan);
        var original = IrReader.read(JSON).plan().ll1();
        var rows = new ArrayList<Object>(Json.array(plan.get("ll1")));
        int last = rows.size() - 1;
        for (int i = 0; i <= last; i++) {
            var row = Json.object(rows.get(i));
            var op = Json.object(row.get("operation"));
            row.put("operation", op);
            var operation = original.get(i).operation();
            switch (operation.kind()) {
                case "sequence" -> op.put("targets", operation.targets().stream().map(target -> last - target).toList());
                case "choice" -> {
                    var transitions = new ArrayList<Map<String, Object>>();
                    var raw = Json.array(op.get("transitions"));
                    for (int j = 0; j < raw.size(); j++) {
                        var edge = Json.object(raw.get(j));
                        edge.put("target", last - operation.transitions().get(j).target());
                        transitions.add(edge);
                    }
                    op.put("transitions", transitions);
                }
                case "optional", "repeat", "call" -> op.put("target", last - operation.target());
                default -> { }
            }
            rows.set(i, row);
        }
        plan.put("ll1", rows.reversed());
        assertTrue(parserSource(root).contains("recognizeLl1(LL1, " + last + ","));
    }

    private static String parserSource(Map<String, Object> root) throws Exception {
        return new JavaEmitter(IrReader.read(Json.write(root)), "ll1.generated.json")
            .sources().get("ll1/generated/json/JsonParser.java");
    }

    @Test void wordBoundaryAndPartialRollback() throws Exception {
        var f = new IrFixture();
        String word = f.literal("Root", "foo", true);
        String emoji = f.literal("Root", "😀");
        String alt = f.choice("Root", "ordered", word, emoji);
        String a = f.literal("Root", "a");
        String b = f.literal("Root", "b");
        String pair = f.seq("Root", a, b);
        String optional = f.expression("Root", "optional", "child", pair);
        String repeat = f.expression("Root", "repeat", "child", pair, "min", 0, "max", 2, "terminator", null);
        String root = f.seq("Root", alt, optional, repeat);
        f.rule("Root", root);
        omitMemo(f, root);
        f.plan.put("ll1", List.of(
            state(root, true, "sequence", "targets", List.of(1, 4, 8)),
            state(alt, false, "choice", "transitions", List.of(edge('f', 2), edge(0x1f600, 3))),
            state(word, false, "literal", "value", "foo", "wordBoundary", true),
            state(emoji, false, "literal", "value", "😀", "wordBoundary", false),
            state(optional, false, "optional", "target", 5),
            state(pair, false, "sequence", "targets", List.of(6, 7)),
            state(a, false, "literal", "value", "a", "wordBoundary", false),
            state(b, false, "literal", "value", "b", "wordBoundary", false),
            state(repeat, false, "repeat", "target", 5, "min", 0, "max", 2)));
        try (var h = f.compile()) {
            assertNotNull(h.parser.getDeclaredField("LL1"));
            Object prefix = recognize(h, false, 10_000);
            for (String input : List.of("foo", "foo1", "foo_", "foo-é", "foo😀", "😀a", "😀aba", "😀abababa", "😀abababab",
                "\ud83d", "\ude00", "😀abracadabra")) {
                equivalent(h, input, prefix);
                equivalent(h, input, recognize(h, true, 10_000));
            }
            var result = h.parse("😀aba", enabled(h, prefix, true));
            assertTrue(ok(result));
            assertEquals(3, get(result, "consumedCp"));
            assertEquals(3, get(result, "matchedCp"));
            assertFalse(ok(h.parse("foo1", enabled(h, prefix, true))));
        }
    }

    @Test void skippedChoiceStillRespectsRuleDepth() throws Exception {
        var f = new IrFixture();
        String other = f.literal("Other", "y");
        String call = f.ref("Root", "Other");
        String x = f.literal("Root", "x");
        String root = f.choice("Root", "ordered", call, x);
        f.rule("Root", root);
        f.rule("Other", other);
        omitMemo(f, root);
        omitMemo(f, other);
        f.plan.put("ll1", List.of(
            state(root, true, "choice", "transitions", List.of(edge('y', 1), edge('x', 3))),
            state(call, false, "call", "target", 2),
            state(other, true, "literal", "value", "y", "wordBoundary", false),
            state(x, false, "literal", "value", "x", "wordBoundary", false)));
        try (var h = f.compile()) {
            assertNotNull(h.parser.getDeclaredField("LL1"));
            Object limited = recognize(h, true, 1);
            assertFalse(ok(h.parse("x", limited)));
            equivalent(h, "x", limited);
            equivalent(h, "x", recognize(h, true, 2));
        }
    }

    private static void omitMemo(IrFixture f, String body) {
        f.memos.add(obj("exprId", body, "cacheSuccess", false, "cacheFailure", false,
            "omissionProof", obj("kind", "unreachableRevisit", "exprIds", List.of(body), "explanation", "各規則を一度だけ呼ぶ"),
            "reason", "provedOmission"));
    }

    private static Map<String, Object> edge(int cp, int target) {
        return obj("ranges", List.of(obj("start", cp, "end", cp)), "target", target);
    }

    private static Map<String, Object> state(String id, boolean root, String kind, Object... fields) {
        var op = obj("kind", kind);
        op.putAll(obj(fields));
        return obj("exprId", id, "root", root, "operation", op);
    }
}
