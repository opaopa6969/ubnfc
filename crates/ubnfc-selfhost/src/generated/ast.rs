//! 採用結果だけから構築する AST。数値 capture は字句 String。
#![allow(non_camel_case_types, non_snake_case)]
use super::api::Span;
#[derive(Clone,Debug,PartialEq)]
pub enum Ast { Text(String), Null,
g_UBNFAST_2e_UBNFFile(g_UBNFAST_2e_UBNFFile),
g_UBNFAST_2e_ImportDecl(g_UBNFAST_2e_ImportDecl),
g_UBNFAST_2e_GrammarDecl(g_UBNFAST_2e_GrammarDecl),
g_UBNFAST_2e_GlobalSetting(g_UBNFAST_2e_GlobalSetting),
g_UBNFAST_2e_StringSettingValue(g_UBNFAST_2e_StringSettingValue),
g_UBNFAST_2e_BlockSettingValue(g_UBNFAST_2e_BlockSettingValue),
g_UBNFAST_2e_KeyValuePair(g_UBNFAST_2e_KeyValuePair),
g_UBNFAST_2e_TokenDecl(g_UBNFAST_2e_TokenDecl),
g_UBNFAST_2e_UntilExpression(g_UBNFAST_2e_UntilExpression),
g_UBNFAST_2e_NegationExpression(g_UBNFAST_2e_NegationExpression),
g_UBNFAST_2e_LookaheadExpression(g_UBNFAST_2e_LookaheadExpression),
g_UBNFAST_2e_NegativeLookaheadExpression(g_UBNFAST_2e_NegativeLookaheadExpression),
g_UBNFAST_2e_CharRangeExpression(g_UBNFAST_2e_CharRangeExpression),
g_UBNFAST_2e_CIExpression(g_UBNFAST_2e_CIExpression),
g_UBNFAST_2e_RegexExpression(g_UBNFAST_2e_RegexExpression),
g_UBNFAST_2e_AnyExpression(g_UBNFAST_2e_AnyExpression),
g_UBNFAST_2e_EofExpression(g_UBNFAST_2e_EofExpression),
g_UBNFAST_2e_EmptyExpression(g_UBNFAST_2e_EmptyExpression),
g_UBNFAST_2e_RuleDecl(g_UBNFAST_2e_RuleDecl),
g_UBNFAST_2e_RootAnnotation(g_UBNFAST_2e_RootAnnotation),
g_UBNFAST_2e_MappingAnnotation(g_UBNFAST_2e_MappingAnnotation),
g_UBNFAST_2e_EvalAnnotation(g_UBNFAST_2e_EvalAnnotation),
g_UBNFAST_2e_WhitespaceAnnotation(g_UBNFAST_2e_WhitespaceAnnotation),
g_UBNFAST_2e_InterleaveAnnotation(g_UBNFAST_2e_InterleaveAnnotation),
g_UBNFAST_2e_BackrefAnnotation(g_UBNFAST_2e_BackrefAnnotation),
g_UBNFAST_2e_ScopeTreeAnnotation(g_UBNFAST_2e_ScopeTreeAnnotation),
g_UBNFAST_2e_DeclaresAnnotation(g_UBNFAST_2e_DeclaresAnnotation),
g_UBNFAST_2e_CatalogAnnotation(g_UBNFAST_2e_CatalogAnnotation),
g_UBNFAST_2e_LeftAssocAnnotation(g_UBNFAST_2e_LeftAssocAnnotation),
g_UBNFAST_2e_RightAssocAnnotation(g_UBNFAST_2e_RightAssocAnnotation),
g_UBNFAST_2e_LongestChoiceAnnotation(g_UBNFAST_2e_LongestChoiceAnnotation),
g_UBNFAST_2e_PredictiveChoiceAnnotation(g_UBNFAST_2e_PredictiveChoiceAnnotation),
g_UBNFAST_2e_PrecedenceAnnotation(g_UBNFAST_2e_PrecedenceAnnotation),
g_UBNFAST_2e_DocAnnotation(g_UBNFAST_2e_DocAnnotation),
g_UBNFAST_2e_RecoveryAnnotation(g_UBNFAST_2e_RecoveryAnnotation),
g_UBNFAST_2e_SkipAnnotation(g_UBNFAST_2e_SkipAnnotation),
g_UBNFAST_2e_CommonFieldAnnotation(g_UBNFAST_2e_CommonFieldAnnotation),
g_UBNFAST_2e_EnumAnnotation(g_UBNFAST_2e_EnumAnnotation),
g_UBNFAST_2e_SimpleAnnotation(g_UBNFAST_2e_SimpleAnnotation),
g_UBNFAST_2e_ChoiceBody(g_UBNFAST_2e_ChoiceBody),
g_UBNFAST_2e_SequenceBody(g_UBNFAST_2e_SequenceBody),
g_UBNFAST_2e_AnnotatedElement(g_UBNFAST_2e_AnnotatedElement),
g_UBNFAST_2e_GroupElement(g_UBNFAST_2e_GroupElement),
g_UBNFAST_2e_OptionalElement(g_UBNFAST_2e_OptionalElement),
g_UBNFAST_2e_RepeatElement(g_UBNFAST_2e_RepeatElement),
g_UBNFAST_2e_ErrorElement(g_UBNFAST_2e_ErrorElement),
g_UBNFAST_2e_TerminalElement(g_UBNFAST_2e_TerminalElement),
g_UBNFAST_2e_TypeofElement(g_UBNFAST_2e_TypeofElement),
g_UBNFAST_2e_DottedIdentifier(g_UBNFAST_2e_DottedIdentifier),
}
pub type g_UBNFAST = Ast;
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_UBNFFile { pub span:Span, pub node_id:usize,
pub g_grammars: Vec<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_ImportDecl { pub span:Span, pub node_id:usize,
pub g_alias: String,
pub g_path: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_GrammarDecl { pub span:Span, pub node_id:usize,
pub g_name: String,
pub g_imports: Vec<Ast>,
pub g_settings: Vec<Ast>,
pub g_tokens: Vec<Ast>,
pub g_rules: Vec<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_GlobalSetting { pub span:Span, pub node_id:usize,
pub g_key: String,
pub g_value: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_StringSettingValue { pub span:Span, pub node_id:usize,
pub g_value: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_BlockSettingValue { pub span:Span, pub node_id:usize,
pub g_entries: Vec<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_KeyValuePair { pub span:Span, pub node_id:usize,
pub g_key: String,
pub g_value: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_TokenDecl { pub span:Span, pub node_id:usize,
pub g_name: String,
pub g_value: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_UntilExpression { pub span:Span, pub node_id:usize,
pub g_terminator: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_NegationExpression { pub span:Span, pub node_id:usize,
pub g_chars: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_LookaheadExpression { pub span:Span, pub node_id:usize,
pub g_pattern: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_NegativeLookaheadExpression { pub span:Span, pub node_id:usize,
pub g_pattern: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_CharRangeExpression { pub span:Span, pub node_id:usize,
pub g_min: String,
pub g_max: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_CIExpression { pub span:Span, pub node_id:usize,
pub g_word: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_RegexExpression { pub span:Span, pub node_id:usize,
pub g_pattern: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_AnyExpression { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_EofExpression { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_EmptyExpression { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_RuleDecl { pub span:Span, pub node_id:usize,
pub g_annotations: Vec<Ast>,
pub g_name: String,
pub g_body: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_RootAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_MappingAnnotation { pub span:Span, pub node_id:usize,
pub g_className: String,
pub g_paramNames: Vec<String>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_EvalAnnotation { pub span:Span, pub node_id:usize,
pub g_entries: Vec<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_WhitespaceAnnotation { pub span:Span, pub node_id:usize,
pub g_style: Option<String>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_InterleaveAnnotation { pub span:Span, pub node_id:usize,
pub g_profile: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_BackrefAnnotation { pub span:Span, pub node_id:usize,
pub g_name: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_ScopeTreeAnnotation { pub span:Span, pub node_id:usize,
pub g_mode: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_DeclaresAnnotation { pub span:Span, pub node_id:usize,
pub g_symbol: String,
pub g_description: Option<String>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_CatalogAnnotation { pub span:Span, pub node_id:usize,
pub g_context: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_LeftAssocAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_RightAssocAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_LongestChoiceAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_PredictiveChoiceAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_PrecedenceAnnotation { pub span:Span, pub node_id:usize,
pub g_level: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_DocAnnotation { pub span:Span, pub node_id:usize,
pub g_text: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_RecoveryAnnotation { pub span:Span, pub node_id:usize,
pub g_mode: Option<String>,
pub g_syncTokens: Option<String>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_SkipAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_CommonFieldAnnotation { pub span:Span, pub node_id:usize,
pub g_fieldNames: Vec<String>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_EnumAnnotation { pub span:Span, pub node_id:usize,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_SimpleAnnotation { pub span:Span, pub node_id:usize,
pub g_name: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_ChoiceBody { pub span:Span, pub node_id:usize,
pub g_alternatives: Vec<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_SequenceBody { pub span:Span, pub node_id:usize,
pub g_elements: Vec<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_AnnotatedElement { pub span:Span, pub node_id:usize,
pub g_element: Box<Ast>,
pub g_captureName: Option<String>,
pub g_typeofConstraint: Option<Box<Ast>>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_GroupElement { pub span:Span, pub node_id:usize,
pub g_body: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_OptionalElement { pub span:Span, pub node_id:usize,
pub g_body: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_RepeatElement { pub span:Span, pub node_id:usize,
pub g_body: Box<Ast>,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_ErrorElement { pub span:Span, pub node_id:usize,
pub g_message: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_TerminalElement { pub span:Span, pub node_id:usize,
pub g_value: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_TypeofElement { pub span:Span, pub node_id:usize,
pub g_captureName: String,
}
#[derive(Clone,Debug,PartialEq)] pub struct g_UBNFAST_2e_DottedIdentifier { pub span:Span, pub node_id:usize,
pub g_parts: Vec<String>,
}
impl Ast { pub fn span(&self)->Option<Span> { match self {Self::Text(_)|Self::Null=>None,
Self::g_UBNFAST_2e_UBNFFile(node)=>Some(node.span),
Self::g_UBNFAST_2e_ImportDecl(node)=>Some(node.span),
Self::g_UBNFAST_2e_GrammarDecl(node)=>Some(node.span),
Self::g_UBNFAST_2e_GlobalSetting(node)=>Some(node.span),
Self::g_UBNFAST_2e_StringSettingValue(node)=>Some(node.span),
Self::g_UBNFAST_2e_BlockSettingValue(node)=>Some(node.span),
Self::g_UBNFAST_2e_KeyValuePair(node)=>Some(node.span),
Self::g_UBNFAST_2e_TokenDecl(node)=>Some(node.span),
Self::g_UBNFAST_2e_UntilExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_NegationExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_LookaheadExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_NegativeLookaheadExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_CharRangeExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_CIExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_RegexExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_AnyExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_EofExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_EmptyExpression(node)=>Some(node.span),
Self::g_UBNFAST_2e_RuleDecl(node)=>Some(node.span),
Self::g_UBNFAST_2e_RootAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_MappingAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_EvalAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_WhitespaceAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_InterleaveAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_BackrefAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_ScopeTreeAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_DeclaresAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_CatalogAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_LeftAssocAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_RightAssocAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_LongestChoiceAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_PredictiveChoiceAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_PrecedenceAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_DocAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_RecoveryAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_SkipAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_CommonFieldAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_EnumAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_SimpleAnnotation(node)=>Some(node.span),
Self::g_UBNFAST_2e_ChoiceBody(node)=>Some(node.span),
Self::g_UBNFAST_2e_SequenceBody(node)=>Some(node.span),
Self::g_UBNFAST_2e_AnnotatedElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_GroupElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_OptionalElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_RepeatElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_ErrorElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_TerminalElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_TypeofElement(node)=>Some(node.span),
Self::g_UBNFAST_2e_DottedIdentifier(node)=>Some(node.span),
}}
pub fn type_name(&self)->&'static str { match self {Self::Text(_)=>"text",Self::Null=>"null",
Self::g_UBNFAST_2e_UBNFFile(_)=>"UBNFFile",
Self::g_UBNFAST_2e_ImportDecl(_)=>"ImportDecl",
Self::g_UBNFAST_2e_GrammarDecl(_)=>"GrammarDecl",
Self::g_UBNFAST_2e_GlobalSetting(_)=>"GlobalSetting",
Self::g_UBNFAST_2e_StringSettingValue(_)=>"StringSettingValue",
Self::g_UBNFAST_2e_BlockSettingValue(_)=>"BlockSettingValue",
Self::g_UBNFAST_2e_KeyValuePair(_)=>"KeyValuePair",
Self::g_UBNFAST_2e_TokenDecl(_)=>"TokenDecl",
Self::g_UBNFAST_2e_UntilExpression(_)=>"UntilExpression",
Self::g_UBNFAST_2e_NegationExpression(_)=>"NegationExpression",
Self::g_UBNFAST_2e_LookaheadExpression(_)=>"LookaheadExpression",
Self::g_UBNFAST_2e_NegativeLookaheadExpression(_)=>"NegativeLookaheadExpression",
Self::g_UBNFAST_2e_CharRangeExpression(_)=>"CharRangeExpression",
Self::g_UBNFAST_2e_CIExpression(_)=>"CIExpression",
Self::g_UBNFAST_2e_RegexExpression(_)=>"RegexExpression",
Self::g_UBNFAST_2e_AnyExpression(_)=>"AnyExpression",
Self::g_UBNFAST_2e_EofExpression(_)=>"EofExpression",
Self::g_UBNFAST_2e_EmptyExpression(_)=>"EmptyExpression",
Self::g_UBNFAST_2e_RuleDecl(_)=>"RuleDecl",
Self::g_UBNFAST_2e_RootAnnotation(_)=>"RootAnnotation",
Self::g_UBNFAST_2e_MappingAnnotation(_)=>"MappingAnnotation",
Self::g_UBNFAST_2e_EvalAnnotation(_)=>"EvalAnnotation",
Self::g_UBNFAST_2e_WhitespaceAnnotation(_)=>"WhitespaceAnnotation",
Self::g_UBNFAST_2e_InterleaveAnnotation(_)=>"InterleaveAnnotation",
Self::g_UBNFAST_2e_BackrefAnnotation(_)=>"BackrefAnnotation",
Self::g_UBNFAST_2e_ScopeTreeAnnotation(_)=>"ScopeTreeAnnotation",
Self::g_UBNFAST_2e_DeclaresAnnotation(_)=>"DeclaresAnnotation",
Self::g_UBNFAST_2e_CatalogAnnotation(_)=>"CatalogAnnotation",
Self::g_UBNFAST_2e_LeftAssocAnnotation(_)=>"LeftAssocAnnotation",
Self::g_UBNFAST_2e_RightAssocAnnotation(_)=>"RightAssocAnnotation",
Self::g_UBNFAST_2e_LongestChoiceAnnotation(_)=>"LongestChoiceAnnotation",
Self::g_UBNFAST_2e_PredictiveChoiceAnnotation(_)=>"PredictiveChoiceAnnotation",
Self::g_UBNFAST_2e_PrecedenceAnnotation(_)=>"PrecedenceAnnotation",
Self::g_UBNFAST_2e_DocAnnotation(_)=>"DocAnnotation",
Self::g_UBNFAST_2e_RecoveryAnnotation(_)=>"RecoveryAnnotation",
Self::g_UBNFAST_2e_SkipAnnotation(_)=>"SkipAnnotation",
Self::g_UBNFAST_2e_CommonFieldAnnotation(_)=>"CommonFieldAnnotation",
Self::g_UBNFAST_2e_EnumAnnotation(_)=>"EnumAnnotation",
Self::g_UBNFAST_2e_SimpleAnnotation(_)=>"SimpleAnnotation",
Self::g_UBNFAST_2e_ChoiceBody(_)=>"ChoiceBody",
Self::g_UBNFAST_2e_SequenceBody(_)=>"SequenceBody",
Self::g_UBNFAST_2e_AnnotatedElement(_)=>"AnnotatedElement",
Self::g_UBNFAST_2e_GroupElement(_)=>"GroupElement",
Self::g_UBNFAST_2e_OptionalElement(_)=>"OptionalElement",
Self::g_UBNFAST_2e_RepeatElement(_)=>"RepeatElement",
Self::g_UBNFAST_2e_ErrorElement(_)=>"ErrorElement",
Self::g_UBNFAST_2e_TerminalElement(_)=>"TerminalElement",
Self::g_UBNFAST_2e_TypeofElement(_)=>"TypeofElement",
Self::g_UBNFAST_2e_DottedIdentifier(_)=>"DottedIdentifier",
}}
pub fn type_id(&self)->&'static str {match self {Self::Text(_)=>"text",Self::Null=>"null",
Self::g_UBNFAST_2e_UBNFFile(_)=>"UBNFAST.UBNFFile",
Self::g_UBNFAST_2e_ImportDecl(_)=>"UBNFAST.ImportDecl",
Self::g_UBNFAST_2e_GrammarDecl(_)=>"UBNFAST.GrammarDecl",
Self::g_UBNFAST_2e_GlobalSetting(_)=>"UBNFAST.GlobalSetting",
Self::g_UBNFAST_2e_StringSettingValue(_)=>"UBNFAST.StringSettingValue",
Self::g_UBNFAST_2e_BlockSettingValue(_)=>"UBNFAST.BlockSettingValue",
Self::g_UBNFAST_2e_KeyValuePair(_)=>"UBNFAST.KeyValuePair",
Self::g_UBNFAST_2e_TokenDecl(_)=>"UBNFAST.TokenDecl",
Self::g_UBNFAST_2e_UntilExpression(_)=>"UBNFAST.UntilExpression",
Self::g_UBNFAST_2e_NegationExpression(_)=>"UBNFAST.NegationExpression",
Self::g_UBNFAST_2e_LookaheadExpression(_)=>"UBNFAST.LookaheadExpression",
Self::g_UBNFAST_2e_NegativeLookaheadExpression(_)=>"UBNFAST.NegativeLookaheadExpression",
Self::g_UBNFAST_2e_CharRangeExpression(_)=>"UBNFAST.CharRangeExpression",
Self::g_UBNFAST_2e_CIExpression(_)=>"UBNFAST.CIExpression",
Self::g_UBNFAST_2e_RegexExpression(_)=>"UBNFAST.RegexExpression",
Self::g_UBNFAST_2e_AnyExpression(_)=>"UBNFAST.AnyExpression",
Self::g_UBNFAST_2e_EofExpression(_)=>"UBNFAST.EofExpression",
Self::g_UBNFAST_2e_EmptyExpression(_)=>"UBNFAST.EmptyExpression",
Self::g_UBNFAST_2e_RuleDecl(_)=>"UBNFAST.RuleDecl",
Self::g_UBNFAST_2e_RootAnnotation(_)=>"UBNFAST.RootAnnotation",
Self::g_UBNFAST_2e_MappingAnnotation(_)=>"UBNFAST.MappingAnnotation",
Self::g_UBNFAST_2e_EvalAnnotation(_)=>"UBNFAST.EvalAnnotation",
Self::g_UBNFAST_2e_WhitespaceAnnotation(_)=>"UBNFAST.WhitespaceAnnotation",
Self::g_UBNFAST_2e_InterleaveAnnotation(_)=>"UBNFAST.InterleaveAnnotation",
Self::g_UBNFAST_2e_BackrefAnnotation(_)=>"UBNFAST.BackrefAnnotation",
Self::g_UBNFAST_2e_ScopeTreeAnnotation(_)=>"UBNFAST.ScopeTreeAnnotation",
Self::g_UBNFAST_2e_DeclaresAnnotation(_)=>"UBNFAST.DeclaresAnnotation",
Self::g_UBNFAST_2e_CatalogAnnotation(_)=>"UBNFAST.CatalogAnnotation",
Self::g_UBNFAST_2e_LeftAssocAnnotation(_)=>"UBNFAST.LeftAssocAnnotation",
Self::g_UBNFAST_2e_RightAssocAnnotation(_)=>"UBNFAST.RightAssocAnnotation",
Self::g_UBNFAST_2e_LongestChoiceAnnotation(_)=>"UBNFAST.LongestChoiceAnnotation",
Self::g_UBNFAST_2e_PredictiveChoiceAnnotation(_)=>"UBNFAST.PredictiveChoiceAnnotation",
Self::g_UBNFAST_2e_PrecedenceAnnotation(_)=>"UBNFAST.PrecedenceAnnotation",
Self::g_UBNFAST_2e_DocAnnotation(_)=>"UBNFAST.DocAnnotation",
Self::g_UBNFAST_2e_RecoveryAnnotation(_)=>"UBNFAST.RecoveryAnnotation",
Self::g_UBNFAST_2e_SkipAnnotation(_)=>"UBNFAST.SkipAnnotation",
Self::g_UBNFAST_2e_CommonFieldAnnotation(_)=>"UBNFAST.CommonFieldAnnotation",
Self::g_UBNFAST_2e_EnumAnnotation(_)=>"UBNFAST.EnumAnnotation",
Self::g_UBNFAST_2e_SimpleAnnotation(_)=>"UBNFAST.SimpleAnnotation",
Self::g_UBNFAST_2e_ChoiceBody(_)=>"UBNFAST.ChoiceBody",
Self::g_UBNFAST_2e_SequenceBody(_)=>"UBNFAST.SequenceBody",
Self::g_UBNFAST_2e_AnnotatedElement(_)=>"UBNFAST.AnnotatedElement",
Self::g_UBNFAST_2e_GroupElement(_)=>"UBNFAST.GroupElement",
Self::g_UBNFAST_2e_OptionalElement(_)=>"UBNFAST.OptionalElement",
Self::g_UBNFAST_2e_RepeatElement(_)=>"UBNFAST.RepeatElement",
Self::g_UBNFAST_2e_ErrorElement(_)=>"UBNFAST.ErrorElement",
Self::g_UBNFAST_2e_TerminalElement(_)=>"UBNFAST.TerminalElement",
Self::g_UBNFAST_2e_TypeofElement(_)=>"UBNFAST.TypeofElement",
Self::g_UBNFAST_2e_DottedIdentifier(_)=>"UBNFAST.DottedIdentifier",
}}
#[cfg(feature="json")] pub fn canonical_value(&self)->serde_json::Value { match self { Self::Text(value)=>serde_json::Value::String(value.clone()),Self::Null=>serde_json::Value::Null,
Self::g_UBNFAST_2e_UBNFFile(node)=>serde_json::json!({"fields":{
"grammars":node.g_grammars.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
},"span":node.span,"type":"UBNFFile"}),
Self::g_UBNFAST_2e_ImportDecl(node)=>serde_json::json!({"fields":{
"alias":node.g_alias,
"path":node.g_path,
},"span":node.span,"type":"ImportDecl"}),
Self::g_UBNFAST_2e_GrammarDecl(node)=>serde_json::json!({"fields":{
"imports":node.g_imports.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
"name":node.g_name,
"rules":node.g_rules.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
"settings":node.g_settings.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
"tokens":node.g_tokens.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
},"span":node.span,"type":"GrammarDecl"}),
Self::g_UBNFAST_2e_GlobalSetting(node)=>serde_json::json!({"fields":{
"key":node.g_key,
"value":node.g_value.canonical_value(),
},"span":node.span,"type":"GlobalSetting"}),
Self::g_UBNFAST_2e_StringSettingValue(node)=>serde_json::json!({"fields":{
"value":node.g_value.canonical_value(),
},"span":node.span,"type":"StringSettingValue"}),
Self::g_UBNFAST_2e_BlockSettingValue(node)=>serde_json::json!({"fields":{
"entries":node.g_entries.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
},"span":node.span,"type":"BlockSettingValue"}),
Self::g_UBNFAST_2e_KeyValuePair(node)=>serde_json::json!({"fields":{
"key":node.g_key,
"value":node.g_value,
},"span":node.span,"type":"KeyValuePair"}),
Self::g_UBNFAST_2e_TokenDecl(node)=>serde_json::json!({"fields":{
"name":node.g_name,
"value":node.g_value.canonical_value(),
},"span":node.span,"type":"TokenDecl"}),
Self::g_UBNFAST_2e_UntilExpression(node)=>serde_json::json!({"fields":{
"terminator":node.g_terminator,
},"span":node.span,"type":"UntilExpression"}),
Self::g_UBNFAST_2e_NegationExpression(node)=>serde_json::json!({"fields":{
"chars":node.g_chars,
},"span":node.span,"type":"NegationExpression"}),
Self::g_UBNFAST_2e_LookaheadExpression(node)=>serde_json::json!({"fields":{
"pattern":node.g_pattern,
},"span":node.span,"type":"LookaheadExpression"}),
Self::g_UBNFAST_2e_NegativeLookaheadExpression(node)=>serde_json::json!({"fields":{
"pattern":node.g_pattern,
},"span":node.span,"type":"NegativeLookaheadExpression"}),
Self::g_UBNFAST_2e_CharRangeExpression(node)=>serde_json::json!({"fields":{
"max":node.g_max,
"min":node.g_min,
},"span":node.span,"type":"CharRangeExpression"}),
Self::g_UBNFAST_2e_CIExpression(node)=>serde_json::json!({"fields":{
"word":node.g_word,
},"span":node.span,"type":"CIExpression"}),
Self::g_UBNFAST_2e_RegexExpression(node)=>serde_json::json!({"fields":{
"pattern":node.g_pattern,
},"span":node.span,"type":"RegexExpression"}),
Self::g_UBNFAST_2e_AnyExpression(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"AnyExpression"}),
Self::g_UBNFAST_2e_EofExpression(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"EofExpression"}),
Self::g_UBNFAST_2e_EmptyExpression(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"EmptyExpression"}),
Self::g_UBNFAST_2e_RuleDecl(node)=>serde_json::json!({"fields":{
"annotations":node.g_annotations.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
"body":node.g_body.canonical_value(),
"name":node.g_name,
},"span":node.span,"type":"RuleDecl"}),
Self::g_UBNFAST_2e_RootAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"RootAnnotation"}),
Self::g_UBNFAST_2e_MappingAnnotation(node)=>serde_json::json!({"fields":{
"className":node.g_className,
"paramNames":node.g_paramNames,
},"span":node.span,"type":"MappingAnnotation"}),
Self::g_UBNFAST_2e_EvalAnnotation(node)=>serde_json::json!({"fields":{
"entries":node.g_entries.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
},"span":node.span,"type":"EvalAnnotation"}),
Self::g_UBNFAST_2e_WhitespaceAnnotation(node)=>serde_json::json!({"fields":{
"style":node.g_style,
},"span":node.span,"type":"WhitespaceAnnotation"}),
Self::g_UBNFAST_2e_InterleaveAnnotation(node)=>serde_json::json!({"fields":{
"profile":node.g_profile,
},"span":node.span,"type":"InterleaveAnnotation"}),
Self::g_UBNFAST_2e_BackrefAnnotation(node)=>serde_json::json!({"fields":{
"name":node.g_name,
},"span":node.span,"type":"BackrefAnnotation"}),
Self::g_UBNFAST_2e_ScopeTreeAnnotation(node)=>serde_json::json!({"fields":{
"mode":node.g_mode,
},"span":node.span,"type":"ScopeTreeAnnotation"}),
Self::g_UBNFAST_2e_DeclaresAnnotation(node)=>serde_json::json!({"fields":{
"description":node.g_description,
"symbol":node.g_symbol,
},"span":node.span,"type":"DeclaresAnnotation"}),
Self::g_UBNFAST_2e_CatalogAnnotation(node)=>serde_json::json!({"fields":{
"context":node.g_context,
},"span":node.span,"type":"CatalogAnnotation"}),
Self::g_UBNFAST_2e_LeftAssocAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"LeftAssocAnnotation"}),
Self::g_UBNFAST_2e_RightAssocAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"RightAssocAnnotation"}),
Self::g_UBNFAST_2e_LongestChoiceAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"LongestChoiceAnnotation"}),
Self::g_UBNFAST_2e_PredictiveChoiceAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"PredictiveChoiceAnnotation"}),
Self::g_UBNFAST_2e_PrecedenceAnnotation(node)=>serde_json::json!({"fields":{
"level":node.g_level,
},"span":node.span,"type":"PrecedenceAnnotation"}),
Self::g_UBNFAST_2e_DocAnnotation(node)=>serde_json::json!({"fields":{
"text":node.g_text,
},"span":node.span,"type":"DocAnnotation"}),
Self::g_UBNFAST_2e_RecoveryAnnotation(node)=>serde_json::json!({"fields":{
"mode":node.g_mode,
"syncTokens":node.g_syncTokens,
},"span":node.span,"type":"RecoveryAnnotation"}),
Self::g_UBNFAST_2e_SkipAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"SkipAnnotation"}),
Self::g_UBNFAST_2e_CommonFieldAnnotation(node)=>serde_json::json!({"fields":{
"fieldNames":node.g_fieldNames,
},"span":node.span,"type":"CommonFieldAnnotation"}),
Self::g_UBNFAST_2e_EnumAnnotation(node)=>serde_json::json!({"fields":{
},"span":node.span,"type":"EnumAnnotation"}),
Self::g_UBNFAST_2e_SimpleAnnotation(node)=>serde_json::json!({"fields":{
"name":node.g_name,
},"span":node.span,"type":"SimpleAnnotation"}),
Self::g_UBNFAST_2e_ChoiceBody(node)=>serde_json::json!({"fields":{
"alternatives":node.g_alternatives.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
},"span":node.span,"type":"ChoiceBody"}),
Self::g_UBNFAST_2e_SequenceBody(node)=>serde_json::json!({"fields":{
"elements":node.g_elements.iter().map(Ast::canonical_value).collect::<Vec<_>>(),
},"span":node.span,"type":"SequenceBody"}),
Self::g_UBNFAST_2e_AnnotatedElement(node)=>serde_json::json!({"fields":{
"captureName":node.g_captureName,
"element":node.g_element.canonical_value(),
"typeofConstraint":node.g_typeofConstraint.as_ref().map(|v|v.canonical_value()),
},"span":node.span,"type":"AnnotatedElement"}),
Self::g_UBNFAST_2e_GroupElement(node)=>serde_json::json!({"fields":{
"body":node.g_body.canonical_value(),
},"span":node.span,"type":"GroupElement"}),
Self::g_UBNFAST_2e_OptionalElement(node)=>serde_json::json!({"fields":{
"body":node.g_body.canonical_value(),
},"span":node.span,"type":"OptionalElement"}),
Self::g_UBNFAST_2e_RepeatElement(node)=>serde_json::json!({"fields":{
"body":node.g_body.canonical_value(),
},"span":node.span,"type":"RepeatElement"}),
Self::g_UBNFAST_2e_ErrorElement(node)=>serde_json::json!({"fields":{
"message":node.g_message,
},"span":node.span,"type":"ErrorElement"}),
Self::g_UBNFAST_2e_TerminalElement(node)=>serde_json::json!({"fields":{
"value":node.g_value,
},"span":node.span,"type":"TerminalElement"}),
Self::g_UBNFAST_2e_TypeofElement(node)=>serde_json::json!({"fields":{
"captureName":node.g_captureName,
},"span":node.span,"type":"TypeofElement"}),
Self::g_UBNFAST_2e_DottedIdentifier(node)=>serde_json::json!({"fields":{
"parts":node.g_parts,
},"span":node.span,"type":"DottedIdentifier"}),
}}
#[cfg(feature="json")] pub fn canonical_json(&self)->String {serde_json::to_string(&self.canonical_value()).expect("finite canonical AST")}
}
#[cfg(feature="json")] impl serde::Serialize for Ast {fn serialize<S:serde::Serializer>(&self,serializer:S)->Result<S::Ok,S::Error> {self.canonical_value().serialize(serializer)}}
