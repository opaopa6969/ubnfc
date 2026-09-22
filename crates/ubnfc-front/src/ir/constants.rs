//! スキーマの const。値を省略せず出力し、異なる値は deserialize 時に拒否する。
use serde::{Deserialize, Serialize};

macro_rules! fixed_value {
    ($name:ident, $value:tt) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "serde_json::Value", into = "serde_json::Value")]
        pub struct $name;

        impl TryFrom<serde_json::Value> for $name {
            type Error = String;
            fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
                let expected = serde_json::json!($value);
                if value == expected {
                    Ok(Self)
                } else {
                    Err(format!(
                        "{}: expected {expected}, got {value}",
                        stringify!($name)
                    ))
                }
            }
        }
        impl From<$name> for serde_json::Value {
            fn from(_: $name) -> Self {
                serde_json::json!($value)
            }
        }
    };
}

fixed_value!(GrammarIrAnalysisVersion, "ubnfc-analysis.v1-draft1");
fixed_value!(GrammarIrSerialization, "ubnfc-json.v1");
fixed_value!(GrammarIrVersion, 1);
fixed_value!(CaptureOccurrenceOrder, "completionInnerBeforeOuter");
fixed_value!(ContractsChoice, "orderedFirstSuccess");
fixed_value!(
    ContractsLeftRecursion,
    "directRewriteExtensionIndirectError"
);
fixed_value!(ContractsLongest, "maxConsumedThenDeclarationOrder");
fixed_value!(ContractsMappingFailure, "syntaxSuccessMappingFailure");
fixed_value!(ContractsMappingPhase, "selectedResultOnly");
fixed_value!(ContractsPredictive, "conservativeFilterThenFullFallback");
fixed_value!(
    ContractsRegexPolicy,
    "regularDfaJavaPatternFallbackRustReject"
);
fixed_value!(
    ContractsRepeat,
    "terminatorThenChildThenCountThenActiveCursorProgressThenMaxThenBounds"
);
fixed_value!(ContractsRepeatZeroMax, "attemptChildBeforeCheckingMax");
fixed_value!(ContractsSpanUnit, "codePoint");
fixed_value!(CursorContractBegin, "copy;M=flag?C:M");
fixed_value!(CursorContractCommit, "copyChildCursorsOnly");
fixed_value!(CursorContractConsume, "C+=length;M=C");
fixed_value!(CursorContractCoordinate, "codePoint");
fixed_value!(CursorContractFrameFlag, "resetMatchedWithConsumed");
fixed_value!(CursorContractInternalJava, "utf16");
fixed_value!(CursorContractInternalRust, "utf8Byte");
fixed_value!(CursorContractMatchOnly, "M+=length");
fixed_value!(CursorContractMode, ["consumed", "matchOnly"]);
fixed_value!(CursorContractPositions, ["consumed", "matched"]);
fixed_value!(CursorContractRollback, "restoreParentFrame");
fixed_value!(CursorTransitionMatchedOnConsume, "resetToConsumed");
fixed_value!(DelimiterLineCommentLineEndProfile, "legacyLineTerminator");
fixed_value!(
    DiagnosticsDeepestRule,
    "javaTrialMaxConsumedElseDeepestStack"
);
fixed_value!(DiagnosticsJavaSyntaxOffset, "maxFailedFrameConsumedMatched");
fixed_value!(DiagnosticsMerge, "farthestThenUnionHints");
fixed_value!(
    DiagnosticsNegativeLookahead,
    "discardChildFailuresOnSuccess"
);
fixed_value!(DiagnosticsOffsetUnit, "codePoint");
fixed_value!(DiagnosticsRetainFarthestSeparately, true);
fixed_value!(DiagnosticsRustSyntaxOffset, "legacyRustFailureSite");
fixed_value!(DiagnosticsSuccessfulRuleFailures, "retain");
fixed_value!(DiagnosticsTrailingInputOffset, "acceptedConsumedEnd");
fixed_value!(ExprNodeChoiceTieBreak, "declarationOrder");
fixed_value!(ExprNodeSeparatedPairRollback, true);
fixed_value!(ExprNodeLiteralWordBoundary, false);
fixed_value!(MemoPlanReferenceMode, "allEligible");
fixed_value!(
    MemoPlanReplay,
    [
        "endCursors",
        "frameFlagEffects",
        "constructionRecipe",
        "captureOccurrences",
        "lexicalOccurrences",
        "scopeEffects",
        "localDiagnostics",
        "rebasedRuleStack"
    ]
);
fixed_value!(
    MemoPlanRequiredKey,
    [
        "exprId",
        "consumed",
        "matched",
        "mode",
        "invertMatch",
        "resetMatchedWithConsumed",
        "stateVersion"
    ]
);
fixed_value!(
    OccurrenceContractCaptureFields,
    ["occurrenceId", "siteId", "span", "completionOrder"]
);
fixed_value!(OccurrenceContractDiagnostics, true);
fixed_value!(
    OccurrenceContractLexicalFields,
    [
        "occurrenceId",
        "parentOccurrenceId",
        "ruleId",
        "exprId",
        "span",
        "completionOrder"
    ]
);
fixed_value!(OccurrenceContractLexicalOutput, "optional");
fixed_value!(OccurrenceContractNodeSpans, true);
fixed_value!(OccurrenceContractScopeEvents, true);
fixed_value!(
    OccurrenceContractSelfHostTransform,
    "selectedStructuredOccurrences"
);
fixed_value!(
    PlanTriviaCacheKey,
    [
        "policyId",
        "consumed",
        "matched",
        "mode",
        "resetMatchedWithConsumed"
    ]
);
fixed_value!(RecipeLeafFallbackLeft, null);
fixed_value!(RecipeLeafFallbackOp, "singleStrippedLeafText");
fixed_value!(RecipeLeafFallbackRight, []);
fixed_value!(RecipeLeafFallbackSpan, "leafConsumedExtent");
fixed_value!(RecipeOnFailure, "mappingFailure");
fixed_value!(RecipePhase, "selectedMapping");
fixed_value!(RewriteKind, "directLeftRecursionExtension");
fixed_value!(ScopeRollback, "undoLog");
fixed_value!(ScopeTrimMaxCodePoint, 32);
fixed_value!(ScopeVersionChanges, ["enter", "leave", "declare", "clear"]);
fixed_value!(TokenSpecUntilEmptyTerminator, "consumeToEof");
fixed_value!(TokenSpecUntilMissingTerminator, "consumeToEofAndSucceed");
fixed_value!(TokenSpecUntilPreserveMatched, true);
fixed_value!(TokenSpecCaseInsensitiveComparison, "javaEqualsIgnoreCase");
fixed_value!(TokenSpecRegexMatching, "lookingAt");
fixed_value!(TokenSpecRegexRustFallback, "generationError");
fixed_value!(TokenSpecRegexZeroWidth, "fail");
fixed_value!(
    TokenSpecExternResultFields,
    [
        "ok",
        "consumedEnd",
        "matchedEnd",
        "valueSpan",
        "diagnostics",
        "effects"
    ]
);
fixed_value!(
    TriviaPolicyPrecedence,
    "ruleWhitespaceThenRuleInterleaveThenGrammar"
);
fixed_value!(TypeofComparisonKind, "runtimeClassEquality");
fixed_value!(TypeofMismatch, "mappingFailure");
fixed_value!(TypeofPhase, "mapping");
fixed_value!(ValueTypeNumberJava, "int32Exact");
fixed_value!(ValueTypeNumberRust, "lexeme");
