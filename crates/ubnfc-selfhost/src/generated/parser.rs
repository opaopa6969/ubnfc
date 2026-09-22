//! IR の expression/rule ごとの静的関数。c=consumed, m=matchOnly。
#![allow(dead_code, unused_mut)]
use super::api::*;
use super::runtime::*;
#[allow(unused_imports)]
use super::rt::{diag::Diag,memo::Key};
pub fn parse(text:&str)->ParseResult {parse_with_options(text,ParseOptions::default())}
pub fn parse_with_options(text:&str,options:ParseOptions)->ParseResult {parse_with_scanner(text,options,&mut RejectExtern)}
pub fn parse_with_scanner(text:&str,options:ParseOptions,scanner:&mut dyn TokenScanner)->ParseResult {
let mut parser=Session::new(text,options,scanner);
let step=parser.r0_c(State::default());parser.finish(step)
}
pub fn parse_entry_with_options(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions)->Result<ParseResult,&'static str> {
let mut scanner=RejectExtern;let mut parser=Session::new(text,options,&mut scanner);let step=match (grammar,entry) {
("UBNF",None)=>parser.r0_c(State::default()),
("UBNF",Some("UBNFFile"))=>parser.r0_c(State::default()),
("UBNF",Some("ImportDecl"))=>parser.r1_c(State::default()),
("UBNF",Some("GrammarDecl"))=>parser.r2_c(State::default()),
("UBNF",Some("GlobalSetting"))=>parser.r3_c(State::default()),
("UBNF",Some("SettingValue"))=>parser.r4_c(State::default()),
("UBNF",Some("StringSettingValue"))=>parser.r5_c(State::default()),
("UBNF",Some("BlockSettingValue"))=>parser.r6_c(State::default()),
("UBNF",Some("KeyValuePair"))=>parser.r7_c(State::default()),
("UBNF",Some("TokenDecl"))=>parser.r8_c(State::default()),
("UBNF",Some("TokenValue"))=>parser.r9_c(State::default()),
("UBNF",Some("UntilExpression"))=>parser.r10_c(State::default()),
("UBNF",Some("NegationExpression"))=>parser.r11_c(State::default()),
("UBNF",Some("LookaheadExpression"))=>parser.r12_c(State::default()),
("UBNF",Some("NegativeLookaheadExpression"))=>parser.r13_c(State::default()),
("UBNF",Some("CharRangeExpression"))=>parser.r14_c(State::default()),
("UBNF",Some("CIExpression"))=>parser.r15_c(State::default()),
("UBNF",Some("RegexExpression"))=>parser.r16_c(State::default()),
("UBNF",Some("AnyExpression"))=>parser.r17_c(State::default()),
("UBNF",Some("EofExpression"))=>parser.r18_c(State::default()),
("UBNF",Some("EmptyExpression"))=>parser.r19_c(State::default()),
("UBNF",Some("RuleDecl"))=>parser.r20_c(State::default()),
("UBNF",Some("Annotation"))=>parser.r21_c(State::default()),
("UBNF",Some("RootAnnotation"))=>parser.r22_c(State::default()),
("UBNF",Some("MappingAnnotation"))=>parser.r23_c(State::default()),
("UBNF",Some("EvalAnnotation"))=>parser.r24_c(State::default()),
("UBNF",Some("WhitespaceAnnotation"))=>parser.r25_c(State::default()),
("UBNF",Some("InterleaveAnnotation"))=>parser.r26_c(State::default()),
("UBNF",Some("BackrefAnnotation"))=>parser.r27_c(State::default()),
("UBNF",Some("ScopeTreeAnnotation"))=>parser.r28_c(State::default()),
("UBNF",Some("DeclaresAnnotation"))=>parser.r29_c(State::default()),
("UBNF",Some("CatalogAnnotation"))=>parser.r30_c(State::default()),
("UBNF",Some("LeftAssocAnnotation"))=>parser.r31_c(State::default()),
("UBNF",Some("RightAssocAnnotation"))=>parser.r32_c(State::default()),
("UBNF",Some("LongestChoiceAnnotation"))=>parser.r33_c(State::default()),
("UBNF",Some("PredictiveChoiceAnnotation"))=>parser.r34_c(State::default()),
("UBNF",Some("PrecedenceAnnotation"))=>parser.r35_c(State::default()),
("UBNF",Some("DocAnnotation"))=>parser.r36_c(State::default()),
("UBNF",Some("RecoveryAnnotation"))=>parser.r37_c(State::default()),
("UBNF",Some("SkipAnnotation"))=>parser.r38_c(State::default()),
("UBNF",Some("CommonFieldAnnotation"))=>parser.r39_c(State::default()),
("UBNF",Some("EnumAnnotation"))=>parser.r40_c(State::default()),
("UBNF",Some("SimpleAnnotation"))=>parser.r41_c(State::default()),
("UBNF",Some("ChoiceBody"))=>parser.r42_c(State::default()),
("UBNF",Some("SequenceBody"))=>parser.r43_c(State::default()),
("UBNF",Some("AnnotatedElement"))=>parser.r44_c(State::default()),
("UBNF",Some("PostfixQuantifier"))=>parser.r45_c(State::default()),
("UBNF",Some("SeparatedBy"))=>parser.r46_c(State::default()),
("UBNF",Some("BoundedQuantifier"))=>parser.r47_c(State::default()),
("UBNF",Some("Digits"))=>parser.r48_c(State::default()),
("UBNF",Some("AtomicElement"))=>parser.r49_c(State::default()),
("UBNF",Some("GroupElement"))=>parser.r50_c(State::default()),
("UBNF",Some("OptionalElement"))=>parser.r51_c(State::default()),
("UBNF",Some("RepeatElement"))=>parser.r52_c(State::default()),
("UBNF",Some("ErrorElement"))=>parser.r53_c(State::default()),
("UBNF",Some("TerminalElement"))=>parser.r54_c(State::default()),
("UBNF",Some("QuantifiedRef"))=>parser.r55_c(State::default()),
("UBNF",Some("TypeofElement"))=>parser.r56_c(State::default()),
("UBNF",Some("DottedIdentifier"))=>parser.r57_c(State::default()),
("UBNF",Some("RuleBody"))=>parser.r58_c(State::default()),
_=>return Err("unsupported grammar or entry"),};Ok(parser.finish(step))}
impl Session<'_> {
fn r0_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e0_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:0,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(0,out.diag);out
}
fn r1_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e3_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:1,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(1,out.diag);out
}
fn r2_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e8_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:2,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(2,out.diag);out
}
fn r3_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e21_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:3,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(3,out.diag);out
}
fn r4_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e26_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:4,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(4,out.diag);out
}
fn r5_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e29_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:5,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(5,out.diag);out
}
fn r6_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e31_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:6,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(6,out.diag);out
}
fn r7_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e36_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:7,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(7,out.diag);out
}
fn r8_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e40_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:8,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(8,out.diag);out
}
fn r9_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e45_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:9,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(9,out.diag);out
}
fn r10_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e57_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:10,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(10,out.diag);out
}
fn r11_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e62_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:11,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(11,out.diag);out
}
fn r12_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e67_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:12,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(12,out.diag);out
}
fn r13_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e72_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:13,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(13,out.diag);out
}
fn r14_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e77_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:14,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(14,out.diag);out
}
fn r15_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e84_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:15,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(15,out.diag);out
}
fn r16_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e89_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:16,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(16,out.diag);out
}
fn r17_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e94_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:17,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(17,out.diag);out
}
fn r18_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e96_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:18,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(18,out.diag);out
}
fn r19_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e98_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:19,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(19,out.diag);out
}
fn r20_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e100_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:20,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(20,out.diag);out
}
fn r21_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e107_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:21,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(21,out.diag);out
}
fn r22_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e128_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:22,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(22,out.diag);out
}
fn r23_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e130_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:23,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(23,out.diag);out
}
fn r24_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e147_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:24,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(24,out.diag);out
}
fn r25_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e156_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:25,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(25,out.diag);out
}
fn r26_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e163_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:26,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(26,out.diag);out
}
fn r27_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e170_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:27,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(27,out.diag);out
}
fn r28_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e177_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:28,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(28,out.diag);out
}
fn r29_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e184_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:29,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(29,out.diag);out
}
fn r30_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e197_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:30,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(30,out.diag);out
}
fn r31_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e204_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:31,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(31,out.diag);out
}
fn r32_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e206_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:32,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(32,out.diag);out
}
fn r33_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e208_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:33,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(33,out.diag);out
}
fn r34_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e210_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:34,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(34,out.diag);out
}
fn r35_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e212_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:35,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(35,out.diag);out
}
fn r36_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e219_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:36,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(36,out.diag);out
}
fn r37_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e224_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:37,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(37,out.diag);out
}
fn r38_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e235_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:38,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(38,out.diag);out
}
fn r39_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e237_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:39,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(39,out.diag);out
}
fn r40_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e246_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:40,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(40,out.diag);out
}
fn r41_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e248_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:41,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(41,out.diag);out
}
fn r42_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e251_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:42,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(42,out.diag);out
}
fn r43_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e257_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:43,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(43,out.diag);out
}
fn r44_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e260_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:44,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(44,out.diag);out
}
fn r45_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e274_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:45,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(45,out.diag);out
}
fn r46_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e278_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:46,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(46,out.diag);out
}
fn r47_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e281_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:47,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(47,out.diag);out
}
fn r48_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e290_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:48,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(48,out.diag);out
}
fn r49_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e294_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:49,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(49,out.diag);out
}
fn r50_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e301_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:50,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(50,out.diag);out
}
fn r51_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e305_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:51,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(51,out.diag);out
}
fn r52_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e309_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:52,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(52,out.diag);out
}
fn r53_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e313_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:53,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(53,out.diag);out
}
fn r54_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e318_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:54,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(54,out.diag);out
}
fn r55_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e320_c(state);
if out.ok {
for (site,span) in self.scope_captures(out.events,&[58]) { let name=self.text(span);let offset_cp=self.cp(span[0]);let len_cp=self.cp(span[1])-offset_cp;
if site==58 {if let Some((_,target))=self.scope_captures(out.events,&[58]).first() {let expected=self.text(*target);if expected!=name {self.effect(ScanEffect::Diagnostic{message:format!("back-reference mismatch: expected '{expected}' but got '{name}'"),offset_cp,len_cp,severity:Severity::Error});}}}
let _=(&name,offset_cp,len_cp); }
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:55,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(55,out.diag);out
}
fn r56_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e326_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:56,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(56,out.diag);out
}
fn r57_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e331_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:57,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(57,out.diag);out
}
fn r58_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e337_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:58,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
self.depth-=1;out.diag=self.diag.rule(58,out.diag);out
}
fn e0_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:0,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b0_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b0_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e1_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e1_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:1,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b1_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b1_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e2_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:0,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e2_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:2,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b2_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b2_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r2_c(out.state);
out
}
fn e3_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:3,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b3_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b3_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e4_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e5_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e6_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e7_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e4_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:4,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b4_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:4,rule:1,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b4_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@import",true,"@import");
out
}
fn e5_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:5,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b5_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:5,rule:1,span,text_span});}
out.events=self.event(Event::Capture {site:1,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b5_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e6_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:6,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b6_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:6,rule:1,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b6_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"from",true,"from");
out
}
fn e7_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:7,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b7_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:7,rule:1,span,text_span});}
out.events=self.event(Event::Capture {site:2,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b7_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e8_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:8,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b8_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b8_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e9_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e10_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e11_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e12_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e14_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e16_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e18_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e20_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e9_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:9,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b9_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:9,rule:2,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b9_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"grammar",true,"grammar");
out
}
fn e10_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:10,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b10_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:10,rule:2,span,text_span});}
out.events=self.event(Event::Capture {site:3,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b10_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e11_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:11,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b11_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:11,rule:2,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b11_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,"{");
out
}
fn e12_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:12,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b12_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b12_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e13_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:4,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e13_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:13,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b13_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b13_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r1_c(out.state);
out
}
fn e14_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:14,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b14_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b14_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e15_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:5,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e15_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:15,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b15_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b15_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r3_c(out.state);
out
}
fn e16_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:16,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b16_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b16_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e17_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:6,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e17_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:17,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b17_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b17_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r8_c(out.state);
out
}
fn e18_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:18,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b18_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b18_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e19_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:7,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e19_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:19,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b19_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b19_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r20_c(out.state);
out
}
fn e20_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:20,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b20_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:20,rule:2,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b20_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,"}");
out
}
fn e21_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:21,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b21_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b21_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e22_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e23_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e24_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e25_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e22_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:22,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b22_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:22,rule:3,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b22_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,"@");
out
}
fn e23_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:23,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b23_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:23,rule:3,span,text_span});}
out.events=self.event(Event::Capture {site:8,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b23_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e24_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:24,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b24_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:24,rule:3,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b24_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,":");
out
}
fn e25_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:25,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b25_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:9,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b25_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r4_c(out.state);
out
}
fn e26_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:26,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b26_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b26_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e27_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e28_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e27_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:27,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b27_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b27_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r5_c(out.state);
out
}
fn e28_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:28,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b28_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b28_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r6_c(out.state);
out
}
fn e29_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:29,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b29_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b29_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e30_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e30_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:30,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b30_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:10,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b30_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r57_c(out.state);
out
}
fn e31_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:31,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b31_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b31_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e32_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e33_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e35_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e32_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:32,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b32_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:32,rule:6,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b32_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,"{");
out
}
fn e33_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:33,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b33_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b33_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e34_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:11,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e34_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:34,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b34_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b34_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e35_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:35,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b35_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:35,rule:6,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b35_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,"}");
out
}
fn e36_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:36,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b36_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b36_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e37_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e38_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e39_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e37_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:37,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b37_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:37,rule:7,span,text_span});}
out.events=self.event(Event::Capture {site:12,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b37_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e38_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:38,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b38_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:38,rule:7,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b38_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,":");
out
}
fn e39_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:39,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b39_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:39,rule:7,span,text_span});}
out.events=self.event(Event::Capture {site:13,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b39_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e40_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:40,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b40_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b40_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e41_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e42_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e43_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e44_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e41_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:41,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b41_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:41,rule:8,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b41_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"token",true,"token");
out
}
fn e42_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:42,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b42_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:42,rule:8,span,text_span});}
out.events=self.event(Event::Capture {site:14,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b42_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e43_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:43,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b43_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:43,rule:8,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b43_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e44_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:44,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b44_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:15,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b44_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r9_c(out.state);
out
}
fn e45_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:45,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b45_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b45_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e46_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e47_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e48_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e49_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e50_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e51_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e52_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e53_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e54_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e55_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e56_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e46_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:46,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b46_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b46_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e47_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:47,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b47_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b47_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r11_c(out.state);
out
}
fn e48_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:48,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b48_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b48_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r12_c(out.state);
out
}
fn e49_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:49,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b49_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b49_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r13_c(out.state);
out
}
fn e50_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:50,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b50_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b50_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r14_c(out.state);
out
}
fn e51_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:51,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b51_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b51_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r15_c(out.state);
out
}
fn e52_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:52,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b52_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b52_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r16_c(out.state);
out
}
fn e53_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:53,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b53_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b53_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r17_c(out.state);
out
}
fn e54_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:54,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b54_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b54_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r18_c(out.state);
out
}
fn e55_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:55,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b55_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b55_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r19_c(out.state);
out
}
fn e56_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:56,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b56_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b56_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r57_c(out.state);
out
}
fn e57_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:57,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b57_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b57_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e58_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e59_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e60_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e61_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e58_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:58,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b58_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:58,rule:10,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b58_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"UNTIL",true,"UNTIL");
out
}
fn e59_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:59,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b59_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:59,rule:10,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b59_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e60_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:60,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b60_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:60,rule:10,span,text_span});}
out.events=self.event(Event::Capture {site:16,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b60_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e61_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:61,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b61_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:61,rule:10,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b61_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e62_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:62,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b62_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b62_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e63_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e64_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e65_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e66_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e63_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:63,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b63_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:63,rule:11,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b63_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATION",true,"NEGATION");
out
}
fn e64_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:64,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b64_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:64,rule:11,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b64_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e65_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:65,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b65_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:65,rule:11,span,text_span});}
out.events=self.event(Event::Capture {site:17,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b65_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e66_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:66,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b66_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:66,rule:11,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b66_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e67_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:67,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b67_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b67_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e68_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e69_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e70_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e71_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e68_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:68,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b68_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:68,rule:12,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b68_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LOOKAHEAD",true,"LOOKAHEAD");
out
}
fn e69_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:69,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b69_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:69,rule:12,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b69_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e70_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:70,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b70_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:70,rule:12,span,text_span});}
out.events=self.event(Event::Capture {site:18,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b70_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e71_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:71,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b71_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:71,rule:12,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b71_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e72_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:72,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b72_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b72_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e73_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e74_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e75_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e76_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e73_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:73,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b73_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:73,rule:13,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b73_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATIVE_LOOKAHEAD",true,"NEGATIVE_LOOKAHEAD");
out
}
fn e74_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:74,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b74_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:74,rule:13,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b74_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e75_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:75,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b75_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:75,rule:13,span,text_span});}
out.events=self.event(Event::Capture {site:19,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b75_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e76_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:76,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b76_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:76,rule:13,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b76_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e77_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:77,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b77_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b77_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e78_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e79_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e80_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e81_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e82_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e83_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e78_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:78,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b78_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:78,rule:14,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b78_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CHAR_RANGE",true,"CHAR_RANGE");
out
}
fn e79_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:79,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b79_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:79,rule:14,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b79_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e80_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:80,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b80_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:80,rule:14,span,text_span});}
out.events=self.event(Event::Capture {site:20,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b80_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e81_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:81,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b81_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:81,rule:14,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b81_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e82_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:82,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b82_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:82,rule:14,span,text_span});}
out.events=self.event(Event::Capture {site:21,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b82_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e83_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:83,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b83_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:83,rule:14,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b83_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e84_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:84,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b84_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b84_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e85_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e86_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e87_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e88_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e85_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:85,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b85_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:85,rule:15,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b85_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CI",true,"CI");
out
}
fn e86_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:86,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b86_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:86,rule:15,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b86_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e87_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:87,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b87_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:87,rule:15,span,text_span});}
out.events=self.event(Event::Capture {site:22,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b87_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e88_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:88,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b88_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:88,rule:15,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b88_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e89_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:89,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b89_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b89_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e90_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e91_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e92_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e93_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e90_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:90,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b90_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:90,rule:16,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b90_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"REGEX",true,"REGEX");
out
}
fn e91_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:91,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b91_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:91,rule:16,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b91_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e92_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:92,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b92_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:92,rule:16,span,text_span});}
out.events=self.event(Event::Capture {site:23,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b92_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e93_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:93,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b93_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:93,rule:16,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b93_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e94_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:94,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b94_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b94_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e95_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e95_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:95,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b95_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:95,rule:17,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b95_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ANY",true,"ANY");
out
}
fn e96_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:96,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b96_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b96_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e97_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e97_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:97,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b97_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:97,rule:18,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b97_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EOF",true,"EOF");
out
}
fn e98_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:98,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b98_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b98_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e99_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e99_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:99,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b99_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:99,rule:19,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b99_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EMPTY",true,"EMPTY");
out
}
fn e100_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:100,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b100_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b100_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e101_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e103_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e104_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e105_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e106_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e101_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:101,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b101_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b101_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e102_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:24,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e102_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:102,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b102_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b102_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r21_c(out.state);
out
}
fn e103_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:103,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b103_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:103,rule:20,span,text_span});}
out.events=self.event(Event::Capture {site:25,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b103_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e104_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:104,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b104_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:104,rule:20,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b104_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"::=",true,"::=");
out
}
fn e105_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:105,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b105_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:26,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b105_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e106_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:106,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b106_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:106,rule:20,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b106_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,";",true,";");
out
}
fn e107_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:107,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b107_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b107_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e108_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e109_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e110_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e111_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e112_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e113_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e114_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e115_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e116_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e117_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e118_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e119_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e120_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e121_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e122_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e123_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e124_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e125_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e126_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e127_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e108_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:108,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b108_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b108_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r22_c(out.state);
out
}
fn e109_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:109,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b109_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b109_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r23_c(out.state);
out
}
fn e110_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:110,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b110_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b110_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r24_c(out.state);
out
}
fn e111_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:111,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b111_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b111_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r25_c(out.state);
out
}
fn e112_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:112,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b112_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b112_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r26_c(out.state);
out
}
fn e113_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:113,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b113_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b113_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r27_c(out.state);
out
}
fn e114_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:114,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b114_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b114_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r28_c(out.state);
out
}
fn e115_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:115,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b115_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b115_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r29_c(out.state);
out
}
fn e116_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:116,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b116_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b116_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r30_c(out.state);
out
}
fn e117_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:117,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b117_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b117_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r31_c(out.state);
out
}
fn e118_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:118,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b118_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b118_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r32_c(out.state);
out
}
fn e119_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:119,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b119_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b119_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r33_c(out.state);
out
}
fn e120_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:120,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b120_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b120_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r34_c(out.state);
out
}
fn e121_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:121,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b121_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b121_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r35_c(out.state);
out
}
fn e122_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:122,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b122_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b122_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r36_c(out.state);
out
}
fn e123_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:123,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b123_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b123_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r37_c(out.state);
out
}
fn e124_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:124,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b124_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b124_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r38_c(out.state);
out
}
fn e125_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:125,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b125_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b125_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r39_c(out.state);
out
}
fn e126_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:126,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b126_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b126_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r40_c(out.state);
out
}
fn e127_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:127,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b127_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b127_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r41_c(out.state);
out
}
fn e128_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:128,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b128_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b128_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e129_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e129_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:129,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b129_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:129,rule:22,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b129_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@root",true,"@root");
out
}
fn e130_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:130,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b130_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b130_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e131_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e132_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e133_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e134_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e146_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e131_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:131,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b131_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:131,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b131_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@mapping",true,"@mapping");
out
}
fn e132_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:132,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b132_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:132,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b132_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e133_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:133,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b133_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:133,rule:23,span,text_span});}
out.events=self.event(Event::Capture {site:27,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b133_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"CLASS_NAME");
out
}
fn e134_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:134,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b134_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b134_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e135_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e135_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:135,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b135_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b135_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e136_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e137_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e138_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e139_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e140_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e141_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e145_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e136_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:136,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b136_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:136,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b136_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e137_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:137,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b137_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:137,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b137_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"params",true,"params");
out
}
fn e138_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:138,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b138_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:138,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b138_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e139_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:139,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b139_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:139,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b139_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,"[");
out
}
fn e140_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:140,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b140_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:140,rule:23,span,text_span});}
out.events=self.event(Event::Capture {site:28,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b140_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e141_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:141,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b141_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b141_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e142_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e142_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:142,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b142_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b142_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e143_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e144_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e143_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:143,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b143_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:143,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b143_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e144_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:144,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b144_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:144,rule:23,span,text_span});}
out.events=self.event(Event::Capture {site:29,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b144_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e145_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:145,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b145_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:145,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b145_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,"]");
out
}
fn e146_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:146,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b146_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:146,rule:23,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b146_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e147_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:147,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b147_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b147_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e148_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e149_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e150_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e151_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e155_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e148_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:148,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b148_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:148,rule:24,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b148_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@eval",true,"@eval");
out
}
fn e149_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:149,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b149_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:149,rule:24,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b149_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e150_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:150,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b150_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:30,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b150_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e151_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:151,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b151_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b151_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e152_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e152_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:152,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b152_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b152_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e153_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e154_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e153_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:153,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b153_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:153,rule:24,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b153_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e154_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:154,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b154_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:31,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b154_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e155_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:155,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b155_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:155,rule:24,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b155_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e156_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:156,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b156_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b156_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e157_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e158_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e157_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:157,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b157_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:157,rule:25,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b157_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@whitespace",true,"@whitespace");
out
}
fn e158_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:158,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b158_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b158_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e159_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e159_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:159,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b159_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b159_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e160_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e161_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e162_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e160_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:160,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b160_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:160,rule:25,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b160_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e161_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:161,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b161_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:161,rule:25,span,text_span});}
out.events=self.event(Event::Capture {site:32,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b161_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e162_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:162,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b162_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:162,rule:25,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b162_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e163_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:163,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b163_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b163_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e164_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e165_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e166_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e167_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e168_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e169_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e164_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:164,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b164_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:164,rule:26,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b164_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@interleave",true,"@interleave");
out
}
fn e165_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:165,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b165_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:165,rule:26,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b165_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e166_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:166,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b166_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:166,rule:26,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b166_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"profile",true,"profile");
out
}
fn e167_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:167,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b167_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:167,rule:26,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b167_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e168_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:168,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b168_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:168,rule:26,span,text_span});}
out.events=self.event(Event::Capture {site:33,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b168_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e169_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:169,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b169_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:169,rule:26,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b169_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e170_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:170,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b170_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b170_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e171_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e172_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e173_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e174_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e175_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e176_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e171_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:171,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b171_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:171,rule:27,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b171_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@backref",true,"@backref");
out
}
fn e172_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:172,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b172_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:172,rule:27,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b172_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e173_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:173,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b173_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:173,rule:27,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b173_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"name",true,"name");
out
}
fn e174_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:174,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b174_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:174,rule:27,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b174_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e175_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:175,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b175_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:175,rule:27,span,text_span});}
out.events=self.event(Event::Capture {site:34,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b175_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e176_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:176,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b176_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:176,rule:27,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b176_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e177_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:177,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b177_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b177_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e178_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e179_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e180_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e181_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e182_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e183_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e178_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:178,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b178_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:178,rule:28,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b178_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@scopeTree",true,"@scopeTree");
out
}
fn e179_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:179,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b179_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:179,rule:28,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b179_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e180_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:180,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b180_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:180,rule:28,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b180_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"mode",true,"mode");
out
}
fn e181_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:181,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b181_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:181,rule:28,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b181_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e182_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:182,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b182_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:182,rule:28,span,text_span});}
out.events=self.event(Event::Capture {site:35,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b182_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e183_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:183,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b183_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:183,rule:28,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b183_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e184_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:184,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b184_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b184_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e185_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e186_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e187_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e188_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e189_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e190_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e196_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e185_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:185,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b185_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:185,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b185_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@declares",true,"@declares");
out
}
fn e186_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:186,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b186_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:186,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b186_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e187_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:187,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b187_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:187,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b187_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"symbol",true,"symbol");
out
}
fn e188_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:188,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b188_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:188,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b188_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e189_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:189,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b189_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:189,rule:29,span,text_span});}
out.events=self.event(Event::Capture {site:36,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b189_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e190_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:190,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b190_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b190_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e191_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e191_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:191,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b191_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b191_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e192_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e193_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e194_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e195_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e192_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:192,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b192_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:192,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b192_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e193_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:193,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b193_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:193,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b193_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"description",true,"description");
out
}
fn e194_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:194,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b194_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:194,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b194_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e195_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:195,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b195_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:195,rule:29,span,text_span});}
out.events=self.event(Event::Capture {site:37,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b195_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e196_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:196,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b196_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:196,rule:29,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b196_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e197_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:197,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b197_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b197_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e198_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e199_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e200_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e201_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e202_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e203_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e198_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:198,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b198_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:198,rule:30,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b198_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@catalog",true,"@catalog");
out
}
fn e199_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:199,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b199_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:199,rule:30,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b199_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e200_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:200,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b200_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:200,rule:30,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b200_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"context",true,"context");
out
}
fn e201_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:201,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b201_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:201,rule:30,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b201_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e202_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:202,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b202_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:202,rule:30,span,text_span});}
out.events=self.event(Event::Capture {site:38,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b202_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e203_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:203,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b203_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:203,rule:30,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b203_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e204_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:204,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b204_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b204_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e205_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e205_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:205,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b205_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:205,rule:31,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b205_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@leftAssoc",true,"@leftAssoc");
out
}
fn e206_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:206,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b206_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b206_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e207_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e207_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:207,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b207_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:207,rule:32,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b207_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@rightAssoc",true,"@rightAssoc");
out
}
fn e208_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:208,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b208_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b208_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e209_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e209_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:209,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b209_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:209,rule:33,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b209_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@longestChoice",true,"@longestChoice");
out
}
fn e210_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:210,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b210_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b210_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e211_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e211_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:211,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b211_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:211,rule:34,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b211_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@predictiveChoice",true,"@predictiveChoice");
out
}
fn e212_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:212,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b212_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b212_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e213_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e214_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e215_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e216_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e217_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e218_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e213_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:213,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b213_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:213,rule:35,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b213_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@precedence",true,"@precedence");
out
}
fn e214_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:214,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b214_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:214,rule:35,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b214_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e215_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:215,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b215_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:215,rule:35,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b215_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"level",true,"level");
out
}
fn e216_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:216,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b216_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:216,rule:35,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b216_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e217_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:217,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b217_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:217,rule:35,span,text_span});}
out.events=self.event(Event::Capture {site:39,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b217_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"UNSIGNED_INTEGER");
out
}
fn e218_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:218,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b218_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:218,rule:35,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b218_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e219_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:219,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b219_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b219_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e220_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e221_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e222_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e223_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e220_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:220,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b220_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:220,rule:36,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b220_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@doc",true,"@doc");
out
}
fn e221_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:221,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b221_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:221,rule:36,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b221_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e222_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:222,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b222_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:222,rule:36,span,text_span});}
out.events=self.event(Event::Capture {site:40,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b222_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e223_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:223,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b223_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:223,rule:36,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b223_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e224_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:224,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b224_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b224_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e225_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e226_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e227_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e234_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e225_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:225,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b225_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:225,rule:37,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b225_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@recovery",true,"@recovery");
out
}
fn e226_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:226,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b226_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:226,rule:37,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b226_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e227_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:227,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b227_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b227_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e228_c(out.state);
out
}
fn e228_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:228,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b228_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b228_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e229_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e233_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e229_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:229,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b229_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b229_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e230_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e231_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e232_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e230_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:230,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b230_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:230,rule:37,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b230_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"sync",true,"sync");
out
}
fn e231_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:231,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b231_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:231,rule:37,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b231_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,"=");
out
}
fn e232_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:232,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b232_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:232,rule:37,span,text_span});}
out.events=self.event(Event::Capture {site:41,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b232_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e233_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:233,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b233_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:233,rule:37,span,text_span});}
out.events=self.event(Event::Capture {site:42,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b233_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e234_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:234,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b234_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:234,rule:37,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b234_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e235_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:235,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b235_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b235_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e236_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e236_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:236,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b236_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:236,rule:38,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b236_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@skip",true,"@skip");
out
}
fn e237_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:237,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b237_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b237_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e238_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e239_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e240_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e241_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e245_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e238_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:238,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b238_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:238,rule:39,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b238_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@commonField",true,"@commonField");
out
}
fn e239_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:239,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b239_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:239,rule:39,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b239_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e240_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:240,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b240_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:240,rule:39,span,text_span});}
out.events=self.event(Event::Capture {site:43,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b240_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e241_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:241,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b241_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b241_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e242_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e242_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:242,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b242_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b242_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e243_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e244_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e243_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:243,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b243_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:243,rule:39,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b243_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e244_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:244,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b244_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:244,rule:39,span,text_span});}
out.events=self.event(Event::Capture {site:44,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b244_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e245_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:245,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b245_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:245,rule:39,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b245_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e246_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:246,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b246_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b246_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e247_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e247_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:247,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b247_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:247,rule:40,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b247_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@enum",true,"@enum");
out
}
fn e248_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:248,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b248_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b248_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e249_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e250_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e249_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:249,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b249_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:249,rule:41,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b249_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,"@");
out
}
fn e250_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:250,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b250_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:250,rule:41,span,text_span});}
out.events=self.event(Event::Capture {site:45,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b250_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e251_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:251,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b251_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b251_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e252_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e253_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e252_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:252,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b252_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:46,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b252_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r43_c(out.state);
out
}
fn e253_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:253,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b253_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b253_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e254_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e254_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:254,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b254_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b254_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e255_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e256_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e255_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:255,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b255_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:255,rule:42,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b255_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"|",true,"|");
out
}
fn e256_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:256,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b256_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:47,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b256_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r43_c(out.state);
out
}
fn e257_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:257,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b257_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b257_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e258_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e258_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:258,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b258_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b258_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e259_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:48,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e259_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:259,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b259_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b259_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r44_c(out.state);
out
}
fn e260_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:260,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b260_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b260_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e261_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e263_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e264_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e266_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e268_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e270_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e261_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:261,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b261_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b261_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e262_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e262_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:262,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b262_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:49,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b262_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r56_c(out.state);
out
}
fn e263_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:263,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b263_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:50,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b263_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r49_c(out.state);
out
}
fn e264_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:264,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b264_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b264_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e265_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e265_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:265,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b265_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b265_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r45_c(out.state);
out
}
fn e266_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:266,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b266_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b266_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e267_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e267_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:267,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b267_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b267_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r47_c(out.state);
out
}
fn e268_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:268,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b268_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b268_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e269_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e269_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:269,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b269_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b269_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r46_c(out.state);
out
}
fn e270_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:270,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b270_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b270_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e271_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e271_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:271,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b271_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b271_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e272_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e273_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e272_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:272,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b272_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:272,rule:44,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b272_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,"@");
out
}
fn e273_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:273,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b273_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:273,rule:44,span,text_span});}
out.events=self.event(Event::Capture {site:51,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b273_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e274_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:274,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b274_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b274_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;
if !out.state.invert {let predicted=match self.input.cp_at(out.state.begin().position::<false>()).map(|(c,_)|c) {
Some('+')=>0usize,
Some('?')=>1usize,
Some('*')=>2usize,
_=>usize::MAX,};let candidate=match predicted {
0=>Some(self.e275_c(out.state.begin())),
1=>Some(self.e276_c(out.state.begin())),
2=>Some(self.e277_c(out.state.begin())),
_=>None,};if let Some(mut child)=candidate {if child.ok {
if predicted>0 {let d=self.diag.fail(out.state.begin().position::<false>(),"+");diagnostic=self.diag.join(diagnostic,d);}
if predicted>1 {let d=self.diag.fail(out.state.begin().position::<false>(),"?");diagnostic=self.diag.join(diagnostic,d);}
if predicted>2 {let d=self.diag.fail(out.state.begin().position::<false>(),"*");diagnostic=self.diag.join(diagnostic,d);}
child.state=out.state.commit(child.state);child.diag=self.diag.join(diagnostic,child.diag);break 'choice child;}}self.restore(mark); }
self.restore(mark);let mut child=self.e275_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e276_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e277_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e275_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let mut out=self.b275_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:275,rule:45,span,text_span});}
} else {self.restore(mark);}
out
}
fn b275_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"+",true,"+");
out
}
fn e276_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let mut out=self.b276_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:276,rule:45,span,text_span});}
} else {self.restore(mark);}
out
}
fn b276_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"?",true,"?");
out
}
fn e277_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let mut out=self.b277_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:277,rule:45,span,text_span});}
} else {self.restore(mark);}
out
}
fn b277_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"*",true,"*");
out
}
fn e278_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:278,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b278_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b278_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e279_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e280_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e279_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:279,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b279_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:279,rule:46,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b279_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"%",true,"%");
out
}
fn e280_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:280,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b280_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b280_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r49_c(out.state);
out
}
fn e281_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:281,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b281_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b281_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e282_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e283_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e284_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e289_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e282_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:282,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b282_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:282,rule:47,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b282_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,"{");
out
}
fn e283_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:283,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b283_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b283_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r48_c(out.state);
out
}
fn e284_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:284,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b284_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b284_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e285_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e285_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:285,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b285_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b285_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e286_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e287_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e286_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:286,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b286_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:286,rule:47,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b286_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,",");
out
}
fn e287_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:287,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b287_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b287_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e288_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e288_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:288,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b288_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b288_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r48_c(out.state);
out
}
fn e289_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:289,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b289_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:289,rule:47,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b289_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,"}");
out
}
fn e290_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:290,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b290_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b290_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e291_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e292_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e291_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:291,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b291_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:291,rule:48,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b291_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"DIGIT");
out
}
fn e292_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:292,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b292_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b292_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e293_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e293_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:293,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b293_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:293,rule:48,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b293_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"DIGIT");
out
}
fn e294_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:294,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b294_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b294_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag(0);let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e295_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e296_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e297_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e298_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e299_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e300_c(out.state.begin());diagnostic=self.diag.join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e295_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:295,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b295_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b295_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r50_c(out.state);
out
}
fn e296_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:296,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b296_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b296_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r51_c(out.state);
out
}
fn e297_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:297,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b297_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b297_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r52_c(out.state);
out
}
fn e298_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:298,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b298_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b298_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r53_c(out.state);
out
}
fn e299_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:299,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b299_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b299_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r54_c(out.state);
out
}
fn e300_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:300,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b300_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b300_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r55_c(out.state);
out
}
fn e301_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:301,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b301_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b301_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e302_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e303_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e304_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e302_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:302,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b302_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:302,rule:50,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b302_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e303_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:303,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b303_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:52,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b303_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e304_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:304,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b304_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:304,rule:50,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b304_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e305_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:305,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b305_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b305_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e306_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e307_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e308_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e306_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:306,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b306_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:306,rule:51,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b306_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,"[");
out
}
fn e307_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:307,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b307_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:53,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b307_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e308_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:308,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b308_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:308,rule:51,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b308_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,"]");
out
}
fn e309_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:309,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b309_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b309_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e310_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e311_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e312_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e310_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:310,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b310_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:310,rule:52,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b310_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,"{");
out
}
fn e311_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:311,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b311_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:54,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b311_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e312_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:312,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b312_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:312,rule:52,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b312_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,"}");
out
}
fn e313_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:313,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b313_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b313_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e314_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e315_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e316_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e317_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e314_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:314,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b314_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:314,rule:53,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b314_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ERROR",true,"ERROR");
out
}
fn e315_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:315,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b315_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:315,rule:53,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b315_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e316_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:316,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b316_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:316,rule:53,span,text_span});}
out.events=self.event(Event::Capture {site:55,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b316_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e317_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:317,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b317_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:317,rule:53,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b317_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e318_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:318,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b318_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b318_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e319_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e319_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:319,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b319_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:319,rule:54,span,text_span});}
out.events=self.event(Event::Capture {site:56,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b319_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING");
out
}
fn e320_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:320,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b320_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b320_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e321_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e325_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e321_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:321,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b321_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b321_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e322_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e322_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:322,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b322_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b322_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e323_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e324_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e323_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:323,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b323_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:323,rule:55,span,text_span});}
out.events=self.event(Event::Capture {site:57,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b323_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e324_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:324,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b324_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:324,rule:55,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b324_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,".");
out
}
fn e325_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:325,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b325_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:325,rule:55,span,text_span});}
out.events=self.event(Event::Capture {site:58,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b325_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e326_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:326,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b326_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b326_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e327_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e328_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e329_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e330_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e327_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:327,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b327_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:327,rule:56,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b327_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@typeof",true,"@typeof");
out
}
fn e328_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:328,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b328_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:328,rule:56,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b328_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,"(");
out
}
fn e329_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:329,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b329_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:329,rule:56,span,text_span});}
out.events=self.event(Event::Capture {site:59,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b329_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e330_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:330,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b330_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:330,rule:56,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b330_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,")");
out
}
fn e331_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:331,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b331_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b331_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e332_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e333_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e332_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:332,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b332_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:332,rule:57,span,text_span});}
out.events=self.event(Event::Capture {site:60,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b332_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e333_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:333,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b333_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b333_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e334_c(out.state);out.diag=self.diag.join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag.fail(state.position::<false>(),"expression");out.diag=self.diag.join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e334_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:334,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b334_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b334_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e335_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e336_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e335_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:335,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b335_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:335,rule:57,span,text_span});}
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b335_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,".");
out
}
fn e336_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:336,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b336_c(state);
if out.ok {
if let Event::Token{span,text_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:336,rule:57,span,text_span});}
out.events=self.event(Event::Capture {site:61,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b336_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER");
out
}
fn e337_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:337,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b337_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b337_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e338_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e338_c(&mut self,state:State)->Step {
if let Some(limit)=self.limit(state) {return limit;}
let mark=self.mark();
let key=Key {expression:338,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
let mut out=self.b338_c(state);
if out.ok {
} else {self.restore(mark);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}
out
}
fn b338_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r42_c(out.state);
out
}
fn token_0<const MATCH:bool>(&mut self,state:State,label:&'static str)->Step {let _=label;
self.identifier::<MATCH>(state,label)
}
fn token_1<const MATCH:bool>(&mut self,state:State,label:&'static str)->Step {let _=label;
self.quoted::<MATCH>(state,'\'',label)
}
fn token_2<const MATCH:bool>(&mut self,state:State,label:&'static str)->Step {let _=label;
self.identifier::<MATCH>(state,label)
}
fn token_3<const MATCH:bool>(&mut self,state:State,label:&'static str)->Step {let _=label;
self.number::<MATCH>(state,label)
}
fn token_4<const MATCH:bool>(&mut self,state:State,label:&'static str)->Step {let _=label;
let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|c.is_ascii_digit()).map(|(_,n)|n);self.primitive::<MATCH>(state,length,label)
}
fn trivia_0<const MATCH:bool>(&mut self,state:State)->Step {
let mut out=Step::yes(state);loop {let before=out.state.position::<MATCH>();
{let start=out.state.position::<MATCH>();let mut p=start;
while let Some((c,n))=self.input.cp_at(p) {if ![9, 10, 11, 12, 13, 32].contains(&(c as u64)) {break;}p+=n;}
if p>start {out.state.advance::<MATCH>(p-start);continue;}
}
{let start=out.state.position::<MATCH>();let mut p=start;
if self.input.starts_with(p,"//") {p+=2;while let Some((c,n))=self.input.cp_at(p) {if matches!(c,'\n'|'\r') {break;}p+=n;}let end=if self.input.starts_with(p,"\r\n") {p+2} else {self.input.cp_at(p).map_or(p,|(_,n)|p+n)};out.state.advance::<MATCH>(p-start);if !MATCH {out.state.matched=end;}continue;}
}
{let start=out.state.position::<MATCH>();let mut p=start;
if self.input.starts_with(p,"/*") {let body=p+2;if let Some(n)=self.text[body..].find("*/") {p=body+n+2;out.state.advance::<MATCH>(p-start);continue;}}
}
if out.state.position::<MATCH>()==before {break;}}
out}
}
