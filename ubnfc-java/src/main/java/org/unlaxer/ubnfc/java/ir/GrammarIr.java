package org.unlaxer.ubnfc.java.ir;

import java.util.List;
import java.util.Map;

/** Typed v1 contract. Discriminated variants have nullable variant-specific components. */
public record GrammarIr(Analysis analysis, String analysisVersion, GrammarIrEntry entry, List<GrammarUnit> grammarUnits, List<ImportEdge> importEdges, Integer languageVersion, Plan plan, List<String> requiredFeatures, Semantics semantics, String serialization, List<Source> sources, Integer version) {
    public record Analysis(List<ExprAnalysis> expressions, List<PrefixTrie> prefixTries, List<RuleAnalysis> rules, List<Scc> sccs) {}
    public record Annotation(Map<String, Object> arguments, String name, Span source) {}
    public record AstField(String cardinality, String name, ValueType valueType) {}
    public record AstType(String enclosingTypeId, List<AstField> fields, String id, List<String> implements_, String kind, String name, List<AstTypeVariantsItem> variants, String ruleId) {}
    public record AstTypeVariantsItem(String javaName, String literal, String rustName, String variant) {}
    public record Boundary(String at, Integer childIndex, String ownerExprId) {}
    public record Capture(String cardinality, String conversion, String exprId, String name, String occurrenceOrder, String ruleId, String siteId, Boolean skip, String spanRule, ValueType valueType) {}
    public record Catalog(String captureSiteId, String context, String ruleId) {}
    public record CharRange(Integer end, Integer start) {}
    public record ChoicePlan(List<String> candidateOrder, String exprId, String fallback, Proof proof, String strategy, String trieId) {}
    public record CommonField(AstField field, String interfaceTypeId, List<String> variantTypeIds) {}
    public record Contracts(String choice, CursorContract cursor, String leftRecursion, String longest, String mappingFailure, String mappingPhase, OccurrenceContract occurrences, String predictive, String regexPolicy, String repeat, String repeatZeroMax, String spanUnit) {}
    public record CursorContract(String begin, String commit, String consume, String coordinate, String frameFlag, String internalJava, String internalRust, String matchOnly, List<String> mode, List<String> positions, String rollback) {}
    public record CursorTransition(String consumedMode, String failure, String flagEffect, String matchedMode, String matchedOnConsume) {}
    public record Delimiter(List<Integer> codePoints, String kind, String lineEndProfile, String open, String close) {}
    public record Diagnostics(String deepestRule, List<DiagnosticsExpectedDisplayItem> expectedDisplay, String javaSyntaxOffset, String merge, String negativeLookahead, String offsetUnit, Boolean retainFarthestSeparately, String rustSyntaxOffset, String successfulRuleFailures, String trailingInputOffset) {}
    public record DiagnosticsExpectedDisplayItem(String exprId, String java_, String rust) {}
    public record Eval(String kind, Map<String, Object> params, String strategy) {}
    public record Execution(String childMode, String entryFlag, String failureCursor, String primitiveProfile, String transaction) {}
    public record Expr(List<String> captureSiteIds, String diagnosticLabel, Execution execution, String id, ExprNode node, String ruleId, Span source) {}
    public record ExprAnalysis(CursorTransition cursorTransition, List<String> effects, String exprId, MayStart mayStart, MemoSafety memoSafety, Integer minConsume, Boolean nullable, String prefixTrieId) {}
    public record ExprNode(List<String> items, String kind, List<String> alternatives, String selection, String tieBreak, String child, Integer max, Integer min, String terminator, String element, Boolean pairRollback, String separator, Boolean caseSensitive, String value, Boolean wordBoundary, String symbolId, String message) {}
    public record GrammarUnit(List<Annotation> annotations, String id, String name, String package_, Settings rawSettings, Span source, String sourceId) {}
    public record ImportEdge(String alias, List<ImportEdgeBindingsItem> bindings, String fromGrammarId, String requestedPath, String resolvedGrammarId, String sourceId) {}
    public record ImportEdgeBindingsItem(String localName, String qualifiedSymbolId) {}
    public record LexicalPlan(LexicalPlanDfa dfa, String strategy, String tokenId) {}
    public record LexicalPlanDfa(Integer startState, List<LexicalPlanDfaStatesItem> states) {}
    public record LexicalPlanDfaStatesItem(Boolean accepting, Integer id, List<LexicalPlanDfaStatesItemTransitionsItem> transitions) {}
    public record LexicalPlanDfaStatesItemTransitionsItem(CharRange range, Integer target) {}
    public record MayStart(String certainty, Boolean eof, Boolean includesTrivia, List<CharRange> ranges, String zeroConsumeCondition) {}
    public record MemoEntry(Boolean cacheFailure, Boolean cacheSuccess, String exprId, Proof omissionProof, String reason) {}
    public record MemoPlan(List<MemoEntry> entries, String referenceMode, List<String> replay, List<String> requiredKey) {}
    public record MemoSafety(String assertedAlias, Boolean failure, String originalDeclaration, String provenance, String purity, String reason, Boolean success) {}
    public record OccurrenceContract(List<String> captureFields, Boolean diagnostics, List<String> lexicalFields, String lexicalOutput, Boolean nodeSpans, Boolean scopeEvents, String selfHostTransform) {}
    public record Operator(String associativity, String baseRuleId, String canonicalForm, String leftSiteId, String opSiteId, String operatorRuleId, Integer precedence, String rightSiteId) {}
    public record OperatorPlan(List<String> boundaryExprIds, List<String> liftRecipeIds, Proof proof, List<String> ruleIds, String strategy) {}
    public record Plan(List<ChoicePlan> choices, List<LexicalPlan> lexical, MemoPlan memo, List<OperatorPlan> operators, String profile, List<ScanPlan> scans, PlanTriviaCache triviaCache) {}
    /** D-072: 観測を要求しない終端領域（{@code plan.scans}、任意キー。無ければ空）。 */
    public record ScanPlan(ScanClass class_, String exprId, String kind, Proof proof, Boolean root, List<String> ruleIds) {}
    /** 反復の一括走査表。bitset は u64 なので {@code BigDecimal} のまま受けて 2 の補数の {@code long} へ落とす。 */
    public record ScanClass(java.math.BigDecimal asciiHigh, java.math.BigDecimal asciiLow, Boolean complete, List<CharRange> ranges) {}
    public record PlanTriviaCache(Boolean enabled, List<String> key) {}
    public record PrefixTrie(String id, Integer maxDepth, List<PrefixTrieNodesItem> nodes) {}
    public record PrefixTrieNodesItem(Boolean acceptsEof, List<String> candidateExprIds, List<String> completedCandidateExprIds, List<PrefixTrieNodesItemEdgesItem> edges, Integer index, List<String> unknownCandidateExprIds) {}
    public record PrefixTrieNodesItemEdgesItem(Integer codePoint, Integer target) {}
    public record Proof(String explanation, List<String> exprIds, String kind) {}
    public record Recipe(List<RecipeFieldsItem> fields, String id, String kind, RecipeLeafFallback leafFallback, String onFailure, String phase, String resultTypeId, String ruleId, String transformId) {}
    public record RecipeFieldsItem(String cardinality, String conversion, String field, List<String> siteIds) {}
    public record RecipeLeafFallback(Object left, String op, List<Object> right, String span, String typeId) {}
    public record Recovery(List<String> applicationSites, String astProjection, Boolean consumeSync, String mode, String noSyncAction, String progressRule, String recipeId, List<String> resolvedSyncPatterns, String ruleId) {}
    public record Rewrite(String kind, String originalBody, Proof proof, String rewrittenBody, String ruleId) {}
    public record Rule(List<Annotation> annotations, String body, Eval eval, List<String> mappingParams, String mappingTypeId, Operator operator, Boolean skip, String symbolId, String triviaPolicyId) {}
    public record RuleAnalysis(List<String> followSyncPatterns, String leftRecursion, Boolean progressProven, String ruleId, String sccId) {}
    public record Scc(String id, String memoPolicy, Boolean recursive, List<String> ruleIds) {}
    public record Scope(List<ScopeEffect> effects, String rollback, Integer trimMaxCodePoint, List<String> versionChanges) {}
    public record ScopeEffect(List<ScopeEffectEventsItem> events, String mode, String ruleId) {}
    public record ScopeEffectEventsItem(String action, String captureSiteId, String name, String when) {}
    public record Semantics(List<AstType> astTypes, List<Capture> captures, List<Catalog> catalogs, List<CommonField> commonFields, Contracts contracts, Diagnostics diagnostics, List<Expr> expressions, List<Rewrite> normalizations, List<Recipe> recipes, List<Recovery> recovery, List<Rule> rules, Scope scope, List<Symbol> symbols, List<Token> tokens, Trivia trivia, List<Typeof> typeofConstraints) {}
    public record Settings(SettingsComment comment, String interleave, String whitespace) {}
    public record SettingsComment(String blockClose, String blockOpen, String line) {}
    public record Source(Integer codePointLength, String id, String path, String sha256) {}
    public record Span(Integer end, String sourceId, Integer start) {}
    public record Symbol(String grammarId, String id, String kind, String name, String originalName, String qualifiedSymbolId, Span source, String sourceId) {}
    public record Token(List<Annotation> annotations, TokenDiagnosticProfiles diagnosticProfiles, MemoSafety memoSafety, TokenSpec spec, String symbolId) {}
    public record TokenDiagnosticProfiles(String java_, String rust) {}
    public record RegexDfa(Integer states, Integer start, List<Integer> accepting, List<RegexTransition> transitions,
                           Boolean anchoredAtStart, Boolean longestMatch, String semantics, String zeroWidth) {}
    public record RegexTransition(Integer from, Integer to, List<List<Integer>> ranges) {}
    public record TokenSpec(List<Integer> codePoints, String kind, String which, String emptyTerminator, String missingTerminator, Boolean preserveMatched, String terminator, String excluded, String pattern, Integer max, Integer min, String comparison, String word, Boolean wordBoundary, String javaFallback, String matching, Boolean regularSubset, String rustFallback, String zeroWidth, List<String> inputState, String javaBinding, String logicalId, String originalClassName, List<String> resultFields, String rustBinding, RegexDfa dfa, String fallback, List<CharRange> firstChars) {}
    public record Trivia(List<TriviaInsertionPlanItem> insertionPlan, List<TriviaPolicy> policies) {}
    public record TriviaInsertionPlanItem(Boundary boundary, String policyId) {}
    public record TriviaPolicy(List<Delimiter> effectiveDelimiterSet, String id, String precedence, Settings rawSettings) {}
    public record Typeof(String comparisonKind, String mismatch, String ownCapture, String phase, String referencedCapture, String siteId) {}
    public record ValueType(String kind, String java_, String rust, String typeId, List<ValueType> members, ValueType element) {}
    public record GrammarIrEntry(String grammarId, String ruleId) {}
}
