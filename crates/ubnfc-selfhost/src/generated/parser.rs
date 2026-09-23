//! IR の expression/rule ごとの静的関数。c=consumed, m=matchOnly。
#![allow(dead_code, unused_mut)]
use super::api::*;
use super::runtime::*;
#[allow(unused_imports)]
use super::rt::{diag::Diag,memo::Key};
pub fn parse(text:&str)->ParseResult {parse_with_options(text,ParseOptions::default())}
pub fn parse_with_options(text:&str,options:ParseOptions)->ParseResult {parse_with_scanner(text,options,&mut RejectExtern)}
pub fn parse_with_scanner(text:&str,options:ParseOptions,scanner:&mut dyn TokenScanner)->ParseResult {
// 診断は const DIAG で単相化する（D-027）。要求されていれば診断 session で 1 度だけ走り、
// そうでなければ fast session で走って、診断が要る結果のときだけ診断 session で解析し直す（D-023）。
if options.diagnostics {let mut parser=Session::<true>::new(text,options,scanner);
let step=parser.r0_c(State::default());
return parser.finish(step);}
let mut parser=Session::<false>::new(text,options,&mut *scanner);
let step=parser.r0_c(State::default());
let (result,escalate)=parser.finish_checked(step);
if !escalate {return result;}
let options=ParseOptions{diagnostics:true,..options};
let mut parser=Session::<true>::new(text,options,scanner);
let step=parser.r0_c(State::default());
parser.finish(step)
}
fn entry_step<const DIAG: bool>(parser:&mut Session<'_,DIAG>,grammar:&str,entry:Option<&str>)->Result<Step,&'static str> {
Ok(match (grammar,entry) {
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
_=>return Err("unsupported grammar or entry"),})}
pub fn parse_entry_with_options(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions)->Result<ParseResult,&'static str> {
let mut scanner=RejectExtern;
if options.diagnostics {let mut parser=Session::<true>::new(text,options,&mut scanner);
let step=entry_step(&mut parser,grammar,entry)?;
return Ok(parser.finish(step));}
let mut parser=Session::<false>::new(text,options,&mut scanner);
let step=entry_step(&mut parser,grammar,entry)?;
let (result,escalate)=parser.finish_checked(step);
if !escalate {return Ok(result);}
let options=ParseOptions{diagnostics:true,..options};
let mut parser=Session::<true>::new(text,options,&mut scanner);
let step=entry_step(&mut parser,grammar,entry)?;
Ok(parser.finish(step))}
impl<const DIAG: bool> Session<'_, DIAG> {
fn r0_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e0_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:0,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["UBNFFileParser"]);}
self.depth-=1;out.diag=self.diag_rule(0,out.diag);out
}
fn r1_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e3_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:1,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@import'", "'from'"]);}
self.depth-=1;out.diag=self.diag_rule(1,out.diag);out
}
fn r2_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e8_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:2,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'grammar'", "'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(2,out.diag);out
}
fn r3_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e21_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:3,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@'", "':'"]);}
self.depth-=1;out.diag=self.diag_rule(3,out.diag);out
}
fn r4_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e26_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:4,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(4,out.diag);out
}
fn r5_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e29_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:5,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["StringSettingValueParser"]);}
self.depth-=1;out.diag=self.diag_rule(5,out.diag);out
}
fn r6_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e31_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:6,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(6,out.diag);out
}
fn r7_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e36_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:7,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
self.depth-=1;out.diag=self.diag_rule(7,out.diag);out
}
fn r8_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e40_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:8,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'token'", "'='"]);}
self.depth-=1;out.diag=self.diag_rule(8,out.diag);out
}
fn r9_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e45_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:9,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'", "'('", "')'", "'NEGATION'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'", "'CHAR_RANGE'", "','", "'CI'", "'REGEX'", "'ANY'", "'EOF'", "'EMPTY'"]);}
self.depth-=1;out.diag=self.diag_rule(9,out.diag);out
}
fn r10_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e57_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:10,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(10,out.diag);out
}
fn r11_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e62_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:11,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(11,out.diag);out
}
fn r12_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e67_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:12,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(12,out.diag);out
}
fn r13_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e72_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:13,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(13,out.diag);out
}
fn r14_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e77_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:14,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'", "'('", "','", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(14,out.diag);out
}
fn r15_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e84_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:15,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CI'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(15,out.diag);out
}
fn r16_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e89_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:16,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'REGEX'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(16,out.diag);out
}
fn r17_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e94_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:17,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ANY'"]);}
self.depth-=1;out.diag=self.diag_rule(17,out.diag);out
}
fn r18_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e96_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:18,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'EOF'"]);}
self.depth-=1;out.diag=self.diag_rule(18,out.diag);out
}
fn r19_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e98_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:19,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'EMPTY'"]);}
self.depth-=1;out.diag=self.diag_rule(19,out.diag);out
}
fn r20_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e100_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:20,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'::='", "';'"]);}
self.depth-=1;out.diag=self.diag_rule(20,out.diag);out
}
fn r21_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e107_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:21,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@root'", "'@mapping'", "'('", "')'", "'@eval'", "'@whitespace'", "'@interleave'", "'profile'", "'='", "'@backref'", "'name'", "'@scopeTree'", "'mode'", "'@declares'", "'symbol'", "'@catalog'", "'context'", "'@leftAssoc'", "'@rightAssoc'", "'@longestChoice'", "'@predictiveChoice'", "'@precedence'", "'level'", "'@doc'", "'@recovery'", "'@skip'", "'@commonField'", "'@enum'", "'@'"]);}
self.depth-=1;out.diag=self.diag_rule(21,out.diag);out
}
fn r22_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e128_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:22,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@root'"]);}
self.depth-=1;out.diag=self.diag_rule(22,out.diag);out
}
fn r23_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e130_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:23,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@mapping'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(23,out.diag);out
}
fn r24_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e147_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:24,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@eval'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(24,out.diag);out
}
fn r25_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e156_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:25,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@whitespace'"]);}
self.depth-=1;out.diag=self.diag_rule(25,out.diag);out
}
fn r26_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e163_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:26,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@interleave'", "'('", "'profile'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(26,out.diag);out
}
fn r27_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e170_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:27,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@backref'", "'('", "'name'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(27,out.diag);out
}
fn r28_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e177_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:28,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@scopeTree'", "'('", "'mode'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(28,out.diag);out
}
fn r29_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e184_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:29,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@declares'", "'('", "'symbol'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(29,out.diag);out
}
fn r30_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e197_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:30,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@catalog'", "'('", "'context'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(30,out.diag);out
}
fn r31_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e204_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:31,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@leftAssoc'"]);}
self.depth-=1;out.diag=self.diag_rule(31,out.diag);out
}
fn r32_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e206_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:32,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@rightAssoc'"]);}
self.depth-=1;out.diag=self.diag_rule(32,out.diag);out
}
fn r33_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e208_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:33,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@longestChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(33,out.diag);out
}
fn r34_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e210_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:34,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@predictiveChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(34,out.diag);out
}
fn r35_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e212_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:35,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@precedence'", "'('", "'level'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(35,out.diag);out
}
fn r36_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e219_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:36,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@doc'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(36,out.diag);out
}
fn r37_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e224_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:37,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@recovery'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(37,out.diag);out
}
fn r38_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e235_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:38,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@skip'"]);}
self.depth-=1;out.diag=self.diag_rule(38,out.diag);out
}
fn r39_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e237_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:39,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@commonField'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(39,out.diag);out
}
fn r40_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e246_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:40,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@enum'"]);}
self.depth-=1;out.diag=self.diag_rule(40,out.diag);out
}
fn r41_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e248_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:41,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
self.depth-=1;out.diag=self.diag_rule(41,out.diag);out
}
fn r42_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e251_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:42,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["ChoiceBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(42,out.diag);out
}
fn r43_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e257_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:43,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["SequenceBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(43,out.diag);out
}
fn r44_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e260_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:44,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["AnnotatedElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(44,out.diag);out
}
fn r45_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e274_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:45,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'+'", "'?'", "'*'"]);}
self.depth-=1;out.diag=self.diag_rule(45,out.diag);out
}
fn r46_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e278_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:46,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
self.depth-=1;out.diag=self.diag_rule(46,out.diag);out
}
fn r47_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e281_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:47,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(47,out.diag);out
}
fn r48_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e290_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:48,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["DigitsParser"]);}
self.depth-=1;out.diag=self.diag_rule(48,out.diag);out
}
fn r49_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e294_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:49,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'['", "']'", "'{'", "'}'", "'ERROR'"]);}
self.depth-=1;out.diag=self.diag_rule(49,out.diag);out
}
fn r50_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e301_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:50,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(50,out.diag);out
}
fn r51_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e305_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:51,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'['", "']'"]);}
self.depth-=1;out.diag=self.diag_rule(51,out.diag);out
}
fn r52_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e309_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:52,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(52,out.diag);out
}
fn r53_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e313_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:53,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ERROR'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(53,out.diag);out
}
fn r54_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e318_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:54,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["TerminalElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(54,out.diag);out
}
fn r55_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e320_c(state);
if out.ok {
let mut scope_sites=self.take_scope();self.scope_captures(out.events,&[58],&mut scope_sites);for &(site,span) in &scope_sites { let name=self.text(span);let offset_cp=self.cp(span[0]);let len_cp=self.cp(span[1])-offset_cp;
if site==58 {let mut targets=self.take_scope();self.scope_captures(out.events,&[58],&mut targets);if let Some((_,target))=targets.first() {let expected=self.text(*target);if expected!=name {self.effect(ScanEffect::Diagnostic{message:format!("back-reference mismatch: expected '{expected}' but got '{name}'"),offset_cp,len_cp,severity:Severity::Error});}}self.give_scope(targets);}
let _=(&name,offset_cp,len_cp); }
self.give_scope(scope_sites);
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:55,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["QuantifiedRefParser"]);}
self.depth-=1;out.diag=self.diag_rule(55,out.diag);out
}
fn r56_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e326_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:56,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@typeof'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(56,out.diag);out
}
fn r57_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e331_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:57,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["DottedIdentifierParser"]);}
self.depth-=1;out.diag=self.diag_rule(57,out.diag);out
}
fn r58_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e337_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:58,span:[state.consumed,end],child:out.events});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["RuleBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(58,out.diag);out
}
fn e0_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:0,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b0_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b1_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'grammar'", "'{'", "'}'", "__CaptureSite"]);}
out
}
fn b1_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e2_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:0,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e2_c(&mut self,state:State)->Step {
let mut out=self.b2_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b2_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r2_c(out.state);
out
}
fn e3_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:3,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b3_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b4_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:4,rule:1,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@import'"]);}
out
}
fn b4_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@import",true,20,"@import");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@import'");}
out
}
fn e5_c(&mut self,state:State)->Step {
let mut out=self.b5_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:5,rule:1,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:1,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b5_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e6_c(&mut self,state:State)->Step {
let mut out=self.b6_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:6,rule:1,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'from'"]);}
out
}
fn b6_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"from",true,49,"from");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'from'");}
out
}
fn e7_c(&mut self,state:State)->Step {
let mut out=self.b7_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:7,rule:1,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:2,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b7_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e8_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:8,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b8_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b9_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:9,rule:2,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'grammar'"]);}
out
}
fn b9_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"grammar",true,50,"grammar");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'grammar'");}
out
}
fn e10_c(&mut self,state:State)->Step {
let mut out=self.b10_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:10,rule:2,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:3,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b10_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e11_c(&mut self,state:State)->Step {
let mut out=self.b11_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:11,rule:2,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b11_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,59,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e12_c(&mut self,state:State)->Step {
let mut out=self.b12_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'@import'", "'from'", "__CaptureSite"]);}
out
}
fn b12_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e13_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:4,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e13_c(&mut self,state:State)->Step {
let mut out=self.b13_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b13_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r1_c(out.state);
out
}
fn e14_c(&mut self,state:State)->Step {
let mut out=self.b14_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'", "'@'", "__CaptureSite"]);}
out
}
fn b14_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e15_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:5,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e15_c(&mut self,state:State)->Step {
let mut out=self.b15_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b15_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r3_c(out.state);
out
}
fn e16_c(&mut self,state:State)->Step {
let mut out=self.b16_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'token'", "__CaptureSite"]);}
out
}
fn b16_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e17_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:6,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e17_c(&mut self,state:State)->Step {
let mut out=self.b17_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b17_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r8_c(out.state);
out
}
fn e18_c(&mut self,state:State)->Step {
let mut out=self.b18_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'::='", "';'", "__CaptureSite"]);}
out
}
fn b18_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e19_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:7,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e19_c(&mut self,state:State)->Step {
let mut out=self.b19_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b19_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r20_c(out.state);
out
}
fn e20_c(&mut self,state:State)->Step {
let mut out=self.b20_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:20,rule:2,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b20_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,61,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e21_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:21,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b21_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b22_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:22,rule:3,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b22_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,12,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e23_c(&mut self,state:State)->Step {
let mut out=self.b23_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:23,rule:3,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:8,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b23_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e24_c(&mut self,state:State)->Step {
let mut out=self.b24_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:24,rule:3,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b24_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,7,":");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"':'");}
out
}
fn e25_c(&mut self,state:State)->Step {
let mut out=self.b25_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:9,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b25_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r4_c(out.state);
out
}
fn e26_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:26,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b26_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b26_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65..=90|95|97..=122))) {self.guard_e27_c(out.state.begin())} else {self.e27_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e28_c(out.state.begin())} else {self.e28_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e27_c(&mut self,state:State)->Step {
let mut out=self.b27_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b27_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r5_c(out.state);
out
}
fn e28_c(&mut self,state:State)->Step {
let mut out=self.b28_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b28_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r6_c(out.state);
out
}
fn e29_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:29,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b29_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b30_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:10,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b30_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r57_c(out.state);
out
}
fn e31_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:31,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b31_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b32_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:32,rule:6,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b32_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,59,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e33_c(&mut self,state:State)->Step {
let mut out=self.b33_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'", "__CaptureSite"]);}
out
}
fn b33_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e34_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:11,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e34_c(&mut self,state:State)->Step {
let mut out=self.b34_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b34_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e35_c(&mut self,state:State)->Step {
let mut out=self.b35_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:35,rule:6,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b35_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,61,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e36_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:36,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b36_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b37_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:37,rule:7,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:12,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b37_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e38_c(&mut self,state:State)->Step {
let mut out=self.b38_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:38,rule:7,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b38_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,7,":");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"':'");}
out
}
fn e39_c(&mut self,state:State)->Step {
let mut out=self.b39_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:39,rule:7,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:13,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b39_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e40_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:40,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b40_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b41_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:41,rule:8,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'token'"]);}
out
}
fn b41_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"token",true,58,"token");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'token'");}
out
}
fn e42_c(&mut self,state:State)->Step {
let mut out=self.b42_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:42,rule:8,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:14,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b42_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e43_c(&mut self,state:State)->Step {
let mut out=self.b43_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:43,rule:8,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b43_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e44_c(&mut self,state:State)->Step {
let mut out=self.b44_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:15,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b44_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r9_c(out.state);
out
}
fn e45_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:45,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b45_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b45_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,85))) {self.guard_e46_c(out.state.begin())} else {self.e46_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e47_c(out.state.begin())} else {self.e47_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,76))) {self.guard_e48_c(out.state.begin())} else {self.e48_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e49_c(out.state.begin())} else {self.e49_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e50_c(out.state.begin())} else {self.e50_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e51_c(out.state.begin())} else {self.e51_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,82))) {self.guard_e52_c(out.state.begin())} else {self.e52_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65))) {self.guard_e53_c(out.state.begin())} else {self.e53_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e54_c(out.state.begin())} else {self.e54_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e55_c(out.state.begin())} else {self.e55_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65..=90|95|97..=122))) {self.guard_e56_c(out.state.begin())} else {self.e56_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e46_c(&mut self,state:State)->Step {
let mut out=self.b46_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b46_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e47_c(&mut self,state:State)->Step {
let mut out=self.b47_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b47_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r11_c(out.state);
out
}
fn e48_c(&mut self,state:State)->Step {
let mut out=self.b48_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b48_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r12_c(out.state);
out
}
fn e49_c(&mut self,state:State)->Step {
let mut out=self.b49_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b49_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r13_c(out.state);
out
}
fn e50_c(&mut self,state:State)->Step {
let mut out=self.b50_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b50_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r14_c(out.state);
out
}
fn e51_c(&mut self,state:State)->Step {
let mut out=self.b51_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b51_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r15_c(out.state);
out
}
fn e52_c(&mut self,state:State)->Step {
let mut out=self.b52_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b52_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r16_c(out.state);
out
}
fn e53_c(&mut self,state:State)->Step {
let mut out=self.b53_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b53_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r17_c(out.state);
out
}
fn e54_c(&mut self,state:State)->Step {
let mut out=self.b54_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b54_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r18_c(out.state);
out
}
fn e55_c(&mut self,state:State)->Step {
let mut out=self.b55_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b55_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r19_c(out.state);
out
}
fn e56_c(&mut self,state:State)->Step {
let mut out=self.b56_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b56_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r57_c(out.state);
out
}
fn e57_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:57,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b57_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b58_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:58,rule:10,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'"]);}
out
}
fn b58_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"UNTIL",true,44,"UNTIL");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'UNTIL'");}
out
}
fn e59_c(&mut self,state:State)->Step {
let mut out=self.b59_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:59,rule:10,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b59_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e60_c(&mut self,state:State)->Step {
let mut out=self.b60_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:60,rule:10,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:16,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b60_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e61_c(&mut self,state:State)->Step {
let mut out=self.b61_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:61,rule:10,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b61_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e62_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:62,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b62_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b63_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:63,rule:11,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'"]);}
out
}
fn b63_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATION",true,41,"NEGATION");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATION'");}
out
}
fn e64_c(&mut self,state:State)->Step {
let mut out=self.b64_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:64,rule:11,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b64_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e65_c(&mut self,state:State)->Step {
let mut out=self.b65_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:65,rule:11,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:17,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b65_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e66_c(&mut self,state:State)->Step {
let mut out=self.b66_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:66,rule:11,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b66_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e67_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:67,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b67_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b68_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:68,rule:12,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'"]);}
out
}
fn b68_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LOOKAHEAD",true,40,"LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'LOOKAHEAD'");}
out
}
fn e69_c(&mut self,state:State)->Step {
let mut out=self.b69_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:69,rule:12,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b69_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e70_c(&mut self,state:State)->Step {
let mut out=self.b70_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:70,rule:12,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:18,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b70_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e71_c(&mut self,state:State)->Step {
let mut out=self.b71_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:71,rule:12,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b71_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e72_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:72,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b72_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b73_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:73,rule:13,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b73_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATIVE_LOOKAHEAD",true,42,"NEGATIVE_LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATIVE_LOOKAHEAD'");}
out
}
fn e74_c(&mut self,state:State)->Step {
let mut out=self.b74_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:74,rule:13,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b74_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e75_c(&mut self,state:State)->Step {
let mut out=self.b75_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:75,rule:13,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:19,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b75_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e76_c(&mut self,state:State)->Step {
let mut out=self.b76_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:76,rule:13,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b76_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e77_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:77,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b77_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b78_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:78,rule:14,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'"]);}
out
}
fn b78_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CHAR_RANGE",true,35,"CHAR_RANGE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CHAR_RANGE'");}
out
}
fn e79_c(&mut self,state:State)->Step {
let mut out=self.b79_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:79,rule:14,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b79_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e80_c(&mut self,state:State)->Step {
let mut out=self.b80_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:80,rule:14,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:20,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b80_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e81_c(&mut self,state:State)->Step {
let mut out=self.b81_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:81,rule:14,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b81_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e82_c(&mut self,state:State)->Step {
let mut out=self.b82_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:82,rule:14,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:21,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b82_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e83_c(&mut self,state:State)->Step {
let mut out=self.b83_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:83,rule:14,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b83_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e84_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:84,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b84_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b85_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:85,rule:15,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'CI'"]);}
out
}
fn b85_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CI",true,36,"CI");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CI'");}
out
}
fn e86_c(&mut self,state:State)->Step {
let mut out=self.b86_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:86,rule:15,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b86_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e87_c(&mut self,state:State)->Step {
let mut out=self.b87_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:87,rule:15,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:22,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b87_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e88_c(&mut self,state:State)->Step {
let mut out=self.b88_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:88,rule:15,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b88_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e89_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:89,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b89_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b90_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:90,rule:16,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'REGEX'"]);}
out
}
fn b90_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"REGEX",true,43,"REGEX");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'REGEX'");}
out
}
fn e91_c(&mut self,state:State)->Step {
let mut out=self.b91_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:91,rule:16,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b91_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e92_c(&mut self,state:State)->Step {
let mut out=self.b92_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:92,rule:16,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:23,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b92_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e93_c(&mut self,state:State)->Step {
let mut out=self.b93_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:93,rule:16,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b93_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e94_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:94,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b94_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b95_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:95,rule:17,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'ANY'"]);}
out
}
fn b95_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ANY",true,34,"ANY");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ANY'");}
out
}
fn e96_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:96,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b96_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b97_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:97,rule:18,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'EOF'"]);}
out
}
fn b97_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EOF",true,38,"EOF");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'EOF'");}
out
}
fn e98_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:98,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b98_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b99_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:99,rule:19,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'EMPTY'"]);}
out
}
fn b99_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EMPTY",true,37,"EMPTY");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'EMPTY'");}
out
}
fn e100_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:100,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b100_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b101_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b101_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e102_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:24,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e102_c(&mut self,state:State)->Step {
let mut out=self.b102_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b102_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r21_c(out.state);
out
}
fn e103_c(&mut self,state:State)->Step {
let mut out=self.b103_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:103,rule:20,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:25,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b103_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e104_c(&mut self,state:State)->Step {
let mut out=self.b104_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:104,rule:20,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
out
}
fn b104_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"::=",true,8,"::=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'::='");}
out
}
fn e105_c(&mut self,state:State)->Step {
let mut out=self.b105_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:26,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b105_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e106_c(&mut self,state:State)->Step {
let mut out=self.b106_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:106,rule:20,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b106_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,";",true,9,";");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"';'");}
out
}
fn e107_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:107,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b107_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b107_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e108_c(out.state.begin())} else {self.e108_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e109_c(out.state.begin())} else {self.e109_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e110_c(out.state.begin())} else {self.e110_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e111_c(out.state.begin())} else {self.e111_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e112_c(out.state.begin())} else {self.e112_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e113_c(out.state.begin())} else {self.e113_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e114_c(out.state.begin())} else {self.e114_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e115_c(out.state.begin())} else {self.e115_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e116_c(out.state.begin())} else {self.e116_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e117_c(out.state.begin())} else {self.e117_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e118_c(out.state.begin())} else {self.e118_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e119_c(out.state.begin())} else {self.e119_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e120_c(out.state.begin())} else {self.e120_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e121_c(out.state.begin())} else {self.e121_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e122_c(out.state.begin())} else {self.e122_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e123_c(out.state.begin())} else {self.e123_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e124_c(out.state.begin())} else {self.e124_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e125_c(out.state.begin())} else {self.e125_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e126_c(out.state.begin())} else {self.e126_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e127_c(out.state.begin())} else {self.e127_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e108_c(&mut self,state:State)->Step {
let mut out=self.b108_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b108_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r22_c(out.state);
out
}
fn e109_c(&mut self,state:State)->Step {
let mut out=self.b109_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b109_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r23_c(out.state);
out
}
fn e110_c(&mut self,state:State)->Step {
let mut out=self.b110_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b110_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r24_c(out.state);
out
}
fn e111_c(&mut self,state:State)->Step {
let mut out=self.b111_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b111_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r25_c(out.state);
out
}
fn e112_c(&mut self,state:State)->Step {
let mut out=self.b112_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b112_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r26_c(out.state);
out
}
fn e113_c(&mut self,state:State)->Step {
let mut out=self.b113_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b113_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r27_c(out.state);
out
}
fn e114_c(&mut self,state:State)->Step {
let mut out=self.b114_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b114_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r28_c(out.state);
out
}
fn e115_c(&mut self,state:State)->Step {
let mut out=self.b115_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b115_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r29_c(out.state);
out
}
fn e116_c(&mut self,state:State)->Step {
let mut out=self.b116_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b116_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r30_c(out.state);
out
}
fn e117_c(&mut self,state:State)->Step {
let mut out=self.b117_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b117_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r31_c(out.state);
out
}
fn e118_c(&mut self,state:State)->Step {
let mut out=self.b118_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b118_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r32_c(out.state);
out
}
fn e119_c(&mut self,state:State)->Step {
let mut out=self.b119_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b119_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r33_c(out.state);
out
}
fn e120_c(&mut self,state:State)->Step {
let mut out=self.b120_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b120_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r34_c(out.state);
out
}
fn e121_c(&mut self,state:State)->Step {
let mut out=self.b121_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b121_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r35_c(out.state);
out
}
fn e122_c(&mut self,state:State)->Step {
let mut out=self.b122_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b122_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r36_c(out.state);
out
}
fn e123_c(&mut self,state:State)->Step {
let mut out=self.b123_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b123_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r37_c(out.state);
out
}
fn e124_c(&mut self,state:State)->Step {
let mut out=self.b124_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b124_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r38_c(out.state);
out
}
fn e125_c(&mut self,state:State)->Step {
let mut out=self.b125_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b125_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r39_c(out.state);
out
}
fn e126_c(&mut self,state:State)->Step {
let mut out=self.b126_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b126_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r40_c(out.state);
out
}
fn e127_c(&mut self,state:State)->Step {
let mut out=self.b127_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b127_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r41_c(out.state);
out
}
fn e128_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:128,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b128_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b129_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:129,rule:22,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@root'"]);}
out
}
fn b129_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@root",true,29,"@root");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@root'");}
out
}
fn e130_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:130,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b130_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b131_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:131,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@mapping'"]);}
out
}
fn b131_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@mapping",true,24,"@mapping");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@mapping'");}
out
}
fn e132_c(&mut self,state:State)->Step {
let mut out=self.b132_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:132,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b132_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e133_c(&mut self,state:State)->Step {
let mut out=self.b133_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:133,rule:23,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:27,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b133_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"CLASS_NAME","IdentifierParser");
out
}
fn e134_c(&mut self,state:State)->Step {
let mut out=self.b134_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'params'"]);}
out
}
fn b134_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e135_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e135_c(&mut self,state:State)->Step {
let mut out=self.b135_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'params'"]);}
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
let mut out=self.b136_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:136,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b136_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e137_c(&mut self,state:State)->Step {
let mut out=self.b137_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:137,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'params'"]);}
out
}
fn b137_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"params",true,54,"params");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'params'");}
out
}
fn e138_c(&mut self,state:State)->Step {
let mut out=self.b138_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:138,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b138_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e139_c(&mut self,state:State)->Step {
let mut out=self.b139_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:139,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b139_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,45,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e140_c(&mut self,state:State)->Step {
let mut out=self.b140_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:140,rule:23,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:28,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b140_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e141_c(&mut self,state:State)->Step {
let mut out=self.b141_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b141_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e142_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e142_c(&mut self,state:State)->Step {
let mut out=self.b142_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
let mut out=self.b143_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:143,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b143_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e144_c(&mut self,state:State)->Step {
let mut out=self.b144_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:144,rule:23,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:29,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b144_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e145_c(&mut self,state:State)->Step {
let mut out=self.b145_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:145,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b145_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,46,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e146_c(&mut self,state:State)->Step {
let mut out=self.b146_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:146,rule:23,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b146_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e147_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:147,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b147_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b148_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:148,rule:24,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@eval'"]);}
out
}
fn b148_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@eval",true,19,"@eval");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@eval'");}
out
}
fn e149_c(&mut self,state:State)->Step {
let mut out=self.b149_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:149,rule:24,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b149_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e150_c(&mut self,state:State)->Step {
let mut out=self.b150_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:30,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b150_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e151_c(&mut self,state:State)->Step {
let mut out=self.b151_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b151_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e152_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e152_c(&mut self,state:State)->Step {
let mut out=self.b152_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
let mut out=self.b153_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:153,rule:24,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b153_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e154_c(&mut self,state:State)->Step {
let mut out=self.b154_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:31,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b154_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e155_c(&mut self,state:State)->Step {
let mut out=self.b155_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:155,rule:24,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b155_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e156_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:156,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b156_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b157_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:157,rule:25,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@whitespace'"]);}
out
}
fn b157_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@whitespace",true,33,"@whitespace");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@whitespace'");}
out
}
fn e158_c(&mut self,state:State)->Step {
let mut out=self.b158_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
out
}
fn b158_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e159_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e159_c(&mut self,state:State)->Step {
let mut out=self.b159_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
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
let mut out=self.b160_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:160,rule:25,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b160_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e161_c(&mut self,state:State)->Step {
let mut out=self.b161_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:161,rule:25,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:32,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b161_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e162_c(&mut self,state:State)->Step {
let mut out=self.b162_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:162,rule:25,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b162_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e163_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:163,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b163_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b164_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:164,rule:26,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@interleave'"]);}
out
}
fn b164_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@interleave",true,21,"@interleave");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@interleave'");}
out
}
fn e165_c(&mut self,state:State)->Step {
let mut out=self.b165_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:165,rule:26,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b165_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e166_c(&mut self,state:State)->Step {
let mut out=self.b166_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:166,rule:26,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'profile'"]);}
out
}
fn b166_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"profile",true,55,"profile");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'profile'");}
out
}
fn e167_c(&mut self,state:State)->Step {
let mut out=self.b167_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:167,rule:26,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b167_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e168_c(&mut self,state:State)->Step {
let mut out=self.b168_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:168,rule:26,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:33,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b168_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e169_c(&mut self,state:State)->Step {
let mut out=self.b169_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:169,rule:26,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b169_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e170_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:170,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b170_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b171_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:171,rule:27,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@backref'"]);}
out
}
fn b171_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@backref",true,13,"@backref");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@backref'");}
out
}
fn e172_c(&mut self,state:State)->Step {
let mut out=self.b172_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:172,rule:27,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b172_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e173_c(&mut self,state:State)->Step {
let mut out=self.b173_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:173,rule:27,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'name'"]);}
out
}
fn b173_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"name",true,53,"name");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'name'");}
out
}
fn e174_c(&mut self,state:State)->Step {
let mut out=self.b174_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:174,rule:27,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b174_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e175_c(&mut self,state:State)->Step {
let mut out=self.b175_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:175,rule:27,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:34,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b175_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e176_c(&mut self,state:State)->Step {
let mut out=self.b176_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:176,rule:27,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b176_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e177_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:177,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b177_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b178_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:178,rule:28,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@scopeTree'"]);}
out
}
fn b178_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@scopeTree",true,30,"@scopeTree");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@scopeTree'");}
out
}
fn e179_c(&mut self,state:State)->Step {
let mut out=self.b179_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:179,rule:28,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b179_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e180_c(&mut self,state:State)->Step {
let mut out=self.b180_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:180,rule:28,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'mode'"]);}
out
}
fn b180_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"mode",true,52,"mode");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'mode'");}
out
}
fn e181_c(&mut self,state:State)->Step {
let mut out=self.b181_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:181,rule:28,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b181_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e182_c(&mut self,state:State)->Step {
let mut out=self.b182_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:182,rule:28,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:35,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b182_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e183_c(&mut self,state:State)->Step {
let mut out=self.b183_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:183,rule:28,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b183_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e184_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:184,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b184_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b185_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:185,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@declares'"]);}
out
}
fn b185_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@declares",true,16,"@declares");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@declares'");}
out
}
fn e186_c(&mut self,state:State)->Step {
let mut out=self.b186_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:186,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b186_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e187_c(&mut self,state:State)->Step {
let mut out=self.b187_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:187,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'symbol'"]);}
out
}
fn b187_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"symbol",true,56,"symbol");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'symbol'");}
out
}
fn e188_c(&mut self,state:State)->Step {
let mut out=self.b188_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:188,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b188_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e189_c(&mut self,state:State)->Step {
let mut out=self.b189_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:189,rule:29,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:36,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b189_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e190_c(&mut self,state:State)->Step {
let mut out=self.b190_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'description'"]);}
out
}
fn b190_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e191_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e191_c(&mut self,state:State)->Step {
let mut out=self.b191_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'description'"]);}
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
let mut out=self.b192_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:192,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b192_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e193_c(&mut self,state:State)->Step {
let mut out=self.b193_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:193,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'description'"]);}
out
}
fn b193_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"description",true,48,"description");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'description'");}
out
}
fn e194_c(&mut self,state:State)->Step {
let mut out=self.b194_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:194,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b194_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e195_c(&mut self,state:State)->Step {
let mut out=self.b195_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:195,rule:29,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:37,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b195_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e196_c(&mut self,state:State)->Step {
let mut out=self.b196_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:196,rule:29,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b196_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e197_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:197,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b197_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b198_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:198,rule:30,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@catalog'"]);}
out
}
fn b198_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@catalog",true,14,"@catalog");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@catalog'");}
out
}
fn e199_c(&mut self,state:State)->Step {
let mut out=self.b199_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:199,rule:30,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b199_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e200_c(&mut self,state:State)->Step {
let mut out=self.b200_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:200,rule:30,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'context'"]);}
out
}
fn b200_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"context",true,47,"context");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'context'");}
out
}
fn e201_c(&mut self,state:State)->Step {
let mut out=self.b201_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:201,rule:30,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b201_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e202_c(&mut self,state:State)->Step {
let mut out=self.b202_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:202,rule:30,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:38,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b202_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e203_c(&mut self,state:State)->Step {
let mut out=self.b203_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:203,rule:30,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b203_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e204_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:204,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b204_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b205_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:205,rule:31,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@leftAssoc'"]);}
out
}
fn b205_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@leftAssoc",true,22,"@leftAssoc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@leftAssoc'");}
out
}
fn e206_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:206,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b206_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b207_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:207,rule:32,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@rightAssoc'"]);}
out
}
fn b207_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@rightAssoc",true,28,"@rightAssoc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@rightAssoc'");}
out
}
fn e208_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:208,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b208_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b209_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:209,rule:33,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@longestChoice'"]);}
out
}
fn b209_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@longestChoice",true,23,"@longestChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@longestChoice'");}
out
}
fn e210_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:210,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b210_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b211_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:211,rule:34,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@predictiveChoice'"]);}
out
}
fn b211_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@predictiveChoice",true,26,"@predictiveChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@predictiveChoice'");}
out
}
fn e212_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:212,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b212_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b213_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:213,rule:35,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@precedence'"]);}
out
}
fn b213_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@precedence",true,25,"@precedence");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@precedence'");}
out
}
fn e214_c(&mut self,state:State)->Step {
let mut out=self.b214_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:214,rule:35,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b214_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e215_c(&mut self,state:State)->Step {
let mut out=self.b215_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:215,rule:35,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'level'"]);}
out
}
fn b215_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"level",true,51,"level");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'level'");}
out
}
fn e216_c(&mut self,state:State)->Step {
let mut out=self.b216_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:216,rule:35,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b216_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e217_c(&mut self,state:State)->Step {
let mut out=self.b217_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:217,rule:35,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:39,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["NumberParser", "__CaptureSite"]);}
out
}
fn b217_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"UNSIGNED_INTEGER","NumberParser");
out
}
fn e218_c(&mut self,state:State)->Step {
let mut out=self.b218_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:218,rule:35,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b218_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e219_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:219,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b219_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b220_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:220,rule:36,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@doc'"]);}
out
}
fn b220_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@doc",true,17,"@doc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@doc'");}
out
}
fn e221_c(&mut self,state:State)->Step {
let mut out=self.b221_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:221,rule:36,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b221_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e222_c(&mut self,state:State)->Step {
let mut out=self.b222_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:222,rule:36,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:40,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b222_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e223_c(&mut self,state:State)->Step {
let mut out=self.b223_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:223,rule:36,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b223_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e224_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:224,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b224_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b225_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:225,rule:37,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@recovery'"]);}
out
}
fn b225_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@recovery",true,27,"@recovery");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@recovery'");}
out
}
fn e226_c(&mut self,state:State)->Step {
let mut out=self.b226_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:226,rule:37,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b226_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e227_c(&mut self,state:State)->Step {
let mut out=self.b227_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b227_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e228_c(out.state);
out
}
fn e228_c(&mut self,state:State)->Step {
let mut out=self.b228_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b228_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,115))) {self.guard_e229_c(out.state.begin())} else {self.e229_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e233_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e229_c(&mut self,state:State)->Step {
let mut out=self.b229_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
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
let mut out=self.b230_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:230,rule:37,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'sync'"]);}
out
}
fn b230_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"sync",true,57,"sync");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'sync'");}
out
}
fn e231_c(&mut self,state:State)->Step {
let mut out=self.b231_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:231,rule:37,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b231_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e232_c(&mut self,state:State)->Step {
let mut out=self.b232_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:232,rule:37,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:41,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b232_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e233_c(&mut self,state:State)->Step {
let mut out=self.b233_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:233,rule:37,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:42,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b233_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e234_c(&mut self,state:State)->Step {
let mut out=self.b234_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:234,rule:37,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b234_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e235_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:235,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b235_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b236_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:236,rule:38,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@skip'"]);}
out
}
fn b236_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@skip",true,31,"@skip");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@skip'");}
out
}
fn e237_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:237,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b237_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b238_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:238,rule:39,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@commonField'"]);}
out
}
fn b238_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@commonField",true,15,"@commonField");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@commonField'");}
out
}
fn e239_c(&mut self,state:State)->Step {
let mut out=self.b239_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:239,rule:39,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b239_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e240_c(&mut self,state:State)->Step {
let mut out=self.b240_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:240,rule:39,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:43,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b240_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e241_c(&mut self,state:State)->Step {
let mut out=self.b241_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b241_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e242_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e242_c(&mut self,state:State)->Step {
let mut out=self.b242_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
let mut out=self.b243_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:243,rule:39,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b243_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e244_c(&mut self,state:State)->Step {
let mut out=self.b244_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:244,rule:39,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:44,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b244_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e245_c(&mut self,state:State)->Step {
let mut out=self.b245_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:245,rule:39,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b245_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e246_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:246,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b246_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b247_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:247,rule:40,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@enum'"]);}
out
}
fn b247_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@enum",true,18,"@enum");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@enum'");}
out
}
fn e248_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:248,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b248_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b249_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:249,rule:41,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b249_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,12,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e250_c(&mut self,state:State)->Step {
let mut out=self.b250_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:250,rule:41,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:45,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b250_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e251_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:251,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b251_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b252_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:46,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b252_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r43_c(out.state);
out
}
fn e253_c(&mut self,state:State)->Step {
let mut out=self.b253_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b253_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e254_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e254_c(&mut self,state:State)->Step {
let mut out=self.b254_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
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
let mut out=self.b255_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:255,rule:42,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b255_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"|",true,60,"|");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'|'");}
out
}
fn e256_c(&mut self,state:State)->Step {
let mut out=self.b256_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:47,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b256_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r43_c(out.state);
out
}
fn e257_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:257,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b257_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b258_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b258_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e259_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:48,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e259_c(&mut self,state:State)->Step {
let mut out=self.b259_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b259_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r44_c(out.state);
out
}
fn e260_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:260,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b260_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b261_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b261_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e262_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e262_c(&mut self,state:State)->Step {
let mut out=self.b262_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:49,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'@typeof'"]);}
out
}
fn b262_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r56_c(out.state);
out
}
fn e263_c(&mut self,state:State)->Step {
let mut out=self.b263_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:50,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b263_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r49_c(out.state);
out
}
fn e264_c(&mut self,state:State)->Step {
let mut out=self.b264_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'"]);}
out
}
fn b264_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e265_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e265_c(&mut self,state:State)->Step {
let mut out=self.b265_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b265_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r45_c(out.state);
out
}
fn e266_c(&mut self,state:State)->Step {
let mut out=self.b266_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b266_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e267_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e267_c(&mut self,state:State)->Step {
let mut out=self.b267_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b267_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r47_c(out.state);
out
}
fn e268_c(&mut self,state:State)->Step {
let mut out=self.b268_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
out
}
fn b268_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e269_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e269_c(&mut self,state:State)->Step {
let mut out=self.b269_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b269_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r46_c(out.state);
out
}
fn e270_c(&mut self,state:State)->Step {
let mut out=self.b270_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b270_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e271_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e271_c(&mut self,state:State)->Step {
let mut out=self.b271_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
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
let mut out=self.b272_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:272,rule:44,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b272_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,12,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e273_c(&mut self,state:State)->Step {
let mut out=self.b273_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:273,rule:44,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:51,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b273_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e274_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:274,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b274_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b274_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
if !out.state.invert {let predicted=match self.input.cp_at(out.state.begin().position::<false>()).map(|(c,_)|c) {
Some('+')=>0usize,
Some('?')=>1usize,
Some('*')=>2usize,
_=>usize::MAX,};let candidate=match predicted {
0=>Some(self.e275_c(out.state.begin())),
1=>Some(self.e276_c(out.state.begin())),
2=>Some(self.e277_c(out.state.begin())),
_=>None,};if let Some(mut child)=candidate {if child.ok {
if predicted>0 {let d=self.diag_fail(out.state.begin().position::<false>(),"+");diagnostic=self.diag_join(diagnostic,d);}
if predicted>1 {let d=self.diag_fail(out.state.begin().position::<false>(),"?");diagnostic=self.diag_join(diagnostic,d);}
if predicted>2 {let d=self.diag_fail(out.state.begin().position::<false>(),"*");diagnostic=self.diag_join(diagnostic,d);}
child.state=out.state.commit(child.state);child.diag=self.diag_join(diagnostic,child.diag);break 'choice child;}}self.restore(mark); }
self.restore(mark);let mut child=self.e275_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e276_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e277_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e275_c(&mut self,state:State)->Step {
let mut out=self.b275_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:275,rule:45,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'+'"]);}
out
}
fn b275_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"+",true,4,"+");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'+'");}
out
}
fn e276_c(&mut self,state:State)->Step {
let mut out=self.b276_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:276,rule:45,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'?'"]);}
out
}
fn b276_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"?",true,11,"?");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'?'");}
out
}
fn e277_c(&mut self,state:State)->Step {
let mut out=self.b277_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:277,rule:45,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'*'"]);}
out
}
fn b277_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"*",true,3,"*");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'*'");}
out
}
fn e278_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:278,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b278_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b279_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:279,rule:46,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
out
}
fn b279_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"%",true,0,"%");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'%'");}
out
}
fn e280_c(&mut self,state:State)->Step {
let mut out=self.b280_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b280_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r49_c(out.state);
out
}
fn e281_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:281,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b281_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b282_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:282,rule:47,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b282_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,59,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e283_c(&mut self,state:State)->Step {
let mut out=self.b283_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b283_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r48_c(out.state);
out
}
fn e284_c(&mut self,state:State)->Step {
let mut out=self.b284_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b284_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e285_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e285_c(&mut self,state:State)->Step {
let mut out=self.b285_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
let mut out=self.b286_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:286,rule:47,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b286_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e287_c(&mut self,state:State)->Step {
let mut out=self.b287_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b287_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e288_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e288_c(&mut self,state:State)->Step {
let mut out=self.b288_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b288_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r48_c(out.state);
out
}
fn e289_c(&mut self,state:State)->Step {
let mut out=self.b289_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:289,rule:47,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b289_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,61,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e290_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:290,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b290_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b291_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:291,rule:48,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["DIGITParser"]);}
out
}
fn b291_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"DIGIT","DIGITParser");
out
}
fn e292_c(&mut self,state:State)->Step {
let mut out=self.b292_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat"]);}
out
}
fn b292_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e293_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e293_c(&mut self,state:State)->Step {
let mut out=self.b293_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:293,rule:48,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["DIGITParser"]);}
out
}
fn b293_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"DIGIT","DIGITParser");
out
}
fn e294_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:294,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b294_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b294_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,40))) {self.guard_e295_c(out.state.begin())} else {self.e295_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,91))) {self.guard_e296_c(out.state.begin())} else {self.e296_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e297_c(out.state.begin())} else {self.e297_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e298_c(out.state.begin())} else {self.e298_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,39))) {self.guard_e299_c(out.state.begin())} else {self.e299_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e300_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e295_c(&mut self,state:State)->Step {
let mut out=self.b295_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b295_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r50_c(out.state);
out
}
fn e296_c(&mut self,state:State)->Step {
let mut out=self.b296_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b296_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r51_c(out.state);
out
}
fn e297_c(&mut self,state:State)->Step {
let mut out=self.b297_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b297_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r52_c(out.state);
out
}
fn e298_c(&mut self,state:State)->Step {
let mut out=self.b298_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b298_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r53_c(out.state);
out
}
fn e299_c(&mut self,state:State)->Step {
let mut out=self.b299_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b299_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r54_c(out.state);
out
}
fn e300_c(&mut self,state:State)->Step {
let mut out=self.b300_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b300_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r55_c(out.state);
out
}
fn e301_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:301,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b301_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b302_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:302,rule:50,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b302_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e303_c(&mut self,state:State)->Step {
let mut out=self.b303_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:52,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b303_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e304_c(&mut self,state:State)->Step {
let mut out=self.b304_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:304,rule:50,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b304_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e305_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:305,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b305_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b306_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:306,rule:51,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b306_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,45,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e307_c(&mut self,state:State)->Step {
let mut out=self.b307_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:53,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b307_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e308_c(&mut self,state:State)->Step {
let mut out=self.b308_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:308,rule:51,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b308_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,46,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e309_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:309,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b309_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b310_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:310,rule:52,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b310_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,59,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e311_c(&mut self,state:State)->Step {
let mut out=self.b311_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:54,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b311_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e312_c(&mut self,state:State)->Step {
let mut out=self.b312_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:312,rule:52,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b312_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,61,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e313_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:313,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b313_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b314_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:314,rule:53,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'ERROR'"]);}
out
}
fn b314_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ERROR",true,39,"ERROR");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ERROR'");}
out
}
fn e315_c(&mut self,state:State)->Step {
let mut out=self.b315_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:315,rule:53,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b315_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e316_c(&mut self,state:State)->Step {
let mut out=self.b316_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:316,rule:53,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:55,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b316_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e317_c(&mut self,state:State)->Step {
let mut out=self.b317_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:317,rule:53,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b317_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e318_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:318,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b318_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b319_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:319,rule:54,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:56,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b319_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_1::<false>(out.state,"STRING","STRINGParser");
out
}
fn e320_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:320,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b320_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b321_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b321_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e322_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e322_c(&mut self,state:State)->Step {
let mut out=self.b322_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
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
let mut out=self.b323_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:323,rule:55,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:57,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b323_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e324_c(&mut self,state:State)->Step {
let mut out=self.b324_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:324,rule:55,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b324_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,6,".");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'.'");}
out
}
fn e325_c(&mut self,state:State)->Step {
let mut out=self.b325_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:325,rule:55,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:58,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b325_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e326_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:326,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b326_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b327_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:327,rule:56,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'@typeof'"]);}
out
}
fn b327_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@typeof",true,32,"@typeof");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@typeof'");}
out
}
fn e328_c(&mut self,state:State)->Step {
let mut out=self.b328_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:328,rule:56,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b328_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e329_c(&mut self,state:State)->Step {
let mut out=self.b329_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:329,rule:56,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:59,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b329_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e330_c(&mut self,state:State)->Step {
let mut out=self.b330_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:330,rule:56,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b330_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e331_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:331,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b331_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b332_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:332,rule:57,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:60,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b332_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e333_c(&mut self,state:State)->Step {
let mut out=self.b333_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b333_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e334_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e334_c(&mut self,state:State)->Step {
let mut out=self.b334_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
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
let mut out=self.b335_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:335,rule:57,span,text_span,content_span});}
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b335_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,6,".");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'.'");}
out
}
fn e336_c(&mut self,state:State)->Step {
let mut out=self.b336_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
if let Event::Token{span,text_span,content_span,..}=self.arena[out.events.0] {out.events=self.event(Event::Token{expr:336,rule:57,span,text_span,content_span});}
out.events=self.event(Event::Capture {site:61,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IdentifierParser", "__CaptureSite"]);}
out
}
fn b336_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"IDENTIFIER","IdentifierParser");
out
}
fn e337_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:337,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b337_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
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
let mut out=self.b338_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b338_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r42_c(out.state);
out
}
fn guard_e27_c(&mut self,state:State)->Step {
if !self.can_replay(2) {return self.e27_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.e332_c(t1).diag;
let d3=self.diag_rule(57,d2);
let d4=self.diag_rule(5,d3);
self.display_failures(t1.consumed.max(t1.matched),&["DottedIdentifierParser", "__CaptureSite"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["StringSettingValueParser"]);}
Step {ok:false,state,events:EventId(0),diag:d4}
}
fn guard_e28_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e28_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
let d3=self.diag_rule(6,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e46_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e46_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"UNTIL");
let d3=self.diag_rule(10,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'UNTIL'", "'UNTIL'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'UNTIL'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e47_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e47_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATION");
let d3=self.diag_rule(11,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'NEGATION'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e48_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e48_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"LOOKAHEAD");
let d3=self.diag_rule(12,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'LOOKAHEAD'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e49_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e49_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATIVE_LOOKAHEAD");
let d3=self.diag_rule(13,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATIVE_LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'NEGATIVE_LOOKAHEAD'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e50_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e50_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CHAR_RANGE");
let d3=self.diag_rule(14,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'CHAR_RANGE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'('", "','", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e51_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e51_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CI");
let d3=self.diag_rule(15,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CI'", "'CI'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CI'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e52_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e52_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"REGEX");
let d3=self.diag_rule(16,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'REGEX'", "'REGEX'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'REGEX'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e53_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e53_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ANY");
let d3=self.diag_rule(17,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ANY'", "'ANY'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ANY'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e54_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e54_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"EOF");
let d3=self.diag_rule(18,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'EOF'", "'EOF'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'EOF'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e55_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e55_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"EMPTY");
let d3=self.diag_rule(19,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'EMPTY'", "'EMPTY'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'EMPTY'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e56_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e56_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.e332_c(t1).diag;
let d3=self.diag_rule(57,d2);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["DottedIdentifierParser"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e108_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e108_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@root");
let d3=self.diag_rule(22,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@root'", "'@root'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@root'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e109_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e109_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@mapping");
let d3=self.diag_rule(23,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@mapping'", "'@mapping'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@mapping'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e110_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e110_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@eval");
let d3=self.diag_rule(24,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@eval'", "'@eval'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@eval'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e111_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e111_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@whitespace");
let d3=self.diag_rule(25,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@whitespace'", "'@whitespace'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@whitespace'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e112_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e112_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@interleave");
let d3=self.diag_rule(26,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@interleave'", "'@interleave'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@interleave'", "'('", "'profile'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e113_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e113_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@backref");
let d3=self.diag_rule(27,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@backref'", "'@backref'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@backref'", "'('", "'name'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e114_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e114_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@scopeTree");
let d3=self.diag_rule(28,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@scopeTree'", "'@scopeTree'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@scopeTree'", "'('", "'mode'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e115_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e115_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@declares");
let d3=self.diag_rule(29,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@declares'", "'@declares'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@declares'", "'('", "'symbol'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e116_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e116_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@catalog");
let d3=self.diag_rule(30,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@catalog'", "'@catalog'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@catalog'", "'('", "'context'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e117_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e117_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@leftAssoc");
let d3=self.diag_rule(31,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@leftAssoc'", "'@leftAssoc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@leftAssoc'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e118_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e118_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@rightAssoc");
let d3=self.diag_rule(32,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@rightAssoc'", "'@rightAssoc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@rightAssoc'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e119_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e119_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@longestChoice");
let d3=self.diag_rule(33,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@longestChoice'", "'@longestChoice'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@longestChoice'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e120_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e120_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@predictiveChoice");
let d3=self.diag_rule(34,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@predictiveChoice'", "'@predictiveChoice'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@predictiveChoice'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e121_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e121_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@precedence");
let d3=self.diag_rule(35,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@precedence'", "'@precedence'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@precedence'", "'('", "'level'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e122_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e122_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@doc");
let d3=self.diag_rule(36,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@doc'", "'@doc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@doc'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e123_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e123_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@recovery");
let d3=self.diag_rule(37,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@recovery'", "'@recovery'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@recovery'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e124_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e124_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@skip");
let d3=self.diag_rule(38,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@skip'", "'@skip'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@skip'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e125_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e125_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@commonField");
let d3=self.diag_rule(39,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@commonField'", "'@commonField'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@commonField'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e126_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e126_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@enum");
let d3=self.diag_rule(40,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@enum'", "'@enum'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@enum'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e127_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e127_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@");
let d3=self.diag_rule(41,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@'", "'@'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e229_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e229_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"sync");
self.display_failures(t1.consumed.max(t1.matched),&["'sync'", "'sync'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'='", "'sync'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e295_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e295_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"(");
let d3=self.diag_rule(50,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'('", "'('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e296_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e296_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"[");
let d3=self.diag_rule(51,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'['", "'['"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'['", "']'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e297_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e297_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
let d3=self.diag_rule(52,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e298_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e298_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ERROR");
let d3=self.diag_rule(53,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ERROR'", "'ERROR'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ERROR'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e299_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e299_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.e319_c(t1).diag;
let d3=self.diag_rule(54,d2);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["TerminalElementParser"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn token_0<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
self.identifier::<MATCH>(state,label)
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_1<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
self.quoted::<MATCH>(state,'\'',label)
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_2<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
self.identifier::<MATCH>(state,label)
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_3<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
self.number::<MATCH>(state,label)
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_4<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let length=self.input.cp_at(state.position::<MATCH>()).filter(|(c,_)|c.is_ascii_digit()).map(|(_,n)|n);self.primitive::<MATCH>(state,length,label)
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn trivia_0<const MATCH:bool>(&mut self,state:State)->Step {
self.display_reach(state.consumed.max(state.matched));let mut out=Step::yes(state);loop {let before=out.state.position::<MATCH>();
if !matches!(self.text.as_bytes().get(before),Some(9|10|11|12|13|32|47)) {self.display_failures(out.state.consumed.max(out.state.matched),&["' '", "'//'", "'/*'", "'*/'"]);break;}
{let start=out.state.position::<MATCH>();let mut p=start;
while let Some((c,n))=self.input.cp_at(p) {if ![9, 10, 11, 12, 13, 32].contains(&(c as u64)) {break;}p+=n;}
if p>start {out.state.advance::<MATCH>(p-start);continue;}
self.display_fail(out.state.consumed.max(out.state.matched),"' '");
}
{let start=out.state.position::<MATCH>();let mut p=start;
if self.input.starts_with(p,"//") {p+=2;while let Some((c,n))=self.input.cp_at(p) {if matches!(c,'\n'|'\r') {break;}p+=n;}let end=if self.input.starts_with(p,"\r\n") {p+2} else {self.input.cp_at(p).map_or(p,|(_,n)|p+n)};out.state.advance::<MATCH>(p-start);if !MATCH {out.state.matched=end;}continue;}
self.display_fail(out.state.consumed.max(out.state.matched),"'//'");
}
{let start=out.state.position::<MATCH>();let mut p=start;
if self.input.starts_with(p,"/*") {let body=p+2;if let Some(n)=self.text[body..].find("*/") {p=body+n+2;out.state.advance::<MATCH>(p-start);continue;}}
if self.input.starts_with(start,"/*") {self.display_fail(self.text.len(),"'*/'");self.display_fail(self.text.len(),"WildCardStringParser");} else {self.display_fail(out.state.consumed.max(out.state.matched),"'/*'");self.display_fail(out.state.consumed.max(out.state.matched),"'*/'");}
}
if out.state.position::<MATCH>()==before {break;}}
self.display_reach(out.state.consumed.max(out.state.matched));out}
fn skip_0<const MATCH:bool>(&self,state:State)->State {let origin=state.position::<MATCH>();
if self.lex_enabled {if let Some((dc,dm))=self.trivia_skip.get(0,MATCH,origin) {
if dc==u32::MAX {return state;}
let mut hit=state;if MATCH {hit.matched=origin+dm as usize;} else {hit.consumed=origin+dc as usize;hit.matched=origin+dm as usize;}
return hit;}}
let mut out=state;loop {let before=out.position::<MATCH>();
if !matches!(self.text.as_bytes().get(before),Some(9|10|11|12|13|32|47)) {break;}
{let start=out.position::<MATCH>();let mut p=start;
while let Some((c,n))=self.input.cp_at(p) {if ![9, 10, 11, 12, 13, 32].contains(&(c as u64)) {break;}p+=n;}
if p>start {out.advance::<MATCH>(p-start);continue;}
}
{let start=out.position::<MATCH>();let mut p=start;
if self.input.starts_with(p,"//") {p+=2;while let Some((c,n))=self.input.cp_at(p) {if matches!(c,'\n'|'\r') {break;}p+=n;}let end=if self.input.starts_with(p,"\r\n") {p+2} else {self.input.cp_at(p).map_or(p,|(_,n)|p+n)};out.advance::<MATCH>(p-start);if !MATCH {out.matched=end;}continue;}
}
{let start=out.position::<MATCH>();let mut p=start;
if self.input.starts_with(p,"/*") {let body=p+2;if let Some(n)=self.text[body..].find("*/") {p=body+n+2;out.advance::<MATCH>(p-start);continue;}}
}
if out.position::<MATCH>()==before {break;}}
if self.lex_enabled {
if out.position::<MATCH>()==origin {self.trivia_skip.insert(0,MATCH,origin,(u32::MAX,0));}
else if let (Ok(dc),Ok(dm))=(u32::try_from(out.consumed.saturating_sub(origin)),u32::try_from(out.matched.saturating_sub(origin))) {
if dc!=u32::MAX {self.trivia_skip.insert(0,MATCH,origin,(dc,dm));}}}
out}
}
