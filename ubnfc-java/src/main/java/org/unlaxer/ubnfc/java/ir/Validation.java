package org.unlaxer.ubnfc.java.ir;

import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Function;
import javax.lang.model.SourceVersion;
import org.unlaxer.ubnfc.java.ir.GrammarIr.*;

/** Cross references and backend capability checks, before any output is written. */
public final class Validation {
    private Validation() {}
    public static <T> Map<String, T> index(List<T> values, Function<T, String> id) {
        var result = new java.util.LinkedHashMap<String, T>();
        for (T value : values) if (result.put(id.apply(value), value) != null) throw new IllegalArgumentException("Duplicate ID: " + id.apply(value));
        return result;
    }
    public static void identifier(String name) {
        if (!SourceVersion.isIdentifier(name) || SourceVersion.isKeyword(name) || Set.of("record", "sealed", "permits", "yield", "var").contains(name))
            throw new IllegalArgumentException("Invalid Java identifier: " + name);
    }
    public static void unsupported(String name) { throw new IllegalArgumentException("Unsupported IR feature: " + name); }
    private static void validateDfa(RegexDfa dfa) {
        require(dfa.states() != null && dfa.states() > 0 && dfa.start() != null && dfa.start() >= 0 && dfa.start() < dfa.states(), "Invalid DFA start");
        require(Boolean.TRUE.equals(dfa.anchoredAtStart()) && "fail".equals(dfa.zeroWidth()), "Invalid DFA anchoring/zeroWidth");
        require("leftmostFirst".equals(dfa.semantics()) || dfa.semantics() == null && Boolean.TRUE.equals(dfa.longestMatch()), "Unsupported DFA semantics");
        for (int state : dfa.accepting()) require(state >= 0 && state < dfa.states(), "Invalid DFA accepting state");
        var ranges = new java.util.HashMap<Integer, java.util.List<int[]>>();
        for (var transition : dfa.transitions()) {
            require(transition.from() >= 0 && transition.from() < dfa.states() && transition.to() >= 0 && transition.to() < dfa.states(), "Invalid DFA transition state");
            for (var range : transition.ranges()) {
                require(range.size() == 2 && range.get(0) >= 0 && range.get(1) <= 0x10ffff && range.get(0) <= range.get(1), "Invalid DFA range");
                var previous = ranges.computeIfAbsent(transition.from(), key -> new java.util.ArrayList<>());
                require(previous.stream().noneMatch(r -> range.get(0) <= r[1] && r[0] <= range.get(1)), "Overlapping DFA transitions");
                previous.add(new int[]{range.get(0), range.get(1)});
            }
        }
    }
    private static void require(boolean condition, String reason) { if (!condition) throw new IllegalArgumentException(reason); }
    private static void reference(Map<String, ?> map, String id) { require(id != null && map.containsKey(id), "Unresolved reference: " + id); }
    /** UBNF 表層言語のバージョン（D-038）。追加フィールドなので欠落は 1 とみなす。 */
    public static final int MAX_LANGUAGE_VERSION = 2;
    public static void validate(GrammarIr ir) {
        int language = ir.languageVersion() == null ? 1 : ir.languageVersion();
        if (language < 1 || language > MAX_LANGUAGE_VERSION) unsupported("languageVersion:" + language);
        var s = ir.semantics();
        var expressions = index(s.expressions(), Expr::id); var rules = index(s.rules(), Rule::symbolId);
        var tokens = index(s.tokens(), Token::symbolId); var captures = index(s.captures(), Capture::siteId);
        var types = index(s.astTypes(), AstType::id); var policies = index(s.trivia().policies(), TriviaPolicy::id);
        var recipes = index(s.recipes(), Recipe::id); var analyses = index(ir.analysis().expressions(), ExprAnalysis::exprId);
        index(s.symbols(), Symbol::id); index(s.recipes(), Recipe::ruleId);
        reference(rules, ir.entry().ruleId());
        require(ir.grammarUnits().stream().anyMatch(g -> g.id().equals(ir.entry().grammarId())), "Missing entry grammar");
        for (Rule rule : rules.values()) {
            reference(expressions, rule.body()); reference(policies, rule.triviaPolicyId());
            if (rule.mappingTypeId() != null) reference(types, rule.mappingTypeId());
            if (rule.eval() != null && rule.eval().strategy().equals("auto") && rule.mappingTypeId() != null) {
                reference(types, rule.mappingTypeId());
                require(!types.get(rule.mappingTypeId()).kind().equals("transparentUnion"),
                    "Automatic @eval requires a concrete mapping, not sum " + rule.mappingTypeId());
                validateEval(rule.eval(), types.get(rule.mappingTypeId()));
            }
        }
        for (Expr e : expressions.values()) {
            reference(rules, e.ruleId()); reference(analyses, e.id());
            for (String site : e.captureSiteIds()) reference(captures, site);
            ExprNode n = e.node();
            for (String child : children(n)) reference(expressions, child);
            if (n.kind().equals("ruleRef")) reference(rules, n.symbolId());
            if (n.kind().equals("tokenRef")) reference(tokens, n.symbolId());
            if (n.kind().equals("repeat") && n.max() != null) require(n.min() <= n.max(), "repeat min > max: " + e.id());
            Execution x = e.execution();
            String transaction = switch (n.kind()) {
                case "seq", "lookahead", "negativeLookahead", "separated" -> "once";
                case "error" -> x.transaction().equals("none") ? "none" : "once";
                case "group" -> x.transaction().equals("delegate") ? "delegate" : "once";
                case "choice" -> "perAlternative"; case "repeat", "eof", "optional" -> "occurs";
                case "ruleRef", "tokenRef", "empty" -> "delegate"; default -> "none";
            };
            if (!x.transaction().equals(transaction)) unsupported("execution.transaction:" + n.kind() + ":" + x.transaction());
            boolean look = n.kind().equals("lookahead") || n.kind().equals("negativeLookahead");
            if (!x.childMode().equals(look ? "matchOnly" : "inherit")) unsupported("execution.childMode:" + n.kind() + ":" + x.childMode());
        }
        for (Capture capture : captures.values()) {
            reference(expressions, capture.exprId()); reference(rules, capture.ruleId());
            require(expressions.get(capture.exprId()).captureSiteIds().contains(capture.siteId()), "Unattached capture " + capture.siteId());
            valueType(capture.valueType(), types);
        }
        var typeNames = new HashSet<String>();
        String astName = ir.grammarUnits().stream().filter(g -> g.id().equals(ir.entry().grammarId())).findFirst().orElseThrow().name() + "AST";
        for (AstType type : types.values()) {
            require(typeNames.add(type.enclosingTypeId() + "/" + type.name()), "Duplicate Java type name: " + type.name());
            require(!type.name().equals(astName) || type.kind().equals("transparentUnion"), "Type collides with AST container: " + type.name());
            var nesting = new HashSet<String>(); AstType enclosing = type;
            while (enclosing != null) {
                require(nesting.add(enclosing.name()), "Cyclic or duplicate enclosing type: " + type.name());
                enclosing = enclosing.enclosingTypeId() == null ? null : types.get(enclosing.enclosingTypeId());
            }
            if (type.kind().equals("transparentUnion")) require(!type.variants().isEmpty(), "Empty sealed union: " + type.id());
            if (type.kind().equals("enum")) require(type.variants().stream().map(AstTypeVariantsItem::literal).distinct().count() == type.variants().size(), "Duplicate enum literal: " + type.id());
            identifier(type.name()); if (type.enclosingTypeId() != null) reference(types, type.enclosingTypeId());
            for (String parent : type.implements_()) reference(types, parent);
            if (type.kind().equals("transparentUnion")) for (var variant : type.variants()) reference(types, variant.variant());
            if (type.kind().equals("enum")) for (var variant : type.variants()) identifier(variant.javaName());
            var names = new HashSet<String>();
            for (var field : type.fields()) { identifier(field.name()); require(!Set.of("clone", "finalize", "getClass", "hashCode", "notify", "notifyAll", "toString", "wait").contains(field.name()), "Invalid record component: " + field.name()); require(names.add(field.name()), "Duplicate field " + field.name()); valueType(field.valueType(), types); }
        }
        for (Recipe recipe : recipes.values()) {
            reference(rules, recipe.ruleId());
            if (recipe.resultTypeId() != null) reference(types, recipe.resultTypeId());
            if (recipe.kind().equals("frontendTransform")) unsupported("frontendTransform:" + recipe.transformId());
            for (var field : recipe.fields()) for (String site : field.siteIds()) {
                reference(captures, site);
                require(captures.get(site).ruleId().equals(recipe.ruleId()), "Recipe capture is not rule-local: " + site);
            }
            if (Set.of("construct", "assocLists", "enum").contains(recipe.kind())) {
                reference(types, recipe.resultTypeId());
                AstType type = types.get(recipe.resultTypeId());
                require(type.kind().equals(recipe.kind().equals("enum") ? "enum" : "record"), "Recipe result kind mismatch: " + recipe.id());
                var fields = index(recipe.fields(), RecipeFieldsItem::field);
                if (!recipe.kind().equals("enum")) {
                    require(type.fields().size() == fields.size(), "Recipe field count mismatch: " + recipe.id());
                    for (var field : type.fields()) {
                        reference(fields, field.name()); var mapping = fields.get(field.name());
                        require(field.cardinality().equals(mapping.cardinality()), "Recipe cardinality mismatch: " + field.name());
                        String value = field.valueType().kind(), conversion = mapping.conversion();
                        boolean compatible = conversion.equals(value) || conversion.equals("node") && value.equals("mixed")
                            && field.valueType().members().stream().allMatch(m -> m.kind().equals("null") || m.kind().equals("node"));
                        require(compatible, "Recipe conversion mismatch: " + recipe.id() + "." + field.name());
                    }
                }
            }
            if (recipe.leafFallback() != null) {
                reference(types, recipe.leafFallback().typeId());
                var leaf = types.get(recipe.leafFallback().typeId());
                require(leaf.kind().equals("record") && leaf.fields().stream().map(AstField::name).toList().equals(List.of("left", "op", "right")), "Unsupported assocLists leafFallback shape");
            }
        }
        for (var field : s.commonFields()) {
            reference(types, field.interfaceTypeId());
            require(types.get(field.interfaceTypeId()).kind().equals("transparentUnion"), "commonField requires interface");
            for (String variant : field.variantTypeIds()) {
                reference(types, variant);
                require(types.get(variant).fields().contains(field.field()), "commonField mismatch: " + variant);
            }
        }
        for (var insertion : s.trivia().insertionPlan()) {
            reference(expressions, insertion.boundary().ownerExprId()); reference(policies, insertion.policyId());
            Expr owner = expressions.get(insertion.boundary().ownerExprId());
            if (!Set.of("seq", "group", "repeat", "separated").contains(owner.node().kind()) || owner.execution().transaction().equals("delegate")) unsupported("triviaInsertion:" + owner.node().kind());
            if (insertion.boundary().at().equals("afterChild") && !Set.of("seq", "group").contains(owner.node().kind())) unsupported("triviaInsertion:afterChild:" + owner.node().kind());
            if (insertion.boundary().at().equals("afterChild")) require(insertion.boundary().childIndex() != null && insertion.boundary().childIndex() < children(expressions.get(insertion.boundary().ownerExprId()).node()).size(), "Invalid trivia childIndex");
        }
        for (var policy : policies.values()) for (var delimiter : policy.effectiveDelimiterSet())
            if (!delimiter.kind().equals("characters")) require(!delimiter.open().isEmpty() && (delimiter.close() == null || !delimiter.close().isEmpty()), "Empty trivia delimiter");
        for (var effect : s.scope().effects()) { reference(rules, effect.ruleId()); for (var event : effect.events()) if (event.captureSiteId() != null) reference(captures, event.captureSiteId()); }
        for (var catalog : s.catalogs()) { reference(rules, catalog.ruleId()); reference(captures, catalog.captureSiteId()); }
        for (var constraint : s.typeofConstraints()) {
            reference(captures, constraint.ownCapture()); reference(captures, constraint.referencedCapture());
            Capture own = captures.get(constraint.ownCapture()), referenced = captures.get(constraint.referencedCapture());
            require(own.ruleId().equals(referenced.ruleId()), "typeof captures must be rule-local");
            Recipe recipe = s.recipes().stream().filter(r -> r.ruleId().equals(own.ruleId())).findFirst()
                .orElseThrow(() -> new IllegalArgumentException("typeof requires a mapping recipe"));
            require(recipe.kind().equals("construct") || recipe.kind().equals("assocLists"), "typeof requires a concrete mapping");
            for (String name : List.of(own.name(), referenced.name())) {
                require(captures.values().stream().anyMatch(c -> c.ruleId().equals(own.ruleId()) && c.name().equals(name)), "Unresolved typeof capture: " + name);
                require(recipe.fields().stream().anyMatch(f -> f.field().equals(name)), "Unmapped typeof capture: " + name);
            }
        }
        for (var recovery : s.recovery()) {
            reference(rules, recovery.ruleId());
            require(recovery.resolvedSyncPatterns().stream().noneMatch(String::isEmpty), "Empty recovery sync pattern");
            for (String site : recovery.applicationSites()) {
                reference(expressions, site);
                require(expressions.get(site).node().kind().equals("ruleRef") && recovery.ruleId().equals(expressions.get(site).node().symbolId()), "recovery site must reference its rule");
                require(s.recovery().stream().filter(r -> r.applicationSites().contains(site)).count() == 1, "Ambiguous recovery site: " + site);
            }
            if (recovery.recipeId() != null) { reference(recipes, recovery.recipeId()); require(recipes.get(recovery.recipeId()).fields().isEmpty(), "Recovery recipe requires unavailable captures"); }
        }
        for (var normalization : s.normalizations()) {
            reference(rules, normalization.ruleId()); reference(expressions, normalization.originalBody()); reference(expressions, normalization.rewrittenBody());
            require(rules.get(normalization.ruleId()).body().equals(normalization.rewrittenBody()), "leftRecursionRewrite body mismatch");
            require(normalization.proof().kind().equals("leftRecursionNormalization"), "leftRecursionRewrite proof required");
        }
        for (var analysis : ir.analysis().rules()) if (analysis.leftRecursion().equals("indirectRejected")) unsupported("indirectLeftRecursion:" + analysis.ruleId());
        for (var plan : ir.plan().operators()) if (!plan.strategy().equals("tierFunctions")) unsupported("precedenceClimbing");
        for (var choice : ir.plan().choices()) {
            reference(expressions, choice.exprId());
            var alternatives = expressions.get(choice.exprId()).node().alternatives();
            require(choice.candidateOrder().equals(alternatives), "Choice candidate order must preserve declaration order");
            if (choice.strategy().equals("predict")) require(choice.proof() != null || choice.fallback().equals("fullChoice"), "Predict requires proof or fullChoice fallback");
            require(expressions.get(choice.exprId()).node().selection().equals("longest") == choice.strategy().equals("longest"), "Longest semantics cannot be replaced");
        }
        var unsafe = new HashSet<String>();
        for (Token token : tokens.values()) {
            TokenSpec spec = token.spec();
            // D-063: 境界は code point（U+0000..U+10FFFF）。surrogate を**含む**範囲も拒否して
            // front / Rust backend（analysis/checks.rs、ir/references.rs）と同じ条件にする。
            if (spec.kind().equals("charRange")) require(spec.min() <= spec.max() && spec.max() <= 0x10ffff
                && !(spec.min() <= 0xdfff && spec.max() >= 0xd800), "Invalid charRange endpoints");
            if (spec.kind().equals("regex") && spec.dfa() == null) {
                try { java.util.regex.Pattern.compile(spec.pattern()); }
                catch (java.util.regex.PatternSyntaxException error) { throw new IllegalArgumentException("Invalid regex: " + token.symbolId(), error); }
            }
            int previous = -1;
            for (int point : spec.codePoints()) { require(point > previous && point <= 0x10ffff, "Invalid codePoints: " + token.symbolId()); previous = point; }
            if (!token.memoSafety().success() || !token.memoSafety().failure()) unsafe.add(token.symbolId());
            if (spec.kind().equals("regex") && spec.dfa() != null) validateDfa(spec.dfa());
            if (spec.kind().equals("regex") && spec.dfa() == null && spec.javaFallback().equals("none")
                && ir.plan().lexical().stream().noneMatch(p -> p.tokenId().equals(token.symbolId()) && p.dfa() != null)) unsupported("regex.dfaRequired");
        }
        boolean changed;
        do {
            changed = false;
            for (Expr expr : expressions.values()) if (unsafe.contains(expr.node().symbolId()) || children(expr.node()).stream().anyMatch(unsafe::contains)) changed |= unsafe.add(expr.id());
            for (Rule rule : rules.values()) if (unsafe.contains(rule.body())) changed |= unsafe.add(rule.symbolId());
        } while (changed);
        for (var memo : ir.plan().memo().entries()) {
            reference(expressions, memo.exprId());
            if (unsafe.contains(memo.exprId())) require(!memo.cacheFailure() && !memo.cacheSuccess(), "Unsafe extern memo: " + memo.exprId());
            if (memo.reason().equals("defaultSafe")) require(memo.cacheSuccess() && memo.cacheFailure(), "Memo omission requires proof: " + memo.exprId());
            if (memo.reason().equals("provedOmission")) require(memo.omissionProof() != null && !memo.omissionProof().kind().equals("none"), "Memo omission requires proof");
        }
        // The front supplies lowering; never attempt seed-growing on an unlowered cycle.
        // directRewritten is history; inspect current bodies, not retained originals.
        var nullable = nullableExpressions(expressions, rules, tokens);
        for (Rule rule : rules.values()) leftCycle(rule.symbolId(), rule.body(), expressions, rules, nullable, new HashSet<>());
    }
    private static void validateEval(Eval eval, AstType type) {
        var fields = index(type.fields(), AstField::name);
        List<String> required = switch (eval.kind()) {
            case "binary_arithmetic" -> List.of("left", "op", "right");
            case "variable_ref" -> List.of("name");
            case "conditional" -> List.of("condition", "thenExpr", "elseExpr");
            case "passthrough", "literal" -> List.of("value");
            default -> List.of();
        };
        for (String field : required) {
            reference(fields, field); AstField value = fields.get(field);
            String cardinality = eval.kind().equals("binary_arithmetic") && !field.equals("left") ? "list" : "scalar";
            require(value.cardinality().equals(cardinality), "Invalid @eval cardinality: " + type.name() + "." + field);
            if (eval.kind().equals("conditional") || eval.kind().equals("binary_arithmetic") && !field.equals("op"))
                require(value.valueType().kind().equals("node"), "@eval requires node field: " + field);
            if (eval.kind().equals("variable_ref") || eval.kind().equals("binary_arithmetic") && field.equals("op"))
                require(value.valueType().kind().equals("text"), "@eval requires text field: " + field);
        }
        if (eval.params().containsKey("strip_prefix")) require(eval.params().get("strip_prefix") instanceof String, "@eval strip_prefix must be text");
    }
    private static void valueType(ValueType value, Map<String, AstType> types) {
        if (value.kind().equals("optional") || value.kind().equals("list")) unsupported("nestedValueType:" + value.kind());
        if (value.kind().equals("node")) reference(types, value.typeId());
        if (value.element() != null) valueType(value.element(), types);
        for (var member : value.members()) valueType(member, types);
    }
    private static Set<String> nullableExpressions(Map<String, Expr> expressions, Map<String, Rule> rules, Map<String, Token> tokens) {
        // Analysis may be conservative or stale on hand-built IR. Derive zero
        // consumption from semantics, including nullable rule-reference chains.
        var nullable = new HashSet<String>();
        boolean changed;
        do {
            changed = false;
            for (Expr expr : expressions.values()) {
                ExprNode n = expr.node();
                boolean empty = switch (n.kind()) {
                    case "seq" -> n.items().stream().allMatch(nullable::contains);
                    case "choice" -> n.alternatives().stream().anyMatch(nullable::contains);
                    case "ruleRef" -> nullable.contains(rules.get(n.symbolId()).body());
                    case "group" -> nullable.contains(n.child());
                    case "repeat" -> n.min() == 0 || nullable.contains(n.child());
                    case "separated" -> n.min() == 0 || nullable.contains(n.element());
                    case "literal" -> n.value().isEmpty();
                    case "tokenRef" -> {
                        TokenSpec spec = tokens.get(n.symbolId()).spec();
                        yield switch (spec.kind()) {
                            case "builtin" -> spec.which().equals("EndOfSource");
                            case "caseInsensitive" -> spec.word().isEmpty();
                            case "eof", "empty", "until", "lookahead", "negativeLookahead", "extern" -> true;
                            default -> false;
                        };
                    }
                    case "optional", "lookahead", "negativeLookahead", "eof", "empty" -> true;
                    default -> false;
                };
                if (empty) changed |= nullable.add(expr.id());
            }
        } while (changed);
        return nullable;
    }
    private static void leftCycle(String target, String id, Map<String, Expr> expressions, Map<String, Rule> rules, Set<String> nullable, Set<String> seen) {
        if (!seen.add(id)) return;
        ExprNode n = expressions.get(id).node();
        if (n.kind().equals("ruleRef")) {
            if (n.symbolId().equals(target)) unsupported("unloweredLeftRecursion:" + target);
            leftCycle(target, rules.get(n.symbolId()).body(), expressions, rules, nullable, seen);
        } else {
            for (String child : children(n)) {
                leftCycle(target, child, expressions, rules, nullable, seen);
                if ((n.kind().equals("seq") || n.kind().equals("separated")) && !nullable.contains(child)) break;
            }
        }
    }
    public static List<String> children(ExprNode node) {
        return switch (node.kind()) {
            case "seq" -> node.items(); case "choice" -> node.alternatives();
            case "repeat" -> node.terminator() == null ? List.of(node.child()) : List.of(node.child(), node.terminator());
            case "group", "optional", "lookahead", "negativeLookahead" -> List.of(node.child());
            case "separated" -> List.of(node.element(), node.separator()); default -> List.of();
        };
    }
}
