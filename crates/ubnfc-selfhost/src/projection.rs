//! 型付き生成 AST の値だけを front の借用投影に渡す。
use crate::generated::ast::Ast;
use ubnfc_front::selfhost::MappedNode;

pub(crate) fn project<'a>(ast: &'a Ast, out: &mut Vec<MappedNode<'a>>) {
    match ast {
        Ast::Text(_) | Ast::Null => {}
        Ast::g_UBNFAST_2e_UBNFFile(node) => {
            for child in &node.g_grammars {
                project(child, out);
            }
            out.push(MappedNode {
                kind: "UBNFFile",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_ImportDecl(node) => {
            out.push(MappedNode {
                kind: "ImportDecl",
                span_cp: node.span,
                fields: vec![
                    ("alias", vec![node.g_alias.as_str()]),
                    ("path", vec![node.g_path.as_str()]),
                ],
            });
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
            out.push(MappedNode {
                kind: "GrammarDecl",
                span_cp: node.span,
                fields: vec![("name", vec![node.g_name.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_GlobalSetting(node) => {
            project(&node.g_value, out);
            out.push(MappedNode {
                kind: "GlobalSetting",
                span_cp: node.span,
                fields: vec![("key", vec![node.g_key.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_StringSettingValue(node) => {
            project(&node.g_value, out);
            out.push(MappedNode {
                kind: "StringSettingValue",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_BlockSettingValue(node) => {
            for child in &node.g_entries {
                project(child, out);
            }
            out.push(MappedNode {
                kind: "BlockSettingValue",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_KeyValuePair(node) => {
            out.push(MappedNode {
                kind: "KeyValuePair",
                span_cp: node.span,
                fields: vec![
                    ("key", vec![node.g_key.as_str()]),
                    ("value", vec![node.g_value.as_str()]),
                ],
            });
        }
        Ast::g_UBNFAST_2e_TokenDecl(node) => {
            project(&node.g_value, out);
            out.push(MappedNode {
                kind: "TokenDecl",
                span_cp: node.span,
                fields: vec![("name", vec![node.g_name.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_UntilExpression(node) => {
            out.push(MappedNode {
                kind: "UntilExpression",
                span_cp: node.span,
                fields: vec![("terminator", vec![node.g_terminator.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_NegationExpression(node) => {
            out.push(MappedNode {
                kind: "NegationExpression",
                span_cp: node.span,
                fields: vec![("chars", vec![node.g_chars.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_LookaheadExpression(node) => {
            out.push(MappedNode {
                kind: "LookaheadExpression",
                span_cp: node.span,
                fields: vec![("pattern", vec![node.g_pattern.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_NegativeLookaheadExpression(node) => {
            out.push(MappedNode {
                kind: "NegativeLookaheadExpression",
                span_cp: node.span,
                fields: vec![("pattern", vec![node.g_pattern.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_CharRangeExpression(node) => {
            out.push(MappedNode {
                kind: "CharRangeExpression",
                span_cp: node.span,
                fields: vec![
                    ("min", vec![node.g_min.as_str()]),
                    ("max", vec![node.g_max.as_str()]),
                ],
            });
        }
        Ast::g_UBNFAST_2e_CIExpression(node) => {
            out.push(MappedNode {
                kind: "CIExpression",
                span_cp: node.span,
                fields: vec![("word", vec![node.g_word.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_RegexExpression(node) => {
            out.push(MappedNode {
                kind: "RegexExpression",
                span_cp: node.span,
                fields: vec![("pattern", vec![node.g_pattern.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_AnyExpression(node) => {
            out.push(MappedNode {
                kind: "AnyExpression",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_EofExpression(node) => {
            out.push(MappedNode {
                kind: "EofExpression",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_EmptyExpression(node) => {
            out.push(MappedNode {
                kind: "EmptyExpression",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_RuleDecl(node) => {
            for child in &node.g_annotations {
                project(child, out);
            }
            project(&node.g_body, out);
            out.push(MappedNode {
                kind: "RuleDecl",
                span_cp: node.span,
                fields: vec![("name", vec![node.g_name.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_RootAnnotation(node) => {
            out.push(MappedNode {
                kind: "RootAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_MappingAnnotation(node) => {
            out.push(MappedNode {
                kind: "MappingAnnotation",
                span_cp: node.span,
                fields: vec![
                    ("className", vec![node.g_className.as_str()]),
                    (
                        "paramNames",
                        node.g_paramNames.iter().map(String::as_str).collect(),
                    ),
                ],
            });
        }
        Ast::g_UBNFAST_2e_EvalAnnotation(node) => {
            for child in &node.g_entries {
                project(child, out);
            }
            out.push(MappedNode {
                kind: "EvalAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_WhitespaceAnnotation(node) => {
            out.push(MappedNode {
                kind: "WhitespaceAnnotation",
                span_cp: node.span,
                fields: vec![("style", node.g_style.as_deref().into_iter().collect())],
            });
        }
        Ast::g_UBNFAST_2e_InterleaveAnnotation(node) => {
            out.push(MappedNode {
                kind: "InterleaveAnnotation",
                span_cp: node.span,
                fields: vec![("profile", vec![node.g_profile.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_BackrefAnnotation(node) => {
            out.push(MappedNode {
                kind: "BackrefAnnotation",
                span_cp: node.span,
                fields: vec![("name", vec![node.g_name.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_ScopeTreeAnnotation(node) => {
            out.push(MappedNode {
                kind: "ScopeTreeAnnotation",
                span_cp: node.span,
                fields: vec![("mode", vec![node.g_mode.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_DeclaresAnnotation(node) => {
            out.push(MappedNode {
                kind: "DeclaresAnnotation",
                span_cp: node.span,
                fields: vec![
                    ("symbol", vec![node.g_symbol.as_str()]),
                    (
                        "description",
                        node.g_description.as_deref().into_iter().collect(),
                    ),
                ],
            });
        }
        Ast::g_UBNFAST_2e_CatalogAnnotation(node) => {
            out.push(MappedNode {
                kind: "CatalogAnnotation",
                span_cp: node.span,
                fields: vec![("context", vec![node.g_context.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_LeftAssocAnnotation(node) => {
            out.push(MappedNode {
                kind: "LeftAssocAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_RightAssocAnnotation(node) => {
            out.push(MappedNode {
                kind: "RightAssocAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_LongestChoiceAnnotation(node) => {
            out.push(MappedNode {
                kind: "LongestChoiceAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(node) => {
            out.push(MappedNode {
                kind: "PredictiveChoiceAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_PrecedenceAnnotation(node) => {
            out.push(MappedNode {
                kind: "PrecedenceAnnotation",
                span_cp: node.span,
                fields: vec![("level", vec![node.g_level.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_DocAnnotation(node) => {
            out.push(MappedNode {
                kind: "DocAnnotation",
                span_cp: node.span,
                fields: vec![("text", vec![node.g_text.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_RecoveryAnnotation(node) => {
            out.push(MappedNode {
                kind: "RecoveryAnnotation",
                span_cp: node.span,
                fields: vec![
                    ("mode", node.g_mode.as_deref().into_iter().collect()),
                    (
                        "syncTokens",
                        node.g_syncTokens.as_deref().into_iter().collect(),
                    ),
                ],
            });
        }
        Ast::g_UBNFAST_2e_SkipAnnotation(node) => {
            out.push(MappedNode {
                kind: "SkipAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_CommonFieldAnnotation(node) => {
            out.push(MappedNode {
                kind: "CommonFieldAnnotation",
                span_cp: node.span,
                fields: vec![(
                    "fieldNames",
                    node.g_fieldNames.iter().map(String::as_str).collect(),
                )],
            });
        }
        Ast::g_UBNFAST_2e_EnumAnnotation(node) => {
            out.push(MappedNode {
                kind: "EnumAnnotation",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_SimpleAnnotation(node) => {
            out.push(MappedNode {
                kind: "SimpleAnnotation",
                span_cp: node.span,
                fields: vec![("name", vec![node.g_name.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_ChoiceBody(node) => {
            for child in &node.g_alternatives {
                project(child, out);
            }
            out.push(MappedNode {
                kind: "ChoiceBody",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_SequenceBody(node) => {
            for child in &node.g_elements {
                project(child, out);
            }
            out.push(MappedNode {
                kind: "SequenceBody",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_AnnotatedElement(node) => {
            project(&node.g_element, out);
            if let Some(child) = &node.g_typeofConstraint {
                project(child, out);
            }
            out.push(MappedNode {
                kind: "AnnotatedElement",
                span_cp: node.span,
                fields: vec![(
                    "captureName",
                    node.g_captureName.as_deref().into_iter().collect(),
                )],
            });
        }
        Ast::g_UBNFAST_2e_GroupElement(node) => {
            project(&node.g_body, out);
            out.push(MappedNode {
                kind: "GroupElement",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_OptionalElement(node) => {
            project(&node.g_body, out);
            out.push(MappedNode {
                kind: "OptionalElement",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_RepeatElement(node) => {
            project(&node.g_body, out);
            out.push(MappedNode {
                kind: "RepeatElement",
                span_cp: node.span,
                fields: vec![],
            });
        }
        Ast::g_UBNFAST_2e_ErrorElement(node) => {
            out.push(MappedNode {
                kind: "ErrorElement",
                span_cp: node.span,
                fields: vec![("message", vec![node.g_message.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_TerminalElement(node) => {
            out.push(MappedNode {
                kind: "TerminalElement",
                span_cp: node.span,
                fields: vec![("value", vec![node.g_value.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_TypeofElement(node) => {
            out.push(MappedNode {
                kind: "TypeofElement",
                span_cp: node.span,
                fields: vec![("captureName", vec![node.g_captureName.as_str()])],
            });
        }
        Ast::g_UBNFAST_2e_DottedIdentifier(node) => {
            out.push(MappedNode {
                kind: "DottedIdentifier",
                span_cp: node.span,
                fields: vec![("parts", node.g_parts.iter().map(String::as_str).collect())],
            });
        }
    }
}
