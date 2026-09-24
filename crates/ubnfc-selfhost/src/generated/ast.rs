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
/// D-077: arena 形の AST。所有 `Ast` はこの木から写す。
pub mod tree {
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code, clippy::all)]
// D-077: 採用結果の AST を arena（SoA）で持つ形。所有 `Ast` は常にこの木から写す。
//
// - 節点は `nodes: Vec<TreeNode>`（24 byte）に作った順で並ぶ。子は `slots: Vec<u32>` の添字で指す。
// - 文字列は持たない。Text 節点は入力の byte 範囲 `[a, b)` で、値は常に入力の部分文字列
//   （semantic text は trim 済み範囲か token の content 範囲で、どちらも入力の中にある）。
// - record の field は宣言順に slot を持つ: scalar / optional は 1 slot（空は `NONE`）、
//   list は 2 slot（`slots` 内の開始位置と個数。要素は record より先に `slots` へ積む）。
// - `#[doc(hidden)] pub` の構築用メソッドは生成 runtime 専用（driver が runtime を別 crate へ
//   `#[path]` で取り込むため crate 内可視にできない）。利用者は呼ばない。
// - 作ったが採用されなかった節点（値の包み直しで捨てた Text 等）も `nodes` に残る。
//   根から辿れないだけで観測には出ない（所有 `Ast` と同じ）。
use super::super::api::{NodeSpan, Span, ValueSpan};
/// 空の slot（optional の無値）と、根の無い木。
pub const NONE: u32 = u32::MAX;
pub const KIND_TEXT: u16 = 0;
pub const KIND_NULL: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TreeNode {
    kind: u16,
    /// 節点を作った規則（`RULE_IDS` の添字）。Text / Null は 0。
    rule: u16,
    /// 型付き節点の作成順（`NodeSpan::node_id`）。Text / Null は 0。
    node_id: u32,
    /// code point の範囲。Text は値の出自（extent）。
    span: [u32; 2],
    /// Text: byte 範囲。record: 最初の slot。enum: variant の番号。
    a: u32,
    b: u32,
}
/// 1 回の parse の AST を arena に持つ木（`ParseOptions::ast_tree`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstTree {
    source: String,
    nodes: Vec<TreeNode>,
    slots: Vec<u32>,
    typed: u32,
    root: u32,
}
impl Default for AstTree {
    fn default() -> Self {
        Self {
            source: String::new(),
            nodes: Vec::new(),
            slots: Vec::new(),
            typed: 0,
            root: NONE,
        }
    }
}
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).expect("AST tree offsets fit in u32 (input_bytes <= u32::MAX)")
}
fn span_of(node: &TreeNode) -> Span {
    [node.span[0] as usize, node.span[1] as usize]
}
impl AstTree {
    /// 解析した入力（Text 節点の値はこの部分文字列）。
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn root(&self) -> Option<NodeRef<'_>> {
        (self.root != NONE).then_some(NodeRef {
            tree: self,
            id: self.root,
        })
    }
    /// 任意の節点（`NodeRef::id` の値）。範囲外は `None`。
    pub fn node(&self, id: u32) -> Option<NodeRef<'_>> {
        ((id as usize) < self.nodes.len()).then_some(NodeRef { tree: self, id })
    }
    /// arena の節点数（Text と、根から辿れない節点を含む）。
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    /// 子の参照に使っている slot（u32）の数。
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }
    /// 型付き節点の数（`Statistics::ast_nodes` と同じ）。
    pub fn typed_node_count(&self) -> usize {
        self.typed as usize
    }
    /// 所有 `Ast` へ写す（`ParseOptions::build_ast` が作るものと同じ値）。
    pub fn to_ast(&self) -> Option<super::Ast> {
        self.root().map(|root| root.to_ast())
    }
    /// `ParseResult::node_spans` と同じ表（作成順）。
    pub fn node_spans(&self) -> Vec<NodeSpan> {
        let mut out = Vec::with_capacity(self.typed as usize);
        self.push_node_spans(&mut out);
        out
    }
    /// `ParseOptions::value_spans` と同じ表（canonical AST の JSON pointer 順）。
    pub fn value_spans(&self) -> Vec<ValueSpan> {
        let mut out = Vec::new();
        if self.root != NONE {
            let mut path = String::with_capacity(128);
            self.collect_value_spans(&self.source, self.root, &mut path, &mut out);
        }
        out
    }
    /// この木が確保している byte 数（容量ベース）。
    pub fn heap_bytes(&self) -> usize {
        self.source.capacity()
            + self.nodes.capacity() * std::mem::size_of::<TreeNode>()
            + self.slots.capacity() * std::mem::size_of::<u32>()
    }
    #[doc(hidden)]
    pub fn push_node_spans(&self, out: &mut Vec<NodeSpan>) {
        for node in &self.nodes {
            if node.kind > KIND_NULL {
                out.push(NodeSpan {
                    node_id: node.node_id as usize,
                    rule_id: RULE_IDS[node.rule as usize],
                    node_type: TYPE_NAMES[node.kind as usize],
                    span: span_of(node),
                });
            }
        }
    }
    #[doc(hidden)]
    pub fn clear(&mut self) {
        self.source.clear();
        self.nodes.clear();
        self.slots.clear();
        self.typed = 0;
        self.root = NONE;
    }
    #[doc(hidden)]
    pub fn reserve(&mut self, nodes: usize, slots: usize) {
        self.nodes.reserve(nodes);
        self.slots.reserve(slots);
    }
    #[doc(hidden)]
    pub fn set_root(&mut self, root: u32) {
        self.root = root;
    }
    #[doc(hidden)]
    pub fn set_source(&mut self, source: &str) {
        self.source.clear();
        self.source.push_str(source);
    }
    #[inline]
    fn push(&mut self, node: TreeNode) -> u32 {
        let id = to_u32(self.nodes.len());
        self.nodes.push(node);
        id
    }
    /// 入力 `[bytes[0], bytes[1])` を値とする Text 節点。`extent` は値の出自（code point）。
    #[inline]
    #[doc(hidden)]
    pub fn text(&mut self, extent: Span, bytes: [usize; 2]) -> u32 {
        self.push(TreeNode {
            kind: KIND_TEXT,
            rule: 0,
            node_id: 0,
            span: [to_u32(extent[0]), to_u32(extent[1])],
            a: to_u32(bytes[0]),
            b: to_u32(bytes[1]),
        })
    }
    /// Text の出自を付け直す（leaf 型へ昇格した Text は capture の範囲を出自にする）。
    #[doc(hidden)]
    pub fn set_extent(&mut self, id: u32, extent: Span) {
        self.nodes[id as usize].span = [to_u32(extent[0]), to_u32(extent[1])];
    }
    /// list field の要素を積み、record の 2 slot（開始, 個数）を返す。
    #[inline]
    #[doc(hidden)]
    pub fn list(&mut self, items: &[u32]) -> [u32; 2] {
        let start = to_u32(self.slots.len());
        self.slots.extend_from_slice(items);
        [start, to_u32(items.len())]
    }
    #[inline]
    #[doc(hidden)]
    pub fn record(&mut self, kind: u16, rule: u16, span: Span, fields: &[u32]) -> u32 {
        let first = to_u32(self.slots.len());
        self.slots.extend_from_slice(fields);
        let node_id = self.typed;
        self.typed += 1;
        self.push(TreeNode {
            kind,
            rule,
            node_id,
            span: [to_u32(span[0]), to_u32(span[1])],
            a: first,
            b: to_u32(fields.len()),
        })
    }
    #[inline]
    #[doc(hidden)]
    pub fn enumeration(&mut self, kind: u16, rule: u16, span: Span, value: u32) -> u32 {
        let node_id = self.typed;
        self.typed += 1;
        self.push(TreeNode {
            kind,
            rule,
            node_id,
            span: [to_u32(span[0]), to_u32(span[1])],
            a: value,
            b: 0,
        })
    }
    #[inline]
    #[doc(hidden)]
    pub fn kind(&self, id: u32) -> u16 {
        self.nodes[id as usize].kind
    }
    #[doc(hidden)]
    pub fn type_id_of(&self, id: u32) -> &'static str {
        TYPE_IDS[self.kind(id) as usize]
    }
    #[doc(hidden)]
    pub fn type_name_of(&self, id: u32) -> &'static str {
        TYPE_NAMES[self.kind(id) as usize]
    }
    #[inline]
    fn slot(&self, id: u32, index: usize) -> u32 {
        self.slots[self.nodes[id as usize].a as usize + index]
    }
    #[inline]
    fn items(&self, id: u32, index: usize) -> &[u32] {
        let first = self.nodes[id as usize].a as usize + index;
        let (start, len) = (self.slots[first] as usize, self.slots[first + 1] as usize);
        &self.slots[start..start + len]
    }
    /// list field の要素（`slots` の開始位置と個数）。
    #[inline]
    fn list_at(&self, start: u32, len: u32) -> &[u32] {
        &self.slots[start as usize..start as usize + len as usize]
    }
    #[inline]
    fn text_in<'s>(&self, source: &'s str, id: u32) -> &'s str {
        let node = &self.nodes[id as usize];
        &source[node.a as usize..node.b as usize]
    }
    fn text_value_span(&self, id: u32, path: &str, out: &mut Vec<ValueSpan>) {
        out.push(ValueSpan {
            path: path.to_owned(),
            span: span_of(&self.nodes[id as usize]),
            text: None,
        });
    }
}
/// 木の 1 節点（Text / Null / 型付き節点）。
#[derive(Clone, Copy)]
pub struct NodeRef<'t> {
    tree: &'t AstTree,
    id: u32,
}
impl PartialEq for NodeRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.tree, other.tree) && self.id == other.id
    }
}
impl Eq for NodeRef<'_> {}
impl std::fmt::Debug for NodeRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeRef")
            .field("id", &self.id)
            .field("type", &self.type_name())
            .finish()
    }
}
impl<'t> NodeRef<'t> {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn tree(&self) -> &'t AstTree {
        self.tree
    }
    /// 型付き節点の範囲（code point）。Text / Null は `None`（`Ast::span` と同じ）。
    pub fn span(&self) -> Option<Span> {
        let node = &self.tree.nodes[self.id as usize];
        (node.kind > KIND_NULL).then(|| span_of(node))
    }
    pub fn type_name(&self) -> &'static str {
        self.tree.type_name_of(self.id)
    }
    pub fn type_id(&self) -> &'static str {
        self.tree.type_id_of(self.id)
    }
    /// Text 節点の値（入力の部分文字列、複製しない）。
    pub fn text(&self) -> Option<&'t str> {
        (self.tree.kind(self.id) == KIND_TEXT)
            .then(|| self.tree.text_in(&self.tree.source, self.id))
    }
    pub fn is_null(&self) -> bool {
        self.tree.kind(self.id) == KIND_NULL
    }
    pub fn to_ast(&self) -> super::Ast {
        self.tree.project(&self.tree.source, self.id)
    }
}
/// list field（要素は節点）。
#[derive(Clone, Copy)]
pub struct NodeList<'t> {
    tree: &'t AstTree,
    ids: &'t [u32],
}
impl std::fmt::Debug for NodeList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<'t> NodeList<'t> {
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<NodeRef<'t>> {
        let tree = self.tree;
        self.ids.get(index).map(|&id| NodeRef { tree, id })
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = NodeRef<'t>> + 't {
        let tree = self.tree;
        self.ids.iter().map(move |&id| NodeRef { tree, id })
    }
}
/// list field（要素は文字列。入力の部分文字列を返す）。
#[derive(Clone, Copy)]
pub struct TextList<'t> {
    tree: &'t AstTree,
    ids: &'t [u32],
}
impl std::fmt::Debug for TextList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<'t> TextList<'t> {
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    pub fn get(&self, index: usize) -> Option<&'t str> {
        let tree = self.tree;
        self.ids
            .get(index)
            .map(|&id| tree.text_in(&tree.source, id))
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &'t str> + 't {
        let tree = self.tree;
        self.ids
            .iter()
            .map(move |&id| tree.text_in(&tree.source, id))
    }
}
pub const K_g_UBNFAST_2e_UBNFFile: u16 = 2;
pub const K_g_UBNFAST_2e_ImportDecl: u16 = 3;
pub const K_g_UBNFAST_2e_GrammarDecl: u16 = 4;
pub const K_g_UBNFAST_2e_GlobalSetting: u16 = 5;
pub const K_g_UBNFAST_2e_StringSettingValue: u16 = 6;
pub const K_g_UBNFAST_2e_BlockSettingValue: u16 = 7;
pub const K_g_UBNFAST_2e_KeyValuePair: u16 = 8;
pub const K_g_UBNFAST_2e_TokenDecl: u16 = 9;
pub const K_g_UBNFAST_2e_UntilExpression: u16 = 10;
pub const K_g_UBNFAST_2e_NegationExpression: u16 = 11;
pub const K_g_UBNFAST_2e_LookaheadExpression: u16 = 12;
pub const K_g_UBNFAST_2e_NegativeLookaheadExpression: u16 = 13;
pub const K_g_UBNFAST_2e_CharRangeExpression: u16 = 14;
pub const K_g_UBNFAST_2e_CIExpression: u16 = 15;
pub const K_g_UBNFAST_2e_RegexExpression: u16 = 16;
pub const K_g_UBNFAST_2e_AnyExpression: u16 = 17;
pub const K_g_UBNFAST_2e_EofExpression: u16 = 18;
pub const K_g_UBNFAST_2e_EmptyExpression: u16 = 19;
pub const K_g_UBNFAST_2e_RuleDecl: u16 = 20;
pub const K_g_UBNFAST_2e_RootAnnotation: u16 = 21;
pub const K_g_UBNFAST_2e_MappingAnnotation: u16 = 22;
pub const K_g_UBNFAST_2e_EvalAnnotation: u16 = 23;
pub const K_g_UBNFAST_2e_WhitespaceAnnotation: u16 = 24;
pub const K_g_UBNFAST_2e_InterleaveAnnotation: u16 = 25;
pub const K_g_UBNFAST_2e_BackrefAnnotation: u16 = 26;
pub const K_g_UBNFAST_2e_ScopeTreeAnnotation: u16 = 27;
pub const K_g_UBNFAST_2e_DeclaresAnnotation: u16 = 28;
pub const K_g_UBNFAST_2e_CatalogAnnotation: u16 = 29;
pub const K_g_UBNFAST_2e_LeftAssocAnnotation: u16 = 30;
pub const K_g_UBNFAST_2e_RightAssocAnnotation: u16 = 31;
pub const K_g_UBNFAST_2e_LongestChoiceAnnotation: u16 = 32;
pub const K_g_UBNFAST_2e_PredictiveChoiceAnnotation: u16 = 33;
pub const K_g_UBNFAST_2e_PrecedenceAnnotation: u16 = 34;
pub const K_g_UBNFAST_2e_DocAnnotation: u16 = 35;
pub const K_g_UBNFAST_2e_RecoveryAnnotation: u16 = 36;
pub const K_g_UBNFAST_2e_SkipAnnotation: u16 = 37;
pub const K_g_UBNFAST_2e_CommonFieldAnnotation: u16 = 38;
pub const K_g_UBNFAST_2e_EnumAnnotation: u16 = 39;
pub const K_g_UBNFAST_2e_SimpleAnnotation: u16 = 40;
pub const K_g_UBNFAST_2e_ChoiceBody: u16 = 41;
pub const K_g_UBNFAST_2e_SequenceBody: u16 = 42;
pub const K_g_UBNFAST_2e_AnnotatedElement: u16 = 43;
pub const K_g_UBNFAST_2e_GroupElement: u16 = 44;
pub const K_g_UBNFAST_2e_OptionalElement: u16 = 45;
pub const K_g_UBNFAST_2e_RepeatElement: u16 = 46;
pub const K_g_UBNFAST_2e_ErrorElement: u16 = 47;
pub const K_g_UBNFAST_2e_TerminalElement: u16 = 48;
pub const K_g_UBNFAST_2e_TypeofElement: u16 = 49;
pub const K_g_UBNFAST_2e_DottedIdentifier: u16 = 50;
pub(crate) const TYPE_IDS: &[&str] = &["text","null","UBNFAST.UBNFFile","UBNFAST.ImportDecl","UBNFAST.GrammarDecl","UBNFAST.GlobalSetting","UBNFAST.StringSettingValue","UBNFAST.BlockSettingValue","UBNFAST.KeyValuePair","UBNFAST.TokenDecl","UBNFAST.UntilExpression","UBNFAST.NegationExpression","UBNFAST.LookaheadExpression","UBNFAST.NegativeLookaheadExpression","UBNFAST.CharRangeExpression","UBNFAST.CIExpression","UBNFAST.RegexExpression","UBNFAST.AnyExpression","UBNFAST.EofExpression","UBNFAST.EmptyExpression","UBNFAST.RuleDecl","UBNFAST.RootAnnotation","UBNFAST.MappingAnnotation","UBNFAST.EvalAnnotation","UBNFAST.WhitespaceAnnotation","UBNFAST.InterleaveAnnotation","UBNFAST.BackrefAnnotation","UBNFAST.ScopeTreeAnnotation","UBNFAST.DeclaresAnnotation","UBNFAST.CatalogAnnotation","UBNFAST.LeftAssocAnnotation","UBNFAST.RightAssocAnnotation","UBNFAST.LongestChoiceAnnotation","UBNFAST.PredictiveChoiceAnnotation","UBNFAST.PrecedenceAnnotation","UBNFAST.DocAnnotation","UBNFAST.RecoveryAnnotation","UBNFAST.SkipAnnotation","UBNFAST.CommonFieldAnnotation","UBNFAST.EnumAnnotation","UBNFAST.SimpleAnnotation","UBNFAST.ChoiceBody","UBNFAST.SequenceBody","UBNFAST.AnnotatedElement","UBNFAST.GroupElement","UBNFAST.OptionalElement","UBNFAST.RepeatElement","UBNFAST.ErrorElement","UBNFAST.TerminalElement","UBNFAST.TypeofElement","UBNFAST.DottedIdentifier"];
pub(crate) const TYPE_NAMES: &[&str] = &["text","null","UBNFFile","ImportDecl","GrammarDecl","GlobalSetting","StringSettingValue","BlockSettingValue","KeyValuePair","TokenDecl","UntilExpression","NegationExpression","LookaheadExpression","NegativeLookaheadExpression","CharRangeExpression","CIExpression","RegexExpression","AnyExpression","EofExpression","EmptyExpression","RuleDecl","RootAnnotation","MappingAnnotation","EvalAnnotation","WhitespaceAnnotation","InterleaveAnnotation","BackrefAnnotation","ScopeTreeAnnotation","DeclaresAnnotation","CatalogAnnotation","LeftAssocAnnotation","RightAssocAnnotation","LongestChoiceAnnotation","PredictiveChoiceAnnotation","PrecedenceAnnotation","DocAnnotation","RecoveryAnnotation","SkipAnnotation","CommonFieldAnnotation","EnumAnnotation","SimpleAnnotation","ChoiceBody","SequenceBody","AnnotatedElement","GroupElement","OptionalElement","RepeatElement","ErrorElement","TerminalElement","TypeofElement","DottedIdentifier"];
pub(crate) const RULE_IDS: &[&str] = &["UBNF::UBNFFile", "UBNF::ImportDecl", "UBNF::GrammarDecl", "UBNF::GlobalSetting", "UBNF::SettingValue", "UBNF::StringSettingValue", "UBNF::BlockSettingValue", "UBNF::KeyValuePair", "UBNF::TokenDecl", "UBNF::TokenValue", "UBNF::UntilExpression", "UBNF::NegationExpression", "UBNF::LookaheadExpression", "UBNF::NegativeLookaheadExpression", "UBNF::CharRangeExpression", "UBNF::CIExpression", "UBNF::RegexExpression", "UBNF::AnyExpression", "UBNF::EofExpression", "UBNF::EmptyExpression", "UBNF::RuleDecl", "UBNF::Annotation", "UBNF::RootAnnotation", "UBNF::MappingAnnotation", "UBNF::EvalAnnotation", "UBNF::WhitespaceAnnotation", "UBNF::InterleaveAnnotation", "UBNF::BackrefAnnotation", "UBNF::ScopeTreeAnnotation", "UBNF::DeclaresAnnotation", "UBNF::CatalogAnnotation", "UBNF::LeftAssocAnnotation", "UBNF::RightAssocAnnotation", "UBNF::LongestChoiceAnnotation", "UBNF::PredictiveChoiceAnnotation", "UBNF::PrecedenceAnnotation", "UBNF::DocAnnotation", "UBNF::RecoveryAnnotation", "UBNF::SkipAnnotation", "UBNF::CommonFieldAnnotation", "UBNF::EnumAnnotation", "UBNF::SimpleAnnotation", "UBNF::ChoiceBody", "UBNF::SequenceBody", "UBNF::AnnotatedElement", "UBNF::PostfixQuantifier", "UBNF::SeparatedBy", "UBNF::BoundedQuantifier", "UBNF::Digits", "UBNF::AtomicElement", "UBNF::GroupElement", "UBNF::OptionalElement", "UBNF::RepeatElement", "UBNF::ErrorElement", "UBNF::TerminalElement", "UBNF::QuantifiedRef", "UBNF::TypeofElement", "UBNF::DottedIdentifier", "UBNF::RuleBody"];
/// `super::Ast` と同じ形の、木への参照。
#[derive(Clone, Copy, Debug)]
pub enum Node<'t> { Text(&'t str), Null,
g_UBNFAST_2e_UBNFFile(g_UBNFAST_2e_UBNFFile<'t>),
g_UBNFAST_2e_ImportDecl(g_UBNFAST_2e_ImportDecl<'t>),
g_UBNFAST_2e_GrammarDecl(g_UBNFAST_2e_GrammarDecl<'t>),
g_UBNFAST_2e_GlobalSetting(g_UBNFAST_2e_GlobalSetting<'t>),
g_UBNFAST_2e_StringSettingValue(g_UBNFAST_2e_StringSettingValue<'t>),
g_UBNFAST_2e_BlockSettingValue(g_UBNFAST_2e_BlockSettingValue<'t>),
g_UBNFAST_2e_KeyValuePair(g_UBNFAST_2e_KeyValuePair<'t>),
g_UBNFAST_2e_TokenDecl(g_UBNFAST_2e_TokenDecl<'t>),
g_UBNFAST_2e_UntilExpression(g_UBNFAST_2e_UntilExpression<'t>),
g_UBNFAST_2e_NegationExpression(g_UBNFAST_2e_NegationExpression<'t>),
g_UBNFAST_2e_LookaheadExpression(g_UBNFAST_2e_LookaheadExpression<'t>),
g_UBNFAST_2e_NegativeLookaheadExpression(g_UBNFAST_2e_NegativeLookaheadExpression<'t>),
g_UBNFAST_2e_CharRangeExpression(g_UBNFAST_2e_CharRangeExpression<'t>),
g_UBNFAST_2e_CIExpression(g_UBNFAST_2e_CIExpression<'t>),
g_UBNFAST_2e_RegexExpression(g_UBNFAST_2e_RegexExpression<'t>),
g_UBNFAST_2e_AnyExpression(g_UBNFAST_2e_AnyExpression<'t>),
g_UBNFAST_2e_EofExpression(g_UBNFAST_2e_EofExpression<'t>),
g_UBNFAST_2e_EmptyExpression(g_UBNFAST_2e_EmptyExpression<'t>),
g_UBNFAST_2e_RuleDecl(g_UBNFAST_2e_RuleDecl<'t>),
g_UBNFAST_2e_RootAnnotation(g_UBNFAST_2e_RootAnnotation<'t>),
g_UBNFAST_2e_MappingAnnotation(g_UBNFAST_2e_MappingAnnotation<'t>),
g_UBNFAST_2e_EvalAnnotation(g_UBNFAST_2e_EvalAnnotation<'t>),
g_UBNFAST_2e_WhitespaceAnnotation(g_UBNFAST_2e_WhitespaceAnnotation<'t>),
g_UBNFAST_2e_InterleaveAnnotation(g_UBNFAST_2e_InterleaveAnnotation<'t>),
g_UBNFAST_2e_BackrefAnnotation(g_UBNFAST_2e_BackrefAnnotation<'t>),
g_UBNFAST_2e_ScopeTreeAnnotation(g_UBNFAST_2e_ScopeTreeAnnotation<'t>),
g_UBNFAST_2e_DeclaresAnnotation(g_UBNFAST_2e_DeclaresAnnotation<'t>),
g_UBNFAST_2e_CatalogAnnotation(g_UBNFAST_2e_CatalogAnnotation<'t>),
g_UBNFAST_2e_LeftAssocAnnotation(g_UBNFAST_2e_LeftAssocAnnotation<'t>),
g_UBNFAST_2e_RightAssocAnnotation(g_UBNFAST_2e_RightAssocAnnotation<'t>),
g_UBNFAST_2e_LongestChoiceAnnotation(g_UBNFAST_2e_LongestChoiceAnnotation<'t>),
g_UBNFAST_2e_PredictiveChoiceAnnotation(g_UBNFAST_2e_PredictiveChoiceAnnotation<'t>),
g_UBNFAST_2e_PrecedenceAnnotation(g_UBNFAST_2e_PrecedenceAnnotation<'t>),
g_UBNFAST_2e_DocAnnotation(g_UBNFAST_2e_DocAnnotation<'t>),
g_UBNFAST_2e_RecoveryAnnotation(g_UBNFAST_2e_RecoveryAnnotation<'t>),
g_UBNFAST_2e_SkipAnnotation(g_UBNFAST_2e_SkipAnnotation<'t>),
g_UBNFAST_2e_CommonFieldAnnotation(g_UBNFAST_2e_CommonFieldAnnotation<'t>),
g_UBNFAST_2e_EnumAnnotation(g_UBNFAST_2e_EnumAnnotation<'t>),
g_UBNFAST_2e_SimpleAnnotation(g_UBNFAST_2e_SimpleAnnotation<'t>),
g_UBNFAST_2e_ChoiceBody(g_UBNFAST_2e_ChoiceBody<'t>),
g_UBNFAST_2e_SequenceBody(g_UBNFAST_2e_SequenceBody<'t>),
g_UBNFAST_2e_AnnotatedElement(g_UBNFAST_2e_AnnotatedElement<'t>),
g_UBNFAST_2e_GroupElement(g_UBNFAST_2e_GroupElement<'t>),
g_UBNFAST_2e_OptionalElement(g_UBNFAST_2e_OptionalElement<'t>),
g_UBNFAST_2e_RepeatElement(g_UBNFAST_2e_RepeatElement<'t>),
g_UBNFAST_2e_ErrorElement(g_UBNFAST_2e_ErrorElement<'t>),
g_UBNFAST_2e_TerminalElement(g_UBNFAST_2e_TerminalElement<'t>),
g_UBNFAST_2e_TypeofElement(g_UBNFAST_2e_TypeofElement<'t>),
g_UBNFAST_2e_DottedIdentifier(g_UBNFAST_2e_DottedIdentifier<'t>),
}
impl<'t> NodeRef<'t> {
/// 型ごとの参照へ分ける。
pub fn get(&self) -> Node<'t> { let (tree, id) = (self.tree, self.id); match tree.kind(id) { KIND_TEXT => Node::Text(tree.text_in(&tree.source, id)), KIND_NULL => Node::Null,
K_g_UBNFAST_2e_UBNFFile => Node::g_UBNFAST_2e_UBNFFile(g_UBNFAST_2e_UBNFFile { tree, id }),
K_g_UBNFAST_2e_ImportDecl => Node::g_UBNFAST_2e_ImportDecl(g_UBNFAST_2e_ImportDecl { tree, id }),
K_g_UBNFAST_2e_GrammarDecl => Node::g_UBNFAST_2e_GrammarDecl(g_UBNFAST_2e_GrammarDecl { tree, id }),
K_g_UBNFAST_2e_GlobalSetting => Node::g_UBNFAST_2e_GlobalSetting(g_UBNFAST_2e_GlobalSetting { tree, id }),
K_g_UBNFAST_2e_StringSettingValue => Node::g_UBNFAST_2e_StringSettingValue(g_UBNFAST_2e_StringSettingValue { tree, id }),
K_g_UBNFAST_2e_BlockSettingValue => Node::g_UBNFAST_2e_BlockSettingValue(g_UBNFAST_2e_BlockSettingValue { tree, id }),
K_g_UBNFAST_2e_KeyValuePair => Node::g_UBNFAST_2e_KeyValuePair(g_UBNFAST_2e_KeyValuePair { tree, id }),
K_g_UBNFAST_2e_TokenDecl => Node::g_UBNFAST_2e_TokenDecl(g_UBNFAST_2e_TokenDecl { tree, id }),
K_g_UBNFAST_2e_UntilExpression => Node::g_UBNFAST_2e_UntilExpression(g_UBNFAST_2e_UntilExpression { tree, id }),
K_g_UBNFAST_2e_NegationExpression => Node::g_UBNFAST_2e_NegationExpression(g_UBNFAST_2e_NegationExpression { tree, id }),
K_g_UBNFAST_2e_LookaheadExpression => Node::g_UBNFAST_2e_LookaheadExpression(g_UBNFAST_2e_LookaheadExpression { tree, id }),
K_g_UBNFAST_2e_NegativeLookaheadExpression => Node::g_UBNFAST_2e_NegativeLookaheadExpression(g_UBNFAST_2e_NegativeLookaheadExpression { tree, id }),
K_g_UBNFAST_2e_CharRangeExpression => Node::g_UBNFAST_2e_CharRangeExpression(g_UBNFAST_2e_CharRangeExpression { tree, id }),
K_g_UBNFAST_2e_CIExpression => Node::g_UBNFAST_2e_CIExpression(g_UBNFAST_2e_CIExpression { tree, id }),
K_g_UBNFAST_2e_RegexExpression => Node::g_UBNFAST_2e_RegexExpression(g_UBNFAST_2e_RegexExpression { tree, id }),
K_g_UBNFAST_2e_AnyExpression => Node::g_UBNFAST_2e_AnyExpression(g_UBNFAST_2e_AnyExpression { tree, id }),
K_g_UBNFAST_2e_EofExpression => Node::g_UBNFAST_2e_EofExpression(g_UBNFAST_2e_EofExpression { tree, id }),
K_g_UBNFAST_2e_EmptyExpression => Node::g_UBNFAST_2e_EmptyExpression(g_UBNFAST_2e_EmptyExpression { tree, id }),
K_g_UBNFAST_2e_RuleDecl => Node::g_UBNFAST_2e_RuleDecl(g_UBNFAST_2e_RuleDecl { tree, id }),
K_g_UBNFAST_2e_RootAnnotation => Node::g_UBNFAST_2e_RootAnnotation(g_UBNFAST_2e_RootAnnotation { tree, id }),
K_g_UBNFAST_2e_MappingAnnotation => Node::g_UBNFAST_2e_MappingAnnotation(g_UBNFAST_2e_MappingAnnotation { tree, id }),
K_g_UBNFAST_2e_EvalAnnotation => Node::g_UBNFAST_2e_EvalAnnotation(g_UBNFAST_2e_EvalAnnotation { tree, id }),
K_g_UBNFAST_2e_WhitespaceAnnotation => Node::g_UBNFAST_2e_WhitespaceAnnotation(g_UBNFAST_2e_WhitespaceAnnotation { tree, id }),
K_g_UBNFAST_2e_InterleaveAnnotation => Node::g_UBNFAST_2e_InterleaveAnnotation(g_UBNFAST_2e_InterleaveAnnotation { tree, id }),
K_g_UBNFAST_2e_BackrefAnnotation => Node::g_UBNFAST_2e_BackrefAnnotation(g_UBNFAST_2e_BackrefAnnotation { tree, id }),
K_g_UBNFAST_2e_ScopeTreeAnnotation => Node::g_UBNFAST_2e_ScopeTreeAnnotation(g_UBNFAST_2e_ScopeTreeAnnotation { tree, id }),
K_g_UBNFAST_2e_DeclaresAnnotation => Node::g_UBNFAST_2e_DeclaresAnnotation(g_UBNFAST_2e_DeclaresAnnotation { tree, id }),
K_g_UBNFAST_2e_CatalogAnnotation => Node::g_UBNFAST_2e_CatalogAnnotation(g_UBNFAST_2e_CatalogAnnotation { tree, id }),
K_g_UBNFAST_2e_LeftAssocAnnotation => Node::g_UBNFAST_2e_LeftAssocAnnotation(g_UBNFAST_2e_LeftAssocAnnotation { tree, id }),
K_g_UBNFAST_2e_RightAssocAnnotation => Node::g_UBNFAST_2e_RightAssocAnnotation(g_UBNFAST_2e_RightAssocAnnotation { tree, id }),
K_g_UBNFAST_2e_LongestChoiceAnnotation => Node::g_UBNFAST_2e_LongestChoiceAnnotation(g_UBNFAST_2e_LongestChoiceAnnotation { tree, id }),
K_g_UBNFAST_2e_PredictiveChoiceAnnotation => Node::g_UBNFAST_2e_PredictiveChoiceAnnotation(g_UBNFAST_2e_PredictiveChoiceAnnotation { tree, id }),
K_g_UBNFAST_2e_PrecedenceAnnotation => Node::g_UBNFAST_2e_PrecedenceAnnotation(g_UBNFAST_2e_PrecedenceAnnotation { tree, id }),
K_g_UBNFAST_2e_DocAnnotation => Node::g_UBNFAST_2e_DocAnnotation(g_UBNFAST_2e_DocAnnotation { tree, id }),
K_g_UBNFAST_2e_RecoveryAnnotation => Node::g_UBNFAST_2e_RecoveryAnnotation(g_UBNFAST_2e_RecoveryAnnotation { tree, id }),
K_g_UBNFAST_2e_SkipAnnotation => Node::g_UBNFAST_2e_SkipAnnotation(g_UBNFAST_2e_SkipAnnotation { tree, id }),
K_g_UBNFAST_2e_CommonFieldAnnotation => Node::g_UBNFAST_2e_CommonFieldAnnotation(g_UBNFAST_2e_CommonFieldAnnotation { tree, id }),
K_g_UBNFAST_2e_EnumAnnotation => Node::g_UBNFAST_2e_EnumAnnotation(g_UBNFAST_2e_EnumAnnotation { tree, id }),
K_g_UBNFAST_2e_SimpleAnnotation => Node::g_UBNFAST_2e_SimpleAnnotation(g_UBNFAST_2e_SimpleAnnotation { tree, id }),
K_g_UBNFAST_2e_ChoiceBody => Node::g_UBNFAST_2e_ChoiceBody(g_UBNFAST_2e_ChoiceBody { tree, id }),
K_g_UBNFAST_2e_SequenceBody => Node::g_UBNFAST_2e_SequenceBody(g_UBNFAST_2e_SequenceBody { tree, id }),
K_g_UBNFAST_2e_AnnotatedElement => Node::g_UBNFAST_2e_AnnotatedElement(g_UBNFAST_2e_AnnotatedElement { tree, id }),
K_g_UBNFAST_2e_GroupElement => Node::g_UBNFAST_2e_GroupElement(g_UBNFAST_2e_GroupElement { tree, id }),
K_g_UBNFAST_2e_OptionalElement => Node::g_UBNFAST_2e_OptionalElement(g_UBNFAST_2e_OptionalElement { tree, id }),
K_g_UBNFAST_2e_RepeatElement => Node::g_UBNFAST_2e_RepeatElement(g_UBNFAST_2e_RepeatElement { tree, id }),
K_g_UBNFAST_2e_ErrorElement => Node::g_UBNFAST_2e_ErrorElement(g_UBNFAST_2e_ErrorElement { tree, id }),
K_g_UBNFAST_2e_TerminalElement => Node::g_UBNFAST_2e_TerminalElement(g_UBNFAST_2e_TerminalElement { tree, id }),
K_g_UBNFAST_2e_TypeofElement => Node::g_UBNFAST_2e_TypeofElement(g_UBNFAST_2e_TypeofElement { tree, id }),
K_g_UBNFAST_2e_DottedIdentifier => Node::g_UBNFAST_2e_DottedIdentifier(g_UBNFAST_2e_DottedIdentifier { tree, id }),
_ => unreachable!("unknown AST node kind"), } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_UBNFFile<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_UBNFFile<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_UBNFFile").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_UBNFFile<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_UBNFFile { self.tree.project_g_UBNFAST_2e_UBNFFile(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_grammars(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_ImportDecl<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_ImportDecl<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_ImportDecl").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_ImportDecl<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_ImportDecl { self.tree.project_g_UBNFAST_2e_ImportDecl(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_alias(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_path(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 1)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_GrammarDecl<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_GrammarDecl<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_GrammarDecl").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_GrammarDecl<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_GrammarDecl { self.tree.project_g_UBNFAST_2e_GrammarDecl(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_name(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_imports(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 1) } }
pub fn g_settings(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 3) } }
pub fn g_tokens(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 5) } }
pub fn g_rules(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 7) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_GlobalSetting<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_GlobalSetting<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_GlobalSetting").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_GlobalSetting<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_GlobalSetting { self.tree.project_g_UBNFAST_2e_GlobalSetting(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_key(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_value(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 1) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_StringSettingValue<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_StringSettingValue<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_StringSettingValue").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_StringSettingValue<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_StringSettingValue { self.tree.project_g_UBNFAST_2e_StringSettingValue(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_value(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_BlockSettingValue<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_BlockSettingValue<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_BlockSettingValue").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_BlockSettingValue<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_BlockSettingValue { self.tree.project_g_UBNFAST_2e_BlockSettingValue(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_entries(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_KeyValuePair<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_KeyValuePair<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_KeyValuePair").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_KeyValuePair<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_KeyValuePair { self.tree.project_g_UBNFAST_2e_KeyValuePair(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_key(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_value(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 1)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_TokenDecl<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_TokenDecl<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_TokenDecl").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_TokenDecl<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_TokenDecl { self.tree.project_g_UBNFAST_2e_TokenDecl(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_name(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_value(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 1) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_UntilExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_UntilExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_UntilExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_UntilExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_UntilExpression { self.tree.project_g_UBNFAST_2e_UntilExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_terminator(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_NegationExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_NegationExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_NegationExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_NegationExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_NegationExpression { self.tree.project_g_UBNFAST_2e_NegationExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_chars(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_LookaheadExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_LookaheadExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_LookaheadExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_LookaheadExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_LookaheadExpression { self.tree.project_g_UBNFAST_2e_LookaheadExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_pattern(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_NegativeLookaheadExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_NegativeLookaheadExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_NegativeLookaheadExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_NegativeLookaheadExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_NegativeLookaheadExpression { self.tree.project_g_UBNFAST_2e_NegativeLookaheadExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_pattern(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_CharRangeExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_CharRangeExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_CharRangeExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_CharRangeExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_CharRangeExpression { self.tree.project_g_UBNFAST_2e_CharRangeExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_min(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_max(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 1)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_CIExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_CIExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_CIExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_CIExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_CIExpression { self.tree.project_g_UBNFAST_2e_CIExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_word(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_RegexExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_RegexExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_RegexExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_RegexExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_RegexExpression { self.tree.project_g_UBNFAST_2e_RegexExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_pattern(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_AnyExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_AnyExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_AnyExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_AnyExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_AnyExpression { self.tree.project_g_UBNFAST_2e_AnyExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_EofExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_EofExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_EofExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_EofExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_EofExpression { self.tree.project_g_UBNFAST_2e_EofExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_EmptyExpression<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_EmptyExpression<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_EmptyExpression").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_EmptyExpression<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_EmptyExpression { self.tree.project_g_UBNFAST_2e_EmptyExpression(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_RuleDecl<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_RuleDecl<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_RuleDecl").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_RuleDecl<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_RuleDecl { self.tree.project_g_UBNFAST_2e_RuleDecl(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_annotations(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
pub fn g_name(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 2)) }
pub fn g_body(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 3) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_RootAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_RootAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_RootAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_RootAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_RootAnnotation { self.tree.project_g_UBNFAST_2e_RootAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_MappingAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_MappingAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_MappingAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_MappingAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_MappingAnnotation { self.tree.project_g_UBNFAST_2e_MappingAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_className(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_paramNames(&self) -> TextList<'t> { TextList { tree: self.tree, ids: self.tree.items(self.id, 1) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_EvalAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_EvalAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_EvalAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_EvalAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_EvalAnnotation { self.tree.project_g_UBNFAST_2e_EvalAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_entries(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_WhitespaceAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_WhitespaceAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_WhitespaceAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_WhitespaceAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_WhitespaceAnnotation { self.tree.project_g_UBNFAST_2e_WhitespaceAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_style(&self) -> Option<&'t str> { let tree = self.tree; let v = tree.slot(self.id, 0); (v != NONE).then(|| tree.text_in(&tree.source, v)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_InterleaveAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_InterleaveAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_InterleaveAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_InterleaveAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_InterleaveAnnotation { self.tree.project_g_UBNFAST_2e_InterleaveAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_profile(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_BackrefAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_BackrefAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_BackrefAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_BackrefAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_BackrefAnnotation { self.tree.project_g_UBNFAST_2e_BackrefAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_name(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_ScopeTreeAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_ScopeTreeAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_ScopeTreeAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_ScopeTreeAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_ScopeTreeAnnotation { self.tree.project_g_UBNFAST_2e_ScopeTreeAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_mode(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_DeclaresAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_DeclaresAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_DeclaresAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_DeclaresAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_DeclaresAnnotation { self.tree.project_g_UBNFAST_2e_DeclaresAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_symbol(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
pub fn g_description(&self) -> Option<&'t str> { let tree = self.tree; let v = tree.slot(self.id, 1); (v != NONE).then(|| tree.text_in(&tree.source, v)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_CatalogAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_CatalogAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_CatalogAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_CatalogAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_CatalogAnnotation { self.tree.project_g_UBNFAST_2e_CatalogAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_context(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_LeftAssocAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_LeftAssocAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_LeftAssocAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_LeftAssocAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_LeftAssocAnnotation { self.tree.project_g_UBNFAST_2e_LeftAssocAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_RightAssocAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_RightAssocAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_RightAssocAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_RightAssocAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_RightAssocAnnotation { self.tree.project_g_UBNFAST_2e_RightAssocAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_LongestChoiceAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_LongestChoiceAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_LongestChoiceAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_LongestChoiceAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_LongestChoiceAnnotation { self.tree.project_g_UBNFAST_2e_LongestChoiceAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_PredictiveChoiceAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_PredictiveChoiceAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_PredictiveChoiceAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_PredictiveChoiceAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_PredictiveChoiceAnnotation { self.tree.project_g_UBNFAST_2e_PredictiveChoiceAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_PrecedenceAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_PrecedenceAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_PrecedenceAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_PrecedenceAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_PrecedenceAnnotation { self.tree.project_g_UBNFAST_2e_PrecedenceAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_level(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_DocAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_DocAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_DocAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_DocAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_DocAnnotation { self.tree.project_g_UBNFAST_2e_DocAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_text(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_RecoveryAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_RecoveryAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_RecoveryAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_RecoveryAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_RecoveryAnnotation { self.tree.project_g_UBNFAST_2e_RecoveryAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_mode(&self) -> Option<&'t str> { let tree = self.tree; let v = tree.slot(self.id, 0); (v != NONE).then(|| tree.text_in(&tree.source, v)) }
pub fn g_syncTokens(&self) -> Option<&'t str> { let tree = self.tree; let v = tree.slot(self.id, 1); (v != NONE).then(|| tree.text_in(&tree.source, v)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_SkipAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_SkipAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_SkipAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_SkipAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_SkipAnnotation { self.tree.project_g_UBNFAST_2e_SkipAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_CommonFieldAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_CommonFieldAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_CommonFieldAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_CommonFieldAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_CommonFieldAnnotation { self.tree.project_g_UBNFAST_2e_CommonFieldAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_fieldNames(&self) -> TextList<'t> { TextList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_EnumAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_EnumAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_EnumAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_EnumAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_EnumAnnotation { self.tree.project_g_UBNFAST_2e_EnumAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_SimpleAnnotation<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_SimpleAnnotation<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_SimpleAnnotation").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_SimpleAnnotation<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_SimpleAnnotation { self.tree.project_g_UBNFAST_2e_SimpleAnnotation(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_name(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_ChoiceBody<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_ChoiceBody<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_ChoiceBody").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_ChoiceBody<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_ChoiceBody { self.tree.project_g_UBNFAST_2e_ChoiceBody(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_alternatives(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_SequenceBody<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_SequenceBody<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_SequenceBody").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_SequenceBody<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_SequenceBody { self.tree.project_g_UBNFAST_2e_SequenceBody(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_elements(&self) -> NodeList<'t> { NodeList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_AnnotatedElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_AnnotatedElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_AnnotatedElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_AnnotatedElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_AnnotatedElement { self.tree.project_g_UBNFAST_2e_AnnotatedElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_element(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 0) } }
pub fn g_captureName(&self) -> Option<&'t str> { let tree = self.tree; let v = tree.slot(self.id, 1); (v != NONE).then(|| tree.text_in(&tree.source, v)) }
pub fn g_typeofConstraint(&self) -> Option<NodeRef<'t>> { let v = self.tree.slot(self.id, 2); (v != NONE).then_some(NodeRef { tree: self.tree, id: v }) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_GroupElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_GroupElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_GroupElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_GroupElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_GroupElement { self.tree.project_g_UBNFAST_2e_GroupElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_body(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_OptionalElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_OptionalElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_OptionalElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_OptionalElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_OptionalElement { self.tree.project_g_UBNFAST_2e_OptionalElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_body(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_RepeatElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_RepeatElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_RepeatElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_RepeatElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_RepeatElement { self.tree.project_g_UBNFAST_2e_RepeatElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_body(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.tree.slot(self.id, 0) } }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_ErrorElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_ErrorElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_ErrorElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_ErrorElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_ErrorElement { self.tree.project_g_UBNFAST_2e_ErrorElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_message(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_TerminalElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_TerminalElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_TerminalElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_TerminalElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_TerminalElement { self.tree.project_g_UBNFAST_2e_TerminalElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_value(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_TypeofElement<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_TypeofElement<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_TypeofElement").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_TypeofElement<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_TypeofElement { self.tree.project_g_UBNFAST_2e_TypeofElement(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_captureName(&self) -> &'t str { let tree = self.tree; tree.text_in(&tree.source, tree.slot(self.id, 0)) }
}
#[derive(Clone, Copy)] pub struct g_UBNFAST_2e_DottedIdentifier<'t> { tree: &'t AstTree, id: u32 }
impl std::fmt::Debug for g_UBNFAST_2e_DottedIdentifier<'_> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("g_UBNFAST_2e_DottedIdentifier").field("id", &self.id).finish() } }
impl<'t> g_UBNFAST_2e_DottedIdentifier<'t> {
pub fn node(&self) -> NodeRef<'t> { NodeRef { tree: self.tree, id: self.id } }
pub fn span(&self) -> Span { span_of(&self.tree.nodes[self.id as usize]) }
pub fn node_id(&self) -> usize { self.tree.nodes[self.id as usize].node_id as usize }
pub fn to_ast(&self) -> super::g_UBNFAST_2e_DottedIdentifier { self.tree.project_g_UBNFAST_2e_DottedIdentifier(&self.tree.source, &self.tree.nodes[self.id as usize]) }
pub fn g_parts(&self) -> TextList<'t> { TextList { tree: self.tree, ids: self.tree.items(self.id, 0) } }
}
impl AstTree {
/// 節点を所有 `Ast` へ写す。`source` は Text の byte 範囲を読む入力。
#[doc(hidden)] pub fn project(&self, source: &str, id: u32) -> super::Ast { let node = self.nodes[id as usize]; match node.kind { KIND_TEXT => super::Ast::Text(source[node.a as usize..node.b as usize].to_owned()), KIND_NULL => super::Ast::Null,
K_g_UBNFAST_2e_UBNFFile => super::Ast::g_UBNFAST_2e_UBNFFile(self.project_g_UBNFAST_2e_UBNFFile(source, &node)),
K_g_UBNFAST_2e_ImportDecl => super::Ast::g_UBNFAST_2e_ImportDecl(self.project_g_UBNFAST_2e_ImportDecl(source, &node)),
K_g_UBNFAST_2e_GrammarDecl => super::Ast::g_UBNFAST_2e_GrammarDecl(self.project_g_UBNFAST_2e_GrammarDecl(source, &node)),
K_g_UBNFAST_2e_GlobalSetting => super::Ast::g_UBNFAST_2e_GlobalSetting(self.project_g_UBNFAST_2e_GlobalSetting(source, &node)),
K_g_UBNFAST_2e_StringSettingValue => super::Ast::g_UBNFAST_2e_StringSettingValue(self.project_g_UBNFAST_2e_StringSettingValue(source, &node)),
K_g_UBNFAST_2e_BlockSettingValue => super::Ast::g_UBNFAST_2e_BlockSettingValue(self.project_g_UBNFAST_2e_BlockSettingValue(source, &node)),
K_g_UBNFAST_2e_KeyValuePair => super::Ast::g_UBNFAST_2e_KeyValuePair(self.project_g_UBNFAST_2e_KeyValuePair(source, &node)),
K_g_UBNFAST_2e_TokenDecl => super::Ast::g_UBNFAST_2e_TokenDecl(self.project_g_UBNFAST_2e_TokenDecl(source, &node)),
K_g_UBNFAST_2e_UntilExpression => super::Ast::g_UBNFAST_2e_UntilExpression(self.project_g_UBNFAST_2e_UntilExpression(source, &node)),
K_g_UBNFAST_2e_NegationExpression => super::Ast::g_UBNFAST_2e_NegationExpression(self.project_g_UBNFAST_2e_NegationExpression(source, &node)),
K_g_UBNFAST_2e_LookaheadExpression => super::Ast::g_UBNFAST_2e_LookaheadExpression(self.project_g_UBNFAST_2e_LookaheadExpression(source, &node)),
K_g_UBNFAST_2e_NegativeLookaheadExpression => super::Ast::g_UBNFAST_2e_NegativeLookaheadExpression(self.project_g_UBNFAST_2e_NegativeLookaheadExpression(source, &node)),
K_g_UBNFAST_2e_CharRangeExpression => super::Ast::g_UBNFAST_2e_CharRangeExpression(self.project_g_UBNFAST_2e_CharRangeExpression(source, &node)),
K_g_UBNFAST_2e_CIExpression => super::Ast::g_UBNFAST_2e_CIExpression(self.project_g_UBNFAST_2e_CIExpression(source, &node)),
K_g_UBNFAST_2e_RegexExpression => super::Ast::g_UBNFAST_2e_RegexExpression(self.project_g_UBNFAST_2e_RegexExpression(source, &node)),
K_g_UBNFAST_2e_AnyExpression => super::Ast::g_UBNFAST_2e_AnyExpression(self.project_g_UBNFAST_2e_AnyExpression(source, &node)),
K_g_UBNFAST_2e_EofExpression => super::Ast::g_UBNFAST_2e_EofExpression(self.project_g_UBNFAST_2e_EofExpression(source, &node)),
K_g_UBNFAST_2e_EmptyExpression => super::Ast::g_UBNFAST_2e_EmptyExpression(self.project_g_UBNFAST_2e_EmptyExpression(source, &node)),
K_g_UBNFAST_2e_RuleDecl => super::Ast::g_UBNFAST_2e_RuleDecl(self.project_g_UBNFAST_2e_RuleDecl(source, &node)),
K_g_UBNFAST_2e_RootAnnotation => super::Ast::g_UBNFAST_2e_RootAnnotation(self.project_g_UBNFAST_2e_RootAnnotation(source, &node)),
K_g_UBNFAST_2e_MappingAnnotation => super::Ast::g_UBNFAST_2e_MappingAnnotation(self.project_g_UBNFAST_2e_MappingAnnotation(source, &node)),
K_g_UBNFAST_2e_EvalAnnotation => super::Ast::g_UBNFAST_2e_EvalAnnotation(self.project_g_UBNFAST_2e_EvalAnnotation(source, &node)),
K_g_UBNFAST_2e_WhitespaceAnnotation => super::Ast::g_UBNFAST_2e_WhitespaceAnnotation(self.project_g_UBNFAST_2e_WhitespaceAnnotation(source, &node)),
K_g_UBNFAST_2e_InterleaveAnnotation => super::Ast::g_UBNFAST_2e_InterleaveAnnotation(self.project_g_UBNFAST_2e_InterleaveAnnotation(source, &node)),
K_g_UBNFAST_2e_BackrefAnnotation => super::Ast::g_UBNFAST_2e_BackrefAnnotation(self.project_g_UBNFAST_2e_BackrefAnnotation(source, &node)),
K_g_UBNFAST_2e_ScopeTreeAnnotation => super::Ast::g_UBNFAST_2e_ScopeTreeAnnotation(self.project_g_UBNFAST_2e_ScopeTreeAnnotation(source, &node)),
K_g_UBNFAST_2e_DeclaresAnnotation => super::Ast::g_UBNFAST_2e_DeclaresAnnotation(self.project_g_UBNFAST_2e_DeclaresAnnotation(source, &node)),
K_g_UBNFAST_2e_CatalogAnnotation => super::Ast::g_UBNFAST_2e_CatalogAnnotation(self.project_g_UBNFAST_2e_CatalogAnnotation(source, &node)),
K_g_UBNFAST_2e_LeftAssocAnnotation => super::Ast::g_UBNFAST_2e_LeftAssocAnnotation(self.project_g_UBNFAST_2e_LeftAssocAnnotation(source, &node)),
K_g_UBNFAST_2e_RightAssocAnnotation => super::Ast::g_UBNFAST_2e_RightAssocAnnotation(self.project_g_UBNFAST_2e_RightAssocAnnotation(source, &node)),
K_g_UBNFAST_2e_LongestChoiceAnnotation => super::Ast::g_UBNFAST_2e_LongestChoiceAnnotation(self.project_g_UBNFAST_2e_LongestChoiceAnnotation(source, &node)),
K_g_UBNFAST_2e_PredictiveChoiceAnnotation => super::Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(self.project_g_UBNFAST_2e_PredictiveChoiceAnnotation(source, &node)),
K_g_UBNFAST_2e_PrecedenceAnnotation => super::Ast::g_UBNFAST_2e_PrecedenceAnnotation(self.project_g_UBNFAST_2e_PrecedenceAnnotation(source, &node)),
K_g_UBNFAST_2e_DocAnnotation => super::Ast::g_UBNFAST_2e_DocAnnotation(self.project_g_UBNFAST_2e_DocAnnotation(source, &node)),
K_g_UBNFAST_2e_RecoveryAnnotation => super::Ast::g_UBNFAST_2e_RecoveryAnnotation(self.project_g_UBNFAST_2e_RecoveryAnnotation(source, &node)),
K_g_UBNFAST_2e_SkipAnnotation => super::Ast::g_UBNFAST_2e_SkipAnnotation(self.project_g_UBNFAST_2e_SkipAnnotation(source, &node)),
K_g_UBNFAST_2e_CommonFieldAnnotation => super::Ast::g_UBNFAST_2e_CommonFieldAnnotation(self.project_g_UBNFAST_2e_CommonFieldAnnotation(source, &node)),
K_g_UBNFAST_2e_EnumAnnotation => super::Ast::g_UBNFAST_2e_EnumAnnotation(self.project_g_UBNFAST_2e_EnumAnnotation(source, &node)),
K_g_UBNFAST_2e_SimpleAnnotation => super::Ast::g_UBNFAST_2e_SimpleAnnotation(self.project_g_UBNFAST_2e_SimpleAnnotation(source, &node)),
K_g_UBNFAST_2e_ChoiceBody => super::Ast::g_UBNFAST_2e_ChoiceBody(self.project_g_UBNFAST_2e_ChoiceBody(source, &node)),
K_g_UBNFAST_2e_SequenceBody => super::Ast::g_UBNFAST_2e_SequenceBody(self.project_g_UBNFAST_2e_SequenceBody(source, &node)),
K_g_UBNFAST_2e_AnnotatedElement => super::Ast::g_UBNFAST_2e_AnnotatedElement(self.project_g_UBNFAST_2e_AnnotatedElement(source, &node)),
K_g_UBNFAST_2e_GroupElement => super::Ast::g_UBNFAST_2e_GroupElement(self.project_g_UBNFAST_2e_GroupElement(source, &node)),
K_g_UBNFAST_2e_OptionalElement => super::Ast::g_UBNFAST_2e_OptionalElement(self.project_g_UBNFAST_2e_OptionalElement(source, &node)),
K_g_UBNFAST_2e_RepeatElement => super::Ast::g_UBNFAST_2e_RepeatElement(self.project_g_UBNFAST_2e_RepeatElement(source, &node)),
K_g_UBNFAST_2e_ErrorElement => super::Ast::g_UBNFAST_2e_ErrorElement(self.project_g_UBNFAST_2e_ErrorElement(source, &node)),
K_g_UBNFAST_2e_TerminalElement => super::Ast::g_UBNFAST_2e_TerminalElement(self.project_g_UBNFAST_2e_TerminalElement(source, &node)),
K_g_UBNFAST_2e_TypeofElement => super::Ast::g_UBNFAST_2e_TypeofElement(self.project_g_UBNFAST_2e_TypeofElement(source, &node)),
K_g_UBNFAST_2e_DottedIdentifier => super::Ast::g_UBNFAST_2e_DottedIdentifier(self.project_g_UBNFAST_2e_DottedIdentifier(source, &node)),
_ => unreachable!("unknown AST node kind"), } }
fn project_g_UBNFAST_2e_UBNFFile(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_UBNFFile { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_UBNFFile { span: span_of(node), node_id: node.node_id as usize,
g_grammars: self.list_at(f[0], f[1]).iter().map(|&v| self.project(source, v)).collect(),
} }
fn project_g_UBNFAST_2e_ImportDecl(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_ImportDecl { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_ImportDecl { span: span_of(node), node_id: node.node_id as usize,
g_alias: self.text_in(source, f[0]).to_owned(),
g_path: self.text_in(source, f[1]).to_owned(),
} }
fn project_g_UBNFAST_2e_GrammarDecl(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_GrammarDecl { let f = &self.slots[node.a as usize..node.a as usize + 9]; super::g_UBNFAST_2e_GrammarDecl { span: span_of(node), node_id: node.node_id as usize,
g_name: self.text_in(source, f[0]).to_owned(),
g_imports: self.list_at(f[1], f[2]).iter().map(|&v| self.project(source, v)).collect(),
g_settings: self.list_at(f[3], f[4]).iter().map(|&v| self.project(source, v)).collect(),
g_tokens: self.list_at(f[5], f[6]).iter().map(|&v| self.project(source, v)).collect(),
g_rules: self.list_at(f[7], f[8]).iter().map(|&v| self.project(source, v)).collect(),
} }
fn project_g_UBNFAST_2e_GlobalSetting(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_GlobalSetting { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_GlobalSetting { span: span_of(node), node_id: node.node_id as usize,
g_key: self.text_in(source, f[0]).to_owned(),
g_value: Box::new(self.project(source, f[1])),
} }
fn project_g_UBNFAST_2e_StringSettingValue(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_StringSettingValue { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_StringSettingValue { span: span_of(node), node_id: node.node_id as usize,
g_value: Box::new(self.project(source, f[0])),
} }
fn project_g_UBNFAST_2e_BlockSettingValue(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_BlockSettingValue { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_BlockSettingValue { span: span_of(node), node_id: node.node_id as usize,
g_entries: self.list_at(f[0], f[1]).iter().map(|&v| self.project(source, v)).collect(),
} }
fn project_g_UBNFAST_2e_KeyValuePair(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_KeyValuePair { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_KeyValuePair { span: span_of(node), node_id: node.node_id as usize,
g_key: self.text_in(source, f[0]).to_owned(),
g_value: self.text_in(source, f[1]).to_owned(),
} }
fn project_g_UBNFAST_2e_TokenDecl(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_TokenDecl { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_TokenDecl { span: span_of(node), node_id: node.node_id as usize,
g_name: self.text_in(source, f[0]).to_owned(),
g_value: Box::new(self.project(source, f[1])),
} }
fn project_g_UBNFAST_2e_UntilExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_UntilExpression { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_UntilExpression { span: span_of(node), node_id: node.node_id as usize,
g_terminator: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_NegationExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_NegationExpression { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_NegationExpression { span: span_of(node), node_id: node.node_id as usize,
g_chars: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_LookaheadExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_LookaheadExpression { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_LookaheadExpression { span: span_of(node), node_id: node.node_id as usize,
g_pattern: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_NegativeLookaheadExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_NegativeLookaheadExpression { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_NegativeLookaheadExpression { span: span_of(node), node_id: node.node_id as usize,
g_pattern: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_CharRangeExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_CharRangeExpression { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_CharRangeExpression { span: span_of(node), node_id: node.node_id as usize,
g_min: self.text_in(source, f[0]).to_owned(),
g_max: self.text_in(source, f[1]).to_owned(),
} }
fn project_g_UBNFAST_2e_CIExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_CIExpression { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_CIExpression { span: span_of(node), node_id: node.node_id as usize,
g_word: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_RegexExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_RegexExpression { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_RegexExpression { span: span_of(node), node_id: node.node_id as usize,
g_pattern: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_AnyExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_AnyExpression { let _ = source; super::g_UBNFAST_2e_AnyExpression { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_EofExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_EofExpression { let _ = source; super::g_UBNFAST_2e_EofExpression { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_EmptyExpression(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_EmptyExpression { let _ = source; super::g_UBNFAST_2e_EmptyExpression { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_RuleDecl(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_RuleDecl { let f = &self.slots[node.a as usize..node.a as usize + 4]; super::g_UBNFAST_2e_RuleDecl { span: span_of(node), node_id: node.node_id as usize,
g_annotations: self.list_at(f[0], f[1]).iter().map(|&v| self.project(source, v)).collect(),
g_name: self.text_in(source, f[2]).to_owned(),
g_body: Box::new(self.project(source, f[3])),
} }
fn project_g_UBNFAST_2e_RootAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_RootAnnotation { let _ = source; super::g_UBNFAST_2e_RootAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_MappingAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_MappingAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 3]; super::g_UBNFAST_2e_MappingAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_className: self.text_in(source, f[0]).to_owned(),
g_paramNames: self.list_at(f[1], f[2]).iter().map(|&v| self.text_in(source, v).to_owned()).collect(),
} }
fn project_g_UBNFAST_2e_EvalAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_EvalAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_EvalAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_entries: self.list_at(f[0], f[1]).iter().map(|&v| self.project(source, v)).collect(),
} }
fn project_g_UBNFAST_2e_WhitespaceAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_WhitespaceAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_WhitespaceAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_style: { let v = f[0]; (v != NONE).then(|| self.text_in(source, v).to_owned()) },
} }
fn project_g_UBNFAST_2e_InterleaveAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_InterleaveAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_InterleaveAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_profile: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_BackrefAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_BackrefAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_BackrefAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_name: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_ScopeTreeAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_ScopeTreeAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_ScopeTreeAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_mode: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_DeclaresAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_DeclaresAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_DeclaresAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_symbol: self.text_in(source, f[0]).to_owned(),
g_description: { let v = f[1]; (v != NONE).then(|| self.text_in(source, v).to_owned()) },
} }
fn project_g_UBNFAST_2e_CatalogAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_CatalogAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_CatalogAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_context: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_LeftAssocAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_LeftAssocAnnotation { let _ = source; super::g_UBNFAST_2e_LeftAssocAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_RightAssocAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_RightAssocAnnotation { let _ = source; super::g_UBNFAST_2e_RightAssocAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_LongestChoiceAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_LongestChoiceAnnotation { let _ = source; super::g_UBNFAST_2e_LongestChoiceAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_PredictiveChoiceAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_PredictiveChoiceAnnotation { let _ = source; super::g_UBNFAST_2e_PredictiveChoiceAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_PrecedenceAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_PrecedenceAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_PrecedenceAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_level: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_DocAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_DocAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_DocAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_text: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_RecoveryAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_RecoveryAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_RecoveryAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_mode: { let v = f[0]; (v != NONE).then(|| self.text_in(source, v).to_owned()) },
g_syncTokens: { let v = f[1]; (v != NONE).then(|| self.text_in(source, v).to_owned()) },
} }
fn project_g_UBNFAST_2e_SkipAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_SkipAnnotation { let _ = source; super::g_UBNFAST_2e_SkipAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_CommonFieldAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_CommonFieldAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_CommonFieldAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_fieldNames: self.list_at(f[0], f[1]).iter().map(|&v| self.text_in(source, v).to_owned()).collect(),
} }
fn project_g_UBNFAST_2e_EnumAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_EnumAnnotation { let _ = source; super::g_UBNFAST_2e_EnumAnnotation { span: span_of(node), node_id: node.node_id as usize,
} }
fn project_g_UBNFAST_2e_SimpleAnnotation(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_SimpleAnnotation { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_SimpleAnnotation { span: span_of(node), node_id: node.node_id as usize,
g_name: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_ChoiceBody(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_ChoiceBody { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_ChoiceBody { span: span_of(node), node_id: node.node_id as usize,
g_alternatives: self.list_at(f[0], f[1]).iter().map(|&v| self.project(source, v)).collect(),
} }
fn project_g_UBNFAST_2e_SequenceBody(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_SequenceBody { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_SequenceBody { span: span_of(node), node_id: node.node_id as usize,
g_elements: self.list_at(f[0], f[1]).iter().map(|&v| self.project(source, v)).collect(),
} }
fn project_g_UBNFAST_2e_AnnotatedElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_AnnotatedElement { let f = &self.slots[node.a as usize..node.a as usize + 3]; super::g_UBNFAST_2e_AnnotatedElement { span: span_of(node), node_id: node.node_id as usize,
g_element: Box::new(self.project(source, f[0])),
g_captureName: { let v = f[1]; (v != NONE).then(|| self.text_in(source, v).to_owned()) },
g_typeofConstraint: { let v = f[2]; (v != NONE).then(|| Box::new(self.project(source, v))) },
} }
fn project_g_UBNFAST_2e_GroupElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_GroupElement { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_GroupElement { span: span_of(node), node_id: node.node_id as usize,
g_body: Box::new(self.project(source, f[0])),
} }
fn project_g_UBNFAST_2e_OptionalElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_OptionalElement { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_OptionalElement { span: span_of(node), node_id: node.node_id as usize,
g_body: Box::new(self.project(source, f[0])),
} }
fn project_g_UBNFAST_2e_RepeatElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_RepeatElement { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_RepeatElement { span: span_of(node), node_id: node.node_id as usize,
g_body: Box::new(self.project(source, f[0])),
} }
fn project_g_UBNFAST_2e_ErrorElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_ErrorElement { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_ErrorElement { span: span_of(node), node_id: node.node_id as usize,
g_message: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_TerminalElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_TerminalElement { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_TerminalElement { span: span_of(node), node_id: node.node_id as usize,
g_value: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_TypeofElement(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_TypeofElement { let f = &self.slots[node.a as usize..node.a as usize + 1]; super::g_UBNFAST_2e_TypeofElement { span: span_of(node), node_id: node.node_id as usize,
g_captureName: self.text_in(source, f[0]).to_owned(),
} }
fn project_g_UBNFAST_2e_DottedIdentifier(&self, source: &str, node: &TreeNode) -> super::g_UBNFAST_2e_DottedIdentifier { let f = &self.slots[node.a as usize..node.a as usize + 2]; super::g_UBNFAST_2e_DottedIdentifier { span: span_of(node), node_id: node.node_id as usize,
g_parts: self.list_at(f[0], f[1]).iter().map(|&v| self.text_in(source, v).to_owned()).collect(),
} }
}
impl AstTree {
#[doc(hidden)] pub fn collect_value_spans(&self, source: &str, id: u32, path: &mut String, out: &mut Vec<ValueSpan>) { let _ = &source; let node = self.nodes[id as usize]; match node.kind { KIND_TEXT => out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: Some(self.text_in(source, id).to_owned()) }), KIND_NULL => {},
K_g_UBNFAST_2e_UBNFFile => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/grammars"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_ImportDecl => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/alias"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/path"); self.text_value_span(self.slot(id, 1), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_GrammarDecl => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/name"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/imports"); for (i, &v) in self.items(id, 1).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/settings"); for (i, &v) in self.items(id, 3).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/tokens"); for (i, &v) in self.items(id, 5).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/rules"); for (i, &v) in self.items(id, 7).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_GlobalSetting => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/key"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/value"); self.collect_value_spans(source, self.slot(id, 1), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_StringSettingValue => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/value"); self.collect_value_spans(source, self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_BlockSettingValue => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/entries"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_KeyValuePair => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/key"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/value"); self.text_value_span(self.slot(id, 1), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_TokenDecl => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/name"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/value"); self.collect_value_spans(source, self.slot(id, 1), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_UntilExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/terminator"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_NegationExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/chars"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_LookaheadExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/pattern"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_NegativeLookaheadExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/pattern"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_CharRangeExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/min"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/max"); self.text_value_span(self.slot(id, 1), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_CIExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/word"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_RegexExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/pattern"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_AnyExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_EofExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_EmptyExpression => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_RuleDecl => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/annotations"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/name"); self.text_value_span(self.slot(id, 2), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/body"); self.collect_value_spans(source, self.slot(id, 3), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_RootAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_MappingAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/className"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/paramNames"); for (i, &v) in self.items(id, 1).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.text_value_span(v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_EvalAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/entries"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_WhitespaceAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/style"); let v = self.slot(id, 0); if v != NONE { self.text_value_span(v, path, out); } path.truncate(len); }
},
K_g_UBNFAST_2e_InterleaveAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/profile"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_BackrefAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/name"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_ScopeTreeAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/mode"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_DeclaresAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/symbol"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/description"); let v = self.slot(id, 1); if v != NONE { self.text_value_span(v, path, out); } path.truncate(len); }
},
K_g_UBNFAST_2e_CatalogAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/context"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_LeftAssocAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_RightAssocAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_LongestChoiceAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_PredictiveChoiceAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_PrecedenceAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/level"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_DocAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/text"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_RecoveryAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/mode"); let v = self.slot(id, 0); if v != NONE { self.text_value_span(v, path, out); } path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/syncTokens"); let v = self.slot(id, 1); if v != NONE { self.text_value_span(v, path, out); } path.truncate(len); }
},
K_g_UBNFAST_2e_SkipAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_CommonFieldAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/fieldNames"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.text_value_span(v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_EnumAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
},
K_g_UBNFAST_2e_SimpleAnnotation => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/name"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_ChoiceBody => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/alternatives"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_SequenceBody => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/elements"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.collect_value_spans(source, v, path, out); path.truncate(len); } path.truncate(len); }
},
K_g_UBNFAST_2e_AnnotatedElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/element"); self.collect_value_spans(source, self.slot(id, 0), path, out); path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/captureName"); let v = self.slot(id, 1); if v != NONE { self.text_value_span(v, path, out); } path.truncate(len); }
{ let len = path.len(); path.push_str("/fields/typeofConstraint"); let v = self.slot(id, 2); if v != NONE { self.collect_value_spans(source, v, path, out); } path.truncate(len); }
},
K_g_UBNFAST_2e_GroupElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/body"); self.collect_value_spans(source, self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_OptionalElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/body"); self.collect_value_spans(source, self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_RepeatElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/body"); self.collect_value_spans(source, self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_ErrorElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/message"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_TerminalElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/value"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_TypeofElement => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/captureName"); self.text_value_span(self.slot(id, 0), path, out); path.truncate(len); }
},
K_g_UBNFAST_2e_DottedIdentifier => { out.push(ValueSpan { path: path.clone(), span: span_of(&node), text: None });
{ let len = path.len(); path.push_str("/fields/parts"); for (i, &v) in self.items(id, 0).iter().enumerate() { let len = path.len(); let _ = std::fmt::Write::write_fmt(path, format_args!("/{i}")); self.text_value_span(v, path, out); path.truncate(len); } path.truncate(len); }
},
_ => unreachable!("unknown AST node kind"), } }
}
#[cfg(feature="json")] impl AstTree {
/// 根の canonical JSON（所有 `Ast::canonical_json` と byte 一致）。
pub fn canonical_json(&self) -> Option<String> { self.root().map(|root| serde_json::to_string(&root.canonical_value()).expect("finite canonical AST")) }
}
#[cfg(feature="json")] impl<'t> NodeRef<'t> {
pub fn canonical_value(&self) -> serde_json::Value { let (tree, id) = (self.tree, self.id); let node = tree.nodes[id as usize]; let text = |v: u32| tree.text_in(&tree.source, v); let value = |v: u32| NodeRef { tree, id: v }.canonical_value(); let _ = (&text, &value); match node.kind { KIND_TEXT => serde_json::Value::String(text(id).to_owned()), KIND_NULL => serde_json::Value::Null,
K_g_UBNFAST_2e_UBNFFile => serde_json::json!({"fields":{
"grammars":(tree.items(id, 0).iter().map(|&v| value(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"UBNFFile"}),
K_g_UBNFAST_2e_ImportDecl => serde_json::json!({"fields":{
"alias":(text(tree.slot(id, 0))),
"path":(text(tree.slot(id, 1))),
},"span":span_of(&node),"type":"ImportDecl"}),
K_g_UBNFAST_2e_GrammarDecl => serde_json::json!({"fields":{
"imports":(tree.items(id, 1).iter().map(|&v| value(v)).collect::<Vec<_>>()),
"name":(text(tree.slot(id, 0))),
"rules":(tree.items(id, 7).iter().map(|&v| value(v)).collect::<Vec<_>>()),
"settings":(tree.items(id, 3).iter().map(|&v| value(v)).collect::<Vec<_>>()),
"tokens":(tree.items(id, 5).iter().map(|&v| value(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"GrammarDecl"}),
K_g_UBNFAST_2e_GlobalSetting => serde_json::json!({"fields":{
"key":(text(tree.slot(id, 0))),
"value":(value(tree.slot(id, 1))),
},"span":span_of(&node),"type":"GlobalSetting"}),
K_g_UBNFAST_2e_StringSettingValue => serde_json::json!({"fields":{
"value":(value(tree.slot(id, 0))),
},"span":span_of(&node),"type":"StringSettingValue"}),
K_g_UBNFAST_2e_BlockSettingValue => serde_json::json!({"fields":{
"entries":(tree.items(id, 0).iter().map(|&v| value(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"BlockSettingValue"}),
K_g_UBNFAST_2e_KeyValuePair => serde_json::json!({"fields":{
"key":(text(tree.slot(id, 0))),
"value":(text(tree.slot(id, 1))),
},"span":span_of(&node),"type":"KeyValuePair"}),
K_g_UBNFAST_2e_TokenDecl => serde_json::json!({"fields":{
"name":(text(tree.slot(id, 0))),
"value":(value(tree.slot(id, 1))),
},"span":span_of(&node),"type":"TokenDecl"}),
K_g_UBNFAST_2e_UntilExpression => serde_json::json!({"fields":{
"terminator":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"UntilExpression"}),
K_g_UBNFAST_2e_NegationExpression => serde_json::json!({"fields":{
"chars":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"NegationExpression"}),
K_g_UBNFAST_2e_LookaheadExpression => serde_json::json!({"fields":{
"pattern":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"LookaheadExpression"}),
K_g_UBNFAST_2e_NegativeLookaheadExpression => serde_json::json!({"fields":{
"pattern":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"NegativeLookaheadExpression"}),
K_g_UBNFAST_2e_CharRangeExpression => serde_json::json!({"fields":{
"max":(text(tree.slot(id, 1))),
"min":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"CharRangeExpression"}),
K_g_UBNFAST_2e_CIExpression => serde_json::json!({"fields":{
"word":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"CIExpression"}),
K_g_UBNFAST_2e_RegexExpression => serde_json::json!({"fields":{
"pattern":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"RegexExpression"}),
K_g_UBNFAST_2e_AnyExpression => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"AnyExpression"}),
K_g_UBNFAST_2e_EofExpression => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"EofExpression"}),
K_g_UBNFAST_2e_EmptyExpression => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"EmptyExpression"}),
K_g_UBNFAST_2e_RuleDecl => serde_json::json!({"fields":{
"annotations":(tree.items(id, 0).iter().map(|&v| value(v)).collect::<Vec<_>>()),
"body":(value(tree.slot(id, 3))),
"name":(text(tree.slot(id, 2))),
},"span":span_of(&node),"type":"RuleDecl"}),
K_g_UBNFAST_2e_RootAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"RootAnnotation"}),
K_g_UBNFAST_2e_MappingAnnotation => serde_json::json!({"fields":{
"className":(text(tree.slot(id, 0))),
"paramNames":(tree.items(id, 1).iter().map(|&v| text(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"MappingAnnotation"}),
K_g_UBNFAST_2e_EvalAnnotation => serde_json::json!({"fields":{
"entries":(tree.items(id, 0).iter().map(|&v| value(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"EvalAnnotation"}),
K_g_UBNFAST_2e_WhitespaceAnnotation => serde_json::json!({"fields":{
"style":({ let v = tree.slot(id, 0); (v != NONE).then(|| text(v)) }),
},"span":span_of(&node),"type":"WhitespaceAnnotation"}),
K_g_UBNFAST_2e_InterleaveAnnotation => serde_json::json!({"fields":{
"profile":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"InterleaveAnnotation"}),
K_g_UBNFAST_2e_BackrefAnnotation => serde_json::json!({"fields":{
"name":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"BackrefAnnotation"}),
K_g_UBNFAST_2e_ScopeTreeAnnotation => serde_json::json!({"fields":{
"mode":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"ScopeTreeAnnotation"}),
K_g_UBNFAST_2e_DeclaresAnnotation => serde_json::json!({"fields":{
"description":({ let v = tree.slot(id, 1); (v != NONE).then(|| text(v)) }),
"symbol":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"DeclaresAnnotation"}),
K_g_UBNFAST_2e_CatalogAnnotation => serde_json::json!({"fields":{
"context":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"CatalogAnnotation"}),
K_g_UBNFAST_2e_LeftAssocAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"LeftAssocAnnotation"}),
K_g_UBNFAST_2e_RightAssocAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"RightAssocAnnotation"}),
K_g_UBNFAST_2e_LongestChoiceAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"LongestChoiceAnnotation"}),
K_g_UBNFAST_2e_PredictiveChoiceAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"PredictiveChoiceAnnotation"}),
K_g_UBNFAST_2e_PrecedenceAnnotation => serde_json::json!({"fields":{
"level":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"PrecedenceAnnotation"}),
K_g_UBNFAST_2e_DocAnnotation => serde_json::json!({"fields":{
"text":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"DocAnnotation"}),
K_g_UBNFAST_2e_RecoveryAnnotation => serde_json::json!({"fields":{
"mode":({ let v = tree.slot(id, 0); (v != NONE).then(|| text(v)) }),
"syncTokens":({ let v = tree.slot(id, 1); (v != NONE).then(|| text(v)) }),
},"span":span_of(&node),"type":"RecoveryAnnotation"}),
K_g_UBNFAST_2e_SkipAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"SkipAnnotation"}),
K_g_UBNFAST_2e_CommonFieldAnnotation => serde_json::json!({"fields":{
"fieldNames":(tree.items(id, 0).iter().map(|&v| text(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"CommonFieldAnnotation"}),
K_g_UBNFAST_2e_EnumAnnotation => serde_json::json!({"fields":{
},"span":span_of(&node),"type":"EnumAnnotation"}),
K_g_UBNFAST_2e_SimpleAnnotation => serde_json::json!({"fields":{
"name":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"SimpleAnnotation"}),
K_g_UBNFAST_2e_ChoiceBody => serde_json::json!({"fields":{
"alternatives":(tree.items(id, 0).iter().map(|&v| value(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"ChoiceBody"}),
K_g_UBNFAST_2e_SequenceBody => serde_json::json!({"fields":{
"elements":(tree.items(id, 0).iter().map(|&v| value(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"SequenceBody"}),
K_g_UBNFAST_2e_AnnotatedElement => serde_json::json!({"fields":{
"captureName":({ let v = tree.slot(id, 1); (v != NONE).then(|| text(v)) }),
"element":(value(tree.slot(id, 0))),
"typeofConstraint":({ let v = tree.slot(id, 2); (v != NONE).then(|| value(v)) }),
},"span":span_of(&node),"type":"AnnotatedElement"}),
K_g_UBNFAST_2e_GroupElement => serde_json::json!({"fields":{
"body":(value(tree.slot(id, 0))),
},"span":span_of(&node),"type":"GroupElement"}),
K_g_UBNFAST_2e_OptionalElement => serde_json::json!({"fields":{
"body":(value(tree.slot(id, 0))),
},"span":span_of(&node),"type":"OptionalElement"}),
K_g_UBNFAST_2e_RepeatElement => serde_json::json!({"fields":{
"body":(value(tree.slot(id, 0))),
},"span":span_of(&node),"type":"RepeatElement"}),
K_g_UBNFAST_2e_ErrorElement => serde_json::json!({"fields":{
"message":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"ErrorElement"}),
K_g_UBNFAST_2e_TerminalElement => serde_json::json!({"fields":{
"value":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"TerminalElement"}),
K_g_UBNFAST_2e_TypeofElement => serde_json::json!({"fields":{
"captureName":(text(tree.slot(id, 0))),
},"span":span_of(&node),"type":"TypeofElement"}),
K_g_UBNFAST_2e_DottedIdentifier => serde_json::json!({"fields":{
"parts":(tree.items(id, 0).iter().map(|&v| text(v)).collect::<Vec<_>>()),
},"span":span_of(&node),"type":"DottedIdentifier"}),
_ => unreachable!("unknown AST node kind"), } }
}
}
