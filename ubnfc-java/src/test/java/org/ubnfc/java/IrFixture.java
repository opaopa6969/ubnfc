package org.ubnfc.java;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.ubnfc.java.ir.GrammarIr;
import org.ubnfc.java.ir.IrReader;
import org.ubnfc.java.ir.Json;

/** Small schema-conforming IRs built directly, without a UBNF lowerer. */
final class IrFixture {
    final Map<String, Object> root, semantics, analysis, plan;
    final List<Map<String, Object>> expressions = new ArrayList<>(), rules = new ArrayList<>(), captures = new ArrayList<>(),
        tokens = new ArrayList<>(), types = new ArrayList<>(), recipes = new ArrayList<>(), choices = new ArrayList<>(), effects = new ArrayList<>(), recoveries = new ArrayList<>(), commons = new ArrayList<>();
    final List<Map<String, Object>> symbols = new ArrayList<>(), expressionAnalyses = new ArrayList<>(), ruleAnalyses = new ArrayList<>(), memos = new ArrayList<>();
    final Map<String, Object> safety;
    private int next;
    IrFixture() throws Exception {
        root = Json.object(Json.read(Files.readString(Path.of("../ir/examples/tinycalc.ir.json"))));
        semantics = Json.object(root.get("semantics")); root.put("semantics", semantics);
        analysis = Json.object(root.get("analysis")); root.put("analysis", analysis);
        plan = Json.object(root.get("plan")); root.put("plan", plan);
        safety = Json.object(Json.object(Json.array(semantics.get("tokens")).getFirst()).get("memoSafety"));
        safety.put("originalDeclaration", null);
        root.put("requiredFeatures", List.of()); root.put("entry", obj("grammarId", "grammar:Probe", "ruleId", "Root"));
        var grammar = Json.object(Json.array(root.get("grammarUnits")).getFirst()); grammar.put("id", "grammar:Probe"); grammar.put("name", "Probe"); root.put("grammarUnits", List.of(grammar));
        semantics.put("expressions", expressions); semantics.put("rules", rules); semantics.put("captures", captures); semantics.put("symbols", symbols);
        semantics.put("tokens", tokens); semantics.put("astTypes", types); semantics.put("recipes", recipes); semantics.put("commonFields", commons);
        semantics.put("normalizations", new ArrayList<>()); semantics.put("recovery", recoveries);
        semantics.put("scope", obj("rollback", "undoLog", "trimMaxCodePoint", 32, "versionChanges", List.of("enter", "leave", "declare", "clear"), "effects", effects));
        var trivia = Json.object(semantics.get("trivia")); trivia.put("insertionPlan", new ArrayList<>()); semantics.put("trivia", trivia);
        var diag = Json.object(semantics.get("diagnostics")); diag.put("expectedDisplay", List.of()); semantics.put("diagnostics", diag);
        analysis.put("expressions", expressionAnalyses); analysis.put("rules", ruleAnalyses); analysis.put("prefixTries", List.of()); analysis.put("sccs", List.of());
        plan.put("choices", choices); plan.put("operators", List.of()); plan.put("lexical", List.of());
        var memo = Json.object(plan.get("memo")); memo.put("entries", memos); plan.put("memo", memo);
    }
    static Map<String, Object> obj(Object... fields) {
        var map = new LinkedHashMap<String, Object>(); for (int i = 0; i < fields.length; i += 2) map.put((String) fields[i], fields[i + 1]); return map;
    }
    String expression(String rule, String kind, Object... fields) {
        String id = "expr:probe.ubnf:0:1:body/" + next++ + "/" + kind;
        var node = obj("kind", kind); node.putAll(obj(fields));
        String transaction = switch (kind) { case "seq", "group", "lookahead", "negativeLookahead", "separated" -> "once"; case "repeat", "optional", "eof" -> "occurs"; case "choice" -> "perAlternative"; case "ruleRef", "tokenRef", "empty" -> "delegate"; default -> "none"; };
        expressions.add(obj("id", id, "ruleId", rule, "source", obj("sourceId", "probe.ubnf", "start", 0, "end", 1), "node", node,
            "execution", obj("entryFlag", kind.equals("seq") ? "setFalse" : "preserve", "transaction", transaction, "childMode", kind.contains("Lookahead") || kind.equals("lookahead") ? "matchOnly" : "inherit", "failureCursor", kind.equals("literal") ? "primitiveMayResetMatched" : "restoreTransaction", "primitiveProfile", kind.equals("literal") ? "legacyWord" : null), "captureSiteIds", new ArrayList<>(), "diagnosticLabel", null));
        expressionAnalyses.add(obj("exprId", id, "nullable", List.of("repeat", "optional", "empty", "lookahead", "negativeLookahead", "eof").contains(kind), "minConsume", 0,
            "mayStart", obj("certainty", "unknown", "ranges", List.of(), "eof", true, "zeroConsumeCondition", "unknown", "includesTrivia", true),
            "cursorTransition", obj("consumedMode", "mayAdvance", "matchedMode", "mayAdvance", "matchedOnConsume", "resetToConsumed", "failure", "contextual", "flagEffect", "contextual"),
            "prefixTrieId", null, "memoSafety", safety, "effects", List.of()));
        return id;
    }
    String literal(String rule, String value) { return literal(rule, value, false); }
    /** D-040: wordBoundary を明示する literal。 */
    String literal(String rule, String value, boolean wordBoundary) { return expression(rule, "literal", "value", value, "caseSensitive", true, "wordBoundary", wordBoundary); }
    String seq(String rule, String... items) { return expression(rule, "seq", "items", List.of(items)); }
    String ref(String rule, String callee) { return expression(rule, "ruleRef", "symbolId", callee); }
    String choice(String rule, String strategy, String... alternatives) {
        String id = expression(rule, "choice", "alternatives", List.of(alternatives), "selection", strategy, "tieBreak", "declarationOrder");
        choices.add(obj("exprId", id, "strategy", strategy.equals("predictive") ? "predict" : strategy, "candidateOrder", List.of(alternatives), "fallback", strategy.equals("predictive") ? "fullChoice" : "none", "proof", null, "trieId", null)); return id;
    }
    void rule(String name, String body) {
        rules.add(obj("symbolId", name, "body", body, "triviaPolicyId", "trivia:javaStyle", "mappingTypeId", null, "mappingParams", List.of(), "skip", false, "operator", null, "eval", null, "annotations", List.of()));
        symbol(name, "rule"); ruleAnalyses.add(obj("ruleId", name, "followSyncPatterns", null, "leftRecursion", "none", "sccId", "scc:probe", "progressProven", false));
    }
    private void symbol(String name, String kind) { symbols.add(obj("id", name, "qualifiedSymbolId", name, "grammarId", "grammar:Probe", "sourceId", "probe.ubnf", "name", name, "originalName", name, "kind", kind, "source", obj("sourceId", "probe.ubnf", "start", 0, "end", 1))); }
    String token(String rule, String name, Map<String, Object> spec) {
        tokens.add(obj("symbolId", name, "spec", spec, "memoSafety", new LinkedHashMap<>(safety), "annotations", List.of(), "diagnosticProfiles", obj("java", "test", "rust", "test")));
        symbol(name, "token"); return expression(rule, "tokenRef", "symbolId", name);
    }
    String capture(String id, String name, String conversion, String cardinality, Map<String, Object> type) {
        Map<String, Object> expression = expressions.stream().filter(e -> e.get("id").equals(id)).findFirst().orElseThrow();
        String site = id + "/capture/0"; expression.put("captureSiteIds", List.of(site));
        captures.add(obj("siteId", site, "exprId", id, "ruleId", expression.get("ruleId"), "name", name, "cardinality", cardinality, "valueType", type, "conversion", conversion, "spanRule", "consumedExtent", "occurrenceOrder", "completionInnerBeforeOuter", "skip", false)); return site;
    }
    void mapping(String rule, String name, List<Map<String, Object>> fields, List<Map<String, Object>> recipeFields) {
        types.add(obj("id", name, "name", name, "kind", "record", "enclosingTypeId", null, "implements", List.of(), "fields", fields));
        var definition = rules.stream().filter(r -> r.get("symbolId").equals(rule)).findFirst().orElseThrow(); definition.put("mappingTypeId", name);
        recipes.add(obj("id", "recipe:" + rule, "ruleId", rule, "resultTypeId", name, "kind", "construct", "fields", recipeFields, "phase", "selectedMapping", "onFailure", "mappingFailure", "transformId", null, "leafFallback", null));
    }
    static Map<String, Object> field(String name, Map<String, Object> type, String card) { return obj("name", name, "valueType", type, "cardinality", card); }
    static Map<String, Object> recipeField(String name, String site, String conversion, String card) { return obj("field", name, "siteIds", List.of(site), "conversion", conversion, "cardinality", card); }
    GrammarIr build() { return IrReader.read(Json.write(root)); }
    CompilerHarness compile() throws Exception { return new CompilerHarness(build(), "test.generated.probe"); }
}
