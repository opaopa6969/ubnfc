use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// ID の一意性・参照存在・所属・source span・trivia 境界・機能宣言を検査する。
///
/// FIRST/memo の証明や mapping の型推論を再実行するものではない。
/// requiredFeatures は実際の構造から必要な機能を求め、不足を拒否する。
/// 未使用の既知機能の宣言は保守的な capability 要求として許容する。
pub fn validate_references(ir: &GrammarIr) -> Vec<IrError> {
    let mut validator = Validator {
        ir,
        ids: BTreeMap::new(),
        errors: Vec::new(),
    };
    validator.index();
    validator.sources();
    validator.semantics();
    validator.analysis_and_plan();
    validator.features();
    validator.errors
}

struct Validator<'a> {
    ir: &'a GrammarIr,
    ids: BTreeMap<&'static str, BTreeSet<&'a str>>,
    errors: Vec<IrError>,
}

impl<'a> Validator<'a> {
    fn error(&mut self, path: &str, message: impl Into<String>) {
        self.errors.push(IrError::Validation {
            path: path.to_owned(),
            message: message.into(),
        });
    }

    fn register(&mut self, kind: &'static str, id: &'a str, path: &str) {
        if id.is_empty() {
            self.error(path, format!("empty {kind} ID"));
        }
        if !self.ids.entry(kind).or_default().insert(id) {
            self.error(path, format!("duplicate {kind} ID: {id}"));
        }
    }

    fn check(&mut self, kind: &str, id: &str, path: &str) {
        if !self.ids.get(kind).is_some_and(|ids| ids.contains(id)) {
            self.error(path, format!("unknown {kind} ID: {id}"));
        }
    }

    fn optional(&mut self, kind: &str, id: &Option<String>, path: &str) {
        if let Some(id) = id {
            self.check(kind, id, path);
        }
    }

    fn refs(&mut self, kind: &str, ids: &[String], path: &str) {
        for (i, id) in ids.iter().enumerate() {
            self.check(kind, id, &format!("{path}/{i}"));
        }
    }

    fn exprs(&mut self, ids: &[ExprId], path: &str) {
        for (i, id) in ids.iter().enumerate() {
            self.check("expression", id.as_str(), &format!("{path}/{i}"));
        }
    }

    fn index(&mut self) {
        let ir = self.ir;
        macro_rules! index {
            ($kind:literal, $items:expr, $field:ident, $path:literal) => {
                for (i, item) in $items.iter().enumerate() {
                    self.register(
                        $kind,
                        &item.$field,
                        &format!(
                            "{}/{i}/{}",
                            $path,
                            match stringify!($field) {
                                "symbol_id" => "symbolId",
                                "qualified_symbol_id" => "qualifiedSymbolId",
                                "site_id" => "siteId",
                                field => field,
                            }
                        ),
                    );
                }
            };
        }
        index!("source", ir.sources, id, "/sources");
        index!("grammar", ir.grammar_units, id, "/grammarUnits");
        index!("symbol", ir.semantics.symbols, id, "/semantics/symbols");
        index!(
            "qualifiedSymbol",
            ir.semantics.symbols,
            qualified_symbol_id,
            "/semantics/symbols"
        );
        index!("rule", ir.semantics.rules, symbol_id, "/semantics/rules");
        index!("token", ir.semantics.tokens, symbol_id, "/semantics/tokens");
        index!(
            "capture",
            ir.semantics.captures,
            site_id,
            "/semantics/captures"
        );
        index!("recipe", ir.semantics.recipes, id, "/semantics/recipes");
        index!(
            "triviaPolicy",
            ir.semantics.trivia.policies,
            id,
            "/semantics/trivia/policies"
        );
        index!(
            "trie",
            ir.analysis.prefix_tries,
            id,
            "/analysis/prefixTries"
        );
        index!("scc", ir.analysis.sccs, id, "/analysis/sccs");
        index!(
            "typeof",
            ir.semantics.typeof_constraints,
            site_id,
            "/semantics/typeofConstraints"
        );
        for (i, expr) in ir.semantics.expressions.iter().enumerate() {
            self.register(
                "expression",
                expr.id.as_str(),
                &format!("/semantics/expressions/{i}/id"),
            );
        }
        for (i, ty) in ir.semantics.ast_types.iter().enumerate() {
            self.register("type", ty.id(), &format!("/semantics/astTypes/{i}/id"));
        }
    }

    fn span(&mut self, span: &Span, path: &str) {
        self.check("source", &span.source_id, &format!("{path}/sourceId"));
        if span.start > span.end {
            self.error(path, "span start exceeds end");
        }
        if let Some(source) = self.ir.sources.iter().find(|s| s.id == span.source_id) {
            if span.end > source.code_point_length {
                self.error(path, "span exceeds source codePointLength");
            }
        }
    }

    fn annotations(&mut self, annotations: &[Annotation], path: &str) {
        for (i, annotation) in annotations.iter().enumerate() {
            self.span(&annotation.source, &format!("{path}/{i}/source"));
        }
    }

    fn sources(&mut self) {
        let ir = self.ir;
        if ir.sources.is_empty() {
            self.error("/sources", "at least one source is required");
        }
        if ir.grammar_units.is_empty() {
            self.error("/grammarUnits", "at least one grammar is required");
        }
        let mut paths = BTreeSet::new();
        for (i, source) in ir.sources.iter().enumerate() {
            let p = format!("/sources/{i}");
            if !canonical_path(&source.path) {
                self.error(
                    &format!("{p}/path"),
                    "expected normalized relative POSIX path within the source root",
                );
            }
            if !paths.insert(&source.path) {
                self.error(&format!("{p}/path"), "duplicate source path");
            }
            if source.id != source_id(&source.path) {
                self.error(
                    &format!("{p}/id"),
                    "sourceId must be the UTF-8 percent-encoded logical path",
                );
            }
            if source.sha256.len() != 64
                || !source
                    .sha256
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            {
                self.error(
                    &format!("{p}/sha256"),
                    "expected 64 lowercase hexadecimal digits",
                );
            }
        }
        for (i, unit) in ir.grammar_units.iter().enumerate() {
            let p = format!("/grammarUnits/{i}");
            self.check("source", &unit.source_id, &format!("{p}/sourceId"));
            self.span(&unit.source, &format!("{p}/source"));
            if unit.source_id != unit.source.source_id {
                self.error(
                    &format!("{p}/sourceId"),
                    "grammar sourceId disagrees with source span",
                );
            }
            self.annotations(&unit.annotations, &format!("{p}/annotations"));
        }
        self.check("grammar", &ir.entry.grammar_id, "/entry/grammarId");
        self.check("rule", &ir.entry.rule_id, "/entry/ruleId");
        if let Some(symbol) = ir
            .semantics
            .symbols
            .iter()
            .find(|s| s.id == ir.entry.rule_id)
        {
            if symbol.grammar_id != ir.entry.grammar_id {
                self.error("/entry", "entry rule belongs to a different grammar");
            }
        }
        for (i, edge) in ir.import_edges.iter().enumerate() {
            let p = format!("/importEdges/{i}");
            self.check("source", &edge.source_id, &format!("{p}/sourceId"));
            self.check(
                "grammar",
                &edge.from_grammar_id,
                &format!("{p}/fromGrammarId"),
            );
            self.check(
                "grammar",
                &edge.resolved_grammar_id,
                &format!("{p}/resolvedGrammarId"),
            );
            if let Some(unit) = ir
                .grammar_units
                .iter()
                .find(|g| g.id == edge.from_grammar_id)
            {
                if unit.source_id != edge.source_id {
                    self.error(
                        &format!("{p}/sourceId"),
                        "import source must match the importing grammar",
                    );
                }
            }
            for (j, binding) in edge.bindings.iter().enumerate() {
                self.check(
                    "qualifiedSymbol",
                    &binding.qualified_symbol_id,
                    &format!("{p}/bindings/{j}/qualifiedSymbolId"),
                );
            }
        }
    }

    fn semantics(&mut self) {
        let s = &self.ir.semantics;
        for (i, symbol) in s.symbols.iter().enumerate() {
            let p = format!("/semantics/symbols/{i}");
            self.check("grammar", &symbol.grammar_id, &format!("{p}/grammarId"));
            self.check("source", &symbol.source_id, &format!("{p}/sourceId"));
            self.span(&symbol.source, &format!("{p}/source"));
            if symbol.source_id != symbol.source.source_id {
                self.error(
                    &format!("{p}/sourceId"),
                    "symbol sourceId disagrees with source span",
                );
            }
            if let Some(unit) = self
                .ir
                .grammar_units
                .iter()
                .find(|g| g.id == symbol.grammar_id)
            {
                if unit.source_id != symbol.source_id {
                    self.error(
                        &format!("{p}/sourceId"),
                        "symbol source differs from its grammar source",
                    );
                }
            }
            self.check(
                match symbol.kind {
                    SymbolKind::Rule => "rule",
                    SymbolKind::Token => "token",
                },
                &symbol.id,
                &format!("{p}/id"),
            );
        }
        for (i, rule) in s.rules.iter().enumerate() {
            let p = format!("/semantics/rules/{i}");
            self.symbol_kind(&rule.symbol_id, SymbolKind::Rule, &format!("{p}/symbolId"));
            self.owned_expr(&rule.body, &rule.symbol_id, &format!("{p}/body"));
            self.check(
                "triviaPolicy",
                &rule.trivia_policy_id,
                &format!("{p}/triviaPolicyId"),
            );
            self.optional("type", &rule.mapping_type_id, &format!("{p}/mappingTypeId"));
            self.annotations(&rule.annotations, &format!("{p}/annotations"));
            if let Some(op) = &rule.operator {
                self.check(
                    "rule",
                    &op.base_rule_id,
                    &format!("{p}/operator/baseRuleId"),
                );
                self.check(
                    "rule",
                    &op.operator_rule_id,
                    &format!("{p}/operator/operatorRuleId"),
                );
                for (key, site) in [
                    ("leftSiteId", &op.left_site_id),
                    ("opSiteId", &op.op_site_id),
                    ("rightSiteId", &op.right_site_id),
                ] {
                    self.owned_capture(site, &rule.symbol_id, &format!("{p}/operator/{key}"));
                }
            }
        }
        for (i, token) in s.tokens.iter().enumerate() {
            let p = format!("/semantics/tokens/{i}");
            self.symbol_kind(
                &token.symbol_id,
                SymbolKind::Token,
                &format!("{p}/symbolId"),
            );
            self.annotations(&token.annotations, &format!("{p}/annotations"));
            self.memo_safety(&token.memo_safety, &format!("{p}/memoSafety"));
            if let TokenSpec::CharRange { min, max } = token.spec {
                if min > max || max > 0x10ffff || (min <= 0xdfff && max >= 0xd800) {
                    self.error(
                        &format!("{p}/spec"),
                        "invalid code point charRange or surrogate range",
                    );
                }
            }
            if let TokenSpec::Builtin { code_points, .. } = &token.spec {
                self.code_points(code_points, &format!("{p}/spec/codePoints"));
            }
            if let TokenSpec::Extern {
                first_chars: Some(ranges),
                ..
            } = &token.spec
            {
                let q = format!("{p}/spec/firstChars");
                if ranges.is_empty() {
                    self.error(&q, "firstChars must not be empty (omit for unknown)");
                }
                for (j, r) in ranges.iter().enumerate() {
                    if r.start > r.end || r.end > 0x10ffff {
                        self.error(&format!("{q}/{j}"), "invalid firstChars range");
                    }
                    if j > 0 && ranges[j - 1].end >= r.start {
                        self.error(
                            &format!("{q}/{j}"),
                            "firstChars must be ascending and disjoint",
                        );
                    }
                }
            }
            self.regex_plan(token, &format!("{p}/spec"));
        }
        for (i, expr) in s.expressions.iter().enumerate() {
            let p = format!("/semantics/expressions/{i}");
            self.check("rule", &expr.rule_id, &format!("{p}/ruleId"));
            self.span(&expr.source, &format!("{p}/source"));
            let expected = format!(
                "expr:{}:{}:{}:",
                expr.source.source_id, expr.source.start, expr.source.end
            );
            if !expr.id.as_str().starts_with(&expected)
                || ExprId::try_from(expr.id.0.clone()).is_err()
            {
                self.error(
                    &format!("{p}/id"),
                    "ExprId must agree with its source span and role syntax",
                );
            }
            if let Some(symbol) = s.symbols.iter().find(|symbol| symbol.id == expr.rule_id) {
                if expr.source.source_id != symbol.source_id {
                    self.error(
                        &format!("{p}/source/sourceId"),
                        "expression source differs from its rule source",
                    );
                }
            }
            for (j, child) in expr.node.children().iter().enumerate() {
                let key = match &expr.node {
                    ExprNode::Seq { .. } => format!("items/{j}"),
                    ExprNode::Choice { .. } => format!("alternatives/{j}"),
                    ExprNode::Repeat { .. } if j == 1 => "terminator".into(),
                    ExprNode::Separated { .. } => {
                        if j == 0 { "element" } else { "separator" }.into()
                    }
                    _ => "child".into(),
                };
                self.owned_expr(child, &expr.rule_id, &format!("{p}/node/{key}"));
            }
            match &expr.node {
                ExprNode::RuleRef { symbol_id } => {
                    self.check("rule", symbol_id, &format!("{p}/node/symbolId"))
                }
                ExprNode::TokenRef { symbol_id } => {
                    self.check("token", symbol_id, &format!("{p}/node/symbolId"))
                }
                ExprNode::Repeat {
                    min,
                    max: Some(max),
                    ..
                } if min > max => self.error(&format!("{p}/node"), "repeat min exceeds max"),
                ExprNode::Choice { alternatives, .. } if alternatives.is_empty() => self.error(
                    &format!("{p}/node/alternatives"),
                    "choice requires an alternative",
                ),
                _ => {}
            }
            let mut sites = BTreeSet::new();
            for (j, site) in expr.capture_site_ids.iter().enumerate() {
                self.owned_capture(site, &expr.rule_id, &format!("{p}/captureSiteIds/{j}"));
                if !sites.insert(site) {
                    self.error(&format!("{p}/captureSiteIds/{j}"), "duplicate capture site");
                }
                if let Some(capture) = s.captures.iter().find(|c| &c.site_id == site) {
                    if capture.expr_id != expr.id {
                        self.error(
                            &format!("{p}/captureSiteIds/{j}"),
                            "capture belongs to another expression",
                        );
                    }
                }
            }
        }
        self.trivia();
        self.mapping();
        self.effects();
    }

    fn symbol_kind(&mut self, id: &str, kind: SymbolKind, path: &str) {
        self.check("symbol", id, path);
        if let Some(symbol) = self.ir.semantics.symbols.iter().find(|s| s.id == id) {
            if symbol.kind != kind {
                self.error(path, "symbol has the wrong kind");
            }
        }
    }

    fn owned_expr(&mut self, id: &ExprId, rule: &str, path: &str) {
        self.check("expression", id.as_str(), path);
        if let Some(expr) = self.ir.semantics.expressions.iter().find(|e| e.id == *id) {
            if expr.rule_id != rule {
                self.error(path, "expression belongs to another rule");
            }
        }
    }

    fn owned_capture(&mut self, id: &str, rule: &str, path: &str) {
        self.check("capture", id, path);
        if let Some(capture) = self.ir.semantics.captures.iter().find(|c| c.site_id == id) {
            if capture.rule_id != rule {
                self.error(path, "capture belongs to another rule");
            }
        }
    }

    fn memo_safety(&mut self, safety: &MemoSafety, path: &str) {
        // assertedAlias は原宣言の alias 名であり、統合後の SymbolId ではない。
        if safety.provenance == MemoSafetyProvenance::UserAliasAssertion
            && safety.asserted_alias.is_none()
        {
            self.error(
                &format!("{path}/assertedAlias"),
                "user alias assertion requires an alias",
            );
        }
    }

    fn code_points(&mut self, points: &[u64], path: &str) {
        if points.iter().any(|p| *p > 0x10ffff) || points.windows(2).any(|w| w[0] >= w[1]) {
            self.error(path, "code points must be in range, unique and ascending");
        }
    }

    fn trivia(&mut self) {
        let trivia = &self.ir.semantics.trivia;
        for (i, policy) in trivia.policies.iter().enumerate() {
            for (j, delimiter) in policy.effective_delimiter_set.iter().enumerate() {
                if let Delimiter::Characters { code_points } = delimiter {
                    self.code_points(
                        code_points,
                        &format!(
                            "/semantics/trivia/policies/{i}/effectiveDelimiterSet/{j}/codePoints"
                        ),
                    );
                }
            }
        }
        for (i, insertion) in trivia.insertion_plan.iter().enumerate() {
            let p = format!("/semantics/trivia/insertionPlan/{i}");
            self.check(
                "triviaPolicy",
                &insertion.policy_id,
                &format!("{p}/policyId"),
            );
            let boundary = &insertion.boundary;
            self.check(
                "expression",
                boundary.owner_expr_id.as_str(),
                &format!("{p}/boundary/ownerExprId"),
            );
            match boundary.at {
                BoundaryAt::AfterChild => {
                    let count = self
                        .ir
                        .semantics
                        .expressions
                        .iter()
                        .find(|e| e.id == boundary.owner_expr_id)
                        .map(|e| e.node.children().len());
                    if boundary.child_index.is_none()
                        || count
                            .is_some_and(|n| boundary.child_index.is_some_and(|i| i >= n as u64))
                    {
                        self.error(
                            &format!("{p}/boundary/childIndex"),
                            "afterChild requires an existing grammar-order child index",
                        );
                    }
                }
                _ if boundary.child_index.is_some() => self.error(
                    &format!("{p}/boundary/childIndex"),
                    "only afterChild accepts a child index",
                ),
                _ => {}
            }
        }
    }

    fn value_type(&mut self, ty: &ValueType, path: &str) {
        match ty {
            ValueType::Node { type_id } => self.check("type", type_id, &format!("{path}/typeId")),
            ValueType::Mixed { members } => {
                if members.len() < 2 {
                    self.error(path, "mixed requires at least two members");
                }
                for (i, member) in members.iter().enumerate() {
                    self.value_type(member, &format!("{path}/members/{i}"));
                }
            }
            ValueType::Optional { element } | ValueType::List { element } => {
                self.value_type(element, &format!("{path}/element"))
            }
            _ => {}
        }
    }

    fn mapping(&mut self) {
        let s = &self.ir.semantics;
        for (i, capture) in s.captures.iter().enumerate() {
            let p = format!("/semantics/captures/{i}");
            self.check("rule", &capture.rule_id, &format!("{p}/ruleId"));
            self.owned_expr(&capture.expr_id, &capture.rule_id, &format!("{p}/exprId"));
            self.value_type(&capture.value_type, &format!("{p}/valueType"));
            if let Some(expr) = s.expressions.iter().find(|e| e.id == capture.expr_id) {
                if let Some(index) = expr
                    .capture_site_ids
                    .iter()
                    .position(|id| id == &capture.site_id)
                {
                    if capture.site_id != format!("{}/capture/{index}", expr.id) {
                        self.error(
                            &format!("{p}/siteId"),
                            "capture site ID disagrees with expression-local declaration order",
                        );
                    }
                } else {
                    self.error(
                        &format!("{p}/siteId"),
                        "capture is absent from its expression's captureSiteIds",
                    );
                }
            }
        }
        for (i, ty) in s.ast_types.iter().enumerate() {
            let p = format!("/semantics/astTypes/{i}");
            self.optional(
                "type",
                ty.enclosing_type_id(),
                &format!("{p}/enclosingTypeId"),
            );
            match ty {
                AstType::Record {
                    implements, fields, ..
                } => {
                    self.refs("type", implements, &format!("{p}/implements"));
                    for (j, field) in fields.iter().enumerate() {
                        self.value_type(&field.value_type, &format!("{p}/fields/{j}/valueType"));
                    }
                }
                AstType::TransparentUnion { variants, .. } => {
                    self.refs("type", variants, &format!("{p}/variants"))
                }
                AstType::Enum {
                    rule_id, variants, ..
                } => {
                    self.check("rule", rule_id, &format!("{p}/ruleId"));
                    if variants.is_empty() {
                        self.error(&format!("{p}/variants"), "enum requires a variant");
                    }
                }
            }
        }
        for (i, common) in s.common_fields.iter().enumerate() {
            let p = format!("/semantics/commonFields/{i}");
            self.check(
                "type",
                &common.interface_type_id,
                &format!("{p}/interfaceTypeId"),
            );
            self.refs(
                "type",
                &common.variant_type_ids,
                &format!("{p}/variantTypeIds"),
            );
            self.value_type(&common.field.value_type, &format!("{p}/field/valueType"));
        }
        for (i, recipe) in s.recipes.iter().enumerate() {
            let p = format!("/semantics/recipes/{i}");
            self.check("rule", &recipe.rule_id, &format!("{p}/ruleId"));
            self.optional("type", &recipe.result_type_id, &format!("{p}/resultTypeId"));
            if let Some(leaf) = &recipe.leaf_fallback {
                self.check("type", &leaf.type_id, &format!("{p}/leafFallback/typeId"));
            }
            for (j, field) in recipe.fields.iter().enumerate() {
                for (k, site) in field.site_ids.iter().enumerate() {
                    self.owned_capture(
                        site,
                        &recipe.rule_id,
                        &format!("{p}/fields/{j}/siteIds/{k}"),
                    );
                }
            }
        }
        for (i, constraint) in s.typeof_constraints.iter().enumerate() {
            let p = format!("/semantics/typeofConstraints/{i}");
            // siteId は制約自身の ID。ownCapture/referencedCapture は capture site ID と解釈する。
            self.check(
                "capture",
                &constraint.own_capture,
                &format!("{p}/ownCapture"),
            );
            self.check(
                "capture",
                &constraint.referenced_capture,
                &format!("{p}/referencedCapture"),
            );
        }
        for (i, catalog) in s.catalogs.iter().enumerate() {
            let p = format!("/semantics/catalogs/{i}");
            self.check("rule", &catalog.rule_id, &format!("{p}/ruleId"));
            self.owned_capture(
                &catalog.capture_site_id,
                &catalog.rule_id,
                &format!("{p}/captureSiteId"),
            );
        }
    }

    fn effects(&mut self) {
        let s = &self.ir.semantics;
        for (i, effect) in s.scope.effects.iter().enumerate() {
            let p = format!("/semantics/scope/effects/{i}");
            self.check("rule", &effect.rule_id, &format!("{p}/ruleId"));
            for (j, event) in effect.events.iter().enumerate() {
                if let Some(site) = &event.capture_site_id {
                    self.owned_capture(
                        site,
                        &effect.rule_id,
                        &format!("{p}/events/{j}/captureSiteId"),
                    );
                }
            }
        }
        for (i, display) in s.diagnostics.expected_display.iter().enumerate() {
            self.check(
                "expression",
                display.expr_id.as_str(),
                &format!("/semantics/diagnostics/expectedDisplay/{i}/exprId"),
            );
        }
        for (i, recovery) in s.recovery.iter().enumerate() {
            let p = format!("/semantics/recovery/{i}");
            self.check("rule", &recovery.rule_id, &format!("{p}/ruleId"));
            self.optional("recipe", &recovery.recipe_id, &format!("{p}/recipeId"));
            for (j, site) in recovery.application_sites.iter().enumerate() {
                let path = format!("{p}/applicationSites/{j}");
                self.check("expression", site.as_str(), &path);
                if let Some(expr) = s.expressions.iter().find(|e| e.id == *site) {
                    if !matches!(&expr.node, ExprNode::RuleRef { symbol_id } if symbol_id == &recovery.rule_id)
                    {
                        self.error(&path, "recovery site must reference the recovered rule");
                    }
                }
            }
            if recovery.mode == RecoveryMode::Auto {
                let follow = self
                    .ir
                    .analysis
                    .rules
                    .iter()
                    .find(|r| r.rule_id == recovery.rule_id)
                    .and_then(|r| r.follow_sync_patterns.as_ref());
                if follow.is_none() {
                    self.error(
                        &p,
                        "AUTO recovery requires analyzed FOLLOW; null is not an empty set",
                    );
                }
            }
        }
        for (i, rewrite) in s.normalizations.iter().enumerate() {
            let p = format!("/semantics/normalizations/{i}");
            self.check("rule", &rewrite.rule_id, &format!("{p}/ruleId"));
            self.owned_expr(
                &rewrite.original_body,
                &rewrite.rule_id,
                &format!("{p}/originalBody"),
            );
            self.owned_expr(
                &rewrite.rewritten_body,
                &rewrite.rule_id,
                &format!("{p}/rewrittenBody"),
            );
            self.proof(&rewrite.proof, &format!("{p}/proof"));
            if let Some(rule) = s.rules.iter().find(|r| r.symbol_id == rewrite.rule_id) {
                if rule.body != rewrite.rewritten_body {
                    self.error(&p, "rule body must reference rewrittenBody");
                }
            }
        }
    }

    fn proof(&mut self, proof: &Proof, path: &str) {
        self.exprs(&proof.expr_ids, &format!("{path}/exprIds"));
    }

    fn optional_proof(&mut self, proof: &Option<Proof>, path: &str) {
        if let Some(proof) = proof {
            self.proof(proof, path);
        }
    }

    fn analysis_and_plan(&mut self) {
        let ir = self.ir;
        let mut analyzed = BTreeSet::new();
        for (i, analysis) in ir.analysis.expressions.iter().enumerate() {
            let p = format!("/analysis/expressions/{i}");
            self.check(
                "expression",
                analysis.expr_id.as_str(),
                &format!("{p}/exprId"),
            );
            if !analyzed.insert(analysis.expr_id.as_str()) {
                self.error(&format!("{p}/exprId"), "duplicate expression analysis");
            }
            self.optional(
                "trie",
                &analysis.prefix_trie_id,
                &format!("{p}/prefixTrieId"),
            );
            self.memo_safety(&analysis.memo_safety, &format!("{p}/memoSafety"));
            let ranges = &analysis.may_start.ranges;
            if ranges.iter().any(|r| r.start > r.end || r.end > 0x10ffff)
                || ranges.windows(2).any(|w| w[0].end >= w[1].start)
            {
                self.error(
                    &format!("{p}/mayStart/ranges"),
                    "ranges must be valid, non-overlapping and ascending",
                );
            }
        }
        for expr in &ir.semantics.expressions {
            if !analyzed.contains(expr.id.as_str()) {
                self.error(
                    "/analysis/expressions",
                    format!("missing analysis for {}", expr.id),
                );
            }
        }
        let mut analyzed_rules = BTreeSet::new();
        for (i, analysis) in ir.analysis.rules.iter().enumerate() {
            let p = format!("/analysis/rules/{i}");
            self.check("rule", &analysis.rule_id, &format!("{p}/ruleId"));
            self.check("scc", &analysis.scc_id, &format!("{p}/sccId"));
            if !analyzed_rules.insert(&analysis.rule_id) {
                self.error(&format!("{p}/ruleId"), "duplicate rule analysis");
            }
            if let Some(scc) = ir.analysis.sccs.iter().find(|s| s.id == analysis.scc_id) {
                if !scc.rule_ids.contains(&analysis.rule_id) {
                    self.error(&format!("{p}/sccId"), "SCC does not contain the rule");
                }
            }
        }
        for (i, scc) in ir.analysis.sccs.iter().enumerate() {
            self.refs(
                "rule",
                &scc.rule_ids,
                &format!("/analysis/sccs/{i}/ruleIds"),
            );
            for rule in &scc.rule_ids {
                if !ir
                    .analysis
                    .rules
                    .iter()
                    .any(|a| &a.rule_id == rule && a.scc_id == scc.id)
                {
                    self.error(
                        &format!("/analysis/sccs/{i}/ruleIds"),
                        "SCC member disagrees with rule analysis",
                    );
                }
            }
        }
        for (i, trie) in ir.analysis.prefix_tries.iter().enumerate() {
            let p = format!("/analysis/prefixTries/{i}");
            let mut nodes = BTreeSet::new();
            for (j, node) in trie.nodes.iter().enumerate() {
                if !nodes.insert(node.index) {
                    self.error(&format!("{p}/nodes/{j}/index"), "duplicate trie node index");
                }
            }
            if !nodes.contains(&0) {
                self.error(&format!("{p}/nodes"), "missing trie root node 0");
            }
            for (j, node) in trie.nodes.iter().enumerate() {
                let np = format!("{p}/nodes/{j}");
                self.exprs(&node.candidate_expr_ids, &format!("{np}/candidateExprIds"));
                self.exprs(
                    &node.completed_candidate_expr_ids,
                    &format!("{np}/completedCandidateExprIds"),
                );
                self.exprs(
                    &node.unknown_candidate_expr_ids,
                    &format!("{np}/unknownCandidateExprIds"),
                );
                for (k, edge) in node.edges.iter().enumerate() {
                    if !nodes.contains(&edge.target) {
                        self.error(&format!("{np}/edges/{k}/target"), "unknown trie node index");
                    }
                }
            }
        }
        for (i, entry) in ir.plan.memo.entries.iter().enumerate() {
            let p = format!("/plan/memo/entries/{i}");
            self.check("expression", entry.expr_id.as_str(), &format!("{p}/exprId"));
            self.optional_proof(&entry.omission_proof, &format!("{p}/omissionProof"));
        }
        for (i, choice) in ir.plan.choices.iter().enumerate() {
            let p = format!("/plan/choices/{i}");
            self.check(
                "expression",
                choice.expr_id.as_str(),
                &format!("{p}/exprId"),
            );
            self.exprs(&choice.candidate_order, &format!("{p}/candidateOrder"));
            self.optional("trie", &choice.trie_id, &format!("{p}/trieId"));
            self.optional_proof(&choice.proof, &format!("{p}/proof"));
        }
        for (i, operator) in ir.plan.operators.iter().enumerate() {
            let p = format!("/plan/operators/{i}");
            self.refs("rule", &operator.rule_ids, &format!("{p}/ruleIds"));
            self.refs(
                "recipe",
                &operator.lift_recipe_ids,
                &format!("{p}/liftRecipeIds"),
            );
            self.exprs(&operator.boundary_expr_ids, &format!("{p}/boundaryExprIds"));
            self.optional_proof(&operator.proof, &format!("{p}/proof"));
        }
        for (i, scan) in ir.plan.scans.iter().enumerate() {
            let p = format!("/plan/scans/{i}");
            self.check("expression", scan.expr_id.as_str(), &format!("{p}/exprId"));
            self.refs("rule", &scan.rule_ids, &format!("{p}/ruleIds"));
            self.proof(&scan.proof, &format!("{p}/proof"));
            if let Some(class) = &scan.class {
                let ranges = &class.ranges;
                if ranges
                    .iter()
                    .any(|r| r.start > r.end || r.end > 0x10ffff || r.start < 128)
                    || ranges.windows(2).any(|w| w[0].end >= w[1].start)
                {
                    self.error(
                        &format!("{p}/class/ranges"),
                        "ranges must be non-ASCII, valid, non-overlapping and ascending",
                    );
                }
            }
        }
        for (i, lexical) in ir.plan.lexical.iter().enumerate() {
            let p = format!("/plan/lexical/{i}");
            self.check("token", &lexical.token_id, &format!("{p}/tokenId"));
            if let Some(dfa) = &lexical.dfa {
                let mut states = BTreeSet::new();
                for (j, state) in dfa.states.iter().enumerate() {
                    if !states.insert(state.id) {
                        self.error(&format!("{p}/dfa/states/{j}/id"), "duplicate DFA state ID");
                    }
                }
                if !states.contains(&dfa.start_state) {
                    self.error(&format!("{p}/dfa/startState"), "unknown DFA start state");
                }
                for (j, state) in dfa.states.iter().enumerate() {
                    for (k, transition) in state.transitions.iter().enumerate() {
                        if !states.contains(&transition.target) {
                            self.error(
                                &format!("{p}/dfa/states/{j}/transitions/{k}/target"),
                                "unknown DFA target state",
                            );
                        }
                    }
                }
            }
        }
    }

    fn regex_plan(&mut self, token: &Token, path: &str) {
        let TokenSpec::Regex {
            dfa: Some(dfa),
            fallback,
            regular_subset,
            java_fallback,
            ..
        } = &token.spec
        else {
            return;
        };
        let plans: Vec<_> = self
            .ir
            .plan
            .lexical
            .iter()
            .filter(|p| p.token_id == token.symbol_id)
            .collect();
        if plans.len() != 1 {
            self.error(path, "analyzed regex requires exactly one lexical plan");
            return;
        }
        let plan = plans[0];
        if let Some(dfa) = dfa {
            if !regular_subset
                || fallback.is_some()
                || *java_fallback != TokenSpecRegexJavaFallback::None
                || plan.strategy != LexicalPlanStrategy::Dfa
            {
                self.error(path, "DFA regex has inconsistent fallback or strategy");
            }
            if plan.dfa.as_ref().map(RegexDfa::from).as_ref() != Some(dfa) {
                self.error(path, "token DFA differs from lexical DFA");
            }
            if dfa.states == 0
                || dfa.start >= dfa.states
                || dfa.accepting.iter().any(|s| *s >= dfa.states)
                || dfa.accepting.windows(2).any(|w| w[0] >= w[1])
            {
                self.error(path, "invalid DFA state or accepting set");
            }
            let mut ranges = BTreeMap::<u64, Vec<[u64; 2]>>::new();
            for transition in &dfa.transitions {
                if transition.from >= dfa.states
                    || transition.to >= dfa.states
                    || transition.ranges.is_empty()
                {
                    self.error(path, "invalid DFA transition state or empty ranges");
                }
                for &[lo, hi] in &transition.ranges {
                    if lo > hi || hi > 0x10ffff {
                        self.error(path, "invalid DFA code point range");
                    }
                    ranges.entry(transition.from).or_default().push([lo, hi]);
                }
            }
            if ranges
                .values()
                .any(|rs| rs.windows(2).any(|w| w[0][1] >= w[1][0]))
            {
                self.error(path, "DFA ranges must be sorted and disjoint per state");
            }
        } else if *regular_subset
            || fallback.is_none()
            || *java_fallback != TokenSpecRegexJavaFallback::JavaUtilRegexPattern
            || plan.strategy != LexicalPlanStrategy::JavaPattern
            || plan.dfa.is_some()
        {
            self.error(path, "regex without DFA requires Java Pattern fallback");
        }
    }

    fn features(&mut self) {
        use RequiredFeature::*;
        let ir = self.ir;
        let s = &ir.semantics;
        // これらは全 IR が保持する固定契約に由来する。
        let mut needed = BTreeSet::from([
            DualCursor,
            CursorFrameFlag,
            BackendDiagnostics,
            StructuredOccurrences,
        ]);
        let mut require = |feature, used: bool| {
            if used {
                needed.insert(feature);
            }
        };
        require(Captures, !s.captures.is_empty());
        require(
            DeferredMapping,
            !s.recipes.is_empty()
                || !s.typeof_constraints.is_empty()
                || s.rules.iter().any(|r| r.mapping_type_id.is_some())
                || s.captures.iter().any(|c| c.conversion != Conversion::Text),
        );
        require(TriviaInsertion, !s.trivia.insertion_plan.is_empty());
        require(ScopeUndo, !s.scope.effects.is_empty());
        require(
            MemoReplay,
            ir.plan
                .memo
                .entries
                .iter()
                .any(|m| m.cache_success || m.cache_failure),
        );
        require(
            ExternTokens,
            s.tokens
                .iter()
                .any(|t| matches!(t.spec, TokenSpec::Extern { .. }))
                || ir
                    .plan
                    .lexical
                    .iter()
                    .any(|l| l.strategy == LexicalPlanStrategy::Extern),
        );
        require(
            Regex,
            s.tokens
                .iter()
                .any(|t| matches!(t.spec, TokenSpec::Regex { .. }))
                || ir
                    .plan
                    .lexical
                    .iter()
                    .any(|l| l.strategy == LexicalPlanStrategy::JavaPattern),
        );
        require(
            Operators,
            s.rules.iter().any(|r| r.operator.is_some()) || !ir.plan.operators.is_empty(),
        );
        // lowering 済みの履歴は backend に左再帰の処理を要求しない。
        require(Recovery, !s.recovery.is_empty());
        require(Imports, !ir.import_edges.is_empty());
        require(
            Enum,
            s.ast_types
                .iter()
                .any(|t| matches!(t, AstType::Enum { .. }))
                || s.recipes.iter().any(|r| r.kind == RecipeKind::Enum),
        );
        require(CommonField, !s.common_fields.is_empty());
        require(Eval, s.rules.iter().any(|r| r.eval.is_some()));
        require(Catalog, !s.catalogs.is_empty());
        require(Typeof, !s.typeof_constraints.is_empty());
        for expr in &s.expressions {
            if let ExprNode::Choice { selection, .. } = expr.node {
                needed.insert(match selection {
                    ChoiceSelection::Ordered => OrderedChoice,
                    ChoiceSelection::Predictive => PredictiveChoice,
                    ChoiceSelection::Longest => LongestChoice,
                });
            }
        }
        for choice in &ir.plan.choices {
            needed.insert(match choice.strategy {
                ChoicePlanStrategy::Ordered => OrderedChoice,
                ChoicePlanStrategy::Predict => PredictiveChoice,
                ChoicePlanStrategy::Longest => LongestChoice,
            });
        }
        let declared: BTreeSet<_> = ir.required_features.iter().copied().collect();
        if declared.len() != ir.required_features.len() {
            self.error("/requiredFeatures", "duplicate feature");
        }
        let names: Vec<_> = ir
            .required_features
            .iter()
            .map(|f| serde_json::to_string(f).expect("feature serialization"))
            .collect();
        if names.windows(2).any(|w| w[0] >= w[1]) {
            self.error(
                "/requiredFeatures",
                "features must be unique and sorted by name",
            );
        }
        for missing in needed.difference(&declared) {
            self.error(
                "/requiredFeatures",
                format!(
                    "missing required feature {}",
                    serde_json::to_string(missing).expect("feature serialization")
                ),
            );
        }
    }
}

impl AstType {
    pub fn id(&self) -> &str {
        match self {
            Self::Record { id, .. } | Self::TransparentUnion { id, .. } | Self::Enum { id, .. } => {
                id
            }
        }
    }

    pub fn enclosing_type_id(&self) -> &Option<String> {
        match self {
            Self::Record {
                enclosing_type_id, ..
            }
            | Self::TransparentUnion {
                enclosing_type_id, ..
            }
            | Self::Enum {
                enclosing_type_id, ..
            } => enclosing_type_id,
        }
    }
}

impl ExprNode {
    /// 文法順の構造上の子。ruleRef/tokenRef の参照先は子に含めない。
    pub fn children(&self) -> Vec<&ExprId> {
        match self {
            Self::Seq { items } => items.iter().collect(),
            Self::Choice { alternatives, .. } => alternatives.iter().collect(),
            Self::Optional { child }
            | Self::Group { child }
            | Self::Lookahead { child }
            | Self::NegativeLookahead { child } => vec![child],
            Self::Repeat {
                child, terminator, ..
            } => std::iter::once(child).chain(terminator.iter()).collect(),
            Self::Separated {
                element, separator, ..
            } => vec![element, separator],
            _ => Vec::new(),
        }
    }
}

fn canonical_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\\', '\0'])
        && !(path.as_bytes().get(1) == Some(&b':') && path.as_bytes().get(2) == Some(&b'/'))
        && path
            .split('/')
            .all(|c| !c.is_empty() && c != "." && c != "..")
}

fn source_id(path: &str) -> String {
    use std::fmt::Write;
    let mut id = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(&byte) {
            id.push(char::from(byte));
        } else {
            write!(&mut id, "%{byte:02X}").expect("String formatting");
        }
    }
    id
}
