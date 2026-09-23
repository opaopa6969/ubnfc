package org.ubnfc.p4.generated;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.regex.Pattern;
import org.ubnfc.p4.generated.api.*;
import org.ubnfc.p4.generated.internal.Recipe;
import org.ubnfc.p4.generated.internal.Session;
import org.ubnfc.p4.generated.internal.Session.*;

public final class TinyExpressionP4Parser {
    private TinyExpressionP4Parser() {}
    public static ParseResult<TinyExpressionP4AST> parse(CharSequence source) { return parse(source, ParseOptions.DEFAULT); }
    public static ParseResult<TinyExpressionP4AST> parse(CharSequence source, ParseOptions options) {
        return parseEntry("TinyExpressionP4", "Formula", source, options);
    }
    public static ParseResult<TinyExpressionP4AST> parseEntry(String grammar, String entry, CharSequence source, ParseOptions options) {
        return parseEntryInternal(grammar, entry, source, options, false);
    }
    public static ParseResult<TinyExpressionP4AST> parseEntryRollback(String grammar, String entry, CharSequence source, ParseOptions options) {
        return parseEntryInternal(grammar, entry, source, options, true);
    }
    private static ParseResult<TinyExpressionP4AST> parseEntryInternal(String grammar, String entry, CharSequence source, ParseOptions options, boolean rollback) {
        Call entryPoint = switch (grammar) {
            case "TinyExpressionP4" -> switch (entry) {
                case null -> TinyExpressionP4Parser::parseFormula_0;
                case "Formula" -> TinyExpressionP4Parser::parseFormula_0;
                case "CodeBlock" -> TinyExpressionP4Parser::parseCodeBlock_1;
                case "ImportDeclaration" -> TinyExpressionP4Parser::parseImportDeclaration_2;
                case "ClassName" -> TinyExpressionP4Parser::parseClassName_3;
                case "VariableDeclaration" -> TinyExpressionP4Parser::parseVariableDeclaration_4;
                case "NumberVariableDeclaration" -> TinyExpressionP4Parser::parseNumberVariableDeclaration_5;
                case "StringVariableDeclaration" -> TinyExpressionP4Parser::parseStringVariableDeclaration_6;
                case "BooleanVariableDeclaration" -> TinyExpressionP4Parser::parseBooleanVariableDeclaration_7;
                case "ObjectVariableDeclaration" -> TinyExpressionP4Parser::parseObjectVariableDeclaration_8;
                case "TypeHint" -> TinyExpressionP4Parser::parseTypeHint_9;
                case "NumberTypeHint" -> TinyExpressionP4Parser::parseNumberTypeHint_10;
                case "StringTypeHint" -> TinyExpressionP4Parser::parseStringTypeHint_11;
                case "BooleanTypeHint" -> TinyExpressionP4Parser::parseBooleanTypeHint_12;
                case "ObjectTypeHint" -> TinyExpressionP4Parser::parseObjectTypeHint_13;
                case "OnlyIfAbsent" -> TinyExpressionP4Parser::parseOnlyIfAbsent_14;
                case "Description" -> TinyExpressionP4Parser::parseDescription_15;
                case "Annotation" -> TinyExpressionP4Parser::parseAnnotation_16;
                case "AnnotationParameters" -> TinyExpressionP4Parser::parseAnnotationParameters_17;
                case "AnnotationParameter" -> TinyExpressionP4Parser::parseAnnotationParameter_18;
                case "MethodDeclaration" -> TinyExpressionP4Parser::parseMethodDeclaration_19;
                case "NumberMethodDeclaration" -> TinyExpressionP4Parser::parseNumberMethodDeclaration_20;
                case "StringMethodDeclaration" -> TinyExpressionP4Parser::parseStringMethodDeclaration_21;
                case "BooleanMethodDeclaration" -> TinyExpressionP4Parser::parseBooleanMethodDeclaration_22;
                case "ObjectMethodDeclaration" -> TinyExpressionP4Parser::parseObjectMethodDeclaration_23;
                case "MethodParameters" -> TinyExpressionP4Parser::parseMethodParameters_24;
                case "MethodParameter" -> TinyExpressionP4Parser::parseMethodParameter_25;
                case "NumberReturnType" -> TinyExpressionP4Parser::parseNumberReturnType_26;
                case "StringReturnType" -> TinyExpressionP4Parser::parseStringReturnType_27;
                case "BooleanReturnType" -> TinyExpressionP4Parser::parseBooleanReturnType_28;
                case "ObjectReturnType" -> TinyExpressionP4Parser::parseObjectReturnType_29;
                case "ReturnType" -> TinyExpressionP4Parser::parseReturnType_30;
                case "ExternalBooleanInvocation" -> TinyExpressionP4Parser::parseExternalBooleanInvocation_31;
                case "ExternalNumberInvocation" -> TinyExpressionP4Parser::parseExternalNumberInvocation_32;
                case "ExternalStringInvocation" -> TinyExpressionP4Parser::parseExternalStringInvocation_33;
                case "ExternalObjectInvocation" -> TinyExpressionP4Parser::parseExternalObjectInvocation_34;
                case "MethodInvocationHeader" -> TinyExpressionP4Parser::parseMethodInvocationHeader_35;
                case "MethodInvocation" -> TinyExpressionP4Parser::parseMethodInvocation_36;
                case "ArgumentTernary" -> TinyExpressionP4Parser::parseArgumentTernary_37;
                case "ArgumentExpression" -> TinyExpressionP4Parser::parseArgumentExpression_38;
                case "Arguments" -> TinyExpressionP4Parser::parseArguments_39;
                case "NumberExpression" -> TinyExpressionP4Parser::parseNumberExpression_40;
                case "NumberTerm" -> TinyExpressionP4Parser::parseNumberTerm_41;
                case "AddOp" -> TinyExpressionP4Parser::parseAddOp_42;
                case "MulOp" -> TinyExpressionP4Parser::parseMulOp_43;
                case "MathFunction" -> TinyExpressionP4Parser::parseMathFunction_44;
                case "SinFunction" -> TinyExpressionP4Parser::parseSinFunction_45;
                case "CosFunction" -> TinyExpressionP4Parser::parseCosFunction_46;
                case "TanFunction" -> TinyExpressionP4Parser::parseTanFunction_47;
                case "SqrtFunction" -> TinyExpressionP4Parser::parseSqrtFunction_48;
                case "MinFunction" -> TinyExpressionP4Parser::parseMinFunction_49;
                case "MaxFunction" -> TinyExpressionP4Parser::parseMaxFunction_50;
                case "RandomFunction" -> TinyExpressionP4Parser::parseRandomFunction_51;
                case "AbsFunction" -> TinyExpressionP4Parser::parseAbsFunction_52;
                case "RoundFunction" -> TinyExpressionP4Parser::parseRoundFunction_53;
                case "CeilFunction" -> TinyExpressionP4Parser::parseCeilFunction_54;
                case "FloorFunction" -> TinyExpressionP4Parser::parseFloorFunction_55;
                case "PowFunction" -> TinyExpressionP4Parser::parsePowFunction_56;
                case "LogFunction" -> TinyExpressionP4Parser::parseLogFunction_57;
                case "ExpFunction" -> TinyExpressionP4Parser::parseExpFunction_58;
                case "ToNumFunction" -> TinyExpressionP4Parser::parseToNumFunction_59;
                case "NumberFactor" -> TinyExpressionP4Parser::parseNumberFactor_60;
                case "ToUpperCaseFunction" -> TinyExpressionP4Parser::parseToUpperCaseFunction_61;
                case "ToLowerCaseFunction" -> TinyExpressionP4Parser::parseToLowerCaseFunction_62;
                case "TrimFunction" -> TinyExpressionP4Parser::parseTrimFunction_63;
                case "LengthFunction" -> TinyExpressionP4Parser::parseLengthFunction_64;
                case "LenFunction" -> TinyExpressionP4Parser::parseLenFunction_65;
                case "ToUpperCaseDotMethod" -> TinyExpressionP4Parser::parseToUpperCaseDotMethod_66;
                case "ToLowerCaseDotMethod" -> TinyExpressionP4Parser::parseToLowerCaseDotMethod_67;
                case "TrimDotMethod" -> TinyExpressionP4Parser::parseTrimDotMethod_68;
                case "LengthDotMethod" -> TinyExpressionP4Parser::parseLengthDotMethod_69;
                case "StartsWithFunction" -> TinyExpressionP4Parser::parseStartsWithFunction_70;
                case "EndsWithFunction" -> TinyExpressionP4Parser::parseEndsWithFunction_71;
                case "ContainsFunction" -> TinyExpressionP4Parser::parseContainsFunction_72;
                case "InMethod" -> TinyExpressionP4Parser::parseInMethod_73;
                case "StartsWithDotMethod" -> TinyExpressionP4Parser::parseStartsWithDotMethod_74;
                case "EndsWithDotMethod" -> TinyExpressionP4Parser::parseEndsWithDotMethod_75;
                case "ContainsDotMethod" -> TinyExpressionP4Parser::parseContainsDotMethod_76;
                case "StringPredicateReceiver" -> TinyExpressionP4Parser::parseStringPredicateReceiver_77;
                case "IsPresentFunction" -> TinyExpressionP4Parser::parseIsPresentFunction_78;
                case "InTimeRangeFunction" -> TinyExpressionP4Parser::parseInTimeRangeFunction_79;
                case "InDayTimeRangeFunction" -> TinyExpressionP4Parser::parseInDayTimeRangeFunction_80;
                case "DayOfWeek" -> TinyExpressionP4Parser::parseDayOfWeek_81;
                case "SliceBaseReceiver" -> TinyExpressionP4Parser::parseSliceBaseReceiver_82;
                case "SliceStartIndex" -> TinyExpressionP4Parser::parseSliceStartIndex_83;
                case "SliceEndIndex" -> TinyExpressionP4Parser::parseSliceEndIndex_84;
                case "SliceStepIndex" -> TinyExpressionP4Parser::parseSliceStepIndex_85;
                case "SliceBaseExpression" -> TinyExpressionP4Parser::parseSliceBaseExpression_86;
                case "SliceNestedExpression" -> TinyExpressionP4Parser::parseSliceNestedExpression_87;
                case "SliceExpression" -> TinyExpressionP4Parser::parseSliceExpression_88;
                case "StringExpression" -> TinyExpressionP4Parser::parseStringExpression_89;
                case "ParenthesizedStringExpression" -> TinyExpressionP4Parser::parseParenthesizedStringExpression_90;
                case "StringTerm" -> TinyExpressionP4Parser::parseStringTerm_91;
                case "StringCastVariable" -> TinyExpressionP4Parser::parseStringCastVariable_92;
                case "StringTypedVariable" -> TinyExpressionP4Parser::parseStringTypedVariable_93;
                case "BooleanExpression" -> TinyExpressionP4Parser::parseBooleanExpression_94;
                case "BooleanAndExpression" -> TinyExpressionP4Parser::parseBooleanAndExpression_95;
                case "BooleanXorExpression" -> TinyExpressionP4Parser::parseBooleanXorExpression_96;
                case "NotExpression" -> TinyExpressionP4Parser::parseNotExpression_97;
                case "BooleanComparable" -> TinyExpressionP4Parser::parseBooleanComparable_98;
                case "BooleanEqualityExpression" -> TinyExpressionP4Parser::parseBooleanEqualityExpression_99;
                case "BooleanFactor" -> TinyExpressionP4Parser::parseBooleanFactor_100;
                case "StringComparisonExpression" -> TinyExpressionP4Parser::parseStringComparisonExpression_101;
                case "EqualityOp" -> TinyExpressionP4Parser::parseEqualityOp_102;
                case "ComparisonExpression" -> TinyExpressionP4Parser::parseComparisonExpression_103;
                case "CompareOp" -> TinyExpressionP4Parser::parseCompareOp_104;
                case "ObjectExpression" -> TinyExpressionP4Parser::parseObjectExpression_105;
                case "IfExpression" -> TinyExpressionP4Parser::parseIfExpression_106;
                case "BranchExpression" -> TinyExpressionP4Parser::parseBranchExpression_107;
                case "TernaryExpression" -> TinyExpressionP4Parser::parseTernaryExpression_108;
                case "NumberMatchExpression" -> TinyExpressionP4Parser::parseNumberMatchExpression_109;
                case "NumberCase" -> TinyExpressionP4Parser::parseNumberCase_110;
                case "NumberDefaultCase" -> TinyExpressionP4Parser::parseNumberDefaultCase_111;
                case "NumberCaseValue" -> TinyExpressionP4Parser::parseNumberCaseValue_112;
                case "StringMatchExpression" -> TinyExpressionP4Parser::parseStringMatchExpression_113;
                case "StringCase" -> TinyExpressionP4Parser::parseStringCase_114;
                case "StringDefaultCase" -> TinyExpressionP4Parser::parseStringDefaultCase_115;
                case "StringCaseValue" -> TinyExpressionP4Parser::parseStringCaseValue_116;
                case "BooleanMatchExpression" -> TinyExpressionP4Parser::parseBooleanMatchExpression_117;
                case "BooleanCase" -> TinyExpressionP4Parser::parseBooleanCase_118;
                case "BooleanDefaultCase" -> TinyExpressionP4Parser::parseBooleanDefaultCase_119;
                case "BooleanCaseValue" -> TinyExpressionP4Parser::parseBooleanCaseValue_120;
                case "VariableRef" -> TinyExpressionP4Parser::parseVariableRef_121;
                case "TypeKeyword" -> TinyExpressionP4Parser::parseTypeKeyword_122;
                case "Expression" -> TinyExpressionP4Parser::parseExpression_123;
                default -> throw new IllegalArgumentException("Unsupported entry: " + entry);
            };
            default -> throw new IllegalArgumentException("Unsupported grammar: " + grammar);
        };
        var s = new Session(source, options, K0);
        var f = new Frame(); Match match = s.parse(entryPoint, f);
        if (rollback) { s.rollback(f); return s.result(f, null, Map.of()); }
        ParseResult<TinyExpressionP4AST> syntax = s.result(f, match, K1);
        if (!syntax.ok() || !options.buildAst()) return syntax;
        if (match.mappingFailure()) throw s.mapping(match.start(), match.end(), "Recovered input cannot be mapped", null);
        if (match.nodes().isEmpty()) return syntax;
        return syntax.withAst((TinyExpressionP4AST) build(match.nodes().getFirst(), s), s.spans);
    }
    private static Object build(Recipe r, Session s) {
        if (s.hasBuilt(r)) return s.built(r);
        if (r.id().equals("#recovery")) return s.remember(r, null);
        try {
        Object node = switch (r.id()) {
            case "recipe:Formula" -> {
                java.util.List<TinyExpressionP4AST.ImportDeclarationExpr> field0 = s.list(r,"imports",v -> (TinyExpressionP4AST.ImportDeclarationExpr) build(s.node(v, r), s));
                java.util.List<java.lang.Object> field1 = s.list(r,"declarations",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                TinyExpressionP4AST.ExpressionExpr field2 = s.scalar(r,"expression",v -> (TinyExpressionP4AST.ExpressionExpr) build(s.node(v, r), s));
                java.util.List<java.lang.Object> field3 = s.list(r,"methods",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.FormulaExpr(field0, field1, field2, field3);
            }
            case "recipe:CodeBlock" -> {
                yield new TinyExpressionP4AST.CodeBlockExpr();
            }
            case "recipe:ImportDeclaration" -> {
                TinyExpressionP4AST.QualifiedNameExpr field0 = s.scalar(r,"className",v -> (TinyExpressionP4AST.QualifiedNameExpr) build(s.node(v, r), s));
                java.util.Optional<java.lang.String> field1 = s.optional(r,"method",v -> s.text(v));
                java.lang.String field2 = s.scalar(r,"alias",v -> s.text(v));
                yield new TinyExpressionP4AST.ImportDeclarationExpr(field0, field1, field2);
            }
            case "recipe:ClassName" -> {
                java.lang.String field0 = s.scalar(r,"head",v -> s.text(v));
                java.util.List<java.lang.String> field1 = s.list(r,"tail",v -> s.text(v));
                yield new TinyExpressionP4AST.QualifiedNameExpr(field0, field1);
            }
            case "recipe:NumberVariableDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"varName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.OnlyIfAbsentExpr> field1 = s.optional(r,"onlyIfAbsent",v -> (TinyExpressionP4AST.OnlyIfAbsentExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field2 = s.optional(r,"value",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.util.Optional<java.lang.String> field3 = s.optional(r,"desc",v -> s.text(v));
                yield new TinyExpressionP4AST.NumberVariableDeclarationExpr(field0, field1, field2, field3);
            }
            case "recipe:StringVariableDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"varName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.OnlyIfAbsentExpr> field1 = s.optional(r,"onlyIfAbsent",v -> (TinyExpressionP4AST.OnlyIfAbsentExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.StringConcatExpr> field2 = s.optional(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                java.util.Optional<java.lang.String> field3 = s.optional(r,"desc",v -> s.text(v));
                yield new TinyExpressionP4AST.StringVariableDeclarationExpr(field0, field1, field2, field3);
            }
            case "recipe:BooleanVariableDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"varName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.OnlyIfAbsentExpr> field1 = s.optional(r,"onlyIfAbsent",v -> (TinyExpressionP4AST.OnlyIfAbsentExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.BooleanOrExpr> field2 = s.optional(r,"value",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                java.util.Optional<java.lang.String> field3 = s.optional(r,"desc",v -> s.text(v));
                yield new TinyExpressionP4AST.BooleanVariableDeclarationExpr(field0, field1, field2, field3);
            }
            case "recipe:ObjectVariableDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"varName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.OnlyIfAbsentExpr> field1 = s.optional(r,"onlyIfAbsent",v -> (TinyExpressionP4AST.OnlyIfAbsentExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.ObjectExpr> field2 = s.optional(r,"value",v -> (TinyExpressionP4AST.ObjectExpr) build(s.node(v, r), s));
                java.util.Optional<java.lang.String> field3 = s.optional(r,"desc",v -> s.text(v));
                yield new TinyExpressionP4AST.ObjectVariableDeclarationExpr(field0, field1, field2, field3);
            }
            case "recipe:OnlyIfAbsent" -> {
                yield new TinyExpressionP4AST.OnlyIfAbsentExpr();
            }
            case "recipe:NumberMethodDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"methodName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.MethodParametersExpr> field1 = s.optional(r,"parameters",v -> (TinyExpressionP4AST.MethodParametersExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BinaryExpr field2 = s.scalar(r,"expression",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.NumberMethodDeclarationExpr(field0, field1, field2);
            }
            case "recipe:StringMethodDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"methodName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.MethodParametersExpr> field1 = s.optional(r,"parameters",v -> (TinyExpressionP4AST.MethodParametersExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.StringConcatExpr field2 = s.scalar(r,"expression",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StringMethodDeclarationExpr(field0, field1, field2);
            }
            case "recipe:BooleanMethodDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"methodName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.MethodParametersExpr> field1 = s.optional(r,"parameters",v -> (TinyExpressionP4AST.MethodParametersExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BooleanOrExpr field2 = s.scalar(r,"expression",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanMethodDeclarationExpr(field0, field1, field2);
            }
            case "recipe:ObjectMethodDeclaration" -> {
                java.lang.String field0 = s.scalar(r,"methodName",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.MethodParametersExpr> field1 = s.optional(r,"parameters",v -> (TinyExpressionP4AST.MethodParametersExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.ObjectExpr field2 = s.scalar(r,"expression",v -> (TinyExpressionP4AST.ObjectExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ObjectMethodDeclarationExpr(field0, field1, field2);
            }
            case "recipe:MethodParameters" -> {
                java.util.List<TinyExpressionP4AST.MethodParameterExpr> field0 = s.list(r,"values",v -> (TinyExpressionP4AST.MethodParameterExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.MethodParametersExpr(field0);
            }
            case "recipe:MethodParameter" -> {
                java.lang.String field0 = s.scalar(r,"paramName",v -> s.text(v));
                java.util.Optional<java.lang.String> field1 = s.optional(r,"type",v -> s.text(v));
                yield new TinyExpressionP4AST.MethodParameterExpr(field0, field1);
            }
            case "recipe:ExternalBooleanInvocation" -> {
                java.util.Optional<TinyExpressionP4AST.QualifiedNameExpr> field0 = s.optional(r,"className",v -> (TinyExpressionP4AST.QualifiedNameExpr) build(s.node(v, r), s));
                java.lang.String field1 = s.scalar(r,"name",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.ArgumentsExpr> field2 = s.optional(r,"args",v -> (TinyExpressionP4AST.ArgumentsExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ExternalBooleanInvocationExpr(field0, field1, field2);
            }
            case "recipe:ExternalNumberInvocation" -> {
                java.util.Optional<TinyExpressionP4AST.QualifiedNameExpr> field0 = s.optional(r,"className",v -> (TinyExpressionP4AST.QualifiedNameExpr) build(s.node(v, r), s));
                java.lang.String field1 = s.scalar(r,"name",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.ArgumentsExpr> field2 = s.optional(r,"args",v -> (TinyExpressionP4AST.ArgumentsExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ExternalNumberInvocationExpr(field0, field1, field2);
            }
            case "recipe:ExternalStringInvocation" -> {
                java.util.Optional<TinyExpressionP4AST.QualifiedNameExpr> field0 = s.optional(r,"className",v -> (TinyExpressionP4AST.QualifiedNameExpr) build(s.node(v, r), s));
                java.lang.String field1 = s.scalar(r,"name",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.ArgumentsExpr> field2 = s.optional(r,"args",v -> (TinyExpressionP4AST.ArgumentsExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ExternalStringInvocationExpr(field0, field1, field2);
            }
            case "recipe:ExternalObjectInvocation" -> {
                java.util.Optional<TinyExpressionP4AST.QualifiedNameExpr> field0 = s.optional(r,"className",v -> (TinyExpressionP4AST.QualifiedNameExpr) build(s.node(v, r), s));
                java.lang.String field1 = s.scalar(r,"name",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.ArgumentsExpr> field2 = s.optional(r,"args",v -> (TinyExpressionP4AST.ArgumentsExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ExternalObjectInvocationExpr(field0, field1, field2);
            }
            case "recipe:MethodInvocation" -> {
                java.lang.String field0 = s.scalar(r,"name",v -> s.text(v));
                java.util.Optional<TinyExpressionP4AST.ArgumentsExpr> field1 = s.optional(r,"args",v -> (TinyExpressionP4AST.ArgumentsExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.MethodInvocationExpr(field0, field1);
            }
            case "recipe:ArgumentTernary" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"condition",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BranchExpressionExpr field1 = s.scalar(r,"thenExpr",v -> (TinyExpressionP4AST.BranchExpressionExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BranchExpressionExpr field2 = s.scalar(r,"elseExpr",v -> (TinyExpressionP4AST.BranchExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.TernaryExpr(field0, field1, field2);
            }
            case "recipe:ArgumentExpression" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.ArgumentExpressionExpr(field0);
            }
            case "recipe:Arguments" -> {
                java.util.List<TinyExpressionP4AST.ArgumentExpressionExpr> field0 = s.list(r,"values",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ArgumentsExpr(field0);
            }
            case "recipe:NumberExpression" -> {
                TinyExpressionP4AST field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST) build(s.node(v, r), s));
                java.util.List<java.lang.String> field1 = s.list(r,"op",v -> s.text(v));
                java.util.List<TinyExpressionP4AST> field2 = s.list(r,"right",v -> (TinyExpressionP4AST) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BinaryExpr(field0, field1, field2);
            }
            case "recipe:NumberTerm" -> {
                TinyExpressionP4AST field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST) build(s.node(v, r), s));
                java.util.List<java.lang.String> field1 = s.list(r,"op",v -> s.text(v));
                java.util.List<TinyExpressionP4AST> field2 = s.list(r,"right",v -> (TinyExpressionP4AST) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BinaryExpr(field0, field1, field2);
            }
            case "recipe:SinFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.SinExpr(field0);
            }
            case "recipe:CosFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.CosExpr(field0);
            }
            case "recipe:TanFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.TanExpr(field0);
            }
            case "recipe:SqrtFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.SqrtExpr(field0);
            }
            case "recipe:MinFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"first",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.ArgumentExpressionExpr> field1 = s.list(r,"rest",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.MinExpr(field0, field1);
            }
            case "recipe:MaxFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"first",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.ArgumentExpressionExpr> field1 = s.list(r,"rest",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.MaxExpr(field0, field1);
            }
            case "recipe:RandomFunction" -> {
                yield new TinyExpressionP4AST.RandomExpr();
            }
            case "recipe:AbsFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.AbsExpr(field0);
            }
            case "recipe:RoundFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.RoundExpr(field0);
            }
            case "recipe:CeilFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.CeilExpr(field0);
            }
            case "recipe:FloorFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.FloorExpr(field0);
            }
            case "recipe:PowFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"base",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.ArgumentExpressionExpr field1 = s.scalar(r,"exponent",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.PowExpr(field0, field1);
            }
            case "recipe:LogFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.LogExpr(field0);
            }
            case "recipe:ExpFunction" -> {
                TinyExpressionP4AST.ArgumentExpressionExpr field0 = s.scalar(r,"arg",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ExpExpr(field0);
            }
            case "recipe:ToNumFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.ArgumentExpressionExpr field1 = s.scalar(r,"defaultValue",v -> (TinyExpressionP4AST.ArgumentExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ToNumExpr(field0, field1);
            }
            case "recipe:ToUpperCaseFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ToUpperCaseExpr(field0);
            }
            case "recipe:ToLowerCaseFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ToLowerCaseExpr(field0);
            }
            case "recipe:TrimFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.TrimExpr(field0);
            }
            case "recipe:LengthFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.LengthExpr(field0);
            }
            case "recipe:LenFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.LengthExpr(field0);
            }
            case "recipe:ToUpperCaseDotMethod" -> {
                TinyExpressionP4AST.VariableRefExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.VariableRefExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ToUpperCaseDotExpr(field0);
            }
            case "recipe:ToLowerCaseDotMethod" -> {
                TinyExpressionP4AST.VariableRefExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.VariableRefExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ToLowerCaseDotExpr(field0);
            }
            case "recipe:TrimDotMethod" -> {
                TinyExpressionP4AST.VariableRefExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.VariableRefExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.TrimDotExpr(field0);
            }
            case "recipe:LengthDotMethod" -> {
                TinyExpressionP4AST.VariableRefExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.VariableRefExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.LengthDotExpr(field0);
            }
            case "recipe:StartsWithFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"patterns",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StartsWithExpr(field0, field1);
            }
            case "recipe:EndsWithFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"patterns",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.EndsWithExpr(field0, field1);
            }
            case "recipe:ContainsFunction" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"patterns",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ContainsExpr(field0, field1);
            }
            case "recipe:InMethod" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"candidates",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.InExpr(field0, field1);
            }
            case "recipe:StartsWithDotMethod" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"patterns",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StartsWithDotExpr(field0, field1);
            }
            case "recipe:EndsWithDotMethod" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"patterns",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.EndsWithDotExpr(field0, field1);
            }
            case "recipe:ContainsDotMethod" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                java.util.List<TinyExpressionP4AST.StringConcatExpr> field1 = s.list(r,"patterns",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ContainsDotExpr(field0, field1);
            }
            case "recipe:IsPresentFunction" -> {
                TinyExpressionP4AST.VariableRefExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.VariableRefExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.IsPresentExpr(field0);
            }
            case "recipe:InTimeRangeFunction" -> {
                TinyExpressionP4AST.BinaryExpr field0 = s.scalar(r,"startHour",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BinaryExpr field1 = s.scalar(r,"endHour",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.InTimeRangeExpr(field0, field1);
            }
            case "recipe:InDayTimeRangeFunction" -> {
                java.lang.String field0 = s.scalar(r,"startDay",v -> s.text(v));
                TinyExpressionP4AST.BinaryExpr field1 = s.scalar(r,"startHour",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.lang.String field2 = s.scalar(r,"endDay",v -> s.text(v));
                TinyExpressionP4AST.BinaryExpr field3 = s.scalar(r,"endHour",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.InDayTimeRangeExpr(field0, field1, field2, field3);
            }
            case "recipe:SliceBaseExpression" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field1 = s.optional(r,"start",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field2 = s.optional(r,"end",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field3 = s.optional(r,"step",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.SliceExpr(field0, field1, field2, field3);
            }
            case "recipe:SliceNestedExpression" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field1 = s.optional(r,"start",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field2 = s.optional(r,"end",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.util.Optional<TinyExpressionP4AST.BinaryExpr> field3 = s.optional(r,"step",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.SliceExpr(field0, field1, field2, field3);
            }
            case "recipe:StringExpression" -> {
                java.lang.Object field0 = s.scalar(r,"left",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                java.util.List<java.lang.String> field1 = s.list(r,"op",v -> s.text(v));
                java.util.List<java.lang.Object> field2 = s.list(r,"right",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.StringConcatExpr(field0, field1, field2);
            }
            case "recipe:StringCastVariable" -> {
                java.lang.String field0 = s.scalar(r,"name",v -> s.text(v));
                yield new TinyExpressionP4AST.StringCastVariableRefExpr(field0);
            }
            case "recipe:StringTypedVariable" -> {
                java.lang.String field0 = s.scalar(r,"name",v -> s.text(v));
                yield new TinyExpressionP4AST.StringTypedVariableRefExpr(field0);
            }
            case "recipe:BooleanExpression" -> {
                TinyExpressionP4AST.BooleanAndExpr field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST.BooleanAndExpr) build(s.node(v, r), s));
                java.util.List<java.lang.String> field1 = s.list(r,"op",v -> s.text(v));
                java.util.List<TinyExpressionP4AST.BooleanAndExpr> field2 = s.list(r,"right",v -> (TinyExpressionP4AST.BooleanAndExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanOrExpr(field0, field1, field2);
            }
            case "recipe:BooleanAndExpression" -> {
                TinyExpressionP4AST.BooleanXorExpr field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST.BooleanXorExpr) build(s.node(v, r), s));
                java.util.List<java.lang.String> field1 = s.list(r,"op",v -> s.text(v));
                java.util.List<TinyExpressionP4AST.BooleanXorExpr> field2 = s.list(r,"right",v -> (TinyExpressionP4AST.BooleanXorExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanAndExpr(field0, field1, field2);
            }
            case "recipe:BooleanXorExpression" -> {
                TinyExpressionP4AST.BooleanFactorExpr field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST.BooleanFactorExpr) build(s.node(v, r), s));
                java.util.List<java.lang.String> field1 = s.list(r,"op",v -> s.text(v));
                java.util.List<TinyExpressionP4AST.BooleanFactorExpr> field2 = s.list(r,"right",v -> (TinyExpressionP4AST.BooleanFactorExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanXorExpr(field0, field1, field2);
            }
            case "recipe:NotExpression" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.NotExpr(field0);
            }
            case "recipe:BooleanEqualityExpression" -> {
                java.lang.Object field0 = s.scalar(r,"left",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                java.lang.String field1 = s.scalar(r,"op",v -> s.text(v));
                java.lang.Object field2 = s.scalar(r,"right",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.BooleanEqualityExpr(field0, field1, field2);
            }
            case "recipe:BooleanFactor" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? s.text(v) : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.BooleanFactorExpr(field0);
            }
            case "recipe:StringComparisonExpression" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                java.lang.String field1 = s.scalar(r,"op",v -> s.text(v));
                TinyExpressionP4AST.StringConcatExpr field2 = s.scalar(r,"right",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StringComparisonExpr(field0, field1, field2);
            }
            case "recipe:ComparisonExpression" -> {
                TinyExpressionP4AST.BinaryExpr field0 = s.scalar(r,"left",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                java.lang.String field1 = s.scalar(r,"op",v -> s.text(v));
                TinyExpressionP4AST.BinaryExpr field2 = s.scalar(r,"right",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.ComparisonExpr(field0, field1, field2);
            }
            case "recipe:ObjectExpression" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.ObjectExpr(field0);
            }
            case "recipe:IfExpression" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"condition",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BranchExpressionExpr field1 = s.scalar(r,"thenExpr",v -> (TinyExpressionP4AST.BranchExpressionExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BranchExpressionExpr field2 = s.scalar(r,"elseExpr",v -> (TinyExpressionP4AST.BranchExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.IfExpr(field0, field1, field2);
            }
            case "recipe:BranchExpression" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.BranchExpressionExpr(field0);
            }
            case "recipe:TernaryExpression" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"condition",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BranchExpressionExpr field1 = s.scalar(r,"thenExpr",v -> (TinyExpressionP4AST.BranchExpressionExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BranchExpressionExpr field2 = s.scalar(r,"elseExpr",v -> (TinyExpressionP4AST.BranchExpressionExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.TernaryExpr(field0, field1, field2);
            }
            case "recipe:NumberMatchExpression" -> {
                TinyExpressionP4AST.NumberCaseExpr field0 = s.scalar(r,"firstCase",v -> (TinyExpressionP4AST.NumberCaseExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.NumberCaseExpr> field1 = s.list(r,"moreCases",v -> (TinyExpressionP4AST.NumberCaseExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.NumberDefaultCaseExpr field2 = s.scalar(r,"defaultCase",v -> (TinyExpressionP4AST.NumberDefaultCaseExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.NumberMatchExpr(field0, field1, field2);
            }
            case "recipe:NumberCase" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"condition",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.NumberCaseValueExpr field1 = s.scalar(r,"value",v -> (TinyExpressionP4AST.NumberCaseValueExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.NumberCaseExpr(field0, field1);
            }
            case "recipe:NumberDefaultCase" -> {
                TinyExpressionP4AST.NumberCaseValueExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.NumberCaseValueExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.NumberDefaultCaseExpr(field0);
            }
            case "recipe:NumberCaseValue" -> {
                TinyExpressionP4AST.BinaryExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.BinaryExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.NumberCaseValueExpr(field0);
            }
            case "recipe:StringMatchExpression" -> {
                TinyExpressionP4AST.StringCaseExpr field0 = s.scalar(r,"firstCase",v -> (TinyExpressionP4AST.StringCaseExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.StringCaseExpr> field1 = s.list(r,"moreCases",v -> (TinyExpressionP4AST.StringCaseExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.StringDefaultCaseExpr field2 = s.scalar(r,"defaultCase",v -> (TinyExpressionP4AST.StringDefaultCaseExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StringMatchExpr(field0, field1, field2);
            }
            case "recipe:StringCase" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"condition",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.StringCaseValueExpr field1 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringCaseValueExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StringCaseExpr(field0, field1);
            }
            case "recipe:StringDefaultCase" -> {
                TinyExpressionP4AST.StringCaseValueExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringCaseValueExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StringDefaultCaseExpr(field0);
            }
            case "recipe:StringCaseValue" -> {
                TinyExpressionP4AST.StringConcatExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.StringConcatExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.StringCaseValueExpr(field0);
            }
            case "recipe:BooleanMatchExpression" -> {
                TinyExpressionP4AST.BooleanCaseExpr field0 = s.scalar(r,"firstCase",v -> (TinyExpressionP4AST.BooleanCaseExpr) build(s.node(v, r), s));
                java.util.List<TinyExpressionP4AST.BooleanCaseExpr> field1 = s.list(r,"moreCases",v -> (TinyExpressionP4AST.BooleanCaseExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BooleanDefaultCaseExpr field2 = s.scalar(r,"defaultCase",v -> (TinyExpressionP4AST.BooleanDefaultCaseExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanMatchExpr(field0, field1, field2);
            }
            case "recipe:BooleanCase" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"condition",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                TinyExpressionP4AST.BooleanCaseValueExpr field1 = s.scalar(r,"value",v -> (TinyExpressionP4AST.BooleanCaseValueExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanCaseExpr(field0, field1);
            }
            case "recipe:BooleanDefaultCase" -> {
                TinyExpressionP4AST.BooleanCaseValueExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.BooleanCaseValueExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanDefaultCaseExpr(field0);
            }
            case "recipe:BooleanCaseValue" -> {
                TinyExpressionP4AST.BooleanOrExpr field0 = s.scalar(r,"value",v -> (TinyExpressionP4AST.BooleanOrExpr) build(s.node(v, r), s));
                yield new TinyExpressionP4AST.BooleanCaseValueExpr(field0);
            }
            case "recipe:VariableRef" -> {
                java.lang.String field0 = s.scalar(r,"name",v -> s.text(v));
                java.util.Optional<java.lang.String> field1 = s.optional(r,"type",v -> s.text(v));
                yield new TinyExpressionP4AST.VariableRefExpr(field0, field1);
            }
            case "recipe:Expression" -> {
                java.lang.Object field0 = s.scalar(r,"value",v -> (v.nodes().isEmpty() ? null : build(s.node(v, r), s)));
                yield new TinyExpressionP4AST.ExpressionExpr(field0);
            }
            case "#leaf:TinyExpressionP4AST.BinaryExpr" -> new TinyExpressionP4AST.BinaryExpr(null, List.of(s.source.substring(r.start(), r.end()).strip()), List.of());
            default -> throw s.mapping(r.start(), r.end(), "Cannot build recipe: " + r.id(), null);
        };
        return s.remember(r, node);
        } catch (Session.RecoveredNodeMissing missing) { return s.remember(r, null); }
    }
    private static Match parseFormula_0(Session s, Frame f) {
        return s.rule(f, 0, 1043, true, true, TinyExpressionP4Parser::e0, "recipe:Formula", null, "construct", K2, K3, false, K4, false, K5);
    }
    private static Match parseCodeBlock_1(Session s, Frame f) {
        return s.rule(f, 1, 1044, true, true, TinyExpressionP4Parser::e13, "recipe:CodeBlock", null, "construct", K6, K7, false, K4, false, K8);
    }
    private static Match parseImportDeclaration_2(Session s, Frame f) {
        return s.rule(f, 2, 1045, true, true, TinyExpressionP4Parser::e17, "recipe:ImportDeclaration", null, "construct", K9, K7, false, K4, false, K10);
    }
    private static Match parseClassName_3(Session s, Frame f) {
        return s.rule(f, 3, 1046, true, true, TinyExpressionP4Parser::e27, "recipe:ClassName", null, "construct", K11, K7, false, K4, false, K12);
    }
    private static Match parseVariableDeclaration_4(Session s, Frame f) {
        return s.rule(f, 4, 1047, true, true, TinyExpressionP4Parser::e33, null, null, null, K6, K7, false, K4, false, K13);
    }
    private static Match parseNumberVariableDeclaration_5(Session s, Frame f) {
        return s.rule(f, 5, 1048, true, true, TinyExpressionP4Parser::e38, "recipe:NumberVariableDeclaration", null, "construct", K14, K15, false, K4, false, K16);
    }
    private static Match parseStringVariableDeclaration_6(Session s, Frame f) {
        return s.rule(f, 6, 1049, true, true, TinyExpressionP4Parser::e56, "recipe:StringVariableDeclaration", null, "construct", K17, K18, false, K4, false, K16);
    }
    private static Match parseBooleanVariableDeclaration_7(Session s, Frame f) {
        return s.rule(f, 7, 1050, true, true, TinyExpressionP4Parser::e74, "recipe:BooleanVariableDeclaration", null, "construct", K19, K20, false, K4, false, K16);
    }
    private static Match parseObjectVariableDeclaration_8(Session s, Frame f) {
        return s.rule(f, 8, 1051, true, true, TinyExpressionP4Parser::e92, "recipe:ObjectVariableDeclaration", null, "construct", K21, K22, false, K4, false, K16);
    }
    private static Match parseTypeHint_9(Session s, Frame f) {
        return s.rule(f, 9, 1052, true, true, TinyExpressionP4Parser::e110, null, null, null, K6, K7, false, K4, false, K23);
    }
    private static Match parseNumberTypeHint_10(Session s, Frame f) {
        return s.rule(f, 10, 1053, true, true, TinyExpressionP4Parser::e123, null, null, null, K6, K7, false, K4, false, K24);
    }
    private static Match parseStringTypeHint_11(Session s, Frame f) {
        return s.rule(f, 11, 1054, true, true, TinyExpressionP4Parser::e132, null, null, null, K6, K7, false, K4, false, K25);
    }
    private static Match parseBooleanTypeHint_12(Session s, Frame f) {
        return s.rule(f, 12, 1055, true, true, TinyExpressionP4Parser::e139, null, null, null, K6, K7, false, K4, false, K26);
    }
    private static Match parseObjectTypeHint_13(Session s, Frame f) {
        return s.rule(f, 13, 1056, true, true, TinyExpressionP4Parser::e146, null, null, null, K6, K7, false, K4, false, K27);
    }
    private static Match parseOnlyIfAbsent_14(Session s, Frame f) {
        return s.rule(f, 14, 1057, true, true, TinyExpressionP4Parser::e153, "recipe:OnlyIfAbsent", null, "construct", K6, K7, false, K4, false, K28);
    }
    private static Match parseDescription_15(Session s, Frame f) {
        return s.rule(f, 15, 1058, true, true, TinyExpressionP4Parser::e157, null, null, null, K6, K7, false, K4, false, K29);
    }
    private static Match parseAnnotation_16(Session s, Frame f) {
        return s.rule(f, 16, 1059, true, true, TinyExpressionP4Parser::e161, null, null, null, K6, K7, false, K4, false, K30);
    }
    private static Match parseAnnotationParameters_17(Session s, Frame f) {
        return s.rule(f, 17, 1060, true, true, TinyExpressionP4Parser::e168, null, null, null, K6, K7, false, K4, false, K31);
    }
    private static Match parseAnnotationParameter_18(Session s, Frame f) {
        return s.rule(f, 18, 1061, true, true, TinyExpressionP4Parser::e174, null, null, null, K6, K7, false, K4, false, K31);
    }
    private static Match parseMethodDeclaration_19(Session s, Frame f) {
        return s.rule(f, 19, 1062, true, true, TinyExpressionP4Parser::e178, null, null, null, K6, K7, false, K4, false, K32);
    }
    private static Match parseNumberMethodDeclaration_20(Session s, Frame f) {
        return s.rule(f, 20, 1063, true, true, TinyExpressionP4Parser::e183, "recipe:NumberMethodDeclaration", null, "construct", K33, K34, false, K4, false, K35);
    }
    private static Match parseStringMethodDeclaration_21(Session s, Frame f) {
        return s.rule(f, 21, 1064, true, true, TinyExpressionP4Parser::e193, "recipe:StringMethodDeclaration", null, "construct", K36, K37, false, K4, false, K38);
    }
    private static Match parseBooleanMethodDeclaration_22(Session s, Frame f) {
        return s.rule(f, 22, 1065, true, true, TinyExpressionP4Parser::e203, "recipe:BooleanMethodDeclaration", null, "construct", K39, K40, false, K4, false, K41);
    }
    private static Match parseObjectMethodDeclaration_23(Session s, Frame f) {
        return s.rule(f, 23, 1066, true, true, TinyExpressionP4Parser::e213, "recipe:ObjectMethodDeclaration", null, "construct", K42, K43, false, K4, false, K44);
    }
    private static Match parseMethodParameters_24(Session s, Frame f) {
        return s.rule(f, 24, 1067, true, true, TinyExpressionP4Parser::e223, "recipe:MethodParameters", null, "construct", K45, K7, false, K4, false, K46);
    }
    private static Match parseMethodParameter_25(Session s, Frame f) {
        return s.rule(f, 25, 1068, true, true, TinyExpressionP4Parser::e229, "recipe:MethodParameter", null, "construct", K47, K48, false, K4, false, K49);
    }
    private static Match parseNumberReturnType_26(Session s, Frame f) {
        return s.rule(f, 26, 1069, true, true, TinyExpressionP4Parser::e236, null, null, null, K6, K7, false, K4, false, K50);
    }
    private static Match parseStringReturnType_27(Session s, Frame f) {
        return s.rule(f, 27, 1070, true, true, TinyExpressionP4Parser::e239, null, null, null, K6, K7, false, K4, false, K51);
    }
    private static Match parseBooleanReturnType_28(Session s, Frame f) {
        return s.rule(f, 28, 1071, true, true, TinyExpressionP4Parser::e241, null, null, null, K6, K7, false, K4, false, K52);
    }
    private static Match parseObjectReturnType_29(Session s, Frame f) {
        return s.rule(f, 29, 1072, true, true, TinyExpressionP4Parser::e243, null, null, null, K6, K7, false, K4, false, K53);
    }
    private static Match parseReturnType_30(Session s, Frame f) {
        return s.rule(f, 30, 1073, true, true, TinyExpressionP4Parser::e245, null, null, null, K6, K7, false, K4, false, K54);
    }
    private static Match parseExternalBooleanInvocation_31(Session s, Frame f) {
        return s.rule(f, 31, 1074, true, true, TinyExpressionP4Parser::e250, "recipe:ExternalBooleanInvocation", null, "construct", K55, K7, false, K4, false, K56);
    }
    private static Match parseExternalNumberInvocation_32(Session s, Frame f) {
        return s.rule(f, 32, 1075, true, true, TinyExpressionP4Parser::e271, "recipe:ExternalNumberInvocation", null, "construct", K57, K7, false, K4, false, K58);
    }
    private static Match parseExternalStringInvocation_33(Session s, Frame f) {
        return s.rule(f, 33, 1076, true, true, TinyExpressionP4Parser::e297, "recipe:ExternalStringInvocation", null, "construct", K59, K7, false, K4, false, K60);
    }
    private static Match parseExternalObjectInvocation_34(Session s, Frame f) {
        return s.rule(f, 34, 1077, true, true, TinyExpressionP4Parser::e318, "recipe:ExternalObjectInvocation", null, "construct", K61, K7, false, K4, false, K62);
    }
    private static Match parseMethodInvocationHeader_35(Session s, Frame f) {
        return s.rule(f, 35, 1078, true, true, TinyExpressionP4Parser::e339, null, null, null, K6, K7, false, K4, false, K63);
    }
    private static Match parseMethodInvocation_36(Session s, Frame f) {
        return s.rule(f, 36, 1079, true, true, TinyExpressionP4Parser::e345, "recipe:MethodInvocation", null, "construct", K64, K65, false, K4, false, K66);
    }
    private static Match parseArgumentTernary_37(Session s, Frame f) {
        return s.rule(f, 37, 1080, true, true, TinyExpressionP4Parser::e352, "recipe:ArgumentTernary", null, "construct", K67, K7, false, K4, false, K68);
    }
    private static Match parseArgumentExpression_38(Session s, Frame f) {
        return s.rule(f, 38, 1081, true, true, TinyExpressionP4Parser::e358, "recipe:ArgumentExpression", null, "construct", K69, K7, false, K4, false, K70);
    }
    private static Match parseArguments_39(Session s, Frame f) {
        return s.rule(f, 39, 1082, true, true, TinyExpressionP4Parser::e363, "recipe:Arguments", null, "construct", K71, K7, false, K4, false, K72);
    }
    private static Match parseNumberExpression_40(Session s, Frame f) {
        return s.rule(f, 40, 1083, true, true, TinyExpressionP4Parser::e369, "recipe:NumberExpression", "TinyExpressionP4AST.BinaryExpr", "assocLists", K73, K7, false, K4, false, K74);
    }
    private static Match parseNumberTerm_41(Session s, Frame f) {
        return s.rule(f, 41, 1084, true, true, TinyExpressionP4Parser::e375, "recipe:NumberTerm", "TinyExpressionP4AST.BinaryExpr", "assocLists", K75, K7, false, K4, false, K76);
    }
    private static Match parseAddOp_42(Session s, Frame f) {
        return s.rule(f, 42, 1085, true, true, TinyExpressionP4Parser::e381, null, null, null, K6, K7, false, K4, false, K77);
    }
    private static Match parseMulOp_43(Session s, Frame f) {
        return s.rule(f, 43, 1086, true, true, TinyExpressionP4Parser::e384, null, null, null, K6, K7, false, K4, false, K78);
    }
    private static Match parseMathFunction_44(Session s, Frame f) {
        return s.rule(f, 44, 1087, true, true, TinyExpressionP4Parser::e387, null, null, null, K6, K7, false, K4, false, K79);
    }
    private static Match parseSinFunction_45(Session s, Frame f) {
        return s.rule(f, 45, 1088, true, true, TinyExpressionP4Parser::e402, "recipe:SinFunction", null, "construct", K80, K7, false, K4, false, K81);
    }
    private static Match parseCosFunction_46(Session s, Frame f) {
        return s.rule(f, 46, 1089, true, true, TinyExpressionP4Parser::e407, "recipe:CosFunction", null, "construct", K82, K7, false, K4, false, K83);
    }
    private static Match parseTanFunction_47(Session s, Frame f) {
        return s.rule(f, 47, 1090, true, true, TinyExpressionP4Parser::e412, "recipe:TanFunction", null, "construct", K84, K7, false, K4, false, K85);
    }
    private static Match parseSqrtFunction_48(Session s, Frame f) {
        return s.rule(f, 48, 1091, true, true, TinyExpressionP4Parser::e417, "recipe:SqrtFunction", null, "construct", K86, K7, false, K4, false, K87);
    }
    private static Match parseMinFunction_49(Session s, Frame f) {
        return s.rule(f, 49, 1092, true, true, TinyExpressionP4Parser::e422, "recipe:MinFunction", null, "construct", K88, K7, false, K4, false, K89);
    }
    private static Match parseMaxFunction_50(Session s, Frame f) {
        return s.rule(f, 50, 1093, true, true, TinyExpressionP4Parser::e431, "recipe:MaxFunction", null, "construct", K90, K7, false, K4, false, K91);
    }
    private static Match parseRandomFunction_51(Session s, Frame f) {
        return s.rule(f, 51, 1094, true, true, TinyExpressionP4Parser::e440, "recipe:RandomFunction", null, "construct", K6, K7, false, K4, false, K92);
    }
    private static Match parseAbsFunction_52(Session s, Frame f) {
        return s.rule(f, 52, 1095, true, true, TinyExpressionP4Parser::e444, "recipe:AbsFunction", null, "construct", K93, K7, false, K4, false, K94);
    }
    private static Match parseRoundFunction_53(Session s, Frame f) {
        return s.rule(f, 53, 1096, true, true, TinyExpressionP4Parser::e449, "recipe:RoundFunction", null, "construct", K95, K7, false, K4, false, K96);
    }
    private static Match parseCeilFunction_54(Session s, Frame f) {
        return s.rule(f, 54, 1097, true, true, TinyExpressionP4Parser::e454, "recipe:CeilFunction", null, "construct", K97, K7, false, K4, false, K98);
    }
    private static Match parseFloorFunction_55(Session s, Frame f) {
        return s.rule(f, 55, 1098, true, true, TinyExpressionP4Parser::e459, "recipe:FloorFunction", null, "construct", K99, K7, false, K4, false, K100);
    }
    private static Match parsePowFunction_56(Session s, Frame f) {
        return s.rule(f, 56, 1099, true, true, TinyExpressionP4Parser::e464, "recipe:PowFunction", null, "construct", K101, K7, false, K4, false, K102);
    }
    private static Match parseLogFunction_57(Session s, Frame f) {
        return s.rule(f, 57, 1100, true, true, TinyExpressionP4Parser::e471, "recipe:LogFunction", null, "construct", K103, K7, false, K4, false, K104);
    }
    private static Match parseExpFunction_58(Session s, Frame f) {
        return s.rule(f, 58, 1101, true, true, TinyExpressionP4Parser::e476, "recipe:ExpFunction", null, "construct", K105, K7, false, K4, false, K106);
    }
    private static Match parseToNumFunction_59(Session s, Frame f) {
        return s.rule(f, 59, 1102, true, true, TinyExpressionP4Parser::e481, "recipe:ToNumFunction", null, "construct", K107, K7, false, K4, false, K108);
    }
    private static Match parseNumberFactor_60(Session s, Frame f) {
        return s.rule(f, 60, 1103, true, true, TinyExpressionP4Parser::e488, null, null, null, K6, K7, false, K4, false, K109);
    }
    private static Match parseToUpperCaseFunction_61(Session s, Frame f) {
        return s.rule(f, 61, 1104, true, true, TinyExpressionP4Parser::e505, "recipe:ToUpperCaseFunction", null, "construct", K110, K7, false, K4, false, K111);
    }
    private static Match parseToLowerCaseFunction_62(Session s, Frame f) {
        return s.rule(f, 62, 1105, true, true, TinyExpressionP4Parser::e510, "recipe:ToLowerCaseFunction", null, "construct", K112, K7, false, K4, false, K113);
    }
    private static Match parseTrimFunction_63(Session s, Frame f) {
        return s.rule(f, 63, 1106, true, true, TinyExpressionP4Parser::e515, "recipe:TrimFunction", null, "construct", K114, K7, false, K4, false, K115);
    }
    private static Match parseLengthFunction_64(Session s, Frame f) {
        return s.rule(f, 64, 1107, true, true, TinyExpressionP4Parser::e520, "recipe:LengthFunction", null, "construct", K116, K7, false, K4, false, K117);
    }
    private static Match parseLenFunction_65(Session s, Frame f) {
        return s.rule(f, 65, 1108, true, true, TinyExpressionP4Parser::e525, "recipe:LenFunction", null, "construct", K118, K7, false, K4, false, K119);
    }
    private static Match parseToUpperCaseDotMethod_66(Session s, Frame f) {
        return s.rule(f, 66, 1109, true, true, TinyExpressionP4Parser::e530, "recipe:ToUpperCaseDotMethod", null, "construct", K120, K7, false, K4, false, K121);
    }
    private static Match parseToLowerCaseDotMethod_67(Session s, Frame f) {
        return s.rule(f, 67, 1110, true, true, TinyExpressionP4Parser::e535, "recipe:ToLowerCaseDotMethod", null, "construct", K122, K7, false, K4, false, K123);
    }
    private static Match parseTrimDotMethod_68(Session s, Frame f) {
        return s.rule(f, 68, 1111, true, true, TinyExpressionP4Parser::e540, "recipe:TrimDotMethod", null, "construct", K124, K7, false, K4, false, K125);
    }
    private static Match parseLengthDotMethod_69(Session s, Frame f) {
        return s.rule(f, 69, 1112, true, true, TinyExpressionP4Parser::e545, "recipe:LengthDotMethod", null, "construct", K126, K7, false, K4, false, K127);
    }
    private static Match parseStartsWithFunction_70(Session s, Frame f) {
        return s.rule(f, 70, 1113, true, true, TinyExpressionP4Parser::e550, "recipe:StartsWithFunction", null, "construct", K128, K7, false, K4, false, K129);
    }
    private static Match parseEndsWithFunction_71(Session s, Frame f) {
        return s.rule(f, 71, 1114, true, true, TinyExpressionP4Parser::e561, "recipe:EndsWithFunction", null, "construct", K130, K7, false, K4, false, K131);
    }
    private static Match parseContainsFunction_72(Session s, Frame f) {
        return s.rule(f, 72, 1115, true, true, TinyExpressionP4Parser::e572, "recipe:ContainsFunction", null, "construct", K132, K7, false, K4, false, K133);
    }
    private static Match parseInMethod_73(Session s, Frame f) {
        return s.rule(f, 73, 1116, true, true, TinyExpressionP4Parser::e583, "recipe:InMethod", null, "construct", K134, K7, false, K4, false, K135);
    }
    private static Match parseStartsWithDotMethod_74(Session s, Frame f) {
        return s.rule(f, 74, 1117, true, true, TinyExpressionP4Parser::e593, "recipe:StartsWithDotMethod", null, "construct", K136, K7, false, K4, false, K137);
    }
    private static Match parseEndsWithDotMethod_75(Session s, Frame f) {
        return s.rule(f, 75, 1118, true, true, TinyExpressionP4Parser::e603, "recipe:EndsWithDotMethod", null, "construct", K138, K7, false, K4, false, K139);
    }
    private static Match parseContainsDotMethod_76(Session s, Frame f) {
        return s.rule(f, 76, 1119, true, true, TinyExpressionP4Parser::e613, "recipe:ContainsDotMethod", null, "construct", K140, K7, false, K4, false, K141);
    }
    private static Match parseStringPredicateReceiver_77(Session s, Frame f) {
        return s.rule(f, 77, 1120, true, true, TinyExpressionP4Parser::e623, null, null, null, K6, K7, false, K4, false, K142);
    }
    private static Match parseIsPresentFunction_78(Session s, Frame f) {
        return s.rule(f, 78, 1121, true, true, TinyExpressionP4Parser::e628, "recipe:IsPresentFunction", null, "construct", K143, K7, false, K4, false, K144);
    }
    private static Match parseInTimeRangeFunction_79(Session s, Frame f) {
        return s.rule(f, 79, 1122, true, true, TinyExpressionP4Parser::e633, "recipe:InTimeRangeFunction", null, "construct", K145, K7, false, K4, false, K146);
    }
    private static Match parseInDayTimeRangeFunction_80(Session s, Frame f) {
        return s.rule(f, 80, 1123, true, true, TinyExpressionP4Parser::e640, "recipe:InDayTimeRangeFunction", null, "construct", K147, K7, false, K4, false, K148);
    }
    private static Match parseDayOfWeek_81(Session s, Frame f) {
        return s.rule(f, 81, 1124, true, true, TinyExpressionP4Parser::e651, null, null, null, K6, K7, false, K4, false, K149);
    }
    private static Match parseSliceBaseReceiver_82(Session s, Frame f) {
        return s.rule(f, 82, 1125, true, true, TinyExpressionP4Parser::e659, null, null, null, K6, K7, false, K4, false, K150);
    }
    private static Match parseSliceStartIndex_83(Session s, Frame f) {
        return s.rule(f, 83, 1126, true, true, TinyExpressionP4Parser::e675, null, null, null, K6, K7, false, K4, false, K151);
    }
    private static Match parseSliceEndIndex_84(Session s, Frame f) {
        return s.rule(f, 84, 1127, true, true, TinyExpressionP4Parser::e677, null, null, null, K6, K7, false, K4, false, K152);
    }
    private static Match parseSliceStepIndex_85(Session s, Frame f) {
        return s.rule(f, 85, 1128, true, true, TinyExpressionP4Parser::e679, null, null, null, K6, K7, false, K4, false, K153);
    }
    private static Match parseSliceBaseExpression_86(Session s, Frame f) {
        return s.rule(f, 86, 1129, true, true, TinyExpressionP4Parser::e681, "recipe:SliceBaseExpression", null, "construct", K154, K7, false, K4, false, K155);
    }
    private static Match parseSliceNestedExpression_87(Session s, Frame f) {
        return s.rule(f, 87, 1130, true, true, TinyExpressionP4Parser::e738, "recipe:SliceNestedExpression", null, "construct", K156, K7, false, K4, false, K155);
    }
    private static Match parseSliceExpression_88(Session s, Frame f) {
        return s.rule(f, 88, 1131, true, true, TinyExpressionP4Parser::e795, null, null, null, K6, K7, false, K4, false, K157);
    }
    private static Match parseStringExpression_89(Session s, Frame f) {
        return s.rule(f, 89, 1132, true, true, TinyExpressionP4Parser::e798, "recipe:StringExpression", null, "assocLists", K158, K7, false, K4, false, K159);
    }
    private static Match parseParenthesizedStringExpression_90(Session s, Frame f) {
        return s.rule(f, 90, 1133, true, true, TinyExpressionP4Parser::e804, null, null, null, K6, K7, false, K4, false, K160);
    }
    private static Match parseStringTerm_91(Session s, Frame f) {
        return s.rule(f, 91, 1134, true, true, TinyExpressionP4Parser::e808, null, null, null, K6, K7, false, K4, false, K161);
    }
    private static Match parseStringCastVariable_92(Session s, Frame f) {
        return s.rule(f, 92, 1135, true, true, TinyExpressionP4Parser::e825, "recipe:StringCastVariable", null, "construct", K162, K7, false, K4, false, K163);
    }
    private static Match parseStringTypedVariable_93(Session s, Frame f) {
        return s.rule(f, 93, 1136, true, true, TinyExpressionP4Parser::e834, "recipe:StringTypedVariable", null, "construct", K164, K7, false, K4, false, K165);
    }
    private static Match parseBooleanExpression_94(Session s, Frame f) {
        return s.rule(f, 94, 1137, true, true, TinyExpressionP4Parser::e842, "recipe:BooleanExpression", null, "assocLists", K166, K7, false, K4, false, K167);
    }
    private static Match parseBooleanAndExpression_95(Session s, Frame f) {
        return s.rule(f, 95, 1138, true, true, TinyExpressionP4Parser::e848, "recipe:BooleanAndExpression", null, "assocLists", K168, K7, false, K4, false, K169);
    }
    private static Match parseBooleanXorExpression_96(Session s, Frame f) {
        return s.rule(f, 96, 1139, true, true, TinyExpressionP4Parser::e854, "recipe:BooleanXorExpression", null, "assocLists", K170, K7, false, K4, false, K171);
    }
    private static Match parseNotExpression_97(Session s, Frame f) {
        return s.rule(f, 97, 1140, true, true, TinyExpressionP4Parser::e860, "recipe:NotExpression", null, "construct", K172, K7, false, K4, false, K173);
    }
    private static Match parseBooleanComparable_98(Session s, Frame f) {
        return s.rule(f, 98, 1141, true, true, TinyExpressionP4Parser::e865, null, null, null, K6, K7, false, K4, false, K174);
    }
    private static Match parseBooleanEqualityExpression_99(Session s, Frame f) {
        return s.rule(f, 99, 1142, true, true, TinyExpressionP4Parser::e888, "recipe:BooleanEqualityExpression", null, "construct", K175, K7, false, K4, false, K176);
    }
    private static Match parseBooleanFactor_100(Session s, Frame f) {
        return s.rule(f, 100, 1143, true, true, TinyExpressionP4Parser::e892, "recipe:BooleanFactor", null, "construct", K177, K7, false, K4, false, K178);
    }
    private static Match parseStringComparisonExpression_101(Session s, Frame f) {
        return s.rule(f, 101, 1144, true, true, TinyExpressionP4Parser::e897, "recipe:StringComparisonExpression", null, "construct", K179, K7, false, K4, false, K180);
    }
    private static Match parseEqualityOp_102(Session s, Frame f) {
        return s.rule(f, 102, 1145, true, true, TinyExpressionP4Parser::e901, null, null, null, K6, K7, false, K4, false, K181);
    }
    private static Match parseComparisonExpression_103(Session s, Frame f) {
        return s.rule(f, 103, 1146, true, true, TinyExpressionP4Parser::e904, "recipe:ComparisonExpression", null, "construct", K182, K7, false, K4, false, K183);
    }
    private static Match parseCompareOp_104(Session s, Frame f) {
        return s.rule(f, 104, 1147, true, true, TinyExpressionP4Parser::e908, null, null, null, K6, K7, false, K4, false, K184);
    }
    private static Match parseObjectExpression_105(Session s, Frame f) {
        return s.rule(f, 105, 1148, true, true, TinyExpressionP4Parser::e915, "recipe:ObjectExpression", null, "construct", K185, K7, false, K4, false, K186);
    }
    private static Match parseIfExpression_106(Session s, Frame f) {
        return s.rule(f, 106, 1149, true, true, TinyExpressionP4Parser::e922, "recipe:IfExpression", null, "construct", K187, K7, false, K4, false, K188);
    }
    private static Match parseBranchExpression_107(Session s, Frame f) {
        return s.rule(f, 107, 1150, true, true, TinyExpressionP4Parser::e934, "recipe:BranchExpression", null, "construct", K189, K7, false, K4, false, K190);
    }
    private static Match parseTernaryExpression_108(Session s, Frame f) {
        return s.rule(f, 108, 1151, true, true, TinyExpressionP4Parser::e943, "recipe:TernaryExpression", null, "construct", K191, K7, false, K4, false, K192);
    }
    private static Match parseNumberMatchExpression_109(Session s, Frame f) {
        return s.rule(f, 109, 1152, true, true, TinyExpressionP4Parser::e951, "recipe:NumberMatchExpression", null, "construct", K193, K7, false, K4, false, K194);
    }
    private static Match parseNumberCase_110(Session s, Frame f) {
        return s.rule(f, 110, 1153, true, true, TinyExpressionP4Parser::e962, "recipe:NumberCase", null, "construct", K195, K7, false, K4, false, K196);
    }
    private static Match parseNumberDefaultCase_111(Session s, Frame f) {
        return s.rule(f, 111, 1154, true, true, TinyExpressionP4Parser::e966, "recipe:NumberDefaultCase", null, "construct", K197, K7, false, K4, false, K198);
    }
    private static Match parseNumberCaseValue_112(Session s, Frame f) {
        return s.rule(f, 112, 1155, true, true, TinyExpressionP4Parser::e970, "recipe:NumberCaseValue", null, "construct", K199, K7, false, K4, false, K200);
    }
    private static Match parseStringMatchExpression_113(Session s, Frame f) {
        return s.rule(f, 113, 1156, true, true, TinyExpressionP4Parser::e972, "recipe:StringMatchExpression", null, "construct", K201, K7, false, K4, false, K194);
    }
    private static Match parseStringCase_114(Session s, Frame f) {
        return s.rule(f, 114, 1157, true, true, TinyExpressionP4Parser::e983, "recipe:StringCase", null, "construct", K202, K7, false, K4, false, K196);
    }
    private static Match parseStringDefaultCase_115(Session s, Frame f) {
        return s.rule(f, 115, 1158, true, true, TinyExpressionP4Parser::e987, "recipe:StringDefaultCase", null, "construct", K203, K7, false, K4, false, K198);
    }
    private static Match parseStringCaseValue_116(Session s, Frame f) {
        return s.rule(f, 116, 1159, true, true, TinyExpressionP4Parser::e991, "recipe:StringCaseValue", null, "construct", K204, K7, false, K4, false, K205);
    }
    private static Match parseBooleanMatchExpression_117(Session s, Frame f) {
        return s.rule(f, 117, 1160, true, true, TinyExpressionP4Parser::e993, "recipe:BooleanMatchExpression", null, "construct", K206, K7, false, K4, false, K194);
    }
    private static Match parseBooleanCase_118(Session s, Frame f) {
        return s.rule(f, 118, 1161, true, true, TinyExpressionP4Parser::e1004, "recipe:BooleanCase", null, "construct", K207, K7, false, K4, false, K196);
    }
    private static Match parseBooleanDefaultCase_119(Session s, Frame f) {
        return s.rule(f, 119, 1162, true, true, TinyExpressionP4Parser::e1008, "recipe:BooleanDefaultCase", null, "construct", K208, K7, false, K4, false, K198);
    }
    private static Match parseBooleanCaseValue_120(Session s, Frame f) {
        return s.rule(f, 120, 1163, true, true, TinyExpressionP4Parser::e1012, "recipe:BooleanCaseValue", null, "construct", K209, K7, false, K4, false, K210);
    }
    private static Match parseVariableRef_121(Session s, Frame f) {
        return s.rule(f, 121, 1164, true, true, TinyExpressionP4Parser::e1014, "recipe:VariableRef", null, "construct", K211, K212, false, K4, false, K49);
    }
    private static Match parseTypeKeyword_122(Session s, Frame f) {
        return s.rule(f, 122, 1165, true, true, TinyExpressionP4Parser::e1022, null, null, null, K6, K7, false, K4, false, K213);
    }
    private static Match parseExpression_123(Session s, Frame f) {
        return s.rule(f, 123, 1166, true, true, TinyExpressionP4Parser::e1033, "recipe:Expression", null, "construct", K214, K7, false, K4, false, K160);
    }
    private static Match e0(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e0M : TinyExpressionP4Parser::e0C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:886:1045:body/seq", false, false, "node", true, K4);
    }
    private static Match e0C(Session s, Frame f) {
        return s.sequence(f, K220, K219);
    }
    private static Match e0M(Session s, Frame f) {
        return s.sequence(f, K220, K219);
    }
    private static Match e1(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1M : TinyExpressionP4Parser::e1C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:886:899:body/0/repeat", true, false, "node", true, K221);
    }
    private static Match e1C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e2, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e1M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e2, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e2(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e2M : TinyExpressionP4Parser::e2C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:888:897:body/0/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e2C(Session s, Frame f) {
        return parseCodeBlock_1(s, f);
    }
    private static Match e2M(Session s, Frame f) {
        return parseCodeBlock_1(s, f);
    }
    private static Match e3(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e3M : TinyExpressionP4Parser::e3C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:900:930:body/1/repeat", true, false, "node", true, K221);
    }
    private static Match e3C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e4, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e3M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e4, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e4(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e4M : TinyExpressionP4Parser::e4C, K222, K223, K224, K225, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:902:919:body/1/0/ruleRef", false, false, "node", false, K10);
    }
    private static Match e4C(Session s, Frame f) {
        return parseImportDeclaration_2(s, f);
    }
    private static Match e4M(Session s, Frame f) {
        return parseImportDeclaration_2(s, f);
    }
    private static Match e5(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e5M : TinyExpressionP4Parser::e5C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:931:968:body/2/repeat", true, false, "node", true, K221);
    }
    private static Match e5C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e6, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e5M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e6, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e6(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e6M : TinyExpressionP4Parser::e6C, K226, K227, K224, K225, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:933:952:body/2/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e6C(Session s, Frame f) {
        return parseVariableDeclaration_4(s, f);
    }
    private static Match e6M(Session s, Frame f) {
        return parseVariableDeclaration_4(s, f);
    }
    private static Match e7(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e7M : TinyExpressionP4Parser::e7C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:973:987:body/3/repeat", true, false, "node", true, K30);
    }
    private static Match e7C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e8, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e7M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e8, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e8(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e8M : TinyExpressionP4Parser::e8C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:975:985:body/3/0/ruleRef", false, false, "node", true, K4);
    }
    private static Match e8C(Session s, Frame f) {
        return parseAnnotation_16(s, f);
    }
    private static Match e8M(Session s, Frame f) {
        return parseAnnotation_16(s, f);
    }
    private static Match e9(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e9M : TinyExpressionP4Parser::e9C, K229, K230, K224, K225, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:988:998:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e9C(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e9M(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e10(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e10M : TinyExpressionP4Parser::e10C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1011:1041:body/5/repeat", true, false, "node", true, K221);
    }
    private static Match e10C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e11, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e10M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e11, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e11(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e11M : TinyExpressionP4Parser::e11C, K231, K232, K224, K225, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1013:1030:body/5/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e11C(Session s, Frame f) {
        return parseMethodDeclaration_19(s, f);
    }
    private static Match e11M(Session s, Frame f) {
        return parseMethodDeclaration_19(s, f);
    }
    private static Match e12(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e12M : TinyExpressionP4Parser::e12C, K215, K215, K215, K216, "TinyExpressionP4::Formula", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1042:1045:body/6/tokenRef", false, true, "text", false, K233);
    }
    private static Match e12C(Session s, Frame f) {
        return s.builtin(f,"EndOfSource",K234,K235);
    }
    private static Match e12M(Session s, Frame f) {
        return s.builtin(f,"EndOfSource",K234,K235);
    }
    private static Match e13(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e13M : TinyExpressionP4Parser::e13C, K215, K215, K215, K216, "TinyExpressionP4::CodeBlock", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1290:1319:body/seq", false, false, "text", false, K4);
    }
    private static Match e13C(Session s, Frame f) {
        return s.sequence(f, K238, K237);
    }
    private static Match e13M(Session s, Frame f) {
        return s.sequence(f, K238, K237);
    }
    private static Match e14(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e14M : TinyExpressionP4Parser::e14C, K215, K215, K215, K216, "TinyExpressionP4::CodeBlock", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1290:1300:body/0/tokenRef", false, true, "text", false, K4);
    }
    private static Match e14C(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::CODE_START",0);
    }
    private static Match e14M(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::CODE_START",0);
    }
    private static Match e15(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e15M : TinyExpressionP4Parser::e15C, K215, K215, K215, K216, "TinyExpressionP4::CodeBlock", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1301:1310:body/1/tokenRef", false, true, "text", false, K239);
    }
    private static Match e15C(Session s, Frame f) {
        return s.until(f,"```");
    }
    private static Match e15M(Session s, Frame f) {
        return s.until(f,"```");
    }
    private static Match e16(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e16M : TinyExpressionP4Parser::e16C, K215, K215, K215, K216, "TinyExpressionP4::CodeBlock", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1311:1319:body/2/tokenRef", false, true, "text", false, K4);
    }
    private static Match e16C(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::CODE_END",1);
    }
    private static Match e16M(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::CODE_END",1);
    }
    private static Match e17(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e17M : TinyExpressionP4Parser::e17C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1767:1850:body/seq", false, false, "node", false, K4);
    }
    private static Match e17C(Session s, Frame f) {
        return s.sequence(f, K242, K241);
    }
    private static Match e17M(Session s, Frame f) {
        return s.sequence(f, K242, K241);
    }
    private static Match e18(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e18M : TinyExpressionP4Parser::e18C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1767:1775:body/0/literal", false, true, "text", false, K243);
    }
    private static Match e18C(Session s, Frame f) {
        return s.literal(f, "import", true, false, K244);
    }
    private static Match e18M(Session s, Frame f) {
        return s.literal(f, "import", true, false, K244);
    }
    private static Match e19(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e19M : TinyExpressionP4Parser::e19C, K245, K246, K224, K225, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1776:1785:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e19C(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e19M(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e20(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e20M : TinyExpressionP4Parser::e20C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1797:1823:body/2/optional", true, false, "text", false, K247);
    }
    private static Match e20C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e21);
    }
    private static Match e20M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e21);
    }
    private static Match e21(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e21M : TinyExpressionP4Parser::e21C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1799:1813:body/2/0/seq", false, false, "text", false, K247);
    }
    private static Match e21C(Session s, Frame f) {
        return s.sequence(f, K250, K249);
    }
    private static Match e21M(Session s, Frame f) {
        return s.sequence(f, K250, K249);
    }
    private static Match e22(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e22M : TinyExpressionP4Parser::e22C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1799:1802:body/2/0/0/literal", false, true, "text", false, K247);
    }
    private static Match e22C(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e22M(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e23(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e23M : TinyExpressionP4Parser::e23C, K252, K253, K224, K225, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1803:1813:body/2/0/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e23C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e23M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e24(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e24M : TinyExpressionP4Parser::e24C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1824:1828:body/3/literal", false, true, "text", false, K256);
    }
    private static Match e24C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e24M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e25(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e25M : TinyExpressionP4Parser::e25C, K258, K259, K224, K225, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1829:1839:body/4/tokenRef", false, true, "text", false, K254);
    }
    private static Match e25C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e25M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e26(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e26M : TinyExpressionP4Parser::e26C, K215, K215, K215, K216, "TinyExpressionP4::ImportDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1847:1850:body/5/literal", false, true, "text", false, K260);
    }
    private static Match e26C(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e26M(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e27(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e27M : TinyExpressionP4Parser::e27C, K215, K215, K215, K216, "TinyExpressionP4::ClassName", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1962:body/seq", false, false, "text", false, K4);
    }
    private static Match e27C(Session s, Frame f) {
        return s.sequence(f, K262, K249);
    }
    private static Match e27M(Session s, Frame f) {
        return s.sequence(f, K262, K249);
    }
    private static Match e28(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e28M : TinyExpressionP4Parser::e28C, K263, K264, K224, K225, "TinyExpressionP4::ClassName", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1931:body/0/tokenRef", false, true, "text", false, K254);
    }
    private static Match e28C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e28M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e29(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e29M : TinyExpressionP4Parser::e29C, K215, K215, K215, K216, "TinyExpressionP4::ClassName", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1938:1962:body/1/repeat", true, false, "text", false, K265);
    }
    private static Match e29C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e30, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e29M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e30, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e30(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e30M : TinyExpressionP4Parser::e30C, K215, K215, K215, K216, "TinyExpressionP4::ClassName", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1940:1954:body/1/0/seq", false, false, "text", false, K265);
    }
    private static Match e30C(Session s, Frame f) {
        return s.sequence(f, K266, K249);
    }
    private static Match e30M(Session s, Frame f) {
        return s.sequence(f, K266, K249);
    }
    private static Match e31(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e31M : TinyExpressionP4Parser::e31C, K215, K215, K215, K216, "TinyExpressionP4::ClassName", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1940:1943:body/1/0/0/literal", false, true, "text", false, K265);
    }
    private static Match e31C(Session s, Frame f) {
        return s.literal(f, ".", true, false, K267);
    }
    private static Match e31M(Session s, Frame f) {
        return s.literal(f, ".", true, false, K267);
    }
    private static Match e32(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e32M : TinyExpressionP4Parser::e32C, K268, K269, K224, K225, "TinyExpressionP4::ClassName", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1944:1954:body/1/0/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e32C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e32M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e33(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e33M : TinyExpressionP4Parser::e33C, K215, K215, K215, K216, "TinyExpressionP4::VariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2113:2235:body/choice", false, false, "node", false, K4);
    }
    private static Match e33C(Session s, Frame f) {
        return s.choice(f,K281,false,null,false,K280);
    }
    private static Match e33M(Session s, Frame f) {
        return s.choice(f,K281,false,null,false,K280);
    }
    private static Match e34(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e34M : TinyExpressionP4Parser::e34C, K215, K215, K215, K216, "TinyExpressionP4::VariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2113:2138:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e34C(Session s, Frame f) {
        return parseNumberVariableDeclaration_5(s, f);
    }
    private static Match e34M(Session s, Frame f) {
        return parseNumberVariableDeclaration_5(s, f);
    }
    private static Match e35(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e35M : TinyExpressionP4Parser::e35C, K215, K215, K215, K216, "TinyExpressionP4::VariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2145:2170:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e35C(Session s, Frame f) {
        return parseStringVariableDeclaration_6(s, f);
    }
    private static Match e35M(Session s, Frame f) {
        return parseStringVariableDeclaration_6(s, f);
    }
    private static Match e36(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e36M : TinyExpressionP4Parser::e36C, K215, K215, K215, K216, "TinyExpressionP4::VariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2177:2203:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e36C(Session s, Frame f) {
        return parseBooleanVariableDeclaration_7(s, f);
    }
    private static Match e36M(Session s, Frame f) {
        return parseBooleanVariableDeclaration_7(s, f);
    }
    private static Match e37(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e37M : TinyExpressionP4Parser::e37C, K215, K215, K215, K216, "TinyExpressionP4::VariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2210:2235:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e37C(Session s, Frame f) {
        return parseObjectVariableDeclaration_8(s, f);
    }
    private static Match e37M(Session s, Frame f) {
        return parseObjectVariableDeclaration_8(s, f);
    }
    private static Match e38(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e38M : TinyExpressionP4Parser::e38C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2408:2584:body/seq", false, false, "node", true, K4);
    }
    private static Match e38C(Session s, Frame f) {
        return s.sequence(f, K282, K219);
    }
    private static Match e38M(Session s, Frame f) {
        return s.sequence(f, K282, K219);
    }
    private static Match e39(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e39M : TinyExpressionP4Parser::e39C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2408:2430:body/0/group", false, false, "text", false, K283);
    }
    private static Match e39C(Session s, Frame f) {
        return e40(s, f);
    }
    private static Match e39M(Session s, Frame f) {
        return e40(s, f);
    }
    private static Match e40(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e40M : TinyExpressionP4Parser::e40C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2410:2428:body/0/0/choice", false, false, "text", false, K283);
    }
    private static Match e40C(Session s, Frame f) {
        return s.choice(f,K284,false,null,false);
    }
    private static Match e40M(Session s, Frame f) {
        return s.choice(f,K284,false,null,false);
    }
    private static Match e41(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e41M : TinyExpressionP4Parser::e41C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2410:2420:body/0/0/0/literal", false, true, "text", false, K285);
    }
    private static Match e41C(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e41M(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e42(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e42M : TinyExpressionP4Parser::e42C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2423:2428:body/0/0/1/literal", false, true, "text", false, K287);
    }
    private static Match e42C(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e42M(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e43(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e43M : TinyExpressionP4Parser::e43C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2435:2438:body/1/literal", false, true, "text", false, K49);
    }
    private static Match e43C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e43M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e44(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e44M : TinyExpressionP4Parser::e44C, K290, K291, K224, K225, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2439:2449:body/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e44C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e44M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e45(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e45M : TinyExpressionP4Parser::e45C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2463:2481:body/3/optional", true, false, "text", false, K292);
    }
    private static Match e45C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e46);
    }
    private static Match e45M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e46);
    }
    private static Match e46(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e46M : TinyExpressionP4Parser::e46C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2465:2479:body/3/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e46C(Session s, Frame f) {
        return parseNumberTypeHint_10(s, f);
    }
    private static Match e46M(Session s, Frame f) {
        return parseNumberTypeHint_10(s, f);
    }
    private static Match e47(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e47M : TinyExpressionP4Parser::e47C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2486:2550:body/4/optional", true, false, "node", true, K293);
    }
    private static Match e47C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e48);
    }
    private static Match e47M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e48);
    }
    private static Match e48(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e48M : TinyExpressionP4Parser::e48C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2488:2541:body/4/0/seq", false, false, "node", true, K293);
    }
    private static Match e48C(Session s, Frame f) {
        return s.sequence(f, K294, K237);
    }
    private static Match e48M(Session s, Frame f) {
        return s.sequence(f, K294, K237);
    }
    private static Match e49(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e49M : TinyExpressionP4Parser::e49C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2488:2493:body/4/0/0/literal", false, true, "text", false, K293);
    }
    private static Match e49C(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e49M(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e50(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e50M : TinyExpressionP4Parser::e50C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2494:2524:body/4/0/1/optional", true, false, "node", false, K292);
    }
    private static Match e50C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e51);
    }
    private static Match e50M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e51);
    }
    private static Match e51(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e51M : TinyExpressionP4Parser::e51C, K296, K297, K224, K225, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2496:2508:body/4/0/1/0/ruleRef", false, false, "node", false, K28);
    }
    private static Match e51C(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e51M(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e52(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e52M : TinyExpressionP4Parser::e52C, K298, K299, K224, K225, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2525:2541:body/4/0/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e52C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e52M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e53(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e53M : TinyExpressionP4Parser::e53C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2555:2576:body/5/optional", true, false, "text", false, K292);
    }
    private static Match e53C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e54);
    }
    private static Match e53M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e54);
    }
    private static Match e54(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e54M : TinyExpressionP4Parser::e54C, K300, K301, K224, K225, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2557:2568:body/5/0/ruleRef", false, false, "text", false, K29);
    }
    private static Match e54C(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e54M(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e55(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e55M : TinyExpressionP4Parser::e55C, K215, K215, K215, K216, "TinyExpressionP4::NumberVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2581:2584:body/6/literal", false, true, "text", false, K260);
    }
    private static Match e55C(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e55M(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e56(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e56M : TinyExpressionP4Parser::e56C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2757:2933:body/seq", false, false, "node", true, K4);
    }
    private static Match e56C(Session s, Frame f) {
        return s.sequence(f, K302, K219);
    }
    private static Match e56M(Session s, Frame f) {
        return s.sequence(f, K302, K219);
    }
    private static Match e57(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e57M : TinyExpressionP4Parser::e57C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2757:2779:body/0/group", false, false, "text", false, K283);
    }
    private static Match e57C(Session s, Frame f) {
        return e58(s, f);
    }
    private static Match e57M(Session s, Frame f) {
        return e58(s, f);
    }
    private static Match e58(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e58M : TinyExpressionP4Parser::e58C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2759:2777:body/0/0/choice", false, false, "text", false, K283);
    }
    private static Match e58C(Session s, Frame f) {
        return s.choice(f,K303,false,null,false);
    }
    private static Match e58M(Session s, Frame f) {
        return s.choice(f,K303,false,null,false);
    }
    private static Match e59(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e59M : TinyExpressionP4Parser::e59C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2759:2769:body/0/0/0/literal", false, true, "text", false, K285);
    }
    private static Match e59C(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e59M(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e60(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e60M : TinyExpressionP4Parser::e60C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2772:2777:body/0/0/1/literal", false, true, "text", false, K287);
    }
    private static Match e60C(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e60M(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e61(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e61M : TinyExpressionP4Parser::e61C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2784:2787:body/1/literal", false, true, "text", false, K49);
    }
    private static Match e61C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e61M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e62(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e62M : TinyExpressionP4Parser::e62C, K304, K291, K224, K225, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2788:2798:body/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e62C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e62M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e63(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e63M : TinyExpressionP4Parser::e63C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2812:2830:body/3/optional", true, false, "text", false, K292);
    }
    private static Match e63C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e64);
    }
    private static Match e63M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e64);
    }
    private static Match e64(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e64M : TinyExpressionP4Parser::e64C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2814:2828:body/3/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e64C(Session s, Frame f) {
        return parseStringTypeHint_11(s, f);
    }
    private static Match e64M(Session s, Frame f) {
        return parseStringTypeHint_11(s, f);
    }
    private static Match e65(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e65M : TinyExpressionP4Parser::e65C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2835:2899:body/4/optional", true, false, "node", true, K293);
    }
    private static Match e65C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e66);
    }
    private static Match e65M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e66);
    }
    private static Match e66(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e66M : TinyExpressionP4Parser::e66C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2837:2890:body/4/0/seq", false, false, "node", true, K293);
    }
    private static Match e66C(Session s, Frame f) {
        return s.sequence(f, K305, K237);
    }
    private static Match e66M(Session s, Frame f) {
        return s.sequence(f, K305, K237);
    }
    private static Match e67(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e67M : TinyExpressionP4Parser::e67C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2837:2842:body/4/0/0/literal", false, true, "text", false, K293);
    }
    private static Match e67C(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e67M(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e68(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e68M : TinyExpressionP4Parser::e68C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2843:2873:body/4/0/1/optional", true, false, "node", false, K292);
    }
    private static Match e68C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e69);
    }
    private static Match e68M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e69);
    }
    private static Match e69(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e69M : TinyExpressionP4Parser::e69C, K306, K297, K224, K225, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2845:2857:body/4/0/1/0/ruleRef", false, false, "node", false, K28);
    }
    private static Match e69C(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e69M(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e70(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e70M : TinyExpressionP4Parser::e70C, K307, K299, K224, K225, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2874:2890:body/4/0/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e70C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e70M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e71(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e71M : TinyExpressionP4Parser::e71C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2904:2925:body/5/optional", true, false, "text", false, K292);
    }
    private static Match e71C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e72);
    }
    private static Match e71M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e72);
    }
    private static Match e72(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e72M : TinyExpressionP4Parser::e72C, K308, K301, K224, K225, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2906:2917:body/5/0/ruleRef", false, false, "text", false, K29);
    }
    private static Match e72C(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e72M(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e73(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e73M : TinyExpressionP4Parser::e73C, K215, K215, K215, K216, "TinyExpressionP4::StringVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2930:2933:body/6/literal", false, true, "text", false, K260);
    }
    private static Match e73C(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e73M(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e74(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e74M : TinyExpressionP4Parser::e74C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3108:3286:body/seq", false, false, "node", true, K4);
    }
    private static Match e74C(Session s, Frame f) {
        return s.sequence(f, K309, K219);
    }
    private static Match e74M(Session s, Frame f) {
        return s.sequence(f, K309, K219);
    }
    private static Match e75(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e75M : TinyExpressionP4Parser::e75C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3108:3130:body/0/group", false, false, "text", false, K283);
    }
    private static Match e75C(Session s, Frame f) {
        return e76(s, f);
    }
    private static Match e75M(Session s, Frame f) {
        return e76(s, f);
    }
    private static Match e76(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e76M : TinyExpressionP4Parser::e76C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3110:3128:body/0/0/choice", false, false, "text", false, K283);
    }
    private static Match e76C(Session s, Frame f) {
        return s.choice(f,K310,false,null,false);
    }
    private static Match e76M(Session s, Frame f) {
        return s.choice(f,K310,false,null,false);
    }
    private static Match e77(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e77M : TinyExpressionP4Parser::e77C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3110:3120:body/0/0/0/literal", false, true, "text", false, K285);
    }
    private static Match e77C(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e77M(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e78(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e78M : TinyExpressionP4Parser::e78C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3123:3128:body/0/0/1/literal", false, true, "text", false, K287);
    }
    private static Match e78C(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e78M(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e79(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e79M : TinyExpressionP4Parser::e79C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3135:3138:body/1/literal", false, true, "text", false, K49);
    }
    private static Match e79C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e79M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e80(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e80M : TinyExpressionP4Parser::e80C, K311, K291, K224, K225, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3139:3149:body/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e80C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e80M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e81(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e81M : TinyExpressionP4Parser::e81C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3163:3182:body/3/optional", true, false, "text", false, K292);
    }
    private static Match e81C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e82);
    }
    private static Match e81M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e82);
    }
    private static Match e82(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e82M : TinyExpressionP4Parser::e82C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3165:3180:body/3/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e82C(Session s, Frame f) {
        return parseBooleanTypeHint_12(s, f);
    }
    private static Match e82M(Session s, Frame f) {
        return parseBooleanTypeHint_12(s, f);
    }
    private static Match e83(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e83M : TinyExpressionP4Parser::e83C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3187:3252:body/4/optional", true, false, "node", true, K293);
    }
    private static Match e83C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e84);
    }
    private static Match e83M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e84);
    }
    private static Match e84(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e84M : TinyExpressionP4Parser::e84C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3189:3243:body/4/0/seq", false, false, "node", true, K293);
    }
    private static Match e84C(Session s, Frame f) {
        return s.sequence(f, K312, K237);
    }
    private static Match e84M(Session s, Frame f) {
        return s.sequence(f, K312, K237);
    }
    private static Match e85(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e85M : TinyExpressionP4Parser::e85C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3189:3194:body/4/0/0/literal", false, true, "text", false, K293);
    }
    private static Match e85C(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e85M(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e86(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e86M : TinyExpressionP4Parser::e86C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3195:3225:body/4/0/1/optional", true, false, "node", false, K292);
    }
    private static Match e86C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e87);
    }
    private static Match e86M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e87);
    }
    private static Match e87(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e87M : TinyExpressionP4Parser::e87C, K313, K297, K224, K225, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3197:3209:body/4/0/1/0/ruleRef", false, false, "node", false, K28);
    }
    private static Match e87C(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e87M(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e88(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e88M : TinyExpressionP4Parser::e88C, K314, K299, K224, K225, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3226:3243:body/4/0/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e88C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e88M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e89(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e89M : TinyExpressionP4Parser::e89C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3257:3278:body/5/optional", true, false, "text", false, K292);
    }
    private static Match e89C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e90);
    }
    private static Match e89M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e90);
    }
    private static Match e90(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e90M : TinyExpressionP4Parser::e90C, K315, K301, K224, K225, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3259:3270:body/5/0/ruleRef", false, false, "text", false, K29);
    }
    private static Match e90C(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e90M(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e91(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e91M : TinyExpressionP4Parser::e91C, K215, K215, K215, K216, "TinyExpressionP4::BooleanVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3283:3286:body/6/literal", false, true, "text", false, K260);
    }
    private static Match e91C(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e91M(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e92(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e92M : TinyExpressionP4Parser::e92C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3459:3635:body/seq", false, false, "node", true, K4);
    }
    private static Match e92C(Session s, Frame f) {
        return s.sequence(f, K316, K219);
    }
    private static Match e92M(Session s, Frame f) {
        return s.sequence(f, K316, K219);
    }
    private static Match e93(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e93M : TinyExpressionP4Parser::e93C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3459:3481:body/0/group", false, false, "text", false, K283);
    }
    private static Match e93C(Session s, Frame f) {
        return e94(s, f);
    }
    private static Match e93M(Session s, Frame f) {
        return e94(s, f);
    }
    private static Match e94(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e94M : TinyExpressionP4Parser::e94C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3461:3479:body/0/0/choice", false, false, "text", false, K283);
    }
    private static Match e94C(Session s, Frame f) {
        return s.choice(f,K317,false,null,false);
    }
    private static Match e94M(Session s, Frame f) {
        return s.choice(f,K317,false,null,false);
    }
    private static Match e95(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e95M : TinyExpressionP4Parser::e95C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3461:3471:body/0/0/0/literal", false, true, "text", false, K285);
    }
    private static Match e95C(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e95M(Session s, Frame f) {
        return s.literal(f, "variable", true, false, K286);
    }
    private static Match e96(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e96M : TinyExpressionP4Parser::e96C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3474:3479:body/0/0/1/literal", false, true, "text", false, K287);
    }
    private static Match e96C(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e96M(Session s, Frame f) {
        return s.literal(f, "var", true, false, K288);
    }
    private static Match e97(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e97M : TinyExpressionP4Parser::e97C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3486:3489:body/1/literal", false, true, "text", false, K49);
    }
    private static Match e97C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e97M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e98(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e98M : TinyExpressionP4Parser::e98C, K318, K291, K224, K225, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3490:3500:body/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e98C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e98M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e99(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e99M : TinyExpressionP4Parser::e99C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3514:3532:body/3/optional", true, false, "text", false, K292);
    }
    private static Match e99C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e100);
    }
    private static Match e99M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e100);
    }
    private static Match e100(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e100M : TinyExpressionP4Parser::e100C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3516:3530:body/3/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e100C(Session s, Frame f) {
        return parseObjectTypeHint_13(s, f);
    }
    private static Match e100M(Session s, Frame f) {
        return parseObjectTypeHint_13(s, f);
    }
    private static Match e101(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e101M : TinyExpressionP4Parser::e101C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3537:3601:body/4/optional", true, false, "node", true, K293);
    }
    private static Match e101C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e102);
    }
    private static Match e101M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e102);
    }
    private static Match e102(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e102M : TinyExpressionP4Parser::e102C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3539:3592:body/4/0/seq", false, false, "node", true, K293);
    }
    private static Match e102C(Session s, Frame f) {
        return s.sequence(f, K319, K237);
    }
    private static Match e102M(Session s, Frame f) {
        return s.sequence(f, K319, K237);
    }
    private static Match e103(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e103M : TinyExpressionP4Parser::e103C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3539:3544:body/4/0/0/literal", false, true, "text", false, K293);
    }
    private static Match e103C(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e103M(Session s, Frame f) {
        return s.literal(f, "set", true, false, K295);
    }
    private static Match e104(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e104M : TinyExpressionP4Parser::e104C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3545:3575:body/4/0/1/optional", true, false, "node", false, K292);
    }
    private static Match e104C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e105);
    }
    private static Match e104M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e105);
    }
    private static Match e105(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e105M : TinyExpressionP4Parser::e105C, K320, K297, K224, K225, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3547:3559:body/4/0/1/0/ruleRef", false, false, "node", false, K28);
    }
    private static Match e105C(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e105M(Session s, Frame f) {
        return parseOnlyIfAbsent_14(s, f);
    }
    private static Match e106(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e106M : TinyExpressionP4Parser::e106C, K321, K299, K224, K225, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3576:3592:body/4/0/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e106C(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e106M(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e107(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e107M : TinyExpressionP4Parser::e107C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3606:3627:body/5/optional", true, false, "text", false, K292);
    }
    private static Match e107C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e108);
    }
    private static Match e107M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e108);
    }
    private static Match e108(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e108M : TinyExpressionP4Parser::e108C, K322, K301, K224, K225, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3608:3619:body/5/0/ruleRef", false, false, "text", false, K29);
    }
    private static Match e108C(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e108M(Session s, Frame f) {
        return parseDescription_15(s, f);
    }
    private static Match e109(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e109M : TinyExpressionP4Parser::e109C, K215, K215, K215, K216, "TinyExpressionP4::ObjectVariableDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3632:3635:body/6/literal", false, true, "text", false, K260);
    }
    private static Match e109C(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e109M(Session s, Frame f) {
        return s.literal(f, ";", true, false, K261);
    }
    private static Match e110(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e110M : TinyExpressionP4Parser::e110C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3654:3758:body/seq", false, false, "text", false, K4);
    }
    private static Match e110C(Session s, Frame f) {
        return s.sequence(f, K323, K249);
    }
    private static Match e110M(Session s, Frame f) {
        return s.sequence(f, K323, K249);
    }
    private static Match e111(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e111M : TinyExpressionP4Parser::e111C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3654:3662:body/0/optional", true, false, "text", false, K256);
    }
    private static Match e111C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e112);
    }
    private static Match e111M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e112);
    }
    private static Match e112(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e112M : TinyExpressionP4Parser::e112C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3656:3660:body/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e112C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e112M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e113(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e113M : TinyExpressionP4Parser::e113C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3663:3758:body/1/group", false, false, "text", false, K324);
    }
    private static Match e113C(Session s, Frame f) {
        return e114(s, f);
    }
    private static Match e113M(Session s, Frame f) {
        return e114(s, f);
    }
    private static Match e114(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e114M : TinyExpressionP4Parser::e114C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3665:3756:body/1/0/choice", false, false, "text", false, K324);
    }
    private static Match e114C(Session s, Frame f) {
        return s.choice(f,K325,false,null,false);
    }
    private static Match e114M(Session s, Frame f) {
        return s.choice(f,K325,false,null,false);
    }
    private static Match e115(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e115M : TinyExpressionP4Parser::e115C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3665:3673:body/1/0/0/literal", false, true, "text", false, K326);
    }
    private static Match e115C(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e115M(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e116(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e116M : TinyExpressionP4Parser::e116C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3676:3684:body/1/0/1/literal", false, true, "text", false, K328);
    }
    private static Match e116C(Session s, Frame f) {
        return s.literal(f, "Number", true, false, K329);
    }
    private static Match e116M(Session s, Frame f) {
        return s.literal(f, "Number", true, false, K329);
    }
    private static Match e117(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e117M : TinyExpressionP4Parser::e117C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3687:3695:body/1/0/2/literal", false, true, "text", false, K51);
    }
    private static Match e117C(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e117M(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e118(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e118M : TinyExpressionP4Parser::e118C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3698:3706:body/1/0/3/literal", false, true, "text", false, K331);
    }
    private static Match e118C(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e118M(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e119(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e119M : TinyExpressionP4Parser::e119C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3713:3722:body/1/0/4/literal", false, true, "text", false, K52);
    }
    private static Match e119C(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e119M(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e120(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e120M : TinyExpressionP4Parser::e120C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3725:3734:body/1/0/5/literal", false, true, "text", false, K334);
    }
    private static Match e120C(Session s, Frame f) {
        return s.literal(f, "Boolean", true, false, K335);
    }
    private static Match e120M(Session s, Frame f) {
        return s.literal(f, "Boolean", true, false, K335);
    }
    private static Match e121(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e121M : TinyExpressionP4Parser::e121C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3737:3745:body/1/0/6/literal", false, true, "text", false, K53);
    }
    private static Match e121C(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e121M(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e122(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e122M : TinyExpressionP4Parser::e122C, K215, K215, K215, K216, "TinyExpressionP4::TypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3748:3756:body/1/0/7/literal", false, true, "text", false, K337);
    }
    private static Match e122C(Session s, Frame f) {
        return s.literal(f, "Object", true, false, K338);
    }
    private static Match e122M(Session s, Frame f) {
        return s.literal(f, "Object", true, false, K338);
    }
    private static Match e123(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e123M : TinyExpressionP4Parser::e123C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3783:3835:body/seq", false, false, "text", false, K4);
    }
    private static Match e123C(Session s, Frame f) {
        return s.sequence(f, K339, K249);
    }
    private static Match e123M(Session s, Frame f) {
        return s.sequence(f, K339, K249);
    }
    private static Match e124(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e124M : TinyExpressionP4Parser::e124C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3783:3791:body/0/optional", true, false, "text", false, K256);
    }
    private static Match e124C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e125);
    }
    private static Match e124M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e125);
    }
    private static Match e125(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e125M : TinyExpressionP4Parser::e125C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3785:3789:body/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e125C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e125M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e126(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e126M : TinyExpressionP4Parser::e126C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3792:3835:body/1/group", false, false, "text", false, K340);
    }
    private static Match e126C(Session s, Frame f) {
        return e127(s, f);
    }
    private static Match e126M(Session s, Frame f) {
        return e127(s, f);
    }
    private static Match e127(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e127M : TinyExpressionP4Parser::e127C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3794:3833:body/1/0/choice", false, false, "text", false, K340);
    }
    private static Match e127C(Session s, Frame f) {
        return s.choice(f,K341,false,null,false);
    }
    private static Match e127M(Session s, Frame f) {
        return s.choice(f,K341,false,null,false);
    }
    private static Match e128(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e128M : TinyExpressionP4Parser::e128C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3794:3802:body/1/0/0/literal", false, true, "text", false, K326);
    }
    private static Match e128C(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e128M(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e129(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e129M : TinyExpressionP4Parser::e129C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3805:3813:body/1/0/1/literal", false, true, "text", false, K328);
    }
    private static Match e129C(Session s, Frame f) {
        return s.literal(f, "Number", true, false, K329);
    }
    private static Match e129M(Session s, Frame f) {
        return s.literal(f, "Number", true, false, K329);
    }
    private static Match e130(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e130M : TinyExpressionP4Parser::e130C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3816:3823:body/1/0/2/literal", false, true, "text", false, K342);
    }
    private static Match e130C(Session s, Frame f) {
        return s.literal(f, "float", true, false, K343);
    }
    private static Match e130M(Session s, Frame f) {
        return s.literal(f, "float", true, false, K343);
    }
    private static Match e131(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e131M : TinyExpressionP4Parser::e131C, K215, K215, K215, K216, "TinyExpressionP4::NumberTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3826:3833:body/1/0/3/literal", false, true, "text", false, K344);
    }
    private static Match e131C(Session s, Frame f) {
        return s.literal(f, "Float", true, false, K345);
    }
    private static Match e131M(Session s, Frame f) {
        return s.literal(f, "Float", true, false, K345);
    }
    private static Match e132(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e132M : TinyExpressionP4Parser::e132C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3859:3891:body/seq", false, false, "text", false, K4);
    }
    private static Match e132C(Session s, Frame f) {
        return s.sequence(f, K346, K249);
    }
    private static Match e132M(Session s, Frame f) {
        return s.sequence(f, K346, K249);
    }
    private static Match e133(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e133M : TinyExpressionP4Parser::e133C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3859:3867:body/0/optional", true, false, "text", false, K256);
    }
    private static Match e133C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e134);
    }
    private static Match e133M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e134);
    }
    private static Match e134(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e134M : TinyExpressionP4Parser::e134C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3861:3865:body/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e134C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e134M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e135(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e135M : TinyExpressionP4Parser::e135C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3868:3891:body/1/group", false, false, "text", false, K347);
    }
    private static Match e135C(Session s, Frame f) {
        return e136(s, f);
    }
    private static Match e135M(Session s, Frame f) {
        return e136(s, f);
    }
    private static Match e136(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e136M : TinyExpressionP4Parser::e136C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3870:3889:body/1/0/choice", false, false, "text", false, K347);
    }
    private static Match e136C(Session s, Frame f) {
        return s.choice(f,K348,false,null,false);
    }
    private static Match e136M(Session s, Frame f) {
        return s.choice(f,K348,false,null,false);
    }
    private static Match e137(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e137M : TinyExpressionP4Parser::e137C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3870:3878:body/1/0/0/literal", false, true, "text", false, K51);
    }
    private static Match e137C(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e137M(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e138(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e138M : TinyExpressionP4Parser::e138C, K215, K215, K215, K216, "TinyExpressionP4::StringTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3881:3889:body/1/0/1/literal", false, true, "text", false, K331);
    }
    private static Match e138C(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e138M(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e139(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e139M : TinyExpressionP4Parser::e139C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3916:3950:body/seq", false, false, "text", false, K4);
    }
    private static Match e139C(Session s, Frame f) {
        return s.sequence(f, K349, K249);
    }
    private static Match e139M(Session s, Frame f) {
        return s.sequence(f, K349, K249);
    }
    private static Match e140(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e140M : TinyExpressionP4Parser::e140C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3916:3924:body/0/optional", true, false, "text", false, K256);
    }
    private static Match e140C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e141);
    }
    private static Match e140M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e141);
    }
    private static Match e141(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e141M : TinyExpressionP4Parser::e141C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3918:3922:body/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e141C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e141M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e142(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e142M : TinyExpressionP4Parser::e142C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3925:3950:body/1/group", false, false, "text", false, K350);
    }
    private static Match e142C(Session s, Frame f) {
        return e143(s, f);
    }
    private static Match e142M(Session s, Frame f) {
        return e143(s, f);
    }
    private static Match e143(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e143M : TinyExpressionP4Parser::e143C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3927:3948:body/1/0/choice", false, false, "text", false, K350);
    }
    private static Match e143C(Session s, Frame f) {
        return s.choice(f,K351,false,null,false);
    }
    private static Match e143M(Session s, Frame f) {
        return s.choice(f,K351,false,null,false);
    }
    private static Match e144(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e144M : TinyExpressionP4Parser::e144C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3927:3936:body/1/0/0/literal", false, true, "text", false, K52);
    }
    private static Match e144C(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e144M(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e145(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e145M : TinyExpressionP4Parser::e145C, K215, K215, K215, K216, "TinyExpressionP4::BooleanTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3939:3948:body/1/0/1/literal", false, true, "text", false, K334);
    }
    private static Match e145C(Session s, Frame f) {
        return s.literal(f, "Boolean", true, false, K335);
    }
    private static Match e145M(Session s, Frame f) {
        return s.literal(f, "Boolean", true, false, K335);
    }
    private static Match e146(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e146M : TinyExpressionP4Parser::e146C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3974:4006:body/seq", false, false, "text", false, K4);
    }
    private static Match e146C(Session s, Frame f) {
        return s.sequence(f, K352, K249);
    }
    private static Match e146M(Session s, Frame f) {
        return s.sequence(f, K352, K249);
    }
    private static Match e147(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e147M : TinyExpressionP4Parser::e147C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3974:3982:body/0/optional", true, false, "text", false, K256);
    }
    private static Match e147C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e148);
    }
    private static Match e147M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e148);
    }
    private static Match e148(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e148M : TinyExpressionP4Parser::e148C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3976:3980:body/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e148C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e148M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e149(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e149M : TinyExpressionP4Parser::e149C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3983:4006:body/1/group", false, false, "text", false, K353);
    }
    private static Match e149C(Session s, Frame f) {
        return e150(s, f);
    }
    private static Match e149M(Session s, Frame f) {
        return e150(s, f);
    }
    private static Match e150(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e150M : TinyExpressionP4Parser::e150C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3985:4004:body/1/0/choice", false, false, "text", false, K353);
    }
    private static Match e150C(Session s, Frame f) {
        return s.choice(f,K354,false,null,false);
    }
    private static Match e150M(Session s, Frame f) {
        return s.choice(f,K354,false,null,false);
    }
    private static Match e151(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e151M : TinyExpressionP4Parser::e151C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3985:3993:body/1/0/0/literal", false, true, "text", false, K53);
    }
    private static Match e151C(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e151M(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e152(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e152M : TinyExpressionP4Parser::e152C, K215, K215, K215, K216, "TinyExpressionP4::ObjectTypeHint", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3996:4004:body/1/0/1/literal", false, true, "text", false, K337);
    }
    private static Match e152C(Session s, Frame f) {
        return s.literal(f, "Object", true, false, K338);
    }
    private static Match e152M(Session s, Frame f) {
        return s.literal(f, "Object", true, false, K338);
    }
    private static Match e153(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e153M : TinyExpressionP4Parser::e153C, K215, K215, K215, K216, "TinyExpressionP4::OnlyIfAbsent", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4058:4077:body/seq", false, false, "text", false, K4);
    }
    private static Match e153C(Session s, Frame f) {
        return s.sequence(f, K355, K237);
    }
    private static Match e153M(Session s, Frame f) {
        return s.sequence(f, K355, K237);
    }
    private static Match e154(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e154M : TinyExpressionP4Parser::e154C, K215, K215, K215, K216, "TinyExpressionP4::OnlyIfAbsent", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4058:4062:body/0/literal", false, true, "text", false, K356);
    }
    private static Match e154C(Session s, Frame f) {
        return s.literal(f, "if", true, false, K357);
    }
    private static Match e154M(Session s, Frame f) {
        return s.literal(f, "if", true, false, K357);
    }
    private static Match e155(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e155M : TinyExpressionP4Parser::e155C, K215, K215, K215, K216, "TinyExpressionP4::OnlyIfAbsent", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4063:4068:body/1/literal", false, true, "text", false, K358);
    }
    private static Match e155C(Session s, Frame f) {
        return s.literal(f, "not", true, false, K359);
    }
    private static Match e155M(Session s, Frame f) {
        return s.literal(f, "not", true, false, K359);
    }
    private static Match e156(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e156M : TinyExpressionP4Parser::e156C, K215, K215, K215, K216, "TinyExpressionP4::OnlyIfAbsent", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4069:4077:body/2/literal", false, true, "text", false, K360);
    }
    private static Match e156C(Session s, Frame f) {
        return s.literal(f, "exists", true, false, K361);
    }
    private static Match e156M(Session s, Frame f) {
        return s.literal(f, "exists", true, false, K361);
    }
    private static Match e157(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e157M : TinyExpressionP4Parser::e157C, K215, K215, K215, K216, "TinyExpressionP4::Description", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4099:4123:body/seq", false, false, "text", false, K4);
    }
    private static Match e157C(Session s, Frame f) {
        return s.sequence(f, K362, K237);
    }
    private static Match e157M(Session s, Frame f) {
        return s.sequence(f, K362, K237);
    }
    private static Match e158(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e158M : TinyExpressionP4Parser::e158C, K215, K215, K215, K216, "TinyExpressionP4::Description", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4099:4112:body/0/literal", false, true, "text", false, K363);
    }
    private static Match e158C(Session s, Frame f) {
        return s.literal(f, "description", true, false, K364);
    }
    private static Match e158M(Session s, Frame f) {
        return s.literal(f, "description", true, false, K364);
    }
    private static Match e159(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e159M : TinyExpressionP4Parser::e159C, K215, K215, K215, K216, "TinyExpressionP4::Description", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4113:4116:body/1/literal", false, true, "text", false, K31);
    }
    private static Match e159C(Session s, Frame f) {
        return s.literal(f, "=", true, false, K365);
    }
    private static Match e159M(Session s, Frame f) {
        return s.literal(f, "=", true, false, K365);
    }
    private static Match e160(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e160M : TinyExpressionP4Parser::e160C, K215, K215, K215, K216, "TinyExpressionP4::Description", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4117:4123:body/2/tokenRef", false, true, "text", false, K4);
    }
    private static Match e160C(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::STRING",2);
    }
    private static Match e160M(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::STRING",2);
    }
    private static Match e161(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e161M : TinyExpressionP4Parser::e161C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4144:4191:body/seq", false, false, "node", true, K4);
    }
    private static Match e161C(Session s, Frame f) {
        return s.sequence(f, K368, K367);
    }
    private static Match e161M(Session s, Frame f) {
        return s.sequence(f, K368, K367);
    }
    private static Match e162(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e162M : TinyExpressionP4Parser::e162C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4144:4147:body/0/literal", false, true, "text", false, K369);
    }
    private static Match e162C(Session s, Frame f) {
        return s.literal(f, "@", true, false, K370);
    }
    private static Match e162M(Session s, Frame f) {
        return s.literal(f, "@", true, false, K370);
    }
    private static Match e163(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e163M : TinyExpressionP4Parser::e163C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4148:4158:body/1/tokenRef", false, true, "text", false, K371);
    }
    private static Match e163C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e163M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e164(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e164M : TinyExpressionP4Parser::e164C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4159:4162:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e164C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e164M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e165(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e165M : TinyExpressionP4Parser::e165C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4163:4187:body/3/optional", true, false, "node", true, K292);
    }
    private static Match e165C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e166);
    }
    private static Match e165M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e166);
    }
    private static Match e166(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e166M : TinyExpressionP4Parser::e166C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4165:4185:body/3/0/ruleRef", false, false, "node", true, K4);
    }
    private static Match e166C(Session s, Frame f) {
        return parseAnnotationParameters_17(s, f);
    }
    private static Match e166M(Session s, Frame f) {
        return parseAnnotationParameters_17(s, f);
    }
    private static Match e167(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e167M : TinyExpressionP4Parser::e167C, K215, K215, K215, K216, "TinyExpressionP4::Annotation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4188:4191:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e167C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e167M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e168(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e168M : TinyExpressionP4Parser::e168C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4221:4268:body/seq", false, false, "node", true, K4);
    }
    private static Match e168C(Session s, Frame f) {
        return s.sequence(f, K376, K249);
    }
    private static Match e168M(Session s, Frame f) {
        return s.sequence(f, K376, K249);
    }
    private static Match e169(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e169M : TinyExpressionP4Parser::e169C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4221:4240:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e169C(Session s, Frame f) {
        return parseAnnotationParameter_18(s, f);
    }
    private static Match e169M(Session s, Frame f) {
        return parseAnnotationParameter_18(s, f);
    }
    private static Match e170(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e170M : TinyExpressionP4Parser::e170C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4241:4268:body/1/repeat", true, false, "node", true, K377);
    }
    private static Match e170C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e171, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e170M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e171, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e171(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e171M : TinyExpressionP4Parser::e171C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4243:4266:body/1/0/seq", false, false, "node", false, K378);
    }
    private static Match e171C(Session s, Frame f) {
        return s.sequence(f, K379, K249);
    }
    private static Match e171M(Session s, Frame f) {
        return s.sequence(f, K379, K249);
    }
    private static Match e172(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e172M : TinyExpressionP4Parser::e172C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4243:4246:body/1/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e172C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e172M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e173(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e173M : TinyExpressionP4Parser::e173C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4247:4266:body/1/0/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e173C(Session s, Frame f) {
        return parseAnnotationParameter_18(s, f);
    }
    private static Match e173M(Session s, Frame f) {
        return parseAnnotationParameter_18(s, f);
    }
    private static Match e174(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e174M : TinyExpressionP4Parser::e174C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4297:4322:body/seq", false, false, "node", false, K4);
    }
    private static Match e174C(Session s, Frame f) {
        return s.sequence(f, K381, K237);
    }
    private static Match e174M(Session s, Frame f) {
        return s.sequence(f, K381, K237);
    }
    private static Match e175(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e175M : TinyExpressionP4Parser::e175C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4297:4307:body/0/tokenRef", false, true, "text", false, K371);
    }
    private static Match e175C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e175M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e176(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e176M : TinyExpressionP4Parser::e176C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4308:4311:body/1/literal", false, true, "text", false, K31);
    }
    private static Match e176C(Session s, Frame f) {
        return s.literal(f, "=", true, false, K365);
    }
    private static Match e176M(Session s, Frame f) {
        return s.literal(f, "=", true, false, K365);
    }
    private static Match e177(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e177M : TinyExpressionP4Parser::e177C, K215, K215, K215, K216, "TinyExpressionP4::AnnotationParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4312:4322:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e177C(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e177M(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e178(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e178M : TinyExpressionP4Parser::e178C, K215, K215, K215, K216, "TinyExpressionP4::MethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4356:4470:body/choice", false, false, "node", false, K4);
    }
    private static Match e178C(Session s, Frame f) {
        return s.choice(f,K382,false,null,false);
    }
    private static Match e178M(Session s, Frame f) {
        return s.choice(f,K382,false,null,false);
    }
    private static Match e179(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e179M : TinyExpressionP4Parser::e179C, K215, K215, K215, K216, "TinyExpressionP4::MethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4356:4379:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e179C(Session s, Frame f) {
        return parseNumberMethodDeclaration_20(s, f);
    }
    private static Match e179M(Session s, Frame f) {
        return parseNumberMethodDeclaration_20(s, f);
    }
    private static Match e180(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e180M : TinyExpressionP4Parser::e180C, K215, K215, K215, K216, "TinyExpressionP4::MethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4386:4409:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e180C(Session s, Frame f) {
        return parseStringMethodDeclaration_21(s, f);
    }
    private static Match e180M(Session s, Frame f) {
        return parseStringMethodDeclaration_21(s, f);
    }
    private static Match e181(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e181M : TinyExpressionP4Parser::e181C, K215, K215, K215, K216, "TinyExpressionP4::MethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4416:4440:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e181C(Session s, Frame f) {
        return parseBooleanMethodDeclaration_22(s, f);
    }
    private static Match e181M(Session s, Frame f) {
        return parseBooleanMethodDeclaration_22(s, f);
    }
    private static Match e182(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e182M : TinyExpressionP4Parser::e182C, K215, K215, K215, K216, "TinyExpressionP4::MethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4447:4470:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e182C(Session s, Frame f) {
        return parseObjectMethodDeclaration_23(s, f);
    }
    private static Match e182M(Session s, Frame f) {
        return parseObjectMethodDeclaration_23(s, f);
    }
    private static Match e183(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e183M : TinyExpressionP4Parser::e183C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4651:4796:body/seq", false, false, "node", true, K4);
    }
    private static Match e183C(Session s, Frame f) {
        return s.sequence(f, K385, K384);
    }
    private static Match e183M(Session s, Frame f) {
        return s.sequence(f, K385, K384);
    }
    private static Match e184(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e184M : TinyExpressionP4Parser::e184C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4651:4667:body/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e184C(Session s, Frame f) {
        return parseNumberReturnType_26(s, f);
    }
    private static Match e184M(Session s, Frame f) {
        return parseNumberReturnType_26(s, f);
    }
    private static Match e185(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e185M : TinyExpressionP4Parser::e185C, K386, K387, K224, K225, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4668:4678:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e185C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e185M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e186(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e186M : TinyExpressionP4Parser::e186C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4695:4698:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e186C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e186M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e187(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e187M : TinyExpressionP4Parser::e187C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4705:4737:body/3/optional", true, false, "node", false, K292);
    }
    private static Match e187C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e188);
    }
    private static Match e187M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e188);
    }
    private static Match e188(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e188M : TinyExpressionP4Parser::e188C, K388, K389, K224, K225, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4707:4723:body/3/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e188C(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e188M(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e189(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e189M : TinyExpressionP4Parser::e189C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4742:4745:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e189C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e189M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e190(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e190M : TinyExpressionP4Parser::e190C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4750:4753:body/5/literal", false, true, "text", false, K390);
    }
    private static Match e190C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e190M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e191(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e191M : TinyExpressionP4Parser::e191C, K392, K230, K224, K225, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4760:4776:body/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e191C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e191M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e192(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e192M : TinyExpressionP4Parser::e192C, K215, K215, K215, K216, "TinyExpressionP4::NumberMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4793:4796:body/7/literal", false, true, "text", false, K393);
    }
    private static Match e192C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e192M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e193(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e193M : TinyExpressionP4Parser::e193C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4977:5122:body/seq", false, false, "node", true, K4);
    }
    private static Match e193C(Session s, Frame f) {
        return s.sequence(f, K395, K384);
    }
    private static Match e193M(Session s, Frame f) {
        return s.sequence(f, K395, K384);
    }
    private static Match e194(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e194M : TinyExpressionP4Parser::e194C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4977:4993:body/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e194C(Session s, Frame f) {
        return parseStringReturnType_27(s, f);
    }
    private static Match e194M(Session s, Frame f) {
        return parseStringReturnType_27(s, f);
    }
    private static Match e195(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e195M : TinyExpressionP4Parser::e195C, K396, K387, K224, K225, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4994:5004:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e195C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e195M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e196(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e196M : TinyExpressionP4Parser::e196C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5021:5024:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e196C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e196M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e197(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e197M : TinyExpressionP4Parser::e197C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5031:5063:body/3/optional", true, false, "node", false, K292);
    }
    private static Match e197C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e198);
    }
    private static Match e197M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e198);
    }
    private static Match e198(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e198M : TinyExpressionP4Parser::e198C, K397, K389, K224, K225, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5033:5049:body/3/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e198C(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e198M(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e199(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e199M : TinyExpressionP4Parser::e199C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5068:5071:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e199C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e199M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e200(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e200M : TinyExpressionP4Parser::e200C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5076:5079:body/5/literal", false, true, "text", false, K390);
    }
    private static Match e200C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e200M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e201(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e201M : TinyExpressionP4Parser::e201C, K398, K230, K224, K225, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5086:5102:body/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e201C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e201M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e202(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e202M : TinyExpressionP4Parser::e202C, K215, K215, K215, K216, "TinyExpressionP4::StringMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5119:5122:body/7/literal", false, true, "text", false, K393);
    }
    private static Match e202C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e202M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e203(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e203M : TinyExpressionP4Parser::e203C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5305:5452:body/seq", false, false, "node", true, K4);
    }
    private static Match e203C(Session s, Frame f) {
        return s.sequence(f, K399, K384);
    }
    private static Match e203M(Session s, Frame f) {
        return s.sequence(f, K399, K384);
    }
    private static Match e204(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e204M : TinyExpressionP4Parser::e204C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5305:5322:body/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e204C(Session s, Frame f) {
        return parseBooleanReturnType_28(s, f);
    }
    private static Match e204M(Session s, Frame f) {
        return parseBooleanReturnType_28(s, f);
    }
    private static Match e205(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e205M : TinyExpressionP4Parser::e205C, K400, K387, K224, K225, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5323:5333:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e205C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e205M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e206(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e206M : TinyExpressionP4Parser::e206C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5350:5353:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e206C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e206M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e207(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e207M : TinyExpressionP4Parser::e207C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5360:5392:body/3/optional", true, false, "node", false, K292);
    }
    private static Match e207C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e208);
    }
    private static Match e207M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e208);
    }
    private static Match e208(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e208M : TinyExpressionP4Parser::e208C, K401, K389, K224, K225, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5362:5378:body/3/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e208C(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e208M(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e209(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e209M : TinyExpressionP4Parser::e209C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5397:5400:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e209C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e209M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e210(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e210M : TinyExpressionP4Parser::e210C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5405:5408:body/5/literal", false, true, "text", false, K390);
    }
    private static Match e210C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e210M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e211(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e211M : TinyExpressionP4Parser::e211C, K402, K230, K224, K225, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5415:5432:body/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e211C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e211M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e212(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e212M : TinyExpressionP4Parser::e212C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5449:5452:body/7/literal", false, true, "text", false, K393);
    }
    private static Match e212C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e212M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e213(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e213M : TinyExpressionP4Parser::e213C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5633:5778:body/seq", false, false, "node", true, K4);
    }
    private static Match e213C(Session s, Frame f) {
        return s.sequence(f, K403, K384);
    }
    private static Match e213M(Session s, Frame f) {
        return s.sequence(f, K403, K384);
    }
    private static Match e214(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e214M : TinyExpressionP4Parser::e214C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5633:5649:body/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e214C(Session s, Frame f) {
        return parseObjectReturnType_29(s, f);
    }
    private static Match e214M(Session s, Frame f) {
        return parseObjectReturnType_29(s, f);
    }
    private static Match e215(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e215M : TinyExpressionP4Parser::e215C, K404, K387, K224, K225, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5650:5660:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e215C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e215M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e216(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e216M : TinyExpressionP4Parser::e216C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5677:5680:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e216C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e216M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e217(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e217M : TinyExpressionP4Parser::e217C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5687:5719:body/3/optional", true, false, "node", false, K292);
    }
    private static Match e217C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e218);
    }
    private static Match e217M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e218);
    }
    private static Match e218(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e218M : TinyExpressionP4Parser::e218C, K405, K389, K224, K225, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5689:5705:body/3/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e218C(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e218M(Session s, Frame f) {
        return parseMethodParameters_24(s, f);
    }
    private static Match e219(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e219M : TinyExpressionP4Parser::e219C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5724:5727:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e219C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e219M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e220(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e220M : TinyExpressionP4Parser::e220C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5732:5735:body/5/literal", false, true, "text", false, K390);
    }
    private static Match e220C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e220M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e221(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e221M : TinyExpressionP4Parser::e221C, K406, K230, K224, K225, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5742:5758:body/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e221C(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e221M(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e222(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e222M : TinyExpressionP4Parser::e222C, K215, K215, K215, K216, "TinyExpressionP4::ObjectMethodDeclaration", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5775:5778:body/7/literal", false, true, "text", false, K393);
    }
    private static Match e222C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e222M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e223(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e223M : TinyExpressionP4Parser::e223C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5910:body/seq", false, false, "node", true, K4);
    }
    private static Match e223C(Session s, Frame f) {
        return s.sequence(f, K407, K249);
    }
    private static Match e223M(Session s, Frame f) {
        return s.sequence(f, K407, K249);
    }
    private static Match e224(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e224M : TinyExpressionP4Parser::e224C, K408, K409, K224, K225, "TinyExpressionP4::MethodParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5870:body/0/ruleRef", false, false, "node", false, K49);
    }
    private static Match e224C(Session s, Frame f) {
        return parseMethodParameter_25(s, f);
    }
    private static Match e224M(Session s, Frame f) {
        return parseMethodParameter_25(s, f);
    }
    private static Match e225(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e225M : TinyExpressionP4Parser::e225C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5879:5910:body/1/repeat", true, false, "node", true, K377);
    }
    private static Match e225C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e226, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e225M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e226, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e226(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e226M : TinyExpressionP4Parser::e226C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5881:5900:body/1/0/seq", false, false, "node", false, K377);
    }
    private static Match e226C(Session s, Frame f) {
        return s.sequence(f, K410, K249);
    }
    private static Match e226M(Session s, Frame f) {
        return s.sequence(f, K410, K249);
    }
    private static Match e227(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e227M : TinyExpressionP4Parser::e227C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5881:5884:body/1/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e227C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e227M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e228(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e228M : TinyExpressionP4Parser::e228C, K411, K409, K224, K225, "TinyExpressionP4::MethodParameters", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5885:5900:body/1/0/1/ruleRef", false, false, "node", false, K49);
    }
    private static Match e228C(Session s, Frame f) {
        return parseMethodParameter_25(s, f);
    }
    private static Match e228M(Session s, Frame f) {
        return parseMethodParameter_25(s, f);
    }
    private static Match e229(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e229M : TinyExpressionP4Parser::e229C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6024:6075:body/seq", false, false, "text", false, K4);
    }
    private static Match e229C(Session s, Frame f) {
        return s.sequence(f, K412, K237);
    }
    private static Match e229M(Session s, Frame f) {
        return s.sequence(f, K412, K237);
    }
    private static Match e230(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e230M : TinyExpressionP4Parser::e230C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6024:6027:body/0/literal", false, true, "text", false, K49);
    }
    private static Match e230C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e230M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e231(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e231M : TinyExpressionP4Parser::e231C, K413, K414, K224, K225, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6028:6038:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e231C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e231M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e232(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e232M : TinyExpressionP4Parser::e232C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6050:6075:body/2/optional", true, false, "text", false, K256);
    }
    private static Match e232C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e233);
    }
    private static Match e232M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e233);
    }
    private static Match e233(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e233M : TinyExpressionP4Parser::e233C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6052:6067:body/2/0/seq", false, false, "text", false, K256);
    }
    private static Match e233C(Session s, Frame f) {
        return s.sequence(f, K415, K249);
    }
    private static Match e233M(Session s, Frame f) {
        return s.sequence(f, K415, K249);
    }
    private static Match e234(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e234M : TinyExpressionP4Parser::e234C, K215, K215, K215, K216, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6052:6056:body/2/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e234C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e234M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e235(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e235M : TinyExpressionP4Parser::e235C, K416, K417, K224, K225, "TinyExpressionP4::MethodParameter", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6057:6067:body/2/0/1/ruleRef", false, false, "text", false, K228);
    }
    private static Match e235C(Session s, Frame f) {
        return parseReturnType_30(s, f);
    }
    private static Match e235M(Session s, Frame f) {
        return parseReturnType_30(s, f);
    }
    private static Match e236(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e236M : TinyExpressionP4Parser::e236C, K215, K215, K215, K216, "TinyExpressionP4::NumberReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6102:6120:body/choice", false, false, "text", false, K4);
    }
    private static Match e236C(Session s, Frame f) {
        return s.choice(f,K418,false,null,false);
    }
    private static Match e236M(Session s, Frame f) {
        return s.choice(f,K418,false,null,false);
    }
    private static Match e237(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e237M : TinyExpressionP4Parser::e237C, K215, K215, K215, K216, "TinyExpressionP4::NumberReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6102:6110:body/0/literal", false, true, "text", false, K326);
    }
    private static Match e237C(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e237M(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e238(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e238M : TinyExpressionP4Parser::e238C, K215, K215, K215, K216, "TinyExpressionP4::NumberReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6113:6120:body/1/literal", false, true, "text", false, K342);
    }
    private static Match e238C(Session s, Frame f) {
        return s.literal(f, "float", true, false, K343);
    }
    private static Match e238M(Session s, Frame f) {
        return s.literal(f, "float", true, false, K343);
    }
    private static Match e239(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e239M : TinyExpressionP4Parser::e239C, K215, K215, K215, K216, "TinyExpressionP4::StringReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6146:6154:body/seq", false, false, "text", false, K4);
    }
    private static Match e239C(Session s, Frame f) {
        return s.sequence(f, K421, K420);
    }
    private static Match e239M(Session s, Frame f) {
        return s.sequence(f, K421, K420);
    }
    private static Match e240(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e240M : TinyExpressionP4Parser::e240C, K215, K215, K215, K216, "TinyExpressionP4::StringReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6146:6154:body/0/literal", false, true, "text", false, K51);
    }
    private static Match e240C(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e240M(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e241(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e241M : TinyExpressionP4Parser::e241C, K215, K215, K215, K216, "TinyExpressionP4::BooleanReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6181:6190:body/seq", false, false, "text", false, K4);
    }
    private static Match e241C(Session s, Frame f) {
        return s.sequence(f, K422, K420);
    }
    private static Match e241M(Session s, Frame f) {
        return s.sequence(f, K422, K420);
    }
    private static Match e242(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e242M : TinyExpressionP4Parser::e242C, K215, K215, K215, K216, "TinyExpressionP4::BooleanReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6181:6190:body/0/literal", false, true, "text", false, K52);
    }
    private static Match e242C(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e242M(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e243(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e243M : TinyExpressionP4Parser::e243C, K215, K215, K215, K216, "TinyExpressionP4::ObjectReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6216:6224:body/seq", false, false, "text", false, K4);
    }
    private static Match e243C(Session s, Frame f) {
        return s.sequence(f, K423, K420);
    }
    private static Match e243M(Session s, Frame f) {
        return s.sequence(f, K423, K420);
    }
    private static Match e244(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e244M : TinyExpressionP4Parser::e244C, K215, K215, K215, K216, "TinyExpressionP4::ObjectReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6216:6224:body/0/literal", false, true, "text", false, K53);
    }
    private static Match e244C(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e244M(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e245(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e245M : TinyExpressionP4Parser::e245C, K215, K215, K215, K216, "TinyExpressionP4::ReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6244:6318:body/choice", false, false, "text", false, K4);
    }
    private static Match e245C(Session s, Frame f) {
        return s.choice(f,K440,false,null,false,K439);
    }
    private static Match e245M(Session s, Frame f) {
        return s.choice(f,K440,false,null,false,K439);
    }
    private static Match e246(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e246M : TinyExpressionP4Parser::e246C, K215, K215, K215, K216, "TinyExpressionP4::ReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6244:6260:body/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e246C(Session s, Frame f) {
        return parseNumberReturnType_26(s, f);
    }
    private static Match e246M(Session s, Frame f) {
        return parseNumberReturnType_26(s, f);
    }
    private static Match e247(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e247M : TinyExpressionP4Parser::e247C, K215, K215, K215, K216, "TinyExpressionP4::ReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6263:6279:body/1/ruleRef", false, false, "text", false, K4);
    }
    private static Match e247C(Session s, Frame f) {
        return parseStringReturnType_27(s, f);
    }
    private static Match e247M(Session s, Frame f) {
        return parseStringReturnType_27(s, f);
    }
    private static Match e248(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e248M : TinyExpressionP4Parser::e248C, K215, K215, K215, K216, "TinyExpressionP4::ReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6282:6299:body/2/ruleRef", false, false, "text", false, K4);
    }
    private static Match e248C(Session s, Frame f) {
        return parseBooleanReturnType_28(s, f);
    }
    private static Match e248M(Session s, Frame f) {
        return parseBooleanReturnType_28(s, f);
    }
    private static Match e249(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e249M : TinyExpressionP4Parser::e249C, K215, K215, K215, K216, "TinyExpressionP4::ReturnType", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6302:6318:body/3/ruleRef", false, false, "text", false, K4);
    }
    private static Match e249C(Session s, Frame f) {
        return parseObjectReturnType_29(s, f);
    }
    private static Match e249M(Session s, Frame f) {
        return parseObjectReturnType_29(s, f);
    }
    private static Match e250(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e250M : TinyExpressionP4Parser::e250C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6644:6810:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e250C(Session s, Frame f) {
        return s.sequence(f, K441, K384);
    }
    private static Match e250M(Session s, Frame f) {
        return s.sequence(f, K441, K384);
    }
    private static Match e251(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e251M : TinyExpressionP4Parser::e251C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6644:6654:body/0/literal", false, true, "text", false, K442);
    }
    private static Match e251C(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e251M(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e252(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e252M : TinyExpressionP4Parser::e252C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6655:6679:body/1/optional", true, false, "text", false, K444);
    }
    private static Match e252C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e253);
    }
    private static Match e252M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e253);
    }
    private static Match e253(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e253M : TinyExpressionP4Parser::e253C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6657:6677:body/1/0/seq", false, false, "text", false, K445);
    }
    private static Match e253C(Session s, Frame f) {
        return s.sequence(f, K446, K249);
    }
    private static Match e253M(Session s, Frame f) {
        return s.sequence(f, K446, K249);
    }
    private static Match e254(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e254M : TinyExpressionP4Parser::e254C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6657:6668:body/1/0/0/literal", false, true, "text", false, K444);
    }
    private static Match e254C(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e254M(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e255(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e255M : TinyExpressionP4Parser::e255C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6669:6677:body/1/0/1/optional", true, false, "text", false, K256);
    }
    private static Match e255C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e256);
    }
    private static Match e255M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e256);
    }
    private static Match e256(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e256M : TinyExpressionP4Parser::e256C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6671:6675:body/1/0/1/0/literal", false, true, "text", false, K256);
    }
    private static Match e256C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e256M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e257(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e257M : TinyExpressionP4Parser::e257C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6680:6697:body/2/ruleRef", false, false, "text", false, K4);
    }
    private static Match e257C(Session s, Frame f) {
        return parseBooleanReturnType_28(s, f);
    }
    private static Match e257M(Session s, Frame f) {
        return parseBooleanReturnType_28(s, f);
    }
    private static Match e258(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e258M : TinyExpressionP4Parser::e258C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6702:6709:body/3/optional", true, false, "text", false, K448);
    }
    private static Match e258C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e259);
    }
    private static Match e258M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e259);
    }
    private static Match e259(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e259M : TinyExpressionP4Parser::e259C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6704:6707:body/3/0/literal", false, true, "text", false, K448);
    }
    private static Match e259C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e259M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e260(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e260M : TinyExpressionP4Parser::e260C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6714:6778:body/4/group", false, false, "mixed", false, K247);
    }
    private static Match e260C(Session s, Frame f) {
        return e261(s, f);
    }
    private static Match e260M(Session s, Frame f) {
        return e261(s, f);
    }
    private static Match e261(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e261M : TinyExpressionP4Parser::e261C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6770:body/4/0/choice", false, false, "mixed", false, K247);
    }
    private static Match e261C(Session s, Frame f) {
        return s.choice(f,K450,false,null,false);
    }
    private static Match e261M(Session s, Frame f) {
        return s.choice(f,K450,false,null,false);
    }
    private static Match e262(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e262M : TinyExpressionP4Parser::e262C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6751:body/4/0/0/seq", false, false, "node", false, K247);
    }
    private static Match e262C(Session s, Frame f) {
        return s.sequence(f, K451, K237);
    }
    private static Match e262M(Session s, Frame f) {
        return s.sequence(f, K451, K237);
    }
    private static Match e263(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e263M : TinyExpressionP4Parser::e263C, K452, K246, K224, K225, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6725:body/4/0/0/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e263C(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e263M(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e264(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e264M : TinyExpressionP4Parser::e264C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6737:6740:body/4/0/0/1/literal", false, true, "text", false, K247);
    }
    private static Match e264C(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e264M(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e265(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e265M : TinyExpressionP4Parser::e265C, K453, K454, K224, K225, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6741:6751:body/4/0/0/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e265C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e265M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e266(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e266M : TinyExpressionP4Parser::e266C, K455, K454, K224, K225, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6760:6770:body/4/0/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e266C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e266M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e267(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e267M : TinyExpressionP4Parser::e267C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6783:6786:body/5/literal", false, true, "text", false, K372);
    }
    private static Match e267C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e267M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e268(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e268M : TinyExpressionP4Parser::e268C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6787:6806:body/6/optional", true, false, "node", false, K292);
    }
    private static Match e268C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e269);
    }
    private static Match e268M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e269);
    }
    private static Match e269(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e269M : TinyExpressionP4Parser::e269C, K456, K457, K224, K225, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6789:6798:body/6/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e269C(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e269M(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e270(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e270M : TinyExpressionP4Parser::e270C, K215, K215, K215, K216, "TinyExpressionP4::ExternalBooleanInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6807:6810:body/7/literal", false, true, "text", false, K374);
    }
    private static Match e270C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e270M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e271(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e271M : TinyExpressionP4Parser::e271C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6922:7101:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e271C(Session s, Frame f) {
        return s.sequence(f, K458, K241);
    }
    private static Match e271M(Session s, Frame f) {
        return s.sequence(f, K458, K241);
    }
    private static Match e272(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e272M : TinyExpressionP4Parser::e272C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6922:6932:body/0/literal", false, true, "text", false, K442);
    }
    private static Match e272C(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e272M(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e273(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e273M : TinyExpressionP4Parser::e273C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6937:7000:body/1/group", false, false, "text", false, K448);
    }
    private static Match e273C(Session s, Frame f) {
        return e274(s, f);
    }
    private static Match e273M(Session s, Frame f) {
        return e274(s, f);
    }
    private static Match e274(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e274M : TinyExpressionP4Parser::e274C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6939:6998:body/1/0/choice", false, false, "text", false, K448);
    }
    private static Match e274C(Session s, Frame f) {
        return s.choice(f,K459,false,null,false);
    }
    private static Match e274M(Session s, Frame f) {
        return s.choice(f,K459,false,null,false);
    }
    private static Match e275(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e275M : TinyExpressionP4Parser::e275C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6939:6988:body/1/0/0/seq", false, false, "text", false, K460);
    }
    private static Match e275C(Session s, Frame f) {
        return s.sequence(f, K461, K237);
    }
    private static Match e275M(Session s, Frame f) {
        return s.sequence(f, K461, K237);
    }
    private static Match e276(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e276M : TinyExpressionP4Parser::e276C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6939:6963:body/1/0/0/0/optional", true, false, "text", false, K444);
    }
    private static Match e276C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e277);
    }
    private static Match e276M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e277);
    }
    private static Match e277(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e277M : TinyExpressionP4Parser::e277C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6941:6961:body/1/0/0/0/0/seq", false, false, "text", false, K445);
    }
    private static Match e277C(Session s, Frame f) {
        return s.sequence(f, K462, K249);
    }
    private static Match e277M(Session s, Frame f) {
        return s.sequence(f, K462, K249);
    }
    private static Match e278(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e278M : TinyExpressionP4Parser::e278C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6941:6952:body/1/0/0/0/0/0/literal", false, true, "text", false, K444);
    }
    private static Match e278C(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e278M(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e279(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e279M : TinyExpressionP4Parser::e279C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6953:6961:body/1/0/0/0/0/1/optional", true, false, "text", false, K256);
    }
    private static Match e279C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e280);
    }
    private static Match e279M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e280);
    }
    private static Match e280(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e280M : TinyExpressionP4Parser::e280C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6955:6959:body/1/0/0/0/0/1/0/literal", false, true, "text", false, K256);
    }
    private static Match e280C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e280M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e281(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e281M : TinyExpressionP4Parser::e281C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6964:6980:body/1/0/0/1/ruleRef", false, false, "text", false, K4);
    }
    private static Match e281C(Session s, Frame f) {
        return parseNumberReturnType_26(s, f);
    }
    private static Match e281M(Session s, Frame f) {
        return parseNumberReturnType_26(s, f);
    }
    private static Match e282(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e282M : TinyExpressionP4Parser::e282C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6981:6988:body/1/0/0/2/optional", true, false, "text", false, K448);
    }
    private static Match e282C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e283);
    }
    private static Match e282M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e283);
    }
    private static Match e283(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e283M : TinyExpressionP4Parser::e283C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6983:6986:body/1/0/0/2/0/literal", false, true, "text", false, K448);
    }
    private static Match e283C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e283M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e284(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e284M : TinyExpressionP4Parser::e284C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6991:6998:body/1/0/1/optional", true, false, "text", false, K448);
    }
    private static Match e284C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e285);
    }
    private static Match e284M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e285);
    }
    private static Match e285(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e285M : TinyExpressionP4Parser::e285C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6993:6996:body/1/0/1/0/literal", false, true, "text", false, K448);
    }
    private static Match e285C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e285M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e286(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e286M : TinyExpressionP4Parser::e286C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7005:7069:body/2/group", false, false, "mixed", false, K247);
    }
    private static Match e286C(Session s, Frame f) {
        return e287(s, f);
    }
    private static Match e286M(Session s, Frame f) {
        return e287(s, f);
    }
    private static Match e287(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e287M : TinyExpressionP4Parser::e287C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7061:body/2/0/choice", false, false, "mixed", false, K247);
    }
    private static Match e287C(Session s, Frame f) {
        return s.choice(f,K463,false,null,false);
    }
    private static Match e287M(Session s, Frame f) {
        return s.choice(f,K463,false,null,false);
    }
    private static Match e288(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e288M : TinyExpressionP4Parser::e288C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7042:body/2/0/0/seq", false, false, "node", false, K247);
    }
    private static Match e288C(Session s, Frame f) {
        return s.sequence(f, K464, K237);
    }
    private static Match e288M(Session s, Frame f) {
        return s.sequence(f, K464, K237);
    }
    private static Match e289(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e289M : TinyExpressionP4Parser::e289C, K465, K246, K224, K225, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7016:body/2/0/0/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e289C(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e289M(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e290(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e290M : TinyExpressionP4Parser::e290C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7028:7031:body/2/0/0/1/literal", false, true, "text", false, K247);
    }
    private static Match e290C(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e290M(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e291(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e291M : TinyExpressionP4Parser::e291C, K466, K454, K224, K225, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7032:7042:body/2/0/0/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e291C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e291M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e292(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e292M : TinyExpressionP4Parser::e292C, K467, K454, K224, K225, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7051:7061:body/2/0/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e292C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e292M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e293(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e293M : TinyExpressionP4Parser::e293C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7074:7077:body/3/literal", false, true, "text", false, K372);
    }
    private static Match e293C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e293M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e294(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e294M : TinyExpressionP4Parser::e294C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7078:7097:body/4/optional", true, false, "node", false, K292);
    }
    private static Match e294C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e295);
    }
    private static Match e294M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e295);
    }
    private static Match e295(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e295M : TinyExpressionP4Parser::e295C, K468, K457, K224, K225, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7080:7089:body/4/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e295C(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e295M(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e296(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e296M : TinyExpressionP4Parser::e296C, K215, K215, K215, K216, "TinyExpressionP4::ExternalNumberInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7098:7101:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e296C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e296M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e297(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e297M : TinyExpressionP4Parser::e297C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7213:7378:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e297C(Session s, Frame f) {
        return s.sequence(f, K469, K384);
    }
    private static Match e297M(Session s, Frame f) {
        return s.sequence(f, K469, K384);
    }
    private static Match e298(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e298M : TinyExpressionP4Parser::e298C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7213:7223:body/0/literal", false, true, "text", false, K442);
    }
    private static Match e298C(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e298M(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e299(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e299M : TinyExpressionP4Parser::e299C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7224:7248:body/1/optional", true, false, "text", false, K444);
    }
    private static Match e299C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e300);
    }
    private static Match e299M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e300);
    }
    private static Match e300(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e300M : TinyExpressionP4Parser::e300C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7226:7246:body/1/0/seq", false, false, "text", false, K445);
    }
    private static Match e300C(Session s, Frame f) {
        return s.sequence(f, K470, K249);
    }
    private static Match e300M(Session s, Frame f) {
        return s.sequence(f, K470, K249);
    }
    private static Match e301(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e301M : TinyExpressionP4Parser::e301C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7226:7237:body/1/0/0/literal", false, true, "text", false, K444);
    }
    private static Match e301C(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e301M(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e302(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e302M : TinyExpressionP4Parser::e302C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7238:7246:body/1/0/1/optional", true, false, "text", false, K256);
    }
    private static Match e302C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e303);
    }
    private static Match e302M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e303);
    }
    private static Match e303(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e303M : TinyExpressionP4Parser::e303C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7240:7244:body/1/0/1/0/literal", false, true, "text", false, K256);
    }
    private static Match e303C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e303M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e304(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e304M : TinyExpressionP4Parser::e304C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7249:7265:body/2/ruleRef", false, false, "text", false, K4);
    }
    private static Match e304C(Session s, Frame f) {
        return parseStringReturnType_27(s, f);
    }
    private static Match e304M(Session s, Frame f) {
        return parseStringReturnType_27(s, f);
    }
    private static Match e305(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e305M : TinyExpressionP4Parser::e305C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7270:7277:body/3/optional", true, false, "text", false, K448);
    }
    private static Match e305C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e306);
    }
    private static Match e305M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e306);
    }
    private static Match e306(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e306M : TinyExpressionP4Parser::e306C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7272:7275:body/3/0/literal", false, true, "text", false, K448);
    }
    private static Match e306C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e306M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e307(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e307M : TinyExpressionP4Parser::e307C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7282:7346:body/4/group", false, false, "mixed", false, K247);
    }
    private static Match e307C(Session s, Frame f) {
        return e308(s, f);
    }
    private static Match e307M(Session s, Frame f) {
        return e308(s, f);
    }
    private static Match e308(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e308M : TinyExpressionP4Parser::e308C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7338:body/4/0/choice", false, false, "mixed", false, K247);
    }
    private static Match e308C(Session s, Frame f) {
        return s.choice(f,K471,false,null,false);
    }
    private static Match e308M(Session s, Frame f) {
        return s.choice(f,K471,false,null,false);
    }
    private static Match e309(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e309M : TinyExpressionP4Parser::e309C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7319:body/4/0/0/seq", false, false, "node", false, K247);
    }
    private static Match e309C(Session s, Frame f) {
        return s.sequence(f, K472, K237);
    }
    private static Match e309M(Session s, Frame f) {
        return s.sequence(f, K472, K237);
    }
    private static Match e310(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e310M : TinyExpressionP4Parser::e310C, K473, K246, K224, K225, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7293:body/4/0/0/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e310C(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e310M(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e311(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e311M : TinyExpressionP4Parser::e311C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7305:7308:body/4/0/0/1/literal", false, true, "text", false, K247);
    }
    private static Match e311C(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e311M(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e312(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e312M : TinyExpressionP4Parser::e312C, K474, K454, K224, K225, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7309:7319:body/4/0/0/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e312C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e312M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e313(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e313M : TinyExpressionP4Parser::e313C, K475, K454, K224, K225, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7328:7338:body/4/0/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e313C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e313M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e314(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e314M : TinyExpressionP4Parser::e314C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7351:7354:body/5/literal", false, true, "text", false, K372);
    }
    private static Match e314C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e314M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e315(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e315M : TinyExpressionP4Parser::e315C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7355:7374:body/6/optional", true, false, "node", false, K292);
    }
    private static Match e315C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e316);
    }
    private static Match e315M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e316);
    }
    private static Match e316(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e316M : TinyExpressionP4Parser::e316C, K476, K457, K224, K225, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7357:7366:body/6/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e316C(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e316M(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e317(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e317M : TinyExpressionP4Parser::e317C, K215, K215, K215, K216, "TinyExpressionP4::ExternalStringInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7375:7378:body/7/literal", false, true, "text", false, K374);
    }
    private static Match e317C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e317M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e318(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e318M : TinyExpressionP4Parser::e318C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7490:7655:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e318C(Session s, Frame f) {
        return s.sequence(f, K477, K384);
    }
    private static Match e318M(Session s, Frame f) {
        return s.sequence(f, K477, K384);
    }
    private static Match e319(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e319M : TinyExpressionP4Parser::e319C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7490:7500:body/0/literal", false, true, "text", false, K442);
    }
    private static Match e319C(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e319M(Session s, Frame f) {
        return s.literal(f, "external", true, false, K443);
    }
    private static Match e320(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e320M : TinyExpressionP4Parser::e320C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7501:7525:body/1/optional", true, false, "text", false, K444);
    }
    private static Match e320C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e321);
    }
    private static Match e320M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e321);
    }
    private static Match e321(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e321M : TinyExpressionP4Parser::e321C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7503:7523:body/1/0/seq", false, false, "text", false, K445);
    }
    private static Match e321C(Session s, Frame f) {
        return s.sequence(f, K478, K249);
    }
    private static Match e321M(Session s, Frame f) {
        return s.sequence(f, K478, K249);
    }
    private static Match e322(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e322M : TinyExpressionP4Parser::e322C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7503:7514:body/1/0/0/literal", false, true, "text", false, K444);
    }
    private static Match e322C(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e322M(Session s, Frame f) {
        return s.literal(f, "returning", true, false, K447);
    }
    private static Match e323(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e323M : TinyExpressionP4Parser::e323C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7515:7523:body/1/0/1/optional", true, false, "text", false, K256);
    }
    private static Match e323C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e324);
    }
    private static Match e323M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e324);
    }
    private static Match e324(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e324M : TinyExpressionP4Parser::e324C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7517:7521:body/1/0/1/0/literal", false, true, "text", false, K256);
    }
    private static Match e324C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e324M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e325(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e325M : TinyExpressionP4Parser::e325C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7526:7542:body/2/ruleRef", false, false, "text", false, K4);
    }
    private static Match e325C(Session s, Frame f) {
        return parseObjectReturnType_29(s, f);
    }
    private static Match e325M(Session s, Frame f) {
        return parseObjectReturnType_29(s, f);
    }
    private static Match e326(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e326M : TinyExpressionP4Parser::e326C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7547:7554:body/3/optional", true, false, "text", false, K448);
    }
    private static Match e326C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e327);
    }
    private static Match e326M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e327);
    }
    private static Match e327(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e327M : TinyExpressionP4Parser::e327C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7549:7552:body/3/0/literal", false, true, "text", false, K448);
    }
    private static Match e327C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e327M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e328(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e328M : TinyExpressionP4Parser::e328C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7559:7623:body/4/group", false, false, "mixed", false, K247);
    }
    private static Match e328C(Session s, Frame f) {
        return e329(s, f);
    }
    private static Match e328M(Session s, Frame f) {
        return e329(s, f);
    }
    private static Match e329(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e329M : TinyExpressionP4Parser::e329C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7615:body/4/0/choice", false, false, "mixed", false, K247);
    }
    private static Match e329C(Session s, Frame f) {
        return s.choice(f,K479,false,null,false);
    }
    private static Match e329M(Session s, Frame f) {
        return s.choice(f,K479,false,null,false);
    }
    private static Match e330(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e330M : TinyExpressionP4Parser::e330C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7596:body/4/0/0/seq", false, false, "node", false, K247);
    }
    private static Match e330C(Session s, Frame f) {
        return s.sequence(f, K480, K237);
    }
    private static Match e330M(Session s, Frame f) {
        return s.sequence(f, K480, K237);
    }
    private static Match e331(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e331M : TinyExpressionP4Parser::e331C, K481, K246, K224, K225, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7570:body/4/0/0/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e331C(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e331M(Session s, Frame f) {
        return parseClassName_3(s, f);
    }
    private static Match e332(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e332M : TinyExpressionP4Parser::e332C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7582:7585:body/4/0/0/1/literal", false, true, "text", false, K247);
    }
    private static Match e332C(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e332M(Session s, Frame f) {
        return s.literal(f, "#", true, false, K251);
    }
    private static Match e333(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e333M : TinyExpressionP4Parser::e333C, K482, K454, K224, K225, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7586:7596:body/4/0/0/2/tokenRef", false, true, "text", false, K254);
    }
    private static Match e333C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e333M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e334(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e334M : TinyExpressionP4Parser::e334C, K483, K454, K224, K225, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7605:7615:body/4/0/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e334C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e334M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e335(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e335M : TinyExpressionP4Parser::e335C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7628:7631:body/5/literal", false, true, "text", false, K372);
    }
    private static Match e335C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e335M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e336(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e336M : TinyExpressionP4Parser::e336C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7632:7651:body/6/optional", true, false, "node", false, K292);
    }
    private static Match e336C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e337);
    }
    private static Match e336M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e337);
    }
    private static Match e337(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e337M : TinyExpressionP4Parser::e337C, K484, K457, K224, K225, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7634:7643:body/6/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e337C(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e337M(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e338(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e338M : TinyExpressionP4Parser::e338C, K215, K215, K215, K216, "TinyExpressionP4::ExternalObjectInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7652:7655:body/7/literal", false, true, "text", false, K374);
    }
    private static Match e338C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e338M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e339(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e339M : TinyExpressionP4Parser::e339C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocationHeader", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7894:body/choice", false, false, "text", false, K4);
    }
    private static Match e339C(Session s, Frame f) {
        return s.choice(f,K489,false,null,false,K488);
    }
    private static Match e339M(Session s, Frame f) {
        return s.choice(f,K489,false,null,false,K488);
    }
    private static Match e340(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e340M : TinyExpressionP4Parser::e340C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocationHeader", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7881:body/0/seq", false, false, "text", false, K63);
    }
    private static Match e340C(Session s, Frame f) {
        return s.sequence(f, K490, K249);
    }
    private static Match e340M(Session s, Frame f) {
        return s.sequence(f, K490, K249);
    }
    private static Match e341(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e341M : TinyExpressionP4Parser::e341C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocationHeader", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7860:7866:body/0/0/literal", false, true, "text", false, K491);
    }
    private static Match e341C(Session s, Frame f) {
        return s.literal(f, "call", true, false, K492);
    }
    private static Match e341M(Session s, Frame f) {
        return s.literal(f, "call", true, false, K492);
    }
    private static Match e342(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e342M : TinyExpressionP4Parser::e342C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocationHeader", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7867:7881:body/0/1/optional", true, false, "text", false, K493);
    }
    private static Match e342C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e343);
    }
    private static Match e342M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e343);
    }
    private static Match e343(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e343M : TinyExpressionP4Parser::e343C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocationHeader", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7869:7879:body/0/1/0/literal", false, true, "text", false, K493);
    }
    private static Match e343C(Session s, Frame f) {
        return s.literal(f, "internal", true, false, K494);
    }
    private static Match e343M(Session s, Frame f) {
        return s.literal(f, "internal", true, false, K494);
    }
    private static Match e344(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e344M : TinyExpressionP4Parser::e344C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocationHeader", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7884:7894:body/1/literal", false, true, "text", false, K493);
    }
    private static Match e344C(Session s, Frame f) {
        return s.literal(f, "internal", true, false, K494);
    }
    private static Match e344M(Session s, Frame f) {
        return s.literal(f, "internal", true, false, K494);
    }
    private static Match e345(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e345M : TinyExpressionP4Parser::e345C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7997:8064:body/seq", false, false, "node", false, K4);
    }
    private static Match e345C(Session s, Frame f) {
        return s.sequence(f, K495, K367);
    }
    private static Match e345M(Session s, Frame f) {
        return s.sequence(f, K495, K367);
    }
    private static Match e346(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e346M : TinyExpressionP4Parser::e346C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7997:8019:body/0/ruleRef", false, false, "text", false, K4);
    }
    private static Match e346C(Session s, Frame f) {
        return parseMethodInvocationHeader_35(s, f);
    }
    private static Match e346M(Session s, Frame f) {
        return parseMethodInvocationHeader_35(s, f);
    }
    private static Match e347(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e347M : TinyExpressionP4Parser::e347C, K496, K454, K224, K225, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8020:8030:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e347C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e347M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e348(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e348M : TinyExpressionP4Parser::e348C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8037:8040:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e348C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e348M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e349(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e349M : TinyExpressionP4Parser::e349C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8041:8060:body/3/optional", true, false, "node", false, K292);
    }
    private static Match e349C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e350);
    }
    private static Match e349M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e350);
    }
    private static Match e350(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e350M : TinyExpressionP4Parser::e350C, K497, K457, K224, K225, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8043:8052:body/3/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e350C(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e350M(Session s, Frame f) {
        return parseArguments_39(s, f);
    }
    private static Match e351(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e351M : TinyExpressionP4Parser::e351C, K215, K215, K215, K216, "TinyExpressionP4::MethodInvocation", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8061:8064:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e351C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e351M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e352(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e352M : TinyExpressionP4Parser::e352C, K215, K215, K215, K216, "TinyExpressionP4::ArgumentTernary", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8311:body/seq", false, false, "node", true, K4);
    }
    private static Match e352C(Session s, Frame f) {
        return s.sequence(f, K498, K367);
    }
    private static Match e352M(Session s, Frame f) {
        return s.sequence(f, K498, K367);
    }
    private static Match e353(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e353M : TinyExpressionP4Parser::e353C, K499, K500, K224, K225, "TinyExpressionP4::ArgumentTernary", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8248:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e353C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e353M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e354(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e354M : TinyExpressionP4Parser::e354C, K215, K215, K215, K216, "TinyExpressionP4::ArgumentTernary", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8260:8263:body/1/literal", false, true, "text", false, K501);
    }
    private static Match e354C(Session s, Frame f) {
        return s.literal(f, "?", true, false, K502);
    }
    private static Match e354M(Session s, Frame f) {
        return s.literal(f, "?", true, false, K502);
    }
    private static Match e355(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e355M : TinyExpressionP4Parser::e355C, K503, K504, K224, K225, "TinyExpressionP4::ArgumentTernary", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8264:8280:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e355C(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e355M(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e356(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e356M : TinyExpressionP4Parser::e356C, K215, K215, K215, K216, "TinyExpressionP4::ArgumentTernary", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8291:8294:body/3/literal", false, true, "text", false, K448);
    }
    private static Match e356C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e356M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e357(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e357M : TinyExpressionP4Parser::e357C, K505, K506, K224, K225, "TinyExpressionP4::ArgumentTernary", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8295:8311:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e357C(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e357M(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e358(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e358M : TinyExpressionP4Parser::e358C, K215, K215, K215, K216, "TinyExpressionP4::ArgumentExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8520:body/choice", false, false, "node", false, K4);
    }
    private static Match e358C(Session s, Frame f) {
        return s.choice(f,K507,false,null,false);
    }
    private static Match e358M(Session s, Frame f) {
        return s.choice(f,K507,false,null,false);
    }
    private static Match e359(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e359M : TinyExpressionP4Parser::e359C, K508, K299, K224, K225, "TinyExpressionP4::ArgumentExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8422:body/0/ruleRef", false, false, "node", false, K68);
    }
    private static Match e359C(Session s, Frame f) {
        return parseArgumentTernary_37(s, f);
    }
    private static Match e359M(Session s, Frame f) {
        return parseArgumentTernary_37(s, f);
    }
    private static Match e360(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e360M : TinyExpressionP4Parser::e360C, K509, K299, K224, K225, "TinyExpressionP4::ArgumentExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8436:8456:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e360C(Session s, Frame f) {
        return parseComparisonExpression_103(s, f);
    }
    private static Match e360M(Session s, Frame f) {
        return parseComparisonExpression_103(s, f);
    }
    private static Match e361(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e361M : TinyExpressionP4Parser::e361C, K510, K299, K224, K225, "TinyExpressionP4::ArgumentExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8470:8496:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e361C(Session s, Frame f) {
        return parseStringComparisonExpression_101(s, f);
    }
    private static Match e361M(Session s, Frame f) {
        return parseStringComparisonExpression_101(s, f);
    }
    private static Match e362(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e362M : TinyExpressionP4Parser::e362C, K511, K299, K224, K225, "TinyExpressionP4::ArgumentExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8510:8520:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e362C(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e362M(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e363(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e363M : TinyExpressionP4Parser::e363C, K215, K215, K215, K216, "TinyExpressionP4::Arguments", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8651:body/seq", false, false, "node", true, K4);
    }
    private static Match e363C(Session s, Frame f) {
        return s.sequence(f, K512, K249);
    }
    private static Match e363M(Session s, Frame f) {
        return s.sequence(f, K512, K249);
    }
    private static Match e364(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e364M : TinyExpressionP4Parser::e364C, K513, K409, K224, K225, "TinyExpressionP4::Arguments", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8608:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e364C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e364M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e365(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e365M : TinyExpressionP4Parser::e365C, K215, K215, K215, K216, "TinyExpressionP4::Arguments", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8617:8651:body/1/repeat", true, false, "node", true, K377);
    }
    private static Match e365C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e366, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e365M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e366, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e366(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e366M : TinyExpressionP4Parser::e366C, K215, K215, K215, K216, "TinyExpressionP4::Arguments", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8619:8641:body/1/0/seq", false, false, "node", false, K377);
    }
    private static Match e366C(Session s, Frame f) {
        return s.sequence(f, K514, K249);
    }
    private static Match e366M(Session s, Frame f) {
        return s.sequence(f, K514, K249);
    }
    private static Match e367(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e367M : TinyExpressionP4Parser::e367C, K215, K215, K215, K216, "TinyExpressionP4::Arguments", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8619:8622:body/1/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e367C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e367M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e368(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e368M : TinyExpressionP4Parser::e368C, K515, K409, K224, K225, "TinyExpressionP4::Arguments", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8623:8641:body/1/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e368C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e368M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e369(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e369M : TinyExpressionP4Parser::e369C, K215, K215, K215, K216, "TinyExpressionP4::NumberExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8894:body/seq", false, false, "node", true, K4);
    }
    private static Match e369C(Session s, Frame f) {
        return s.sequence(f, K516, K249);
    }
    private static Match e369M(Session s, Frame f) {
        return s.sequence(f, K516, K249);
    }
    private static Match e370(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e370M : TinyExpressionP4Parser::e370C, K517, K518, K224, K225, "TinyExpressionP4::NumberExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8856:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e370C(Session s, Frame f) {
        return parseNumberTerm_41(s, f);
    }
    private static Match e370M(Session s, Frame f) {
        return parseNumberTerm_41(s, f);
    }
    private static Match e371(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e371M : TinyExpressionP4Parser::e371C, K215, K215, K215, K216, "TinyExpressionP4::NumberExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8863:8894:body/1/repeat", true, false, "node", true, K221);
    }
    private static Match e371C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e372, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e371M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e372, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e372(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e372M : TinyExpressionP4Parser::e372C, K215, K215, K215, K216, "TinyExpressionP4::NumberExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8885:body/1/0/seq", false, false, "node", false, K519);
    }
    private static Match e372C(Session s, Frame f) {
        return s.sequence(f, K520, K249);
    }
    private static Match e372M(Session s, Frame f) {
        return s.sequence(f, K520, K249);
    }
    private static Match e373(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e373M : TinyExpressionP4Parser::e373C, K521, K522, K224, K225, "TinyExpressionP4::NumberExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8870:body/1/0/0/ruleRef", false, false, "text", false, K77);
    }
    private static Match e373C(Session s, Frame f) {
        return parseAddOp_42(s, f);
    }
    private static Match e373M(Session s, Frame f) {
        return parseAddOp_42(s, f);
    }
    private static Match e374(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e374M : TinyExpressionP4Parser::e374C, K523, K524, K224, K225, "TinyExpressionP4::NumberExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8875:8885:body/1/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e374C(Session s, Frame f) {
        return parseNumberTerm_41(s, f);
    }
    private static Match e374M(Session s, Frame f) {
        return parseNumberTerm_41(s, f);
    }
    private static Match e375(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e375M : TinyExpressionP4Parser::e375C, K215, K215, K215, K216, "TinyExpressionP4::NumberTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9053:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e375C(Session s, Frame f) {
        return s.sequence(f, K525, K249);
    }
    private static Match e375M(Session s, Frame f) {
        return s.sequence(f, K525, K249);
    }
    private static Match e376(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e376M : TinyExpressionP4Parser::e376C, K526, K518, K224, K225, "TinyExpressionP4::NumberTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9013:body/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e376C(Session s, Frame f) {
        return parseNumberFactor_60(s, f);
    }
    private static Match e376M(Session s, Frame f) {
        return parseNumberFactor_60(s, f);
    }
    private static Match e377(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e377M : TinyExpressionP4Parser::e377C, K215, K215, K215, K216, "TinyExpressionP4::NumberTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9020:9053:body/1/repeat", true, false, "mixed", true, K221);
    }
    private static Match e377C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e378, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e377M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e378, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e378(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e378M : TinyExpressionP4Parser::e378C, K215, K215, K215, K216, "TinyExpressionP4::NumberTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9044:body/1/0/seq", false, false, "mixed", false, K527);
    }
    private static Match e378C(Session s, Frame f) {
        return s.sequence(f, K528, K249);
    }
    private static Match e378M(Session s, Frame f) {
        return s.sequence(f, K528, K249);
    }
    private static Match e379(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e379M : TinyExpressionP4Parser::e379C, K529, K522, K224, K225, "TinyExpressionP4::NumberTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9027:body/1/0/0/ruleRef", false, false, "text", false, K78);
    }
    private static Match e379C(Session s, Frame f) {
        return parseMulOp_43(s, f);
    }
    private static Match e379M(Session s, Frame f) {
        return parseMulOp_43(s, f);
    }
    private static Match e380(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e380M : TinyExpressionP4Parser::e380C, K530, K524, K224, K225, "TinyExpressionP4::NumberTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9032:9044:body/1/0/1/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e380C(Session s, Frame f) {
        return parseNumberFactor_60(s, f);
    }
    private static Match e380M(Session s, Frame f) {
        return parseNumberFactor_60(s, f);
    }
    private static Match e381(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e381M : TinyExpressionP4Parser::e381C, K215, K215, K215, K216, "TinyExpressionP4::AddOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9069:9078:body/choice", false, false, "text", false, K4);
    }
    private static Match e381C(Session s, Frame f) {
        return s.choice(f,K531,false,new boolean[]{true,true},false);
    }
    private static Match e381M(Session s, Frame f) {
        return s.choice(f,K531,false,new boolean[]{true,true},false);
    }
    private static Match e382(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e382M : TinyExpressionP4Parser::e382C, K215, K215, K215, K216, "TinyExpressionP4::AddOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9069:9072:body/0/literal", false, true, "text", false, K532);
    }
    private static Match e382C(Session s, Frame f) {
        return s.literal(f, "+", true, false, K533);
    }
    private static Match e382M(Session s, Frame f) {
        return s.literal(f, "+", true, false, K533);
    }
    private static Match e383(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e383M : TinyExpressionP4Parser::e383C, K215, K215, K215, K216, "TinyExpressionP4::AddOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9075:9078:body/1/literal", false, true, "text", false, K534);
    }
    private static Match e383C(Session s, Frame f) {
        return s.literal(f, "-", true, false, K535);
    }
    private static Match e383M(Session s, Frame f) {
        return s.literal(f, "-", true, false, K535);
    }
    private static Match e384(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e384M : TinyExpressionP4Parser::e384C, K215, K215, K215, K216, "TinyExpressionP4::MulOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9093:9102:body/choice", false, false, "text", false, K4);
    }
    private static Match e384C(Session s, Frame f) {
        return s.choice(f,K536,false,new boolean[]{true,true},false);
    }
    private static Match e384M(Session s, Frame f) {
        return s.choice(f,K536,false,new boolean[]{true,true},false);
    }
    private static Match e385(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e385M : TinyExpressionP4Parser::e385C, K215, K215, K215, K216, "TinyExpressionP4::MulOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9093:9096:body/0/literal", false, true, "text", false, K537);
    }
    private static Match e385C(Session s, Frame f) {
        return s.literal(f, "*", true, false, K538);
    }
    private static Match e385M(Session s, Frame f) {
        return s.literal(f, "*", true, false, K538);
    }
    private static Match e386(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e386M : TinyExpressionP4Parser::e386C, K215, K215, K215, K216, "TinyExpressionP4::MulOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9099:9102:body/1/literal", false, true, "text", false, K539);
    }
    private static Match e386C(Session s, Frame f) {
        return s.literal(f, "/", true, false, K540);
    }
    private static Match e386M(Session s, Frame f) {
        return s.literal(f, "/", true, false, K540);
    }
    private static Match e387(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e387M : TinyExpressionP4Parser::e387C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9210:9464:body/choice", false, false, "node", false, K4);
    }
    private static Match e387C(Session s, Frame f) {
        return s.choice(f,K592,false,null,false,K591);
    }
    private static Match e387M(Session s, Frame f) {
        return s.choice(f,K592,false,null,false,K591);
    }
    private static Match e388(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e388M : TinyExpressionP4Parser::e388C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9210:9221:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e388C(Session s, Frame f) {
        return parseSinFunction_45(s, f);
    }
    private static Match e388M(Session s, Frame f) {
        return parseSinFunction_45(s, f);
    }
    private static Match e389(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e389M : TinyExpressionP4Parser::e389C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9228:9239:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e389C(Session s, Frame f) {
        return parseCosFunction_46(s, f);
    }
    private static Match e389M(Session s, Frame f) {
        return parseCosFunction_46(s, f);
    }
    private static Match e390(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e390M : TinyExpressionP4Parser::e390C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9246:9257:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e390C(Session s, Frame f) {
        return parseTanFunction_47(s, f);
    }
    private static Match e390M(Session s, Frame f) {
        return parseTanFunction_47(s, f);
    }
    private static Match e391(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e391M : TinyExpressionP4Parser::e391C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9264:9276:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e391C(Session s, Frame f) {
        return parseSqrtFunction_48(s, f);
    }
    private static Match e391M(Session s, Frame f) {
        return parseSqrtFunction_48(s, f);
    }
    private static Match e392(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e392M : TinyExpressionP4Parser::e392C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9283:9294:body/4/ruleRef", false, false, "node", false, K4);
    }
    private static Match e392C(Session s, Frame f) {
        return parseMinFunction_49(s, f);
    }
    private static Match e392M(Session s, Frame f) {
        return parseMinFunction_49(s, f);
    }
    private static Match e393(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e393M : TinyExpressionP4Parser::e393C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9301:9312:body/5/ruleRef", false, false, "node", false, K4);
    }
    private static Match e393C(Session s, Frame f) {
        return parseMaxFunction_50(s, f);
    }
    private static Match e393M(Session s, Frame f) {
        return parseMaxFunction_50(s, f);
    }
    private static Match e394(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e394M : TinyExpressionP4Parser::e394C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9319:9333:body/6/ruleRef", false, false, "node", false, K4);
    }
    private static Match e394C(Session s, Frame f) {
        return parseRandomFunction_51(s, f);
    }
    private static Match e394M(Session s, Frame f) {
        return parseRandomFunction_51(s, f);
    }
    private static Match e395(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e395M : TinyExpressionP4Parser::e395C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9340:9351:body/7/ruleRef", false, false, "node", false, K4);
    }
    private static Match e395C(Session s, Frame f) {
        return parseAbsFunction_52(s, f);
    }
    private static Match e395M(Session s, Frame f) {
        return parseAbsFunction_52(s, f);
    }
    private static Match e396(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e396M : TinyExpressionP4Parser::e396C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9358:9371:body/8/ruleRef", false, false, "node", false, K4);
    }
    private static Match e396C(Session s, Frame f) {
        return parseRoundFunction_53(s, f);
    }
    private static Match e396M(Session s, Frame f) {
        return parseRoundFunction_53(s, f);
    }
    private static Match e397(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e397M : TinyExpressionP4Parser::e397C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9378:9390:body/9/ruleRef", false, false, "node", false, K4);
    }
    private static Match e397C(Session s, Frame f) {
        return parseCeilFunction_54(s, f);
    }
    private static Match e397M(Session s, Frame f) {
        return parseCeilFunction_54(s, f);
    }
    private static Match e398(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e398M : TinyExpressionP4Parser::e398C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9397:9410:body/10/ruleRef", false, false, "node", false, K4);
    }
    private static Match e398C(Session s, Frame f) {
        return parseFloorFunction_55(s, f);
    }
    private static Match e398M(Session s, Frame f) {
        return parseFloorFunction_55(s, f);
    }
    private static Match e399(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e399M : TinyExpressionP4Parser::e399C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9417:9428:body/11/ruleRef", false, false, "node", false, K4);
    }
    private static Match e399C(Session s, Frame f) {
        return parsePowFunction_56(s, f);
    }
    private static Match e399M(Session s, Frame f) {
        return parsePowFunction_56(s, f);
    }
    private static Match e400(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e400M : TinyExpressionP4Parser::e400C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9435:9446:body/12/ruleRef", false, false, "node", false, K4);
    }
    private static Match e400C(Session s, Frame f) {
        return parseLogFunction_57(s, f);
    }
    private static Match e400M(Session s, Frame f) {
        return parseLogFunction_57(s, f);
    }
    private static Match e401(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e401M : TinyExpressionP4Parser::e401C, K215, K215, K215, K216, "TinyExpressionP4::MathFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9453:9464:body/13/ruleRef", false, false, "node", false, K4);
    }
    private static Match e401C(Session s, Frame f) {
        return parseExpFunction_58(s, f);
    }
    private static Match e401M(Session s, Frame f) {
        return parseExpFunction_58(s, f);
    }
    private static Match e402(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e402M : TinyExpressionP4Parser::e402C, K215, K215, K215, K216, "TinyExpressionP4::SinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9520:9557:body/seq", false, false, "node", false, K4);
    }
    private static Match e402C(Session s, Frame f) {
        return s.sequence(f, K595, K594);
    }
    private static Match e402M(Session s, Frame f) {
        return s.sequence(f, K595, K594);
    }
    private static Match e403(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e403M : TinyExpressionP4Parser::e403C, K215, K215, K215, K216, "TinyExpressionP4::SinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9520:9525:body/0/literal", false, true, "text", false, K596);
    }
    private static Match e403C(Session s, Frame f) {
        return s.literal(f, "sin", true, false, K597);
    }
    private static Match e403M(Session s, Frame f) {
        return s.literal(f, "sin", true, false, K597);
    }
    private static Match e404(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e404M : TinyExpressionP4Parser::e404C, K215, K215, K215, K216, "TinyExpressionP4::SinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9526:9529:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e404C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e404M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e405(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e405M : TinyExpressionP4Parser::e405C, K598, K599, K224, K225, "TinyExpressionP4::SinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9530:9548:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e405C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e405M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e406(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e406M : TinyExpressionP4Parser::e406C, K215, K215, K215, K216, "TinyExpressionP4::SinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9554:9557:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e406C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e406M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e407(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e407M : TinyExpressionP4Parser::e407C, K215, K215, K215, K216, "TinyExpressionP4::CosFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9613:9650:body/seq", false, false, "node", false, K4);
    }
    private static Match e407C(Session s, Frame f) {
        return s.sequence(f, K600, K594);
    }
    private static Match e407M(Session s, Frame f) {
        return s.sequence(f, K600, K594);
    }
    private static Match e408(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e408M : TinyExpressionP4Parser::e408C, K215, K215, K215, K216, "TinyExpressionP4::CosFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9613:9618:body/0/literal", false, true, "text", false, K601);
    }
    private static Match e408C(Session s, Frame f) {
        return s.literal(f, "cos", true, false, K602);
    }
    private static Match e408M(Session s, Frame f) {
        return s.literal(f, "cos", true, false, K602);
    }
    private static Match e409(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e409M : TinyExpressionP4Parser::e409C, K215, K215, K215, K216, "TinyExpressionP4::CosFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9619:9622:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e409C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e409M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e410(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e410M : TinyExpressionP4Parser::e410C, K603, K599, K224, K225, "TinyExpressionP4::CosFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9623:9641:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e410C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e410M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e411(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e411M : TinyExpressionP4Parser::e411C, K215, K215, K215, K216, "TinyExpressionP4::CosFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9647:9650:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e411C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e411M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e412(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e412M : TinyExpressionP4Parser::e412C, K215, K215, K215, K216, "TinyExpressionP4::TanFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9706:9743:body/seq", false, false, "node", false, K4);
    }
    private static Match e412C(Session s, Frame f) {
        return s.sequence(f, K604, K594);
    }
    private static Match e412M(Session s, Frame f) {
        return s.sequence(f, K604, K594);
    }
    private static Match e413(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e413M : TinyExpressionP4Parser::e413C, K215, K215, K215, K216, "TinyExpressionP4::TanFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9706:9711:body/0/literal", false, true, "text", false, K605);
    }
    private static Match e413C(Session s, Frame f) {
        return s.literal(f, "tan", true, false, K606);
    }
    private static Match e413M(Session s, Frame f) {
        return s.literal(f, "tan", true, false, K606);
    }
    private static Match e414(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e414M : TinyExpressionP4Parser::e414C, K215, K215, K215, K216, "TinyExpressionP4::TanFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9712:9715:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e414C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e414M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e415(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e415M : TinyExpressionP4Parser::e415C, K607, K599, K224, K225, "TinyExpressionP4::TanFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9716:9734:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e415C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e415M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e416(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e416M : TinyExpressionP4Parser::e416C, K215, K215, K215, K216, "TinyExpressionP4::TanFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9740:9743:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e416C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e416M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e417(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e417M : TinyExpressionP4Parser::e417C, K215, K215, K215, K216, "TinyExpressionP4::SqrtFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9801:9839:body/seq", false, false, "node", false, K4);
    }
    private static Match e417C(Session s, Frame f) {
        return s.sequence(f, K608, K594);
    }
    private static Match e417M(Session s, Frame f) {
        return s.sequence(f, K608, K594);
    }
    private static Match e418(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e418M : TinyExpressionP4Parser::e418C, K215, K215, K215, K216, "TinyExpressionP4::SqrtFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9801:9807:body/0/literal", false, true, "text", false, K609);
    }
    private static Match e418C(Session s, Frame f) {
        return s.literal(f, "sqrt", true, false, K610);
    }
    private static Match e418M(Session s, Frame f) {
        return s.literal(f, "sqrt", true, false, K610);
    }
    private static Match e419(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e419M : TinyExpressionP4Parser::e419C, K215, K215, K215, K216, "TinyExpressionP4::SqrtFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9808:9811:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e419C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e419M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e420(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e420M : TinyExpressionP4Parser::e420C, K611, K599, K224, K225, "TinyExpressionP4::SqrtFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9812:9830:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e420C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e420M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e421(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e421M : TinyExpressionP4Parser::e421C, K215, K215, K215, K216, "TinyExpressionP4::SqrtFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9836:9839:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e421C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e421M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e422(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e422M : TinyExpressionP4Parser::e422C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9903:9975:body/seq", false, false, "node", true, K4);
    }
    private static Match e422C(Session s, Frame f) {
        return s.sequence(f, K612, K367);
    }
    private static Match e422M(Session s, Frame f) {
        return s.sequence(f, K612, K367);
    }
    private static Match e423(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e423M : TinyExpressionP4Parser::e423C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9903:9908:body/0/literal", false, true, "text", false, K613);
    }
    private static Match e423C(Session s, Frame f) {
        return s.literal(f, "min", true, false, K614);
    }
    private static Match e423M(Session s, Frame f) {
        return s.literal(f, "min", true, false, K614);
    }
    private static Match e424(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e424M : TinyExpressionP4Parser::e424C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9909:9912:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e424C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e424M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e425(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e425M : TinyExpressionP4Parser::e425C, K615, K616, K224, K225, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9913:9931:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e425C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e425M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e426(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e426M : TinyExpressionP4Parser::e426C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9939:9971:body/3/repeat", true, false, "node", true, K377);
    }
    private static Match e426C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e427, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e426M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e427, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e427(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e427M : TinyExpressionP4Parser::e427C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9941:9963:body/3/0/seq", false, false, "node", false, K377);
    }
    private static Match e427C(Session s, Frame f) {
        return s.sequence(f, K617, K249);
    }
    private static Match e427M(Session s, Frame f) {
        return s.sequence(f, K617, K249);
    }
    private static Match e428(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e428M : TinyExpressionP4Parser::e428C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9941:9944:body/3/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e428C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e428M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e429(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e429M : TinyExpressionP4Parser::e429C, K618, K619, K224, K225, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9945:9963:body/3/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e429C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e429M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e430(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e430M : TinyExpressionP4Parser::e430C, K215, K215, K215, K216, "TinyExpressionP4::MinFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9972:9975:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e430C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e430M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e431(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e431M : TinyExpressionP4Parser::e431C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10039:10111:body/seq", false, false, "node", true, K4);
    }
    private static Match e431C(Session s, Frame f) {
        return s.sequence(f, K620, K367);
    }
    private static Match e431M(Session s, Frame f) {
        return s.sequence(f, K620, K367);
    }
    private static Match e432(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e432M : TinyExpressionP4Parser::e432C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10039:10044:body/0/literal", false, true, "text", false, K621);
    }
    private static Match e432C(Session s, Frame f) {
        return s.literal(f, "max", true, false, K622);
    }
    private static Match e432M(Session s, Frame f) {
        return s.literal(f, "max", true, false, K622);
    }
    private static Match e433(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e433M : TinyExpressionP4Parser::e433C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10045:10048:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e433C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e433M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e434(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e434M : TinyExpressionP4Parser::e434C, K623, K616, K224, K225, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10049:10067:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e434C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e434M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e435(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e435M : TinyExpressionP4Parser::e435C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10075:10107:body/3/repeat", true, false, "node", true, K377);
    }
    private static Match e435C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e436, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e435M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e436, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e436(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e436M : TinyExpressionP4Parser::e436C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10077:10099:body/3/0/seq", false, false, "node", false, K377);
    }
    private static Match e436C(Session s, Frame f) {
        return s.sequence(f, K624, K249);
    }
    private static Match e436M(Session s, Frame f) {
        return s.sequence(f, K624, K249);
    }
    private static Match e437(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e437M : TinyExpressionP4Parser::e437C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10077:10080:body/3/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e437C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e437M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e438(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e438M : TinyExpressionP4Parser::e438C, K625, K619, K224, K225, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10081:10099:body/3/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e438C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e438M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e439(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e439M : TinyExpressionP4Parser::e439C, K215, K215, K215, K216, "TinyExpressionP4::MaxFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10108:10111:body/4/literal", false, true, "text", false, K374);
    }
    private static Match e439C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e439M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e440(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e440M : TinyExpressionP4Parser::e440C, K215, K215, K215, K216, "TinyExpressionP4::RandomFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10159:10175:body/seq", false, false, "text", false, K4);
    }
    private static Match e440C(Session s, Frame f) {
        return s.sequence(f, K626, K237);
    }
    private static Match e440M(Session s, Frame f) {
        return s.sequence(f, K626, K237);
    }
    private static Match e441(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e441M : TinyExpressionP4Parser::e441C, K215, K215, K215, K216, "TinyExpressionP4::RandomFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10159:10167:body/0/literal", false, true, "text", false, K627);
    }
    private static Match e441C(Session s, Frame f) {
        return s.literal(f, "random", true, false, K628);
    }
    private static Match e441M(Session s, Frame f) {
        return s.literal(f, "random", true, false, K628);
    }
    private static Match e442(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e442M : TinyExpressionP4Parser::e442C, K215, K215, K215, K216, "TinyExpressionP4::RandomFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10168:10171:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e442C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e442M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e443(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e443M : TinyExpressionP4Parser::e443C, K215, K215, K215, K216, "TinyExpressionP4::RandomFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10172:10175:body/2/literal", false, true, "text", false, K374);
    }
    private static Match e443C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e443M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e444(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e444M : TinyExpressionP4Parser::e444C, K215, K215, K215, K216, "TinyExpressionP4::AbsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10231:10268:body/seq", false, false, "node", false, K4);
    }
    private static Match e444C(Session s, Frame f) {
        return s.sequence(f, K629, K594);
    }
    private static Match e444M(Session s, Frame f) {
        return s.sequence(f, K629, K594);
    }
    private static Match e445(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e445M : TinyExpressionP4Parser::e445C, K215, K215, K215, K216, "TinyExpressionP4::AbsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10231:10236:body/0/literal", false, true, "text", false, K630);
    }
    private static Match e445C(Session s, Frame f) {
        return s.literal(f, "abs", true, false, K631);
    }
    private static Match e445M(Session s, Frame f) {
        return s.literal(f, "abs", true, false, K631);
    }
    private static Match e446(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e446M : TinyExpressionP4Parser::e446C, K215, K215, K215, K216, "TinyExpressionP4::AbsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10237:10240:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e446C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e446M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e447(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e447M : TinyExpressionP4Parser::e447C, K632, K599, K224, K225, "TinyExpressionP4::AbsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10241:10259:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e447C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e447M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e448(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e448M : TinyExpressionP4Parser::e448C, K215, K215, K215, K216, "TinyExpressionP4::AbsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10265:10268:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e448C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e448M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e449(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e449M : TinyExpressionP4Parser::e449C, K215, K215, K215, K216, "TinyExpressionP4::RoundFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10328:10367:body/seq", false, false, "node", false, K4);
    }
    private static Match e449C(Session s, Frame f) {
        return s.sequence(f, K633, K594);
    }
    private static Match e449M(Session s, Frame f) {
        return s.sequence(f, K633, K594);
    }
    private static Match e450(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e450M : TinyExpressionP4Parser::e450C, K215, K215, K215, K216, "TinyExpressionP4::RoundFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10328:10335:body/0/literal", false, true, "text", false, K634);
    }
    private static Match e450C(Session s, Frame f) {
        return s.literal(f, "round", true, false, K635);
    }
    private static Match e450M(Session s, Frame f) {
        return s.literal(f, "round", true, false, K635);
    }
    private static Match e451(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e451M : TinyExpressionP4Parser::e451C, K215, K215, K215, K216, "TinyExpressionP4::RoundFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10336:10339:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e451C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e451M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e452(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e452M : TinyExpressionP4Parser::e452C, K636, K599, K224, K225, "TinyExpressionP4::RoundFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10340:10358:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e452C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e452M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e453(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e453M : TinyExpressionP4Parser::e453C, K215, K215, K215, K216, "TinyExpressionP4::RoundFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10364:10367:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e453C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e453M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e454(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e454M : TinyExpressionP4Parser::e454C, K215, K215, K215, K216, "TinyExpressionP4::CeilFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10425:10463:body/seq", false, false, "node", false, K4);
    }
    private static Match e454C(Session s, Frame f) {
        return s.sequence(f, K637, K594);
    }
    private static Match e454M(Session s, Frame f) {
        return s.sequence(f, K637, K594);
    }
    private static Match e455(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e455M : TinyExpressionP4Parser::e455C, K215, K215, K215, K216, "TinyExpressionP4::CeilFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10425:10431:body/0/literal", false, true, "text", false, K638);
    }
    private static Match e455C(Session s, Frame f) {
        return s.literal(f, "ceil", true, false, K639);
    }
    private static Match e455M(Session s, Frame f) {
        return s.literal(f, "ceil", true, false, K639);
    }
    private static Match e456(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e456M : TinyExpressionP4Parser::e456C, K215, K215, K215, K216, "TinyExpressionP4::CeilFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10432:10435:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e456C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e456M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e457(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e457M : TinyExpressionP4Parser::e457C, K640, K599, K224, K225, "TinyExpressionP4::CeilFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10436:10454:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e457C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e457M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e458(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e458M : TinyExpressionP4Parser::e458C, K215, K215, K215, K216, "TinyExpressionP4::CeilFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10460:10463:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e458C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e458M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e459(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e459M : TinyExpressionP4Parser::e459C, K215, K215, K215, K216, "TinyExpressionP4::FloorFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10523:10562:body/seq", false, false, "node", false, K4);
    }
    private static Match e459C(Session s, Frame f) {
        return s.sequence(f, K641, K594);
    }
    private static Match e459M(Session s, Frame f) {
        return s.sequence(f, K641, K594);
    }
    private static Match e460(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e460M : TinyExpressionP4Parser::e460C, K215, K215, K215, K216, "TinyExpressionP4::FloorFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10523:10530:body/0/literal", false, true, "text", false, K642);
    }
    private static Match e460C(Session s, Frame f) {
        return s.literal(f, "floor", true, false, K643);
    }
    private static Match e460M(Session s, Frame f) {
        return s.literal(f, "floor", true, false, K643);
    }
    private static Match e461(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e461M : TinyExpressionP4Parser::e461C, K215, K215, K215, K216, "TinyExpressionP4::FloorFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10531:10534:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e461C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e461M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e462(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e462M : TinyExpressionP4Parser::e462C, K644, K599, K224, K225, "TinyExpressionP4::FloorFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10535:10553:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e462C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e462M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e463(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e463M : TinyExpressionP4Parser::e463C, K215, K215, K215, K216, "TinyExpressionP4::FloorFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10559:10562:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e463C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e463M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e464(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e464M : TinyExpressionP4Parser::e464C, K215, K215, K215, K216, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10629:10700:body/seq", false, false, "node", true, K4);
    }
    private static Match e464C(Session s, Frame f) {
        return s.sequence(f, K645, K241);
    }
    private static Match e464M(Session s, Frame f) {
        return s.sequence(f, K645, K241);
    }
    private static Match e465(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e465M : TinyExpressionP4Parser::e465C, K215, K215, K215, K216, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10629:10634:body/0/literal", false, true, "text", false, K646);
    }
    private static Match e465C(Session s, Frame f) {
        return s.literal(f, "pow", true, false, K647);
    }
    private static Match e465M(Session s, Frame f) {
        return s.literal(f, "pow", true, false, K647);
    }
    private static Match e466(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e466M : TinyExpressionP4Parser::e466C, K215, K215, K215, K216, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10635:10638:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e466C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e466M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e467(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e467M : TinyExpressionP4Parser::e467C, K648, K649, K224, K225, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10639:10657:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e467C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e467M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e468(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e468M : TinyExpressionP4Parser::e468C, K215, K215, K215, K216, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10664:10667:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e468C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e468M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e469(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e469M : TinyExpressionP4Parser::e469C, K650, K651, K224, K225, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10668:10686:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e469C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e469M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e470(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e470M : TinyExpressionP4Parser::e470C, K215, K215, K215, K216, "TinyExpressionP4::PowFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10697:10700:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e470C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e470M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e471(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e471M : TinyExpressionP4Parser::e471C, K215, K215, K215, K216, "TinyExpressionP4::LogFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10756:10793:body/seq", false, false, "node", false, K4);
    }
    private static Match e471C(Session s, Frame f) {
        return s.sequence(f, K652, K594);
    }
    private static Match e471M(Session s, Frame f) {
        return s.sequence(f, K652, K594);
    }
    private static Match e472(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e472M : TinyExpressionP4Parser::e472C, K215, K215, K215, K216, "TinyExpressionP4::LogFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10756:10761:body/0/literal", false, true, "text", false, K653);
    }
    private static Match e472C(Session s, Frame f) {
        return s.literal(f, "log", true, false, K654);
    }
    private static Match e472M(Session s, Frame f) {
        return s.literal(f, "log", true, false, K654);
    }
    private static Match e473(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e473M : TinyExpressionP4Parser::e473C, K215, K215, K215, K216, "TinyExpressionP4::LogFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10762:10765:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e473C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e473M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e474(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e474M : TinyExpressionP4Parser::e474C, K655, K599, K224, K225, "TinyExpressionP4::LogFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10766:10784:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e474C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e474M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e475(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e475M : TinyExpressionP4Parser::e475C, K215, K215, K215, K216, "TinyExpressionP4::LogFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10790:10793:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e475C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e475M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e476(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e476M : TinyExpressionP4Parser::e476C, K215, K215, K215, K216, "TinyExpressionP4::ExpFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10849:10886:body/seq", false, false, "node", false, K4);
    }
    private static Match e476C(Session s, Frame f) {
        return s.sequence(f, K656, K594);
    }
    private static Match e476M(Session s, Frame f) {
        return s.sequence(f, K656, K594);
    }
    private static Match e477(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e477M : TinyExpressionP4Parser::e477C, K215, K215, K215, K216, "TinyExpressionP4::ExpFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10849:10854:body/0/literal", false, true, "text", false, K657);
    }
    private static Match e477C(Session s, Frame f) {
        return s.literal(f, "exp", true, false, K658);
    }
    private static Match e477M(Session s, Frame f) {
        return s.literal(f, "exp", true, false, K658);
    }
    private static Match e478(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e478M : TinyExpressionP4Parser::e478C, K215, K215, K215, K216, "TinyExpressionP4::ExpFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10855:10858:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e478C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e478M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e479(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e479M : TinyExpressionP4Parser::e479C, K659, K599, K224, K225, "TinyExpressionP4::ExpFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10859:10877:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e479C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e479M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e480(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e480M : TinyExpressionP4Parser::e480C, K215, K215, K215, K216, "TinyExpressionP4::ExpFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10883:10886:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e480C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e480M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e481(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e481M : TinyExpressionP4Parser::e481C, K215, K215, K215, K216, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11040:11116:body/seq", false, false, "node", true, K4);
    }
    private static Match e481C(Session s, Frame f) {
        return s.sequence(f, K660, K241);
    }
    private static Match e481M(Session s, Frame f) {
        return s.sequence(f, K660, K241);
    }
    private static Match e482(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e482M : TinyExpressionP4Parser::e482C, K215, K215, K215, K216, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11040:11047:body/0/literal", false, true, "text", false, K661);
    }
    private static Match e482C(Session s, Frame f) {
        return s.literal(f, "toNum", true, false, K662);
    }
    private static Match e482M(Session s, Frame f) {
        return s.literal(f, "toNum", true, false, K662);
    }
    private static Match e483(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e483M : TinyExpressionP4Parser::e483C, K215, K215, K215, K216, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11048:11051:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e483C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e483M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e484(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e484M : TinyExpressionP4Parser::e484C, K663, K299, K224, K225, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11052:11068:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e484C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e484M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e485(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e485M : TinyExpressionP4Parser::e485C, K215, K215, K215, K216, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11076:11079:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e485C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e485M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e486(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e486M : TinyExpressionP4Parser::e486C, K664, K665, K224, K225, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11080:11098:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e486C(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e486M(Session s, Frame f) {
        return parseArgumentExpression_38(s, f);
    }
    private static Match e487(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e487M : TinyExpressionP4Parser::e487C, K215, K215, K215, K216, "TinyExpressionP4::ToNumFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11113:11116:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e487C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e487M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e488(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e488M : TinyExpressionP4Parser::e488C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11145:11425:body/choice", false, false, "mixed", false, K4);
    }
    private static Match e488C(Session s, Frame f) {
        return s.choice(f,K707,false,null,false,K706);
    }
    private static Match e488M(Session s, Frame f) {
        return s.choice(f,K707,false,null,false,K706);
    }
    private static Match e489(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e489M : TinyExpressionP4Parser::e489C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11145:11162:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e489C(Session s, Frame f) {
        return parseTernaryExpression_108(s, f);
    }
    private static Match e489M(Session s, Frame f) {
        return parseTernaryExpression_108(s, f);
    }
    private static Match e490(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e490M : TinyExpressionP4Parser::e490C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11169:11190:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e490C(Session s, Frame f) {
        return parseNumberMatchExpression_109(s, f);
    }
    private static Match e490M(Session s, Frame f) {
        return parseNumberMatchExpression_109(s, f);
    }
    private static Match e491(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e491M : TinyExpressionP4Parser::e491C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11197:11209:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e491C(Session s, Frame f) {
        return parseIfExpression_106(s, f);
    }
    private static Match e491M(Session s, Frame f) {
        return parseIfExpression_106(s, f);
    }
    private static Match e492(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e492M : TinyExpressionP4Parser::e492C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11216:11228:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e492C(Session s, Frame f) {
        return parseMathFunction_44(s, f);
    }
    private static Match e492M(Session s, Frame f) {
        return parseMathFunction_44(s, f);
    }
    private static Match e493(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e493M : TinyExpressionP4Parser::e493C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11235:11248:body/4/ruleRef", false, false, "node", false, K4);
    }
    private static Match e493C(Session s, Frame f) {
        return parseToNumFunction_59(s, f);
    }
    private static Match e493M(Session s, Frame f) {
        return parseToNumFunction_59(s, f);
    }
    private static Match e494(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e494M : TinyExpressionP4Parser::e494C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11255:11270:body/5/ruleRef", false, false, "node", false, K4);
    }
    private static Match e494C(Session s, Frame f) {
        return parseLengthDotMethod_69(s, f);
    }
    private static Match e494M(Session s, Frame f) {
        return parseLengthDotMethod_69(s, f);
    }
    private static Match e495(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e495M : TinyExpressionP4Parser::e495C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11277:11288:body/6/ruleRef", false, false, "node", false, K4);
    }
    private static Match e495C(Session s, Frame f) {
        return parseLenFunction_65(s, f);
    }
    private static Match e495M(Session s, Frame f) {
        return parseLenFunction_65(s, f);
    }
    private static Match e496(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e496M : TinyExpressionP4Parser::e496C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11295:11309:body/7/ruleRef", false, false, "node", false, K4);
    }
    private static Match e496C(Session s, Frame f) {
        return parseLengthFunction_64(s, f);
    }
    private static Match e496M(Session s, Frame f) {
        return parseLengthFunction_64(s, f);
    }
    private static Match e497(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e497M : TinyExpressionP4Parser::e497C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11316:11340:body/8/ruleRef", false, false, "node", false, K4);
    }
    private static Match e497C(Session s, Frame f) {
        return parseExternalNumberInvocation_32(s, f);
    }
    private static Match e497M(Session s, Frame f) {
        return parseExternalNumberInvocation_32(s, f);
    }
    private static Match e498(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e498M : TinyExpressionP4Parser::e498C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11347:11353:body/9/tokenRef", false, true, "text", false, K708);
    }
    private static Match e498C(Session s, Frame f) {
        return s.builtin(f,"Number",K234,K709);
    }
    private static Match e498M(Session s, Frame f) {
        return s.builtin(f,"Number",K234,K709);
    }
    private static Match e499(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e499M : TinyExpressionP4Parser::e499C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11360:11371:body/10/ruleRef", false, false, "node", false, K4);
    }
    private static Match e499C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e499M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e500(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e500M : TinyExpressionP4Parser::e500C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11378:11394:body/11/ruleRef", false, false, "node", false, K4);
    }
    private static Match e500C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e500M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e501(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e501M : TinyExpressionP4Parser::e501C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11401:11425:body/12/seq", false, false, "node", false, K160);
    }
    private static Match e501C(Session s, Frame f) {
        return s.sequence(f, K710, K237);
    }
    private static Match e501M(Session s, Frame f) {
        return s.sequence(f, K710, K237);
    }
    private static Match e502(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e502M : TinyExpressionP4Parser::e502C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11401:11404:body/12/0/literal", false, true, "text", false, K372);
    }
    private static Match e502C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e502M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e503(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e503M : TinyExpressionP4Parser::e503C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11405:11421:body/12/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e503C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e503M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e504(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e504M : TinyExpressionP4Parser::e504C, K215, K215, K215, K216, "TinyExpressionP4::NumberFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11422:11425:body/12/2/literal", false, true, "text", false, K374);
    }
    private static Match e504C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e504M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e505(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e505M : TinyExpressionP4Parser::e505C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11578:11623:body/seq", false, false, "node", false, K4);
    }
    private static Match e505C(Session s, Frame f) {
        return s.sequence(f, K711, K594);
    }
    private static Match e505M(Session s, Frame f) {
        return s.sequence(f, K711, K594);
    }
    private static Match e506(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e506M : TinyExpressionP4Parser::e506C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11578:11591:body/0/literal", false, true, "text", false, K712);
    }
    private static Match e506C(Session s, Frame f) {
        return s.literal(f, "toUpperCase", true, false, K713);
    }
    private static Match e506M(Session s, Frame f) {
        return s.literal(f, "toUpperCase", true, false, K713);
    }
    private static Match e507(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e507M : TinyExpressionP4Parser::e507C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11592:11595:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e507C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e507M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e508(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e508M : TinyExpressionP4Parser::e508C, K714, K299, K224, K225, "TinyExpressionP4::ToUpperCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11596:11612:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e508C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e508M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e509(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e509M : TinyExpressionP4Parser::e509C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11620:11623:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e509C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e509M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e510(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e510M : TinyExpressionP4Parser::e510C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11697:11742:body/seq", false, false, "node", false, K4);
    }
    private static Match e510C(Session s, Frame f) {
        return s.sequence(f, K715, K594);
    }
    private static Match e510M(Session s, Frame f) {
        return s.sequence(f, K715, K594);
    }
    private static Match e511(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e511M : TinyExpressionP4Parser::e511C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11697:11710:body/0/literal", false, true, "text", false, K716);
    }
    private static Match e511C(Session s, Frame f) {
        return s.literal(f, "toLowerCase", true, false, K717);
    }
    private static Match e511M(Session s, Frame f) {
        return s.literal(f, "toLowerCase", true, false, K717);
    }
    private static Match e512(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e512M : TinyExpressionP4Parser::e512C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11711:11714:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e512C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e512M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e513(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e513M : TinyExpressionP4Parser::e513C, K718, K299, K224, K225, "TinyExpressionP4::ToLowerCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11715:11731:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e513C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e513M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e514(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e514M : TinyExpressionP4Parser::e514C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11739:11742:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e514C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e514M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e515(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e515M : TinyExpressionP4Parser::e515C, K215, K215, K215, K216, "TinyExpressionP4::TrimFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11802:11840:body/seq", false, false, "node", false, K4);
    }
    private static Match e515C(Session s, Frame f) {
        return s.sequence(f, K719, K594);
    }
    private static Match e515M(Session s, Frame f) {
        return s.sequence(f, K719, K594);
    }
    private static Match e516(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e516M : TinyExpressionP4Parser::e516C, K215, K215, K215, K216, "TinyExpressionP4::TrimFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11802:11808:body/0/literal", false, true, "text", false, K720);
    }
    private static Match e516C(Session s, Frame f) {
        return s.literal(f, "trim", true, false, K721);
    }
    private static Match e516M(Session s, Frame f) {
        return s.literal(f, "trim", true, false, K721);
    }
    private static Match e517(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e517M : TinyExpressionP4Parser::e517C, K215, K215, K215, K216, "TinyExpressionP4::TrimFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11809:11812:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e517C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e517M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e518(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e518M : TinyExpressionP4Parser::e518C, K722, K299, K224, K225, "TinyExpressionP4::TrimFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11813:11829:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e518C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e518M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e519(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e519M : TinyExpressionP4Parser::e519C, K215, K215, K215, K216, "TinyExpressionP4::TrimFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11837:11840:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e519C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e519M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e520(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e520M : TinyExpressionP4Parser::e520C, K215, K215, K215, K216, "TinyExpressionP4::LengthFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11904:11944:body/seq", false, false, "node", false, K4);
    }
    private static Match e520C(Session s, Frame f) {
        return s.sequence(f, K723, K594);
    }
    private static Match e520M(Session s, Frame f) {
        return s.sequence(f, K723, K594);
    }
    private static Match e521(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e521M : TinyExpressionP4Parser::e521C, K215, K215, K215, K216, "TinyExpressionP4::LengthFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11904:11912:body/0/literal", false, true, "text", false, K724);
    }
    private static Match e521C(Session s, Frame f) {
        return s.literal(f, "length", true, false, K725);
    }
    private static Match e521M(Session s, Frame f) {
        return s.literal(f, "length", true, false, K725);
    }
    private static Match e522(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e522M : TinyExpressionP4Parser::e522C, K215, K215, K215, K216, "TinyExpressionP4::LengthFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11913:11916:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e522C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e522M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e523(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e523M : TinyExpressionP4Parser::e523C, K726, K299, K224, K225, "TinyExpressionP4::LengthFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11917:11933:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e523C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e523M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e524(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e524M : TinyExpressionP4Parser::e524C, K215, K215, K215, K216, "TinyExpressionP4::LengthFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11941:11944:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e524C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e524M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e525(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e525M : TinyExpressionP4Parser::e525C, K215, K215, K215, K216, "TinyExpressionP4::LenFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12005:12042:body/seq", false, false, "node", false, K4);
    }
    private static Match e525C(Session s, Frame f) {
        return s.sequence(f, K727, K594);
    }
    private static Match e525M(Session s, Frame f) {
        return s.sequence(f, K727, K594);
    }
    private static Match e526(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e526M : TinyExpressionP4Parser::e526C, K215, K215, K215, K216, "TinyExpressionP4::LenFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12005:12010:body/0/literal", false, true, "text", false, K728);
    }
    private static Match e526C(Session s, Frame f) {
        return s.literal(f, "len", true, false, K729);
    }
    private static Match e526M(Session s, Frame f) {
        return s.literal(f, "len", true, false, K729);
    }
    private static Match e527(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e527M : TinyExpressionP4Parser::e527C, K215, K215, K215, K216, "TinyExpressionP4::LenFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12011:12014:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e527C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e527M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e528(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e528M : TinyExpressionP4Parser::e528C, K730, K299, K224, K225, "TinyExpressionP4::LenFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12015:12031:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e528C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e528M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e529(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e529M : TinyExpressionP4Parser::e529C, K215, K215, K215, K216, "TinyExpressionP4::LenFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12039:12042:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e529C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e529M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e530(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e530M : TinyExpressionP4Parser::e530C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12408:body/seq", false, false, "node", false, K4);
    }
    private static Match e530C(Session s, Frame f) {
        return s.sequence(f, K731, K594);
    }
    private static Match e530M(Session s, Frame f) {
        return s.sequence(f, K731, K594);
    }
    private static Match e531(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e531M : TinyExpressionP4Parser::e531C, K732, K299, K224, K225, "TinyExpressionP4::ToUpperCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12378:body/0/ruleRef", false, false, "node", false, K49);
    }
    private static Match e531C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e531M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e532(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e532M : TinyExpressionP4Parser::e532C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12386:12400:body/1/literal", false, true, "text", false, K733);
    }
    private static Match e532C(Session s, Frame f) {
        return s.literal(f, ".toUpperCase", true, false, K734);
    }
    private static Match e532M(Session s, Frame f) {
        return s.literal(f, ".toUpperCase", true, false, K734);
    }
    private static Match e533(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e533M : TinyExpressionP4Parser::e533C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12401:12404:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e533C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e533M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e534(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e534M : TinyExpressionP4Parser::e534C, K215, K215, K215, K216, "TinyExpressionP4::ToUpperCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12405:12408:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e534C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e534M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e535(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e535M : TinyExpressionP4Parser::e535C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12527:body/seq", false, false, "node", false, K4);
    }
    private static Match e535C(Session s, Frame f) {
        return s.sequence(f, K735, K594);
    }
    private static Match e535M(Session s, Frame f) {
        return s.sequence(f, K735, K594);
    }
    private static Match e536(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e536M : TinyExpressionP4Parser::e536C, K736, K299, K224, K225, "TinyExpressionP4::ToLowerCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12497:body/0/ruleRef", false, false, "node", false, K49);
    }
    private static Match e536C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e536M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e537(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e537M : TinyExpressionP4Parser::e537C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12505:12519:body/1/literal", false, true, "text", false, K737);
    }
    private static Match e537C(Session s, Frame f) {
        return s.literal(f, ".toLowerCase", true, false, K738);
    }
    private static Match e537M(Session s, Frame f) {
        return s.literal(f, ".toLowerCase", true, false, K738);
    }
    private static Match e538(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e538M : TinyExpressionP4Parser::e538C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12520:12523:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e538C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e538M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e539(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e539M : TinyExpressionP4Parser::e539C, K215, K215, K215, K216, "TinyExpressionP4::ToLowerCaseDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12524:12527:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e539C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e539M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e540(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e540M : TinyExpressionP4Parser::e540C, K215, K215, K215, K216, "TinyExpressionP4::TrimDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12625:body/seq", false, false, "node", false, K4);
    }
    private static Match e540C(Session s, Frame f) {
        return s.sequence(f, K739, K594);
    }
    private static Match e540M(Session s, Frame f) {
        return s.sequence(f, K739, K594);
    }
    private static Match e541(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e541M : TinyExpressionP4Parser::e541C, K740, K299, K224, K225, "TinyExpressionP4::TrimDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12602:body/0/ruleRef", false, false, "node", false, K49);
    }
    private static Match e541C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e541M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e542(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e542M : TinyExpressionP4Parser::e542C, K215, K215, K215, K216, "TinyExpressionP4::TrimDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12610:12617:body/1/literal", false, true, "text", false, K741);
    }
    private static Match e542C(Session s, Frame f) {
        return s.literal(f, ".trim", true, false, K742);
    }
    private static Match e542M(Session s, Frame f) {
        return s.literal(f, ".trim", true, false, K742);
    }
    private static Match e543(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e543M : TinyExpressionP4Parser::e543C, K215, K215, K215, K216, "TinyExpressionP4::TrimDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12618:12621:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e543C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e543M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e544(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e544M : TinyExpressionP4Parser::e544C, K215, K215, K215, K216, "TinyExpressionP4::TrimDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12622:12625:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e544C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e544M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e545(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e545M : TinyExpressionP4Parser::e545C, K215, K215, K215, K216, "TinyExpressionP4::LengthDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12729:body/seq", false, false, "node", false, K4);
    }
    private static Match e545C(Session s, Frame f) {
        return s.sequence(f, K743, K594);
    }
    private static Match e545M(Session s, Frame f) {
        return s.sequence(f, K743, K594);
    }
    private static Match e546(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e546M : TinyExpressionP4Parser::e546C, K744, K299, K224, K225, "TinyExpressionP4::LengthDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12704:body/0/ruleRef", false, false, "node", false, K49);
    }
    private static Match e546C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e546M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e547(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e547M : TinyExpressionP4Parser::e547C, K215, K215, K215, K216, "TinyExpressionP4::LengthDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12712:12721:body/1/literal", false, true, "text", false, K745);
    }
    private static Match e547C(Session s, Frame f) {
        return s.literal(f, ".length", true, false, K746);
    }
    private static Match e547M(Session s, Frame f) {
        return s.literal(f, ".length", true, false, K746);
    }
    private static Match e548(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e548M : TinyExpressionP4Parser::e548C, K215, K215, K215, K216, "TinyExpressionP4::LengthDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12722:12725:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e548C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e548M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e549(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e549M : TinyExpressionP4Parser::e549C, K215, K215, K215, K216, "TinyExpressionP4::LengthDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12726:12729:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e549C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e549M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e550(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e550M : TinyExpressionP4Parser::e550C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12949:13063:body/seq", false, false, "node", true, K4);
    }
    private static Match e550C(Session s, Frame f) {
        return s.sequence(f, K747, K219);
    }
    private static Match e550M(Session s, Frame f) {
        return s.sequence(f, K747, K219);
    }
    private static Match e551(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e551M : TinyExpressionP4Parser::e551C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12949:12961:body/0/literal", false, true, "text", false, K748);
    }
    private static Match e551C(Session s, Frame f) {
        return s.literal(f, "startsWith", true, false, K749);
    }
    private static Match e551M(Session s, Frame f) {
        return s.literal(f, "startsWith", true, false, K749);
    }
    private static Match e552(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e552M : TinyExpressionP4Parser::e552C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12962:12965:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e552C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e552M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e553(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e553M : TinyExpressionP4Parser::e553C, K750, K299, K224, K225, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12966:12982:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e553C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e553M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e554(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e554M : TinyExpressionP4Parser::e554C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12990:12993:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e554C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e554M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e555(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e555M : TinyExpressionP4Parser::e555C, K751, K752, K224, K225, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12994:13010:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e555C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e555M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e556(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e556M : TinyExpressionP4Parser::e556C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13025:13059:body/5/repeat", true, false, "node", true, K377);
    }
    private static Match e556C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e557, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e556M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e557, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e557(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e557M : TinyExpressionP4Parser::e557C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13027:13047:body/5/0/seq", false, false, "node", false, K377);
    }
    private static Match e557C(Session s, Frame f) {
        return s.sequence(f, K753, K249);
    }
    private static Match e557M(Session s, Frame f) {
        return s.sequence(f, K753, K249);
    }
    private static Match e558(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e558M : TinyExpressionP4Parser::e558C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13027:13030:body/5/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e558C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e558M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e559(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e559M : TinyExpressionP4Parser::e559C, K754, K752, K224, K225, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13031:13047:body/5/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e559C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e559M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e560(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e560M : TinyExpressionP4Parser::e560C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13060:13063:body/6/literal", false, true, "text", false, K374);
    }
    private static Match e560C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e560M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e561(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e561M : TinyExpressionP4Parser::e561C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13141:13253:body/seq", false, false, "node", true, K4);
    }
    private static Match e561C(Session s, Frame f) {
        return s.sequence(f, K755, K219);
    }
    private static Match e561M(Session s, Frame f) {
        return s.sequence(f, K755, K219);
    }
    private static Match e562(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e562M : TinyExpressionP4Parser::e562C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13141:13151:body/0/literal", false, true, "text", false, K756);
    }
    private static Match e562C(Session s, Frame f) {
        return s.literal(f, "endsWith", true, false, K757);
    }
    private static Match e562M(Session s, Frame f) {
        return s.literal(f, "endsWith", true, false, K757);
    }
    private static Match e563(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e563M : TinyExpressionP4Parser::e563C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13152:13155:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e563C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e563M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e564(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e564M : TinyExpressionP4Parser::e564C, K758, K299, K224, K225, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13156:13172:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e564C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e564M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e565(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e565M : TinyExpressionP4Parser::e565C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13180:13183:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e565C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e565M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e566(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e566M : TinyExpressionP4Parser::e566C, K759, K752, K224, K225, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13184:13200:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e566C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e566M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e567(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e567M : TinyExpressionP4Parser::e567C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13215:13249:body/5/repeat", true, false, "node", true, K377);
    }
    private static Match e567C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e568, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e567M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e568, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e568(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e568M : TinyExpressionP4Parser::e568C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13217:13237:body/5/0/seq", false, false, "node", false, K377);
    }
    private static Match e568C(Session s, Frame f) {
        return s.sequence(f, K760, K249);
    }
    private static Match e568M(Session s, Frame f) {
        return s.sequence(f, K760, K249);
    }
    private static Match e569(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e569M : TinyExpressionP4Parser::e569C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13217:13220:body/5/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e569C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e569M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e570(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e570M : TinyExpressionP4Parser::e570C, K761, K752, K224, K225, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13221:13237:body/5/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e570C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e570M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e571(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e571M : TinyExpressionP4Parser::e571C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13250:13253:body/6/literal", false, true, "text", false, K374);
    }
    private static Match e571C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e571M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e572(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e572M : TinyExpressionP4Parser::e572C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13331:13443:body/seq", false, false, "node", true, K4);
    }
    private static Match e572C(Session s, Frame f) {
        return s.sequence(f, K762, K219);
    }
    private static Match e572M(Session s, Frame f) {
        return s.sequence(f, K762, K219);
    }
    private static Match e573(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e573M : TinyExpressionP4Parser::e573C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13331:13341:body/0/literal", false, true, "text", false, K763);
    }
    private static Match e573C(Session s, Frame f) {
        return s.literal(f, "contains", true, false, K764);
    }
    private static Match e573M(Session s, Frame f) {
        return s.literal(f, "contains", true, false, K764);
    }
    private static Match e574(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e574M : TinyExpressionP4Parser::e574C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13342:13345:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e574C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e574M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e575(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e575M : TinyExpressionP4Parser::e575C, K765, K299, K224, K225, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13346:13362:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e575C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e575M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e576(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e576M : TinyExpressionP4Parser::e576C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13370:13373:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e576C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e576M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e577(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e577M : TinyExpressionP4Parser::e577C, K766, K752, K224, K225, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13374:13390:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e577C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e577M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e578(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e578M : TinyExpressionP4Parser::e578C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13405:13439:body/5/repeat", true, false, "node", true, K377);
    }
    private static Match e578C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e579, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e578M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e579, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e579(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e579M : TinyExpressionP4Parser::e579C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13407:13427:body/5/0/seq", false, false, "node", false, K377);
    }
    private static Match e579C(Session s, Frame f) {
        return s.sequence(f, K767, K249);
    }
    private static Match e579M(Session s, Frame f) {
        return s.sequence(f, K767, K249);
    }
    private static Match e580(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e580M : TinyExpressionP4Parser::e580C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13407:13410:body/5/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e580C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e580M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e581(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e581M : TinyExpressionP4Parser::e581C, K768, K752, K224, K225, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13411:13427:body/5/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e581C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e581M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e582(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e582M : TinyExpressionP4Parser::e582C, K215, K215, K215, K216, "TinyExpressionP4::ContainsFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13440:13443:body/6/literal", false, true, "text", false, K374);
    }
    private static Match e582C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e582M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e583(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e583M : TinyExpressionP4Parser::e583C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13612:body/seq", false, false, "node", true, K4);
    }
    private static Match e583C(Session s, Frame f) {
        return s.sequence(f, K769, K241);
    }
    private static Match e583M(Session s, Frame f) {
        return s.sequence(f, K769, K241);
    }
    private static Match e584(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e584M : TinyExpressionP4Parser::e584C, K770, K299, K224, K225, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13525:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e584C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e584M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e585(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e585M : TinyExpressionP4Parser::e585C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13533:13538:body/1/literal", false, true, "text", false, K771);
    }
    private static Match e585C(Session s, Frame f) {
        return s.literal(f, ".in", true, false, K772);
    }
    private static Match e585M(Session s, Frame f) {
        return s.literal(f, ".in", true, false, K772);
    }
    private static Match e586(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e586M : TinyExpressionP4Parser::e586C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13539:13542:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e586C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e586M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e587(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e587M : TinyExpressionP4Parser::e587C, K773, K774, K224, K225, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13543:13559:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e587C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e587M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e588(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e588M : TinyExpressionP4Parser::e588C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13572:13608:body/4/repeat", true, false, "node", true, K377);
    }
    private static Match e588C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e589, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e588M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e589, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e589(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e589M : TinyExpressionP4Parser::e589C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13574:13594:body/4/0/seq", false, false, "node", false, K377);
    }
    private static Match e589C(Session s, Frame f) {
        return s.sequence(f, K775, K249);
    }
    private static Match e589M(Session s, Frame f) {
        return s.sequence(f, K775, K249);
    }
    private static Match e590(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e590M : TinyExpressionP4Parser::e590C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13574:13577:body/4/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e590C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e590M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e591(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e591M : TinyExpressionP4Parser::e591C, K776, K774, K224, K225, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13578:13594:body/4/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e591C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e591M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e592(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e592M : TinyExpressionP4Parser::e592C, K215, K215, K215, K216, "TinyExpressionP4::InMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13609:13612:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e592C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e592M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e593(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e593M : TinyExpressionP4Parser::e593C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13895:body/seq", false, false, "node", true, K4);
    }
    private static Match e593C(Session s, Frame f) {
        return s.sequence(f, K777, K241);
    }
    private static Match e593M(Session s, Frame f) {
        return s.sequence(f, K777, K241);
    }
    private static Match e594(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e594M : TinyExpressionP4Parser::e594C, K778, K299, K224, K225, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13800:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e594C(Session s, Frame f) {
        return parseStringPredicateReceiver_77(s, f);
    }
    private static Match e594M(Session s, Frame f) {
        return parseStringPredicateReceiver_77(s, f);
    }
    private static Match e595(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e595M : TinyExpressionP4Parser::e595C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13808:13821:body/1/literal", false, true, "text", false, K779);
    }
    private static Match e595C(Session s, Frame f) {
        return s.literal(f, ".startsWith", true, false, K780);
    }
    private static Match e595M(Session s, Frame f) {
        return s.literal(f, ".startsWith", true, false, K780);
    }
    private static Match e596(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e596M : TinyExpressionP4Parser::e596C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13822:13825:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e596C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e596M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e597(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e597M : TinyExpressionP4Parser::e597C, K781, K752, K224, K225, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13826:13842:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e597C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e597M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e598(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e598M : TinyExpressionP4Parser::e598C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13857:13891:body/4/repeat", true, false, "node", true, K377);
    }
    private static Match e598C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e599, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e598M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e599, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e599(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e599M : TinyExpressionP4Parser::e599C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13859:13879:body/4/0/seq", false, false, "node", false, K377);
    }
    private static Match e599C(Session s, Frame f) {
        return s.sequence(f, K782, K249);
    }
    private static Match e599M(Session s, Frame f) {
        return s.sequence(f, K782, K249);
    }
    private static Match e600(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e600M : TinyExpressionP4Parser::e600C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13859:13862:body/4/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e600C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e600M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e601(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e601M : TinyExpressionP4Parser::e601C, K783, K752, K224, K225, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13863:13879:body/4/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e601C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e601M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e602(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e602M : TinyExpressionP4Parser::e602C, K215, K215, K215, K216, "TinyExpressionP4::StartsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13892:13895:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e602C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e602M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e603(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e603M : TinyExpressionP4Parser::e603C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14093:body/seq", false, false, "node", true, K4);
    }
    private static Match e603C(Session s, Frame f) {
        return s.sequence(f, K784, K241);
    }
    private static Match e603M(Session s, Frame f) {
        return s.sequence(f, K784, K241);
    }
    private static Match e604(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e604M : TinyExpressionP4Parser::e604C, K785, K299, K224, K225, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14000:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e604C(Session s, Frame f) {
        return parseStringPredicateReceiver_77(s, f);
    }
    private static Match e604M(Session s, Frame f) {
        return parseStringPredicateReceiver_77(s, f);
    }
    private static Match e605(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e605M : TinyExpressionP4Parser::e605C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14008:14019:body/1/literal", false, true, "text", false, K786);
    }
    private static Match e605C(Session s, Frame f) {
        return s.literal(f, ".endsWith", true, false, K787);
    }
    private static Match e605M(Session s, Frame f) {
        return s.literal(f, ".endsWith", true, false, K787);
    }
    private static Match e606(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e606M : TinyExpressionP4Parser::e606C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14020:14023:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e606C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e606M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e607(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e607M : TinyExpressionP4Parser::e607C, K788, K752, K224, K225, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14024:14040:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e607C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e607M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e608(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e608M : TinyExpressionP4Parser::e608C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14055:14089:body/4/repeat", true, false, "node", true, K377);
    }
    private static Match e608C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e609, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e608M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e609, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e609(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e609M : TinyExpressionP4Parser::e609C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14057:14077:body/4/0/seq", false, false, "node", false, K377);
    }
    private static Match e609C(Session s, Frame f) {
        return s.sequence(f, K789, K249);
    }
    private static Match e609M(Session s, Frame f) {
        return s.sequence(f, K789, K249);
    }
    private static Match e610(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e610M : TinyExpressionP4Parser::e610C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14057:14060:body/4/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e610C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e610M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e611(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e611M : TinyExpressionP4Parser::e611C, K790, K752, K224, K225, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14061:14077:body/4/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e611C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e611M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e612(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e612M : TinyExpressionP4Parser::e612C, K215, K215, K215, K216, "TinyExpressionP4::EndsWithDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14090:14093:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e612C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e612M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e613(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e613M : TinyExpressionP4Parser::e613C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14291:body/seq", false, false, "node", true, K4);
    }
    private static Match e613C(Session s, Frame f) {
        return s.sequence(f, K791, K241);
    }
    private static Match e613M(Session s, Frame f) {
        return s.sequence(f, K791, K241);
    }
    private static Match e614(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e614M : TinyExpressionP4Parser::e614C, K792, K299, K224, K225, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14198:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e614C(Session s, Frame f) {
        return parseStringPredicateReceiver_77(s, f);
    }
    private static Match e614M(Session s, Frame f) {
        return parseStringPredicateReceiver_77(s, f);
    }
    private static Match e615(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e615M : TinyExpressionP4Parser::e615C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14206:14217:body/1/literal", false, true, "text", false, K793);
    }
    private static Match e615C(Session s, Frame f) {
        return s.literal(f, ".contains", true, false, K794);
    }
    private static Match e615M(Session s, Frame f) {
        return s.literal(f, ".contains", true, false, K794);
    }
    private static Match e616(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e616M : TinyExpressionP4Parser::e616C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14218:14221:body/2/literal", false, true, "text", false, K372);
    }
    private static Match e616C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e616M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e617(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e617M : TinyExpressionP4Parser::e617C, K795, K752, K224, K225, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14222:14238:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e617C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e617M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e618(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e618M : TinyExpressionP4Parser::e618C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14253:14287:body/4/repeat", true, false, "node", true, K377);
    }
    private static Match e618C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e619, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e618M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e619, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e619(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e619M : TinyExpressionP4Parser::e619C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14255:14275:body/4/0/seq", false, false, "node", false, K377);
    }
    private static Match e619C(Session s, Frame f) {
        return s.sequence(f, K796, K249);
    }
    private static Match e619M(Session s, Frame f) {
        return s.sequence(f, K796, K249);
    }
    private static Match e620(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e620M : TinyExpressionP4Parser::e620C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14255:14258:body/4/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e620C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e620M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e621(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e621M : TinyExpressionP4Parser::e621C, K797, K752, K224, K225, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14259:14275:body/4/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e621C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e621M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e622(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e622M : TinyExpressionP4Parser::e622C, K215, K215, K215, K216, "TinyExpressionP4::ContainsDotMethod", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14288:14291:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e622C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e622M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e623(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e623M : TinyExpressionP4Parser::e623C, K215, K215, K215, K216, "TinyExpressionP4::StringPredicateReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14325:14395:body/choice", false, false, "node", false, K4);
    }
    private static Match e623C(Session s, Frame f) {
        return s.choice(f,K808,false,null,false,K807);
    }
    private static Match e623M(Session s, Frame f) {
        return s.choice(f,K808,false,null,false,K807);
    }
    private static Match e624(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e624M : TinyExpressionP4Parser::e624C, K215, K215, K215, K216, "TinyExpressionP4::StringPredicateReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14325:14344:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e624C(Session s, Frame f) {
        return parseToLowerCaseFunction_62(s, f);
    }
    private static Match e624M(Session s, Frame f) {
        return parseToLowerCaseFunction_62(s, f);
    }
    private static Match e625(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e625M : TinyExpressionP4Parser::e625C, K215, K215, K215, K216, "TinyExpressionP4::StringPredicateReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14347:14366:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e625C(Session s, Frame f) {
        return parseToUpperCaseFunction_61(s, f);
    }
    private static Match e625M(Session s, Frame f) {
        return parseToUpperCaseFunction_61(s, f);
    }
    private static Match e626(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e626M : TinyExpressionP4Parser::e626C, K215, K215, K215, K216, "TinyExpressionP4::StringPredicateReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14369:14381:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e626C(Session s, Frame f) {
        return parseTrimFunction_63(s, f);
    }
    private static Match e626M(Session s, Frame f) {
        return parseTrimFunction_63(s, f);
    }
    private static Match e627(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e627M : TinyExpressionP4Parser::e627C, K215, K215, K215, K216, "TinyExpressionP4::StringPredicateReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14384:14395:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e627C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e627M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e628(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e628M : TinyExpressionP4Parser::e628C, K215, K215, K215, K216, "TinyExpressionP4::IsPresentFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14544:14582:body/seq", false, false, "node", false, K4);
    }
    private static Match e628C(Session s, Frame f) {
        return s.sequence(f, K809, K594);
    }
    private static Match e628M(Session s, Frame f) {
        return s.sequence(f, K809, K594);
    }
    private static Match e629(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e629M : TinyExpressionP4Parser::e629C, K215, K215, K215, K216, "TinyExpressionP4::IsPresentFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14544:14555:body/0/literal", false, true, "text", false, K810);
    }
    private static Match e629C(Session s, Frame f) {
        return s.literal(f, "isPresent", true, false, K811);
    }
    private static Match e629M(Session s, Frame f) {
        return s.literal(f, "isPresent", true, false, K811);
    }
    private static Match e630(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e630M : TinyExpressionP4Parser::e630C, K215, K215, K215, K216, "TinyExpressionP4::IsPresentFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14556:14559:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e630C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e630M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e631(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e631M : TinyExpressionP4Parser::e631C, K812, K299, K224, K225, "TinyExpressionP4::IsPresentFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14560:14571:body/2/ruleRef", false, false, "node", false, K49);
    }
    private static Match e631C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e631M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e632(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e632M : TinyExpressionP4Parser::e632C, K215, K215, K215, K216, "TinyExpressionP4::IsPresentFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14579:14582:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e632C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e632M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e633(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e633M : TinyExpressionP4Parser::e633C, K215, K215, K215, K216, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14746:14825:body/seq", false, false, "node", true, K4);
    }
    private static Match e633C(Session s, Frame f) {
        return s.sequence(f, K813, K241);
    }
    private static Match e633M(Session s, Frame f) {
        return s.sequence(f, K813, K241);
    }
    private static Match e634(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e634M : TinyExpressionP4Parser::e634C, K215, K215, K215, K216, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14746:14759:body/0/literal", false, true, "text", false, K814);
    }
    private static Match e634C(Session s, Frame f) {
        return s.literal(f, "inTimeRange", true, false, K815);
    }
    private static Match e634M(Session s, Frame f) {
        return s.literal(f, "inTimeRange", true, false, K815);
    }
    private static Match e635(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e635M : TinyExpressionP4Parser::e635C, K215, K215, K215, K216, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14760:14763:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e635C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e635M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e636(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e636M : TinyExpressionP4Parser::e636C, K816, K817, K224, K225, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14764:14780:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e636C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e636M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e637(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e637M : TinyExpressionP4Parser::e637C, K215, K215, K215, K216, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14792:14795:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e637C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e637M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e638(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e638M : TinyExpressionP4Parser::e638C, K818, K819, K224, K225, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14796:14812:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e638C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e638M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e639(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e639M : TinyExpressionP4Parser::e639C, K215, K215, K215, K216, "TinyExpressionP4::InTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14822:14825:body/5/literal", false, true, "text", false, K374);
    }
    private static Match e639C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e639M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e640(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e640M : TinyExpressionP4Parser::e640C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14936:15064:body/seq", false, false, "node", true, K4);
    }
    private static Match e640C(Session s, Frame f) {
        return s.sequence(f, K822, K821);
    }
    private static Match e640M(Session s, Frame f) {
        return s.sequence(f, K822, K821);
    }
    private static Match e641(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e641M : TinyExpressionP4Parser::e641C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14936:14952:body/0/literal", false, true, "text", false, K823);
    }
    private static Match e641C(Session s, Frame f) {
        return s.literal(f, "inDayTimeRange", true, false, K824);
    }
    private static Match e641M(Session s, Frame f) {
        return s.literal(f, "inDayTimeRange", true, false, K824);
    }
    private static Match e642(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e642M : TinyExpressionP4Parser::e642C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14953:14956:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e642C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e642M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e643(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e643M : TinyExpressionP4Parser::e643C, K825, K826, K224, K225, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14957:14966:body/2/ruleRef", false, false, "text", false, K149);
    }
    private static Match e643C(Session s, Frame f) {
        return parseDayOfWeek_81(s, f);
    }
    private static Match e643M(Session s, Frame f) {
        return parseDayOfWeek_81(s, f);
    }
    private static Match e644(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e644M : TinyExpressionP4Parser::e644C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14977:14980:body/3/literal", false, true, "text", false, K377);
    }
    private static Match e644C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e644M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e645(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e645M : TinyExpressionP4Parser::e645C, K827, K817, K224, K225, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14981:14997:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e645C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e645M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e646(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e646M : TinyExpressionP4Parser::e646C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15009:15012:body/5/literal", false, true, "text", false, K377);
    }
    private static Match e646C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e646M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e647(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e647M : TinyExpressionP4Parser::e647C, K828, K829, K224, K225, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15013:15022:body/6/ruleRef", false, false, "text", false, K149);
    }
    private static Match e647C(Session s, Frame f) {
        return parseDayOfWeek_81(s, f);
    }
    private static Match e647M(Session s, Frame f) {
        return parseDayOfWeek_81(s, f);
    }
    private static Match e648(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e648M : TinyExpressionP4Parser::e648C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15031:15034:body/7/literal", false, true, "text", false, K377);
    }
    private static Match e648C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e648M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e649(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e649M : TinyExpressionP4Parser::e649C, K830, K819, K224, K225, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15035:15051:body/8/ruleRef", false, false, "node", false, K228);
    }
    private static Match e649C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e649M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e650(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e650M : TinyExpressionP4Parser::e650C, K215, K215, K215, K216, "TinyExpressionP4::InDayTimeRangeFunction", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15061:15064:body/9/literal", false, true, "text", false, K374);
    }
    private static Match e650C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e650M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e651(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e651M : TinyExpressionP4Parser::e651C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15084:15166:body/choice", false, false, "text", false, K4);
    }
    private static Match e651C(Session s, Frame f) {
        return s.choice(f,K831,false,null,false);
    }
    private static Match e651M(Session s, Frame f) {
        return s.choice(f,K831,false,null,false);
    }
    private static Match e652(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e652M : TinyExpressionP4Parser::e652C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15084:15092:body/0/literal", false, true, "text", false, K832);
    }
    private static Match e652C(Session s, Frame f) {
        return s.literal(f, "MONDAY", true, false, K833);
    }
    private static Match e652M(Session s, Frame f) {
        return s.literal(f, "MONDAY", true, false, K833);
    }
    private static Match e653(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e653M : TinyExpressionP4Parser::e653C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15095:15104:body/1/literal", false, true, "text", false, K834);
    }
    private static Match e653C(Session s, Frame f) {
        return s.literal(f, "TUESDAY", true, false, K835);
    }
    private static Match e653M(Session s, Frame f) {
        return s.literal(f, "TUESDAY", true, false, K835);
    }
    private static Match e654(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e654M : TinyExpressionP4Parser::e654C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15107:15118:body/2/literal", false, true, "text", false, K836);
    }
    private static Match e654C(Session s, Frame f) {
        return s.literal(f, "WEDNESDAY", true, false, K837);
    }
    private static Match e654M(Session s, Frame f) {
        return s.literal(f, "WEDNESDAY", true, false, K837);
    }
    private static Match e655(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e655M : TinyExpressionP4Parser::e655C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15121:15131:body/3/literal", false, true, "text", false, K838);
    }
    private static Match e655C(Session s, Frame f) {
        return s.literal(f, "THURSDAY", true, false, K839);
    }
    private static Match e655M(Session s, Frame f) {
        return s.literal(f, "THURSDAY", true, false, K839);
    }
    private static Match e656(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e656M : TinyExpressionP4Parser::e656C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15134:15142:body/4/literal", false, true, "text", false, K840);
    }
    private static Match e656C(Session s, Frame f) {
        return s.literal(f, "FRIDAY", true, false, K841);
    }
    private static Match e656M(Session s, Frame f) {
        return s.literal(f, "FRIDAY", true, false, K841);
    }
    private static Match e657(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e657M : TinyExpressionP4Parser::e657C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15145:15155:body/5/literal", false, true, "text", false, K842);
    }
    private static Match e657C(Session s, Frame f) {
        return s.literal(f, "SATURDAY", true, false, K843);
    }
    private static Match e657M(Session s, Frame f) {
        return s.literal(f, "SATURDAY", true, false, K843);
    }
    private static Match e658(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e658M : TinyExpressionP4Parser::e658C, K215, K215, K215, K216, "TinyExpressionP4::DayOfWeek", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15158:15166:body/6/literal", false, true, "text", false, K844);
    }
    private static Match e658C(Session s, Frame f) {
        return s.literal(f, "SUNDAY", true, false, K845);
    }
    private static Match e658M(Session s, Frame f) {
        return s.literal(f, "SUNDAY", true, false, K845);
    }
    private static Match e659(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e659M : TinyExpressionP4Parser::e659C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15334:15603:body/choice", false, false, "mixed", false, K4);
    }
    private static Match e659C(Session s, Frame f) {
        return s.choice(f,K863,false,null,false,K862);
    }
    private static Match e659M(Session s, Frame f) {
        return s.choice(f,K863,false,null,false,K862);
    }
    private static Match e660(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e660M : TinyExpressionP4Parser::e660C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15334:15358:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e660C(Session s, Frame f) {
        return parseExternalStringInvocation_33(s, f);
    }
    private static Match e660M(Session s, Frame f) {
        return parseExternalStringInvocation_33(s, f);
    }
    private static Match e661(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e661M : TinyExpressionP4Parser::e661C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15365:15384:body/1/seq", false, false, "node", false, K864);
    }
    private static Match e661C(Session s, Frame f) {
        return s.sequence(f, K865, K237);
    }
    private static Match e661M(Session s, Frame f) {
        return s.sequence(f, K865, K237);
    }
    private static Match e662(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e662M : TinyExpressionP4Parser::e662C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15365:15368:body/1/0/literal", false, true, "text", false, K372);
    }
    private static Match e662C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e662M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e663(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e663M : TinyExpressionP4Parser::e663C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15369:15380:body/1/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e663C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e663M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e664(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e664M : TinyExpressionP4Parser::e664C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15381:15384:body/1/2/literal", false, true, "text", false, K374);
    }
    private static Match e664C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e664M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e665(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e665M : TinyExpressionP4Parser::e665C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15391:15420:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e665C(Session s, Frame f) {
        return parseParenthesizedStringExpression_90(s, f);
    }
    private static Match e665M(Session s, Frame f) {
        return parseParenthesizedStringExpression_90(s, f);
    }
    private static Match e666(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e666M : TinyExpressionP4Parser::e666C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15427:15446:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e666C(Session s, Frame f) {
        return parseToUpperCaseFunction_61(s, f);
    }
    private static Match e666M(Session s, Frame f) {
        return parseToUpperCaseFunction_61(s, f);
    }
    private static Match e667(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e667M : TinyExpressionP4Parser::e667C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15449:15468:body/4/ruleRef", false, false, "node", false, K4);
    }
    private static Match e667C(Session s, Frame f) {
        return parseToLowerCaseFunction_62(s, f);
    }
    private static Match e667M(Session s, Frame f) {
        return parseToLowerCaseFunction_62(s, f);
    }
    private static Match e668(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e668M : TinyExpressionP4Parser::e668C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15471:15483:body/5/ruleRef", false, false, "node", false, K4);
    }
    private static Match e668C(Session s, Frame f) {
        return parseTrimFunction_63(s, f);
    }
    private static Match e668M(Session s, Frame f) {
        return parseTrimFunction_63(s, f);
    }
    private static Match e669(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e669M : TinyExpressionP4Parser::e669C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15490:15510:body/6/ruleRef", false, false, "node", false, K4);
    }
    private static Match e669C(Session s, Frame f) {
        return parseToUpperCaseDotMethod_66(s, f);
    }
    private static Match e669M(Session s, Frame f) {
        return parseToUpperCaseDotMethod_66(s, f);
    }
    private static Match e670(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e670M : TinyExpressionP4Parser::e670C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15513:15533:body/7/ruleRef", false, false, "node", false, K4);
    }
    private static Match e670C(Session s, Frame f) {
        return parseToLowerCaseDotMethod_67(s, f);
    }
    private static Match e670M(Session s, Frame f) {
        return parseToLowerCaseDotMethod_67(s, f);
    }
    private static Match e671(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e671M : TinyExpressionP4Parser::e671C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15536:15549:body/8/ruleRef", false, false, "node", false, K4);
    }
    private static Match e671C(Session s, Frame f) {
        return parseTrimDotMethod_68(s, f);
    }
    private static Match e671M(Session s, Frame f) {
        return parseTrimDotMethod_68(s, f);
    }
    private static Match e672(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e672M : TinyExpressionP4Parser::e672C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15556:15562:body/9/tokenRef", false, true, "text", false, K4);
    }
    private static Match e672C(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::STRING",2);
    }
    private static Match e672M(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::STRING",2);
    }
    private static Match e673(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e673M : TinyExpressionP4Parser::e673C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15569:15580:body/10/ruleRef", false, false, "node", false, K4);
    }
    private static Match e673C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e673M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e674(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e674M : TinyExpressionP4Parser::e674C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseReceiver", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15587:15603:body/11/ruleRef", false, false, "node", false, K4);
    }
    private static Match e674C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e674M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e675(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e675M : TinyExpressionP4Parser::e675C, K215, K215, K215, K216, "TinyExpressionP4::SliceStartIndex", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16119:16135:body/seq", false, false, "node", false, K4);
    }
    private static Match e675C(Session s, Frame f) {
        return s.sequence(f, K866, K420);
    }
    private static Match e675M(Session s, Frame f) {
        return s.sequence(f, K866, K420);
    }
    private static Match e676(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e676M : TinyExpressionP4Parser::e676C, K215, K215, K215, K216, "TinyExpressionP4::SliceStartIndex", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16119:16135:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e676C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e676M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e677(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e677M : TinyExpressionP4Parser::e677C, K215, K215, K215, K216, "TinyExpressionP4::SliceEndIndex", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16160:16176:body/seq", false, false, "node", false, K4);
    }
    private static Match e677C(Session s, Frame f) {
        return s.sequence(f, K867, K420);
    }
    private static Match e677M(Session s, Frame f) {
        return s.sequence(f, K867, K420);
    }
    private static Match e678(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e678M : TinyExpressionP4Parser::e678C, K215, K215, K215, K216, "TinyExpressionP4::SliceEndIndex", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16160:16176:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e678C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e678M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e679(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e679M : TinyExpressionP4Parser::e679C, K215, K215, K215, K216, "TinyExpressionP4::SliceStepIndex", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16201:16217:body/seq", false, false, "node", false, K4);
    }
    private static Match e679C(Session s, Frame f) {
        return s.sequence(f, K868, K420);
    }
    private static Match e679M(Session s, Frame f) {
        return s.sequence(f, K868, K420);
    }
    private static Match e680(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e680M : TinyExpressionP4Parser::e680C, K215, K215, K215, K216, "TinyExpressionP4::SliceStepIndex", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16201:16217:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e680C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e680M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e681(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e681M : TinyExpressionP4Parser::e681C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16914:body/choice", false, false, "mixed", true, K4);
    }
    private static Match e681C(Session s, Frame f) {
        return s.choice(f,K869,false,null,false);
    }
    private static Match e681M(Session s, Frame f) {
        return s.choice(f,K869,false,null,false);
    }
    private static Match e682(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e682M : TinyExpressionP4Parser::e682C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16412:body/0/seq", false, false, "mixed", true, K155);
    }
    private static Match e682C(Session s, Frame f) {
        return s.sequence(f, K870, K384);
    }
    private static Match e682M(Session s, Frame f) {
        return s.sequence(f, K870, K384);
    }
    private static Match e683(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e683M : TinyExpressionP4Parser::e683C, K871, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16326:body/0/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e683C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e683M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e684(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e684M : TinyExpressionP4Parser::e684C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16334:16337:body/0/1/literal", false, true, "text", false, K872);
    }
    private static Match e684C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e684M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e685(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e685M : TinyExpressionP4Parser::e685C, K874, K875, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16338:16353:body/0/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e685C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e685M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e686(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e686M : TinyExpressionP4Parser::e686C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16361:16364:body/0/3/literal", false, true, "text", false, K448);
    }
    private static Match e686C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e686M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e687(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e687M : TinyExpressionP4Parser::e687C, K876, K877, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16365:16378:body/0/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e687C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e687M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e688(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e688M : TinyExpressionP4Parser::e688C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16384:16387:body/0/5/literal", false, true, "text", false, K448);
    }
    private static Match e688C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e688M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e689(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e689M : TinyExpressionP4Parser::e689C, K878, K879, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16388:16402:body/0/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e689C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e689M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e690(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e690M : TinyExpressionP4Parser::e690C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16409:16412:body/0/7/literal", false, true, "text", false, K880);
    }
    private static Match e690C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e690M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e691(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e691M : TinyExpressionP4Parser::e691C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16497:body/1/seq", false, false, "mixed", true, K155);
    }
    private static Match e691C(Session s, Frame f) {
        return s.sequence(f, K882, K241);
    }
    private static Match e691M(Session s, Frame f) {
        return s.sequence(f, K882, K241);
    }
    private static Match e692(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e692M : TinyExpressionP4Parser::e692C, K883, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16436:body/1/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e692C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e692M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e693(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e693M : TinyExpressionP4Parser::e693C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16444:16447:body/1/1/literal", false, true, "text", false, K872);
    }
    private static Match e693C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e693M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e694(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e694M : TinyExpressionP4Parser::e694C, K884, K875, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16448:16463:body/1/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e694C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e694M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e695(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e695M : TinyExpressionP4Parser::e695C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16471:16474:body/1/3/literal", false, true, "text", false, K448);
    }
    private static Match e695C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e695M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e696(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e696M : TinyExpressionP4Parser::e696C, K885, K877, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16475:16488:body/1/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e696C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e696M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e697(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e697M : TinyExpressionP4Parser::e697C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16494:16497:body/1/5/literal", false, true, "text", false, K880);
    }
    private static Match e697C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e697M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e698(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e698M : TinyExpressionP4Parser::e698C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16588:body/2/seq", false, false, "mixed", true, K155);
    }
    private static Match e698C(Session s, Frame f) {
        return s.sequence(f, K886, K219);
    }
    private static Match e698M(Session s, Frame f) {
        return s.sequence(f, K886, K219);
    }
    private static Match e699(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e699M : TinyExpressionP4Parser::e699C, K887, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16521:body/2/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e699C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e699M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e700(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e700M : TinyExpressionP4Parser::e700C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16529:16532:body/2/1/literal", false, true, "text", false, K872);
    }
    private static Match e700C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e700M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e701(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e701M : TinyExpressionP4Parser::e701C, K888, K875, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16533:16548:body/2/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e701C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e701M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e702(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e702M : TinyExpressionP4Parser::e702C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16556:16559:body/2/3/literal", false, true, "text", false, K448);
    }
    private static Match e702C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e702M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e703(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e703M : TinyExpressionP4Parser::e703C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16560:16563:body/2/4/literal", false, true, "text", false, K448);
    }
    private static Match e703C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e703M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e704(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e704M : TinyExpressionP4Parser::e704C, K889, K879, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16564:16578:body/2/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e704C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e704M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e705(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e705M : TinyExpressionP4Parser::e705C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16585:16588:body/2/6/literal", false, true, "text", false, K880);
    }
    private static Match e705C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e705M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e706(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e706M : TinyExpressionP4Parser::e706C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16654:body/3/seq", false, false, "mixed", true, K155);
    }
    private static Match e706C(Session s, Frame f) {
        return s.sequence(f, K890, K367);
    }
    private static Match e706M(Session s, Frame f) {
        return s.sequence(f, K890, K367);
    }
    private static Match e707(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e707M : TinyExpressionP4Parser::e707C, K891, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16612:body/3/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e707C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e707M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e708(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e708M : TinyExpressionP4Parser::e708C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16620:16623:body/3/1/literal", false, true, "text", false, K872);
    }
    private static Match e708C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e708M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e709(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e709M : TinyExpressionP4Parser::e709C, K892, K875, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16624:16639:body/3/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e709C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e709M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e710(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e710M : TinyExpressionP4Parser::e710C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16647:16650:body/3/3/literal", false, true, "text", false, K448);
    }
    private static Match e710C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e710M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e711(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e711M : TinyExpressionP4Parser::e711C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16651:16654:body/3/4/literal", false, true, "text", false, K880);
    }
    private static Match e711C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e711M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e712(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e712M : TinyExpressionP4Parser::e712C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16741:body/4/seq", false, false, "mixed", true, K155);
    }
    private static Match e712C(Session s, Frame f) {
        return s.sequence(f, K893, K219);
    }
    private static Match e712M(Session s, Frame f) {
        return s.sequence(f, K893, K219);
    }
    private static Match e713(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e713M : TinyExpressionP4Parser::e713C, K894, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16678:body/4/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e713C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e713M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e714(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e714M : TinyExpressionP4Parser::e714C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16686:16689:body/4/1/literal", false, true, "text", false, K872);
    }
    private static Match e714C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e714M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e715(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e715M : TinyExpressionP4Parser::e715C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16690:16693:body/4/2/literal", false, true, "text", false, K448);
    }
    private static Match e715C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e715M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e716(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e716M : TinyExpressionP4Parser::e716C, K895, K877, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16694:16707:body/4/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e716C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e716M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e717(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e717M : TinyExpressionP4Parser::e717C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16713:16716:body/4/4/literal", false, true, "text", false, K448);
    }
    private static Match e717C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e717M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e718(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e718M : TinyExpressionP4Parser::e718C, K896, K879, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16717:16731:body/4/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e718C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e718M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e719(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e719M : TinyExpressionP4Parser::e719C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16738:16741:body/4/6/literal", false, true, "text", false, K880);
    }
    private static Match e719C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e719M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e720(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e720M : TinyExpressionP4Parser::e720C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16803:body/5/seq", false, false, "mixed", true, K155);
    }
    private static Match e720C(Session s, Frame f) {
        return s.sequence(f, K897, K367);
    }
    private static Match e720M(Session s, Frame f) {
        return s.sequence(f, K897, K367);
    }
    private static Match e721(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e721M : TinyExpressionP4Parser::e721C, K898, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16765:body/5/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e721C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e721M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e722(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e722M : TinyExpressionP4Parser::e722C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16773:16776:body/5/1/literal", false, true, "text", false, K872);
    }
    private static Match e722C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e722M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e723(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e723M : TinyExpressionP4Parser::e723C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16777:16780:body/5/2/literal", false, true, "text", false, K448);
    }
    private static Match e723C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e723M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e724(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e724M : TinyExpressionP4Parser::e724C, K899, K877, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16781:16794:body/5/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e724C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e724M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e725(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e725M : TinyExpressionP4Parser::e725C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16800:16803:body/5/4/literal", false, true, "text", false, K880);
    }
    private static Match e725C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e725M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e726(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e726M : TinyExpressionP4Parser::e726C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16871:body/6/seq", false, false, "mixed", true, K155);
    }
    private static Match e726C(Session s, Frame f) {
        return s.sequence(f, K900, K241);
    }
    private static Match e726M(Session s, Frame f) {
        return s.sequence(f, K900, K241);
    }
    private static Match e727(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e727M : TinyExpressionP4Parser::e727C, K901, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16827:body/6/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e727C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e727M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e728(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e728M : TinyExpressionP4Parser::e728C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16835:16838:body/6/1/literal", false, true, "text", false, K872);
    }
    private static Match e728C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e728M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e729(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e729M : TinyExpressionP4Parser::e729C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16839:16842:body/6/2/literal", false, true, "text", false, K448);
    }
    private static Match e729C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e729M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e730(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e730M : TinyExpressionP4Parser::e730C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16843:16846:body/6/3/literal", false, true, "text", false, K448);
    }
    private static Match e730C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e730M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e731(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e731M : TinyExpressionP4Parser::e731C, K902, K879, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16847:16861:body/6/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e731C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e731M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e732(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e732M : TinyExpressionP4Parser::e732C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16868:16871:body/6/5/literal", false, true, "text", false, K880);
    }
    private static Match e732C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e732M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e733(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e733M : TinyExpressionP4Parser::e733C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16914:body/7/seq", false, false, "mixed", false, K155);
    }
    private static Match e733C(Session s, Frame f) {
        return s.sequence(f, K903, K594);
    }
    private static Match e733M(Session s, Frame f) {
        return s.sequence(f, K903, K594);
    }
    private static Match e734(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e734M : TinyExpressionP4Parser::e734C, K904, K299, K224, K225, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16895:body/7/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e734C(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e734M(Session s, Frame f) {
        return parseSliceBaseReceiver_82(s, f);
    }
    private static Match e735(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e735M : TinyExpressionP4Parser::e735C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16903:16906:body/7/1/literal", false, true, "text", false, K872);
    }
    private static Match e735C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e735M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e736(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e736M : TinyExpressionP4Parser::e736C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16907:16910:body/7/2/literal", false, true, "text", false, K448);
    }
    private static Match e736C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e736M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e737(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e737M : TinyExpressionP4Parser::e737C, K215, K215, K215, K216, "TinyExpressionP4::SliceBaseExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16911:16914:body/7/3/literal", false, true, "text", false, K880);
    }
    private static Match e737C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e737M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e738(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e738M : TinyExpressionP4Parser::e738C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17629:body/choice", false, false, "node", true, K4);
    }
    private static Match e738C(Session s, Frame f) {
        return s.choice(f,K905,false,null,false);
    }
    private static Match e738M(Session s, Frame f) {
        return s.choice(f,K905,false,null,false);
    }
    private static Match e739(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e739M : TinyExpressionP4Parser::e739C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17113:body/0/seq", false, false, "node", true, K155);
    }
    private static Match e739C(Session s, Frame f) {
        return s.sequence(f, K906, K384);
    }
    private static Match e739M(Session s, Frame f) {
        return s.sequence(f, K906, K384);
    }
    private static Match e740(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e740M : TinyExpressionP4Parser::e740C, K907, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17027:body/0/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e740C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e740M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e741(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e741M : TinyExpressionP4Parser::e741C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17035:17038:body/0/1/literal", false, true, "text", false, K872);
    }
    private static Match e741C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e741M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e742(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e742M : TinyExpressionP4Parser::e742C, K908, K875, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17039:17054:body/0/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e742C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e742M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e743(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e743M : TinyExpressionP4Parser::e743C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17062:17065:body/0/3/literal", false, true, "text", false, K448);
    }
    private static Match e743C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e743M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e744(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e744M : TinyExpressionP4Parser::e744C, K909, K877, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17066:17079:body/0/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e744C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e744M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e745(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e745M : TinyExpressionP4Parser::e745C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17085:17088:body/0/5/literal", false, true, "text", false, K448);
    }
    private static Match e745C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e745M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e746(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e746M : TinyExpressionP4Parser::e746C, K910, K879, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17089:17103:body/0/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e746C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e746M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e747(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e747M : TinyExpressionP4Parser::e747C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17110:17113:body/0/7/literal", false, true, "text", false, K880);
    }
    private static Match e747C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e747M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e748(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e748M : TinyExpressionP4Parser::e748C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17200:body/1/seq", false, false, "node", true, K155);
    }
    private static Match e748C(Session s, Frame f) {
        return s.sequence(f, K911, K241);
    }
    private static Match e748M(Session s, Frame f) {
        return s.sequence(f, K911, K241);
    }
    private static Match e749(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e749M : TinyExpressionP4Parser::e749C, K912, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17139:body/1/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e749C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e749M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e750(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e750M : TinyExpressionP4Parser::e750C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17147:17150:body/1/1/literal", false, true, "text", false, K872);
    }
    private static Match e750C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e750M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e751(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e751M : TinyExpressionP4Parser::e751C, K913, K875, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17151:17166:body/1/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e751C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e751M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e752(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e752M : TinyExpressionP4Parser::e752C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17174:17177:body/1/3/literal", false, true, "text", false, K448);
    }
    private static Match e752C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e752M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e753(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e753M : TinyExpressionP4Parser::e753C, K914, K877, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17178:17191:body/1/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e753C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e753M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e754(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e754M : TinyExpressionP4Parser::e754C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17197:17200:body/1/5/literal", false, true, "text", false, K880);
    }
    private static Match e754C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e754M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e755(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e755M : TinyExpressionP4Parser::e755C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17293:body/2/seq", false, false, "node", true, K155);
    }
    private static Match e755C(Session s, Frame f) {
        return s.sequence(f, K915, K219);
    }
    private static Match e755M(Session s, Frame f) {
        return s.sequence(f, K915, K219);
    }
    private static Match e756(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e756M : TinyExpressionP4Parser::e756C, K916, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17226:body/2/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e756C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e756M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e757(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e757M : TinyExpressionP4Parser::e757C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17234:17237:body/2/1/literal", false, true, "text", false, K872);
    }
    private static Match e757C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e757M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e758(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e758M : TinyExpressionP4Parser::e758C, K917, K875, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17238:17253:body/2/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e758C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e758M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e759(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e759M : TinyExpressionP4Parser::e759C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17261:17264:body/2/3/literal", false, true, "text", false, K448);
    }
    private static Match e759C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e759M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e760(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e760M : TinyExpressionP4Parser::e760C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17265:17268:body/2/4/literal", false, true, "text", false, K448);
    }
    private static Match e760C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e760M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e761(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e761M : TinyExpressionP4Parser::e761C, K918, K879, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17269:17283:body/2/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e761C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e761M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e762(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e762M : TinyExpressionP4Parser::e762C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17290:17293:body/2/6/literal", false, true, "text", false, K880);
    }
    private static Match e762C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e762M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e763(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e763M : TinyExpressionP4Parser::e763C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17361:body/3/seq", false, false, "node", true, K155);
    }
    private static Match e763C(Session s, Frame f) {
        return s.sequence(f, K919, K367);
    }
    private static Match e763M(Session s, Frame f) {
        return s.sequence(f, K919, K367);
    }
    private static Match e764(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e764M : TinyExpressionP4Parser::e764C, K920, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17319:body/3/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e764C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e764M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e765(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e765M : TinyExpressionP4Parser::e765C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17327:17330:body/3/1/literal", false, true, "text", false, K872);
    }
    private static Match e765C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e765M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e766(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e766M : TinyExpressionP4Parser::e766C, K921, K875, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17331:17346:body/3/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e766C(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e766M(Session s, Frame f) {
        return parseSliceStartIndex_83(s, f);
    }
    private static Match e767(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e767M : TinyExpressionP4Parser::e767C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17354:17357:body/3/3/literal", false, true, "text", false, K448);
    }
    private static Match e767C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e767M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e768(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e768M : TinyExpressionP4Parser::e768C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17358:17361:body/3/4/literal", false, true, "text", false, K880);
    }
    private static Match e768C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e768M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e769(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e769M : TinyExpressionP4Parser::e769C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17450:body/4/seq", false, false, "node", true, K155);
    }
    private static Match e769C(Session s, Frame f) {
        return s.sequence(f, K922, K219);
    }
    private static Match e769M(Session s, Frame f) {
        return s.sequence(f, K922, K219);
    }
    private static Match e770(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e770M : TinyExpressionP4Parser::e770C, K923, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17387:body/4/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e770C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e770M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e771(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e771M : TinyExpressionP4Parser::e771C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17395:17398:body/4/1/literal", false, true, "text", false, K872);
    }
    private static Match e771C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e771M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e772(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e772M : TinyExpressionP4Parser::e772C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17399:17402:body/4/2/literal", false, true, "text", false, K448);
    }
    private static Match e772C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e772M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e773(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e773M : TinyExpressionP4Parser::e773C, K924, K877, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17403:17416:body/4/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e773C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e773M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e774(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e774M : TinyExpressionP4Parser::e774C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17422:17425:body/4/4/literal", false, true, "text", false, K448);
    }
    private static Match e774C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e774M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e775(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e775M : TinyExpressionP4Parser::e775C, K925, K879, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17426:17440:body/4/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e775C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e775M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e776(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e776M : TinyExpressionP4Parser::e776C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17447:17450:body/4/6/literal", false, true, "text", false, K880);
    }
    private static Match e776C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e776M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e777(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e777M : TinyExpressionP4Parser::e777C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17514:body/5/seq", false, false, "node", true, K155);
    }
    private static Match e777C(Session s, Frame f) {
        return s.sequence(f, K926, K367);
    }
    private static Match e777M(Session s, Frame f) {
        return s.sequence(f, K926, K367);
    }
    private static Match e778(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e778M : TinyExpressionP4Parser::e778C, K927, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17476:body/5/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e778C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e778M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e779(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e779M : TinyExpressionP4Parser::e779C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17484:17487:body/5/1/literal", false, true, "text", false, K872);
    }
    private static Match e779C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e779M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e780(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e780M : TinyExpressionP4Parser::e780C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17488:17491:body/5/2/literal", false, true, "text", false, K448);
    }
    private static Match e780C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e780M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e781(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e781M : TinyExpressionP4Parser::e781C, K928, K877, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17492:17505:body/5/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e781C(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e781M(Session s, Frame f) {
        return parseSliceEndIndex_84(s, f);
    }
    private static Match e782(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e782M : TinyExpressionP4Parser::e782C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17511:17514:body/5/4/literal", false, true, "text", false, K880);
    }
    private static Match e782C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e782M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e783(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e783M : TinyExpressionP4Parser::e783C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17584:body/6/seq", false, false, "node", true, K155);
    }
    private static Match e783C(Session s, Frame f) {
        return s.sequence(f, K929, K241);
    }
    private static Match e783M(Session s, Frame f) {
        return s.sequence(f, K929, K241);
    }
    private static Match e784(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e784M : TinyExpressionP4Parser::e784C, K930, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17540:body/6/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e784C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e784M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e785(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e785M : TinyExpressionP4Parser::e785C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17548:17551:body/6/1/literal", false, true, "text", false, K872);
    }
    private static Match e785C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e785M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e786(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e786M : TinyExpressionP4Parser::e786C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17552:17555:body/6/2/literal", false, true, "text", false, K448);
    }
    private static Match e786C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e786M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e787(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e787M : TinyExpressionP4Parser::e787C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17556:17559:body/6/3/literal", false, true, "text", false, K448);
    }
    private static Match e787C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e787M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e788(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e788M : TinyExpressionP4Parser::e788C, K931, K879, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17560:17574:body/6/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e788C(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e788M(Session s, Frame f) {
        return parseSliceStepIndex_85(s, f);
    }
    private static Match e789(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e789M : TinyExpressionP4Parser::e789C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17581:17584:body/6/5/literal", false, true, "text", false, K880);
    }
    private static Match e789C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e789M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e790(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e790M : TinyExpressionP4Parser::e790C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17629:body/7/seq", false, false, "node", false, K155);
    }
    private static Match e790C(Session s, Frame f) {
        return s.sequence(f, K932, K594);
    }
    private static Match e790M(Session s, Frame f) {
        return s.sequence(f, K932, K594);
    }
    private static Match e791(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e791M : TinyExpressionP4Parser::e791C, K933, K299, K224, K225, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17610:body/7/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e791C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e791M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e792(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e792M : TinyExpressionP4Parser::e792C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17618:17621:body/7/1/literal", false, true, "text", false, K872);
    }
    private static Match e792C(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e792M(Session s, Frame f) {
        return s.literal(f, "[", true, false, K873);
    }
    private static Match e793(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e793M : TinyExpressionP4Parser::e793C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17622:17625:body/7/2/literal", false, true, "text", false, K448);
    }
    private static Match e793C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e793M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e794(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e794M : TinyExpressionP4Parser::e794C, K215, K215, K215, K216, "TinyExpressionP4::SliceNestedExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17626:17629:body/7/3/literal", false, true, "text", false, K880);
    }
    private static Match e794C(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e794M(Session s, Frame f) {
        return s.literal(f, "]", true, false, K881);
    }
    private static Match e795(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e795M : TinyExpressionP4Parser::e795C, K215, K215, K215, K216, "TinyExpressionP4::SliceExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17655:17698:body/choice", false, false, "node", false, K4);
    }
    private static Match e795C(Session s, Frame f) {
        return s.choice(f,K934,false,null,false);
    }
    private static Match e795M(Session s, Frame f) {
        return s.choice(f,K934,false,null,false);
    }
    private static Match e796(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e796M : TinyExpressionP4Parser::e796C, K215, K215, K215, K216, "TinyExpressionP4::SliceExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17655:17676:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e796C(Session s, Frame f) {
        return parseSliceNestedExpression_87(s, f);
    }
    private static Match e796M(Session s, Frame f) {
        return parseSliceNestedExpression_87(s, f);
    }
    private static Match e797(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e797M : TinyExpressionP4Parser::e797C, K215, K215, K215, K216, "TinyExpressionP4::SliceExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17679:17698:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e797C(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e797M(Session s, Frame f) {
        return parseSliceBaseExpression_86(s, f);
    }
    private static Match e798(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e798M : TinyExpressionP4Parser::e798C, K215, K215, K215, K216, "TinyExpressionP4::StringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17945:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e798C(Session s, Frame f) {
        return s.sequence(f, K935, K249);
    }
    private static Match e798M(Session s, Frame f) {
        return s.sequence(f, K935, K249);
    }
    private static Match e799(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e799M : TinyExpressionP4Parser::e799C, K936, K518, K224, K225, "TinyExpressionP4::StringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17909:body/0/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e799C(Session s, Frame f) {
        return parseStringTerm_91(s, f);
    }
    private static Match e799M(Session s, Frame f) {
        return parseStringTerm_91(s, f);
    }
    private static Match e800(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e800M : TinyExpressionP4Parser::e800C, K215, K215, K215, K216, "TinyExpressionP4::StringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17916:17945:body/1/repeat", true, false, "mixed", true, K221);
    }
    private static Match e800C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e801, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e800M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e801, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e801(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e801M : TinyExpressionP4Parser::e801C, K215, K215, K215, K216, "TinyExpressionP4::StringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17936:body/1/0/seq", false, false, "mixed", false, K532);
    }
    private static Match e801C(Session s, Frame f) {
        return s.sequence(f, K937, K249);
    }
    private static Match e801M(Session s, Frame f) {
        return s.sequence(f, K937, K249);
    }
    private static Match e802(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e802M : TinyExpressionP4Parser::e802C, K938, K522, K224, K225, "TinyExpressionP4::StringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17921:body/1/0/0/literal", false, true, "text", false, K532);
    }
    private static Match e802C(Session s, Frame f) {
        return s.literal(f, "+", true, false, K533);
    }
    private static Match e802M(Session s, Frame f) {
        return s.literal(f, "+", true, false, K533);
    }
    private static Match e803(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e803M : TinyExpressionP4Parser::e803C, K939, K524, K224, K225, "TinyExpressionP4::StringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17926:17936:body/1/0/1/ruleRef", false, false, "mixed", false, K228);
    }
    private static Match e803C(Session s, Frame f) {
        return parseStringTerm_91(s, f);
    }
    private static Match e803M(Session s, Frame f) {
        return parseStringTerm_91(s, f);
    }
    private static Match e804(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e804M : TinyExpressionP4Parser::e804C, K215, K215, K215, K216, "TinyExpressionP4::ParenthesizedStringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17985:18009:body/seq", false, false, "node", false, K4);
    }
    private static Match e804C(Session s, Frame f) {
        return s.sequence(f, K940, K237);
    }
    private static Match e804M(Session s, Frame f) {
        return s.sequence(f, K940, K237);
    }
    private static Match e805(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e805M : TinyExpressionP4Parser::e805C, K215, K215, K215, K216, "TinyExpressionP4::ParenthesizedStringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17985:17988:body/0/literal", false, true, "text", false, K372);
    }
    private static Match e805C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e805M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e806(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e806M : TinyExpressionP4Parser::e806C, K215, K215, K215, K216, "TinyExpressionP4::ParenthesizedStringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17989:18005:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e806C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e806M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e807(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e807M : TinyExpressionP4Parser::e807C, K215, K215, K215, K216, "TinyExpressionP4::ParenthesizedStringExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18006:18009:body/2/literal", false, true, "text", false, K374);
    }
    private static Match e807C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e807M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e808(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e808M : TinyExpressionP4Parser::e808C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18216:18579:body/choice", false, false, "mixed", false, K4);
    }
    private static Match e808C(Session s, Frame f) {
        return s.choice(f,K950,false,null,false,K949);
    }
    private static Match e808M(Session s, Frame f) {
        return s.choice(f,K950,false,null,false,K949);
    }
    private static Match e809(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e809M : TinyExpressionP4Parser::e809C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18216:18237:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e809C(Session s, Frame f) {
        return parseStringMatchExpression_113(s, f);
    }
    private static Match e809M(Session s, Frame f) {
        return parseStringMatchExpression_113(s, f);
    }
    private static Match e810(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e810M : TinyExpressionP4Parser::e810C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18244:18256:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e810C(Session s, Frame f) {
        return parseIfExpression_106(s, f);
    }
    private static Match e810M(Session s, Frame f) {
        return parseIfExpression_106(s, f);
    }
    private static Match e811(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e811M : TinyExpressionP4Parser::e811C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18263:18281:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e811C(Session s, Frame f) {
        return parseStringCastVariable_92(s, f);
    }
    private static Match e811M(Session s, Frame f) {
        return parseStringCastVariable_92(s, f);
    }
    private static Match e812(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e812M : TinyExpressionP4Parser::e812C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18288:18307:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e812C(Session s, Frame f) {
        return parseStringTypedVariable_93(s, f);
    }
    private static Match e812M(Session s, Frame f) {
        return parseStringTypedVariable_93(s, f);
    }
    private static Match e813(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e813M : TinyExpressionP4Parser::e813C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18314:18329:body/4/ruleRef", false, false, "node", false, K4);
    }
    private static Match e813C(Session s, Frame f) {
        return parseSliceExpression_88(s, f);
    }
    private static Match e813M(Session s, Frame f) {
        return parseSliceExpression_88(s, f);
    }
    private static Match e814(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e814M : TinyExpressionP4Parser::e814C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18336:18365:body/5/ruleRef", false, false, "node", false, K4);
    }
    private static Match e814C(Session s, Frame f) {
        return parseParenthesizedStringExpression_90(s, f);
    }
    private static Match e814M(Session s, Frame f) {
        return parseParenthesizedStringExpression_90(s, f);
    }
    private static Match e815(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e815M : TinyExpressionP4Parser::e815C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18372:18396:body/6/ruleRef", false, false, "node", false, K4);
    }
    private static Match e815C(Session s, Frame f) {
        return parseExternalStringInvocation_33(s, f);
    }
    private static Match e815M(Session s, Frame f) {
        return parseExternalStringInvocation_33(s, f);
    }
    private static Match e816(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e816M : TinyExpressionP4Parser::e816C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18403:18422:body/7/ruleRef", false, false, "node", false, K4);
    }
    private static Match e816C(Session s, Frame f) {
        return parseToUpperCaseFunction_61(s, f);
    }
    private static Match e816M(Session s, Frame f) {
        return parseToUpperCaseFunction_61(s, f);
    }
    private static Match e817(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e817M : TinyExpressionP4Parser::e817C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18425:18444:body/8/ruleRef", false, false, "node", false, K4);
    }
    private static Match e817C(Session s, Frame f) {
        return parseToLowerCaseFunction_62(s, f);
    }
    private static Match e817M(Session s, Frame f) {
        return parseToLowerCaseFunction_62(s, f);
    }
    private static Match e818(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e818M : TinyExpressionP4Parser::e818C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18447:18459:body/9/ruleRef", false, false, "node", false, K4);
    }
    private static Match e818C(Session s, Frame f) {
        return parseTrimFunction_63(s, f);
    }
    private static Match e818M(Session s, Frame f) {
        return parseTrimFunction_63(s, f);
    }
    private static Match e819(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e819M : TinyExpressionP4Parser::e819C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18466:18486:body/10/ruleRef", false, false, "node", false, K4);
    }
    private static Match e819C(Session s, Frame f) {
        return parseToUpperCaseDotMethod_66(s, f);
    }
    private static Match e819M(Session s, Frame f) {
        return parseToUpperCaseDotMethod_66(s, f);
    }
    private static Match e820(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e820M : TinyExpressionP4Parser::e820C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18489:18509:body/11/ruleRef", false, false, "node", false, K4);
    }
    private static Match e820C(Session s, Frame f) {
        return parseToLowerCaseDotMethod_67(s, f);
    }
    private static Match e820M(Session s, Frame f) {
        return parseToLowerCaseDotMethod_67(s, f);
    }
    private static Match e821(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e821M : TinyExpressionP4Parser::e821C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18512:18525:body/12/ruleRef", false, false, "node", false, K4);
    }
    private static Match e821C(Session s, Frame f) {
        return parseTrimDotMethod_68(s, f);
    }
    private static Match e821M(Session s, Frame f) {
        return parseTrimDotMethod_68(s, f);
    }
    private static Match e822(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e822M : TinyExpressionP4Parser::e822C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18532:18538:body/13/tokenRef", false, true, "text", false, K4);
    }
    private static Match e822C(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::STRING",2);
    }
    private static Match e822M(Session s, Frame f) {
        return s.external(f,"TinyExpressionP4::STRING",2);
    }
    private static Match e823(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e823M : TinyExpressionP4Parser::e823C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18545:18556:body/14/ruleRef", false, false, "node", false, K4);
    }
    private static Match e823C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e823M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e824(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e824M : TinyExpressionP4Parser::e824C, K215, K215, K215, K216, "TinyExpressionP4::StringTerm", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18563:18579:body/15/ruleRef", false, false, "node", false, K4);
    }
    private static Match e824C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e824M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e825(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e825M : TinyExpressionP4Parser::e825C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18661:18707:body/seq", false, false, "text", false, K4);
    }
    private static Match e825C(Session s, Frame f) {
        return s.sequence(f, K951, K367);
    }
    private static Match e825M(Session s, Frame f) {
        return s.sequence(f, K951, K367);
    }
    private static Match e826(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e826M : TinyExpressionP4Parser::e826C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18661:18664:body/0/literal", false, true, "text", false, K372);
    }
    private static Match e826C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e826M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e827(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e827M : TinyExpressionP4Parser::e827C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18665:18688:body/1/group", false, false, "text", false, K347);
    }
    private static Match e827C(Session s, Frame f) {
        return e828(s, f);
    }
    private static Match e827M(Session s, Frame f) {
        return e828(s, f);
    }
    private static Match e828(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e828M : TinyExpressionP4Parser::e828C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18667:18686:body/1/0/choice", false, false, "text", false, K347);
    }
    private static Match e828C(Session s, Frame f) {
        return s.choice(f,K952,false,null,false);
    }
    private static Match e828M(Session s, Frame f) {
        return s.choice(f,K952,false,null,false);
    }
    private static Match e829(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e829M : TinyExpressionP4Parser::e829C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18667:18675:body/1/0/0/literal", false, true, "text", false, K51);
    }
    private static Match e829C(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e829M(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e830(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e830M : TinyExpressionP4Parser::e830C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18678:18686:body/1/0/1/literal", false, true, "text", false, K331);
    }
    private static Match e830C(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e830M(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e831(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e831M : TinyExpressionP4Parser::e831C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18689:18692:body/2/literal", false, true, "text", false, K374);
    }
    private static Match e831C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e831M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e832(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e832M : TinyExpressionP4Parser::e832C, K215, K215, K215, K216, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18693:18696:body/3/literal", false, true, "text", false, K49);
    }
    private static Match e832C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e832M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e833(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e833M : TinyExpressionP4Parser::e833C, K953, K454, K224, K225, "TinyExpressionP4::StringCastVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18697:18707:body/4/tokenRef", false, true, "text", false, K254);
    }
    private static Match e833C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e833M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e834(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e834M : TinyExpressionP4Parser::e834C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18797:18846:body/seq", false, false, "text", false, K4);
    }
    private static Match e834C(Session s, Frame f) {
        return s.sequence(f, K954, K594);
    }
    private static Match e834M(Session s, Frame f) {
        return s.sequence(f, K954, K594);
    }
    private static Match e835(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e835M : TinyExpressionP4Parser::e835C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18797:18800:body/0/literal", false, true, "text", false, K49);
    }
    private static Match e835C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e835M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e836(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e836M : TinyExpressionP4Parser::e836C, K955, K454, K224, K225, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18801:18811:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e836C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e836M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e837(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e837M : TinyExpressionP4Parser::e837C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18818:18822:body/2/literal", false, true, "text", false, K256);
    }
    private static Match e837C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e837M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e838(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e838M : TinyExpressionP4Parser::e838C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18823:18846:body/3/group", false, false, "text", false, K347);
    }
    private static Match e838C(Session s, Frame f) {
        return e839(s, f);
    }
    private static Match e838M(Session s, Frame f) {
        return e839(s, f);
    }
    private static Match e839(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e839M : TinyExpressionP4Parser::e839C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18825:18844:body/3/0/choice", false, false, "text", false, K347);
    }
    private static Match e839C(Session s, Frame f) {
        return s.choice(f,K956,false,null,false);
    }
    private static Match e839M(Session s, Frame f) {
        return s.choice(f,K956,false,null,false);
    }
    private static Match e840(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e840M : TinyExpressionP4Parser::e840C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18825:18833:body/3/0/0/literal", false, true, "text", false, K51);
    }
    private static Match e840C(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e840M(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e841(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e841M : TinyExpressionP4Parser::e841C, K215, K215, K215, K216, "TinyExpressionP4::StringTypedVariable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18836:18844:body/3/0/1/literal", false, true, "text", false, K331);
    }
    private static Match e841C(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e841M(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e842(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e842M : TinyExpressionP4Parser::e842C, K215, K215, K215, K216, "TinyExpressionP4::BooleanExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19143:body/seq", false, false, "node", true, K4);
    }
    private static Match e842C(Session s, Frame f) {
        return s.sequence(f, K957, K249);
    }
    private static Match e842M(Session s, Frame f) {
        return s.sequence(f, K957, K249);
    }
    private static Match e843(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e843M : TinyExpressionP4Parser::e843C, K958, K518, K224, K225, "TinyExpressionP4::BooleanExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19097:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e843C(Session s, Frame f) {
        return parseBooleanAndExpression_95(s, f);
    }
    private static Match e843M(Session s, Frame f) {
        return parseBooleanAndExpression_95(s, f);
    }
    private static Match e844(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e844M : TinyExpressionP4Parser::e844C, K215, K215, K215, K216, "TinyExpressionP4::BooleanExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19104:19143:body/1/repeat", true, false, "node", true, K221);
    }
    private static Match e844C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e845, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e844M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e845, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e845(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e845M : TinyExpressionP4Parser::e845C, K215, K215, K215, K216, "TinyExpressionP4::BooleanExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19134:body/1/0/seq", false, false, "node", false, K959);
    }
    private static Match e845C(Session s, Frame f) {
        return s.sequence(f, K960, K249);
    }
    private static Match e845M(Session s, Frame f) {
        return s.sequence(f, K960, K249);
    }
    private static Match e846(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e846M : TinyExpressionP4Parser::e846C, K961, K522, K224, K225, "TinyExpressionP4::BooleanExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19109:body/1/0/0/literal", false, true, "text", false, K959);
    }
    private static Match e846C(Session s, Frame f) {
        return s.literal(f, "|", true, false, K962);
    }
    private static Match e846M(Session s, Frame f) {
        return s.literal(f, "|", true, false, K962);
    }
    private static Match e847(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e847M : TinyExpressionP4Parser::e847C, K963, K524, K224, K225, "TinyExpressionP4::BooleanExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19114:19134:body/1/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e847C(Session s, Frame f) {
        return parseBooleanAndExpression_95(s, f);
    }
    private static Match e847M(Session s, Frame f) {
        return parseBooleanAndExpression_95(s, f);
    }
    private static Match e848(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e848M : TinyExpressionP4Parser::e848C, K215, K215, K215, K216, "TinyExpressionP4::BooleanAndExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19346:body/seq", false, false, "node", true, K4);
    }
    private static Match e848C(Session s, Frame f) {
        return s.sequence(f, K964, K249);
    }
    private static Match e848M(Session s, Frame f) {
        return s.sequence(f, K964, K249);
    }
    private static Match e849(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e849M : TinyExpressionP4Parser::e849C, K965, K518, K224, K225, "TinyExpressionP4::BooleanAndExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19300:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e849C(Session s, Frame f) {
        return parseBooleanXorExpression_96(s, f);
    }
    private static Match e849M(Session s, Frame f) {
        return parseBooleanXorExpression_96(s, f);
    }
    private static Match e850(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e850M : TinyExpressionP4Parser::e850C, K215, K215, K215, K216, "TinyExpressionP4::BooleanAndExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19307:19346:body/1/repeat", true, false, "node", true, K221);
    }
    private static Match e850C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e851, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e850M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e851, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e851(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e851M : TinyExpressionP4Parser::e851C, K215, K215, K215, K216, "TinyExpressionP4::BooleanAndExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19337:body/1/0/seq", false, false, "node", false, K966);
    }
    private static Match e851C(Session s, Frame f) {
        return s.sequence(f, K967, K249);
    }
    private static Match e851M(Session s, Frame f) {
        return s.sequence(f, K967, K249);
    }
    private static Match e852(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e852M : TinyExpressionP4Parser::e852C, K968, K522, K224, K225, "TinyExpressionP4::BooleanAndExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19312:body/1/0/0/literal", false, true, "text", false, K966);
    }
    private static Match e852C(Session s, Frame f) {
        return s.literal(f, "&", true, false, K969);
    }
    private static Match e852M(Session s, Frame f) {
        return s.literal(f, "&", true, false, K969);
    }
    private static Match e853(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e853M : TinyExpressionP4Parser::e853C, K970, K524, K224, K225, "TinyExpressionP4::BooleanAndExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19317:19337:body/1/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e853C(Session s, Frame f) {
        return parseBooleanXorExpression_96(s, f);
    }
    private static Match e853M(Session s, Frame f) {
        return parseBooleanXorExpression_96(s, f);
    }
    private static Match e854(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e854M : TinyExpressionP4Parser::e854C, K215, K215, K215, K216, "TinyExpressionP4::BooleanXorExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19535:body/seq", false, false, "node", true, K4);
    }
    private static Match e854C(Session s, Frame f) {
        return s.sequence(f, K971, K249);
    }
    private static Match e854M(Session s, Frame f) {
        return s.sequence(f, K971, K249);
    }
    private static Match e855(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e855M : TinyExpressionP4Parser::e855C, K972, K518, K224, K225, "TinyExpressionP4::BooleanXorExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19496:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e855C(Session s, Frame f) {
        return parseBooleanFactor_100(s, f);
    }
    private static Match e855M(Session s, Frame f) {
        return parseBooleanFactor_100(s, f);
    }
    private static Match e856(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e856M : TinyExpressionP4Parser::e856C, K215, K215, K215, K216, "TinyExpressionP4::BooleanXorExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19503:19535:body/1/repeat", true, false, "node", true, K221);
    }
    private static Match e856C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e857, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e856M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e857, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e857(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e857M : TinyExpressionP4Parser::e857C, K215, K215, K215, K216, "TinyExpressionP4::BooleanXorExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19526:body/1/0/seq", false, false, "node", false, K973);
    }
    private static Match e857C(Session s, Frame f) {
        return s.sequence(f, K974, K249);
    }
    private static Match e857M(Session s, Frame f) {
        return s.sequence(f, K974, K249);
    }
    private static Match e858(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e858M : TinyExpressionP4Parser::e858C, K975, K522, K224, K225, "TinyExpressionP4::BooleanXorExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19508:body/1/0/0/literal", false, true, "text", false, K973);
    }
    private static Match e858C(Session s, Frame f) {
        return s.literal(f, "^", true, false, K976);
    }
    private static Match e858M(Session s, Frame f) {
        return s.literal(f, "^", true, false, K976);
    }
    private static Match e859(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e859M : TinyExpressionP4Parser::e859C, K977, K524, K224, K225, "TinyExpressionP4::BooleanXorExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19513:19526:body/1/0/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e859C(Session s, Frame f) {
        return parseBooleanFactor_100(s, f);
    }
    private static Match e859M(Session s, Frame f) {
        return parseBooleanFactor_100(s, f);
    }
    private static Match e860(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e860M : TinyExpressionP4Parser::e860C, K215, K215, K215, K216, "TinyExpressionP4::NotExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19673:19711:body/seq", false, false, "node", false, K4);
    }
    private static Match e860C(Session s, Frame f) {
        return s.sequence(f, K978, K594);
    }
    private static Match e860M(Session s, Frame f) {
        return s.sequence(f, K978, K594);
    }
    private static Match e861(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e861M : TinyExpressionP4Parser::e861C, K215, K215, K215, K216, "TinyExpressionP4::NotExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19673:19678:body/0/literal", false, true, "text", false, K358);
    }
    private static Match e861C(Session s, Frame f) {
        return s.literal(f, "not", true, false, K359);
    }
    private static Match e861M(Session s, Frame f) {
        return s.literal(f, "not", true, false, K359);
    }
    private static Match e862(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e862M : TinyExpressionP4Parser::e862C, K215, K215, K215, K216, "TinyExpressionP4::NotExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19679:19682:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e862C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e862M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e863(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e863M : TinyExpressionP4Parser::e863C, K979, K299, K224, K225, "TinyExpressionP4::NotExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19683:19700:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e863C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e863M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e864(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e864M : TinyExpressionP4Parser::e864C, K215, K215, K215, K216, "TinyExpressionP4::NotExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19708:19711:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e864C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e864M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e865(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e865M : TinyExpressionP4Parser::e865C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19745:20177:body/choice", false, false, "mixed", false, K4);
    }
    private static Match e865C(Session s, Frame f) {
        return s.choice(f,K1018,false,null,false,K1017);
    }
    private static Match e865M(Session s, Frame f) {
        return s.choice(f,K1018,false,null,false,K1017);
    }
    private static Match e866(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e866M : TinyExpressionP4Parser::e866C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19745:19758:body/0/ruleRef", false, false, "node", false, K4);
    }
    private static Match e866C(Session s, Frame f) {
        return parseNotExpression_97(s, f);
    }
    private static Match e866M(Session s, Frame f) {
        return parseNotExpression_97(s, f);
    }
    private static Match e867(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e867M : TinyExpressionP4Parser::e867C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19765:19777:body/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e867C(Session s, Frame f) {
        return parseIfExpression_106(s, f);
    }
    private static Match e867M(Session s, Frame f) {
        return parseIfExpression_106(s, f);
    }
    private static Match e868(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e868M : TinyExpressionP4Parser::e868C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19784:19806:body/2/ruleRef", false, false, "node", false, K4);
    }
    private static Match e868C(Session s, Frame f) {
        return parseBooleanMatchExpression_117(s, f);
    }
    private static Match e868M(Session s, Frame f) {
        return parseBooleanMatchExpression_117(s, f);
    }
    private static Match e869(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e869M : TinyExpressionP4Parser::e869C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19813:19838:body/3/ruleRef", false, false, "node", false, K4);
    }
    private static Match e869C(Session s, Frame f) {
        return parseExternalBooleanInvocation_31(s, f);
    }
    private static Match e869M(Session s, Frame f) {
        return parseExternalBooleanInvocation_31(s, f);
    }
    private static Match e870(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e870M : TinyExpressionP4Parser::e870C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19845:19853:body/4/ruleRef", false, false, "node", false, K4);
    }
    private static Match e870C(Session s, Frame f) {
        return parseInMethod_73(s, f);
    }
    private static Match e870M(Session s, Frame f) {
        return parseInMethod_73(s, f);
    }
    private static Match e871(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e871M : TinyExpressionP4Parser::e871C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19860:19879:body/5/ruleRef", false, false, "node", false, K4);
    }
    private static Match e871C(Session s, Frame f) {
        return parseStartsWithDotMethod_74(s, f);
    }
    private static Match e871M(Session s, Frame f) {
        return parseStartsWithDotMethod_74(s, f);
    }
    private static Match e872(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e872M : TinyExpressionP4Parser::e872C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19886:19903:body/6/ruleRef", false, false, "node", false, K4);
    }
    private static Match e872C(Session s, Frame f) {
        return parseEndsWithDotMethod_75(s, f);
    }
    private static Match e872M(Session s, Frame f) {
        return parseEndsWithDotMethod_75(s, f);
    }
    private static Match e873(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e873M : TinyExpressionP4Parser::e873C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19910:19927:body/7/ruleRef", false, false, "node", false, K4);
    }
    private static Match e873C(Session s, Frame f) {
        return parseContainsDotMethod_76(s, f);
    }
    private static Match e873M(Session s, Frame f) {
        return parseContainsDotMethod_76(s, f);
    }
    private static Match e874(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e874M : TinyExpressionP4Parser::e874C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19934:19952:body/8/ruleRef", false, false, "node", false, K4);
    }
    private static Match e874C(Session s, Frame f) {
        return parseStartsWithFunction_70(s, f);
    }
    private static Match e874M(Session s, Frame f) {
        return parseStartsWithFunction_70(s, f);
    }
    private static Match e875(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e875M : TinyExpressionP4Parser::e875C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19959:19975:body/9/ruleRef", false, false, "node", false, K4);
    }
    private static Match e875C(Session s, Frame f) {
        return parseEndsWithFunction_71(s, f);
    }
    private static Match e875M(Session s, Frame f) {
        return parseEndsWithFunction_71(s, f);
    }
    private static Match e876(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e876M : TinyExpressionP4Parser::e876C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19982:19998:body/10/ruleRef", false, false, "node", false, K4);
    }
    private static Match e876C(Session s, Frame f) {
        return parseContainsFunction_72(s, f);
    }
    private static Match e876M(Session s, Frame f) {
        return parseContainsFunction_72(s, f);
    }
    private static Match e877(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e877M : TinyExpressionP4Parser::e877C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20005:20022:body/11/ruleRef", false, false, "node", false, K4);
    }
    private static Match e877C(Session s, Frame f) {
        return parseIsPresentFunction_78(s, f);
    }
    private static Match e877M(Session s, Frame f) {
        return parseIsPresentFunction_78(s, f);
    }
    private static Match e878(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e878M : TinyExpressionP4Parser::e878C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20029:20048:body/12/ruleRef", false, false, "node", false, K4);
    }
    private static Match e878C(Session s, Frame f) {
        return parseInTimeRangeFunction_79(s, f);
    }
    private static Match e878M(Session s, Frame f) {
        return parseInTimeRangeFunction_79(s, f);
    }
    private static Match e879(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e879M : TinyExpressionP4Parser::e879C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20055:20077:body/13/ruleRef", false, false, "node", false, K4);
    }
    private static Match e879C(Session s, Frame f) {
        return parseInDayTimeRangeFunction_80(s, f);
    }
    private static Match e879M(Session s, Frame f) {
        return parseInDayTimeRangeFunction_80(s, f);
    }
    private static Match e880(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e880M : TinyExpressionP4Parser::e880C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20084:20090:body/14/literal", false, true, "text", false, K1019);
    }
    private static Match e880C(Session s, Frame f) {
        return s.literal(f, "true", true, false, K1020);
    }
    private static Match e880M(Session s, Frame f) {
        return s.literal(f, "true", true, false, K1020);
    }
    private static Match e881(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e881M : TinyExpressionP4Parser::e881C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20097:20104:body/15/literal", false, true, "text", false, K1021);
    }
    private static Match e881C(Session s, Frame f) {
        return s.literal(f, "false", true, false, K1022);
    }
    private static Match e881M(Session s, Frame f) {
        return s.literal(f, "false", true, false, K1022);
    }
    private static Match e882(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e882M : TinyExpressionP4Parser::e882C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20111:20122:body/16/ruleRef", false, false, "node", false, K4);
    }
    private static Match e882C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e882M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e883(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e883M : TinyExpressionP4Parser::e883C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20129:20145:body/17/ruleRef", false, false, "node", false, K4);
    }
    private static Match e883C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e883M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e884(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e884M : TinyExpressionP4Parser::e884C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20152:20177:body/18/seq", false, false, "node", false, K160);
    }
    private static Match e884C(Session s, Frame f) {
        return s.sequence(f, K1023, K237);
    }
    private static Match e884M(Session s, Frame f) {
        return s.sequence(f, K1023, K237);
    }
    private static Match e885(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e885M : TinyExpressionP4Parser::e885C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20152:20155:body/18/0/literal", false, true, "text", false, K372);
    }
    private static Match e885C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e885M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e886(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e886M : TinyExpressionP4Parser::e886C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20156:20173:body/18/1/ruleRef", false, false, "node", false, K4);
    }
    private static Match e886C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e886M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e887(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e887M : TinyExpressionP4Parser::e887C, K215, K215, K215, K216, "TinyExpressionP4::BooleanComparable", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20174:20177:body/18/2/literal", false, true, "text", false, K374);
    }
    private static Match e887C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e887M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e888(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e888M : TinyExpressionP4Parser::e888C, K215, K215, K215, K216, "TinyExpressionP4::BooleanEqualityExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20327:body/seq", false, false, "mixed", true, K4);
    }
    private static Match e888C(Session s, Frame f) {
        return s.sequence(f, K1024, K237);
    }
    private static Match e888M(Session s, Frame f) {
        return s.sequence(f, K1024, K237);
    }
    private static Match e889(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e889M : TinyExpressionP4Parser::e889C, K1025, K518, K224, K225, "TinyExpressionP4::BooleanEqualityExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20288:body/0/ruleRef", false, false, "mixed", false, K1026);
    }
    private static Match e889C(Session s, Frame f) {
        return parseBooleanComparable_98(s, f);
    }
    private static Match e889M(Session s, Frame f) {
        return parseBooleanComparable_98(s, f);
    }
    private static Match e890(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e890M : TinyExpressionP4Parser::e890C, K1027, K522, K224, K225, "TinyExpressionP4::BooleanEqualityExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20295:20305:body/1/ruleRef", false, false, "text", false, K181);
    }
    private static Match e890C(Session s, Frame f) {
        return parseEqualityOp_102(s, f);
    }
    private static Match e890M(Session s, Frame f) {
        return parseEqualityOp_102(s, f);
    }
    private static Match e891(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e891M : TinyExpressionP4Parser::e891C, K1028, K524, K224, K225, "TinyExpressionP4::BooleanEqualityExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20310:20327:body/2/ruleRef", false, false, "mixed", false, K1026);
    }
    private static Match e891C(Session s, Frame f) {
        return parseBooleanComparable_98(s, f);
    }
    private static Match e891M(Session s, Frame f) {
        return parseBooleanComparable_98(s, f);
    }
    private static Match e892(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e892M : TinyExpressionP4Parser::e892C, K215, K215, K215, K216, "TinyExpressionP4::BooleanFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20540:body/choice", false, false, "mixed", false, K4);
    }
    private static Match e892C(Session s, Frame f) {
        return s.choice(f,K1029,false,null,false);
    }
    private static Match e892M(Session s, Frame f) {
        return s.choice(f,K1029,false,null,false);
    }
    private static Match e893(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e893M : TinyExpressionP4Parser::e893C, K1030, K299, K224, K225, "TinyExpressionP4::BooleanFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20435:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e893C(Session s, Frame f) {
        return parseBooleanEqualityExpression_99(s, f);
    }
    private static Match e893M(Session s, Frame f) {
        return parseBooleanEqualityExpression_99(s, f);
    }
    private static Match e894(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e894M : TinyExpressionP4Parser::e894C, K1031, K299, K224, K225, "TinyExpressionP4::BooleanFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20449:20469:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e894C(Session s, Frame f) {
        return parseComparisonExpression_103(s, f);
    }
    private static Match e894M(Session s, Frame f) {
        return parseComparisonExpression_103(s, f);
    }
    private static Match e895(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e895M : TinyExpressionP4Parser::e895C, K1032, K299, K224, K225, "TinyExpressionP4::BooleanFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20483:20509:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e895C(Session s, Frame f) {
        return parseStringComparisonExpression_101(s, f);
    }
    private static Match e895M(Session s, Frame f) {
        return parseStringComparisonExpression_101(s, f);
    }
    private static Match e896(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e896M : TinyExpressionP4Parser::e896C, K1033, K299, K224, K225, "TinyExpressionP4::BooleanFactor", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20523:20540:body/3/ruleRef", false, false, "mixed", false, K1026);
    }
    private static Match e896C(Session s, Frame f) {
        return parseBooleanComparable_98(s, f);
    }
    private static Match e896M(Session s, Frame f) {
        return parseBooleanComparable_98(s, f);
    }
    private static Match e897(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e897M : TinyExpressionP4Parser::e897C, K215, K215, K215, K216, "TinyExpressionP4::StringComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20858:body/seq", false, false, "node", true, K4);
    }
    private static Match e897C(Session s, Frame f) {
        return s.sequence(f, K1034, K237);
    }
    private static Match e897M(Session s, Frame f) {
        return s.sequence(f, K1034, K237);
    }
    private static Match e898(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e898M : TinyExpressionP4Parser::e898C, K1035, K518, K224, K225, "TinyExpressionP4::StringComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20820:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e898C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e898M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e899(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e899M : TinyExpressionP4Parser::e899C, K1036, K522, K224, K225, "TinyExpressionP4::StringComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20827:20837:body/1/ruleRef", false, false, "text", false, K181);
    }
    private static Match e899C(Session s, Frame f) {
        return parseEqualityOp_102(s, f);
    }
    private static Match e899M(Session s, Frame f) {
        return parseEqualityOp_102(s, f);
    }
    private static Match e900(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e900M : TinyExpressionP4Parser::e900C, K1037, K524, K224, K225, "TinyExpressionP4::StringComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20842:20858:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e900C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e900M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e901(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e901M : TinyExpressionP4Parser::e901C, K215, K215, K215, K216, "TinyExpressionP4::EqualityOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20885:20896:body/choice", false, false, "text", false, K4);
    }
    private static Match e901C(Session s, Frame f) {
        return s.choice(f,K1038,false,null,false);
    }
    private static Match e901M(Session s, Frame f) {
        return s.choice(f,K1038,false,null,false);
    }
    private static Match e902(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e902M : TinyExpressionP4Parser::e902C, K215, K215, K215, K216, "TinyExpressionP4::EqualityOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20885:20889:body/0/literal", false, true, "text", false, K1039);
    }
    private static Match e902C(Session s, Frame f) {
        return s.literal(f, "==", true, false, K1040);
    }
    private static Match e902M(Session s, Frame f) {
        return s.literal(f, "==", true, false, K1040);
    }
    private static Match e903(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e903M : TinyExpressionP4Parser::e903C, K215, K215, K215, K216, "TinyExpressionP4::EqualityOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20892:20896:body/1/literal", false, true, "text", false, K1041);
    }
    private static Match e903C(Session s, Frame f) {
        return s.literal(f, "!=", true, false, K1042);
    }
    private static Match e903M(Session s, Frame f) {
        return s.literal(f, "!=", true, false, K1042);
    }
    private static Match e904(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e904M : TinyExpressionP4Parser::e904C, K215, K215, K215, K216, "TinyExpressionP4::ComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:21033:body/seq", false, false, "node", true, K4);
    }
    private static Match e904C(Session s, Frame f) {
        return s.sequence(f, K1043, K237);
    }
    private static Match e904M(Session s, Frame f) {
        return s.sequence(f, K1043, K237);
    }
    private static Match e905(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e905M : TinyExpressionP4Parser::e905C, K1044, K518, K224, K225, "TinyExpressionP4::ComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:20996:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e905C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e905M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e906(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e906M : TinyExpressionP4Parser::e906C, K1045, K522, K224, K225, "TinyExpressionP4::ComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21003:21012:body/1/ruleRef", false, false, "text", false, K184);
    }
    private static Match e906C(Session s, Frame f) {
        return parseCompareOp_104(s, f);
    }
    private static Match e906M(Session s, Frame f) {
        return parseCompareOp_104(s, f);
    }
    private static Match e907(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e907M : TinyExpressionP4Parser::e907C, K1046, K524, K224, K225, "TinyExpressionP4::ComparisonExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21017:21033:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e907C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e907M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e908(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e908M : TinyExpressionP4Parser::e908C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21059:21096:body/choice", false, false, "text", false, K4);
    }
    private static Match e908C(Session s, Frame f) {
        return s.choice(f,K1047,false,null,false);
    }
    private static Match e908M(Session s, Frame f) {
        return s.choice(f,K1047,false,null,false);
    }
    private static Match e909(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e909M : TinyExpressionP4Parser::e909C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21059:21063:body/0/literal", false, true, "text", false, K1039);
    }
    private static Match e909C(Session s, Frame f) {
        return s.literal(f, "==", true, false, K1040);
    }
    private static Match e909M(Session s, Frame f) {
        return s.literal(f, "==", true, false, K1040);
    }
    private static Match e910(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e910M : TinyExpressionP4Parser::e910C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21066:21070:body/1/literal", false, true, "text", false, K1041);
    }
    private static Match e910C(Session s, Frame f) {
        return s.literal(f, "!=", true, false, K1042);
    }
    private static Match e910M(Session s, Frame f) {
        return s.literal(f, "!=", true, false, K1042);
    }
    private static Match e911(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e911M : TinyExpressionP4Parser::e911C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21073:21077:body/2/literal", false, true, "text", false, K1048);
    }
    private static Match e911C(Session s, Frame f) {
        return s.literal(f, "<=", true, false, K1049);
    }
    private static Match e911M(Session s, Frame f) {
        return s.literal(f, "<=", true, false, K1049);
    }
    private static Match e912(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e912M : TinyExpressionP4Parser::e912C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21080:21084:body/3/literal", false, true, "text", false, K1050);
    }
    private static Match e912C(Session s, Frame f) {
        return s.literal(f, ">=", true, false, K1051);
    }
    private static Match e912M(Session s, Frame f) {
        return s.literal(f, ">=", true, false, K1051);
    }
    private static Match e913(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e913M : TinyExpressionP4Parser::e913C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21087:21090:body/4/literal", false, true, "text", false, K1052);
    }
    private static Match e913C(Session s, Frame f) {
        return s.literal(f, "<", true, false, K1053);
    }
    private static Match e913M(Session s, Frame f) {
        return s.literal(f, "<", true, false, K1053);
    }
    private static Match e914(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e914M : TinyExpressionP4Parser::e914C, K215, K215, K215, K216, "TinyExpressionP4::CompareOp", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21093:21096:body/5/literal", false, true, "text", false, K1054);
    }
    private static Match e914C(Session s, Frame f) {
        return s.literal(f, ">", true, false, K1055);
    }
    private static Match e914M(Session s, Frame f) {
        return s.literal(f, ">", true, false, K1055);
    }
    private static Match e915(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e915M : TinyExpressionP4Parser::e915C, K215, K215, K215, K216, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21420:body/choice", false, false, "node", false, K4);
    }
    private static Match e915C(Session s, Frame f) {
        return s.choice(f,K1060,false,null,false,K1059);
    }
    private static Match e915M(Session s, Frame f) {
        return s.choice(f,K1060,false,null,false,K1059);
    }
    private static Match e916(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e916M : TinyExpressionP4Parser::e916C, K1061, K299, K224, K225, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21266:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e916C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e916M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e917(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e917M : TinyExpressionP4Parser::e917C, K1062, K299, K224, K225, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21280:21296:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e917C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e917M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e918(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e918M : TinyExpressionP4Parser::e918C, K1063, K299, K224, K225, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21310:21327:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e918C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e918M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e919(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e919M : TinyExpressionP4Parser::e919C, K1064, K299, K224, K225, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21341:21365:body/3/ruleRef", false, false, "node", false, K58);
    }
    private static Match e919C(Session s, Frame f) {
        return parseExternalObjectInvocation_34(s, f);
    }
    private static Match e919M(Session s, Frame f) {
        return parseExternalObjectInvocation_34(s, f);
    }
    private static Match e920(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e920M : TinyExpressionP4Parser::e920C, K1065, K299, K224, K225, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21379:21390:body/4/ruleRef", false, false, "node", false, K49);
    }
    private static Match e920C(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e920M(Session s, Frame f) {
        return parseVariableRef_121(s, f);
    }
    private static Match e921(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e921M : TinyExpressionP4Parser::e921C, K1066, K299, K224, K225, "TinyExpressionP4::ObjectExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21404:21420:body/5/ruleRef", false, false, "node", false, K160);
    }
    private static Match e921C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e921M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e922(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e922M : TinyExpressionP4Parser::e922C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21595:21725:body/seq", false, false, "node", true, K4);
    }
    private static Match e922C(Session s, Frame f) {
        return s.sequence(f, K1069, K1068);
    }
    private static Match e922M(Session s, Frame f) {
        return s.sequence(f, K1069, K1068);
    }
    private static Match e923(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e923M : TinyExpressionP4Parser::e923C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21595:21599:body/0/literal", false, true, "text", false, K356);
    }
    private static Match e923C(Session s, Frame f) {
        return s.literal(f, "if", true, false, K357);
    }
    private static Match e923M(Session s, Frame f) {
        return s.literal(f, "if", true, false, K357);
    }
    private static Match e924(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e924M : TinyExpressionP4Parser::e924C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21600:21603:body/1/literal", false, true, "text", false, K372);
    }
    private static Match e924C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e924M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e925(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e925M : TinyExpressionP4Parser::e925C, K1070, K500, K224, K225, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21604:21621:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e925C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e925M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e926(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e926M : TinyExpressionP4Parser::e926C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21633:21636:body/3/literal", false, true, "text", false, K374);
    }
    private static Match e926C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e926M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e927(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e927M : TinyExpressionP4Parser::e927C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21641:21644:body/4/literal", false, true, "text", false, K390);
    }
    private static Match e927C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e927M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e928(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e928M : TinyExpressionP4Parser::e928C, K1071, K504, K224, K225, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21645:21661:body/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e928C(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e928M(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e929(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e929M : TinyExpressionP4Parser::e929C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21672:21675:body/6/literal", false, true, "text", false, K393);
    }
    private static Match e929C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e929M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e930(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e930M : TinyExpressionP4Parser::e930C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21680:21686:body/7/literal", false, true, "text", false, K1072);
    }
    private static Match e930C(Session s, Frame f) {
        return s.literal(f, "else", true, false, K1073);
    }
    private static Match e930M(Session s, Frame f) {
        return s.literal(f, "else", true, false, K1073);
    }
    private static Match e931(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e931M : TinyExpressionP4Parser::e931C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21691:21694:body/8/literal", false, true, "text", false, K390);
    }
    private static Match e931C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e931M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e932(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e932M : TinyExpressionP4Parser::e932C, K1074, K506, K224, K225, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21695:21711:body/9/ruleRef", false, false, "node", false, K228);
    }
    private static Match e932C(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e932M(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e933(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e933M : TinyExpressionP4Parser::e933C, K215, K215, K215, K216, "TinyExpressionP4::IfExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21722:21725:body/10/literal", false, true, "text", false, K393);
    }
    private static Match e933C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e933M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e934(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e934M : TinyExpressionP4Parser::e934C, K215, K215, K215, K216, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:22057:body/choice", false, false, "node", false, K4);
    }
    private static Match e934C(Session s, Frame f) {
        return s.choice(f,K1076,false,null,false,K1075);
    }
    private static Match e934M(Session s, Frame f) {
        return s.choice(f,K1076,false,null,false,K1075);
    }
    private static Match e935(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e935M : TinyExpressionP4Parser::e935C, K1077, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:21827:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e935C(Session s, Frame f) {
        return parseComparisonExpression_103(s, f);
    }
    private static Match e935M(Session s, Frame f) {
        return parseComparisonExpression_103(s, f);
    }
    private static Match e936(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e936M : TinyExpressionP4Parser::e936C, K1078, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21841:21867:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e936C(Session s, Frame f) {
        return parseStringComparisonExpression_101(s, f);
    }
    private static Match e936M(Session s, Frame f) {
        return parseStringComparisonExpression_101(s, f);
    }
    private static Match e937(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e937M : TinyExpressionP4Parser::e937C, K1079, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21881:21906:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e937C(Session s, Frame f) {
        return parseBooleanEqualityExpression_99(s, f);
    }
    private static Match e937M(Session s, Frame f) {
        return parseBooleanEqualityExpression_99(s, f);
    }
    private static Match e938(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e938M : TinyExpressionP4Parser::e938C, K1080, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21920:21936:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e938C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e938M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e939(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e939M : TinyExpressionP4Parser::e939C, K1081, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21950:21967:body/4/ruleRef", false, false, "node", false, K228);
    }
    private static Match e939C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e939M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e940(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e940M : TinyExpressionP4Parser::e940C, K1082, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21981:21997:body/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e940C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e940M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e941(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e941M : TinyExpressionP4Parser::e941C, K1083, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22011:22027:body/6/ruleRef", false, false, "node", false, K228);
    }
    private static Match e941C(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e941M(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e942(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e942M : TinyExpressionP4Parser::e942C, K1084, K299, K224, K225, "TinyExpressionP4::BranchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22041:22057:body/7/ruleRef", false, false, "node", false, K160);
    }
    private static Match e942C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e942M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e943(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e943M : TinyExpressionP4Parser::e943C, K215, K215, K215, K216, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22232:22330:body/seq", false, false, "node", true, K4);
    }
    private static Match e943C(Session s, Frame f) {
        return s.sequence(f, K1085, K219);
    }
    private static Match e943M(Session s, Frame f) {
        return s.sequence(f, K1085, K219);
    }
    private static Match e944(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e944M : TinyExpressionP4Parser::e944C, K215, K215, K215, K216, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22232:22235:body/0/literal", false, true, "text", false, K372);
    }
    private static Match e944C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e944M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e945(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e945M : TinyExpressionP4Parser::e945C, K1086, K500, K224, K225, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22236:22253:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e945C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e945M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e946(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e946M : TinyExpressionP4Parser::e946C, K215, K215, K215, K216, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22265:22268:body/2/literal", false, true, "text", false, K501);
    }
    private static Match e946C(Session s, Frame f) {
        return s.literal(f, "?", true, false, K502);
    }
    private static Match e946M(Session s, Frame f) {
        return s.literal(f, "?", true, false, K502);
    }
    private static Match e947(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e947M : TinyExpressionP4Parser::e947C, K1087, K504, K224, K225, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22269:22285:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e947C(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e947M(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e948(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e948M : TinyExpressionP4Parser::e948C, K215, K215, K215, K216, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22296:22299:body/4/literal", false, true, "text", false, K448);
    }
    private static Match e948C(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e948M(Session s, Frame f) {
        return s.literal(f, ":", true, false, K449);
    }
    private static Match e949(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e949M : TinyExpressionP4Parser::e949C, K1088, K506, K224, K225, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22300:22316:body/5/ruleRef", false, false, "node", false, K228);
    }
    private static Match e949C(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e949M(Session s, Frame f) {
        return parseBranchExpression_107(s, f);
    }
    private static Match e950(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e950M : TinyExpressionP4Parser::e950C, K215, K215, K215, K216, "TinyExpressionP4::TernaryExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22327:22330:body/6/literal", false, true, "text", false, K374);
    }
    private static Match e950C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e950M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e951(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e951M : TinyExpressionP4Parser::e951C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22520:22638:body/seq", false, false, "node", true, K4);
    }
    private static Match e951C(Session s, Frame f) {
        return s.sequence(f, K1089, K219);
    }
    private static Match e951M(Session s, Frame f) {
        return s.sequence(f, K1089, K219);
    }
    private static Match e952(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e952M : TinyExpressionP4Parser::e952C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22520:22527:body/0/literal", false, true, "text", false, K1090);
    }
    private static Match e952C(Session s, Frame f) {
        return s.literal(f, "match", true, false, K1091);
    }
    private static Match e952M(Session s, Frame f) {
        return s.literal(f, "match", true, false, K1091);
    }
    private static Match e953(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e953M : TinyExpressionP4Parser::e953C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22528:22531:body/1/literal", false, true, "text", false, K390);
    }
    private static Match e953C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e953M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e954(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e954M : TinyExpressionP4Parser::e954C, K1092, K1093, K224, K225, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22538:22548:body/2/ruleRef", false, false, "node", false, K196);
    }
    private static Match e954C(Session s, Frame f) {
        return parseNumberCase_110(s, f);
    }
    private static Match e954M(Session s, Frame f) {
        return parseNumberCase_110(s, f);
    }
    private static Match e955(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e955M : TinyExpressionP4Parser::e955C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22560:22589:body/3/repeat", true, false, "node", true, K377);
    }
    private static Match e955C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e956, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e955M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e956, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e956(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e956M : TinyExpressionP4Parser::e956C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22562:22576:body/3/0/seq", false, false, "node", false, K377);
    }
    private static Match e956C(Session s, Frame f) {
        return s.sequence(f, K1094, K249);
    }
    private static Match e956M(Session s, Frame f) {
        return s.sequence(f, K1094, K249);
    }
    private static Match e957(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e957M : TinyExpressionP4Parser::e957C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22562:22565:body/3/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e957C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e957M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e958(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e958M : TinyExpressionP4Parser::e958C, K1095, K1096, K224, K225, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22566:22576:body/3/0/1/ruleRef", false, false, "node", false, K196);
    }
    private static Match e958C(Session s, Frame f) {
        return parseNumberCase_110(s, f);
    }
    private static Match e958M(Session s, Frame f) {
        return parseNumberCase_110(s, f);
    }
    private static Match e959(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e959M : TinyExpressionP4Parser::e959C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22596:22599:body/4/literal", false, true, "text", false, K377);
    }
    private static Match e959C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e959M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e960(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e960M : TinyExpressionP4Parser::e960C, K1097, K1098, K224, K225, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22600:22617:body/5/ruleRef", false, false, "node", false, K198);
    }
    private static Match e960C(Session s, Frame f) {
        return parseNumberDefaultCase_111(s, f);
    }
    private static Match e960M(Session s, Frame f) {
        return parseNumberDefaultCase_111(s, f);
    }
    private static Match e961(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e961M : TinyExpressionP4Parser::e961C, K215, K215, K215, K216, "TinyExpressionP4::NumberMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22635:22638:body/6/literal", false, true, "text", false, K393);
    }
    private static Match e961C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e961M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e962(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e962M : TinyExpressionP4Parser::e962C, K215, K215, K215, K216, "TinyExpressionP4::NumberCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22762:body/seq", false, false, "node", true, K4);
    }
    private static Match e962C(Session s, Frame f) {
        return s.sequence(f, K1099, K237);
    }
    private static Match e962M(Session s, Frame f) {
        return s.sequence(f, K1099, K237);
    }
    private static Match e963(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e963M : TinyExpressionP4Parser::e963C, K1100, K500, K224, K225, "TinyExpressionP4::NumberCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22730:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e963C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e963M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e964(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e964M : TinyExpressionP4Parser::e964C, K215, K215, K215, K216, "TinyExpressionP4::NumberCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22742:22746:body/1/literal", false, true, "text", false, K196);
    }
    private static Match e964C(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e964M(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e965(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e965M : TinyExpressionP4Parser::e965C, K1102, K299, K224, K225, "TinyExpressionP4::NumberCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22747:22762:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e965C(Session s, Frame f) {
        return parseNumberCaseValue_112(s, f);
    }
    private static Match e965M(Session s, Frame f) {
        return parseNumberCaseValue_112(s, f);
    }
    private static Match e966(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e966M : TinyExpressionP4Parser::e966C, K215, K215, K215, K216, "TinyExpressionP4::NumberDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22846:22876:body/seq", false, false, "node", false, K4);
    }
    private static Match e966C(Session s, Frame f) {
        return s.sequence(f, K1103, K237);
    }
    private static Match e966M(Session s, Frame f) {
        return s.sequence(f, K1103, K237);
    }
    private static Match e967(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e967M : TinyExpressionP4Parser::e967C, K215, K215, K215, K216, "TinyExpressionP4::NumberDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22846:22855:body/0/literal", false, true, "text", false, K1104);
    }
    private static Match e967C(Session s, Frame f) {
        return s.literal(f, "default", true, false, K1105);
    }
    private static Match e967M(Session s, Frame f) {
        return s.literal(f, "default", true, false, K1105);
    }
    private static Match e968(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e968M : TinyExpressionP4Parser::e968C, K215, K215, K215, K216, "TinyExpressionP4::NumberDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22856:22860:body/1/literal", false, true, "text", false, K196);
    }
    private static Match e968C(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e968M(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e969(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e969M : TinyExpressionP4Parser::e969C, K1106, K299, K224, K225, "TinyExpressionP4::NumberDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22861:22876:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e969C(Session s, Frame f) {
        return parseNumberCaseValue_112(s, f);
    }
    private static Match e969M(Session s, Frame f) {
        return parseNumberCaseValue_112(s, f);
    }
    private static Match e970(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e970M : TinyExpressionP4Parser::e970C, K215, K215, K215, K216, "TinyExpressionP4::NumberCaseValue", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/seq", false, false, "node", false, K4);
    }
    private static Match e970C(Session s, Frame f) {
        return s.sequence(f, K1107, K420);
    }
    private static Match e970M(Session s, Frame f) {
        return s.sequence(f, K1107, K420);
    }
    private static Match e971(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e971M : TinyExpressionP4Parser::e971C, K1108, K299, K224, K225, "TinyExpressionP4::NumberCaseValue", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e971C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e971M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e972(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e972M : TinyExpressionP4Parser::e972C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23087:23205:body/seq", false, false, "node", true, K4);
    }
    private static Match e972C(Session s, Frame f) {
        return s.sequence(f, K1109, K219);
    }
    private static Match e972M(Session s, Frame f) {
        return s.sequence(f, K1109, K219);
    }
    private static Match e973(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e973M : TinyExpressionP4Parser::e973C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23087:23094:body/0/literal", false, true, "text", false, K1090);
    }
    private static Match e973C(Session s, Frame f) {
        return s.literal(f, "match", true, false, K1091);
    }
    private static Match e973M(Session s, Frame f) {
        return s.literal(f, "match", true, false, K1091);
    }
    private static Match e974(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e974M : TinyExpressionP4Parser::e974C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23095:23098:body/1/literal", false, true, "text", false, K390);
    }
    private static Match e974C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e974M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e975(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e975M : TinyExpressionP4Parser::e975C, K1110, K1093, K224, K225, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23105:23115:body/2/ruleRef", false, false, "node", false, K196);
    }
    private static Match e975C(Session s, Frame f) {
        return parseStringCase_114(s, f);
    }
    private static Match e975M(Session s, Frame f) {
        return parseStringCase_114(s, f);
    }
    private static Match e976(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e976M : TinyExpressionP4Parser::e976C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23127:23156:body/3/repeat", true, false, "node", true, K377);
    }
    private static Match e976C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e977, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e976M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e977, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e977(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e977M : TinyExpressionP4Parser::e977C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23129:23143:body/3/0/seq", false, false, "node", false, K377);
    }
    private static Match e977C(Session s, Frame f) {
        return s.sequence(f, K1111, K249);
    }
    private static Match e977M(Session s, Frame f) {
        return s.sequence(f, K1111, K249);
    }
    private static Match e978(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e978M : TinyExpressionP4Parser::e978C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23129:23132:body/3/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e978C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e978M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e979(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e979M : TinyExpressionP4Parser::e979C, K1112, K1096, K224, K225, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23133:23143:body/3/0/1/ruleRef", false, false, "node", false, K196);
    }
    private static Match e979C(Session s, Frame f) {
        return parseStringCase_114(s, f);
    }
    private static Match e979M(Session s, Frame f) {
        return parseStringCase_114(s, f);
    }
    private static Match e980(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e980M : TinyExpressionP4Parser::e980C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23163:23166:body/4/literal", false, true, "text", false, K377);
    }
    private static Match e980C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e980M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e981(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e981M : TinyExpressionP4Parser::e981C, K1113, K1098, K224, K225, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23167:23184:body/5/ruleRef", false, false, "node", false, K198);
    }
    private static Match e981C(Session s, Frame f) {
        return parseStringDefaultCase_115(s, f);
    }
    private static Match e981M(Session s, Frame f) {
        return parseStringDefaultCase_115(s, f);
    }
    private static Match e982(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e982M : TinyExpressionP4Parser::e982C, K215, K215, K215, K216, "TinyExpressionP4::StringMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23202:23205:body/6/literal", false, true, "text", false, K393);
    }
    private static Match e982C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e982M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e983(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e983M : TinyExpressionP4Parser::e983C, K215, K215, K215, K216, "TinyExpressionP4::StringCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23329:body/seq", false, false, "node", true, K4);
    }
    private static Match e983C(Session s, Frame f) {
        return s.sequence(f, K1114, K237);
    }
    private static Match e983M(Session s, Frame f) {
        return s.sequence(f, K1114, K237);
    }
    private static Match e984(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e984M : TinyExpressionP4Parser::e984C, K1115, K500, K224, K225, "TinyExpressionP4::StringCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23297:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e984C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e984M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e985(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e985M : TinyExpressionP4Parser::e985C, K215, K215, K215, K216, "TinyExpressionP4::StringCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23309:23313:body/1/literal", false, true, "text", false, K196);
    }
    private static Match e985C(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e985M(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e986(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e986M : TinyExpressionP4Parser::e986C, K1116, K299, K224, K225, "TinyExpressionP4::StringCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23314:23329:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e986C(Session s, Frame f) {
        return parseStringCaseValue_116(s, f);
    }
    private static Match e986M(Session s, Frame f) {
        return parseStringCaseValue_116(s, f);
    }
    private static Match e987(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e987M : TinyExpressionP4Parser::e987C, K215, K215, K215, K216, "TinyExpressionP4::StringDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23413:23443:body/seq", false, false, "node", false, K4);
    }
    private static Match e987C(Session s, Frame f) {
        return s.sequence(f, K1117, K237);
    }
    private static Match e987M(Session s, Frame f) {
        return s.sequence(f, K1117, K237);
    }
    private static Match e988(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e988M : TinyExpressionP4Parser::e988C, K215, K215, K215, K216, "TinyExpressionP4::StringDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23413:23422:body/0/literal", false, true, "text", false, K1104);
    }
    private static Match e988C(Session s, Frame f) {
        return s.literal(f, "default", true, false, K1105);
    }
    private static Match e988M(Session s, Frame f) {
        return s.literal(f, "default", true, false, K1105);
    }
    private static Match e989(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e989M : TinyExpressionP4Parser::e989C, K215, K215, K215, K216, "TinyExpressionP4::StringDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23423:23427:body/1/literal", false, true, "text", false, K196);
    }
    private static Match e989C(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e989M(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e990(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e990M : TinyExpressionP4Parser::e990C, K1118, K299, K224, K225, "TinyExpressionP4::StringDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23428:23443:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e990C(Session s, Frame f) {
        return parseStringCaseValue_116(s, f);
    }
    private static Match e990M(Session s, Frame f) {
        return parseStringCaseValue_116(s, f);
    }
    private static Match e991(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e991M : TinyExpressionP4Parser::e991C, K215, K215, K215, K216, "TinyExpressionP4::StringCaseValue", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/seq", false, false, "node", false, K4);
    }
    private static Match e991C(Session s, Frame f) {
        return s.sequence(f, K1119, K420);
    }
    private static Match e991M(Session s, Frame f) {
        return s.sequence(f, K1119, K420);
    }
    private static Match e992(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e992M : TinyExpressionP4Parser::e992C, K1120, K299, K224, K225, "TinyExpressionP4::StringCaseValue", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e992C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e992M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e993(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e993M : TinyExpressionP4Parser::e993C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23656:23777:body/seq", false, false, "node", true, K4);
    }
    private static Match e993C(Session s, Frame f) {
        return s.sequence(f, K1121, K219);
    }
    private static Match e993M(Session s, Frame f) {
        return s.sequence(f, K1121, K219);
    }
    private static Match e994(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e994M : TinyExpressionP4Parser::e994C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23656:23663:body/0/literal", false, true, "text", false, K1090);
    }
    private static Match e994C(Session s, Frame f) {
        return s.literal(f, "match", true, false, K1091);
    }
    private static Match e994M(Session s, Frame f) {
        return s.literal(f, "match", true, false, K1091);
    }
    private static Match e995(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e995M : TinyExpressionP4Parser::e995C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23664:23667:body/1/literal", false, true, "text", false, K390);
    }
    private static Match e995C(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e995M(Session s, Frame f) {
        return s.literal(f, "{", true, false, K391);
    }
    private static Match e996(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e996M : TinyExpressionP4Parser::e996C, K1122, K1093, K224, K225, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23674:23685:body/2/ruleRef", false, false, "node", false, K196);
    }
    private static Match e996C(Session s, Frame f) {
        return parseBooleanCase_118(s, f);
    }
    private static Match e996M(Session s, Frame f) {
        return parseBooleanCase_118(s, f);
    }
    private static Match e997(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e997M : TinyExpressionP4Parser::e997C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23697:23727:body/3/repeat", true, false, "node", true, K377);
    }
    private static Match e997C(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e998, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e997M(Session s, Frame f) {
        return s.repeat(f, TinyExpressionP4Parser::e998, 0, Integer.MAX_VALUE, null, Trivia.NONE);
    }
    private static Match e998(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e998M : TinyExpressionP4Parser::e998C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23699:23714:body/3/0/seq", false, false, "node", false, K377);
    }
    private static Match e998C(Session s, Frame f) {
        return s.sequence(f, K1123, K249);
    }
    private static Match e998M(Session s, Frame f) {
        return s.sequence(f, K1123, K249);
    }
    private static Match e999(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e999M : TinyExpressionP4Parser::e999C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23699:23702:body/3/0/0/literal", false, true, "text", false, K377);
    }
    private static Match e999C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e999M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e1000(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1000M : TinyExpressionP4Parser::e1000C, K1124, K1096, K224, K225, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23703:23714:body/3/0/1/ruleRef", false, false, "node", false, K196);
    }
    private static Match e1000C(Session s, Frame f) {
        return parseBooleanCase_118(s, f);
    }
    private static Match e1000M(Session s, Frame f) {
        return parseBooleanCase_118(s, f);
    }
    private static Match e1001(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1001M : TinyExpressionP4Parser::e1001C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23734:23737:body/4/literal", false, true, "text", false, K377);
    }
    private static Match e1001C(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e1001M(Session s, Frame f) {
        return s.literal(f, ",", true, false, K380);
    }
    private static Match e1002(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1002M : TinyExpressionP4Parser::e1002C, K1125, K1098, K224, K225, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23738:23756:body/5/ruleRef", false, false, "node", false, K198);
    }
    private static Match e1002C(Session s, Frame f) {
        return parseBooleanDefaultCase_119(s, f);
    }
    private static Match e1002M(Session s, Frame f) {
        return parseBooleanDefaultCase_119(s, f);
    }
    private static Match e1003(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1003M : TinyExpressionP4Parser::e1003C, K215, K215, K215, K216, "TinyExpressionP4::BooleanMatchExpression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23774:23777:body/6/literal", false, true, "text", false, K393);
    }
    private static Match e1003C(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e1003M(Session s, Frame f) {
        return s.literal(f, "}", true, false, K394);
    }
    private static Match e1004(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1004M : TinyExpressionP4Parser::e1004C, K215, K215, K215, K216, "TinyExpressionP4::BooleanCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23904:body/seq", false, false, "node", true, K4);
    }
    private static Match e1004C(Session s, Frame f) {
        return s.sequence(f, K1126, K237);
    }
    private static Match e1004M(Session s, Frame f) {
        return s.sequence(f, K1126, K237);
    }
    private static Match e1005(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1005M : TinyExpressionP4Parser::e1005C, K1127, K500, K224, K225, "TinyExpressionP4::BooleanCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23871:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1005C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e1005M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e1006(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1006M : TinyExpressionP4Parser::e1006C, K215, K215, K215, K216, "TinyExpressionP4::BooleanCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23883:23887:body/1/literal", false, true, "text", false, K196);
    }
    private static Match e1006C(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e1006M(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e1007(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1007M : TinyExpressionP4Parser::e1007C, K1128, K299, K224, K225, "TinyExpressionP4::BooleanCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23888:23904:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1007C(Session s, Frame f) {
        return parseBooleanCaseValue_120(s, f);
    }
    private static Match e1007M(Session s, Frame f) {
        return parseBooleanCaseValue_120(s, f);
    }
    private static Match e1008(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1008M : TinyExpressionP4Parser::e1008C, K215, K215, K215, K216, "TinyExpressionP4::BooleanDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23990:24021:body/seq", false, false, "node", false, K4);
    }
    private static Match e1008C(Session s, Frame f) {
        return s.sequence(f, K1129, K237);
    }
    private static Match e1008M(Session s, Frame f) {
        return s.sequence(f, K1129, K237);
    }
    private static Match e1009(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1009M : TinyExpressionP4Parser::e1009C, K215, K215, K215, K216, "TinyExpressionP4::BooleanDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23990:23999:body/0/literal", false, true, "text", false, K1104);
    }
    private static Match e1009C(Session s, Frame f) {
        return s.literal(f, "default", true, false, K1105);
    }
    private static Match e1009M(Session s, Frame f) {
        return s.literal(f, "default", true, false, K1105);
    }
    private static Match e1010(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1010M : TinyExpressionP4Parser::e1010C, K215, K215, K215, K216, "TinyExpressionP4::BooleanDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24000:24004:body/1/literal", false, true, "text", false, K196);
    }
    private static Match e1010C(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e1010M(Session s, Frame f) {
        return s.literal(f, "->", true, false, K1101);
    }
    private static Match e1011(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1011M : TinyExpressionP4Parser::e1011C, K1130, K299, K224, K225, "TinyExpressionP4::BooleanDefaultCase", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24005:24021:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1011C(Session s, Frame f) {
        return parseBooleanCaseValue_120(s, f);
    }
    private static Match e1011M(Session s, Frame f) {
        return parseBooleanCaseValue_120(s, f);
    }
    private static Match e1012(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1012M : TinyExpressionP4Parser::e1012C, K215, K215, K215, K216, "TinyExpressionP4::BooleanCaseValue", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/seq", false, false, "node", false, K4);
    }
    private static Match e1012C(Session s, Frame f) {
        return s.sequence(f, K1131, K420);
    }
    private static Match e1012M(Session s, Frame f) {
        return s.sequence(f, K1131, K420);
    }
    private static Match e1013(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1013M : TinyExpressionP4Parser::e1013C, K1132, K299, K224, K225, "TinyExpressionP4::BooleanCaseValue", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1013C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e1013M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e1014(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1014M : TinyExpressionP4Parser::e1014C, K215, K215, K215, K216, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24333:24384:body/seq", false, false, "text", false, K4);
    }
    private static Match e1014C(Session s, Frame f) {
        return s.sequence(f, K1133, K237);
    }
    private static Match e1014M(Session s, Frame f) {
        return s.sequence(f, K1133, K237);
    }
    private static Match e1015(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1015M : TinyExpressionP4Parser::e1015C, K215, K215, K215, K216, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24333:24336:body/0/literal", false, true, "text", false, K49);
    }
    private static Match e1015C(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e1015M(Session s, Frame f) {
        return s.literal(f, "$", true, false, K289);
    }
    private static Match e1016(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1016M : TinyExpressionP4Parser::e1016C, K1134, K454, K224, K225, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef", false, true, "text", false, K254);
    }
    private static Match e1016C(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e1016M(Session s, Frame f) {
        return s.builtin(f,"Identifier",K234,K255);
    }
    private static Match e1017(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1017M : TinyExpressionP4Parser::e1017C, K215, K215, K215, K216, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24354:24384:body/2/optional", true, false, "text", false, K292);
    }
    private static Match e1017C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e1018);
    }
    private static Match e1017M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e1018);
    }
    private static Match e1018(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1018M : TinyExpressionP4Parser::e1018C, K215, K215, K215, K216, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24356:24376:body/2/0/seq", false, false, "text", false, K256);
    }
    private static Match e1018C(Session s, Frame f) {
        return s.sequence(f, K1135, K249);
    }
    private static Match e1018M(Session s, Frame f) {
        return s.sequence(f, K1135, K249);
    }
    private static Match e1019(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1019M : TinyExpressionP4Parser::e1019C, K215, K215, K215, K216, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24356:24364:body/2/0/0/optional", true, false, "text", false, K256);
    }
    private static Match e1019C(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e1020);
    }
    private static Match e1019M(Session s, Frame f) {
        return s.optional(f, TinyExpressionP4Parser::e1020);
    }
    private static Match e1020(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1020M : TinyExpressionP4Parser::e1020C, K215, K215, K215, K216, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24358:24362:body/2/0/0/0/literal", false, true, "text", false, K256);
    }
    private static Match e1020C(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e1020M(Session s, Frame f) {
        return s.literal(f, "as", true, false, K257);
    }
    private static Match e1021(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1021M : TinyExpressionP4Parser::e1021C, K1136, K417, K224, K225, "TinyExpressionP4::VariableRef", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24365:24376:body/2/0/1/ruleRef", false, false, "text", false, K213);
    }
    private static Match e1021C(Session s, Frame f) {
        return parseTypeKeyword_122(s, f);
    }
    private static Match e1021M(Session s, Frame f) {
        return parseTypeKeyword_122(s, f);
    }
    private static Match e1022(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1022M : TinyExpressionP4Parser::e1022C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24406:24517:body/choice", false, false, "text", false, K4);
    }
    private static Match e1022C(Session s, Frame f) {
        return s.choice(f,K1137,false,null,false);
    }
    private static Match e1022M(Session s, Frame f) {
        return s.choice(f,K1137,false,null,false);
    }
    private static Match e1023(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1023M : TinyExpressionP4Parser::e1023C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24406:24414:body/0/literal", false, true, "text", false, K326);
    }
    private static Match e1023C(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e1023M(Session s, Frame f) {
        return s.literal(f, "number", true, false, K327);
    }
    private static Match e1024(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1024M : TinyExpressionP4Parser::e1024C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24417:24425:body/1/literal", false, true, "text", false, K328);
    }
    private static Match e1024C(Session s, Frame f) {
        return s.literal(f, "Number", true, false, K329);
    }
    private static Match e1024M(Session s, Frame f) {
        return s.literal(f, "Number", true, false, K329);
    }
    private static Match e1025(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1025M : TinyExpressionP4Parser::e1025C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24428:24435:body/2/literal", false, true, "text", false, K342);
    }
    private static Match e1025C(Session s, Frame f) {
        return s.literal(f, "float", true, false, K343);
    }
    private static Match e1025M(Session s, Frame f) {
        return s.literal(f, "float", true, false, K343);
    }
    private static Match e1026(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1026M : TinyExpressionP4Parser::e1026C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24438:24445:body/3/literal", false, true, "text", false, K344);
    }
    private static Match e1026C(Session s, Frame f) {
        return s.literal(f, "Float", true, false, K345);
    }
    private static Match e1026M(Session s, Frame f) {
        return s.literal(f, "Float", true, false, K345);
    }
    private static Match e1027(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1027M : TinyExpressionP4Parser::e1027C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24452:24460:body/4/literal", false, true, "text", false, K51);
    }
    private static Match e1027C(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e1027M(Session s, Frame f) {
        return s.literal(f, "string", true, false, K330);
    }
    private static Match e1028(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1028M : TinyExpressionP4Parser::e1028C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24463:24471:body/5/literal", false, true, "text", false, K331);
    }
    private static Match e1028C(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e1028M(Session s, Frame f) {
        return s.literal(f, "String", true, false, K332);
    }
    private static Match e1029(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1029M : TinyExpressionP4Parser::e1029C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24474:24483:body/6/literal", false, true, "text", false, K52);
    }
    private static Match e1029C(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e1029M(Session s, Frame f) {
        return s.literal(f, "boolean", true, false, K333);
    }
    private static Match e1030(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1030M : TinyExpressionP4Parser::e1030C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24486:24495:body/7/literal", false, true, "text", false, K334);
    }
    private static Match e1030C(Session s, Frame f) {
        return s.literal(f, "Boolean", true, false, K335);
    }
    private static Match e1030M(Session s, Frame f) {
        return s.literal(f, "Boolean", true, false, K335);
    }
    private static Match e1031(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1031M : TinyExpressionP4Parser::e1031C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24498:24506:body/8/literal", false, true, "text", false, K53);
    }
    private static Match e1031C(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e1031M(Session s, Frame f) {
        return s.literal(f, "object", true, false, K336);
    }
    private static Match e1032(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1032M : TinyExpressionP4Parser::e1032C, K215, K215, K215, K216, "TinyExpressionP4::TypeKeyword", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24509:24517:body/9/literal", false, true, "text", false, K337);
    }
    private static Match e1032C(Session s, Frame f) {
        return s.literal(f, "Object", true, false, K338);
    }
    private static Match e1032M(Session s, Frame f) {
        return s.literal(f, "Object", true, false, K338);
    }
    private static Match e1033(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1033M : TinyExpressionP4Parser::e1033C, K215, K215, K215, K216, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:25012:body/choice", false, false, "node", false, K4);
    }
    private static Match e1033C(Session s, Frame f) {
        return s.choice(f,K1139,false,null,false,K1138);
    }
    private static Match e1033M(Session s, Frame f) {
        return s.choice(f,K1139,false,null,false,K1138);
    }
    private static Match e1034(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1034M : TinyExpressionP4Parser::e1034C, K1140, K299, K224, K225, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:24852:body/0/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1034C(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e1034M(Session s, Frame f) {
        return parseNumberExpression_40(s, f);
    }
    private static Match e1035(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1035M : TinyExpressionP4Parser::e1035C, K1141, K299, K224, K225, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24866:24883:body/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1035C(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e1035M(Session s, Frame f) {
        return parseBooleanExpression_94(s, f);
    }
    private static Match e1036(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1036M : TinyExpressionP4Parser::e1036C, K1142, K299, K224, K225, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24897:24913:body/2/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1036C(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e1036M(Session s, Frame f) {
        return parseStringExpression_89(s, f);
    }
    private static Match e1037(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1037M : TinyExpressionP4Parser::e1037C, K1143, K299, K224, K225, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24927:24943:body/3/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1037C(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e1037M(Session s, Frame f) {
        return parseObjectExpression_105(s, f);
    }
    private static Match e1038(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1038M : TinyExpressionP4Parser::e1038C, K1144, K299, K224, K225, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24957:24973:body/4/ruleRef", false, false, "node", false, K160);
    }
    private static Match e1038C(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e1038M(Session s, Frame f) {
        return parseMethodInvocation_36(s, f);
    }
    private static Match e1039(Session s, Frame f) {
        f.reset = false;
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1039M : TinyExpressionP4Parser::e1039C, K215, K215, K215, K216, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24987:25012:body/5/seq", false, false, "node", false, K160);
    }
    private static Match e1039C(Session s, Frame f) {
        return s.sequence(f, K1145, K237);
    }
    private static Match e1039M(Session s, Frame f) {
        return s.sequence(f, K1145, K237);
    }
    private static Match e1040(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1040M : TinyExpressionP4Parser::e1040C, K215, K215, K215, K216, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24987:24990:body/5/0/literal", false, true, "text", false, K372);
    }
    private static Match e1040C(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e1040M(Session s, Frame f) {
        return s.literal(f, "(", true, false, K373);
    }
    private static Match e1041(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1041M : TinyExpressionP4Parser::e1041C, K1146, K299, K224, K225, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24991:25001:body/5/1/ruleRef", false, false, "node", false, K228);
    }
    private static Match e1041C(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e1041M(Session s, Frame f) {
        return parseExpression_123(s, f);
    }
    private static Match e1042(Session s, Frame f) {
        return s.capture(f, f.matched ? TinyExpressionP4Parser::e1042M : TinyExpressionP4Parser::e1042C, K215, K215, K215, K216, "TinyExpressionP4::Expression", "expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:25009:25012:body/5/2/literal", false, true, "text", false, K374);
    }
    private static Match e1042C(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static Match e1042M(Session s, Frame f) {
        return s.literal(f, ")", true, false, K375);
    }
    private static final String[] K0 = new String[]{"Formula","CodeBlock","ImportDeclaration","ClassName","VariableDeclaration","NumberVariableDeclaration","StringVariableDeclaration","BooleanVariableDeclaration","ObjectVariableDeclaration","TypeHint","NumberTypeHint","StringTypeHint","BooleanTypeHint","ObjectTypeHint","OnlyIfAbsent","Description","Annotation","AnnotationParameters","AnnotationParameter","MethodDeclaration","NumberMethodDeclaration","StringMethodDeclaration","BooleanMethodDeclaration","ObjectMethodDeclaration","MethodParameters","MethodParameter","NumberReturnType","StringReturnType","BooleanReturnType","ObjectReturnType","ReturnType","ExternalBooleanInvocation","ExternalNumberInvocation","ExternalStringInvocation","ExternalObjectInvocation","MethodInvocationHeader","MethodInvocation","ArgumentTernary","ArgumentExpression","Arguments","NumberExpression","NumberTerm","AddOp","MulOp","MathFunction","SinFunction","CosFunction","TanFunction","SqrtFunction","MinFunction","MaxFunction","RandomFunction","AbsFunction","RoundFunction","CeilFunction","FloorFunction","PowFunction","LogFunction","ExpFunction","ToNumFunction","NumberFactor","ToUpperCaseFunction","ToLowerCaseFunction","TrimFunction","LengthFunction","LenFunction","ToUpperCaseDotMethod","ToLowerCaseDotMethod","TrimDotMethod","LengthDotMethod","StartsWithFunction","EndsWithFunction","ContainsFunction","InMethod","StartsWithDotMethod","EndsWithDotMethod","ContainsDotMethod","StringPredicateReceiver","IsPresentFunction","InTimeRangeFunction","InDayTimeRangeFunction","DayOfWeek","SliceBaseReceiver","SliceStartIndex","SliceEndIndex","SliceStepIndex","SliceBaseExpression","SliceNestedExpression","SliceExpression","StringExpression","ParenthesizedStringExpression","StringTerm","StringCastVariable","StringTypedVariable","BooleanExpression","BooleanAndExpression","BooleanXorExpression","NotExpression","BooleanComparable","BooleanEqualityExpression","BooleanFactor","StringComparisonExpression","EqualityOp","ComparisonExpression","CompareOp","ObjectExpression","IfExpression","BranchExpression","TernaryExpression","NumberMatchExpression","NumberCase","NumberDefaultCase","NumberCaseValue","StringMatchExpression","StringCase","StringDefaultCase","StringCaseValue","BooleanMatchExpression","BooleanCase","BooleanDefaultCase","BooleanCaseValue","VariableRef","TypeKeyword","Expression"};
    private static final Map<String,String> K1 = Map.ofEntries(Map.entry("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef/capture/0","variable"),Map.entry("expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24365:24376:body/2/0/1/ruleRef/capture/0","variable"));
    private static final Field[] K2 = new Field[]{new Field("imports",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:902:919:body/1/0/ruleRef/capture/0"},"node","list"),new Field("declarations",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:933:952:body/2/0/ruleRef/capture/0"},"mixed","list"),new Field("expression",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:988:998:body/4/ruleRef/capture/0"},"node","scalar"),new Field("methods",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1013:1030:body/5/0/ruleRef/capture/0"},"mixed","list")};
    private static final Effect[] K3 = new Effect[]{new Effect("entry","enter",null,"Formula","lexical"),new Effect("successBeforeLeave","leave",null,"Formula","lexical")};
    private static final Label[] K4 = new Label[]{};
    private static final Label[] K5 = new Label[]{Label.of("FormulaParser")};
    private static final Field[] K6 = new Field[]{};
    private static final Effect[] K7 = new Effect[]{};
    private static final Label[] K8 = new Label[]{Label.of("CodeBlockParser")};
    private static final Field[] K9 = new Field[]{new Field("className",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1776:1785:body/1/ruleRef/capture/0"},"node","scalar"),new Field("method",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1803:1813:body/2/0/1/tokenRef/capture/0"},"text","optional"),new Field("alias",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1829:1839:body/4/tokenRef/capture/0"},"text","scalar")};
    private static final Label[] K10 = new Label[]{Label.of("'import'"),Label.of("'as'"),Label.of("';'")};
    private static final Field[] K11 = new Field[]{new Field("head",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1931:body/0/tokenRef/capture/0"},"text","scalar"),new Field("tail",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1944:1954:body/1/0/1/tokenRef/capture/0"},"text","list")};
    private static final Label[] K12 = new Label[]{Label.of("ClassNameParser")};
    private static final Label[] K13 = new Label[]{Label.of("'$'"),Label.of("';'")};
    private static final Field[] K14 = new Field[]{new Field("varName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2439:2449:body/2/tokenRef/capture/0"},"text","scalar"),new Field("onlyIfAbsent",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2496:2508:body/4/0/1/0/ruleRef/capture/0"},"node","optional"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2525:2541:body/4/0/2/ruleRef/capture/0"},"node","optional"),new Field("desc",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2557:2568:body/5/0/ruleRef/capture/0"},"text","optional")};
    private static final Effect[] K15 = new Effect[]{new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2439:2449:body/2/tokenRef/capture/0","varName","lexical")};
    private static final Label[] K16 = new Label[]{Label.of("'variable'"),Label.of("'var'"),Label.of("'$'"),Label.of("';'")};
    private static final Field[] K17 = new Field[]{new Field("varName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2788:2798:body/2/tokenRef/capture/0"},"text","scalar"),new Field("onlyIfAbsent",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2845:2857:body/4/0/1/0/ruleRef/capture/0"},"node","optional"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2874:2890:body/4/0/2/ruleRef/capture/0"},"node","optional"),new Field("desc",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2906:2917:body/5/0/ruleRef/capture/0"},"text","optional")};
    private static final Effect[] K18 = new Effect[]{new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2788:2798:body/2/tokenRef/capture/0","varName","lexical")};
    private static final Field[] K19 = new Field[]{new Field("varName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3139:3149:body/2/tokenRef/capture/0"},"text","scalar"),new Field("onlyIfAbsent",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3197:3209:body/4/0/1/0/ruleRef/capture/0"},"node","optional"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3226:3243:body/4/0/2/ruleRef/capture/0"},"node","optional"),new Field("desc",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3259:3270:body/5/0/ruleRef/capture/0"},"text","optional")};
    private static final Effect[] K20 = new Effect[]{new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3139:3149:body/2/tokenRef/capture/0","varName","lexical")};
    private static final Field[] K21 = new Field[]{new Field("varName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3490:3500:body/2/tokenRef/capture/0"},"text","scalar"),new Field("onlyIfAbsent",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3547:3559:body/4/0/1/0/ruleRef/capture/0"},"node","optional"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3576:3592:body/4/0/2/ruleRef/capture/0"},"node","optional"),new Field("desc",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3608:3619:body/5/0/ruleRef/capture/0"},"text","optional")};
    private static final Effect[] K22 = new Effect[]{new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3490:3500:body/2/tokenRef/capture/0","varName","lexical")};
    private static final Label[] K23 = new Label[]{Label.of("'as'"),Label.of("'number'"),Label.of("'Number'"),Label.of("'string'"),Label.of("'String'"),Label.of("'boolean'"),Label.of("'Boolean'"),Label.of("'object'"),Label.of("'Object'")};
    private static final Label[] K24 = new Label[]{Label.of("'as'"),Label.of("'number'"),Label.of("'Number'"),Label.of("'float'"),Label.of("'Float'")};
    private static final Label[] K25 = new Label[]{Label.of("'as'"),Label.of("'string'"),Label.of("'String'")};
    private static final Label[] K26 = new Label[]{Label.of("'as'"),Label.of("'boolean'"),Label.of("'Boolean'")};
    private static final Label[] K27 = new Label[]{Label.of("'as'"),Label.of("'object'"),Label.of("'Object'")};
    private static final Label[] K28 = new Label[]{Label.of("'if'"),Label.of("'not'"),Label.of("'exists'")};
    private static final Label[] K29 = new Label[]{Label.of("'description'"),Label.of("'='")};
    private static final Label[] K30 = new Label[]{Label.of("'@'"),Label.of("'('"),Label.of("')'")};
    private static final Label[] K31 = new Label[]{Label.of("'='")};
    private static final Label[] K32 = new Label[]{Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'")};
    private static final Field[] K33 = new Field[]{new Field("methodName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4668:4678:body/1/tokenRef/capture/0"},"text","scalar"),new Field("parameters",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4707:4723:body/3/0/ruleRef/capture/0"},"node","optional"),new Field("expression",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4760:4776:body/6/ruleRef/capture/0"},"node","scalar")};
    private static final Effect[] K34 = new Effect[]{new Effect("entry","enter",null,"NumberMethodDeclaration","lexical"),new Effect("successBeforeLeave","leave",null,"NumberMethodDeclaration","lexical"),new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4668:4678:body/1/tokenRef/capture/0","methodName","lexical")};
    private static final Label[] K35 = new Label[]{Label.of("'number'"),Label.of("'float'"),Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'")};
    private static final Field[] K36 = new Field[]{new Field("methodName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4994:5004:body/1/tokenRef/capture/0"},"text","scalar"),new Field("parameters",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5033:5049:body/3/0/ruleRef/capture/0"},"node","optional"),new Field("expression",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5086:5102:body/6/ruleRef/capture/0"},"node","scalar")};
    private static final Effect[] K37 = new Effect[]{new Effect("entry","enter",null,"StringMethodDeclaration","lexical"),new Effect("successBeforeLeave","leave",null,"StringMethodDeclaration","lexical"),new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4994:5004:body/1/tokenRef/capture/0","methodName","lexical")};
    private static final Label[] K38 = new Label[]{Label.of("'string'"),Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'")};
    private static final Field[] K39 = new Field[]{new Field("methodName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5323:5333:body/1/tokenRef/capture/0"},"text","scalar"),new Field("parameters",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5362:5378:body/3/0/ruleRef/capture/0"},"node","optional"),new Field("expression",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5415:5432:body/6/ruleRef/capture/0"},"node","scalar")};
    private static final Effect[] K40 = new Effect[]{new Effect("entry","enter",null,"BooleanMethodDeclaration","lexical"),new Effect("successBeforeLeave","leave",null,"BooleanMethodDeclaration","lexical"),new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5323:5333:body/1/tokenRef/capture/0","methodName","lexical")};
    private static final Label[] K41 = new Label[]{Label.of("'boolean'"),Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'")};
    private static final Field[] K42 = new Field[]{new Field("methodName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5650:5660:body/1/tokenRef/capture/0"},"text","scalar"),new Field("parameters",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5689:5705:body/3/0/ruleRef/capture/0"},"node","optional"),new Field("expression",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5742:5758:body/6/ruleRef/capture/0"},"node","scalar")};
    private static final Effect[] K43 = new Effect[]{new Effect("entry","enter",null,"ObjectMethodDeclaration","lexical"),new Effect("successBeforeLeave","leave",null,"ObjectMethodDeclaration","lexical"),new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5650:5660:body/1/tokenRef/capture/0","methodName","lexical")};
    private static final Label[] K44 = new Label[]{Label.of("'object'"),Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'")};
    private static final Field[] K45 = new Field[]{new Field("values",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5870:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5885:5900:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K46 = new Label[]{Label.of("MethodParametersParser")};
    private static final Field[] K47 = new Field[]{new Field("paramName",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6028:6038:body/1/tokenRef/capture/0"},"text","scalar"),new Field("type",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6057:6067:body/2/0/1/ruleRef/capture/0"},"text","optional")};
    private static final Effect[] K48 = new Effect[]{new Effect("successAfterLeave","declare","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6028:6038:body/1/tokenRef/capture/0","paramName","lexical")};
    private static final Label[] K49 = new Label[]{Label.of("'$'")};
    private static final Label[] K50 = new Label[]{Label.of("'number'"),Label.of("'float'")};
    private static final Label[] K51 = new Label[]{Label.of("'string'")};
    private static final Label[] K52 = new Label[]{Label.of("'boolean'")};
    private static final Label[] K53 = new Label[]{Label.of("'object'")};
    private static final Label[] K54 = new Label[]{Label.of("'number'"),Label.of("'float'"),Label.of("'string'"),Label.of("'boolean'"),Label.of("'object'")};
    private static final Field[] K55 = new Field[]{new Field("className",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6725:body/4/0/0/0/ruleRef/capture/0"},"node","optional"),new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6741:6751:body/4/0/0/2/tokenRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6760:6770:body/4/0/1/tokenRef/capture/0"},"text","scalar"),new Field("args",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6789:6798:body/6/0/ruleRef/capture/0"},"node","optional")};
    private static final Label[] K56 = new Label[]{Label.of("'external'"),Label.of("'boolean'"),Label.of("':'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K57 = new Field[]{new Field("className",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7016:body/2/0/0/0/ruleRef/capture/0"},"node","optional"),new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7032:7042:body/2/0/0/2/tokenRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7051:7061:body/2/0/1/tokenRef/capture/0"},"text","scalar"),new Field("args",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7080:7089:body/4/0/ruleRef/capture/0"},"node","optional")};
    private static final Label[] K58 = new Label[]{Label.of("'external'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K59 = new Field[]{new Field("className",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7293:body/4/0/0/0/ruleRef/capture/0"},"node","optional"),new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7309:7319:body/4/0/0/2/tokenRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7328:7338:body/4/0/1/tokenRef/capture/0"},"text","scalar"),new Field("args",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7357:7366:body/6/0/ruleRef/capture/0"},"node","optional")};
    private static final Label[] K60 = new Label[]{Label.of("'external'"),Label.of("'string'"),Label.of("':'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K61 = new Field[]{new Field("className",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7570:body/4/0/0/0/ruleRef/capture/0"},"node","optional"),new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7586:7596:body/4/0/0/2/tokenRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7605:7615:body/4/0/1/tokenRef/capture/0"},"text","scalar"),new Field("args",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7634:7643:body/6/0/ruleRef/capture/0"},"node","optional")};
    private static final Label[] K62 = new Label[]{Label.of("'external'"),Label.of("'object'"),Label.of("':'"),Label.of("'('"),Label.of("')'")};
    private static final Label[] K63 = new Label[]{Label.of("'call'"),Label.of("'internal'")};
    private static final Field[] K64 = new Field[]{new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8020:8030:body/1/tokenRef/capture/0"},"text","scalar"),new Field("args",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8043:8052:body/3/0/ruleRef/capture/0"},"node","optional")};
    private static final Effect[] K65 = new Effect[]{new Effect("afterDeclarations","use","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8020:8030:body/1/tokenRef/capture/0","name","lexical")};
    private static final Label[] K66 = new Label[]{Label.of("'internal'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K67 = new Field[]{new Field("condition",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8248:body/0/ruleRef/capture/0"},"node","scalar"),new Field("thenExpr",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8264:8280:body/2/ruleRef/capture/0"},"node","scalar"),new Field("elseExpr",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8295:8311:body/4/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K68 = new Label[]{Label.of("'?'"),Label.of("':'")};
    private static final Field[] K69 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8422:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8436:8456:body/1/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8470:8496:body/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8510:8520:body/3/ruleRef/capture/0"},"mixed","scalar")};
    private static final Label[] K70 = new Label[]{Label.of("ArgumentExpressionParser")};
    private static final Field[] K71 = new Field[]{new Field("values",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8608:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8623:8641:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K72 = new Label[]{Label.of("ArgumentsParser")};
    private static final Field[] K73 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8856:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8870:body/1/0/0/ruleRef/capture/0"},"text","list"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8875:8885:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K74 = new Label[]{Label.of("NumberExpressionParser")};
    private static final Field[] K75 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9013:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9027:body/1/0/0/ruleRef/capture/0"},"text","list"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9032:9044:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K76 = new Label[]{Label.of("NumberTermParser")};
    private static final Label[] K77 = new Label[]{Label.of("'+'"),Label.of("'-'")};
    private static final Label[] K78 = new Label[]{Label.of("'*'"),Label.of("'/'")};
    private static final Label[] K79 = new Label[]{Label.of("'sin'"),Label.of("'('"),Label.of("')'"),Label.of("'cos'"),Label.of("'tan'"),Label.of("'sqrt'"),Label.of("'min'"),Label.of("'max'"),Label.of("'random'"),Label.of("'abs'"),Label.of("'round'"),Label.of("'ceil'"),Label.of("'floor'"),Label.of("'pow'"),Label.of("','"),Label.of("'log'"),Label.of("'exp'")};
    private static final Field[] K80 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9530:9548:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K81 = new Label[]{Label.of("'sin'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K82 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9623:9641:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K83 = new Label[]{Label.of("'cos'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K84 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9716:9734:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K85 = new Label[]{Label.of("'tan'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K86 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9812:9830:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K87 = new Label[]{Label.of("'sqrt'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K88 = new Field[]{new Field("first",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9913:9931:body/2/ruleRef/capture/0"},"node","scalar"),new Field("rest",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9945:9963:body/3/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K89 = new Label[]{Label.of("'min'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K90 = new Field[]{new Field("first",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10049:10067:body/2/ruleRef/capture/0"},"node","scalar"),new Field("rest",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10081:10099:body/3/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K91 = new Label[]{Label.of("'max'"),Label.of("'('"),Label.of("')'")};
    private static final Label[] K92 = new Label[]{Label.of("'random'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K93 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10241:10259:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K94 = new Label[]{Label.of("'abs'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K95 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10340:10358:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K96 = new Label[]{Label.of("'round'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K97 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10436:10454:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K98 = new Label[]{Label.of("'ceil'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K99 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10535:10553:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K100 = new Label[]{Label.of("'floor'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K101 = new Field[]{new Field("base",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10639:10657:body/2/ruleRef/capture/0"},"node","scalar"),new Field("exponent",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10668:10686:body/4/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K102 = new Label[]{Label.of("'pow'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Field[] K103 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10766:10784:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K104 = new Label[]{Label.of("'log'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K105 = new Field[]{new Field("arg",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10859:10877:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K106 = new Label[]{Label.of("'exp'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K107 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11052:11068:body/2/ruleRef/capture/0"},"node","scalar"),new Field("defaultValue",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11080:11098:body/4/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K108 = new Label[]{Label.of("'toNum'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Label[] K109 = new Label[]{Label.of("'('"),Label.of("'?'"),Label.of("':'"),Label.of("')'"),Label.of("'match'"),Label.of("'{'"),Label.of("','"),Label.of("'}'"),Label.of("'if'"),Label.of("'else'"),Label.of("'toNum'"),Label.of("'.length'"),Label.of("'len'"),Label.of("'length'"),Label.of("'external'"),Label.of("'$'")};
    private static final Field[] K110 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11596:11612:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K111 = new Label[]{Label.of("'toUpperCase'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K112 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11715:11731:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K113 = new Label[]{Label.of("'toLowerCase'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K114 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11813:11829:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K115 = new Label[]{Label.of("'trim'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K116 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11917:11933:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K117 = new Label[]{Label.of("'length'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K118 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12015:12031:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K119 = new Label[]{Label.of("'len'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K120 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12378:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K121 = new Label[]{Label.of("'.toUpperCase'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K122 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12497:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K123 = new Label[]{Label.of("'.toLowerCase'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K124 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12602:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K125 = new Label[]{Label.of("'.trim'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K126 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12704:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K127 = new Label[]{Label.of("'.length'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K128 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12966:12982:body/2/ruleRef/capture/0"},"node","scalar"),new Field("patterns",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12994:13010:body/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13031:13047:body/5/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K129 = new Label[]{Label.of("'startsWith'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Field[] K130 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13156:13172:body/2/ruleRef/capture/0"},"node","scalar"),new Field("patterns",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13184:13200:body/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13221:13237:body/5/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K131 = new Label[]{Label.of("'endsWith'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Field[] K132 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13346:13362:body/2/ruleRef/capture/0"},"node","scalar"),new Field("patterns",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13374:13390:body/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13411:13427:body/5/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K133 = new Label[]{Label.of("'contains'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Field[] K134 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13525:body/0/ruleRef/capture/0"},"node","scalar"),new Field("candidates",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13543:13559:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13578:13594:body/4/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K135 = new Label[]{Label.of("'.in'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K136 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13800:body/0/ruleRef/capture/0"},"mixed","scalar"),new Field("patterns",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13826:13842:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13863:13879:body/4/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K137 = new Label[]{Label.of("'.startsWith'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K138 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14000:body/0/ruleRef/capture/0"},"mixed","scalar"),new Field("patterns",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14024:14040:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14061:14077:body/4/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K139 = new Label[]{Label.of("'.endsWith'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K140 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14198:body/0/ruleRef/capture/0"},"mixed","scalar"),new Field("patterns",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14222:14238:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14259:14275:body/4/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K141 = new Label[]{Label.of("'.contains'"),Label.of("'('"),Label.of("')'")};
    private static final Label[] K142 = new Label[]{Label.of("'toLowerCase'"),Label.of("'('"),Label.of("')'"),Label.of("'toUpperCase'"),Label.of("'trim'"),Label.of("'$'")};
    private static final Field[] K143 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14560:14571:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K144 = new Label[]{Label.of("'isPresent'"),Label.of("'('"),Label.of("')'")};
    private static final Field[] K145 = new Field[]{new Field("startHour",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14764:14780:body/2/ruleRef/capture/0"},"node","scalar"),new Field("endHour",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14796:14812:body/4/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K146 = new Label[]{Label.of("'inTimeRange'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Field[] K147 = new Field[]{new Field("startDay",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14957:14966:body/2/ruleRef/capture/0"},"text","scalar"),new Field("startHour",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14981:14997:body/4/ruleRef/capture/0"},"node","scalar"),new Field("endDay",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15013:15022:body/6/ruleRef/capture/0"},"text","scalar"),new Field("endHour",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15035:15051:body/8/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K148 = new Label[]{Label.of("'inDayTimeRange'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final Label[] K149 = new Label[]{Label.of("'MONDAY'"),Label.of("'TUESDAY'"),Label.of("'WEDNESDAY'"),Label.of("'THURSDAY'"),Label.of("'FRIDAY'"),Label.of("'SATURDAY'"),Label.of("'SUNDAY'")};
    private static final Label[] K150 = new Label[]{Label.of("'external'"),Label.of("'('"),Label.of("')'"),Label.of("'toUpperCase'"),Label.of("'toLowerCase'"),Label.of("'trim'"),Label.of("'.toUpperCase'"),Label.of("'.toLowerCase'"),Label.of("'.trim'"),Label.of("'$'")};
    private static final Label[] K151 = new Label[]{Label.of("SliceStartIndexParser")};
    private static final Label[] K152 = new Label[]{Label.of("SliceEndIndexParser")};
    private static final Label[] K153 = new Label[]{Label.of("SliceStepIndexParser")};
    private static final Field[] K154 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16326:body/0/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16436:body/1/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16521:body/2/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16612:body/3/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16678:body/4/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16765:body/5/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16827:body/6/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16895:body/7/0/ruleRef/capture/0"},"mixed","scalar"),new Field("start",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16338:16353:body/0/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16448:16463:body/1/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16533:16548:body/2/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16624:16639:body/3/2/ruleRef/capture/0"},"node","optional"),new Field("end",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16365:16378:body/0/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16475:16488:body/1/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16694:16707:body/4/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16781:16794:body/5/3/ruleRef/capture/0"},"node","optional"),new Field("step",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16388:16402:body/0/6/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16564:16578:body/2/5/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16717:16731:body/4/5/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16847:16861:body/6/4/ruleRef/capture/0"},"node","optional")};
    private static final Label[] K155 = new Label[]{Label.of("'['"),Label.of("':'"),Label.of("']'")};
    private static final Field[] K156 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17027:body/0/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17139:body/1/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17226:body/2/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17319:body/3/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17387:body/4/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17476:body/5/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17540:body/6/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17610:body/7/0/ruleRef/capture/0"},"mixed","scalar"),new Field("start",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17039:17054:body/0/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17151:17166:body/1/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17238:17253:body/2/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17331:17346:body/3/2/ruleRef/capture/0"},"node","optional"),new Field("end",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17066:17079:body/0/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17178:17191:body/1/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17403:17416:body/4/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17492:17505:body/5/3/ruleRef/capture/0"},"node","optional"),new Field("step",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17089:17103:body/0/6/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17269:17283:body/2/5/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17426:17440:body/4/5/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17560:17574:body/6/4/ruleRef/capture/0"},"node","optional")};
    private static final Label[] K157 = new Label[]{Label.of("SliceExpressionParser")};
    private static final Field[] K158 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17909:body/0/ruleRef/capture/0"},"mixed","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17921:body/1/0/0/literal/capture/0"},"text","list"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17926:17936:body/1/0/1/ruleRef/capture/0"},"mixed","list")};
    private static final Label[] K159 = new Label[]{Label.of("StringExpressionParser")};
    private static final Label[] K160 = new Label[]{Label.of("'('"),Label.of("')'")};
    private static final Label[] K161 = new Label[]{Label.of("'match'"),Label.of("'{'"),Label.of("','"),Label.of("'}'"),Label.of("'if'"),Label.of("'('"),Label.of("')'"),Label.of("'else'"),Label.of("'$'"),Label.of("'as'"),Label.of("'external'"),Label.of("'toUpperCase'"),Label.of("'toLowerCase'"),Label.of("'trim'"),Label.of("'.toUpperCase'"),Label.of("'.toLowerCase'"),Label.of("'.trim'")};
    private static final Field[] K162 = new Field[]{new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18697:18707:body/4/tokenRef/capture/0"},"text","scalar")};
    private static final Label[] K163 = new Label[]{Label.of("'('"),Label.of("'string'"),Label.of("'String'"),Label.of("')'"),Label.of("'$'")};
    private static final Field[] K164 = new Field[]{new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18801:18811:body/1/tokenRef/capture/0"},"text","scalar")};
    private static final Label[] K165 = new Label[]{Label.of("'$'"),Label.of("'as'"),Label.of("'string'"),Label.of("'String'")};
    private static final Field[] K166 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19097:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19109:body/1/0/0/literal/capture/0"},"text","list"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19114:19134:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K167 = new Label[]{Label.of("BooleanExpressionParser")};
    private static final Field[] K168 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19300:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19312:body/1/0/0/literal/capture/0"},"text","list"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19317:19337:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K169 = new Label[]{Label.of("BooleanAndExpressionParser")};
    private static final Field[] K170 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19496:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19508:body/1/0/0/literal/capture/0"},"text","list"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19513:19526:body/1/0/1/ruleRef/capture/0"},"node","list")};
    private static final Label[] K171 = new Label[]{Label.of("BooleanXorExpressionParser")};
    private static final Field[] K172 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19683:19700:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K173 = new Label[]{Label.of("'not'"),Label.of("'('"),Label.of("')'")};
    private static final Label[] K174 = new Label[]{Label.of("'not'"),Label.of("'('"),Label.of("')'"),Label.of("'if'"),Label.of("'{'"),Label.of("'}'"),Label.of("'else'"),Label.of("'match'"),Label.of("','"),Label.of("'external'"),Label.of("'.in'"),Label.of("'.startsWith'"),Label.of("'.endsWith'"),Label.of("'.contains'"),Label.of("'startsWith'"),Label.of("'endsWith'"),Label.of("'contains'"),Label.of("'isPresent'"),Label.of("'inTimeRange'"),Label.of("'inDayTimeRange'"),Label.of("'true'"),Label.of("'false'"),Label.of("'$'")};
    private static final Field[] K175 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20288:body/0/ruleRef/capture/0"},"mixed","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20295:20305:body/1/ruleRef/capture/0"},"text","scalar"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20310:20327:body/2/ruleRef/capture/0"},"mixed","scalar")};
    private static final Label[] K176 = new Label[]{Label.of("BooleanEqualityExpressionParser")};
    private static final Field[] K177 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20435:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20449:20469:body/1/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20483:20509:body/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20523:20540:body/3/ruleRef/capture/0"},"mixed","scalar")};
    private static final Label[] K178 = new Label[]{Label.of("BooleanFactorParser")};
    private static final Field[] K179 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20820:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20827:20837:body/1/ruleRef/capture/0"},"text","scalar"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20842:20858:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K180 = new Label[]{Label.of("StringComparisonExpressionParser")};
    private static final Label[] K181 = new Label[]{Label.of("'=='"),Label.of("'!='")};
    private static final Field[] K182 = new Field[]{new Field("left",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:20996:body/0/ruleRef/capture/0"},"node","scalar"),new Field("op",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21003:21012:body/1/ruleRef/capture/0"},"text","scalar"),new Field("right",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21017:21033:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K183 = new Label[]{Label.of("ComparisonExpressionParser")};
    private static final Label[] K184 = new Label[]{Label.of("'=='"),Label.of("'!='"),Label.of("'<='"),Label.of("'>='"),Label.of("'<'"),Label.of("'>'")};
    private static final Field[] K185 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21266:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21280:21296:body/1/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21310:21327:body/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21341:21365:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21379:21390:body/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21404:21420:body/5/ruleRef/capture/0"},"mixed","scalar")};
    private static final Label[] K186 = new Label[]{Label.of("ObjectExpressionParser")};
    private static final Field[] K187 = new Field[]{new Field("condition",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21604:21621:body/2/ruleRef/capture/0"},"node","scalar"),new Field("thenExpr",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21645:21661:body/5/ruleRef/capture/0"},"node","scalar"),new Field("elseExpr",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21695:21711:body/9/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K188 = new Label[]{Label.of("'if'"),Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'"),Label.of("'else'")};
    private static final Field[] K189 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:21827:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21841:21867:body/1/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21881:21906:body/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21920:21936:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21950:21967:body/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21981:21997:body/5/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22011:22027:body/6/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22041:22057:body/7/ruleRef/capture/0"},"mixed","scalar")};
    private static final Label[] K190 = new Label[]{Label.of("BranchExpressionParser")};
    private static final Field[] K191 = new Field[]{new Field("condition",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22236:22253:body/1/ruleRef/capture/0"},"node","scalar"),new Field("thenExpr",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22269:22285:body/3/ruleRef/capture/0"},"node","scalar"),new Field("elseExpr",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22300:22316:body/5/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K192 = new Label[]{Label.of("'('"),Label.of("'?'"),Label.of("':'"),Label.of("')'")};
    private static final Field[] K193 = new Field[]{new Field("firstCase",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22538:22548:body/2/ruleRef/capture/0"},"node","scalar"),new Field("moreCases",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22566:22576:body/3/0/1/ruleRef/capture/0"},"node","list"),new Field("defaultCase",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22600:22617:body/5/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K194 = new Label[]{Label.of("'match'"),Label.of("'{'"),Label.of("','"),Label.of("'}'")};
    private static final Field[] K195 = new Field[]{new Field("condition",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22730:body/0/ruleRef/capture/0"},"node","scalar"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22747:22762:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K196 = new Label[]{Label.of("'->'")};
    private static final Field[] K197 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22861:22876:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K198 = new Label[]{Label.of("'default'"),Label.of("'->'")};
    private static final Field[] K199 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K200 = new Label[]{Label.of("NumberCaseValueParser")};
    private static final Field[] K201 = new Field[]{new Field("firstCase",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23105:23115:body/2/ruleRef/capture/0"},"node","scalar"),new Field("moreCases",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23133:23143:body/3/0/1/ruleRef/capture/0"},"node","list"),new Field("defaultCase",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23167:23184:body/5/ruleRef/capture/0"},"node","scalar")};
    private static final Field[] K202 = new Field[]{new Field("condition",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23297:body/0/ruleRef/capture/0"},"node","scalar"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23314:23329:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Field[] K203 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23428:23443:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Field[] K204 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K205 = new Label[]{Label.of("StringCaseValueParser")};
    private static final Field[] K206 = new Field[]{new Field("firstCase",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23674:23685:body/2/ruleRef/capture/0"},"node","scalar"),new Field("moreCases",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23703:23714:body/3/0/1/ruleRef/capture/0"},"node","list"),new Field("defaultCase",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23738:23756:body/5/ruleRef/capture/0"},"node","scalar")};
    private static final Field[] K207 = new Field[]{new Field("condition",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23871:body/0/ruleRef/capture/0"},"node","scalar"),new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23888:23904:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Field[] K208 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24005:24021:body/2/ruleRef/capture/0"},"node","scalar")};
    private static final Field[] K209 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/0/ruleRef/capture/0"},"node","scalar")};
    private static final Label[] K210 = new Label[]{Label.of("BooleanCaseValueParser")};
    private static final Field[] K211 = new Field[]{new Field("name",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef/capture/0"},"text","scalar"),new Field("type",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24365:24376:body/2/0/1/ruleRef/capture/0"},"text","optional")};
    private static final Effect[] K212 = new Effect[]{new Effect("afterDeclarations","use","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef/capture/0","name","lexical")};
    private static final Label[] K213 = new Label[]{Label.of("'number'"),Label.of("'Number'"),Label.of("'float'"),Label.of("'Float'"),Label.of("'string'"),Label.of("'String'"),Label.of("'boolean'"),Label.of("'Boolean'"),Label.of("'object'"),Label.of("'Object'")};
    private static final Field[] K214 = new Field[]{new Field("value",new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:24852:body/0/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24866:24883:body/1/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24897:24913:body/2/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24927:24943:body/3/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24957:24973:body/4/ruleRef/capture/0","expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24991:25001:body/5/1/ruleRef/capture/0"},"mixed","scalar")};
    private static final String[] K215 = new String[]{};
    private static final boolean[] K216 = new boolean[]{};
    private static final Delimiters K217 = new Delimiters(new Delimiter("characters",null,null,new int[]{9,10,11,12,13,32}),new Delimiter("lineComment","//",null,new int[]{}),new Delimiter("blockComment","/*","*/",new int[]{}));
    private static final Delimiters[] K218 = new Delimiters[]{K217,K217,K217,K217,K217,K217,K217};
    private static final Trivia K219 = new Trivia(K217,null,K218,null,null);
    private static final Call[] K220 = new Call[]{TinyExpressionP4Parser::e1,TinyExpressionP4Parser::e3,TinyExpressionP4Parser::e5,TinyExpressionP4Parser::e7,TinyExpressionP4Parser::e9,TinyExpressionP4Parser::e10,TinyExpressionP4Parser::e12};
    private static final Label[] K221 = new Label[]{Label.of("Repeat")};
    private static final String[] K222 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:902:919:body/1/0/ruleRef/capture/0"};
    private static final String[] K223 = new String[]{"imports"};
    private static final String[] K224 = new String[]{"consumedExtent"};
    private static final boolean[] K225 = new boolean[]{false};
    private static final String[] K226 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:933:952:body/2/0/ruleRef/capture/0"};
    private static final String[] K227 = new String[]{"declarations"};
    private static final Label[] K228 = new Label[]{Label.of("__CaptureSite")};
    private static final String[] K229 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:988:998:body/4/ruleRef/capture/0"};
    private static final String[] K230 = new String[]{"expression"};
    private static final String[] K231 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1013:1030:body/5/0/ruleRef/capture/0"};
    private static final String[] K232 = new String[]{"methods"};
    private static final Label[] K233 = new Label[]{Label.of("EndOfSourceParser")};
    private static final int[] K234 = new int[]{};
    private static final Label K235 = Label.of("EndOfSourceParser");
    private static final Delimiters[] K236 = new Delimiters[]{K217,K217,K217};
    private static final Trivia K237 = new Trivia(K217,null,K236,null,null);
    private static final Call[] K238 = new Call[]{TinyExpressionP4Parser::e14,TinyExpressionP4Parser::e15,TinyExpressionP4Parser::e16};
    private static final Label[] K239 = new Label[]{Label.of("WildCardStringTerminatorParser")};
    private static final Delimiters[] K240 = new Delimiters[]{K217,K217,K217,K217,K217,K217};
    private static final Trivia K241 = new Trivia(K217,null,K240,null,null);
    private static final Call[] K242 = new Call[]{TinyExpressionP4Parser::e18,TinyExpressionP4Parser::e19,TinyExpressionP4Parser::e20,TinyExpressionP4Parser::e24,TinyExpressionP4Parser::e25,TinyExpressionP4Parser::e26};
    private static final Label[] K243 = new Label[]{Label.of("'import'")};
    private static final Label K244 = Label.of("'import'");
    private static final String[] K245 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1776:1785:body/1/ruleRef/capture/0"};
    private static final String[] K246 = new String[]{"className"};
    private static final Label[] K247 = new Label[]{Label.of("'#'")};
    private static final Delimiters[] K248 = new Delimiters[]{K217,K217};
    private static final Trivia K249 = new Trivia(K217,null,K248,null,null);
    private static final Call[] K250 = new Call[]{TinyExpressionP4Parser::e22,TinyExpressionP4Parser::e23};
    private static final Label K251 = Label.of("'#'");
    private static final String[] K252 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1803:1813:body/2/0/1/tokenRef/capture/0"};
    private static final String[] K253 = new String[]{"method"};
    private static final Label[] K254 = new Label[]{Label.of("IdentifierParser"),Label.of("__CaptureSite")};
    private static final Label K255 = Label.of("IdentifierParser");
    private static final Label[] K256 = new Label[]{Label.of("'as'")};
    private static final Label K257 = Label.of("'as'");
    private static final String[] K258 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1829:1839:body/4/tokenRef/capture/0"};
    private static final String[] K259 = new String[]{"alias"};
    private static final Label[] K260 = new Label[]{Label.of("';'")};
    private static final Label K261 = Label.of("';'");
    private static final Call[] K262 = new Call[]{TinyExpressionP4Parser::e28,TinyExpressionP4Parser::e29};
    private static final String[] K263 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1921:1931:body/0/tokenRef/capture/0"};
    private static final String[] K264 = new String[]{"head"};
    private static final Label[] K265 = new Label[]{Label.of("'.'")};
    private static final Call[] K266 = new Call[]{TinyExpressionP4Parser::e31,TinyExpressionP4Parser::e32};
    private static final Label K267 = Label.of("'.'");
    private static final String[] K268 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:1944:1954:body/1/0/1/tokenRef/capture/0"};
    private static final String[] K269 = new String[]{"tail"};
    private static final int[] K270 = new int[]{9,13,32,32,47,47,118,118};
    private static final Label[] K271 = new Label[]{null,Label.of("'variable'"),Label.of("'var'"),Label.of("'$'"),Label.of("';'")};
    private static final int[] K272 = new int[]{5};
    private static final Guard K273 = new Guard(K270,K271,K217,K272,1);
    private static final int[] K274 = new int[]{6};
    private static final Guard K275 = new Guard(K270,K271,K217,K274,1);
    private static final int[] K276 = new int[]{7};
    private static final Guard K277 = new Guard(K270,K271,K217,K276,1);
    private static final int[] K278 = new int[]{8};
    private static final Guard K279 = new Guard(K270,K271,K217,K278,1);
    private static final Guard[] K280 = new Guard[]{K273,K275,K277,K279};
    private static final Call[] K281 = new Call[]{TinyExpressionP4Parser::e34,TinyExpressionP4Parser::e35,TinyExpressionP4Parser::e36,TinyExpressionP4Parser::e37};
    private static final Call[] K282 = new Call[]{TinyExpressionP4Parser::e39,TinyExpressionP4Parser::e43,TinyExpressionP4Parser::e44,TinyExpressionP4Parser::e45,TinyExpressionP4Parser::e47,TinyExpressionP4Parser::e53,TinyExpressionP4Parser::e55};
    private static final Label[] K283 = new Label[]{Label.of("'variable'"),Label.of("'var'")};
    private static final Call[] K284 = new Call[]{TinyExpressionP4Parser::e41,TinyExpressionP4Parser::e42};
    private static final Label[] K285 = new Label[]{Label.of("'variable'")};
    private static final Label K286 = Label.of("'variable'");
    private static final Label[] K287 = new Label[]{Label.of("'var'")};
    private static final Label K288 = Label.of("'var'");
    private static final Label K289 = Label.of("'$'");
    private static final String[] K290 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2439:2449:body/2/tokenRef/capture/0"};
    private static final String[] K291 = new String[]{"varName"};
    private static final Label[] K292 = new Label[]{Label.of("Optional")};
    private static final Label[] K293 = new Label[]{Label.of("'set'")};
    private static final Call[] K294 = new Call[]{TinyExpressionP4Parser::e49,TinyExpressionP4Parser::e50,TinyExpressionP4Parser::e52};
    private static final Label K295 = Label.of("'set'");
    private static final String[] K296 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2496:2508:body/4/0/1/0/ruleRef/capture/0"};
    private static final String[] K297 = new String[]{"onlyIfAbsent"};
    private static final String[] K298 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2525:2541:body/4/0/2/ruleRef/capture/0"};
    private static final String[] K299 = new String[]{"value"};
    private static final String[] K300 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2557:2568:body/5/0/ruleRef/capture/0"};
    private static final String[] K301 = new String[]{"desc"};
    private static final Call[] K302 = new Call[]{TinyExpressionP4Parser::e57,TinyExpressionP4Parser::e61,TinyExpressionP4Parser::e62,TinyExpressionP4Parser::e63,TinyExpressionP4Parser::e65,TinyExpressionP4Parser::e71,TinyExpressionP4Parser::e73};
    private static final Call[] K303 = new Call[]{TinyExpressionP4Parser::e59,TinyExpressionP4Parser::e60};
    private static final String[] K304 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2788:2798:body/2/tokenRef/capture/0"};
    private static final Call[] K305 = new Call[]{TinyExpressionP4Parser::e67,TinyExpressionP4Parser::e68,TinyExpressionP4Parser::e70};
    private static final String[] K306 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2845:2857:body/4/0/1/0/ruleRef/capture/0"};
    private static final String[] K307 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2874:2890:body/4/0/2/ruleRef/capture/0"};
    private static final String[] K308 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:2906:2917:body/5/0/ruleRef/capture/0"};
    private static final Call[] K309 = new Call[]{TinyExpressionP4Parser::e75,TinyExpressionP4Parser::e79,TinyExpressionP4Parser::e80,TinyExpressionP4Parser::e81,TinyExpressionP4Parser::e83,TinyExpressionP4Parser::e89,TinyExpressionP4Parser::e91};
    private static final Call[] K310 = new Call[]{TinyExpressionP4Parser::e77,TinyExpressionP4Parser::e78};
    private static final String[] K311 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3139:3149:body/2/tokenRef/capture/0"};
    private static final Call[] K312 = new Call[]{TinyExpressionP4Parser::e85,TinyExpressionP4Parser::e86,TinyExpressionP4Parser::e88};
    private static final String[] K313 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3197:3209:body/4/0/1/0/ruleRef/capture/0"};
    private static final String[] K314 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3226:3243:body/4/0/2/ruleRef/capture/0"};
    private static final String[] K315 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3259:3270:body/5/0/ruleRef/capture/0"};
    private static final Call[] K316 = new Call[]{TinyExpressionP4Parser::e93,TinyExpressionP4Parser::e97,TinyExpressionP4Parser::e98,TinyExpressionP4Parser::e99,TinyExpressionP4Parser::e101,TinyExpressionP4Parser::e107,TinyExpressionP4Parser::e109};
    private static final Call[] K317 = new Call[]{TinyExpressionP4Parser::e95,TinyExpressionP4Parser::e96};
    private static final String[] K318 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3490:3500:body/2/tokenRef/capture/0"};
    private static final Call[] K319 = new Call[]{TinyExpressionP4Parser::e103,TinyExpressionP4Parser::e104,TinyExpressionP4Parser::e106};
    private static final String[] K320 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3547:3559:body/4/0/1/0/ruleRef/capture/0"};
    private static final String[] K321 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3576:3592:body/4/0/2/ruleRef/capture/0"};
    private static final String[] K322 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:3608:3619:body/5/0/ruleRef/capture/0"};
    private static final Call[] K323 = new Call[]{TinyExpressionP4Parser::e111,TinyExpressionP4Parser::e113};
    private static final Label[] K324 = new Label[]{Label.of("'number'"),Label.of("'Number'"),Label.of("'string'"),Label.of("'String'"),Label.of("'boolean'"),Label.of("'Boolean'"),Label.of("'object'"),Label.of("'Object'")};
    private static final Call[] K325 = new Call[]{TinyExpressionP4Parser::e115,TinyExpressionP4Parser::e116,TinyExpressionP4Parser::e117,TinyExpressionP4Parser::e118,TinyExpressionP4Parser::e119,TinyExpressionP4Parser::e120,TinyExpressionP4Parser::e121,TinyExpressionP4Parser::e122};
    private static final Label[] K326 = new Label[]{Label.of("'number'")};
    private static final Label K327 = Label.of("'number'");
    private static final Label[] K328 = new Label[]{Label.of("'Number'")};
    private static final Label K329 = Label.of("'Number'");
    private static final Label K330 = Label.of("'string'");
    private static final Label[] K331 = new Label[]{Label.of("'String'")};
    private static final Label K332 = Label.of("'String'");
    private static final Label K333 = Label.of("'boolean'");
    private static final Label[] K334 = new Label[]{Label.of("'Boolean'")};
    private static final Label K335 = Label.of("'Boolean'");
    private static final Label K336 = Label.of("'object'");
    private static final Label[] K337 = new Label[]{Label.of("'Object'")};
    private static final Label K338 = Label.of("'Object'");
    private static final Call[] K339 = new Call[]{TinyExpressionP4Parser::e124,TinyExpressionP4Parser::e126};
    private static final Label[] K340 = new Label[]{Label.of("'number'"),Label.of("'Number'"),Label.of("'float'"),Label.of("'Float'")};
    private static final Call[] K341 = new Call[]{TinyExpressionP4Parser::e128,TinyExpressionP4Parser::e129,TinyExpressionP4Parser::e130,TinyExpressionP4Parser::e131};
    private static final Label[] K342 = new Label[]{Label.of("'float'")};
    private static final Label K343 = Label.of("'float'");
    private static final Label[] K344 = new Label[]{Label.of("'Float'")};
    private static final Label K345 = Label.of("'Float'");
    private static final Call[] K346 = new Call[]{TinyExpressionP4Parser::e133,TinyExpressionP4Parser::e135};
    private static final Label[] K347 = new Label[]{Label.of("'string'"),Label.of("'String'")};
    private static final Call[] K348 = new Call[]{TinyExpressionP4Parser::e137,TinyExpressionP4Parser::e138};
    private static final Call[] K349 = new Call[]{TinyExpressionP4Parser::e140,TinyExpressionP4Parser::e142};
    private static final Label[] K350 = new Label[]{Label.of("'boolean'"),Label.of("'Boolean'")};
    private static final Call[] K351 = new Call[]{TinyExpressionP4Parser::e144,TinyExpressionP4Parser::e145};
    private static final Call[] K352 = new Call[]{TinyExpressionP4Parser::e147,TinyExpressionP4Parser::e149};
    private static final Label[] K353 = new Label[]{Label.of("'object'"),Label.of("'Object'")};
    private static final Call[] K354 = new Call[]{TinyExpressionP4Parser::e151,TinyExpressionP4Parser::e152};
    private static final Call[] K355 = new Call[]{TinyExpressionP4Parser::e154,TinyExpressionP4Parser::e155,TinyExpressionP4Parser::e156};
    private static final Label[] K356 = new Label[]{Label.of("'if'")};
    private static final Label K357 = Label.of("'if'");
    private static final Label[] K358 = new Label[]{Label.of("'not'")};
    private static final Label K359 = Label.of("'not'");
    private static final Label[] K360 = new Label[]{Label.of("'exists'")};
    private static final Label K361 = Label.of("'exists'");
    private static final Call[] K362 = new Call[]{TinyExpressionP4Parser::e158,TinyExpressionP4Parser::e159,TinyExpressionP4Parser::e160};
    private static final Label[] K363 = new Label[]{Label.of("'description'")};
    private static final Label K364 = Label.of("'description'");
    private static final Label K365 = Label.of("'='");
    private static final Delimiters[] K366 = new Delimiters[]{K217,K217,K217,K217,K217};
    private static final Trivia K367 = new Trivia(K217,null,K366,null,null);
    private static final Call[] K368 = new Call[]{TinyExpressionP4Parser::e162,TinyExpressionP4Parser::e163,TinyExpressionP4Parser::e164,TinyExpressionP4Parser::e165,TinyExpressionP4Parser::e167};
    private static final Label[] K369 = new Label[]{Label.of("'@'")};
    private static final Label K370 = Label.of("'@'");
    private static final Label[] K371 = new Label[]{Label.of("IdentifierParser")};
    private static final Label[] K372 = new Label[]{Label.of("'('")};
    private static final Label K373 = Label.of("'('");
    private static final Label[] K374 = new Label[]{Label.of("')'")};
    private static final Label K375 = Label.of("')'");
    private static final Call[] K376 = new Call[]{TinyExpressionP4Parser::e169,TinyExpressionP4Parser::e170};
    private static final Label[] K377 = new Label[]{Label.of("','")};
    private static final Label[] K378 = new Label[]{Label.of("','"),Label.of("'='")};
    private static final Call[] K379 = new Call[]{TinyExpressionP4Parser::e172,TinyExpressionP4Parser::e173};
    private static final Label K380 = Label.of("','");
    private static final Call[] K381 = new Call[]{TinyExpressionP4Parser::e175,TinyExpressionP4Parser::e176,TinyExpressionP4Parser::e177};
    private static final Call[] K382 = new Call[]{TinyExpressionP4Parser::e179,TinyExpressionP4Parser::e180,TinyExpressionP4Parser::e181,TinyExpressionP4Parser::e182};
    private static final Delimiters[] K383 = new Delimiters[]{K217,K217,K217,K217,K217,K217,K217,K217};
    private static final Trivia K384 = new Trivia(K217,null,K383,null,null);
    private static final Call[] K385 = new Call[]{TinyExpressionP4Parser::e184,TinyExpressionP4Parser::e185,TinyExpressionP4Parser::e186,TinyExpressionP4Parser::e187,TinyExpressionP4Parser::e189,TinyExpressionP4Parser::e190,TinyExpressionP4Parser::e191,TinyExpressionP4Parser::e192};
    private static final String[] K386 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4668:4678:body/1/tokenRef/capture/0"};
    private static final String[] K387 = new String[]{"methodName"};
    private static final String[] K388 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4707:4723:body/3/0/ruleRef/capture/0"};
    private static final String[] K389 = new String[]{"parameters"};
    private static final Label[] K390 = new Label[]{Label.of("'{'")};
    private static final Label K391 = Label.of("'{'");
    private static final String[] K392 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4760:4776:body/6/ruleRef/capture/0"};
    private static final Label[] K393 = new Label[]{Label.of("'}'")};
    private static final Label K394 = Label.of("'}'");
    private static final Call[] K395 = new Call[]{TinyExpressionP4Parser::e194,TinyExpressionP4Parser::e195,TinyExpressionP4Parser::e196,TinyExpressionP4Parser::e197,TinyExpressionP4Parser::e199,TinyExpressionP4Parser::e200,TinyExpressionP4Parser::e201,TinyExpressionP4Parser::e202};
    private static final String[] K396 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:4994:5004:body/1/tokenRef/capture/0"};
    private static final String[] K397 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5033:5049:body/3/0/ruleRef/capture/0"};
    private static final String[] K398 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5086:5102:body/6/ruleRef/capture/0"};
    private static final Call[] K399 = new Call[]{TinyExpressionP4Parser::e204,TinyExpressionP4Parser::e205,TinyExpressionP4Parser::e206,TinyExpressionP4Parser::e207,TinyExpressionP4Parser::e209,TinyExpressionP4Parser::e210,TinyExpressionP4Parser::e211,TinyExpressionP4Parser::e212};
    private static final String[] K400 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5323:5333:body/1/tokenRef/capture/0"};
    private static final String[] K401 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5362:5378:body/3/0/ruleRef/capture/0"};
    private static final String[] K402 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5415:5432:body/6/ruleRef/capture/0"};
    private static final Call[] K403 = new Call[]{TinyExpressionP4Parser::e214,TinyExpressionP4Parser::e215,TinyExpressionP4Parser::e216,TinyExpressionP4Parser::e217,TinyExpressionP4Parser::e219,TinyExpressionP4Parser::e220,TinyExpressionP4Parser::e221,TinyExpressionP4Parser::e222};
    private static final String[] K404 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5650:5660:body/1/tokenRef/capture/0"};
    private static final String[] K405 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5689:5705:body/3/0/ruleRef/capture/0"};
    private static final String[] K406 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5742:5758:body/6/ruleRef/capture/0"};
    private static final Call[] K407 = new Call[]{TinyExpressionP4Parser::e224,TinyExpressionP4Parser::e225};
    private static final String[] K408 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5855:5870:body/0/ruleRef/capture/0"};
    private static final String[] K409 = new String[]{"values"};
    private static final Call[] K410 = new Call[]{TinyExpressionP4Parser::e227,TinyExpressionP4Parser::e228};
    private static final String[] K411 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:5885:5900:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K412 = new Call[]{TinyExpressionP4Parser::e230,TinyExpressionP4Parser::e231,TinyExpressionP4Parser::e232};
    private static final String[] K413 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6028:6038:body/1/tokenRef/capture/0"};
    private static final String[] K414 = new String[]{"paramName"};
    private static final Call[] K415 = new Call[]{TinyExpressionP4Parser::e234,TinyExpressionP4Parser::e235};
    private static final String[] K416 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6057:6067:body/2/0/1/ruleRef/capture/0"};
    private static final String[] K417 = new String[]{"type"};
    private static final Call[] K418 = new Call[]{TinyExpressionP4Parser::e237,TinyExpressionP4Parser::e238};
    private static final Delimiters[] K419 = new Delimiters[]{K217};
    private static final Trivia K420 = new Trivia(K217,null,K419,null,null);
    private static final Call[] K421 = new Call[]{TinyExpressionP4Parser::e240};
    private static final Call[] K422 = new Call[]{TinyExpressionP4Parser::e242};
    private static final Call[] K423 = new Call[]{TinyExpressionP4Parser::e244};
    private static final int[] K424 = new int[]{9,13,32,32,47,47,102,102,110,110};
    private static final int[] K425 = new int[]{26};
    private static final Guard K426 = new Guard(K424,K50,null,K425,1);
    private static final int[] K427 = new int[]{9,13,32,32,47,47,115,115};
    private static final Label[] K428 = new Label[]{null,Label.of("'string'")};
    private static final int[] K429 = new int[]{27};
    private static final Guard K430 = new Guard(K427,K428,K217,K429,1);
    private static final int[] K431 = new int[]{9,13,32,32,47,47,98,98};
    private static final Label[] K432 = new Label[]{null,Label.of("'boolean'")};
    private static final int[] K433 = new int[]{28};
    private static final Guard K434 = new Guard(K431,K432,K217,K433,1);
    private static final int[] K435 = new int[]{9,13,32,32,47,47,111,111};
    private static final Label[] K436 = new Label[]{null,Label.of("'object'")};
    private static final int[] K437 = new int[]{29};
    private static final Guard K438 = new Guard(K435,K436,K217,K437,1);
    private static final Guard[] K439 = new Guard[]{K426,K430,K434,K438};
    private static final Call[] K440 = new Call[]{TinyExpressionP4Parser::e246,TinyExpressionP4Parser::e247,TinyExpressionP4Parser::e248,TinyExpressionP4Parser::e249};
    private static final Call[] K441 = new Call[]{TinyExpressionP4Parser::e251,TinyExpressionP4Parser::e252,TinyExpressionP4Parser::e257,TinyExpressionP4Parser::e258,TinyExpressionP4Parser::e260,TinyExpressionP4Parser::e267,TinyExpressionP4Parser::e268,TinyExpressionP4Parser::e270};
    private static final Label[] K442 = new Label[]{Label.of("'external'")};
    private static final Label K443 = Label.of("'external'");
    private static final Label[] K444 = new Label[]{Label.of("'returning'")};
    private static final Label[] K445 = new Label[]{Label.of("'returning'"),Label.of("'as'")};
    private static final Call[] K446 = new Call[]{TinyExpressionP4Parser::e254,TinyExpressionP4Parser::e255};
    private static final Label K447 = Label.of("'returning'");
    private static final Label[] K448 = new Label[]{Label.of("':'")};
    private static final Label K449 = Label.of("':'");
    private static final Call[] K450 = new Call[]{TinyExpressionP4Parser::e262,TinyExpressionP4Parser::e266};
    private static final Call[] K451 = new Call[]{TinyExpressionP4Parser::e263,TinyExpressionP4Parser::e264,TinyExpressionP4Parser::e265};
    private static final String[] K452 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6716:6725:body/4/0/0/0/ruleRef/capture/0"};
    private static final String[] K453 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6741:6751:body/4/0/0/2/tokenRef/capture/0"};
    private static final String[] K454 = new String[]{"name"};
    private static final String[] K455 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6760:6770:body/4/0/1/tokenRef/capture/0"};
    private static final String[] K456 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:6789:6798:body/6/0/ruleRef/capture/0"};
    private static final String[] K457 = new String[]{"args"};
    private static final Call[] K458 = new Call[]{TinyExpressionP4Parser::e272,TinyExpressionP4Parser::e273,TinyExpressionP4Parser::e286,TinyExpressionP4Parser::e293,TinyExpressionP4Parser::e294,TinyExpressionP4Parser::e296};
    private static final Call[] K459 = new Call[]{TinyExpressionP4Parser::e275,TinyExpressionP4Parser::e284};
    private static final Label[] K460 = new Label[]{Label.of("'number'"),Label.of("'float'"),Label.of("':'")};
    private static final Call[] K461 = new Call[]{TinyExpressionP4Parser::e276,TinyExpressionP4Parser::e281,TinyExpressionP4Parser::e282};
    private static final Call[] K462 = new Call[]{TinyExpressionP4Parser::e278,TinyExpressionP4Parser::e279};
    private static final Call[] K463 = new Call[]{TinyExpressionP4Parser::e288,TinyExpressionP4Parser::e292};
    private static final Call[] K464 = new Call[]{TinyExpressionP4Parser::e289,TinyExpressionP4Parser::e290,TinyExpressionP4Parser::e291};
    private static final String[] K465 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7007:7016:body/2/0/0/0/ruleRef/capture/0"};
    private static final String[] K466 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7032:7042:body/2/0/0/2/tokenRef/capture/0"};
    private static final String[] K467 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7051:7061:body/2/0/1/tokenRef/capture/0"};
    private static final String[] K468 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7080:7089:body/4/0/ruleRef/capture/0"};
    private static final Call[] K469 = new Call[]{TinyExpressionP4Parser::e298,TinyExpressionP4Parser::e299,TinyExpressionP4Parser::e304,TinyExpressionP4Parser::e305,TinyExpressionP4Parser::e307,TinyExpressionP4Parser::e314,TinyExpressionP4Parser::e315,TinyExpressionP4Parser::e317};
    private static final Call[] K470 = new Call[]{TinyExpressionP4Parser::e301,TinyExpressionP4Parser::e302};
    private static final Call[] K471 = new Call[]{TinyExpressionP4Parser::e309,TinyExpressionP4Parser::e313};
    private static final Call[] K472 = new Call[]{TinyExpressionP4Parser::e310,TinyExpressionP4Parser::e311,TinyExpressionP4Parser::e312};
    private static final String[] K473 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7284:7293:body/4/0/0/0/ruleRef/capture/0"};
    private static final String[] K474 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7309:7319:body/4/0/0/2/tokenRef/capture/0"};
    private static final String[] K475 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7328:7338:body/4/0/1/tokenRef/capture/0"};
    private static final String[] K476 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7357:7366:body/6/0/ruleRef/capture/0"};
    private static final Call[] K477 = new Call[]{TinyExpressionP4Parser::e319,TinyExpressionP4Parser::e320,TinyExpressionP4Parser::e325,TinyExpressionP4Parser::e326,TinyExpressionP4Parser::e328,TinyExpressionP4Parser::e335,TinyExpressionP4Parser::e336,TinyExpressionP4Parser::e338};
    private static final Call[] K478 = new Call[]{TinyExpressionP4Parser::e322,TinyExpressionP4Parser::e323};
    private static final Call[] K479 = new Call[]{TinyExpressionP4Parser::e330,TinyExpressionP4Parser::e334};
    private static final Call[] K480 = new Call[]{TinyExpressionP4Parser::e331,TinyExpressionP4Parser::e332,TinyExpressionP4Parser::e333};
    private static final String[] K481 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7561:7570:body/4/0/0/0/ruleRef/capture/0"};
    private static final String[] K482 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7586:7596:body/4/0/0/2/tokenRef/capture/0"};
    private static final String[] K483 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7605:7615:body/4/0/1/tokenRef/capture/0"};
    private static final String[] K484 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:7634:7643:body/6/0/ruleRef/capture/0"};
    private static final int[] K485 = new int[]{9,13,32,32,47,47,99,99};
    private static final Label[] K486 = new Label[]{null,Label.of("'call'"),Label.of("'internal'")};
    private static final Guard K487 = new Guard(K485,K486,K217,K234,0);
    private static final Guard[] K488 = new Guard[]{K487,null};
    private static final Call[] K489 = new Call[]{TinyExpressionP4Parser::e340,TinyExpressionP4Parser::e344};
    private static final Call[] K490 = new Call[]{TinyExpressionP4Parser::e341,TinyExpressionP4Parser::e342};
    private static final Label[] K491 = new Label[]{Label.of("'call'")};
    private static final Label K492 = Label.of("'call'");
    private static final Label[] K493 = new Label[]{Label.of("'internal'")};
    private static final Label K494 = Label.of("'internal'");
    private static final Call[] K495 = new Call[]{TinyExpressionP4Parser::e346,TinyExpressionP4Parser::e347,TinyExpressionP4Parser::e348,TinyExpressionP4Parser::e349,TinyExpressionP4Parser::e351};
    private static final String[] K496 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8020:8030:body/1/tokenRef/capture/0"};
    private static final String[] K497 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8043:8052:body/3/0/ruleRef/capture/0"};
    private static final Call[] K498 = new Call[]{TinyExpressionP4Parser::e353,TinyExpressionP4Parser::e354,TinyExpressionP4Parser::e355,TinyExpressionP4Parser::e356,TinyExpressionP4Parser::e357};
    private static final String[] K499 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8231:8248:body/0/ruleRef/capture/0"};
    private static final String[] K500 = new String[]{"condition"};
    private static final Label[] K501 = new Label[]{Label.of("'?'")};
    private static final Label K502 = Label.of("'?'");
    private static final String[] K503 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8264:8280:body/2/ruleRef/capture/0"};
    private static final String[] K504 = new String[]{"thenExpr"};
    private static final String[] K505 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8295:8311:body/4/ruleRef/capture/0"};
    private static final String[] K506 = new String[]{"elseExpr"};
    private static final Call[] K507 = new Call[]{TinyExpressionP4Parser::e359,TinyExpressionP4Parser::e360,TinyExpressionP4Parser::e361,TinyExpressionP4Parser::e362};
    private static final String[] K508 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8407:8422:body/0/ruleRef/capture/0"};
    private static final String[] K509 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8436:8456:body/1/ruleRef/capture/0"};
    private static final String[] K510 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8470:8496:body/2/ruleRef/capture/0"};
    private static final String[] K511 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8510:8520:body/3/ruleRef/capture/0"};
    private static final Call[] K512 = new Call[]{TinyExpressionP4Parser::e364,TinyExpressionP4Parser::e365};
    private static final String[] K513 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8590:8608:body/0/ruleRef/capture/0"};
    private static final Call[] K514 = new Call[]{TinyExpressionP4Parser::e367,TinyExpressionP4Parser::e368};
    private static final String[] K515 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8623:8641:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K516 = new Call[]{TinyExpressionP4Parser::e370,TinyExpressionP4Parser::e371};
    private static final String[] K517 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8846:8856:body/0/ruleRef/capture/0"};
    private static final String[] K518 = new String[]{"left"};
    private static final Label[] K519 = new Label[]{Label.of("NumberExpressionRepeat0Parser")};
    private static final Call[] K520 = new Call[]{TinyExpressionP4Parser::e373,TinyExpressionP4Parser::e374};
    private static final String[] K521 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8865:8870:body/1/0/0/ruleRef/capture/0"};
    private static final String[] K522 = new String[]{"op"};
    private static final String[] K523 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:8875:8885:body/1/0/1/ruleRef/capture/0"};
    private static final String[] K524 = new String[]{"right"};
    private static final Call[] K525 = new Call[]{TinyExpressionP4Parser::e376,TinyExpressionP4Parser::e377};
    private static final String[] K526 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9001:9013:body/0/ruleRef/capture/0"};
    private static final Label[] K527 = new Label[]{Label.of("NumberTermRepeat0Parser")};
    private static final Call[] K528 = new Call[]{TinyExpressionP4Parser::e379,TinyExpressionP4Parser::e380};
    private static final String[] K529 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9022:9027:body/1/0/0/ruleRef/capture/0"};
    private static final String[] K530 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9032:9044:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K531 = new Call[]{TinyExpressionP4Parser::e382,TinyExpressionP4Parser::e383};
    private static final Label[] K532 = new Label[]{Label.of("'+'")};
    private static final Label K533 = Label.of("'+'");
    private static final Label[] K534 = new Label[]{Label.of("'-'")};
    private static final Label K535 = Label.of("'-'");
    private static final Call[] K536 = new Call[]{TinyExpressionP4Parser::e385,TinyExpressionP4Parser::e386};
    private static final Label[] K537 = new Label[]{Label.of("'*'")};
    private static final Label K538 = Label.of("'*'");
    private static final Label[] K539 = new Label[]{Label.of("'/'")};
    private static final Label K540 = Label.of("'/'");
    private static final Label[] K541 = new Label[]{null,Label.of("'sin'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K542 = new int[]{45};
    private static final Guard K543 = new Guard(K427,K541,K217,K542,1);
    private static final Label[] K544 = new Label[]{null,Label.of("'cos'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K545 = new int[]{46};
    private static final Guard K546 = new Guard(K485,K544,K217,K545,1);
    private static final int[] K547 = new int[]{9,13,32,32,47,47,116,116};
    private static final Label[] K548 = new Label[]{null,Label.of("'tan'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K549 = new int[]{47};
    private static final Guard K550 = new Guard(K547,K548,K217,K549,1);
    private static final Label[] K551 = new Label[]{null,Label.of("'sqrt'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K552 = new int[]{48};
    private static final Guard K553 = new Guard(K427,K551,K217,K552,1);
    private static final int[] K554 = new int[]{9,13,32,32,47,47,109,109};
    private static final Label[] K555 = new Label[]{null,Label.of("'min'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K556 = new int[]{49};
    private static final Guard K557 = new Guard(K554,K555,K217,K556,1);
    private static final Label[] K558 = new Label[]{null,Label.of("'max'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K559 = new int[]{50};
    private static final Guard K560 = new Guard(K554,K558,K217,K559,1);
    private static final int[] K561 = new int[]{9,13,32,32,47,47,114,114};
    private static final Label[] K562 = new Label[]{null,Label.of("'random'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K563 = new int[]{51};
    private static final Guard K564 = new Guard(K561,K562,K217,K563,1);
    private static final int[] K565 = new int[]{9,13,32,32,47,47,97,97};
    private static final Label[] K566 = new Label[]{null,Label.of("'abs'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K567 = new int[]{52};
    private static final Guard K568 = new Guard(K565,K566,K217,K567,1);
    private static final Label[] K569 = new Label[]{null,Label.of("'round'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K570 = new int[]{53};
    private static final Guard K571 = new Guard(K561,K569,K217,K570,1);
    private static final Label[] K572 = new Label[]{null,Label.of("'ceil'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K573 = new int[]{54};
    private static final Guard K574 = new Guard(K485,K572,K217,K573,1);
    private static final int[] K575 = new int[]{9,13,32,32,47,47,102,102};
    private static final Label[] K576 = new Label[]{null,Label.of("'floor'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K577 = new int[]{55};
    private static final Guard K578 = new Guard(K575,K576,K217,K577,1);
    private static final int[] K579 = new int[]{9,13,32,32,47,47,112,112};
    private static final Label[] K580 = new Label[]{null,Label.of("'pow'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K581 = new int[]{56};
    private static final Guard K582 = new Guard(K579,K580,K217,K581,1);
    private static final int[] K583 = new int[]{9,13,32,32,47,47,108,108};
    private static final Label[] K584 = new Label[]{null,Label.of("'log'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K585 = new int[]{57};
    private static final Guard K586 = new Guard(K583,K584,K217,K585,1);
    private static final int[] K587 = new int[]{9,13,32,32,47,47,101,101};
    private static final Label[] K588 = new Label[]{null,Label.of("'exp'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K589 = new int[]{58};
    private static final Guard K590 = new Guard(K587,K588,K217,K589,1);
    private static final Guard[] K591 = new Guard[]{K543,K546,K550,K553,K557,K560,K564,K568,K571,K574,K578,K582,K586,K590};
    private static final Call[] K592 = new Call[]{TinyExpressionP4Parser::e388,TinyExpressionP4Parser::e389,TinyExpressionP4Parser::e390,TinyExpressionP4Parser::e391,TinyExpressionP4Parser::e392,TinyExpressionP4Parser::e393,TinyExpressionP4Parser::e394,TinyExpressionP4Parser::e395,TinyExpressionP4Parser::e396,TinyExpressionP4Parser::e397,TinyExpressionP4Parser::e398,TinyExpressionP4Parser::e399,TinyExpressionP4Parser::e400,TinyExpressionP4Parser::e401};
    private static final Delimiters[] K593 = new Delimiters[]{K217,K217,K217,K217};
    private static final Trivia K594 = new Trivia(K217,null,K593,null,null);
    private static final Call[] K595 = new Call[]{TinyExpressionP4Parser::e403,TinyExpressionP4Parser::e404,TinyExpressionP4Parser::e405,TinyExpressionP4Parser::e406};
    private static final Label[] K596 = new Label[]{Label.of("'sin'")};
    private static final Label K597 = Label.of("'sin'");
    private static final String[] K598 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9530:9548:body/2/ruleRef/capture/0"};
    private static final String[] K599 = new String[]{"arg"};
    private static final Call[] K600 = new Call[]{TinyExpressionP4Parser::e408,TinyExpressionP4Parser::e409,TinyExpressionP4Parser::e410,TinyExpressionP4Parser::e411};
    private static final Label[] K601 = new Label[]{Label.of("'cos'")};
    private static final Label K602 = Label.of("'cos'");
    private static final String[] K603 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9623:9641:body/2/ruleRef/capture/0"};
    private static final Call[] K604 = new Call[]{TinyExpressionP4Parser::e413,TinyExpressionP4Parser::e414,TinyExpressionP4Parser::e415,TinyExpressionP4Parser::e416};
    private static final Label[] K605 = new Label[]{Label.of("'tan'")};
    private static final Label K606 = Label.of("'tan'");
    private static final String[] K607 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9716:9734:body/2/ruleRef/capture/0"};
    private static final Call[] K608 = new Call[]{TinyExpressionP4Parser::e418,TinyExpressionP4Parser::e419,TinyExpressionP4Parser::e420,TinyExpressionP4Parser::e421};
    private static final Label[] K609 = new Label[]{Label.of("'sqrt'")};
    private static final Label K610 = Label.of("'sqrt'");
    private static final String[] K611 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9812:9830:body/2/ruleRef/capture/0"};
    private static final Call[] K612 = new Call[]{TinyExpressionP4Parser::e423,TinyExpressionP4Parser::e424,TinyExpressionP4Parser::e425,TinyExpressionP4Parser::e426,TinyExpressionP4Parser::e430};
    private static final Label[] K613 = new Label[]{Label.of("'min'")};
    private static final Label K614 = Label.of("'min'");
    private static final String[] K615 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9913:9931:body/2/ruleRef/capture/0"};
    private static final String[] K616 = new String[]{"first"};
    private static final Call[] K617 = new Call[]{TinyExpressionP4Parser::e428,TinyExpressionP4Parser::e429};
    private static final String[] K618 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:9945:9963:body/3/0/1/ruleRef/capture/0"};
    private static final String[] K619 = new String[]{"rest"};
    private static final Call[] K620 = new Call[]{TinyExpressionP4Parser::e432,TinyExpressionP4Parser::e433,TinyExpressionP4Parser::e434,TinyExpressionP4Parser::e435,TinyExpressionP4Parser::e439};
    private static final Label[] K621 = new Label[]{Label.of("'max'")};
    private static final Label K622 = Label.of("'max'");
    private static final String[] K623 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10049:10067:body/2/ruleRef/capture/0"};
    private static final Call[] K624 = new Call[]{TinyExpressionP4Parser::e437,TinyExpressionP4Parser::e438};
    private static final String[] K625 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10081:10099:body/3/0/1/ruleRef/capture/0"};
    private static final Call[] K626 = new Call[]{TinyExpressionP4Parser::e441,TinyExpressionP4Parser::e442,TinyExpressionP4Parser::e443};
    private static final Label[] K627 = new Label[]{Label.of("'random'")};
    private static final Label K628 = Label.of("'random'");
    private static final Call[] K629 = new Call[]{TinyExpressionP4Parser::e445,TinyExpressionP4Parser::e446,TinyExpressionP4Parser::e447,TinyExpressionP4Parser::e448};
    private static final Label[] K630 = new Label[]{Label.of("'abs'")};
    private static final Label K631 = Label.of("'abs'");
    private static final String[] K632 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10241:10259:body/2/ruleRef/capture/0"};
    private static final Call[] K633 = new Call[]{TinyExpressionP4Parser::e450,TinyExpressionP4Parser::e451,TinyExpressionP4Parser::e452,TinyExpressionP4Parser::e453};
    private static final Label[] K634 = new Label[]{Label.of("'round'")};
    private static final Label K635 = Label.of("'round'");
    private static final String[] K636 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10340:10358:body/2/ruleRef/capture/0"};
    private static final Call[] K637 = new Call[]{TinyExpressionP4Parser::e455,TinyExpressionP4Parser::e456,TinyExpressionP4Parser::e457,TinyExpressionP4Parser::e458};
    private static final Label[] K638 = new Label[]{Label.of("'ceil'")};
    private static final Label K639 = Label.of("'ceil'");
    private static final String[] K640 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10436:10454:body/2/ruleRef/capture/0"};
    private static final Call[] K641 = new Call[]{TinyExpressionP4Parser::e460,TinyExpressionP4Parser::e461,TinyExpressionP4Parser::e462,TinyExpressionP4Parser::e463};
    private static final Label[] K642 = new Label[]{Label.of("'floor'")};
    private static final Label K643 = Label.of("'floor'");
    private static final String[] K644 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10535:10553:body/2/ruleRef/capture/0"};
    private static final Call[] K645 = new Call[]{TinyExpressionP4Parser::e465,TinyExpressionP4Parser::e466,TinyExpressionP4Parser::e467,TinyExpressionP4Parser::e468,TinyExpressionP4Parser::e469,TinyExpressionP4Parser::e470};
    private static final Label[] K646 = new Label[]{Label.of("'pow'")};
    private static final Label K647 = Label.of("'pow'");
    private static final String[] K648 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10639:10657:body/2/ruleRef/capture/0"};
    private static final String[] K649 = new String[]{"base"};
    private static final String[] K650 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10668:10686:body/4/ruleRef/capture/0"};
    private static final String[] K651 = new String[]{"exponent"};
    private static final Call[] K652 = new Call[]{TinyExpressionP4Parser::e472,TinyExpressionP4Parser::e473,TinyExpressionP4Parser::e474,TinyExpressionP4Parser::e475};
    private static final Label[] K653 = new Label[]{Label.of("'log'")};
    private static final Label K654 = Label.of("'log'");
    private static final String[] K655 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10766:10784:body/2/ruleRef/capture/0"};
    private static final Call[] K656 = new Call[]{TinyExpressionP4Parser::e477,TinyExpressionP4Parser::e478,TinyExpressionP4Parser::e479,TinyExpressionP4Parser::e480};
    private static final Label[] K657 = new Label[]{Label.of("'exp'")};
    private static final Label K658 = Label.of("'exp'");
    private static final String[] K659 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:10859:10877:body/2/ruleRef/capture/0"};
    private static final Call[] K660 = new Call[]{TinyExpressionP4Parser::e482,TinyExpressionP4Parser::e483,TinyExpressionP4Parser::e484,TinyExpressionP4Parser::e485,TinyExpressionP4Parser::e486,TinyExpressionP4Parser::e487};
    private static final Label[] K661 = new Label[]{Label.of("'toNum'")};
    private static final Label K662 = Label.of("'toNum'");
    private static final String[] K663 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11052:11068:body/2/ruleRef/capture/0"};
    private static final String[] K664 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11080:11098:body/4/ruleRef/capture/0"};
    private static final String[] K665 = new String[]{"defaultValue"};
    private static final int[] K666 = new int[]{9,13,32,32,40,40,47,47};
    private static final Label[] K667 = new Label[]{null,Label.of("'('"),Label.of("'?'"),Label.of("':'"),Label.of("')'")};
    private static final int[] K668 = new int[]{108};
    private static final Guard K669 = new Guard(K666,K667,K217,K668,1);
    private static final Label[] K670 = new Label[]{null,Label.of("'match'"),Label.of("'{'"),Label.of("','"),Label.of("'}'")};
    private static final int[] K671 = new int[]{109};
    private static final Guard K672 = new Guard(K554,K670,K217,K671,1);
    private static final int[] K673 = new int[]{9,13,32,32,47,47,105,105};
    private static final Label[] K674 = new Label[]{null,Label.of("'if'"),Label.of("'('"),Label.of("')'"),Label.of("'{'"),Label.of("'}'"),Label.of("'else'")};
    private static final int[] K675 = new int[]{106};
    private static final Guard K676 = new Guard(K673,K674,K217,K675,1);
    private static final int[] K677 = new int[]{9,13,32,32,47,47,97,97,99,99,101,102,108,109,112,112,114,116};
    private static final Label[] K678 = new Label[]{null,Label.of("'sin'"),Label.of("'('"),Label.of("')'"),Label.of("'cos'"),Label.of("'tan'"),Label.of("'sqrt'"),Label.of("'min'"),Label.of("'max'"),Label.of("'random'"),Label.of("'abs'"),Label.of("'round'"),Label.of("'ceil'"),Label.of("'floor'"),Label.of("'pow'"),Label.of("','"),Label.of("'log'"),Label.of("'exp'")};
    private static final int[] K679 = new int[]{44,45};
    private static final Guard K680 = new Guard(K677,K678,K217,K679,2);
    private static final Label[] K681 = new Label[]{null,Label.of("'toNum'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K682 = new int[]{59};
    private static final Guard K683 = new Guard(K547,K681,K217,K682,1);
    private static final int[] K684 = new int[]{9,13,32,32,36,36,47,47};
    private static final Label[] K685 = new Label[]{null,Label.of("'$'"),Label.of("'.length'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K686 = new int[]{69,121};
    private static final Guard K687 = new Guard(K684,K685,K217,K686,2);
    private static final Label[] K688 = new Label[]{null,Label.of("'len'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K689 = new int[]{65};
    private static final Guard K690 = new Guard(K583,K688,K217,K689,1);
    private static final Label[] K691 = new Label[]{null,Label.of("'length'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K692 = new int[]{64};
    private static final Guard K693 = new Guard(K583,K691,K217,K692,1);
    private static final Label[] K694 = new Label[]{null,Label.of("'external'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K695 = new int[]{32};
    private static final Guard K696 = new Guard(K587,K694,K217,K695,1);
    private static final Label[] K697 = new Label[]{null,Label.of("'$'")};
    private static final int[] K698 = new int[]{121};
    private static final Guard K699 = new Guard(K684,K697,K217,K698,1);
    private static final int[] K700 = new int[]{9,13,32,32,47,47,99,99,105,105};
    private static final Label[] K701 = new Label[]{null,Label.of("'call'"),Label.of("'internal'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K702 = new int[]{36,35};
    private static final Guard K703 = new Guard(K700,K701,K217,K702,2);
    private static final Label[] K704 = new Label[]{null,Label.of("'('"),Label.of("')'")};
    private static final Guard K705 = new Guard(K666,K704,K217,K234,0);
    private static final Guard[] K706 = new Guard[]{K669,K672,K676,K680,K683,K687,K690,K693,K696,null,K699,K703,K705};
    private static final Call[] K707 = new Call[]{TinyExpressionP4Parser::e489,TinyExpressionP4Parser::e490,TinyExpressionP4Parser::e491,TinyExpressionP4Parser::e492,TinyExpressionP4Parser::e493,TinyExpressionP4Parser::e494,TinyExpressionP4Parser::e495,TinyExpressionP4Parser::e496,TinyExpressionP4Parser::e497,TinyExpressionP4Parser::e498,TinyExpressionP4Parser::e499,TinyExpressionP4Parser::e500,TinyExpressionP4Parser::e501};
    private static final Label[] K708 = new Label[]{Label.of("NumberParser")};
    private static final Label K709 = Label.of("NumberParser");
    private static final Call[] K710 = new Call[]{TinyExpressionP4Parser::e502,TinyExpressionP4Parser::e503,TinyExpressionP4Parser::e504};
    private static final Call[] K711 = new Call[]{TinyExpressionP4Parser::e506,TinyExpressionP4Parser::e507,TinyExpressionP4Parser::e508,TinyExpressionP4Parser::e509};
    private static final Label[] K712 = new Label[]{Label.of("'toUpperCase'")};
    private static final Label K713 = Label.of("'toUpperCase'");
    private static final String[] K714 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11596:11612:body/2/ruleRef/capture/0"};
    private static final Call[] K715 = new Call[]{TinyExpressionP4Parser::e511,TinyExpressionP4Parser::e512,TinyExpressionP4Parser::e513,TinyExpressionP4Parser::e514};
    private static final Label[] K716 = new Label[]{Label.of("'toLowerCase'")};
    private static final Label K717 = Label.of("'toLowerCase'");
    private static final String[] K718 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11715:11731:body/2/ruleRef/capture/0"};
    private static final Call[] K719 = new Call[]{TinyExpressionP4Parser::e516,TinyExpressionP4Parser::e517,TinyExpressionP4Parser::e518,TinyExpressionP4Parser::e519};
    private static final Label[] K720 = new Label[]{Label.of("'trim'")};
    private static final Label K721 = Label.of("'trim'");
    private static final String[] K722 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11813:11829:body/2/ruleRef/capture/0"};
    private static final Call[] K723 = new Call[]{TinyExpressionP4Parser::e521,TinyExpressionP4Parser::e522,TinyExpressionP4Parser::e523,TinyExpressionP4Parser::e524};
    private static final Label[] K724 = new Label[]{Label.of("'length'")};
    private static final Label K725 = Label.of("'length'");
    private static final String[] K726 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:11917:11933:body/2/ruleRef/capture/0"};
    private static final Call[] K727 = new Call[]{TinyExpressionP4Parser::e526,TinyExpressionP4Parser::e527,TinyExpressionP4Parser::e528,TinyExpressionP4Parser::e529};
    private static final Label[] K728 = new Label[]{Label.of("'len'")};
    private static final Label K729 = Label.of("'len'");
    private static final String[] K730 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12015:12031:body/2/ruleRef/capture/0"};
    private static final Call[] K731 = new Call[]{TinyExpressionP4Parser::e531,TinyExpressionP4Parser::e532,TinyExpressionP4Parser::e533,TinyExpressionP4Parser::e534};
    private static final String[] K732 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12367:12378:body/0/ruleRef/capture/0"};
    private static final Label[] K733 = new Label[]{Label.of("'.toUpperCase'")};
    private static final Label K734 = Label.of("'.toUpperCase'");
    private static final Call[] K735 = new Call[]{TinyExpressionP4Parser::e536,TinyExpressionP4Parser::e537,TinyExpressionP4Parser::e538,TinyExpressionP4Parser::e539};
    private static final String[] K736 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12486:12497:body/0/ruleRef/capture/0"};
    private static final Label[] K737 = new Label[]{Label.of("'.toLowerCase'")};
    private static final Label K738 = Label.of("'.toLowerCase'");
    private static final Call[] K739 = new Call[]{TinyExpressionP4Parser::e541,TinyExpressionP4Parser::e542,TinyExpressionP4Parser::e543,TinyExpressionP4Parser::e544};
    private static final String[] K740 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12591:12602:body/0/ruleRef/capture/0"};
    private static final Label[] K741 = new Label[]{Label.of("'.trim'")};
    private static final Label K742 = Label.of("'.trim'");
    private static final Call[] K743 = new Call[]{TinyExpressionP4Parser::e546,TinyExpressionP4Parser::e547,TinyExpressionP4Parser::e548,TinyExpressionP4Parser::e549};
    private static final String[] K744 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12693:12704:body/0/ruleRef/capture/0"};
    private static final Label[] K745 = new Label[]{Label.of("'.length'")};
    private static final Label K746 = Label.of("'.length'");
    private static final Call[] K747 = new Call[]{TinyExpressionP4Parser::e551,TinyExpressionP4Parser::e552,TinyExpressionP4Parser::e553,TinyExpressionP4Parser::e554,TinyExpressionP4Parser::e555,TinyExpressionP4Parser::e556,TinyExpressionP4Parser::e560};
    private static final Label[] K748 = new Label[]{Label.of("'startsWith'")};
    private static final Label K749 = Label.of("'startsWith'");
    private static final String[] K750 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12966:12982:body/2/ruleRef/capture/0"};
    private static final String[] K751 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:12994:13010:body/4/ruleRef/capture/0"};
    private static final String[] K752 = new String[]{"patterns"};
    private static final Call[] K753 = new Call[]{TinyExpressionP4Parser::e558,TinyExpressionP4Parser::e559};
    private static final String[] K754 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13031:13047:body/5/0/1/ruleRef/capture/0"};
    private static final Call[] K755 = new Call[]{TinyExpressionP4Parser::e562,TinyExpressionP4Parser::e563,TinyExpressionP4Parser::e564,TinyExpressionP4Parser::e565,TinyExpressionP4Parser::e566,TinyExpressionP4Parser::e567,TinyExpressionP4Parser::e571};
    private static final Label[] K756 = new Label[]{Label.of("'endsWith'")};
    private static final Label K757 = Label.of("'endsWith'");
    private static final String[] K758 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13156:13172:body/2/ruleRef/capture/0"};
    private static final String[] K759 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13184:13200:body/4/ruleRef/capture/0"};
    private static final Call[] K760 = new Call[]{TinyExpressionP4Parser::e569,TinyExpressionP4Parser::e570};
    private static final String[] K761 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13221:13237:body/5/0/1/ruleRef/capture/0"};
    private static final Call[] K762 = new Call[]{TinyExpressionP4Parser::e573,TinyExpressionP4Parser::e574,TinyExpressionP4Parser::e575,TinyExpressionP4Parser::e576,TinyExpressionP4Parser::e577,TinyExpressionP4Parser::e578,TinyExpressionP4Parser::e582};
    private static final Label[] K763 = new Label[]{Label.of("'contains'")};
    private static final Label K764 = Label.of("'contains'");
    private static final String[] K765 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13346:13362:body/2/ruleRef/capture/0"};
    private static final String[] K766 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13374:13390:body/4/ruleRef/capture/0"};
    private static final Call[] K767 = new Call[]{TinyExpressionP4Parser::e580,TinyExpressionP4Parser::e581};
    private static final String[] K768 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13411:13427:body/5/0/1/ruleRef/capture/0"};
    private static final Call[] K769 = new Call[]{TinyExpressionP4Parser::e584,TinyExpressionP4Parser::e585,TinyExpressionP4Parser::e586,TinyExpressionP4Parser::e587,TinyExpressionP4Parser::e588,TinyExpressionP4Parser::e592};
    private static final String[] K770 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13509:13525:body/0/ruleRef/capture/0"};
    private static final Label[] K771 = new Label[]{Label.of("'.in'")};
    private static final Label K772 = Label.of("'.in'");
    private static final String[] K773 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13543:13559:body/3/ruleRef/capture/0"};
    private static final String[] K774 = new String[]{"candidates"};
    private static final Call[] K775 = new Call[]{TinyExpressionP4Parser::e590,TinyExpressionP4Parser::e591};
    private static final String[] K776 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13578:13594:body/4/0/1/ruleRef/capture/0"};
    private static final Call[] K777 = new Call[]{TinyExpressionP4Parser::e594,TinyExpressionP4Parser::e595,TinyExpressionP4Parser::e596,TinyExpressionP4Parser::e597,TinyExpressionP4Parser::e598,TinyExpressionP4Parser::e602};
    private static final String[] K778 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13777:13800:body/0/ruleRef/capture/0"};
    private static final Label[] K779 = new Label[]{Label.of("'.startsWith'")};
    private static final Label K780 = Label.of("'.startsWith'");
    private static final String[] K781 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13826:13842:body/3/ruleRef/capture/0"};
    private static final Call[] K782 = new Call[]{TinyExpressionP4Parser::e600,TinyExpressionP4Parser::e601};
    private static final String[] K783 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13863:13879:body/4/0/1/ruleRef/capture/0"};
    private static final Call[] K784 = new Call[]{TinyExpressionP4Parser::e604,TinyExpressionP4Parser::e605,TinyExpressionP4Parser::e606,TinyExpressionP4Parser::e607,TinyExpressionP4Parser::e608,TinyExpressionP4Parser::e612};
    private static final String[] K785 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:13977:14000:body/0/ruleRef/capture/0"};
    private static final Label[] K786 = new Label[]{Label.of("'.endsWith'")};
    private static final Label K787 = Label.of("'.endsWith'");
    private static final String[] K788 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14024:14040:body/3/ruleRef/capture/0"};
    private static final Call[] K789 = new Call[]{TinyExpressionP4Parser::e610,TinyExpressionP4Parser::e611};
    private static final String[] K790 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14061:14077:body/4/0/1/ruleRef/capture/0"};
    private static final Call[] K791 = new Call[]{TinyExpressionP4Parser::e614,TinyExpressionP4Parser::e615,TinyExpressionP4Parser::e616,TinyExpressionP4Parser::e617,TinyExpressionP4Parser::e618,TinyExpressionP4Parser::e622};
    private static final String[] K792 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14175:14198:body/0/ruleRef/capture/0"};
    private static final Label[] K793 = new Label[]{Label.of("'.contains'")};
    private static final Label K794 = Label.of("'.contains'");
    private static final String[] K795 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14222:14238:body/3/ruleRef/capture/0"};
    private static final Call[] K796 = new Call[]{TinyExpressionP4Parser::e620,TinyExpressionP4Parser::e621};
    private static final String[] K797 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14259:14275:body/4/0/1/ruleRef/capture/0"};
    private static final Label[] K798 = new Label[]{null,Label.of("'toLowerCase'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K799 = new int[]{62};
    private static final Guard K800 = new Guard(K547,K798,K217,K799,1);
    private static final Label[] K801 = new Label[]{null,Label.of("'toUpperCase'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K802 = new int[]{61};
    private static final Guard K803 = new Guard(K547,K801,K217,K802,1);
    private static final Label[] K804 = new Label[]{null,Label.of("'trim'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K805 = new int[]{63};
    private static final Guard K806 = new Guard(K547,K804,K217,K805,1);
    private static final Guard[] K807 = new Guard[]{K800,K803,K806,K699};
    private static final Call[] K808 = new Call[]{TinyExpressionP4Parser::e624,TinyExpressionP4Parser::e625,TinyExpressionP4Parser::e626,TinyExpressionP4Parser::e627};
    private static final Call[] K809 = new Call[]{TinyExpressionP4Parser::e629,TinyExpressionP4Parser::e630,TinyExpressionP4Parser::e631,TinyExpressionP4Parser::e632};
    private static final Label[] K810 = new Label[]{Label.of("'isPresent'")};
    private static final Label K811 = Label.of("'isPresent'");
    private static final String[] K812 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14560:14571:body/2/ruleRef/capture/0"};
    private static final Call[] K813 = new Call[]{TinyExpressionP4Parser::e634,TinyExpressionP4Parser::e635,TinyExpressionP4Parser::e636,TinyExpressionP4Parser::e637,TinyExpressionP4Parser::e638,TinyExpressionP4Parser::e639};
    private static final Label[] K814 = new Label[]{Label.of("'inTimeRange'")};
    private static final Label K815 = Label.of("'inTimeRange'");
    private static final String[] K816 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14764:14780:body/2/ruleRef/capture/0"};
    private static final String[] K817 = new String[]{"startHour"};
    private static final String[] K818 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14796:14812:body/4/ruleRef/capture/0"};
    private static final String[] K819 = new String[]{"endHour"};
    private static final Delimiters[] K820 = new Delimiters[]{K217,K217,K217,K217,K217,K217,K217,K217,K217,K217};
    private static final Trivia K821 = new Trivia(K217,null,K820,null,null);
    private static final Call[] K822 = new Call[]{TinyExpressionP4Parser::e641,TinyExpressionP4Parser::e642,TinyExpressionP4Parser::e643,TinyExpressionP4Parser::e644,TinyExpressionP4Parser::e645,TinyExpressionP4Parser::e646,TinyExpressionP4Parser::e647,TinyExpressionP4Parser::e648,TinyExpressionP4Parser::e649,TinyExpressionP4Parser::e650};
    private static final Label[] K823 = new Label[]{Label.of("'inDayTimeRange'")};
    private static final Label K824 = Label.of("'inDayTimeRange'");
    private static final String[] K825 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14957:14966:body/2/ruleRef/capture/0"};
    private static final String[] K826 = new String[]{"startDay"};
    private static final String[] K827 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:14981:14997:body/4/ruleRef/capture/0"};
    private static final String[] K828 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15013:15022:body/6/ruleRef/capture/0"};
    private static final String[] K829 = new String[]{"endDay"};
    private static final String[] K830 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:15035:15051:body/8/ruleRef/capture/0"};
    private static final Call[] K831 = new Call[]{TinyExpressionP4Parser::e652,TinyExpressionP4Parser::e653,TinyExpressionP4Parser::e654,TinyExpressionP4Parser::e655,TinyExpressionP4Parser::e656,TinyExpressionP4Parser::e657,TinyExpressionP4Parser::e658};
    private static final Label[] K832 = new Label[]{Label.of("'MONDAY'")};
    private static final Label K833 = Label.of("'MONDAY'");
    private static final Label[] K834 = new Label[]{Label.of("'TUESDAY'")};
    private static final Label K835 = Label.of("'TUESDAY'");
    private static final Label[] K836 = new Label[]{Label.of("'WEDNESDAY'")};
    private static final Label K837 = Label.of("'WEDNESDAY'");
    private static final Label[] K838 = new Label[]{Label.of("'THURSDAY'")};
    private static final Label K839 = Label.of("'THURSDAY'");
    private static final Label[] K840 = new Label[]{Label.of("'FRIDAY'")};
    private static final Label K841 = Label.of("'FRIDAY'");
    private static final Label[] K842 = new Label[]{Label.of("'SATURDAY'")};
    private static final Label K843 = Label.of("'SATURDAY'");
    private static final Label[] K844 = new Label[]{Label.of("'SUNDAY'")};
    private static final Label K845 = Label.of("'SUNDAY'");
    private static final Label[] K846 = new Label[]{null,Label.of("'external'"),Label.of("'string'"),Label.of("':'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K847 = new int[]{33};
    private static final Guard K848 = new Guard(K587,K846,K217,K847,1);
    private static final Label[] K849 = new Label[]{null,Label.of("'('"),Label.of("'$'"),Label.of("')'")};
    private static final Guard K850 = new Guard(K666,K849,K217,K234,0);
    private static final int[] K851 = new int[]{90};
    private static final Guard K852 = new Guard(K666,K704,K217,K851,1);
    private static final Label[] K853 = new Label[]{null,Label.of("'$'"),Label.of("'.toUpperCase'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K854 = new int[]{66,121};
    private static final Guard K855 = new Guard(K684,K853,K217,K854,2);
    private static final Label[] K856 = new Label[]{null,Label.of("'$'"),Label.of("'.toLowerCase'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K857 = new int[]{67,121};
    private static final Guard K858 = new Guard(K684,K856,K217,K857,2);
    private static final Label[] K859 = new Label[]{null,Label.of("'$'"),Label.of("'.trim'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K860 = new int[]{68,121};
    private static final Guard K861 = new Guard(K684,K859,K217,K860,2);
    private static final Guard[] K862 = new Guard[]{K848,K850,K852,K803,K800,K806,K855,K858,K861,null,K699,K703};
    private static final Call[] K863 = new Call[]{TinyExpressionP4Parser::e660,TinyExpressionP4Parser::e661,TinyExpressionP4Parser::e665,TinyExpressionP4Parser::e666,TinyExpressionP4Parser::e667,TinyExpressionP4Parser::e668,TinyExpressionP4Parser::e669,TinyExpressionP4Parser::e670,TinyExpressionP4Parser::e671,TinyExpressionP4Parser::e672,TinyExpressionP4Parser::e673,TinyExpressionP4Parser::e674};
    private static final Label[] K864 = new Label[]{Label.of("'('"),Label.of("'$'"),Label.of("')'")};
    private static final Call[] K865 = new Call[]{TinyExpressionP4Parser::e662,TinyExpressionP4Parser::e663,TinyExpressionP4Parser::e664};
    private static final Call[] K866 = new Call[]{TinyExpressionP4Parser::e676};
    private static final Call[] K867 = new Call[]{TinyExpressionP4Parser::e678};
    private static final Call[] K868 = new Call[]{TinyExpressionP4Parser::e680};
    private static final Call[] K869 = new Call[]{TinyExpressionP4Parser::e682,TinyExpressionP4Parser::e691,TinyExpressionP4Parser::e698,TinyExpressionP4Parser::e706,TinyExpressionP4Parser::e712,TinyExpressionP4Parser::e720,TinyExpressionP4Parser::e726,TinyExpressionP4Parser::e733};
    private static final Call[] K870 = new Call[]{TinyExpressionP4Parser::e683,TinyExpressionP4Parser::e684,TinyExpressionP4Parser::e685,TinyExpressionP4Parser::e686,TinyExpressionP4Parser::e687,TinyExpressionP4Parser::e688,TinyExpressionP4Parser::e689,TinyExpressionP4Parser::e690};
    private static final String[] K871 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16309:16326:body/0/0/ruleRef/capture/0"};
    private static final Label[] K872 = new Label[]{Label.of("'['")};
    private static final Label K873 = Label.of("'['");
    private static final String[] K874 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16338:16353:body/0/2/ruleRef/capture/0"};
    private static final String[] K875 = new String[]{"start"};
    private static final String[] K876 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16365:16378:body/0/4/ruleRef/capture/0"};
    private static final String[] K877 = new String[]{"end"};
    private static final String[] K878 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16388:16402:body/0/6/ruleRef/capture/0"};
    private static final String[] K879 = new String[]{"step"};
    private static final Label[] K880 = new Label[]{Label.of("']'")};
    private static final Label K881 = Label.of("']'");
    private static final Call[] K882 = new Call[]{TinyExpressionP4Parser::e692,TinyExpressionP4Parser::e693,TinyExpressionP4Parser::e694,TinyExpressionP4Parser::e695,TinyExpressionP4Parser::e696,TinyExpressionP4Parser::e697};
    private static final String[] K883 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16419:16436:body/1/0/ruleRef/capture/0"};
    private static final String[] K884 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16448:16463:body/1/2/ruleRef/capture/0"};
    private static final String[] K885 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16475:16488:body/1/4/ruleRef/capture/0"};
    private static final Call[] K886 = new Call[]{TinyExpressionP4Parser::e699,TinyExpressionP4Parser::e700,TinyExpressionP4Parser::e701,TinyExpressionP4Parser::e702,TinyExpressionP4Parser::e703,TinyExpressionP4Parser::e704,TinyExpressionP4Parser::e705};
    private static final String[] K887 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16504:16521:body/2/0/ruleRef/capture/0"};
    private static final String[] K888 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16533:16548:body/2/2/ruleRef/capture/0"};
    private static final String[] K889 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16564:16578:body/2/5/ruleRef/capture/0"};
    private static final Call[] K890 = new Call[]{TinyExpressionP4Parser::e707,TinyExpressionP4Parser::e708,TinyExpressionP4Parser::e709,TinyExpressionP4Parser::e710,TinyExpressionP4Parser::e711};
    private static final String[] K891 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16595:16612:body/3/0/ruleRef/capture/0"};
    private static final String[] K892 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16624:16639:body/3/2/ruleRef/capture/0"};
    private static final Call[] K893 = new Call[]{TinyExpressionP4Parser::e713,TinyExpressionP4Parser::e714,TinyExpressionP4Parser::e715,TinyExpressionP4Parser::e716,TinyExpressionP4Parser::e717,TinyExpressionP4Parser::e718,TinyExpressionP4Parser::e719};
    private static final String[] K894 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16661:16678:body/4/0/ruleRef/capture/0"};
    private static final String[] K895 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16694:16707:body/4/3/ruleRef/capture/0"};
    private static final String[] K896 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16717:16731:body/4/5/ruleRef/capture/0"};
    private static final Call[] K897 = new Call[]{TinyExpressionP4Parser::e721,TinyExpressionP4Parser::e722,TinyExpressionP4Parser::e723,TinyExpressionP4Parser::e724,TinyExpressionP4Parser::e725};
    private static final String[] K898 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16748:16765:body/5/0/ruleRef/capture/0"};
    private static final String[] K899 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16781:16794:body/5/3/ruleRef/capture/0"};
    private static final Call[] K900 = new Call[]{TinyExpressionP4Parser::e727,TinyExpressionP4Parser::e728,TinyExpressionP4Parser::e729,TinyExpressionP4Parser::e730,TinyExpressionP4Parser::e731,TinyExpressionP4Parser::e732};
    private static final String[] K901 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16810:16827:body/6/0/ruleRef/capture/0"};
    private static final String[] K902 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16847:16861:body/6/4/ruleRef/capture/0"};
    private static final Call[] K903 = new Call[]{TinyExpressionP4Parser::e734,TinyExpressionP4Parser::e735,TinyExpressionP4Parser::e736,TinyExpressionP4Parser::e737};
    private static final String[] K904 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:16878:16895:body/7/0/ruleRef/capture/0"};
    private static final Call[] K905 = new Call[]{TinyExpressionP4Parser::e739,TinyExpressionP4Parser::e748,TinyExpressionP4Parser::e755,TinyExpressionP4Parser::e763,TinyExpressionP4Parser::e769,TinyExpressionP4Parser::e777,TinyExpressionP4Parser::e783,TinyExpressionP4Parser::e790};
    private static final Call[] K906 = new Call[]{TinyExpressionP4Parser::e740,TinyExpressionP4Parser::e741,TinyExpressionP4Parser::e742,TinyExpressionP4Parser::e743,TinyExpressionP4Parser::e744,TinyExpressionP4Parser::e745,TinyExpressionP4Parser::e746,TinyExpressionP4Parser::e747};
    private static final String[] K907 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17008:17027:body/0/0/ruleRef/capture/0"};
    private static final String[] K908 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17039:17054:body/0/2/ruleRef/capture/0"};
    private static final String[] K909 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17066:17079:body/0/4/ruleRef/capture/0"};
    private static final String[] K910 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17089:17103:body/0/6/ruleRef/capture/0"};
    private static final Call[] K911 = new Call[]{TinyExpressionP4Parser::e749,TinyExpressionP4Parser::e750,TinyExpressionP4Parser::e751,TinyExpressionP4Parser::e752,TinyExpressionP4Parser::e753,TinyExpressionP4Parser::e754};
    private static final String[] K912 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17120:17139:body/1/0/ruleRef/capture/0"};
    private static final String[] K913 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17151:17166:body/1/2/ruleRef/capture/0"};
    private static final String[] K914 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17178:17191:body/1/4/ruleRef/capture/0"};
    private static final Call[] K915 = new Call[]{TinyExpressionP4Parser::e756,TinyExpressionP4Parser::e757,TinyExpressionP4Parser::e758,TinyExpressionP4Parser::e759,TinyExpressionP4Parser::e760,TinyExpressionP4Parser::e761,TinyExpressionP4Parser::e762};
    private static final String[] K916 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17207:17226:body/2/0/ruleRef/capture/0"};
    private static final String[] K917 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17238:17253:body/2/2/ruleRef/capture/0"};
    private static final String[] K918 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17269:17283:body/2/5/ruleRef/capture/0"};
    private static final Call[] K919 = new Call[]{TinyExpressionP4Parser::e764,TinyExpressionP4Parser::e765,TinyExpressionP4Parser::e766,TinyExpressionP4Parser::e767,TinyExpressionP4Parser::e768};
    private static final String[] K920 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17300:17319:body/3/0/ruleRef/capture/0"};
    private static final String[] K921 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17331:17346:body/3/2/ruleRef/capture/0"};
    private static final Call[] K922 = new Call[]{TinyExpressionP4Parser::e770,TinyExpressionP4Parser::e771,TinyExpressionP4Parser::e772,TinyExpressionP4Parser::e773,TinyExpressionP4Parser::e774,TinyExpressionP4Parser::e775,TinyExpressionP4Parser::e776};
    private static final String[] K923 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17368:17387:body/4/0/ruleRef/capture/0"};
    private static final String[] K924 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17403:17416:body/4/3/ruleRef/capture/0"};
    private static final String[] K925 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17426:17440:body/4/5/ruleRef/capture/0"};
    private static final Call[] K926 = new Call[]{TinyExpressionP4Parser::e778,TinyExpressionP4Parser::e779,TinyExpressionP4Parser::e780,TinyExpressionP4Parser::e781,TinyExpressionP4Parser::e782};
    private static final String[] K927 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17457:17476:body/5/0/ruleRef/capture/0"};
    private static final String[] K928 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17492:17505:body/5/3/ruleRef/capture/0"};
    private static final Call[] K929 = new Call[]{TinyExpressionP4Parser::e784,TinyExpressionP4Parser::e785,TinyExpressionP4Parser::e786,TinyExpressionP4Parser::e787,TinyExpressionP4Parser::e788,TinyExpressionP4Parser::e789};
    private static final String[] K930 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17521:17540:body/6/0/ruleRef/capture/0"};
    private static final String[] K931 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17560:17574:body/6/4/ruleRef/capture/0"};
    private static final Call[] K932 = new Call[]{TinyExpressionP4Parser::e791,TinyExpressionP4Parser::e792,TinyExpressionP4Parser::e793,TinyExpressionP4Parser::e794};
    private static final String[] K933 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17591:17610:body/7/0/ruleRef/capture/0"};
    private static final Call[] K934 = new Call[]{TinyExpressionP4Parser::e796,TinyExpressionP4Parser::e797};
    private static final Call[] K935 = new Call[]{TinyExpressionP4Parser::e799,TinyExpressionP4Parser::e800};
    private static final String[] K936 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17899:17909:body/0/ruleRef/capture/0"};
    private static final Call[] K937 = new Call[]{TinyExpressionP4Parser::e802,TinyExpressionP4Parser::e803};
    private static final String[] K938 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17918:17921:body/1/0/0/literal/capture/0"};
    private static final String[] K939 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:17926:17936:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K940 = new Call[]{TinyExpressionP4Parser::e805,TinyExpressionP4Parser::e806,TinyExpressionP4Parser::e807};
    private static final int[] K941 = new int[]{113};
    private static final Guard K942 = new Guard(K554,K670,K217,K941,1);
    private static final Label[] K943 = new Label[]{null,Label.of("'('"),Label.of("'string'"),Label.of("'String'"),Label.of("')'"),Label.of("'$'")};
    private static final int[] K944 = new int[]{92};
    private static final Guard K945 = new Guard(K666,K943,K217,K944,1);
    private static final Label[] K946 = new Label[]{null,Label.of("'$'"),Label.of("'as'"),Label.of("'string'"),Label.of("'String'")};
    private static final int[] K947 = new int[]{93};
    private static final Guard K948 = new Guard(K684,K946,K217,K947,1);
    private static final Guard[] K949 = new Guard[]{K942,K676,K945,K948,null,K852,K848,K803,K800,K806,K855,K858,K861,null,K699,K703};
    private static final Call[] K950 = new Call[]{TinyExpressionP4Parser::e809,TinyExpressionP4Parser::e810,TinyExpressionP4Parser::e811,TinyExpressionP4Parser::e812,TinyExpressionP4Parser::e813,TinyExpressionP4Parser::e814,TinyExpressionP4Parser::e815,TinyExpressionP4Parser::e816,TinyExpressionP4Parser::e817,TinyExpressionP4Parser::e818,TinyExpressionP4Parser::e819,TinyExpressionP4Parser::e820,TinyExpressionP4Parser::e821,TinyExpressionP4Parser::e822,TinyExpressionP4Parser::e823,TinyExpressionP4Parser::e824};
    private static final Call[] K951 = new Call[]{TinyExpressionP4Parser::e826,TinyExpressionP4Parser::e827,TinyExpressionP4Parser::e831,TinyExpressionP4Parser::e832,TinyExpressionP4Parser::e833};
    private static final Call[] K952 = new Call[]{TinyExpressionP4Parser::e829,TinyExpressionP4Parser::e830};
    private static final String[] K953 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18697:18707:body/4/tokenRef/capture/0"};
    private static final Call[] K954 = new Call[]{TinyExpressionP4Parser::e835,TinyExpressionP4Parser::e836,TinyExpressionP4Parser::e837,TinyExpressionP4Parser::e838};
    private static final String[] K955 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:18801:18811:body/1/tokenRef/capture/0"};
    private static final Call[] K956 = new Call[]{TinyExpressionP4Parser::e840,TinyExpressionP4Parser::e841};
    private static final Call[] K957 = new Call[]{TinyExpressionP4Parser::e843,TinyExpressionP4Parser::e844};
    private static final String[] K958 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19077:19097:body/0/ruleRef/capture/0"};
    private static final Label[] K959 = new Label[]{Label.of("'|'")};
    private static final Call[] K960 = new Call[]{TinyExpressionP4Parser::e846,TinyExpressionP4Parser::e847};
    private static final String[] K961 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19106:19109:body/1/0/0/literal/capture/0"};
    private static final Label K962 = Label.of("'|'");
    private static final String[] K963 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19114:19134:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K964 = new Call[]{TinyExpressionP4Parser::e849,TinyExpressionP4Parser::e850};
    private static final String[] K965 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19280:19300:body/0/ruleRef/capture/0"};
    private static final Label[] K966 = new Label[]{Label.of("'&'")};
    private static final Call[] K967 = new Call[]{TinyExpressionP4Parser::e852,TinyExpressionP4Parser::e853};
    private static final String[] K968 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19309:19312:body/1/0/0/literal/capture/0"};
    private static final Label K969 = Label.of("'&'");
    private static final String[] K970 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19317:19337:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K971 = new Call[]{TinyExpressionP4Parser::e855,TinyExpressionP4Parser::e856};
    private static final String[] K972 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19483:19496:body/0/ruleRef/capture/0"};
    private static final Label[] K973 = new Label[]{Label.of("'^'")};
    private static final Call[] K974 = new Call[]{TinyExpressionP4Parser::e858,TinyExpressionP4Parser::e859};
    private static final String[] K975 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19505:19508:body/1/0/0/literal/capture/0"};
    private static final Label K976 = Label.of("'^'");
    private static final String[] K977 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19513:19526:body/1/0/1/ruleRef/capture/0"};
    private static final Call[] K978 = new Call[]{TinyExpressionP4Parser::e861,TinyExpressionP4Parser::e862,TinyExpressionP4Parser::e863,TinyExpressionP4Parser::e864};
    private static final String[] K979 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:19683:19700:body/2/ruleRef/capture/0"};
    private static final int[] K980 = new int[]{9,13,32,32,47,47,110,110};
    private static final Label[] K981 = new Label[]{null,Label.of("'not'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K982 = new int[]{97};
    private static final Guard K983 = new Guard(K980,K981,K217,K982,1);
    private static final int[] K984 = new int[]{117};
    private static final Guard K985 = new Guard(K554,K670,K217,K984,1);
    private static final Label[] K986 = new Label[]{null,Label.of("'external'"),Label.of("'boolean'"),Label.of("':'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K987 = new int[]{31};
    private static final Guard K988 = new Guard(K587,K986,K217,K987,1);
    private static final int[] K989 = new int[]{9,13,32,32,36,36,47,47,116,116};
    private static final Label[] K990 = new Label[]{null,Label.of("'toLowerCase'"),Label.of("'('"),Label.of("')'"),Label.of("'toUpperCase'"),Label.of("'trim'"),Label.of("'$'"),Label.of("__CaptureSite"),Label.of("'.startsWith'")};
    private static final int[] K991 = new int[]{74,77,62};
    private static final Guard K992 = new Guard(K989,K990,K217,K991,3);
    private static final Label[] K993 = new Label[]{null,Label.of("'toLowerCase'"),Label.of("'('"),Label.of("')'"),Label.of("'toUpperCase'"),Label.of("'trim'"),Label.of("'$'"),Label.of("__CaptureSite"),Label.of("'.endsWith'")};
    private static final int[] K994 = new int[]{75,77,62};
    private static final Guard K995 = new Guard(K989,K993,K217,K994,3);
    private static final Label[] K996 = new Label[]{null,Label.of("'toLowerCase'"),Label.of("'('"),Label.of("')'"),Label.of("'toUpperCase'"),Label.of("'trim'"),Label.of("'$'"),Label.of("__CaptureSite"),Label.of("'.contains'")};
    private static final int[] K997 = new int[]{76,77,62};
    private static final Guard K998 = new Guard(K989,K996,K217,K997,3);
    private static final Label[] K999 = new Label[]{null,Label.of("'startsWith'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K1000 = new int[]{70};
    private static final Guard K1001 = new Guard(K427,K999,K217,K1000,1);
    private static final Label[] K1002 = new Label[]{null,Label.of("'endsWith'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K1003 = new int[]{71};
    private static final Guard K1004 = new Guard(K587,K1002,K217,K1003,1);
    private static final Label[] K1005 = new Label[]{null,Label.of("'contains'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K1006 = new int[]{72};
    private static final Guard K1007 = new Guard(K485,K1005,K217,K1006,1);
    private static final Label[] K1008 = new Label[]{null,Label.of("'isPresent'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K1009 = new int[]{78};
    private static final Guard K1010 = new Guard(K673,K1008,K217,K1009,1);
    private static final Label[] K1011 = new Label[]{null,Label.of("'inTimeRange'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K1012 = new int[]{79};
    private static final Guard K1013 = new Guard(K673,K1011,K217,K1012,1);
    private static final Label[] K1014 = new Label[]{null,Label.of("'inDayTimeRange'"),Label.of("'('"),Label.of("','"),Label.of("')'")};
    private static final int[] K1015 = new int[]{80};
    private static final Guard K1016 = new Guard(K673,K1014,K217,K1015,1);
    private static final Guard[] K1017 = new Guard[]{K983,K676,K985,K988,null,K992,K995,K998,K1001,K1004,K1007,K1010,K1013,K1016,null,null,K699,K703,K705};
    private static final Call[] K1018 = new Call[]{TinyExpressionP4Parser::e866,TinyExpressionP4Parser::e867,TinyExpressionP4Parser::e868,TinyExpressionP4Parser::e869,TinyExpressionP4Parser::e870,TinyExpressionP4Parser::e871,TinyExpressionP4Parser::e872,TinyExpressionP4Parser::e873,TinyExpressionP4Parser::e874,TinyExpressionP4Parser::e875,TinyExpressionP4Parser::e876,TinyExpressionP4Parser::e877,TinyExpressionP4Parser::e878,TinyExpressionP4Parser::e879,TinyExpressionP4Parser::e880,TinyExpressionP4Parser::e881,TinyExpressionP4Parser::e882,TinyExpressionP4Parser::e883,TinyExpressionP4Parser::e884};
    private static final Label[] K1019 = new Label[]{Label.of("'true'")};
    private static final Label K1020 = Label.of("'true'");
    private static final Label[] K1021 = new Label[]{Label.of("'false'")};
    private static final Label K1022 = Label.of("'false'");
    private static final Call[] K1023 = new Call[]{TinyExpressionP4Parser::e885,TinyExpressionP4Parser::e886,TinyExpressionP4Parser::e887};
    private static final Call[] K1024 = new Call[]{TinyExpressionP4Parser::e889,TinyExpressionP4Parser::e890,TinyExpressionP4Parser::e891};
    private static final String[] K1025 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20271:20288:body/0/ruleRef/capture/0"};
    private static final Label[] K1026 = new Label[]{Label.of("'true'"),Label.of("'false'")};
    private static final String[] K1027 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20295:20305:body/1/ruleRef/capture/0"};
    private static final String[] K1028 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20310:20327:body/2/ruleRef/capture/0"};
    private static final Call[] K1029 = new Call[]{TinyExpressionP4Parser::e893,TinyExpressionP4Parser::e894,TinyExpressionP4Parser::e895,TinyExpressionP4Parser::e896};
    private static final String[] K1030 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20410:20435:body/0/ruleRef/capture/0"};
    private static final String[] K1031 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20449:20469:body/1/ruleRef/capture/0"};
    private static final String[] K1032 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20483:20509:body/2/ruleRef/capture/0"};
    private static final String[] K1033 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20523:20540:body/3/ruleRef/capture/0"};
    private static final Call[] K1034 = new Call[]{TinyExpressionP4Parser::e898,TinyExpressionP4Parser::e899,TinyExpressionP4Parser::e900};
    private static final String[] K1035 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20804:20820:body/0/ruleRef/capture/0"};
    private static final String[] K1036 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20827:20837:body/1/ruleRef/capture/0"};
    private static final String[] K1037 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20842:20858:body/2/ruleRef/capture/0"};
    private static final Call[] K1038 = new Call[]{TinyExpressionP4Parser::e902,TinyExpressionP4Parser::e903};
    private static final Label[] K1039 = new Label[]{Label.of("'=='")};
    private static final Label K1040 = Label.of("'=='");
    private static final Label[] K1041 = new Label[]{Label.of("'!='")};
    private static final Label K1042 = Label.of("'!='");
    private static final Call[] K1043 = new Call[]{TinyExpressionP4Parser::e905,TinyExpressionP4Parser::e906,TinyExpressionP4Parser::e907};
    private static final String[] K1044 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:20980:20996:body/0/ruleRef/capture/0"};
    private static final String[] K1045 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21003:21012:body/1/ruleRef/capture/0"};
    private static final String[] K1046 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21017:21033:body/2/ruleRef/capture/0"};
    private static final Call[] K1047 = new Call[]{TinyExpressionP4Parser::e909,TinyExpressionP4Parser::e910,TinyExpressionP4Parser::e911,TinyExpressionP4Parser::e912,TinyExpressionP4Parser::e913,TinyExpressionP4Parser::e914};
    private static final Label[] K1048 = new Label[]{Label.of("'<='")};
    private static final Label K1049 = Label.of("'<='");
    private static final Label[] K1050 = new Label[]{Label.of("'>='")};
    private static final Label K1051 = Label.of("'>='");
    private static final Label[] K1052 = new Label[]{Label.of("'<'")};
    private static final Label K1053 = Label.of("'<'");
    private static final Label[] K1054 = new Label[]{Label.of("'>'")};
    private static final Label K1055 = Label.of("'>'");
    private static final Label[] K1056 = new Label[]{null,Label.of("'external'"),Label.of("'object'"),Label.of("':'"),Label.of("'('"),Label.of("')'")};
    private static final int[] K1057 = new int[]{34};
    private static final Guard K1058 = new Guard(K587,K1056,K217,K1057,1);
    private static final Guard[] K1059 = new Guard[]{null,null,null,K1058,K699,K703};
    private static final Call[] K1060 = new Call[]{TinyExpressionP4Parser::e916,TinyExpressionP4Parser::e917,TinyExpressionP4Parser::e918,TinyExpressionP4Parser::e919,TinyExpressionP4Parser::e920,TinyExpressionP4Parser::e921};
    private static final String[] K1061 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21250:21266:body/0/ruleRef/capture/0"};
    private static final String[] K1062 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21280:21296:body/1/ruleRef/capture/0"};
    private static final String[] K1063 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21310:21327:body/2/ruleRef/capture/0"};
    private static final String[] K1064 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21341:21365:body/3/ruleRef/capture/0"};
    private static final String[] K1065 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21379:21390:body/4/ruleRef/capture/0"};
    private static final String[] K1066 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21404:21420:body/5/ruleRef/capture/0"};
    private static final Delimiters[] K1067 = new Delimiters[]{K217,K217,K217,K217,K217,K217,K217,K217,K217,K217,K217};
    private static final Trivia K1068 = new Trivia(K217,null,K1067,null,null);
    private static final Call[] K1069 = new Call[]{TinyExpressionP4Parser::e923,TinyExpressionP4Parser::e924,TinyExpressionP4Parser::e925,TinyExpressionP4Parser::e926,TinyExpressionP4Parser::e927,TinyExpressionP4Parser::e928,TinyExpressionP4Parser::e929,TinyExpressionP4Parser::e930,TinyExpressionP4Parser::e931,TinyExpressionP4Parser::e932,TinyExpressionP4Parser::e933};
    private static final String[] K1070 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21604:21621:body/2/ruleRef/capture/0"};
    private static final String[] K1071 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21645:21661:body/5/ruleRef/capture/0"};
    private static final Label[] K1072 = new Label[]{Label.of("'else'")};
    private static final Label K1073 = Label.of("'else'");
    private static final String[] K1074 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21695:21711:body/9/ruleRef/capture/0"};
    private static final Guard[] K1075 = new Guard[]{null,null,null,null,null,null,null,K703};
    private static final Call[] K1076 = new Call[]{TinyExpressionP4Parser::e935,TinyExpressionP4Parser::e936,TinyExpressionP4Parser::e937,TinyExpressionP4Parser::e938,TinyExpressionP4Parser::e939,TinyExpressionP4Parser::e940,TinyExpressionP4Parser::e941,TinyExpressionP4Parser::e942};
    private static final String[] K1077 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21807:21827:body/0/ruleRef/capture/0"};
    private static final String[] K1078 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21841:21867:body/1/ruleRef/capture/0"};
    private static final String[] K1079 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21881:21906:body/2/ruleRef/capture/0"};
    private static final String[] K1080 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21920:21936:body/3/ruleRef/capture/0"};
    private static final String[] K1081 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21950:21967:body/4/ruleRef/capture/0"};
    private static final String[] K1082 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:21981:21997:body/5/ruleRef/capture/0"};
    private static final String[] K1083 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22011:22027:body/6/ruleRef/capture/0"};
    private static final String[] K1084 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22041:22057:body/7/ruleRef/capture/0"};
    private static final Call[] K1085 = new Call[]{TinyExpressionP4Parser::e944,TinyExpressionP4Parser::e945,TinyExpressionP4Parser::e946,TinyExpressionP4Parser::e947,TinyExpressionP4Parser::e948,TinyExpressionP4Parser::e949,TinyExpressionP4Parser::e950};
    private static final String[] K1086 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22236:22253:body/1/ruleRef/capture/0"};
    private static final String[] K1087 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22269:22285:body/3/ruleRef/capture/0"};
    private static final String[] K1088 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22300:22316:body/5/ruleRef/capture/0"};
    private static final Call[] K1089 = new Call[]{TinyExpressionP4Parser::e952,TinyExpressionP4Parser::e953,TinyExpressionP4Parser::e954,TinyExpressionP4Parser::e955,TinyExpressionP4Parser::e959,TinyExpressionP4Parser::e960,TinyExpressionP4Parser::e961};
    private static final Label[] K1090 = new Label[]{Label.of("'match'")};
    private static final Label K1091 = Label.of("'match'");
    private static final String[] K1092 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22538:22548:body/2/ruleRef/capture/0"};
    private static final String[] K1093 = new String[]{"firstCase"};
    private static final Call[] K1094 = new Call[]{TinyExpressionP4Parser::e957,TinyExpressionP4Parser::e958};
    private static final String[] K1095 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22566:22576:body/3/0/1/ruleRef/capture/0"};
    private static final String[] K1096 = new String[]{"moreCases"};
    private static final String[] K1097 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22600:22617:body/5/ruleRef/capture/0"};
    private static final String[] K1098 = new String[]{"defaultCase"};
    private static final Call[] K1099 = new Call[]{TinyExpressionP4Parser::e963,TinyExpressionP4Parser::e964,TinyExpressionP4Parser::e965};
    private static final String[] K1100 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22713:22730:body/0/ruleRef/capture/0"};
    private static final Label K1101 = Label.of("'->'");
    private static final String[] K1102 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22747:22762:body/2/ruleRef/capture/0"};
    private static final Call[] K1103 = new Call[]{TinyExpressionP4Parser::e967,TinyExpressionP4Parser::e968,TinyExpressionP4Parser::e969};
    private static final Label[] K1104 = new Label[]{Label.of("'default'")};
    private static final Label K1105 = Label.of("'default'");
    private static final String[] K1106 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22861:22876:body/2/ruleRef/capture/0"};
    private static final Call[] K1107 = new Call[]{TinyExpressionP4Parser::e971};
    private static final String[] K1108 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:22956:22972:body/0/ruleRef/capture/0"};
    private static final Call[] K1109 = new Call[]{TinyExpressionP4Parser::e973,TinyExpressionP4Parser::e974,TinyExpressionP4Parser::e975,TinyExpressionP4Parser::e976,TinyExpressionP4Parser::e980,TinyExpressionP4Parser::e981,TinyExpressionP4Parser::e982};
    private static final String[] K1110 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23105:23115:body/2/ruleRef/capture/0"};
    private static final Call[] K1111 = new Call[]{TinyExpressionP4Parser::e978,TinyExpressionP4Parser::e979};
    private static final String[] K1112 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23133:23143:body/3/0/1/ruleRef/capture/0"};
    private static final String[] K1113 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23167:23184:body/5/ruleRef/capture/0"};
    private static final Call[] K1114 = new Call[]{TinyExpressionP4Parser::e984,TinyExpressionP4Parser::e985,TinyExpressionP4Parser::e986};
    private static final String[] K1115 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23280:23297:body/0/ruleRef/capture/0"};
    private static final String[] K1116 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23314:23329:body/2/ruleRef/capture/0"};
    private static final Call[] K1117 = new Call[]{TinyExpressionP4Parser::e988,TinyExpressionP4Parser::e989,TinyExpressionP4Parser::e990};
    private static final String[] K1118 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23428:23443:body/2/ruleRef/capture/0"};
    private static final Call[] K1119 = new Call[]{TinyExpressionP4Parser::e992};
    private static final String[] K1120 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23523:23539:body/0/ruleRef/capture/0"};
    private static final Call[] K1121 = new Call[]{TinyExpressionP4Parser::e994,TinyExpressionP4Parser::e995,TinyExpressionP4Parser::e996,TinyExpressionP4Parser::e997,TinyExpressionP4Parser::e1001,TinyExpressionP4Parser::e1002,TinyExpressionP4Parser::e1003};
    private static final String[] K1122 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23674:23685:body/2/ruleRef/capture/0"};
    private static final Call[] K1123 = new Call[]{TinyExpressionP4Parser::e999,TinyExpressionP4Parser::e1000};
    private static final String[] K1124 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23703:23714:body/3/0/1/ruleRef/capture/0"};
    private static final String[] K1125 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23738:23756:body/5/ruleRef/capture/0"};
    private static final Call[] K1126 = new Call[]{TinyExpressionP4Parser::e1005,TinyExpressionP4Parser::e1006,TinyExpressionP4Parser::e1007};
    private static final String[] K1127 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23854:23871:body/0/ruleRef/capture/0"};
    private static final String[] K1128 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:23888:23904:body/2/ruleRef/capture/0"};
    private static final Call[] K1129 = new Call[]{TinyExpressionP4Parser::e1009,TinyExpressionP4Parser::e1010,TinyExpressionP4Parser::e1011};
    private static final String[] K1130 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24005:24021:body/2/ruleRef/capture/0"};
    private static final Call[] K1131 = new Call[]{TinyExpressionP4Parser::e1013};
    private static final String[] K1132 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24103:24120:body/0/ruleRef/capture/0"};
    private static final Call[] K1133 = new Call[]{TinyExpressionP4Parser::e1015,TinyExpressionP4Parser::e1016,TinyExpressionP4Parser::e1017};
    private static final String[] K1134 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24337:24347:body/1/tokenRef/capture/0"};
    private static final Call[] K1135 = new Call[]{TinyExpressionP4Parser::e1019,TinyExpressionP4Parser::e1021};
    private static final String[] K1136 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24365:24376:body/2/0/1/ruleRef/capture/0"};
    private static final Call[] K1137 = new Call[]{TinyExpressionP4Parser::e1023,TinyExpressionP4Parser::e1024,TinyExpressionP4Parser::e1025,TinyExpressionP4Parser::e1026,TinyExpressionP4Parser::e1027,TinyExpressionP4Parser::e1028,TinyExpressionP4Parser::e1029,TinyExpressionP4Parser::e1030,TinyExpressionP4Parser::e1031,TinyExpressionP4Parser::e1032};
    private static final Guard[] K1138 = new Guard[]{null,null,null,null,K703,K705};
    private static final Call[] K1139 = new Call[]{TinyExpressionP4Parser::e1034,TinyExpressionP4Parser::e1035,TinyExpressionP4Parser::e1036,TinyExpressionP4Parser::e1037,TinyExpressionP4Parser::e1038,TinyExpressionP4Parser::e1039};
    private static final String[] K1140 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24836:24852:body/0/ruleRef/capture/0"};
    private static final String[] K1141 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24866:24883:body/1/ruleRef/capture/0"};
    private static final String[] K1142 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24897:24913:body/2/ruleRef/capture/0"};
    private static final String[] K1143 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24927:24943:body/3/ruleRef/capture/0"};
    private static final String[] K1144 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24957:24973:body/4/ruleRef/capture/0"};
    private static final Call[] K1145 = new Call[]{TinyExpressionP4Parser::e1040,TinyExpressionP4Parser::e1041,TinyExpressionP4Parser::e1042};
    private static final String[] K1146 = new String[]{"expr:tools/tinyexpression-p4-lsp-vscode/grammar/tinyexpression-p4.ubnf:24991:25001:body/5/1/ruleRef/capture/0"};
}
