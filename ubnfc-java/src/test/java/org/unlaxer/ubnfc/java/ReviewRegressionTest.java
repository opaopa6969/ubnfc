package org.unlaxer.ubnfc.java;

import static org.junit.jupiter.api.Assertions.*;
import static org.unlaxer.ubnfc.java.CompilerHarness.*;
import static org.unlaxer.ubnfc.java.IrFixture.*;
import java.lang.reflect.Proxy;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;
import org.unlaxer.ubnfc.java.ir.IrReader;
import org.unlaxer.ubnfc.java.ir.Json;

/** Values from corpus/oracle and the independent Java runs in the adversarial review. */
@Timeout(60)
class ReviewRegressionTest {
    private static Map<String, Object> oracle(String name, int index) throws Exception {
        return Json.object(Json.array(Json.object(Json.read(Files.readString(Path.of("../corpus/oracle/" + name + ".json")))).get("cases")).get(index));
    }
    private static void cursors(Object result, boolean success, int c, int m) throws Exception {
        assertEquals(success, ok(result)); assertEquals(c, get(result, "consumedCp")); assertEquals(m, get(result, "matchedCp"));
    }
    private static void diagnostic(Object result, String kind, int offset, List<String> expected) throws Exception {
        Object d = get(result, "diagnostics");
        assertEquals(kind, get(d, "kind")); assertEquals(offset, get(d, "offset")); assertEquals(expected, get(d, "expected"));
    }
    private static void oracleFailure(Object result, Map<String, Object> row) throws Exception {
        var d = Json.object(Json.array(row.get("diagnostics")).getFirst());
        assertFalse(ok(result));
        diagnostic(result, (String) d.get("kind"), ((Number) d.get("offset")).intValue(),
            Json.array(d.get("expected")).stream().map(Object::toString).toList());
        assertEquals(((Number) d.get("farthestOffset")).intValue(), get(get(result, "diagnostics"), "farthestCp"));
        assertEquals(d.get("farthestExpected"), get(get(result, "diagnostics"), "farthestExpected"));
    }
    private static Map<String, Object> extern() {
        return obj("kind", "extern", "logicalId", "scan", "javaBinding", null, "rustBinding", null,
            "originalClassName", "Scanner", "inputState", List.of("consumed", "matched", "mode", "invertMatch", "resetMatchedWithConsumed"),
            "resultFields", List.of("ok", "consumedEnd", "matchedEnd", "valueSpan", "diagnostics", "effects"));
    }
    private static void display(IrFixture f, String expr, String label) {
        var d = Json.object(f.semantics.get("diagnostics"));
        d.put("expectedDisplay", List.of(obj("exprId", expr, "java", label, "rust", label))); f.semantics.put("diagnostics", d);
    }
    private static void memo(IrFixture f, String expr) {
        f.memos.add(obj("exprId", expr, "cacheSuccess", true, "cacheFailure", true, "reason", "defaultSafe", "omissionProof", null));
    }

    @Test void matchOnlyRepeatRetainsSameFrameAndOptionalCommitsFailureReset() throws Exception {
        for (int bound : List.of(2, 3)) {
            var f = new IrFixture();
            String repeat = f.expression("Root", "repeat", "child", f.expression("Root", "any"), "min", bound, "max", bound, "terminator", null);
            f.rule("Root", f.expression("Root", "lookahead", "child", repeat));
            try (var h = f.compile()) {
                for (boolean memo : List.of(false, true)) cursors(h.parse("abc", h.options(false, false, false, memo, Map.of())), true, 0, bound);
            }
        }
        var f = new IrFixture();
        String look = f.token("Root", "L", obj("kind", "lookahead", "pattern", "a"));
        f.rule("Root", f.seq("Root", look, f.expression("Root", "optional", "child", f.literal("Root", "x"))));
        try (var h = f.compile()) { cursors(h.parse("ab", h.options(false, false, false, true, Map.of())), true, 0, 0); }
    }

    @Test void emptyWordLookaheadsFollowLegacyLengthZeroFailure() throws Exception {
        for (String kind : List.of("lookahead", "negativeLookahead")) {
            var f = new IrFixture(); f.rule("Root", f.token("Root", "T", obj("kind", kind, "pattern", "")));
            try (var h = f.compile()) {
                for (String input : List.of("", "ab")) cursors(h.parse(input, h.options(false, false, false, true, Map.of())), kind.equals("negativeLookahead"), 0, 0);
            }
        }
    }

    @Test void repeatAndEofFailuresMatchOracleDiagnostics() throws Exception {
        for (int bound : List.of(0, 2)) {
            var f = new IrFixture();
            f.rule("Root", f.expression("Root", "repeat", "child", f.expression("Root", "empty"), "min", bound, "max", bound, "terminator", null));
            try (var h = f.compile()) {
                Object result = h.parse("ab"); cursors(result, false, 0, 0);
                oracleFailure(result, oracle(bound == 0 ? "empty-zero" : "empty-two", 0));
                diagnostic(result, "syntax", 0, List.of("Repeat", "RootParser"));
            }
        }
        var f = new IrFixture(); String eof = f.expression("Root", "eof");
        f.capture(eof, "value", "text", "scalar", obj("kind", "text")); f.rule("Root", eof);
        try (var h = f.compile()) {
            for (int i : List.of(1, 2)) {
                var row = oracle("eof", i); Object result = h.parse((String) row.get("input"));
                cursors(result, false, 0, 0); oracleFailure(result, row);
            }
        }
    }

    @Test void notDiscardsChildDiagnosticsOnlyWhenNotSucceedsIncludingMemoReplay() throws Exception {
        var f = new IrFixture();
        String child = f.choice("Root", "ordered", f.seq("Root", f.literal("Root", "a"), f.literal("Root", "x")), f.literal("Root", "a"));
        memo(f, child);
        f.rule("Root", f.expression("Root", "negativeLookahead", "child", child));
        try (var h = f.compile()) {
            for (boolean memo : List.of(false, true)) {
                Object result = h.parse("ab", h.options(false, false, false, memo, Map.of()));
                diagnostic(result, "syntax", 1, List.of("'x'"));
                oracleFailure(result, oracle("lookahead-failure", 0));
                Object success = h.parse("b", h.options(false, false, false, memo, Map.of()));
                cursors(success, true, 0, 0); assertEquals(List.of(), get(get(success, "diagnostics"), "farthestExpected"));
            }
        }
        // The first attempt stores a failure under Not; its later ordinary use must replay it.
        var g = new IrFixture(); String word = g.literal("Root", "x"); memo(g, word);
        g.rule("Root", g.seq("Root", g.expression("Root", "negativeLookahead", "child", word),
            g.expression("Root", "lookahead", "child", word)));
        try (var h = g.compile()) { diagnostic(h.parse("b"), "syntax", 0, List.of("'x'")); }
    }

    @Test void expectedDisplayAndPrimaryTrailingExpectedAreIndependent() throws Exception {
        for (String label : List.of("'let'", "CUSTOM_CI")) {
            var f = new IrFixture(); String ci = f.token("Root", "CI", obj("kind", "caseInsensitive", "word", "let", "comparison", "javaEqualsIgnoreCase", "wordBoundary", false));
            display(f, ci, label); f.rule("Root", ci);
            try (var h = f.compile()) {
                diagnostic(h.parse("le"), "syntax", 0, List.of(label));
                if (label.equals("'let'")) oracleFailure(h.parse("le"), oracle("ci", 3));
            }
        }
        var f = new IrFixture();
        f.rule("Root", f.choice("Root", "ordered", f.seq("Root", f.literal("Root", "a"), f.literal("Root", "b"), f.literal("Root", "x")), f.literal("Root", "a")));
        try (var h = f.compile()) {
            Object result = h.parse("abq"); diagnostic(result, "trailing_input", 1, List.of("end of input"));
            assertEquals(2, get(get(result, "diagnostics"), "farthestCp"));
            assertEquals(List.of("'x'"), get(get(result, "diagnostics"), "farthestExpected"));
        }
        var g = new IrFixture();
        String regex = g.token("Root", "RX", obj("kind", "regex", "pattern", "a*", "regularSubset", true,
            "matching", "lookingAt", "zeroWidth", "fail", "javaFallback", "java.util.regex.Pattern", "rustFallback", "generationError"));
        display(g, regex, "REGEX('a*')"); g.rule("Root", regex);
        try (var h = g.compile()) { oracleFailure(h.parse(""), oracle("regex-zero", 0)); oracleFailure(h.parse("b"), oracle("regex-zero", 1)); }
    }

    @Test void lineCommentTerminatorRetainsMatchedReach() throws Exception {
        var f = new IrFixture(); String body = f.seq("Root", f.literal("Root", "a"), f.literal("Root", "b")); f.rule("Root", body);
        var trivia = Json.object(f.semantics.get("trivia")); var policy = Json.object(Json.array(trivia.get("policies")).getFirst());
        policy.put("effectiveDelimiterSet", List.of(obj("kind", "lineComment", "open", "//", "lineEndProfile", "legacyLineTerminator")));
        trivia.put("policies", List.of(policy));
        trivia.put("insertionPlan", List.of(obj("boundary", obj("ownerExprId", body, "at", "afterChild", "childIndex", 0), "policyId", "trivia:javaStyle")));
        f.semantics.put("trivia", trivia);
        try (var h = f.compile()) {
            oracleFailure(h.parse("a//x\nb"), oracle("comment-line", 1));
            oracleFailure(h.parse("a#x\nb"), oracle("comment-line", 2));
            diagnostic(h.parse("a//x\nb"), "syntax", 5, List.of("'//'", "'b'"));
            diagnostic(h.parse("a//x\r\nb"), "syntax", 6, List.of("'//'", "'b'"));
        }
    }

    @Test void selfHostAnnotationsGoldenRetainsEmptyAndEveryOccurrence() throws Exception {
        try (var h = new CompilerHarness(IrReader.read(Path.of("../ir/fixtures/unlaxer-dsl__grammar__ubnf.ubnf.ir.json")), "test.selfhost")) {
            for (Object item : Json.array(Json.read(Files.readString(Path.of("src/test/resources/golden/selfhost-annotations.json"))))) {
                var row = Json.object(item); String input = (String) row.get("input");
                for (boolean memo : List.of(false, true)) {
                    Object result = h.parse(input, h.options(true, true, true, memo, Map.of())); assertTrue(ok(result));
                    var root = Json.object(Json.object(h.canonical(result)).get("fields"));
                    var grammar = Json.object(Json.array(root.get("grammars")).getFirst());
                    var rule = Json.object(Json.array(Json.object(grammar.get("fields")).get("rules")).getFirst());
                    var annotations = Json.array(Json.object(rule.get("fields")).get("annotations"));
                    assertEquals(row.get("annotations"), annotations.stream().map(a -> {
                        var node = Json.object(a); return obj("type", node.get("type"), "fields", node.get("fields"));
                    }).toList());
                    assertEquals(row.get("annotations") instanceof List<?> list ? list.size() : -1, annotations.size());
                    for (Object annotation : annotations) {
                        var node = Json.object(annotation); var span = Json.array(node.get("span"));
                        int start = ((Number) span.get(0)).intValue(), end = ((Number) span.get(1)).intValue();
                        assertEquals(node.get("type").equals("RootAnnotation") ? "@root" : "@mapping(V)", input.substring(start, end).strip());
                    }
                }
            }
        }
    }

    @Test void packageAndJavaTypeNamesCannotShadowGeneratedTypes() throws Exception {
        for (String name : List.of("java", "java_", "javax", "org", "util", "String", "List", "Optional", "Object")) {
            var f = new IrFixture(); String word = f.literal("Root", "a");
            String site = f.capture(word, name, "text", "list", obj("kind", "text")); f.rule("Root", word);
            f.mapping("Root", name, List.of(field(name, obj("kind", "text"), "list")), List.of(recipeField(name, site, "text", "list")));
            try (var h = f.compile()) {
                var ast = Json.object(h.canonical(h.parse("a")));
                assertEquals(name, ast.get("type")); assertEquals(Map.of(name, List.of("a")), ast.get("fields"));
            }
        }
    }

    @Test void p4HasOnlyFrontRecipeMismatchesAndAllStatefulExternsCompile() throws Exception {
        var raw = Json.object(Json.read(Files.readString(Path.of("../ir/fixtures/tools__tinyexpression-p4-lsp-vscode__grammar__tinyexpression-p4.ubnf.ir.json"))));
        var semantics = Json.object(raw.get("semantics")); raw.put("semantics", semantics);
        var tokens = Json.array(semantics.get("tokens")).stream().map(Json::object).filter(t -> Json.object(t.get("spec")).get("kind").equals("extern")).toList();
        assertEquals(List.of("TinyExpressionP4::STRING", "TinyExpressionP4::CODE_START", "TinyExpressionP4::CODE_END"), tokens.stream().map(t -> t.get("symbolId")).toList());
        var errors = new ArrayList<String>();
        // Preserve the fixture and normal validation. Repair only the reported front mismatch
        // in memory, then revalidate to prove inputState or any other rejection is not hidden.
        for (int attempt = 0; ; attempt++) {
            assertTrue(attempt < 20, errors.toString());
            try {
                var ir = IrReader.read(Json.write(raw));
                try (var h = new CompilerHarness(ir, "test.p4")) { assertNotNull(h.parser); }
                break;
            } catch (IllegalArgumentException error) {
                assertTrue(error.getMessage().startsWith("Recipe conversion mismatch: "), error.toString());
                assertFalse(errors.contains(error.getMessage())); errors.add(error.getMessage());
                String key = error.getMessage().substring("Recipe conversion mismatch: ".length());
                var recipes = new ArrayList<Map<String, Object>>();
                boolean fixed = false;
                for (Object item : Json.array(semantics.get("recipes"))) {
                    var recipe = Json.object(item); var fields = new ArrayList<Map<String, Object>>();
                    for (Object value : Json.array(recipe.get("fields"))) {
                        var field = Json.object(value);
                        if ((recipe.get("id") + "." + field.get("field")).equals(key)) { assertEquals("node", field.get("conversion")); field.put("conversion", "mixed"); fixed = true; }
                        fields.add(field);
                    }
                    recipe.put("fields", fields); recipes.add(recipe);
                }
                assertTrue(fixed, key); semantics.put("recipes", recipes);
            }
        }
        // An updated front fixture may have no mismatch. Otherwise this is the known defect.
        if (!errors.isEmpty()) assertEquals("Recipe conversion mismatch: recipe:SliceNestedExpression.value", errors.getFirst());
    }

    @Test void scannerReceivesFiveStateFieldsAndOptionalValueSpan() throws Exception {
        var f = new IrFixture(); String token = f.token("Root", "T", extern());
        f.rule("Root", f.seq("Root", f.expression("Root", "lookahead", "child", f.literal("Root", "a")),
            f.expression("Root", "lookahead", "child", token)));
        try (var h = f.compile()) {
            var states = new ArrayList<Object>();
            Class<?> scannerType = h.loader.loadClass(h.pkg + ".api.TokenScanner");
            Class<?> spanType = h.loader.loadClass(h.pkg + ".api.TokenScanner$ValueSpan");
            var ctor = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanResult").getConstructor(boolean.class, int.class, int.class, spanType, List.class, List.class);
            Object scanner = Proxy.newProxyInstance(h.loader, new Class<?>[]{scannerType}, (proxy, method, args) -> {
                Object state = args[1]; states.add(state);
                assertEquals(0, get(state, "consumed")); assertEquals(1, get(state, "matched"));
                assertEquals("matchOnly", get(state, "mode").toString()); assertEquals(false, get(state, "invertMatch")); assertEquals(false, get(state, "resetMatchedWithConsumed"));
                return ctor.newInstance(true, 0, 2, null, null, null);
            });
            cursors(h.parse("ab", h.options(false, false, false, true, Map.of("scan", scanner))), true, 0, 2);
            assertEquals(1, states.size());
        }
    }

    @Test void nonRuleBodyMemoCachesSuccessFailureEffectsAndDiagnostics() throws Exception {
        for (boolean success : List.of(false, true)) {
            var f = new IrFixture(); String token = f.token("Root", "T", extern()); memo(f, token);
            String a = f.seq("Root", token, f.literal("Root", "x")), b = f.seq("Root", token, f.literal("Root", "y"));
            f.rule("Root", f.choice("Root", "ordered", a, b));
            try (var h = f.compile()) {
                var count = new AtomicInteger();
                Class<?> scannerType = h.loader.loadClass(h.pkg + ".api.TokenScanner");
                var effect = h.loader.loadClass(h.pkg + ".api.TokenScanner$Effect").getConstructor(String.class, String.class, int.class, int.class).newInstance("declare", "a", 0, 1);
                var scanDiagnostic = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanDiagnostic").getConstructor(int.class, String.class).newInstance(0, "SCAN");
                var ctor = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanResult").getConstructor(boolean.class, int.class, int.class, int.class, int.class, List.class, List.class);
                Object scanner = Proxy.newProxyInstance(h.loader, new Class<?>[]{scannerType}, (proxy, method, args) -> {
                    count.incrementAndGet(); return ctor.newInstance(success, success ? 1 : 0, success ? 1 : 0, 0, success ? 1 : 0, List.of(scanDiagnostic), success ? List.of(effect) : List.of());
                });
                Object on = h.parse("ay", h.options(false, true, true, true, Map.of("scan", scanner))); assertEquals(1, count.get());
                count.set(0); Object off = h.parse("ay", h.options(false, true, true, false, Map.of("scan", scanner))); assertEquals(2, count.get());
                assertEquals(success, ok(on)); assertEquals(get(off, "diagnostics"), get(on, "diagnostics"));
                assertEquals(get(off, "scope"), get(on, "scope")); assertEquals(get(off, "lexical"), get(on, "lexical"));
                assertEquals(success ? 1 : 0, ((List<?>) get(get(on, "scope"), "declarations")).size());
            }
        }
    }

    @Test void scannerStateIncludesBothValuesOfEveryModeFlag() throws Exception {
        var f = new IrFixture(); f.rule("Root", f.token("Root", "T", extern()));
        try (var h = f.compile()) {
            var received = new ArrayList<Object>();
            Class<?> scannerType = h.loader.loadClass(h.pkg + ".api.TokenScanner");
            Class<?> spanType = h.loader.loadClass(h.pkg + ".api.TokenScanner$ValueSpan");
            var result = h.loader.loadClass(h.pkg + ".api.TokenScanner$ScanResult").getConstructor(boolean.class, int.class, int.class, spanType, List.class, List.class);
            Object scanner = Proxy.newProxyInstance(h.loader, new Class<?>[]{scannerType}, (proxy, method, args) -> {
                received.add(args[1]);
                return result.newInstance(true, get(args[1], "mode").toString().equals("matchOnly") ? 1 : 3, 3, null, null, null);
            });
            Class<?> sessionType = h.loader.loadClass(h.pkg + ".internal.Session"), frameType = h.loader.loadClass(h.pkg + ".internal.Session$Frame");
            Object session = sessionType.getConstructor(CharSequence.class, h.options, String[].class)
                .newInstance("abc", h.options(false, false, false, true, Map.of("scan", scanner)), new String[]{"Root"});
            for (boolean matched : List.of(false, true)) for (boolean invert : List.of(false, true)) for (boolean reset : List.of(false, true)) {
                Object frame = frameType.getConstructor().newInstance();
                frameType.getField("c").setInt(frame, 1); frameType.getField("m").setInt(frame, 2);
                frameType.getField("matched").setBoolean(frame, matched); frameType.getField("invert").setBoolean(frame, invert); frameType.getField("reset").setBoolean(frame, reset);
                sessionType.getMethod("external", frameType, String.class).invoke(session, frame, "scan");
                Object state = received.getLast();
                assertEquals(1, get(state, "consumed")); assertEquals(2, get(state, "matched"));
                assertEquals(matched ? "matchOnly" : "consumed", get(state, "mode").toString());
                assertEquals(invert, get(state, "invertMatch")); assertEquals(reset, get(state, "resetMatchedWithConsumed"));
                assertEquals(matched ? 1 : 3, frameType.getField("c").getInt(frame)); assertEquals(3, frameType.getField("m").getInt(frame));
            }
            assertEquals(8, received.size());
        }
    }

    @Test void scopeEventsKeepOrderModeRollbackAndReplay() throws Exception {
        for (String mode : List.of("lexical", "dynamic")) {
            var f = new IrFixture(); String word = f.literal("Scoped", "a");
            String site = f.capture(word, "value", "text", "scalar", obj("kind", "text")); f.rule("Scoped", word);
            f.effects.add(obj("ruleId", "Scoped", "mode", mode, "events", List.of(
                obj("when", "entry", "action", "enter", "captureSiteId", null, "name", null),
                obj("when", "successBeforeLeave", "action", "leave", "captureSiteId", null, "name", null),
                obj("when", "successAfterLeave", "action", "declare", "captureSiteId", site, "name", "var"),
                obj("when", "afterDeclarations", "action", "use", "captureSiteId", site, "name", "var"))));
            f.rule("Root", f.choice("Root", "ordered", f.seq("Root", f.ref("Root", "Scoped"), f.literal("Root", "x")), f.ref("Root", "Scoped")));
            f.rule("Peek", f.expression("Peek", "lookahead", "child", f.ref("Peek", "Scoped")));
            try (var h = f.compile()) {
                Object on = h.parse("a"), off = h.parse("a", h.options(true, true, false, false, Map.of()));
                assertEquals(get(off, "scope"), get(on, "scope"));
                List<?> events = (List<?>) get(get(on, "scope"), "events"); assertEquals(4, events.size());
                for (int i = 0; i < events.size(); i++) {
                    assertEquals((long) i, get(events.get(i), "order")); assertEquals(mode, get(events.get(i), "mode"));
                    assertEquals(List.of("enter", "leave", "declare", "use").get(i), get(events.get(i), "action"));
                }
                assertEquals(0, get(events.getFirst(), "offsetCp")); assertEquals(1, get(events.get(1), "offsetCp"));
                assertEquals(List.of(), get(get(h.parse("b"), "scope"), "events"));
                Object peek = h.parser.getMethod("parseEntry", String.class, String.class, CharSequence.class, h.options)
                    .invoke(null, "Probe", "Peek", "a", h.options(false, false, false, true, Map.of()));
                cursors(peek, true, 0, 1); assertEquals(List.of(), get(get(peek, "scope"), "events"));
            }
        }
    }

    @Test void recursionLimitAbortsWholeParseAndNeverLeaksStackOverflow() throws Exception {
        var f = new IrFixture();
        String body = f.choice("Root", "ordered", f.seq("Root", f.literal("Root", "("), f.ref("Root", "Root"), f.literal("Root", ")")), f.literal("Root", "a"));
        f.rule("Root", f.expression("Root", "optional", "child", body));
        try (var h = f.compile()) {
            for (boolean memo : List.of(false, true)) {
                Object opts = h.options.getConstructor(boolean.class, boolean.class, boolean.class, boolean.class, Map.class, int.class)
                    .newInstance(false, false, false, memo, Map.of(), 8);
                assertTrue(ok(h.parse("(".repeat(7) + "a" + ")".repeat(7), opts)));
                Object failed = h.parse("(".repeat(8) + "a" + ")".repeat(8), opts);
                cursors(failed, false, 0, 0); diagnostic(failed, "syntax", 8, List.of("maximum parse depth exceeded (maxDepth=8)"));
                // issue #35 / D-070: 2,000 段は maxDepth（既定 10,000）の範囲内なので、
                // StackOverflowError を捕まえても maxDepth から見積もった十分な stack を持つ
                // 専用 Thread へ 1 度だけエスカレーションして通る（以前はここで誤って
                // 失敗になっていた＝本 issue の再現条件）。20,000 段は maxDepth を超えるので、
                // エスカレーション後でも論理上限に当たり、クラッシュせず同じ診断になる。
                Object shallowDeep = h.parse("(".repeat(2000) + "a" + ")".repeat(2000), h.options(false, false, false, memo, Map.of()));
                assertTrue(ok(shallowDeep));
                Object tooDeep = h.parse("(".repeat(20_000) + "a" + ")".repeat(20_000), h.options(false, false, false, memo, Map.of()));
                assertFalse(ok(tooDeep)); assertEquals("syntax", get(get(tooDeep, "diagnostics"), "kind"));
                assertEquals(List.of(), get(get(tooDeep, "scope"), "events"));
                assertTrue(((List<?>) get(get(tooDeep, "diagnostics"), "expected")).getFirst().toString().contains("maximum parse depth"));
            }
            for (String stack : List.of("256k", "1m")) {
                var process = new ProcessBuilder("java", "-Xss" + stack, "-cp", h.directory.resolve("classes").toString(), h.pkg + ".driver.Driver")
                    .redirectError(h.directory.resolve("depth.stderr").toFile()).start();
                try {
                    try (var input = process.getOutputStream()) {
                        for (String source : List.of("(".repeat(2000) + "a" + ")".repeat(2000), "(".repeat(20_000) + "a" + ")".repeat(20_000), "a")) {
                            var request = obj("id", source.length(), "grammar", "Probe", "entry", "Root", "input", source,
                                "options", obj("lexical", false, "scope", false));
                            input.write((Json.write(request) + "\n").getBytes(StandardCharsets.UTF_8));
                        }
                    }
                    String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
                    assertEquals(0, process.waitFor(), Files.readString(h.directory.resolve("depth.stderr")));
                    assertEquals("", Files.readString(h.directory.resolve("depth.stderr")));
                    var rows = output.lines().map(Json::read).map(Json::object).toList(); assertEquals(3, rows.size());
                    // issue #35 / D-070: 呼出し元プロセスの -Xss を絞っていても、2,000 段は
                    // エスカレーション先の専用 Thread で通る。20,000 段は maxDepth に当たって
                    // クラッシュせず診断になる。
                    assertEquals(true, rows.get(0).get("allConsumed"));
                    assertEquals(false, rows.get(1).get("accepted"));
                    assertEquals("syntax", Json.object(Json.array(rows.get(1).get("diagnostics")).getFirst()).get("kind"));
                    assertEquals(true, rows.getLast().get("allConsumed"));
                } finally { process.destroyForcibly(); }
            }
        }
    }
}
