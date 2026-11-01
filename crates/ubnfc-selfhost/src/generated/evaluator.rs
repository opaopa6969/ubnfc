//! Typed evaluator dispatch. Rust numeric captures remain lexemes.
#![allow(non_snake_case,unused_imports)]
use super::ast::*;
#[derive(Clone,Debug,PartialEq)] pub enum Value {Text(String),Number(f64),Bool(bool),Node(Box<Ast>),Null}
#[derive(Clone,Debug,PartialEq,Eq)] pub struct EvalError(pub String);
impl std::fmt::Display for EvalError {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {self.0.fmt(f)}}
impl std::error::Error for EvalError {}
pub type EvalResult=Result<Value,EvalError>;
pub trait Evaluator {
fn eval(&mut self,node:&Ast)->EvalResult {match node {Ast::Text(text)=>Ok(Value::Text(text.clone())),Ast::Null=>Ok(Value::Null),
Ast::g_UBNFAST_2e_UBNFFile(node)=>self.eval_g_UBNFAST_2e_UBNFFile(node),
Ast::g_UBNFAST_2e_ImportDecl(node)=>self.eval_g_UBNFAST_2e_ImportDecl(node),
Ast::g_UBNFAST_2e_GrammarDecl(node)=>self.eval_g_UBNFAST_2e_GrammarDecl(node),
Ast::g_UBNFAST_2e_GlobalSetting(node)=>self.eval_g_UBNFAST_2e_GlobalSetting(node),
Ast::g_UBNFAST_2e_StringSettingValue(node)=>self.eval_g_UBNFAST_2e_StringSettingValue(node),
Ast::g_UBNFAST_2e_BlockSettingValue(node)=>self.eval_g_UBNFAST_2e_BlockSettingValue(node),
Ast::g_UBNFAST_2e_KeyValuePair(node)=>self.eval_g_UBNFAST_2e_KeyValuePair(node),
Ast::g_UBNFAST_2e_TokenDecl(node)=>self.eval_g_UBNFAST_2e_TokenDecl(node),
Ast::g_UBNFAST_2e_DeclarativeTokenValue(node)=>self.eval_g_UBNFAST_2e_DeclarativeTokenValue(node),
Ast::g_UBNFAST_2e_LexicalChoice(node)=>self.eval_g_UBNFAST_2e_LexicalChoice(node),
Ast::g_UBNFAST_2e_LexicalSequence(node)=>self.eval_g_UBNFAST_2e_LexicalSequence(node),
Ast::g_UBNFAST_2e_LexicalElement(node)=>self.eval_g_UBNFAST_2e_LexicalElement(node),
Ast::g_UBNFAST_2e_LexicalSuffix(node)=>self.eval_g_UBNFAST_2e_LexicalSuffix(node),
Ast::g_UBNFAST_2e_LexicalAtom(node)=>self.eval_g_UBNFAST_2e_LexicalAtom(node),
Ast::g_UBNFAST_2e_UntilExpression(node)=>self.eval_g_UBNFAST_2e_UntilExpression(node),
Ast::g_UBNFAST_2e_NegationExpression(node)=>self.eval_g_UBNFAST_2e_NegationExpression(node),
Ast::g_UBNFAST_2e_LookaheadExpression(node)=>self.eval_g_UBNFAST_2e_LookaheadExpression(node),
Ast::g_UBNFAST_2e_NegativeLookaheadExpression(node)=>self.eval_g_UBNFAST_2e_NegativeLookaheadExpression(node),
Ast::g_UBNFAST_2e_CharRangeExpression(node)=>self.eval_g_UBNFAST_2e_CharRangeExpression(node),
Ast::g_UBNFAST_2e_CIExpression(node)=>self.eval_g_UBNFAST_2e_CIExpression(node),
Ast::g_UBNFAST_2e_AdapterExpression(node)=>self.eval_g_UBNFAST_2e_AdapterExpression(node),
Ast::g_UBNFAST_2e_AnyExpression(node)=>self.eval_g_UBNFAST_2e_AnyExpression(node),
Ast::g_UBNFAST_2e_EofExpression(node)=>self.eval_g_UBNFAST_2e_EofExpression(node),
Ast::g_UBNFAST_2e_EmptyExpression(node)=>self.eval_g_UBNFAST_2e_EmptyExpression(node),
Ast::g_UBNFAST_2e_RuleDecl(node)=>self.eval_g_UBNFAST_2e_RuleDecl(node),
Ast::g_UBNFAST_2e_RootAnnotation(node)=>self.eval_g_UBNFAST_2e_RootAnnotation(node),
Ast::g_UBNFAST_2e_MappingAnnotation(node)=>self.eval_g_UBNFAST_2e_MappingAnnotation(node),
Ast::g_UBNFAST_2e_EvalAnnotation(node)=>self.eval_g_UBNFAST_2e_EvalAnnotation(node),
Ast::g_UBNFAST_2e_WhitespaceAnnotation(node)=>self.eval_g_UBNFAST_2e_WhitespaceAnnotation(node),
Ast::g_UBNFAST_2e_LexicalContextAnnotation(node)=>self.eval_g_UBNFAST_2e_LexicalContextAnnotation(node),
Ast::g_UBNFAST_2e_InterleaveAnnotation(node)=>self.eval_g_UBNFAST_2e_InterleaveAnnotation(node),
Ast::g_UBNFAST_2e_BackrefAnnotation(node)=>self.eval_g_UBNFAST_2e_BackrefAnnotation(node),
Ast::g_UBNFAST_2e_ScopeTreeAnnotation(node)=>self.eval_g_UBNFAST_2e_ScopeTreeAnnotation(node),
Ast::g_UBNFAST_2e_DeclaresAnnotation(node)=>self.eval_g_UBNFAST_2e_DeclaresAnnotation(node),
Ast::g_UBNFAST_2e_CatalogAnnotation(node)=>self.eval_g_UBNFAST_2e_CatalogAnnotation(node),
Ast::g_UBNFAST_2e_LeftAssocAnnotation(node)=>self.eval_g_UBNFAST_2e_LeftAssocAnnotation(node),
Ast::g_UBNFAST_2e_RightAssocAnnotation(node)=>self.eval_g_UBNFAST_2e_RightAssocAnnotation(node),
Ast::g_UBNFAST_2e_LongestChoiceAnnotation(node)=>self.eval_g_UBNFAST_2e_LongestChoiceAnnotation(node),
Ast::g_UBNFAST_2e_UniqueLongestChoiceAnnotation(node)=>self.eval_g_UBNFAST_2e_UniqueLongestChoiceAnnotation(node),
Ast::g_UBNFAST_2e_NamePredicateAnnotation(node)=>self.eval_g_UBNFAST_2e_NamePredicateAnnotation(node),
Ast::g_UBNFAST_2e_PredicateAnnotation(node)=>self.eval_g_UBNFAST_2e_PredicateAnnotation(node),
Ast::g_UBNFAST_2e_PredictiveChoiceAnnotation(node)=>self.eval_g_UBNFAST_2e_PredictiveChoiceAnnotation(node),
Ast::g_UBNFAST_2e_PrecedenceAnnotation(node)=>self.eval_g_UBNFAST_2e_PrecedenceAnnotation(node),
Ast::g_UBNFAST_2e_DocAnnotation(node)=>self.eval_g_UBNFAST_2e_DocAnnotation(node),
Ast::g_UBNFAST_2e_RecoveryAnnotation(node)=>self.eval_g_UBNFAST_2e_RecoveryAnnotation(node),
Ast::g_UBNFAST_2e_SkipAnnotation(node)=>self.eval_g_UBNFAST_2e_SkipAnnotation(node),
Ast::g_UBNFAST_2e_CommonFieldAnnotation(node)=>self.eval_g_UBNFAST_2e_CommonFieldAnnotation(node),
Ast::g_UBNFAST_2e_EnumAnnotation(node)=>self.eval_g_UBNFAST_2e_EnumAnnotation(node),
Ast::g_UBNFAST_2e_SimpleAnnotation(node)=>self.eval_g_UBNFAST_2e_SimpleAnnotation(node),
Ast::g_UBNFAST_2e_ChoiceBody(node)=>self.eval_g_UBNFAST_2e_ChoiceBody(node),
Ast::g_UBNFAST_2e_SequenceBody(node)=>self.eval_g_UBNFAST_2e_SequenceBody(node),
Ast::g_UBNFAST_2e_AnnotatedElement(node)=>self.eval_g_UBNFAST_2e_AnnotatedElement(node),
Ast::g_UBNFAST_2e_OwnedCaptureElement(node)=>self.eval_g_UBNFAST_2e_OwnedCaptureElement(node),
Ast::g_UBNFAST_2e_ReplayElement(node)=>self.eval_g_UBNFAST_2e_ReplayElement(node),
Ast::g_UBNFAST_2e_CaptureScopeElement(node)=>self.eval_g_UBNFAST_2e_CaptureScopeElement(node),
Ast::g_UBNFAST_2e_CommitElement(node)=>self.eval_g_UBNFAST_2e_CommitElement(node),
Ast::g_UBNFAST_2e_LabelElement(node)=>self.eval_g_UBNFAST_2e_LabelElement(node),
Ast::g_UBNFAST_2e_CatchElement(node)=>self.eval_g_UBNFAST_2e_CatchElement(node),
Ast::g_UBNFAST_2e_GroupElement(node)=>self.eval_g_UBNFAST_2e_GroupElement(node),
Ast::g_UBNFAST_2e_OptionalElement(node)=>self.eval_g_UBNFAST_2e_OptionalElement(node),
Ast::g_UBNFAST_2e_RepeatElement(node)=>self.eval_g_UBNFAST_2e_RepeatElement(node),
Ast::g_UBNFAST_2e_ErrorElement(node)=>self.eval_g_UBNFAST_2e_ErrorElement(node),
Ast::g_UBNFAST_2e_TerminalElement(node)=>self.eval_g_UBNFAST_2e_TerminalElement(node),
Ast::g_UBNFAST_2e_TypeofElement(node)=>self.eval_g_UBNFAST_2e_TypeofElement(node),
Ast::g_UBNFAST_2e_DottedIdentifier(node)=>self.eval_g_UBNFAST_2e_DottedIdentifier(node),
}}
fn eval_g_UBNFAST_2e_UBNFFile(&mut self,node:&g_UBNFAST_2e_UBNFFile)->EvalResult;
fn eval_g_UBNFAST_2e_ImportDecl(&mut self,node:&g_UBNFAST_2e_ImportDecl)->EvalResult;
fn eval_g_UBNFAST_2e_GrammarDecl(&mut self,node:&g_UBNFAST_2e_GrammarDecl)->EvalResult;
fn eval_g_UBNFAST_2e_GlobalSetting(&mut self,node:&g_UBNFAST_2e_GlobalSetting)->EvalResult;
fn eval_g_UBNFAST_2e_StringSettingValue(&mut self,node:&g_UBNFAST_2e_StringSettingValue)->EvalResult;
fn eval_g_UBNFAST_2e_BlockSettingValue(&mut self,node:&g_UBNFAST_2e_BlockSettingValue)->EvalResult;
fn eval_g_UBNFAST_2e_KeyValuePair(&mut self,node:&g_UBNFAST_2e_KeyValuePair)->EvalResult;
fn eval_g_UBNFAST_2e_TokenDecl(&mut self,node:&g_UBNFAST_2e_TokenDecl)->EvalResult;
fn eval_g_UBNFAST_2e_DeclarativeTokenValue(&mut self,node:&g_UBNFAST_2e_DeclarativeTokenValue)->EvalResult;
fn eval_g_UBNFAST_2e_LexicalChoice(&mut self,node:&g_UBNFAST_2e_LexicalChoice)->EvalResult;
fn eval_g_UBNFAST_2e_LexicalSequence(&mut self,node:&g_UBNFAST_2e_LexicalSequence)->EvalResult;
fn eval_g_UBNFAST_2e_LexicalElement(&mut self,node:&g_UBNFAST_2e_LexicalElement)->EvalResult;
fn eval_g_UBNFAST_2e_LexicalSuffix(&mut self,node:&g_UBNFAST_2e_LexicalSuffix)->EvalResult;
fn eval_g_UBNFAST_2e_LexicalAtom(&mut self,node:&g_UBNFAST_2e_LexicalAtom)->EvalResult;
fn eval_g_UBNFAST_2e_UntilExpression(&mut self,node:&g_UBNFAST_2e_UntilExpression)->EvalResult;
fn eval_g_UBNFAST_2e_NegationExpression(&mut self,node:&g_UBNFAST_2e_NegationExpression)->EvalResult;
fn eval_g_UBNFAST_2e_LookaheadExpression(&mut self,node:&g_UBNFAST_2e_LookaheadExpression)->EvalResult;
fn eval_g_UBNFAST_2e_NegativeLookaheadExpression(&mut self,node:&g_UBNFAST_2e_NegativeLookaheadExpression)->EvalResult;
fn eval_g_UBNFAST_2e_CharRangeExpression(&mut self,node:&g_UBNFAST_2e_CharRangeExpression)->EvalResult;
fn eval_g_UBNFAST_2e_CIExpression(&mut self,node:&g_UBNFAST_2e_CIExpression)->EvalResult;
fn eval_g_UBNFAST_2e_AdapterExpression(&mut self,node:&g_UBNFAST_2e_AdapterExpression)->EvalResult;
fn eval_g_UBNFAST_2e_AnyExpression(&mut self,node:&g_UBNFAST_2e_AnyExpression)->EvalResult;
fn eval_g_UBNFAST_2e_EofExpression(&mut self,node:&g_UBNFAST_2e_EofExpression)->EvalResult;
fn eval_g_UBNFAST_2e_EmptyExpression(&mut self,node:&g_UBNFAST_2e_EmptyExpression)->EvalResult;
fn eval_g_UBNFAST_2e_RuleDecl(&mut self,node:&g_UBNFAST_2e_RuleDecl)->EvalResult;
fn eval_g_UBNFAST_2e_RootAnnotation(&mut self,node:&g_UBNFAST_2e_RootAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_MappingAnnotation(&mut self,node:&g_UBNFAST_2e_MappingAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_EvalAnnotation(&mut self,node:&g_UBNFAST_2e_EvalAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_WhitespaceAnnotation(&mut self,node:&g_UBNFAST_2e_WhitespaceAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_LexicalContextAnnotation(&mut self,node:&g_UBNFAST_2e_LexicalContextAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_InterleaveAnnotation(&mut self,node:&g_UBNFAST_2e_InterleaveAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_BackrefAnnotation(&mut self,node:&g_UBNFAST_2e_BackrefAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_ScopeTreeAnnotation(&mut self,node:&g_UBNFAST_2e_ScopeTreeAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_DeclaresAnnotation(&mut self,node:&g_UBNFAST_2e_DeclaresAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_CatalogAnnotation(&mut self,node:&g_UBNFAST_2e_CatalogAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_LeftAssocAnnotation(&mut self,node:&g_UBNFAST_2e_LeftAssocAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_RightAssocAnnotation(&mut self,node:&g_UBNFAST_2e_RightAssocAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_LongestChoiceAnnotation(&mut self,node:&g_UBNFAST_2e_LongestChoiceAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_UniqueLongestChoiceAnnotation(&mut self,node:&g_UBNFAST_2e_UniqueLongestChoiceAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_NamePredicateAnnotation(&mut self,node:&g_UBNFAST_2e_NamePredicateAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_PredicateAnnotation(&mut self,node:&g_UBNFAST_2e_PredicateAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_PredictiveChoiceAnnotation(&mut self,node:&g_UBNFAST_2e_PredictiveChoiceAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_PrecedenceAnnotation(&mut self,node:&g_UBNFAST_2e_PrecedenceAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_DocAnnotation(&mut self,node:&g_UBNFAST_2e_DocAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_RecoveryAnnotation(&mut self,node:&g_UBNFAST_2e_RecoveryAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_SkipAnnotation(&mut self,node:&g_UBNFAST_2e_SkipAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_CommonFieldAnnotation(&mut self,node:&g_UBNFAST_2e_CommonFieldAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_EnumAnnotation(&mut self,node:&g_UBNFAST_2e_EnumAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_SimpleAnnotation(&mut self,node:&g_UBNFAST_2e_SimpleAnnotation)->EvalResult;
fn eval_g_UBNFAST_2e_ChoiceBody(&mut self,node:&g_UBNFAST_2e_ChoiceBody)->EvalResult;
fn eval_g_UBNFAST_2e_SequenceBody(&mut self,node:&g_UBNFAST_2e_SequenceBody)->EvalResult;
fn eval_g_UBNFAST_2e_AnnotatedElement(&mut self,node:&g_UBNFAST_2e_AnnotatedElement)->EvalResult;
fn eval_g_UBNFAST_2e_OwnedCaptureElement(&mut self,node:&g_UBNFAST_2e_OwnedCaptureElement)->EvalResult;
fn eval_g_UBNFAST_2e_ReplayElement(&mut self,node:&g_UBNFAST_2e_ReplayElement)->EvalResult;
fn eval_g_UBNFAST_2e_CaptureScopeElement(&mut self,node:&g_UBNFAST_2e_CaptureScopeElement)->EvalResult;
fn eval_g_UBNFAST_2e_CommitElement(&mut self,node:&g_UBNFAST_2e_CommitElement)->EvalResult;
fn eval_g_UBNFAST_2e_LabelElement(&mut self,node:&g_UBNFAST_2e_LabelElement)->EvalResult;
fn eval_g_UBNFAST_2e_CatchElement(&mut self,node:&g_UBNFAST_2e_CatchElement)->EvalResult;
fn eval_g_UBNFAST_2e_GroupElement(&mut self,node:&g_UBNFAST_2e_GroupElement)->EvalResult;
fn eval_g_UBNFAST_2e_OptionalElement(&mut self,node:&g_UBNFAST_2e_OptionalElement)->EvalResult;
fn eval_g_UBNFAST_2e_RepeatElement(&mut self,node:&g_UBNFAST_2e_RepeatElement)->EvalResult;
fn eval_g_UBNFAST_2e_ErrorElement(&mut self,node:&g_UBNFAST_2e_ErrorElement)->EvalResult;
fn eval_g_UBNFAST_2e_TerminalElement(&mut self,node:&g_UBNFAST_2e_TerminalElement)->EvalResult;
fn eval_g_UBNFAST_2e_TypeofElement(&mut self,node:&g_UBNFAST_2e_TypeofElement)->EvalResult;
fn eval_g_UBNFAST_2e_DottedIdentifier(&mut self,node:&g_UBNFAST_2e_DottedIdentifier)->EvalResult;
}
/// Automatic kinds work without overrides; semantic callbacks report a missing implementation.
pub struct DefaultEvaluator;
impl Evaluator for DefaultEvaluator {
fn eval_g_UBNFAST_2e_AdapterExpression(&mut self,node:&g_UBNFAST_2e_AdapterExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_AdapterExpression".into()))}
fn eval_g_UBNFAST_2e_AnnotatedElement(&mut self,node:&g_UBNFAST_2e_AnnotatedElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_AnnotatedElement".into()))}
fn eval_g_UBNFAST_2e_AnyExpression(&mut self,node:&g_UBNFAST_2e_AnyExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_AnyExpression".into()))}
fn eval_g_UBNFAST_2e_BackrefAnnotation(&mut self,node:&g_UBNFAST_2e_BackrefAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_BackrefAnnotation".into()))}
fn eval_g_UBNFAST_2e_BlockSettingValue(&mut self,node:&g_UBNFAST_2e_BlockSettingValue)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_BlockSettingValue".into()))}
fn eval_g_UBNFAST_2e_CIExpression(&mut self,node:&g_UBNFAST_2e_CIExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CIExpression".into()))}
fn eval_g_UBNFAST_2e_CaptureScopeElement(&mut self,node:&g_UBNFAST_2e_CaptureScopeElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CaptureScopeElement".into()))}
fn eval_g_UBNFAST_2e_CatalogAnnotation(&mut self,node:&g_UBNFAST_2e_CatalogAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CatalogAnnotation".into()))}
fn eval_g_UBNFAST_2e_CatchElement(&mut self,node:&g_UBNFAST_2e_CatchElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CatchElement".into()))}
fn eval_g_UBNFAST_2e_CharRangeExpression(&mut self,node:&g_UBNFAST_2e_CharRangeExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CharRangeExpression".into()))}
fn eval_g_UBNFAST_2e_ChoiceBody(&mut self,node:&g_UBNFAST_2e_ChoiceBody)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_ChoiceBody".into()))}
fn eval_g_UBNFAST_2e_CommitElement(&mut self,node:&g_UBNFAST_2e_CommitElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CommitElement".into()))}
fn eval_g_UBNFAST_2e_CommonFieldAnnotation(&mut self,node:&g_UBNFAST_2e_CommonFieldAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_CommonFieldAnnotation".into()))}
fn eval_g_UBNFAST_2e_DeclarativeTokenValue(&mut self,node:&g_UBNFAST_2e_DeclarativeTokenValue)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_DeclarativeTokenValue".into()))}
fn eval_g_UBNFAST_2e_DeclaresAnnotation(&mut self,node:&g_UBNFAST_2e_DeclaresAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_DeclaresAnnotation".into()))}
fn eval_g_UBNFAST_2e_DocAnnotation(&mut self,node:&g_UBNFAST_2e_DocAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_DocAnnotation".into()))}
fn eval_g_UBNFAST_2e_DottedIdentifier(&mut self,node:&g_UBNFAST_2e_DottedIdentifier)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_DottedIdentifier".into()))}
fn eval_g_UBNFAST_2e_EmptyExpression(&mut self,node:&g_UBNFAST_2e_EmptyExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_EmptyExpression".into()))}
fn eval_g_UBNFAST_2e_EnumAnnotation(&mut self,node:&g_UBNFAST_2e_EnumAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_EnumAnnotation".into()))}
fn eval_g_UBNFAST_2e_EofExpression(&mut self,node:&g_UBNFAST_2e_EofExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_EofExpression".into()))}
fn eval_g_UBNFAST_2e_ErrorElement(&mut self,node:&g_UBNFAST_2e_ErrorElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_ErrorElement".into()))}
fn eval_g_UBNFAST_2e_EvalAnnotation(&mut self,node:&g_UBNFAST_2e_EvalAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_EvalAnnotation".into()))}
fn eval_g_UBNFAST_2e_GlobalSetting(&mut self,node:&g_UBNFAST_2e_GlobalSetting)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_GlobalSetting".into()))}
fn eval_g_UBNFAST_2e_GrammarDecl(&mut self,node:&g_UBNFAST_2e_GrammarDecl)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_GrammarDecl".into()))}
fn eval_g_UBNFAST_2e_GroupElement(&mut self,node:&g_UBNFAST_2e_GroupElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_GroupElement".into()))}
fn eval_g_UBNFAST_2e_ImportDecl(&mut self,node:&g_UBNFAST_2e_ImportDecl)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_ImportDecl".into()))}
fn eval_g_UBNFAST_2e_InterleaveAnnotation(&mut self,node:&g_UBNFAST_2e_InterleaveAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_InterleaveAnnotation".into()))}
fn eval_g_UBNFAST_2e_KeyValuePair(&mut self,node:&g_UBNFAST_2e_KeyValuePair)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_KeyValuePair".into()))}
fn eval_g_UBNFAST_2e_LabelElement(&mut self,node:&g_UBNFAST_2e_LabelElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LabelElement".into()))}
fn eval_g_UBNFAST_2e_LeftAssocAnnotation(&mut self,node:&g_UBNFAST_2e_LeftAssocAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LeftAssocAnnotation".into()))}
fn eval_g_UBNFAST_2e_LexicalAtom(&mut self,node:&g_UBNFAST_2e_LexicalAtom)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LexicalAtom".into()))}
fn eval_g_UBNFAST_2e_LexicalChoice(&mut self,node:&g_UBNFAST_2e_LexicalChoice)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LexicalChoice".into()))}
fn eval_g_UBNFAST_2e_LexicalContextAnnotation(&mut self,node:&g_UBNFAST_2e_LexicalContextAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LexicalContextAnnotation".into()))}
fn eval_g_UBNFAST_2e_LexicalElement(&mut self,node:&g_UBNFAST_2e_LexicalElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LexicalElement".into()))}
fn eval_g_UBNFAST_2e_LexicalSequence(&mut self,node:&g_UBNFAST_2e_LexicalSequence)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LexicalSequence".into()))}
fn eval_g_UBNFAST_2e_LexicalSuffix(&mut self,node:&g_UBNFAST_2e_LexicalSuffix)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LexicalSuffix".into()))}
fn eval_g_UBNFAST_2e_LongestChoiceAnnotation(&mut self,node:&g_UBNFAST_2e_LongestChoiceAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LongestChoiceAnnotation".into()))}
fn eval_g_UBNFAST_2e_LookaheadExpression(&mut self,node:&g_UBNFAST_2e_LookaheadExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_LookaheadExpression".into()))}
fn eval_g_UBNFAST_2e_MappingAnnotation(&mut self,node:&g_UBNFAST_2e_MappingAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_MappingAnnotation".into()))}
fn eval_g_UBNFAST_2e_NamePredicateAnnotation(&mut self,node:&g_UBNFAST_2e_NamePredicateAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_NamePredicateAnnotation".into()))}
fn eval_g_UBNFAST_2e_NegationExpression(&mut self,node:&g_UBNFAST_2e_NegationExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_NegationExpression".into()))}
fn eval_g_UBNFAST_2e_NegativeLookaheadExpression(&mut self,node:&g_UBNFAST_2e_NegativeLookaheadExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_NegativeLookaheadExpression".into()))}
fn eval_g_UBNFAST_2e_OptionalElement(&mut self,node:&g_UBNFAST_2e_OptionalElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_OptionalElement".into()))}
fn eval_g_UBNFAST_2e_OwnedCaptureElement(&mut self,node:&g_UBNFAST_2e_OwnedCaptureElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_OwnedCaptureElement".into()))}
fn eval_g_UBNFAST_2e_PrecedenceAnnotation(&mut self,node:&g_UBNFAST_2e_PrecedenceAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_PrecedenceAnnotation".into()))}
fn eval_g_UBNFAST_2e_PredicateAnnotation(&mut self,node:&g_UBNFAST_2e_PredicateAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_PredicateAnnotation".into()))}
fn eval_g_UBNFAST_2e_PredictiveChoiceAnnotation(&mut self,node:&g_UBNFAST_2e_PredictiveChoiceAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_PredictiveChoiceAnnotation".into()))}
fn eval_g_UBNFAST_2e_RecoveryAnnotation(&mut self,node:&g_UBNFAST_2e_RecoveryAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_RecoveryAnnotation".into()))}
fn eval_g_UBNFAST_2e_RepeatElement(&mut self,node:&g_UBNFAST_2e_RepeatElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_RepeatElement".into()))}
fn eval_g_UBNFAST_2e_ReplayElement(&mut self,node:&g_UBNFAST_2e_ReplayElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_ReplayElement".into()))}
fn eval_g_UBNFAST_2e_RightAssocAnnotation(&mut self,node:&g_UBNFAST_2e_RightAssocAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_RightAssocAnnotation".into()))}
fn eval_g_UBNFAST_2e_RootAnnotation(&mut self,node:&g_UBNFAST_2e_RootAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_RootAnnotation".into()))}
fn eval_g_UBNFAST_2e_RuleDecl(&mut self,node:&g_UBNFAST_2e_RuleDecl)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_RuleDecl".into()))}
fn eval_g_UBNFAST_2e_ScopeTreeAnnotation(&mut self,node:&g_UBNFAST_2e_ScopeTreeAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_ScopeTreeAnnotation".into()))}
fn eval_g_UBNFAST_2e_SequenceBody(&mut self,node:&g_UBNFAST_2e_SequenceBody)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_SequenceBody".into()))}
fn eval_g_UBNFAST_2e_SimpleAnnotation(&mut self,node:&g_UBNFAST_2e_SimpleAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_SimpleAnnotation".into()))}
fn eval_g_UBNFAST_2e_SkipAnnotation(&mut self,node:&g_UBNFAST_2e_SkipAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_SkipAnnotation".into()))}
fn eval_g_UBNFAST_2e_StringSettingValue(&mut self,node:&g_UBNFAST_2e_StringSettingValue)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_StringSettingValue".into()))}
fn eval_g_UBNFAST_2e_TerminalElement(&mut self,node:&g_UBNFAST_2e_TerminalElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_TerminalElement".into()))}
fn eval_g_UBNFAST_2e_TokenDecl(&mut self,node:&g_UBNFAST_2e_TokenDecl)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_TokenDecl".into()))}
fn eval_g_UBNFAST_2e_TypeofElement(&mut self,node:&g_UBNFAST_2e_TypeofElement)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_TypeofElement".into()))}
fn eval_g_UBNFAST_2e_UBNFFile(&mut self,node:&g_UBNFAST_2e_UBNFFile)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_UBNFFile".into()))}
fn eval_g_UBNFAST_2e_UniqueLongestChoiceAnnotation(&mut self,node:&g_UBNFAST_2e_UniqueLongestChoiceAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_UniqueLongestChoiceAnnotation".into()))}
fn eval_g_UBNFAST_2e_UntilExpression(&mut self,node:&g_UBNFAST_2e_UntilExpression)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_UntilExpression".into()))}
fn eval_g_UBNFAST_2e_WhitespaceAnnotation(&mut self,node:&g_UBNFAST_2e_WhitespaceAnnotation)->EvalResult {let _=(node,);Err(EvalError("callback required: eval_g_UBNFAST_2e_WhitespaceAnnotation".into()))}
}
pub fn evaluate<E:Evaluator+?Sized>(node:&Ast,evaluator:&mut E)->EvalResult {evaluator.eval(node)}
