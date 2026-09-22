//! Grammar IR v1 の全定義。意味順を持つ配列は Vec、自由キーのマップは BTreeMap。

use super::constants::*;
use super::ExprId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RequiredFeature {
    #[serde(rename = "dualCursor")]
    DualCursor,
    #[serde(rename = "cursorFrameFlag")]
    CursorFrameFlag,
    #[serde(rename = "orderedChoice")]
    OrderedChoice,
    #[serde(rename = "predictiveChoice")]
    PredictiveChoice,
    #[serde(rename = "longestChoice")]
    LongestChoice,
    #[serde(rename = "triviaInsertion")]
    TriviaInsertion,
    #[serde(rename = "captures")]
    Captures,
    #[serde(rename = "deferredMapping")]
    DeferredMapping,
    #[serde(rename = "scopeUndo")]
    ScopeUndo,
    #[serde(rename = "backendDiagnostics")]
    BackendDiagnostics,
    #[serde(rename = "memoReplay")]
    MemoReplay,
    #[serde(rename = "externTokens")]
    ExternTokens,
    #[serde(rename = "regex")]
    Regex,
    #[serde(rename = "operators")]
    Operators,
    #[serde(rename = "leftRecursionRewrite")]
    LeftRecursionRewrite,
    #[serde(rename = "recovery")]
    Recovery,
    #[serde(rename = "imports")]
    Imports,
    #[serde(rename = "enum")]
    Enum,
    #[serde(rename = "commonField")]
    CommonField,
    #[serde(rename = "eval")]
    Eval,
    #[serde(rename = "catalog")]
    Catalog,
    #[serde(rename = "typeof")]
    Typeof,
    #[serde(rename = "structuredOccurrences")]
    StructuredOccurrences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Cardinality {
    #[serde(rename = "scalar")]
    Scalar,
    #[serde(rename = "optional")]
    Optional,
    #[serde(rename = "list")]
    List,
}

impl Cardinality {
    /// 量指定子 `{min,max}` が**一度の成功解析で共存させ得る値の個数**。
    /// 名前が文法に何回書かれているかではない（`docs/nested-capture-migration.md`）。
    /// `max > 1`（上限なしを含む）だけが List、`{1}` は Scalar、`{0}` / `{0,1}` は Optional。
    pub fn quantified(min: u64, max: Option<u64>) -> Self {
        match max {
            Some(0 | 1) if min == 0 => Self::Optional,
            Some(1) => Self::Scalar,
            Some(_) | None => Self::List,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BoundaryAt {
    #[serde(rename = "entry")]
    Entry,
    #[serde(rename = "afterChild")]
    AfterChild,
    #[serde(rename = "exit")]
    Exit,
    #[serde(rename = "helperEntry")]
    HelperEntry,
    #[serde(rename = "helperExit")]
    HelperExit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Conversion {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "node")]
    Node,
    #[serde(rename = "mixed")]
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CaptureSpanRule {
    #[serde(rename = "consumedExtent")]
    ConsumedExtent,
    #[serde(rename = "selectedChildExtent")]
    SelectedChildExtent,
    #[serde(rename = "matchedExtent")]
    MatchedExtent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ChoicePlanFallback {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "fullChoice")]
    FullChoice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ChoicePlanStrategy {
    #[serde(rename = "ordered")]
    Ordered,
    #[serde(rename = "predict")]
    Predict,
    #[serde(rename = "longest")]
    Longest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CursorProgress {
    #[serde(rename = "preserve")]
    Preserve,
    #[serde(rename = "advance")]
    Advance,
    #[serde(rename = "mayAdvance")]
    MayAdvance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CursorTransitionFailure {
    #[serde(rename = "restore")]
    Restore,
    #[serde(rename = "mayResetMatched")]
    MayResetMatched,
    #[serde(rename = "contextual")]
    Contextual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CursorTransitionFlagEffect {
    #[serde(rename = "preserve")]
    Preserve,
    #[serde(rename = "setFalse")]
    SetFalse,
    #[serde(rename = "contextual")]
    Contextual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EvalStrategy {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "manual")]
    Manual,
    #[serde(rename = "abstractCallback")]
    AbstractCallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ExecutionChildMode {
    #[serde(rename = "inherit")]
    Inherit,
    #[serde(rename = "matchOnly")]
    MatchOnly,
    #[serde(rename = "calleeDefault")]
    CalleeDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ExecutionEntryFlag {
    #[serde(rename = "preserve")]
    Preserve,
    #[serde(rename = "setFalse")]
    SetFalse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ExecutionFailureCursor {
    #[serde(rename = "restoreTransaction")]
    RestoreTransaction,
    #[serde(rename = "primitiveMayResetMatched")]
    PrimitiveMayResetMatched,
    #[serde(rename = "delegate")]
    Delegate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ExecutionTransaction {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "once")]
    Once,
    #[serde(rename = "perAlternative")]
    PerAlternative,
    #[serde(rename = "occurs")]
    Occurs,
    #[serde(rename = "delegate")]
    Delegate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ChoiceSelection {
    #[serde(rename = "ordered")]
    Ordered,
    #[serde(rename = "predictive")]
    Predictive,
    #[serde(rename = "longest")]
    Longest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LexicalPlanStrategy {
    #[serde(rename = "scalar")]
    Scalar,
    #[serde(rename = "dfa")]
    Dfa,
    #[serde(rename = "javaPattern")]
    JavaPattern,
    #[serde(rename = "extern")]
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MayStartCertainty {
    #[serde(rename = "exact")]
    Exact,
    #[serde(rename = "conservative")]
    Conservative,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MayStartZeroConsumeCondition {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "always")]
    Always,
    #[serde(rename = "atEof")]
    AtEof,
    #[serde(rename = "predicate")]
    Predicate,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MemoEntryReason {
    #[serde(rename = "defaultSafe")]
    DefaultSafe,
    #[serde(rename = "unsafeExternClosure")]
    UnsafeExternClosure,
    #[serde(rename = "provedOmission")]
    ProvedOmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MemoSafetyProvenance {
    #[serde(rename = "builtinProof")]
    BuiltinProof,
    #[serde(rename = "userAliasAssertion")]
    UserAliasAssertion,
    #[serde(rename = "derivedProof")]
    DerivedProof,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MemoSafetyPurity {
    #[serde(rename = "pure")]
    Pure,
    #[serde(rename = "stateRead")]
    StateRead,
    #[serde(rename = "effectful")]
    Effectful,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OperatorAssociativity {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OperatorCanonicalForm {
    #[serde(rename = "leftOpRightLists")]
    LeftOpRightLists,
    #[serde(rename = "baseOpSelfOrBase")]
    BaseOpSelfOrBase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OperatorPlanStrategy {
    #[serde(rename = "tierFunctions")]
    TierFunctions,
    #[serde(rename = "precedenceClimbing")]
    PrecedenceClimbing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PlanProfile {
    #[serde(rename = "conservative")]
    Conservative,
    #[serde(rename = "optimized")]
    Optimized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ProofKind {
    #[serde(rename = "unreachableRevisit")]
    UnreachableRevisit,
    #[serde(rename = "singleExecutionPath")]
    SingleExecutionPath,
    #[serde(rename = "exclusiveCandidates")]
    ExclusiveCandidates,
    #[serde(rename = "monotoneNoRollback")]
    MonotoneNoRollback,
    #[serde(rename = "diagnosticEquivalent")]
    DiagnosticEquivalent,
    #[serde(rename = "noTriviaEffectsRecovery")]
    NoTriviaEffectsRecovery,
    #[serde(rename = "leftRecursionNormalization")]
    LeftRecursionNormalization,
    #[serde(rename = "none")]
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecipeKind {
    #[serde(rename = "construct")]
    Construct,
    #[serde(rename = "transparent")]
    Transparent,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "enum")]
    Enum,
    #[serde(rename = "assocLists")]
    AssocLists,
    #[serde(rename = "skip")]
    Skip,
    #[serde(rename = "frontendTransform")]
    FrontendTransform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecoveryAstProjection {
    #[serde(rename = "legacyMarker")]
    LegacyMarker,
    #[serde(rename = "mappingFailure")]
    MappingFailure,
    #[serde(rename = "explicitRecipe")]
    ExplicitRecipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecoveryMode {
    #[serde(rename = "SYNC")]
    Sync,
    #[serde(rename = "AUTO")]
    Auto,
    #[serde(rename = "SKIP")]
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecoveryNoSyncAction {
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "consumeToEof")]
    ConsumeToEof,
    #[serde(rename = "succeed")]
    Succeed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RecoveryProgressRule {
    #[serde(rename = "legacySync")]
    LegacySync,
    #[serde(rename = "legacySkip")]
    LegacySkip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RuleAnalysisLeftRecursion {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "directRewritten")]
    DirectRewritten,
    #[serde(rename = "indirectRejected")]
    IndirectRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SccMemoPolicy {
    #[serde(rename = "allEligible")]
    AllEligible,
    #[serde(rename = "provedOmissions")]
    ProvedOmissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScopeEffectEventsAction {
    #[serde(rename = "enter")]
    Enter,
    #[serde(rename = "leave")]
    Leave,
    #[serde(rename = "declare")]
    Declare,
    #[serde(rename = "use")]
    Use,
    #[serde(rename = "clear")]
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScopeEffectEventsWhen {
    #[serde(rename = "entry")]
    Entry,
    #[serde(rename = "successBeforeLeave")]
    SuccessBeforeLeave,
    #[serde(rename = "successAfterLeave")]
    SuccessAfterLeave,
    #[serde(rename = "afterDeclarations")]
    AfterDeclarations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScopeEffectMode {
    #[serde(rename = "lexical")]
    Lexical,
    #[serde(rename = "dynamic")]
    Dynamic,
    #[serde(rename = "sameRuleText")]
    SameRuleText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SettingsWhitespace {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "javaStyle")]
    JavaStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SymbolKind {
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "token")]
    Token,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BuiltinToken {
    #[serde(rename = "Number")]
    Number,
    #[serde(rename = "Identifier")]
    Identifier,
    #[serde(rename = "SingleQuoted")]
    SingleQuoted,
    #[serde(rename = "Quoted")]
    Quoted,
    #[serde(rename = "Digit")]
    Digit,
    #[serde(rename = "Alphabet")]
    Alphabet,
    #[serde(rename = "AlphabetUnderScore")]
    AlphabetUnderScore,
    #[serde(rename = "AlphabetNumericUnderScore")]
    AlphabetNumericUnderScore,
    #[serde(rename = "Space")]
    Space,
    #[serde(rename = "EndOfSource")]
    EndOfSource,
    #[serde(rename = "CodePointSet")]
    CodePointSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TokenSpecRegexJavaFallback {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "java.util.regex.Pattern")]
    JavaUtilRegexPattern,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub grammar_id: String,
    pub rule_id: String,
}

/// ubnfc 設計 v1 §3・§4 の実装境界の草案。semantics（認識・trivia・capture・mapping・scope・診断）と analysis（front の解析結果）、plan（最適化）を分離する。analysisVersion と requiredFeatures の未対応値を backend は明示拒否する。D-013（カーソル frame flag）、D-011 の言語別診断、D-012 の構造付きフロント変換は今回のレビューによる補足提案で、実装済みを意味しない。
/// 直列化契約 ubnfc-json.v1: UTF-8、BOMなし、LF、インデント2空白、末尾LF1個。全 object/map のキーを Unicode scalar 昇順に再帰ソート。整数は10進（先頭0・+・-0なし）、この IR の実行数値に浮動小数を使わない。文字列は Unicode を正規化せず非ASCIIをそのまま出力し、quote/backslash と制御文字のみ JSON escape（\b,\t,\n,\f,\r 優先、他は小文字hexの\u00xx）。/ を escape しない。source path は論理ソースルート相対 POSIX、. を除去し .. を解決、ルート越えと絶対パスを拒否。sourceId/ExprId は各定義に従い、作業ディレクトリやランダム採番に依存しない。
/// 配列は意味のある順序を保持する。特に source/grammar/import の宣言順、統合シンボルの import 展開順、rule、choice、同長 longest、capture site、AST field/enum、scope event、trivia insertion は並べ替え禁止。expressions は rule 宣言順の構文前順（親→文法順の子）。analysis と plan の式別配列も同じ順。集合である ranges/codePoints と feature 名だけ規定の昇順、SCC は最初の所属 rule 順。動的 capture は完了順・同名 nested は内側→外側。Schema 検証は object キー順やこの直列化形式を強制できないため serializer の別検証が必要。
/// 意味検証の責務: ID一意性・参照存在・ExprIdとsource spanの対応・全Expr解析網羅・min<=max・範囲端点・rule所属・capture/recipe/typeof/catalog型整合・memo安全性と省略証明・climbing前提・recovery適用site・import循環と未対応機能をフロントで検証。Schema 単独ではこれらの横断制約や解析結果の正しさを証明しない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrammarIr {
    pub analysis: Analysis,
    pub analysis_version: GrammarIrAnalysisVersion,
    pub entry: Entry,
    pub grammar_units: Vec<GrammarUnit>,
    pub import_edges: Vec<ImportEdge>,
    pub plan: Plan,
    pub required_features: Vec<RequiredFeature>,
    pub semantics: Semantics,
    pub serialization: GrammarIrSerialization,
    pub sources: Vec<Source>,
    pub version: GrammarIrVersion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Analysis {
    pub expressions: Vec<ExprAnalysis>,
    pub prefix_tries: Vec<PrefixTrie>,
    pub rules: Vec<RuleAnalysis>,
    pub sccs: Vec<Scc>,
}

/// 原注釈を宣言順で保存する。実行意味論は各構造化フィールドに解決済み。未知注釈を黙って実行意味論に変換しない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Annotation {
    pub arguments: BTreeMap<String, JsonValue>,
    pub name: String,
    pub source: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AstField {
    pub cardinality: Cardinality,
    pub name: String,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnumVariant {
    pub java_name: String,
    pub literal: String,
    pub rust_name: String,
    pub variant: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AstType {
    Record {
        #[serde(deserialize_with = "super::json::required_nullable")]
        enclosing_type_id: Option<String>,
        fields: Vec<AstField>,
        id: String,
        implements: Vec<String>,
        name: String,
    },
    TransparentUnion {
        #[serde(deserialize_with = "super::json::required_nullable")]
        enclosing_type_id: Option<String>,
        id: String,
        name: String,
        variants: Vec<String>,
    },
    Enum {
        #[serde(deserialize_with = "super::json::required_nullable")]
        enclosing_type_id: Option<String>,
        id: String,
        name: String,
        rule_id: String,
        variants: Vec<EnumVariant>,
    },
}

/// afterChild の childIndex は owner の文法順0始まり。その他は null。単一要素を helper 化しない既存生成境界も保持する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Boundary {
    pub at: BoundaryAt,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub child_index: Option<u64>,
    pub owner_expr_id: ExprId,
}

/// siteId は ExprId + /capture/<同一式内の文法順番号>。静的 site と動的 occurrence は別。空 span を削除しない。数値・型検査を採用結果の mapping まで遅延する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capture {
    pub cardinality: Cardinality,
    pub conversion: Conversion,
    pub expr_id: ExprId,
    pub name: String,
    pub occurrence_order: CaptureOccurrenceOrder,
    pub rule_id: String,
    pub site_id: String,
    pub skip: bool,
    pub span_rule: CaptureSpanRule,
    pub value_type: ValueType,
}

/// 対象 capture の不在は検証エラー。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Catalog {
    pub capture_site_id: String,
    pub context: String,
    pub rule_id: String,
}

/// 両端を含む code point 集合。start<=end、重複なし・昇順を意味検証する。surrogate の扱いはバックエンド入力契約と照合する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CharRange {
    pub end: u64,
    pub start: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoicePlan {
    pub candidate_order: Vec<ExprId>,
    pub expr_id: ExprId,
    pub fallback: ChoicePlanFallback,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub proof: Option<Proof>,
    pub strategy: ChoicePlanStrategy,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub trie_id: Option<String>,
}

/// @commonField は対象 variant 全体の型・cardinality を検証済み。Java は抽象メソッドとして生成。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommonField {
    pub field: AstField,
    pub interface_type_id: String,
    pub variant_type_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contracts {
    pub choice: ContractsChoice,
    pub cursor: CursorContract,
    pub left_recursion: ContractsLeftRecursion,
    pub longest: ContractsLongest,
    pub mapping_failure: ContractsMappingFailure,
    pub mapping_phase: ContractsMappingPhase,
    pub occurrences: OccurrenceContract,
    pub predictive: ContractsPredictive,
    pub regex_policy: ContractsRegexPolicy,
    pub repeat: ContractsRepeat,
    pub repeat_zero_max: ContractsRepeatZeroMax,
    pub span_unit: ContractsSpanUnit,
}

/// D-013 補足案。M は最遠位置ではなく現在の照合位置で、巻戻し・consume(0) により減る。状態には mode / invertMatch / frameFlag が必要。Chain は親 flag=false にしてから begin。MatchOnly/Not は matchOnly mode を強制。EMPTY=Optional(MatchOnly(ANY))、EOF=Occurs(ANY,0,0)。UNTIL の terminator は calleeDefault で呼び、実際の Word は MatchOnly により M から読む。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CursorContract {
    pub begin: CursorContractBegin,
    pub commit: CursorContractCommit,
    pub consume: CursorContractConsume,
    pub coordinate: CursorContractCoordinate,
    pub frame_flag: CursorContractFrameFlag,
    pub internal_java: CursorContractInternalJava,
    pub internal_rust: CursorContractInternalRust,
    pub match_only: CursorContractMatchOnly,
    pub mode: CursorContractMode,
    pub positions: CursorContractPositions,
    pub rollback: CursorContractRollback,
}

/// モード別に「選択されたカーソル」が進むかの保守的要約。matched は consume や begin で後退し得る。詳細遷移は semantics の execution と cursor 契約から決まり、この要約だけで書き換えない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CursorTransition {
    pub consumed_mode: CursorProgress,
    pub failure: CursorTransitionFailure,
    pub flag_effect: CursorTransitionFlagEffect,
    pub matched_mode: CursorProgress,
    pub matched_on_consume: CursorTransitionMatchedOnConsume,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Delimiter {
    Characters {
        code_points: Vec<u64>,
    },
    LineComment {
        line_end_profile: DelimiterLineCommentLineEndProfile,
        open: String,
    },
    BlockComment {
        close: String,
        open: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExpectedDisplay {
    pub expr_id: ExprId,
    pub java: String,
    pub rust: String,
}

/// D-011: offset と farthestOffset を分離。Java syntax は失敗 frame 内 max(C,M) の最遠、失敗なしなら最大到達点。Rust は fail_at の記録位置（内部 byte→code point）。trailing_input は採用 prefix の消費末尾。expected は言語別表示契約。Java facade は expectedTokens をソート。corpus は一般の hints 完全一致や deepest rule を検証していない。負の先読みは成功時に子の失敗診断を捨て、自身の失敗は記録。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Diagnostics {
    pub deepest_rule: DiagnosticsDeepestRule,
    pub expected_display: Vec<ExpectedDisplay>,
    pub java_syntax_offset: DiagnosticsJavaSyntaxOffset,
    pub merge: DiagnosticsMerge,
    pub negative_lookahead: DiagnosticsNegativeLookahead,
    pub offset_unit: DiagnosticsOffsetUnit,
    pub retain_farthest_separately: DiagnosticsRetainFarthestSeparately,
    pub rust_syntax_offset: DiagnosticsRustSyntaxOffset,
    pub successful_rule_failures: DiagnosticsSuccessfulRuleFailures,
    pub trailing_input_offset: DiagnosticsTrailingInputOffset,
}

/// 自動 kind は binary_arithmetic / variable_ref / conditional / passthrough / literal。manual・未知 kind は abstractCallback。透明な和型の自動 eval は意味検証で拒否。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Eval {
    pub kind: String,
    pub params: BTreeMap<String, JsonValue>,
    pub strategy: EvalStrategy,
}

/// D-013 補足案。mode と transaction の resetMatchedWithConsumed を意味論として保持する。begin は flag=true なら M=C、false なら M を継承。consumed の長さ0加算でも M=C に戻る。commit は子の両位置だけを親へコピーし、flag はコピーしない。rollback は親 frame に戻る。フラグを親 frame へ書く処理は子 rollback だけでは取り消されない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Execution {
    pub child_mode: ExecutionChildMode,
    pub entry_flag: ExecutionEntryFlag,
    pub failure_cursor: ExecutionFailureCursor,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub primitive_profile: Option<String>,
    pub transaction: ExecutionTransaction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Expr {
    pub capture_site_ids: Vec<String>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub diagnostic_label: Option<String>,
    pub execution: Execution,
    pub id: ExprId,
    pub node: ExprNode,
    pub rule_id: String,
    pub source: Span,
}

/// minConsume は consumed mode で成功時の code point 消費下限。未知は0。nullable はゼロ消費成功が可能（不明も true）。trivia を含む mayStart と素の字句 FIRST を混同しない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExprAnalysis {
    pub cursor_transition: CursorTransition,
    pub effects: Vec<String>,
    pub expr_id: ExprId,
    pub may_start: MayStart,
    pub memo_safety: MemoSafety,
    pub min_consume: u64,
    pub nullable: bool,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub prefix_trie_id: Option<String>,
}

/// repeat は先に上限を判定しない。周回開始の getPosition(mode) と子成功後を比較し、同値でもその周回を数える。terminator は外側 mode を渡さず既定呼出し。成功かつ外側 consumed mode なら terminator の tokenKind で commit、それ以外は rollback。terminator 成功なら count を増やさず停止する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ExprNode {
    Seq {
        items: Vec<ExprId>,
    },
    Choice {
        alternatives: Vec<ExprId>,
        selection: ChoiceSelection,
        tie_break: ExprNodeChoiceTieBreak,
    },
    Optional {
        child: ExprId,
    },
    Repeat {
        child: ExprId,
        #[serde(deserialize_with = "super::json::required_nullable")]
        max: Option<u64>,
        min: u64,
        #[serde(deserialize_with = "super::json::required_nullable")]
        terminator: Option<ExprId>,
    },
    Separated {
        element: ExprId,
        min: u64,
        pair_rollback: ExprNodeSeparatedPairRollback,
        separator: ExprId,
    },
    Group {
        child: ExprId,
    },
    Literal {
        case_sensitive: bool,
        value: String,
        word_boundary: ExprNodeLiteralWordBoundary,
    },
    RuleRef {
        symbol_id: String,
    },
    TokenRef {
        symbol_id: String,
    },
    Error {
        message: String,
    },
    Lookahead {
        child: ExprId,
    },
    NegativeLookahead {
        child: ExprId,
    },
    Any {},
    Eof {},
    Empty {},
}

/// GrammarUnit は出所 metadata。実行時シンボル表は semantics.symbols に統合済み。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrammarUnit {
    pub annotations: Vec<Annotation>,
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub package: Option<String>,
    pub raw_settings: Settings,
    pub source: Span,
    pub source_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportBinding {
    pub local_name: String,
    pub qualified_symbol_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportEdge {
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub alias: Option<String>,
    pub bindings: Vec<ImportBinding>,
    pub from_grammar_id: String,
    pub requested_path: String,
    pub resolved_grammar_id: String,
    pub source_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DfaTransition {
    pub range: CharRange,
    pub target: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DfaState {
    pub accepting: bool,
    pub id: u64,
    pub transitions: Vec<DfaTransition>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Dfa {
    pub start_state: u64,
    pub states: Vec<DfaState>,
}

/// token に付随する optional 計画。状態 ID は 0..states、範囲は code point 閉区間。
/// 優先順位付き構成の受理到達で候補を更新し、入力終端または遷移なしで最後の候補を採用する。
/// Java lookingAt と同じ leftmost-first。最終候補がゼロ幅なら成功しない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegexDfa {
    pub states: u64,
    pub start: u64,
    pub accepting: Vec<u64>,
    pub transitions: Vec<RegexDfaTransition>,
    pub anchored_at_start: RegexDfaAnchoredAtStart,
    pub semantics: RegexDfaSemantics,
    pub zero_width: TokenSpecRegexZeroWidth,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegexDfaTransition {
    pub from: u64,
    pub ranges: Vec<[u64; 2]>,
    pub to: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LexicalPlan {
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub dfa: Option<Dfa>,
    pub strategy: LexicalPlanStrategy,
    pub token_id: String,
}

/// 先頭文字の必要条件の上界。負の先読みは FIRST の単純な補集合にしない。unknown（Extern を含む）は全候補を残す。nullable は空文字受理とは限らずゼロ消費成功可能性を表す。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MayStart {
    pub certainty: MayStartCertainty,
    pub eof: bool,
    pub includes_trivia: bool,
    pub ranges: Vec<CharRange>,
    pub zero_consume_condition: MayStartZeroConsumeCondition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemoEntry {
    pub cache_failure: bool,
    pub cache_success: bool,
    pub expr_id: ExprId,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub omission_proof: Option<Proof>,
    pub reason: MemoEntryReason,
}

/// 全 memo は安全な領域内の全 memo。Extern 禁止領域まで memo しない。scope 効果は順序を保ち1回だけ再生。省略は証明付き。D-013 で frame flag と invertMatch を明示追加。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemoPlan {
    pub entries: Vec<MemoEntry>,
    pub reference_mode: MemoPlanReferenceMode,
    pub replay: MemoPlanReplay,
    pub required_key: MemoPlanRequiredKey,
}

/// @memoSafeToken は alias 単位の保証。成功時の再現性も含む。保証のない Extern と到達可能な祖先は memo 不可。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemoSafety {
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub asserted_alias: Option<String>,
    pub failure: bool,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub original_declaration: Option<String>,
    pub provenance: MemoSafetyProvenance,
    pub purity: MemoSafetyPurity,
    pub reason: String,
    pub success: bool,
}

/// D-012 推奨。静的 RuleId/ExprId だけでは再帰呼出しの所属を復元できない。フロント変換では採用枝と親 occurrence、必要な非終端完了イベントを選択保持する。汎用 lexical 出力だけから再構文解析しない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OccurrenceContract {
    pub capture_fields: OccurrenceContractCaptureFields,
    pub diagnostics: OccurrenceContractDiagnostics,
    pub lexical_fields: OccurrenceContractLexicalFields,
    pub lexical_output: OccurrenceContractLexicalOutput,
    pub node_spans: OccurrenceContractNodeSpans,
    pub scope_events: OccurrenceContractScopeEvents,
    pub self_host_transform: OccurrenceContractSelfHostTransform,
}

/// 数値 precedence は検証・metadata。実際の優先順位は参照階層。右結合も既存の各段ノードを復元する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Operator {
    pub associativity: OperatorAssociativity,
    pub base_rule_id: String,
    pub canonical_form: OperatorCanonicalForm,
    pub left_site_id: String,
    pub op_site_id: String,
    pub operator_rule_id: String,
    pub precedence: i64,
    pub right_site_id: String,
}

/// climbing は trivia・scope・recovery・先読みのない正規形で差分比較済みの場合のみ。lift は各段 AST/span を復元。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperatorPlan {
    pub boundary_expr_ids: Vec<ExprId>,
    pub lift_recipe_ids: Vec<String>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub proof: Option<Proof>,
    pub rule_ids: Vec<String>,
    pub strategy: OperatorPlanStrategy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriviaCache {
    pub enabled: bool,
    pub key: PlanTriviaCacheKey,
}

/// 最適化計画。semantics で定義した意味を変更しない。前提証明と未対応 feature/version を検査してから使用。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plan {
    pub choices: Vec<ChoicePlan>,
    pub lexical: Vec<LexicalPlan>,
    pub memo: MemoPlan,
    pub operators: Vec<OperatorPlan>,
    pub profile: PlanProfile,
    pub trivia_cache: TriviaCache,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrieEdge {
    pub code_point: u64,
    pub target: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrieNode {
    pub accepts_eof: bool,
    pub candidate_expr_ids: Vec<ExprId>,
    pub completed_candidate_expr_ids: Vec<ExprId>,
    pub edges: Vec<TrieEdge>,
    pub index: u64,
    pub unknown_candidate_expr_ids: Vec<ExprId>,
}

/// 候補順は元の選択肢順。未知候補を保持。node0 が root、prefix は code point。 completedCandidateExprIds は現在 prefix の途中までに既に完全一致した候補。後続辺がなくても保持する（var は variable の途中で除外できない）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrefixTrie {
    pub id: String,
    pub max_depth: u64,
    pub nodes: Vec<TrieNode>,
}

/// 説明だけを証明器と見なさない。解析版と式参照に結びつく front の検証結果。証明不能なら保守経路。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Proof {
    pub explanation: String,
    pub expr_ids: Vec<ExprId>,
    pub kind: ProofKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeField {
    pub cardinality: Cardinality,
    pub conversion: Conversion,
    pub field: String,
    pub site_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LeafFallback {
    pub left: RecipeLeafFallbackLeft,
    pub op: RecipeLeafFallbackOp,
    pub right: RecipeLeafFallbackRight,
    pub span: RecipeLeafFallbackSpan,
    pub type_id: String,
}

/// CST オブジェクト木を要さない構築レシピ。transformId は版管理されたフロント専用変換。自己ホストでは手書き UBNF parser に戻らず、採用枝・呼出し occurrence と対応した構造イベントを使う。 assocLists の leafFallback は既存の共有結合型が未 mapping 葉を同型の (null,[字句],[]) に包む契約。一般の number capture の数値変換とは別。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    pub fields: Vec<RecipeField>,
    pub id: String,
    pub kind: RecipeKind,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub leaf_fallback: Option<LeafFallback>,
    pub on_failure: RecipeOnFailure,
    pub phase: RecipePhase,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub result_type_id: Option<String>,
    pub rule_id: String,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub transform_id: Option<String>,
}

/// AUTO の FOLLOW が空なら ;。SYNC は同期文字列を消費し未検出なら失敗。ruleRef の適用 site を明記し root へ暗黙適用しない。typed AST への投影は独立に指定。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recovery {
    pub application_sites: Vec<ExprId>,
    pub ast_projection: RecoveryAstProjection,
    pub consume_sync: bool,
    pub mode: RecoveryMode,
    pub no_sync_action: RecoveryNoSyncAction,
    pub progress_rule: RecoveryProgressRule,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub recipe_id: Option<String>,
    pub resolved_sync_patterns: Vec<String>,
    pub rule_id: String,
}

/// 元の順序付き選択と対応を保つ。nullable suffix・capture・trivia の投影規則を確定できない書換えを黙って適用しない。元式も IR に保持する。 最適化ではなく D-010 の拡張意味論の lowering 記録。Rule.body は rewrittenBody を参照する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rewrite {
    pub kind: RewriteKind,
    pub original_body: ExprId,
    pub proof: Proof,
    pub rewritten_body: ExprId,
    pub rule_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    pub annotations: Vec<Annotation>,
    pub body: ExprId,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub eval: Option<Eval>,
    pub mapping_params: Vec<String>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub mapping_type_id: Option<String>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub operator: Option<Operator>,
    pub skip: bool,
    pub symbol_id: String,
    pub trivia_policy_id: String,
}

/// FOLLOW の同期文字列は FIRST の文字集合と別。生成可能 IR で indirectRejected は許容しない。 recovery が要求しないため未解析なら null。解析済みの空集合 [] と区別し、null を AUTO の ; fallback に使ってはならない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuleAnalysis {
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub follow_sync_patterns: Option<Vec<String>>,
    pub left_recursion: RuleAnalysisLeftRecursion,
    pub progress_proven: bool,
    pub rule_id: String,
    pub scc_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scc {
    pub id: String,
    pub memo_policy: SccMemoPolicy,
    pub recursive: bool,
    pub rule_ids: Vec<String>,
}

/// 同名再宣言の旧 lookup、pop 済み frame、診断 clear を undo に保存する。版 ID は分岐間で異なる状態に再利用しない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    pub effects: Vec<ScopeEffect>,
    pub rollback: ScopeRollback,
    pub trim_max_code_point: ScopeTrimMaxCodePoint,
    pub version_changes: ScopeVersionChanges,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeEvent {
    pub action: ScopeEffectEventsAction,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub capture_site_id: Option<String>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub name: Option<String>,
    pub when: ScopeEffectEventsWhen,
}

/// 開始 enter、成功 leave、同 rule の declare は leave 後に親へ登録、declare 後に use。scope なし backref は同 rule 内テキスト一致、ERROR 診断。scope あり未定義 use は WARNING で構文成功。trim は U+0000..U+0020。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeEffect {
    pub events: Vec<ScopeEvent>,
    pub mode: ScopeEffectMode,
    pub rule_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Semantics {
    pub ast_types: Vec<AstType>,
    pub captures: Vec<Capture>,
    pub catalogs: Vec<Catalog>,
    pub common_fields: Vec<CommonField>,
    pub contracts: Contracts,
    pub diagnostics: Diagnostics,
    pub expressions: Vec<Expr>,
    pub normalizations: Vec<Rewrite>,
    pub recipes: Vec<Recipe>,
    pub recovery: Vec<Recovery>,
    pub rules: Vec<Rule>,
    pub scope: Scope,
    pub symbols: Vec<Symbol>,
    pub tokens: Vec<Token>,
    pub trivia: Trivia,
    pub typeof_constraints: Vec<Typeof>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommentSettings {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::json::present_value"
    )]
    pub block_close: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::json::present_value"
    )]
    pub block_open: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "super::json::present_value"
    )]
    pub line: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub comment: Option<CommentSettings>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub interleave: Option<String>,
    #[serde(deserialize_with = "super::json::required_nullable")]
    pub whitespace: Option<SettingsWhitespace>,
}

/// path は明示したソースルート相対の POSIX パス。絶対パス・環境固有の作業ディレクトリを記録しない。sha256 は改行を含む原 UTF-8 バイト列。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Source {
    pub code_point_length: u64,
    pub id: String,
    pub path: String,
    pub sha256: String,
}

/// 文法ソース上の code point 半開区間。start <= end を意味検証する。入力解析結果の span と区別する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Span {
    pub end: u64,
    pub source_id: String,
    pub start: u64,
}

/// import alias を prefix した名前もここで解決する。id と qualifiedSymbolId の一意性、参照解決はフロントで検証する。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Symbol {
    pub grammar_id: String,
    pub id: String,
    pub kind: SymbolKind,
    pub name: String,
    pub original_name: String,
    pub qualified_symbol_id: String,
    pub source: Span,
    pub source_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TokenDiagnosticProfiles {
    pub java: String,
    pub rust: String,
}

/// Number は ASCII 数字の符号・小数・指数、Identifier は ASCII [A-Za-z_][A-Za-z0-9_]*。引用 token の escape は任意1 code point。builtin CodePointSet は集合を明記。charRange の surrogate 境界は意味検証で拒否。空 UNTIL は既存生成器で終端なし相当。 diagnosticProfiles は既知組込みの言語別観測契約。Java Number は ExponentParser の試行失敗を記録し、Rust Number の不完全指数巻戻しは診断を追加しない。backend が共通スキャナへの統合でこの差を消してはならない。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Token {
    pub annotations: Vec<Annotation>,
    pub diagnostic_profiles: TokenDiagnosticProfiles,
    pub memo_safety: MemoSafety,
    pub spec: TokenSpec,
    pub symbol_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TokenSpec {
    Builtin {
        code_points: Vec<u64>,
        which: BuiltinToken,
    },
    Until {
        empty_terminator: TokenSpecUntilEmptyTerminator,
        missing_terminator: TokenSpecUntilMissingTerminator,
        preserve_matched: TokenSpecUntilPreserveMatched,
        terminator: String,
    },
    Negation {
        excluded: String,
    },
    Lookahead {
        pattern: String,
    },
    NegativeLookahead {
        pattern: String,
    },
    CharRange {
        max: u64,
        min: u64,
    },
    CaseInsensitive {
        comparison: TokenSpecCaseInsensitiveComparison,
        word: String,
    },
    Regex {
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::json::present_value"
        )]
        dfa: Option<Option<RegexDfa>>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::json::present_value"
        )]
        fallback: Option<RegexFallback>,
        java_fallback: TokenSpecRegexJavaFallback,
        matching: TokenSpecRegexMatching,
        pattern: String,
        regular_subset: bool,
        rust_fallback: TokenSpecRegexRustFallback,
        zero_width: TokenSpecRegexZeroWidth,
    },
    Any {},
    Eof {},
    Empty {},
    Extern {
        /// scanner 実装が宣言する「成功し得る先頭 code point」の集合（昇順・互いに素な閉区間）。
        /// 省略は unknown（backend は候補除外に使わない）。宣言は必要条件であり、
        /// 集合外の先頭文字では scanner が消費なしで失敗することを scanner 側が保証する。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        first_chars: Option<Vec<CharRange>>,
        input_state: Vec<String>,
        #[serde(deserialize_with = "super::json::required_nullable")]
        java_binding: Option<String>,
        logical_id: String,
        original_class_name: String,
        result_fields: TokenSpecExternResultFields,
        #[serde(deserialize_with = "super::json::required_nullable")]
        rust_binding: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriviaInsertion {
    pub boundary: Boundary,
    pub policy_id: String,
}

/// insertionPlan は意味論。最適化 plan へ移動・省略してはならない。chain の先頭・各要素後と helper 境界の実行順。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Trivia {
    pub insertion_plan: Vec<TriviaInsertion>,
    pub policies: Vec<TriviaPolicy>,
}

/// javaStyle は ASCII 9..13,32 と既存共有 delimiter のコメントを含む。line キーの値ではなく存在が CPPComment（//）を有効にする。delimiter の順序を保持。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TriviaPolicy {
    pub effective_delimiter_set: Vec<Delimiter>,
    pub id: String,
    pub precedence: TriviaPolicyPrecedence,
    pub raw_settings: Settings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Typeof {
    pub comparison_kind: TypeofComparisonKind,
    pub mismatch: TypeofMismatch,
    pub own_capture: String,
    pub phase: TypeofPhase,
    pub referenced_capture: String,
    pub site_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ValueType {
    Text {},
    Number {
        java: ValueTypeNumberJava,
        rust: ValueTypeNumberRust,
    },
    Node {
        type_id: String,
    },
    Mixed {
        members: Vec<ValueType>,
    },
    Optional {
        element: Box<ValueType>,
    },
    List {
        element: Box<ValueType>,
    },
    Null {},
}

impl From<&Dfa> for RegexDfa {
    fn from(dfa: &Dfa) -> Self {
        Self {
            states: dfa.states.len() as u64,
            start: dfa.start_state,
            accepting: dfa
                .states
                .iter()
                .filter(|s| s.accepting)
                .map(|s| s.id)
                .collect(),
            transitions: dfa
                .states
                .iter()
                .flat_map(|s| {
                    s.transitions.iter().map(|t| RegexDfaTransition {
                        from: s.id,
                        ranges: vec![[t.range.start, t.range.end]],
                        to: t.target,
                    })
                })
                .collect(),
            anchored_at_start: RegexDfaAnchoredAtStart,
            semantics: RegexDfaSemantics,
            zero_width: TokenSpecRegexZeroWidth,
        }
    }
}
