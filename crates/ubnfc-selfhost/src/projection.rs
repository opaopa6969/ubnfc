//! 型付き生成 AST の値だけを front の借用投影に渡す。
use crate::generated::ast::Ast;
use ubnfc_front::selfhost::MappedNodes;

pub(crate) fn project<'a>(ast: &'a Ast, out: &mut MappedNodes<'a>) {
    match ast {
        Ast::Text(_) | Ast::Null => {}
        Ast::g_UBNFAST_2e_UBNFFile(node) => {
            for child in &node.g_grammars {
                project(child, out);
            }
            out.node("UBNFFile", node.span);
        }
        Ast::g_UBNFAST_2e_ImportDecl(node) => {
            out.node("ImportDecl", node.span)
                .field("alias", [node.g_alias.as_str()])
                .field("path", [node.g_path.as_str()]);
        }
        Ast::g_UBNFAST_2e_GrammarDecl(node) => {
            for child in &node.g_imports {
                project(child, out);
            }
            for child in &node.g_settings {
                project(child, out);
            }
            for child in &node.g_tokens {
                project(child, out);
            }
            for child in &node.g_rules {
                project(child, out);
            }
            out.node("GrammarDecl", node.span)
                .field("name", [node.g_name.as_str()]);
        }
        Ast::g_UBNFAST_2e_GlobalSetting(node) => {
            project(&node.g_value, out);
            out.node("GlobalSetting", node.span)
                .field("key", [node.g_key.as_str()]);
        }
        Ast::g_UBNFAST_2e_StringSettingValue(node) => {
            project(&node.g_value, out);
            out.node("StringSettingValue", node.span);
        }
        Ast::g_UBNFAST_2e_BlockSettingValue(node) => {
            for child in &node.g_entries {
                project(child, out);
            }
            out.node("BlockSettingValue", node.span);
        }
        Ast::g_UBNFAST_2e_KeyValuePair(node) => {
            out.node("KeyValuePair", node.span)
                .field("key", [node.g_key.as_str()])
                .field("value", [node.g_value.as_str()]);
        }
        Ast::g_UBNFAST_2e_TokenDecl(node) => {
            project(&node.g_value, out);
            out.node("TokenDecl", node.span)
                .field("name", [node.g_name.as_str()]);
        }
        Ast::g_UBNFAST_2e_DeclarativeTokenValue(node) => {
            project(&node.g_source, out);
            out.node("DeclarativeTokenValue", node.span);
        }
        Ast::g_UBNFAST_2e_LexicalChoice(node) => {
            for child in &node.g_alternatives {
                project(child, out);
            }
            out.node("LexicalBody", node.span);
        }
        Ast::g_UBNFAST_2e_LexicalSequence(node) => {
            for child in &node.g_elements {
                project(child, out);
            }
            out.node("LexicalSequence", node.span);
        }
        Ast::g_UBNFAST_2e_LexicalElement(node) => {
            project(&node.g_atom, out);
            if let Some(child) = &node.g_suffix {
                project(child, out);
            }
            out.node("LexicalElement", node.span);
        }
        Ast::g_UBNFAST_2e_LexicalSuffix(node) => {
            out.node("LexicalSuffix", node.span)
                .field("text", [node.g_text.as_str()]);
        }
        Ast::g_UBNFAST_2e_LexicalAtom(node) => {
            project(&node.g_value, out);
            out.node("LexicalAtom", node.span);
        }
        Ast::g_UBNFAST_2e_AdapterExpression(node) => {
            out.node("AdapterExpression", node.span)
                .field("id", [node.g_id.as_str()])
                .field("version", [node.g_version.as_str()]);
        }
        Ast::g_UBNFAST_2e_UntilExpression(node) => {
            out.node("UntilExpression", node.span)
                .field("terminator", [node.g_terminator.as_str()]);
        }
        Ast::g_UBNFAST_2e_NegationExpression(node) => {
            out.node("NegationExpression", node.span)
                .field("chars", [node.g_chars.as_str()]);
        }
        Ast::g_UBNFAST_2e_LookaheadExpression(node) => {
            out.node("LookaheadExpression", node.span)
                .field("pattern", [node.g_pattern.as_str()]);
        }
        Ast::g_UBNFAST_2e_NegativeLookaheadExpression(node) => {
            out.node("NegativeLookaheadExpression", node.span)
                .field("pattern", [node.g_pattern.as_str()]);
        }
        Ast::g_UBNFAST_2e_CharRangeExpression(node) => {
            out.node("CharRangeExpression", node.span)
                .field("min", [node.g_min.as_str()])
                .field("max", [node.g_max.as_str()]);
        }
        Ast::g_UBNFAST_2e_CIExpression(node) => {
            out.node("CIExpression", node.span)
                .field("word", [node.g_word.as_str()]);
        }
        Ast::g_UBNFAST_2e_RegexExpression(node) => {
            out.node("RegexExpression", node.span)
                .field("pattern", [node.g_pattern.as_str()]);
        }
        Ast::g_UBNFAST_2e_AnyExpression(node) => {
            out.node("AnyExpression", node.span);
        }
        Ast::g_UBNFAST_2e_EofExpression(node) => {
            out.node("EofExpression", node.span);
        }
        Ast::g_UBNFAST_2e_EmptyExpression(node) => {
            out.node("EmptyExpression", node.span);
        }
        Ast::g_UBNFAST_2e_RuleDecl(node) => {
            for child in &node.g_annotations {
                project(child, out);
            }
            project(&node.g_body, out);
            out.node("RuleDecl", node.span)
                .field("name", [node.g_name.as_str()]);
        }
        Ast::g_UBNFAST_2e_RootAnnotation(node) => {
            out.node("RootAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_MappingAnnotation(node) => {
            out.node("MappingAnnotation", node.span)
                .field("className", [node.g_className.as_str()])
                .field("paramNames", node.g_paramNames.iter().map(String::as_str));
        }
        Ast::g_UBNFAST_2e_EvalAnnotation(node) => {
            for child in &node.g_entries {
                project(child, out);
            }
            out.node("EvalAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_WhitespaceAnnotation(node) => {
            out.node("WhitespaceAnnotation", node.span)
                .field("style", node.g_style.as_deref());
        }
        Ast::g_UBNFAST_2e_InterleaveAnnotation(node) => {
            out.node("InterleaveAnnotation", node.span)
                .field("profile", [node.g_profile.as_str()]);
        }
        Ast::g_UBNFAST_2e_BackrefAnnotation(node) => {
            out.node("BackrefAnnotation", node.span)
                .field("name", [node.g_name.as_str()]);
        }
        Ast::g_UBNFAST_2e_ScopeTreeAnnotation(node) => {
            out.node("ScopeTreeAnnotation", node.span)
                .field("mode", [node.g_mode.as_str()]);
        }
        Ast::g_UBNFAST_2e_DeclaresAnnotation(node) => {
            out.node("DeclaresAnnotation", node.span)
                .field("symbol", [node.g_symbol.as_str()])
                .field("description", node.g_description.as_deref());
        }
        Ast::g_UBNFAST_2e_CatalogAnnotation(node) => {
            out.node("CatalogAnnotation", node.span)
                .field("context", [node.g_context.as_str()]);
        }
        Ast::g_UBNFAST_2e_LeftAssocAnnotation(node) => {
            out.node("LeftAssocAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_RightAssocAnnotation(node) => {
            out.node("RightAssocAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_LongestChoiceAnnotation(node) => {
            out.node("LongestChoiceAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(node) => {
            out.node("PredictiveChoiceAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_PrecedenceAnnotation(node) => {
            out.node("PrecedenceAnnotation", node.span)
                .field("level", [node.g_level.as_str()]);
        }
        Ast::g_UBNFAST_2e_DocAnnotation(node) => {
            out.node("DocAnnotation", node.span)
                .field("text", [node.g_text.as_str()]);
        }
        Ast::g_UBNFAST_2e_RecoveryAnnotation(node) => {
            out.node("RecoveryAnnotation", node.span)
                .field("mode", node.g_mode.as_deref())
                .field("syncTokens", node.g_syncTokens.as_deref());
        }
        Ast::g_UBNFAST_2e_SkipAnnotation(node) => {
            out.node("SkipAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_CommonFieldAnnotation(node) => {
            out.node("CommonFieldAnnotation", node.span)
                .field("fieldNames", node.g_fieldNames.iter().map(String::as_str));
        }
        Ast::g_UBNFAST_2e_EnumAnnotation(node) => {
            out.node("EnumAnnotation", node.span);
        }
        Ast::g_UBNFAST_2e_SimpleAnnotation(node) => {
            out.node("SimpleAnnotation", node.span)
                .field("name", [node.g_name.as_str()]);
        }
        Ast::g_UBNFAST_2e_ChoiceBody(node) => {
            for child in &node.g_alternatives {
                project(child, out);
            }
            out.node("ChoiceBody", node.span);
        }
        Ast::g_UBNFAST_2e_SequenceBody(node) => {
            for child in &node.g_elements {
                project(child, out);
            }
            out.node("SequenceBody", node.span);
        }
        Ast::g_UBNFAST_2e_AnnotatedElement(node) => {
            project(&node.g_element, out);
            if let Some(child) = &node.g_typeofConstraint {
                project(child, out);
            }
            out.node("AnnotatedElement", node.span)
                .field("captureName", node.g_captureName.as_deref());
        }
        Ast::g_UBNFAST_2e_GroupElement(node) => {
            project(&node.g_body, out);
            out.node("GroupElement", node.span);
        }
        Ast::g_UBNFAST_2e_OptionalElement(node) => {
            project(&node.g_body, out);
            out.node("OptionalElement", node.span);
        }
        Ast::g_UBNFAST_2e_RepeatElement(node) => {
            project(&node.g_body, out);
            out.node("RepeatElement", node.span);
        }
        Ast::g_UBNFAST_2e_ErrorElement(node) => {
            out.node("ErrorElement", node.span)
                .field("message", [node.g_message.as_str()]);
        }
        Ast::g_UBNFAST_2e_TerminalElement(node) => {
            out.node("TerminalElement", node.span)
                .field("value", [node.g_value.as_str()]);
        }
        Ast::g_UBNFAST_2e_TypeofElement(node) => {
            out.node("TypeofElement", node.span)
                .field("captureName", [node.g_captureName.as_str()]);
        }
        Ast::g_UBNFAST_2e_DottedIdentifier(node) => {
            out.node("DottedIdentifier", node.span)
                .field("parts", node.g_parts.iter().map(String::as_str));
        }
    }
}
