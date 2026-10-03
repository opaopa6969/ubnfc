//! IR の expression/rule ごとの静的関数。c=consumed, m=matchOnly。
#![allow(dead_code, unused_mut)]
use super::api::*;
use super::runtime::*;
#[allow(unused_imports)]
use super::rt::{diag::Diag,memo::Key};
pub fn parse(text:&str)->ParseResult {parse_with_options(text,ParseOptions::default())}
// issue #35 / D-070: 既定呼出しは呼出し元 thread の上で保守的な native stack 番兵
// （NATIVE_STACK_SENTINEL_BYTES、呼出し元が明示的に小さい stack で呼んでも安全な値）の
// まま解析する。番兵に max_depth より先に当たった（＝呼出し元の stack が小さいだけで、
// 論理上限にはまだ余裕がある）ときだけ、max_depth から見積もった十分な stack を持つ
// 専用 thread で 1 度だけ解析し直す。外部 TokenScanner を受け取る parse_with_scanner は
// 任意の状態を持つ scanner を thread 境界へ渡せないためこの対象外（既知の制約）。
pub fn parse_with_options(text:&str,options:ParseOptions)->ParseResult {
let (result,stack_exhausted)=parse_with_options_budget(text,options,NATIVE_STACK_SENTINEL_BYTES);
if result.ok || !stack_exhausted {return result;}
parse_with_options_escalated(text,options)
}
// issue #65 / D-070 wasm: wasm32 には OS スレッドの stack size 制御が無く
// （`wasm32-unknown-unknown` は既定で `std::thread` 自体を欠く）、この escalation は
// コンパイルできない。生成物は依存ゼロ・std のみのままにするため wasm32 では
// escalation を丸ごとコンパイル対象から外し、番兵budget（NATIVE_STACK_SENTINEL_BYTES）
// の結果、つまり既に "maximum parse depth exceeded" が入った診断へそのまま fallback する
// （crash しない。D-070 の契約どおり）。
#[cfg(not(target_arch = "wasm32"))]
fn parse_with_options_escalated(text:&str,options:ParseOptions)->ParseResult {
let budget=escalated_stack_budget(options.max_depth);
let stack_bytes=escalated_thread_stack_bytes(options.max_depth);
std::thread::scope(|scope| {
std::thread::Builder::new().stack_size(stack_bytes)
.spawn_scoped(scope, move || parse_with_options_budget(text,options,budget).0)
.expect("spawn escalated parser thread")
.join()
.unwrap_or_else(|payload| std::panic::resume_unwind(payload))
})
}
#[cfg(target_arch = "wasm32")]
fn parse_with_options_escalated(text:&str,options:ParseOptions)->ParseResult {
parse_with_options_budget(text,options,NATIVE_STACK_SENTINEL_BYTES).0
}
fn parse_with_options_budget(text:&str,options:ParseOptions,budget:usize)->(ParseResult,bool) {
parse_with_scanner_budget(text,options,&mut RejectExtern,budget)
}
pub fn parse_with_scanner(text:&str,options:ParseOptions,scanner:&mut dyn TokenScanner)->ParseResult {
parse_with_scanner_budget(text,options,scanner,NATIVE_STACK_SENTINEL_BYTES).0
}
fn parse_with_scanner_budget(text:&str,options:ParseOptions,scanner:&mut dyn TokenScanner,budget:usize)->(ParseResult,bool) {
// 診断は const DIAG で単相化する（D-036）。要求されていれば診断 session で 1 度だけ走り、
// そうでなければ fast session で走って、診断が要る結果のときだけ診断 session で解析し直す（D-023）。
if options.diagnostics {
let mut parser=Session::<true>::new(text,options,scanner,budget);
let step=parser.r0_c(State::default());
let stack_exhausted=parser.stack_sentinel_tripped();
return (parser.finish(step),stack_exhausted);
}
let mut parser=Session::<false>::new(text,options,&mut *scanner,budget);
let step=parser.r0_c(State::default());
let stack_exhausted=parser.stack_sentinel_tripped();
let (result,escalate)=parser.finish_checked(step);
if !escalate {return (result,stack_exhausted);}
let diag_options=ParseOptions{diagnostics:true,..options};
let mut parser=Session::<true>::new(text,diag_options,scanner,budget);
let step=parser.r0_c(State::default());
let stack_exhausted=stack_exhausted || parser.stack_sentinel_tripped();
(parser.finish(step),stack_exhausted)
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
("UBNF",Some("DeclarativeTokenValue"))=>parser.r9_c(State::default()),
("UBNF",Some("LexicalBody"))=>parser.r10_c(State::default()),
("UBNF",Some("LexicalSequence"))=>parser.r11_c(State::default()),
("UBNF",Some("LexicalElement"))=>parser.r12_c(State::default()),
("UBNF",Some("LexicalSuffix"))=>parser.r13_c(State::default()),
("UBNF",Some("LexicalAtom"))=>parser.r14_c(State::default()),
("UBNF",Some("TokenValue"))=>parser.r15_c(State::default()),
("UBNF",Some("UntilExpression"))=>parser.r16_c(State::default()),
("UBNF",Some("NegationExpression"))=>parser.r17_c(State::default()),
("UBNF",Some("LookaheadExpression"))=>parser.r18_c(State::default()),
("UBNF",Some("NegativeLookaheadExpression"))=>parser.r19_c(State::default()),
("UBNF",Some("CharRangeExpression"))=>parser.r20_c(State::default()),
("UBNF",Some("CIExpression"))=>parser.r21_c(State::default()),
("UBNF",Some("RegexExpression"))=>parser.r22_c(State::default()),
("UBNF",Some("AdapterExpression"))=>parser.r23_c(State::default()),
("UBNF",Some("AnyExpression"))=>parser.r24_c(State::default()),
("UBNF",Some("EofExpression"))=>parser.r25_c(State::default()),
("UBNF",Some("EmptyExpression"))=>parser.r26_c(State::default()),
("UBNF",Some("RuleDecl"))=>parser.r27_c(State::default()),
("UBNF",Some("Annotation"))=>parser.r28_c(State::default()),
("UBNF",Some("RootAnnotation"))=>parser.r29_c(State::default()),
("UBNF",Some("MappingAnnotation"))=>parser.r30_c(State::default()),
("UBNF",Some("EvalAnnotation"))=>parser.r31_c(State::default()),
("UBNF",Some("WhitespaceAnnotation"))=>parser.r32_c(State::default()),
("UBNF",Some("InterleaveAnnotation"))=>parser.r33_c(State::default()),
("UBNF",Some("BackrefAnnotation"))=>parser.r34_c(State::default()),
("UBNF",Some("ScopeTreeAnnotation"))=>parser.r35_c(State::default()),
("UBNF",Some("DeclaresAnnotation"))=>parser.r36_c(State::default()),
("UBNF",Some("CatalogAnnotation"))=>parser.r37_c(State::default()),
("UBNF",Some("LeftAssocAnnotation"))=>parser.r38_c(State::default()),
("UBNF",Some("RightAssocAnnotation"))=>parser.r39_c(State::default()),
("UBNF",Some("LongestChoiceAnnotation"))=>parser.r40_c(State::default()),
("UBNF",Some("PredictiveChoiceAnnotation"))=>parser.r41_c(State::default()),
("UBNF",Some("PrecedenceAnnotation"))=>parser.r42_c(State::default()),
("UBNF",Some("DocAnnotation"))=>parser.r43_c(State::default()),
("UBNF",Some("RecoveryAnnotation"))=>parser.r44_c(State::default()),
("UBNF",Some("SkipAnnotation"))=>parser.r45_c(State::default()),
("UBNF",Some("CommonFieldAnnotation"))=>parser.r46_c(State::default()),
("UBNF",Some("EnumAnnotation"))=>parser.r47_c(State::default()),
("UBNF",Some("SimpleAnnotation"))=>parser.r48_c(State::default()),
("UBNF",Some("ChoiceBody"))=>parser.r49_c(State::default()),
("UBNF",Some("SequenceBody"))=>parser.r50_c(State::default()),
("UBNF",Some("AnnotatedElement"))=>parser.r51_c(State::default()),
("UBNF",Some("PostfixQuantifier"))=>parser.r52_c(State::default()),
("UBNF",Some("SeparatedBy"))=>parser.r53_c(State::default()),
("UBNF",Some("BoundedQuantifier"))=>parser.r54_c(State::default()),
("UBNF",Some("Digits"))=>parser.r55_c(State::default()),
("UBNF",Some("AtomicElement"))=>parser.r56_c(State::default()),
("UBNF",Some("GroupElement"))=>parser.r57_c(State::default()),
("UBNF",Some("OptionalElement"))=>parser.r58_c(State::default()),
("UBNF",Some("RepeatElement"))=>parser.r59_c(State::default()),
("UBNF",Some("ErrorElement"))=>parser.r60_c(State::default()),
("UBNF",Some("TerminalElement"))=>parser.r61_c(State::default()),
("UBNF",Some("QuantifiedRef"))=>parser.r62_c(State::default()),
("UBNF",Some("TypeofElement"))=>parser.r63_c(State::default()),
("UBNF",Some("DottedIdentifier"))=>parser.r64_c(State::default()),
("UBNF",Some("RuleBody"))=>parser.r65_c(State::default()),
_=>return Err("unsupported grammar or entry"),})}
// D-070: `parse_entry_budget` / `parse_entry_escalated` は `parse_entry_with_options` より
// 前に置く。driver.rs の scanner_hook / examples/p4-rust の build.rs はこの関数を
// テキストごと切り出して `parse_entry_with_scanner` へ改名するので、その範囲に
// この 2 つを含めない（含めると全文を丸ごと差し込む方の写しと二重定義になる）。
fn parse_entry_budget(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions,scanner:&mut dyn TokenScanner,budget:usize)->Result<(ParseResult,bool),&'static str> {
if options.diagnostics {
let mut parser=Session::<true>::new(text,options,scanner,budget);
let step=entry_step(&mut parser,grammar,entry)?;
let stack_exhausted=parser.stack_sentinel_tripped();
return Ok((parser.finish(step),stack_exhausted));
}
let mut parser=Session::<false>::new(text,options,scanner,budget);
let step=entry_step(&mut parser,grammar,entry)?;
let stack_exhausted=parser.stack_sentinel_tripped();
let (result,escalate)=parser.finish_checked(step);
if !escalate {return Ok((result,stack_exhausted));}
let diag_options=ParseOptions{diagnostics:true,..options};
let mut parser=Session::<true>::new(text,diag_options,scanner,budget);
let step=entry_step(&mut parser,grammar,entry)?;
let stack_exhausted=stack_exhausted || parser.stack_sentinel_tripped();
Ok((parser.finish(step),stack_exhausted))
}
// issue #35 / D-070: native stack 番兵に max_depth より先へ当たった（＝呼出し元 thread の
// stack が小さいだけ）ときだけ、max_depth から見積もった十分な stack を持つ専用 thread で
// 1 度だけ解析し直す。RejectExtern は状態を持たないので thread をまたいで再構築してよい。
#[cfg(not(target_arch = "wasm32"))]
fn parse_entry_escalated(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions)->Result<ParseResult,&'static str> {
let budget=escalated_stack_budget(options.max_depth);
let stack_bytes=escalated_thread_stack_bytes(options.max_depth);
std::thread::scope(|scope| {
std::thread::Builder::new().stack_size(stack_bytes)
.spawn_scoped(scope, move || {
let mut scanner=RejectExtern;
parse_entry_budget(grammar,entry,text,options,&mut scanner,budget)
})
.expect("spawn escalated parser thread")
.join()
.unwrap_or_else(|payload| std::panic::resume_unwind(payload))
}).map(|(result,_)| result)
}
// issue #65 / D-070 wasm: wasm32 では escalation を丸ごとコンパイル対象から外し、番兵
// budget（NATIVE_STACK_SENTINEL_BYTES）での再解析結果、つまり既に "maximum parse depth
// exceeded" が入った診断へそのまま fallback する（crash しない。D-070 の契約どおり）。
#[cfg(target_arch = "wasm32")]
fn parse_entry_escalated(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions)->Result<ParseResult,&'static str> {
let mut scanner=RejectExtern;
parse_entry_budget(grammar,entry,text,options,&mut scanner,NATIVE_STACK_SENTINEL_BYTES).map(|(result,_)| result)
}
pub fn parse_entry_with_options(grammar:&str,entry:Option<&str>,text:&str,options:ParseOptions)->Result<ParseResult,&'static str> {
let mut scanner=RejectExtern;
let (result,stack_exhausted)=parse_entry_budget(grammar,entry,text,options,&mut scanner,NATIVE_STACK_SENTINEL_BYTES)?;
if result.ok || !stack_exhausted {return Ok(result);}
/*STACK_ESCALATION*/parse_entry_escalated(grammar,entry,text,options)/*STACK_ESCALATION*/
}
impl<const DIAG: bool> Session<'_, DIAG> {
fn r0_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e0_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:0,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["UBNFFileParser"]);}
self.depth-=1;out.diag=self.diag_rule(0,out.diag);out
}
fn r1_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e3_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:1,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@import'", "'from'"]);}
self.depth-=1;out.diag=self.diag_rule(1,out.diag);out
}
fn r2_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e8_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:2,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'grammar'", "'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(2,out.diag);out
}
fn r3_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e21_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:3,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@'", "':'"]);}
self.depth-=1;out.diag=self.diag_rule(3,out.diag);out
}
fn r4_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e26_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:4,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(4,out.diag);out
}
fn r5_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e29_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:5,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["StringSettingValueParser"]);}
self.depth-=1;out.diag=self.diag_rule(5,out.diag);out
}
fn r6_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e31_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:6,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(6,out.diag);out
}
fn r7_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e36_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:7,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
self.depth-=1;out.diag=self.diag_rule(7,out.diag);out
}
fn r8_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e40_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:8,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'token'"]);}
self.depth-=1;out.diag=self.diag_rule(8,out.diag);out
}
fn r9_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e51_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:9,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
self.depth-=1;out.diag=self.diag_rule(9,out.diag);out
}
fn r10_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e54_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:10,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(10,out.diag);out
}
fn r11_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e60_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:11,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalSequenceParser"]);}
self.depth-=1;out.diag=self.diag_rule(11,out.diag);out
}
fn r12_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e63_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:12,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(12,out.diag);out
}
fn r13_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e67_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:13,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalSuffixParser"]);}
self.depth-=1;out.diag=self.diag_rule(13,out.diag);out
}
fn r14_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e82_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:14,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["LexicalAtomParser"]);}
self.depth-=1;out.diag=self.diag_rule(14,out.diag);out
}
fn r15_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e131_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:15,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'", "'('", "')'", "'NEGATION'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'", "'CHAR_RANGE'", "','", "'CI'", "'REGEX'", "'ADAPTER'", "'version'", "'='", "'ANY'", "'EOF'", "'EMPTY'"]);}
self.depth-=1;out.diag=self.diag_rule(15,out.diag);out
}
fn r16_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e144_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:16,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(16,out.diag);out
}
fn r17_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e149_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:17,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(17,out.diag);out
}
fn r18_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e154_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:18,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(18,out.diag);out
}
fn r19_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e159_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:19,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(19,out.diag);out
}
fn r20_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e164_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:20,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'", "'('", "','", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(20,out.diag);out
}
fn r21_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e171_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:21,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'CI'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(21,out.diag);out
}
fn r22_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e176_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:22,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'REGEX'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(22,out.diag);out
}
fn r23_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e181_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:23,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ADAPTER'", "'('", "','", "'version'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(23,out.diag);out
}
fn r24_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e190_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:24,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ANY'"]);}
self.depth-=1;out.diag=self.diag_rule(24,out.diag);out
}
fn r25_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e192_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:25,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'EOF'"]);}
self.depth-=1;out.diag=self.diag_rule(25,out.diag);out
}
fn r26_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e194_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:26,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'EMPTY'"]);}
self.depth-=1;out.diag=self.diag_rule(26,out.diag);out
}
fn r27_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e196_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:27,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'::='", "';'"]);}
self.depth-=1;out.diag=self.diag_rule(27,out.diag);out
}
fn r28_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e203_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:28,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@root'", "'@mapping'", "'('", "')'", "'@eval'", "'@whitespace'", "'@interleave'", "'profile'", "'='", "'@backref'", "'name'", "'@scopeTree'", "'mode'", "'@declares'", "'symbol'", "'@catalog'", "'context'", "'@leftAssoc'", "'@rightAssoc'", "'@longestChoice'", "'@predictiveChoice'", "'@precedence'", "'level'", "'@doc'", "'@recovery'", "'@skip'", "'@commonField'", "'@enum'", "'@'"]);}
self.depth-=1;out.diag=self.diag_rule(28,out.diag);out
}
fn r29_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e224_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:29,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@root'"]);}
self.depth-=1;out.diag=self.diag_rule(29,out.diag);out
}
fn r30_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e226_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:30,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@mapping'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(30,out.diag);out
}
fn r31_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e243_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:31,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@eval'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(31,out.diag);out
}
fn r32_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e252_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:32,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@whitespace'"]);}
self.depth-=1;out.diag=self.diag_rule(32,out.diag);out
}
fn r33_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e259_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:33,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@interleave'", "'('", "'profile'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(33,out.diag);out
}
fn r34_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e266_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:34,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@backref'", "'('", "'name'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(34,out.diag);out
}
fn r35_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e273_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:35,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@scopeTree'", "'('", "'mode'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(35,out.diag);out
}
fn r36_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e280_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:36,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@declares'", "'('", "'symbol'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(36,out.diag);out
}
fn r37_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e293_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:37,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@catalog'", "'('", "'context'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(37,out.diag);out
}
fn r38_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e300_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:38,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@leftAssoc'"]);}
self.depth-=1;out.diag=self.diag_rule(38,out.diag);out
}
fn r39_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e302_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:39,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@rightAssoc'"]);}
self.depth-=1;out.diag=self.diag_rule(39,out.diag);out
}
fn r40_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e304_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:40,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@longestChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(40,out.diag);out
}
fn r41_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e306_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:41,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@predictiveChoice'"]);}
self.depth-=1;out.diag=self.diag_rule(41,out.diag);out
}
fn r42_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e308_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:42,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@precedence'", "'('", "'level'", "'='", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(42,out.diag);out
}
fn r43_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e315_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:43,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@doc'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(43,out.diag);out
}
fn r44_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e320_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:44,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@recovery'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(44,out.diag);out
}
fn r45_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e331_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:45,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@skip'"]);}
self.depth-=1;out.diag=self.diag_rule(45,out.diag);out
}
fn r46_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e333_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:46,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@commonField'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(46,out.diag);out
}
fn r47_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e342_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:47,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@enum'"]);}
self.depth-=1;out.diag=self.diag_rule(47,out.diag);out
}
fn r48_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e344_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:48,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
self.depth-=1;out.diag=self.diag_rule(48,out.diag);out
}
fn r49_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e347_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:49,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["ChoiceBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(49,out.diag);out
}
fn r50_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e353_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:50,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["SequenceBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(50,out.diag);out
}
fn r51_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e356_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:51,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["AnnotatedElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(51,out.diag);out
}
fn r52_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e370_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:52,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'+'", "'?'", "'*'"]);}
self.depth-=1;out.diag=self.diag_rule(52,out.diag);out
}
fn r53_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e374_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:53,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
self.depth-=1;out.diag=self.diag_rule(53,out.diag);out
}
fn r54_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e377_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:54,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(54,out.diag);out
}
fn r55_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e386_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:55,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["DigitsParser"]);}
self.depth-=1;out.diag=self.diag_rule(55,out.diag);out
}
fn r56_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e390_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:56,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'['", "']'", "'{'", "'}'", "'ERROR'"]);}
self.depth-=1;out.diag=self.diag_rule(56,out.diag);out
}
fn r57_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e397_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:57,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(57,out.diag);out
}
fn r58_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e401_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:58,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'['", "']'"]);}
self.depth-=1;out.diag=self.diag_rule(58,out.diag);out
}
fn r59_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e405_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:59,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
self.depth-=1;out.diag=self.diag_rule(59,out.diag);out
}
fn r60_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e409_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:60,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'ERROR'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(60,out.diag);out
}
fn r61_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e414_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:61,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["TerminalElementParser"]);}
self.depth-=1;out.diag=self.diag_rule(61,out.diag);out
}
fn r62_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e416_c(state);
if out.ok {
let mut scope_sites=self.take_scope();self.scope_captures(out.events,&[69],&mut scope_sites);for &(site,span) in &scope_sites { let name=self.text(span);if name.is_empty() {continue;}let offset_cp=self.cp(span[0]);let len_cp=self.cp(span[1])-offset_cp;
if site==69 {let mut targets=self.take_scope();self.scope_captures(out.events,&[69],&mut targets);if let Some((_,target))=targets.first() {let expected=self.text(*target);if expected!=name {self.effect(ScanEffect::Diagnostic{message:format!("back-reference mismatch: expected '{expected}' but got '{name}'"),offset_cp,len_cp,severity:Severity::Error});}}self.give_scope(targets);}
let _=(&name,offset_cp,len_cp); }
self.give_scope(scope_sites);
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:62,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["QuantifiedRefParser"]);}
self.depth-=1;out.diag=self.diag_rule(62,out.diag);out
}
fn r63_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e422_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:63,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["'@typeof'", "'('", "')'"]);}
self.depth-=1;out.diag=self.diag_rule(63,out.diag);out
}
fn r64_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e427_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:64,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["DottedIdentifierParser"]);}
self.depth-=1;out.diag=self.diag_rule(64,out.diag);out
}
fn r65_c(&mut self,state:State)->Step { if let Some(limit)=self.enter_rule(state) {return limit;}self.statistics.rule_evaluations+=1;let mark=self.mark();
let mut out=self.e433_c(state);
if out.ok {
let end=out.state.consumed;
out.events=self.event(Event::Rule {rule:65,span:[state.consumed,end],child:out.events,caps:(0,0)});
} else { self.restore(mark); }
if !out.ok {self.display_failures(state.consumed.max(state.matched),&["RuleBodyParser"]);}
self.depth-=1;out.diag=self.diag_rule(65,out.diag);out
}
fn e0_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:0,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b0_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
let mut out=self.b3_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.events=self.relabel_token(out.events,4,1);
} else {self.display_failures(state.consumed.max(state.matched),&["'@import'"]);}
out
}
fn b4_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@import",true,20,true,"@import");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@import'");}
out
}
fn e5_c(&mut self,state:State)->Step {
let mut out=self.b5_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,5,1);
out.events=self.event(Event::Capture {site:1,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b5_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e6_c(&mut self,state:State)->Step {
let mut out=self.b6_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,6,1);
} else {self.display_failures(state.consumed.max(state.matched),&["'from'"]);}
out
}
fn b6_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"from",true,52,true,"from");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'from'");}
out
}
fn e7_c(&mut self,state:State)->Step {
let mut out=self.b7_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,7,1);
out.events=self.event(Event::Capture {site:2,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b7_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e8_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:8,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b8_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.events=self.relabel_token(out.events,9,2);
} else {self.display_failures(state.consumed.max(state.matched),&["'grammar'"]);}
out
}
fn b9_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"grammar",true,53,true,"grammar");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'grammar'");}
out
}
fn e10_c(&mut self,state:State)->Step {
let mut out=self.b10_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,10,2);
out.events=self.event(Event::Capture {site:3,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b10_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e11_c(&mut self,state:State)->Step {
let mut out=self.b11_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,11,2);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b11_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,63,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e12_c(&mut self,state:State)->Step {
let mut out=self.b12_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'token'", "__CaptureSite"]);}
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
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out=self.r27_c(out.state);
out
}
fn e20_c(&mut self,state:State)->Step {
let mut out=self.b20_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,20,2);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b20_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,65,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e21_c(&mut self,state:State)->Step {
let mut out=self.b21_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.events=self.relabel_token(out.events,22,3);
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b22_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,12,false,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e23_c(&mut self,state:State)->Step {
let mut out=self.b23_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,23,3);
out.events=self.event(Event::Capture {site:8,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b23_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e24_c(&mut self,state:State)->Step {
let mut out=self.b24_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,24,3);
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b24_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,7,false,":");
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
let mut out=self.b26_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b26_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=self.e27_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
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
let mut out=self.b29_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out=self.r64_c(out.state);
out
}
fn e31_c(&mut self,state:State)->Step {
let mut out=self.b31_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.events=self.relabel_token(out.events,32,6);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b32_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,63,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e33_c(&mut self,state:State)->Step {
let mut out=self.b33_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
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
out.events=self.relabel_token(out.events,35,6);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b35_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,65,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e36_c(&mut self,state:State)->Step {
let mut out=self.b36_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.events=self.relabel_token(out.events,37,7);
out.events=self.event(Event::Capture {site:12,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b37_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e38_c(&mut self,state:State)->Step {
let mut out=self.b38_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,38,7);
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b38_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,":",true,7,false,":");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"':'");}
out
}
fn e39_c(&mut self,state:State)->Step {
let mut out=self.b39_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,39,7);
out.events=self.event(Event::Capture {site:13,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b39_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e40_c(&mut self,state:State)->Step {
let mut out=self.b40_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e41_c(&mut self,state:State)->Step {
let mut out=self.b41_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,41,8);
} else {self.display_failures(state.consumed.max(state.matched),&["'token'"]);}
out
}
fn b41_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"token",true,61,true,"token");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'token'");}
out
}
fn e42_c(&mut self,state:State)->Step {
let mut out=self.b42_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,42,8);
out.events=self.event(Event::Capture {site:14,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b42_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e43_c(&mut self,state:State)->Step {
let mut out=self.b43_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'::='", "'='"]);}
out
}
fn b43_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e44_c(out.state);
out
}
fn e44_c(&mut self,state:State)->Step {
let mut out=self.b44_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'::='", "'='"]);}
out
}
fn b44_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,61))) {self.guard_e45_c(out.state.begin())} else {self.e45_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,58))) {self.guard_e48_c(out.state.begin())} else {self.e48_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e45_c(&mut self,state:State)->Step {
let mut out=self.b45_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b45_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e46_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e47_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e46_c(&mut self,state:State)->Step {
let mut out=self.b46_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,46,8);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b46_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e47_c(&mut self,state:State)->Step {
let mut out=self.b47_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:15,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b47_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r15_c(out.state);
out
}
fn e48_c(&mut self,state:State)->Step {
let mut out=self.b48_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
out
}
fn b48_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e49_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e50_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e49_c(&mut self,state:State)->Step {
let mut out=self.b49_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,49,8);
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
out
}
fn b49_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"::=",true,8,false,"::=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'::='");}
out
}
fn e50_c(&mut self,state:State)->Step {
let mut out=self.b50_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:16,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b50_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r9_c(out.state);
out
}
fn e51_c(&mut self,state:State)->Step {
let mut out=self.b51_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b51_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e52_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e53_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e52_c(&mut self,state:State)->Step {
let mut out=self.b52_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:17,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b52_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e53_c(&mut self,state:State)->Step {
let mut out=self.b53_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,53,9);
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b53_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,";",true,9,false,";");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"';'");}
out
}
fn e54_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:54,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b54_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b54_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e55_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e56_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e55_c(&mut self,state:State)->Step {
let mut out=self.b55_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:18,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b55_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r11_c(out.state);
out
}
fn e56_c(&mut self,state:State)->Step {
let mut out=self.b56_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b56_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e57_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e57_c(&mut self,state:State)->Step {
let mut out=self.b57_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e58_c(&mut self,state:State)->Step {
let mut out=self.b58_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,58,10);
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b58_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"|",true,64,false,"|");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'|'");}
out
}
fn e59_c(&mut self,state:State)->Step {
let mut out=self.b59_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:19,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b59_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r11_c(out.state);
out
}
fn e60_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:60,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b60_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b60_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e61_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e61_c(&mut self,state:State)->Step {
let mut out=self.b61_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b61_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e62_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:20,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if count<1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e62_c(&mut self,state:State)->Step {
let mut out=self.b62_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b62_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r12_c(out.state);
out
}
fn e63_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:63,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b63_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b63_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e64_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e65_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e64_c(&mut self,state:State)->Step {
let mut out=self.b64_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:21,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b64_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r14_c(out.state);
out
}
fn e65_c(&mut self,state:State)->Step {
let mut out=self.b65_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b65_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e66_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e66_c(&mut self,state:State)->Step {
let mut out=self.b66_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:22,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b66_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r13_c(out.state);
out
}
fn e67_c(&mut self,state:State)->Step {
let mut out=self.b67_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b67_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e68_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e68_c(&mut self,state:State)->Step {
let mut out=self.b68_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:23,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'", "'{'", "'}'"]);}
out
}
fn b68_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e69_c(out.state);
out
}
fn e69_c(&mut self,state:State)->Step {
let mut out=self.b69_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'", "'{'", "'}'"]);}
out
}
fn b69_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=self.e70_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e71_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e72_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e73_c(out.state.begin())} else {self.e73_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e70_c(&mut self,state:State)->Step {
let mut out=self.b70_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,70,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'?'"]);}
out
}
fn b70_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"?",true,11,false,"?");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'?'");}
out
}
fn e71_c(&mut self,state:State)->Step {
let mut out=self.b71_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,71,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'*'"]);}
out
}
fn b71_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"*",true,3,false,"*");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'*'");}
out
}
fn e72_c(&mut self,state:State)->Step {
let mut out=self.b72_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,72,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'+'"]);}
out
}
fn b72_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"+",true,4,false,"+");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'+'");}
out
}
fn e73_c(&mut self,state:State)->Step {
let mut out=self.b73_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b73_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e74_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e75_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e76_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e81_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e74_c(&mut self,state:State)->Step {
let mut out=self.b74_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,74,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b74_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,63,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e75_c(&mut self,state:State)->Step {
let mut out=self.b75_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,75,13);
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser"]);}
out
}
fn b75_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_5::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e76_c(&mut self,state:State)->Step {
let mut out=self.b76_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b76_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e77_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e77_c(&mut self,state:State)->Step {
let mut out=self.b77_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e78_c(&mut self,state:State)->Step {
let mut out=self.b78_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,78,13);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b78_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e79_c(&mut self,state:State)->Step {
let mut out=self.b79_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b79_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e80_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e80_c(&mut self,state:State)->Step {
let mut out=self.b80_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,80,13);
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser"]);}
out
}
fn b80_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_5::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e81_c(&mut self,state:State)->Step {
let mut out=self.b81_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,81,13);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b81_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,65,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e82_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:82,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b82_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b82_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e83_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e83_c(&mut self,state:State)->Step {
let mut out=self.b83_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
out.events=self.event(Event::Capture {site:24,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CAPTURE'", "'CHAR_RANGE'", "'NEGATION'", "'SAME_AS'", "'['", "']'", "'{'", "'}'", "__CaptureSite"]);}
out
}
fn b83_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e84_c(out.state);
out
}
fn e84_c(&mut self,state:State)->Step {
let mut out=self.b84_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CAPTURE'", "'CHAR_RANGE'", "'NEGATION'", "'SAME_AS'", "'['", "']'", "'{'", "'}'"]);}
out
}
fn b84_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=self.e85_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,40))) {self.guard_e86_c(out.state.begin())} else {self.e86_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,91))) {self.guard_e90_c(out.state.begin())} else {self.e90_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e94_c(out.state.begin())} else {self.e94_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,76|78))) {self.guard_e98_c(out.state.begin())} else {self.e98_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e106_c(out.state.begin())} else {self.e106_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,83))) {self.guard_e113_c(out.state.begin())} else {self.e113_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e118_c(out.state.begin())} else {self.e118_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e125_c(out.state.begin())} else {self.e125_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e130_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e85_c(&mut self,state:State)->Step {
let mut out=self.b85_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,85,14);
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b85_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e86_c(&mut self,state:State)->Step {
let mut out=self.b86_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
out
}
fn b86_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e87_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e88_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e89_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e87_c(&mut self,state:State)->Step {
let mut out=self.b87_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,87,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b87_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e88_c(&mut self,state:State)->Step {
let mut out=self.b88_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b88_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e89_c(&mut self,state:State)->Step {
let mut out=self.b89_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,89,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b89_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e90_c(&mut self,state:State)->Step {
let mut out=self.b90_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'['", "']'"]);}
out
}
fn b90_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
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
fn e91_c(&mut self,state:State)->Step {
let mut out=self.b91_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,91,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b91_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,48,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e92_c(&mut self,state:State)->Step {
let mut out=self.b92_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b92_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e93_c(&mut self,state:State)->Step {
let mut out=self.b93_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,93,14);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b93_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,49,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e94_c(&mut self,state:State)->Step {
let mut out=self.b94_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b94_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e95_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e96_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e97_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e95_c(&mut self,state:State)->Step {
let mut out=self.b95_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,95,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b95_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,63,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e96_c(&mut self,state:State)->Step {
let mut out=self.b96_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b96_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e97_c(&mut self,state:State)->Step {
let mut out=self.b97_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,97,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b97_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,65,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e98_c(&mut self,state:State)->Step {
let mut out=self.b98_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b98_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e99_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e103_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e104_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e105_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e99_c(&mut self,state:State)->Step {
let mut out=self.b99_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b99_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e100_c(out.state);
out
}
fn e100_c(&mut self,state:State)->Step {
let mut out=self.b100_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b100_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
self.restore(mark);let mut child=self.e101_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e102_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e101_c(&mut self,state:State)->Step {
let mut out=self.b101_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,101,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'"]);}
out
}
fn b101_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LOOKAHEAD",true,42,true,"LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'LOOKAHEAD'");}
out
}
fn e102_c(&mut self,state:State)->Step {
let mut out=self.b102_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,102,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b102_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATIVE_LOOKAHEAD",true,44,true,"NEGATIVE_LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATIVE_LOOKAHEAD'");}
out
}
fn e103_c(&mut self,state:State)->Step {
let mut out=self.b103_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,103,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b103_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e104_c(&mut self,state:State)->Step {
let mut out=self.b104_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b104_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r10_c(out.state);
out
}
fn e105_c(&mut self,state:State)->Step {
let mut out=self.b105_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,105,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b105_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e106_c(&mut self,state:State)->Step {
let mut out=self.b106_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CAPTURE'"]);}
out
}
fn b106_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e107_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e108_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e109_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e110_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e111_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e112_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e107_c(&mut self,state:State)->Step {
let mut out=self.b107_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,107,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'CAPTURE'"]);}
out
}
fn b107_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CAPTURE",true,36,true,"CAPTURE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CAPTURE'");}
out
}
fn e108_c(&mut self,state:State)->Step {
let mut out=self.b108_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,108,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b108_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e109_c(&mut self,state:State)->Step {
let mut out=self.b109_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,109,14);
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser"]);}
out
}
fn b109_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e110_c(&mut self,state:State)->Step {
let mut out=self.b110_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,110,14);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b110_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
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
out=self.r10_c(out.state);
out
}
fn e112_c(&mut self,state:State)->Step {
let mut out=self.b112_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,112,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b112_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e113_c(&mut self,state:State)->Step {
let mut out=self.b113_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'SAME_AS'"]);}
out
}
fn b113_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e114_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e115_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e116_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e117_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e114_c(&mut self,state:State)->Step {
let mut out=self.b114_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,114,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'SAME_AS'"]);}
out
}
fn b114_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"SAME_AS",true,46,true,"SAME_AS");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'SAME_AS'");}
out
}
fn e115_c(&mut self,state:State)->Step {
let mut out=self.b115_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,115,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b115_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e116_c(&mut self,state:State)->Step {
let mut out=self.b116_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,116,14);
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser"]);}
out
}
fn b116_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e117_c(&mut self,state:State)->Step {
let mut out=self.b117_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,117,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b117_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e118_c(&mut self,state:State)->Step {
let mut out=self.b118_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "','", "'CHAR_RANGE'"]);}
out
}
fn b118_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e119_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e120_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e121_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e122_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e123_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e124_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e119_c(&mut self,state:State)->Step {
let mut out=self.b119_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,119,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'"]);}
out
}
fn b119_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CHAR_RANGE",true,37,true,"CHAR_RANGE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CHAR_RANGE'");}
out
}
fn e120_c(&mut self,state:State)->Step {
let mut out=self.b120_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,120,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b120_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e121_c(&mut self,state:State)->Step {
let mut out=self.b121_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,121,14);
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b121_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e122_c(&mut self,state:State)->Step {
let mut out=self.b122_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,122,14);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b122_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e123_c(&mut self,state:State)->Step {
let mut out=self.b123_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,123,14);
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b123_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e124_c(&mut self,state:State)->Step {
let mut out=self.b124_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,124,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b124_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e125_c(&mut self,state:State)->Step {
let mut out=self.b125_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'NEGATION'"]);}
out
}
fn b125_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e126_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e127_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e128_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e129_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e126_c(&mut self,state:State)->Step {
let mut out=self.b126_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,126,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'"]);}
out
}
fn b126_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATION",true,43,true,"NEGATION");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATION'");}
out
}
fn e127_c(&mut self,state:State)->Step {
let mut out=self.b127_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,127,14);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b127_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e128_c(&mut self,state:State)->Step {
let mut out=self.b128_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,128,14);
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser"]);}
out
}
fn b128_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e129_c(&mut self,state:State)->Step {
let mut out=self.b129_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,129,14);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b129_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e130_c(&mut self,state:State)->Step {
let mut out=self.b130_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b130_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r64_c(out.state);
out
}
fn e131_c(&mut self,state:State)->Step {
let mut out=self.b131_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b131_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,85))) {self.guard_e132_c(out.state.begin())} else {self.e132_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e133_c(out.state.begin())} else {self.e133_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,76))) {self.guard_e134_c(out.state.begin())} else {self.e134_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,78))) {self.guard_e135_c(out.state.begin())} else {self.e135_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e136_c(out.state.begin())} else {self.e136_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,67))) {self.guard_e137_c(out.state.begin())} else {self.e137_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,82))) {self.guard_e138_c(out.state.begin())} else {self.e138_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65))) {self.guard_e139_c(out.state.begin())} else {self.e139_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,65))) {self.guard_e140_c(out.state.begin())} else {self.e140_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e141_c(out.state.begin())} else {self.e141_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e142_c(out.state.begin())} else {self.e142_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e143_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e132_c(&mut self,state:State)->Step {
let mut out=self.b132_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b132_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r16_c(out.state);
out
}
fn e133_c(&mut self,state:State)->Step {
let mut out=self.b133_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b133_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r17_c(out.state);
out
}
fn e134_c(&mut self,state:State)->Step {
let mut out=self.b134_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b134_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r18_c(out.state);
out
}
fn e135_c(&mut self,state:State)->Step {
let mut out=self.b135_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b135_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r19_c(out.state);
out
}
fn e136_c(&mut self,state:State)->Step {
let mut out=self.b136_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b136_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r20_c(out.state);
out
}
fn e137_c(&mut self,state:State)->Step {
let mut out=self.b137_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b137_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r21_c(out.state);
out
}
fn e138_c(&mut self,state:State)->Step {
let mut out=self.b138_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b138_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r22_c(out.state);
out
}
fn e139_c(&mut self,state:State)->Step {
let mut out=self.b139_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b139_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r23_c(out.state);
out
}
fn e140_c(&mut self,state:State)->Step {
let mut out=self.b140_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b140_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r24_c(out.state);
out
}
fn e141_c(&mut self,state:State)->Step {
let mut out=self.b141_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b141_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r25_c(out.state);
out
}
fn e142_c(&mut self,state:State)->Step {
let mut out=self.b142_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b142_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r26_c(out.state);
out
}
fn e143_c(&mut self,state:State)->Step {
let mut out=self.b143_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b143_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r64_c(out.state);
out
}
fn e144_c(&mut self,state:State)->Step {
let mut out=self.b144_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b144_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e145_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e146_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e147_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e148_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e145_c(&mut self,state:State)->Step {
let mut out=self.b145_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,145,16);
} else {self.display_failures(state.consumed.max(state.matched),&["'UNTIL'"]);}
out
}
fn b145_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"UNTIL",true,47,true,"UNTIL");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'UNTIL'");}
out
}
fn e146_c(&mut self,state:State)->Step {
let mut out=self.b146_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,146,16);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b146_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e147_c(&mut self,state:State)->Step {
let mut out=self.b147_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,147,16);
out.events=self.event(Event::Capture {site:25,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b147_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e148_c(&mut self,state:State)->Step {
let mut out=self.b148_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,148,16);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b148_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e149_c(&mut self,state:State)->Step {
let mut out=self.b149_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b149_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e150_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e151_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e152_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e153_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e150_c(&mut self,state:State)->Step {
let mut out=self.b150_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,150,17);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATION'"]);}
out
}
fn b150_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATION",true,43,true,"NEGATION");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATION'");}
out
}
fn e151_c(&mut self,state:State)->Step {
let mut out=self.b151_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,151,17);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b151_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e152_c(&mut self,state:State)->Step {
let mut out=self.b152_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,152,17);
out.events=self.event(Event::Capture {site:26,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b152_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e153_c(&mut self,state:State)->Step {
let mut out=self.b153_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,153,17);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b153_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e154_c(&mut self,state:State)->Step {
let mut out=self.b154_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b154_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e155_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e156_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e157_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e158_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e155_c(&mut self,state:State)->Step {
let mut out=self.b155_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,155,18);
} else {self.display_failures(state.consumed.max(state.matched),&["'LOOKAHEAD'"]);}
out
}
fn b155_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"LOOKAHEAD",true,42,true,"LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'LOOKAHEAD'");}
out
}
fn e156_c(&mut self,state:State)->Step {
let mut out=self.b156_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,156,18);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b156_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e157_c(&mut self,state:State)->Step {
let mut out=self.b157_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,157,18);
out.events=self.event(Event::Capture {site:27,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b157_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e158_c(&mut self,state:State)->Step {
let mut out=self.b158_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,158,18);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b158_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e159_c(&mut self,state:State)->Step {
let mut out=self.b159_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
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
let child=self.e163_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e160_c(&mut self,state:State)->Step {
let mut out=self.b160_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,160,19);
} else {self.display_failures(state.consumed.max(state.matched),&["'NEGATIVE_LOOKAHEAD'"]);}
out
}
fn b160_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"NEGATIVE_LOOKAHEAD",true,44,true,"NEGATIVE_LOOKAHEAD");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'NEGATIVE_LOOKAHEAD'");}
out
}
fn e161_c(&mut self,state:State)->Step {
let mut out=self.b161_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,161,19);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b161_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e162_c(&mut self,state:State)->Step {
let mut out=self.b162_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,162,19);
out.events=self.event(Event::Capture {site:28,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b162_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e163_c(&mut self,state:State)->Step {
let mut out=self.b163_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,163,19);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b163_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e164_c(&mut self,state:State)->Step {
let mut out=self.b164_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b164_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
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
let child=self.e170_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e165_c(&mut self,state:State)->Step {
let mut out=self.b165_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,165,20);
} else {self.display_failures(state.consumed.max(state.matched),&["'CHAR_RANGE'"]);}
out
}
fn b165_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CHAR_RANGE",true,37,true,"CHAR_RANGE");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CHAR_RANGE'");}
out
}
fn e166_c(&mut self,state:State)->Step {
let mut out=self.b166_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,166,20);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b166_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e167_c(&mut self,state:State)->Step {
let mut out=self.b167_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,167,20);
out.events=self.event(Event::Capture {site:29,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b167_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e168_c(&mut self,state:State)->Step {
let mut out=self.b168_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,168,20);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b168_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e169_c(&mut self,state:State)->Step {
let mut out=self.b169_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,169,20);
out.events=self.event(Event::Capture {site:30,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b169_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e170_c(&mut self,state:State)->Step {
let mut out=self.b170_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,170,20);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b170_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e171_c(&mut self,state:State)->Step {
let mut out=self.b171_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b171_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e172_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e173_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e174_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e175_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e172_c(&mut self,state:State)->Step {
let mut out=self.b172_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,172,21);
} else {self.display_failures(state.consumed.max(state.matched),&["'CI'"]);}
out
}
fn b172_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"CI",true,38,true,"CI");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'CI'");}
out
}
fn e173_c(&mut self,state:State)->Step {
let mut out=self.b173_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,173,21);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b173_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e174_c(&mut self,state:State)->Step {
let mut out=self.b174_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,174,21);
out.events=self.event(Event::Capture {site:31,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b174_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e175_c(&mut self,state:State)->Step {
let mut out=self.b175_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,175,21);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b175_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e176_c(&mut self,state:State)->Step {
let mut out=self.b176_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b176_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e177_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e178_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e179_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e180_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e177_c(&mut self,state:State)->Step {
let mut out=self.b177_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,177,22);
} else {self.display_failures(state.consumed.max(state.matched),&["'REGEX'"]);}
out
}
fn b177_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"REGEX",true,45,true,"REGEX");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'REGEX'");}
out
}
fn e178_c(&mut self,state:State)->Step {
let mut out=self.b178_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,178,22);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b178_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e179_c(&mut self,state:State)->Step {
let mut out=self.b179_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,179,22);
out.events=self.event(Event::Capture {site:32,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b179_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e180_c(&mut self,state:State)->Step {
let mut out=self.b180_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,180,22);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b180_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e181_c(&mut self,state:State)->Step {
let mut out=self.b181_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b181_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e182_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e183_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e184_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
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
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e182_c(&mut self,state:State)->Step {
let mut out=self.b182_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,182,23);
} else {self.display_failures(state.consumed.max(state.matched),&["'ADAPTER'"]);}
out
}
fn b182_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ADAPTER",true,34,true,"ADAPTER");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ADAPTER'");}
out
}
fn e183_c(&mut self,state:State)->Step {
let mut out=self.b183_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,183,23);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b183_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e184_c(&mut self,state:State)->Step {
let mut out=self.b184_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,184,23);
out.events=self.event(Event::Capture {site:33,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b184_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e185_c(&mut self,state:State)->Step {
let mut out=self.b185_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,185,23);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b185_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e186_c(&mut self,state:State)->Step {
let mut out=self.b186_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,186,23);
} else {self.display_failures(state.consumed.max(state.matched),&["'version'"]);}
out
}
fn b186_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"version",true,62,true,"version");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'version'");}
out
}
fn e187_c(&mut self,state:State)->Step {
let mut out=self.b187_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,187,23);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b187_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e188_c(&mut self,state:State)->Step {
let mut out=self.b188_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,188,23);
out.events=self.event(Event::Capture {site:34,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser", "__CaptureSite"]);}
out
}
fn b188_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_5::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e189_c(&mut self,state:State)->Step {
let mut out=self.b189_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,189,23);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b189_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e190_c(&mut self,state:State)->Step {
let mut out=self.b190_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b190_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e191_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e191_c(&mut self,state:State)->Step {
let mut out=self.b191_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,191,24);
} else {self.display_failures(state.consumed.max(state.matched),&["'ANY'"]);}
out
}
fn b191_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ANY",true,35,true,"ANY");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ANY'");}
out
}
fn e192_c(&mut self,state:State)->Step {
let mut out=self.b192_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b192_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e193_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e193_c(&mut self,state:State)->Step {
let mut out=self.b193_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,193,25);
} else {self.display_failures(state.consumed.max(state.matched),&["'EOF'"]);}
out
}
fn b193_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EOF",true,40,true,"EOF");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'EOF'");}
out
}
fn e194_c(&mut self,state:State)->Step {
let mut out=self.b194_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b194_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e195_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e195_c(&mut self,state:State)->Step {
let mut out=self.b195_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,195,26);
} else {self.display_failures(state.consumed.max(state.matched),&["'EMPTY'"]);}
out
}
fn b195_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"EMPTY",true,39,true,"EMPTY");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'EMPTY'");}
out
}
fn e196_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:196,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b196_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b196_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e197_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e199_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e200_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e201_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e202_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e197_c(&mut self,state:State)->Step {
let mut out=self.b197_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b197_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e198_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:35,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e198_c(&mut self,state:State)->Step {
let mut out=self.b198_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b198_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r28_c(out.state);
out
}
fn e199_c(&mut self,state:State)->Step {
let mut out=self.b199_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,199,27);
out.events=self.event(Event::Capture {site:36,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b199_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e200_c(&mut self,state:State)->Step {
let mut out=self.b200_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,200,27);
} else {self.display_failures(state.consumed.max(state.matched),&["'::='"]);}
out
}
fn b200_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"::=",true,8,false,"::=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'::='");}
out
}
fn e201_c(&mut self,state:State)->Step {
let mut out=self.b201_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:37,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b201_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r65_c(out.state);
out
}
fn e202_c(&mut self,state:State)->Step {
let mut out=self.b202_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,202,27);
} else {self.display_failures(state.consumed.max(state.matched),&["';'"]);}
out
}
fn b202_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,";",true,9,false,";");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"';'");}
out
}
fn e203_c(&mut self,state:State)->Step {
let mut out=self.b203_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b203_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e204_c(out.state.begin())} else {self.e204_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e205_c(out.state.begin())} else {self.e205_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e206_c(out.state.begin())} else {self.e206_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e207_c(out.state.begin())} else {self.e207_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e208_c(out.state.begin())} else {self.e208_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e209_c(out.state.begin())} else {self.e209_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e210_c(out.state.begin())} else {self.e210_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e211_c(out.state.begin())} else {self.e211_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e212_c(out.state.begin())} else {self.e212_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e213_c(out.state.begin())} else {self.e213_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e214_c(out.state.begin())} else {self.e214_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e215_c(out.state.begin())} else {self.e215_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e216_c(out.state.begin())} else {self.e216_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e217_c(out.state.begin())} else {self.e217_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e218_c(out.state.begin())} else {self.e218_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e219_c(out.state.begin())} else {self.e219_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e220_c(out.state.begin())} else {self.e220_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e221_c(out.state.begin())} else {self.e221_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e222_c(out.state.begin())} else {self.e222_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,64))) {self.guard_e223_c(out.state.begin())} else {self.e223_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e204_c(&mut self,state:State)->Step {
let mut out=self.b204_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b204_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r29_c(out.state);
out
}
fn e205_c(&mut self,state:State)->Step {
let mut out=self.b205_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b205_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r30_c(out.state);
out
}
fn e206_c(&mut self,state:State)->Step {
let mut out=self.b206_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b206_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r31_c(out.state);
out
}
fn e207_c(&mut self,state:State)->Step {
let mut out=self.b207_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b207_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r32_c(out.state);
out
}
fn e208_c(&mut self,state:State)->Step {
let mut out=self.b208_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b208_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r33_c(out.state);
out
}
fn e209_c(&mut self,state:State)->Step {
let mut out=self.b209_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b209_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r34_c(out.state);
out
}
fn e210_c(&mut self,state:State)->Step {
let mut out=self.b210_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b210_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r35_c(out.state);
out
}
fn e211_c(&mut self,state:State)->Step {
let mut out=self.b211_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b211_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r36_c(out.state);
out
}
fn e212_c(&mut self,state:State)->Step {
let mut out=self.b212_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b212_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r37_c(out.state);
out
}
fn e213_c(&mut self,state:State)->Step {
let mut out=self.b213_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b213_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r38_c(out.state);
out
}
fn e214_c(&mut self,state:State)->Step {
let mut out=self.b214_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b214_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r39_c(out.state);
out
}
fn e215_c(&mut self,state:State)->Step {
let mut out=self.b215_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b215_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r40_c(out.state);
out
}
fn e216_c(&mut self,state:State)->Step {
let mut out=self.b216_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b216_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r41_c(out.state);
out
}
fn e217_c(&mut self,state:State)->Step {
let mut out=self.b217_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b217_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r42_c(out.state);
out
}
fn e218_c(&mut self,state:State)->Step {
let mut out=self.b218_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b218_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r43_c(out.state);
out
}
fn e219_c(&mut self,state:State)->Step {
let mut out=self.b219_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b219_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r44_c(out.state);
out
}
fn e220_c(&mut self,state:State)->Step {
let mut out=self.b220_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b220_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r45_c(out.state);
out
}
fn e221_c(&mut self,state:State)->Step {
let mut out=self.b221_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b221_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r46_c(out.state);
out
}
fn e222_c(&mut self,state:State)->Step {
let mut out=self.b222_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b222_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r47_c(out.state);
out
}
fn e223_c(&mut self,state:State)->Step {
let mut out=self.b223_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b223_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r48_c(out.state);
out
}
fn e224_c(&mut self,state:State)->Step {
let mut out=self.b224_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b224_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e225_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e225_c(&mut self,state:State)->Step {
let mut out=self.b225_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,225,29);
} else {self.display_failures(state.consumed.max(state.matched),&["'@root'"]);}
out
}
fn b225_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@root",true,29,true,"@root");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@root'");}
out
}
fn e226_c(&mut self,state:State)->Step {
let mut out=self.b226_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b226_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e227_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e228_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e229_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e230_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e242_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e227_c(&mut self,state:State)->Step {
let mut out=self.b227_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,227,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'@mapping'"]);}
out
}
fn b227_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@mapping",true,24,true,"@mapping");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@mapping'");}
out
}
fn e228_c(&mut self,state:State)->Step {
let mut out=self.b228_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,228,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b228_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e229_c(&mut self,state:State)->Step {
let mut out=self.b229_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,229,30);
out.events=self.event(Event::Capture {site:38,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["CLASS_NAMEParser", "__CaptureSite"]);}
out
}
fn b229_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_4::<false>(out.state,"CLASS_NAME","CLASS_NAMEParser");
out
}
fn e230_c(&mut self,state:State)->Step {
let mut out=self.b230_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'params'"]);}
out
}
fn b230_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e231_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e231_c(&mut self,state:State)->Step {
let mut out=self.b231_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'['", "']'", "'params'"]);}
out
}
fn b231_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e232_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e233_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e234_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e235_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e236_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e237_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e241_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e232_c(&mut self,state:State)->Step {
let mut out=self.b232_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,232,30);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b232_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e233_c(&mut self,state:State)->Step {
let mut out=self.b233_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,233,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'params'"]);}
out
}
fn b233_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"params",true,57,true,"params");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'params'");}
out
}
fn e234_c(&mut self,state:State)->Step {
let mut out=self.b234_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,234,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b234_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e235_c(&mut self,state:State)->Step {
let mut out=self.b235_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,235,30);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b235_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,48,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e236_c(&mut self,state:State)->Step {
let mut out=self.b236_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,236,30);
out.events=self.event(Event::Capture {site:39,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b236_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e237_c(&mut self,state:State)->Step {
let mut out=self.b237_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b237_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e238_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e238_c(&mut self,state:State)->Step {
let mut out=self.b238_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b238_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e239_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e240_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e239_c(&mut self,state:State)->Step {
let mut out=self.b239_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,239,30);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b239_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e240_c(&mut self,state:State)->Step {
let mut out=self.b240_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,240,30);
out.events=self.event(Event::Capture {site:40,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b240_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e241_c(&mut self,state:State)->Step {
let mut out=self.b241_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,241,30);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b241_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,49,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e242_c(&mut self,state:State)->Step {
let mut out=self.b242_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,242,30);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b242_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e243_c(&mut self,state:State)->Step {
let mut out=self.b243_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b243_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e244_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e245_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e246_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e247_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e251_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e244_c(&mut self,state:State)->Step {
let mut out=self.b244_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,244,31);
} else {self.display_failures(state.consumed.max(state.matched),&["'@eval'"]);}
out
}
fn b244_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@eval",true,19,true,"@eval");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@eval'");}
out
}
fn e245_c(&mut self,state:State)->Step {
let mut out=self.b245_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,245,31);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b245_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e246_c(&mut self,state:State)->Step {
let mut out=self.b246_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:41,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b246_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e247_c(&mut self,state:State)->Step {
let mut out=self.b247_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b247_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e248_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e248_c(&mut self,state:State)->Step {
let mut out=self.b248_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
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
out.events=self.relabel_token(out.events,249,31);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b249_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e250_c(&mut self,state:State)->Step {
let mut out=self.b250_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:42,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["':'"]);}
out
}
fn b250_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r7_c(out.state);
out
}
fn e251_c(&mut self,state:State)->Step {
let mut out=self.b251_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,251,31);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b251_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e252_c(&mut self,state:State)->Step {
let mut out=self.b252_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b252_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e253_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e254_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e253_c(&mut self,state:State)->Step {
let mut out=self.b253_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,253,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'@whitespace'"]);}
out
}
fn b253_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@whitespace",true,33,true,"@whitespace");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@whitespace'");}
out
}
fn e254_c(&mut self,state:State)->Step {
let mut out=self.b254_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
out
}
fn b254_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e255_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e255_c(&mut self,state:State)->Step {
let mut out=self.b255_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'"]);}
out
}
fn b255_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e256_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e257_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e258_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e256_c(&mut self,state:State)->Step {
let mut out=self.b256_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,256,32);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b256_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e257_c(&mut self,state:State)->Step {
let mut out=self.b257_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,257,32);
out.events=self.event(Event::Capture {site:43,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b257_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e258_c(&mut self,state:State)->Step {
let mut out=self.b258_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,258,32);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b258_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e259_c(&mut self,state:State)->Step {
let mut out=self.b259_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b259_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e260_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e261_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e262_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e263_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e264_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e265_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e260_c(&mut self,state:State)->Step {
let mut out=self.b260_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,260,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'@interleave'"]);}
out
}
fn b260_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@interleave",true,21,true,"@interleave");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@interleave'");}
out
}
fn e261_c(&mut self,state:State)->Step {
let mut out=self.b261_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,261,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b261_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e262_c(&mut self,state:State)->Step {
let mut out=self.b262_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,262,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'profile'"]);}
out
}
fn b262_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"profile",true,58,true,"profile");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'profile'");}
out
}
fn e263_c(&mut self,state:State)->Step {
let mut out=self.b263_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,263,33);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b263_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e264_c(&mut self,state:State)->Step {
let mut out=self.b264_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,264,33);
out.events=self.event(Event::Capture {site:44,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b264_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e265_c(&mut self,state:State)->Step {
let mut out=self.b265_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,265,33);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b265_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e266_c(&mut self,state:State)->Step {
let mut out=self.b266_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b266_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e267_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e268_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e269_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e270_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e271_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e272_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e267_c(&mut self,state:State)->Step {
let mut out=self.b267_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,267,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'@backref'"]);}
out
}
fn b267_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@backref",true,13,true,"@backref");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@backref'");}
out
}
fn e268_c(&mut self,state:State)->Step {
let mut out=self.b268_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,268,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b268_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e269_c(&mut self,state:State)->Step {
let mut out=self.b269_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,269,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'name'"]);}
out
}
fn b269_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"name",true,56,true,"name");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'name'");}
out
}
fn e270_c(&mut self,state:State)->Step {
let mut out=self.b270_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,270,34);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b270_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e271_c(&mut self,state:State)->Step {
let mut out=self.b271_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,271,34);
out.events=self.event(Event::Capture {site:45,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b271_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e272_c(&mut self,state:State)->Step {
let mut out=self.b272_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,272,34);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b272_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e273_c(&mut self,state:State)->Step {
let mut out=self.b273_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b273_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e274_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e275_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e276_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e277_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e278_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e279_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e274_c(&mut self,state:State)->Step {
let mut out=self.b274_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,274,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'@scopeTree'"]);}
out
}
fn b274_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@scopeTree",true,30,true,"@scopeTree");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@scopeTree'");}
out
}
fn e275_c(&mut self,state:State)->Step {
let mut out=self.b275_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,275,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b275_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e276_c(&mut self,state:State)->Step {
let mut out=self.b276_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,276,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'mode'"]);}
out
}
fn b276_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"mode",true,55,true,"mode");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'mode'");}
out
}
fn e277_c(&mut self,state:State)->Step {
let mut out=self.b277_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,277,35);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b277_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e278_c(&mut self,state:State)->Step {
let mut out=self.b278_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,278,35);
out.events=self.event(Event::Capture {site:46,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b278_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e279_c(&mut self,state:State)->Step {
let mut out=self.b279_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,279,35);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b279_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e280_c(&mut self,state:State)->Step {
let mut out=self.b280_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b280_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e281_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e282_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e283_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e284_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e285_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e286_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e292_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e281_c(&mut self,state:State)->Step {
let mut out=self.b281_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,281,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'@declares'"]);}
out
}
fn b281_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@declares",true,16,true,"@declares");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@declares'");}
out
}
fn e282_c(&mut self,state:State)->Step {
let mut out=self.b282_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,282,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b282_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e283_c(&mut self,state:State)->Step {
let mut out=self.b283_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,283,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'symbol'"]);}
out
}
fn b283_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"symbol",true,59,true,"symbol");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'symbol'");}
out
}
fn e284_c(&mut self,state:State)->Step {
let mut out=self.b284_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,284,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b284_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e285_c(&mut self,state:State)->Step {
let mut out=self.b285_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,285,36);
out.events=self.event(Event::Capture {site:47,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b285_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e286_c(&mut self,state:State)->Step {
let mut out=self.b286_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'description'"]);}
out
}
fn b286_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e287_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e287_c(&mut self,state:State)->Step {
let mut out=self.b287_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','", "'='", "'description'"]);}
out
}
fn b287_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e288_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e289_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e290_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e291_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e288_c(&mut self,state:State)->Step {
let mut out=self.b288_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,288,36);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b288_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e289_c(&mut self,state:State)->Step {
let mut out=self.b289_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,289,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'description'"]);}
out
}
fn b289_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"description",true,51,true,"description");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'description'");}
out
}
fn e290_c(&mut self,state:State)->Step {
let mut out=self.b290_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,290,36);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b290_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e291_c(&mut self,state:State)->Step {
let mut out=self.b291_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,291,36);
out.events=self.event(Event::Capture {site:48,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b291_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e292_c(&mut self,state:State)->Step {
let mut out=self.b292_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,292,36);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b292_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e293_c(&mut self,state:State)->Step {
let mut out=self.b293_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b293_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e294_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e295_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e296_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e297_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e298_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e299_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e294_c(&mut self,state:State)->Step {
let mut out=self.b294_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,294,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'@catalog'"]);}
out
}
fn b294_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@catalog",true,14,true,"@catalog");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@catalog'");}
out
}
fn e295_c(&mut self,state:State)->Step {
let mut out=self.b295_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,295,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b295_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e296_c(&mut self,state:State)->Step {
let mut out=self.b296_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,296,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'context'"]);}
out
}
fn b296_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"context",true,50,true,"context");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'context'");}
out
}
fn e297_c(&mut self,state:State)->Step {
let mut out=self.b297_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,297,37);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b297_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e298_c(&mut self,state:State)->Step {
let mut out=self.b298_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,298,37);
out.events=self.event(Event::Capture {site:49,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b298_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e299_c(&mut self,state:State)->Step {
let mut out=self.b299_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,299,37);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b299_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e300_c(&mut self,state:State)->Step {
let mut out=self.b300_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b300_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e301_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e301_c(&mut self,state:State)->Step {
let mut out=self.b301_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,301,38);
} else {self.display_failures(state.consumed.max(state.matched),&["'@leftAssoc'"]);}
out
}
fn b301_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@leftAssoc",true,22,true,"@leftAssoc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@leftAssoc'");}
out
}
fn e302_c(&mut self,state:State)->Step {
let mut out=self.b302_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b302_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e303_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e303_c(&mut self,state:State)->Step {
let mut out=self.b303_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,303,39);
} else {self.display_failures(state.consumed.max(state.matched),&["'@rightAssoc'"]);}
out
}
fn b303_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@rightAssoc",true,28,true,"@rightAssoc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@rightAssoc'");}
out
}
fn e304_c(&mut self,state:State)->Step {
let mut out=self.b304_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b304_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e305_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e305_c(&mut self,state:State)->Step {
let mut out=self.b305_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,305,40);
} else {self.display_failures(state.consumed.max(state.matched),&["'@longestChoice'"]);}
out
}
fn b305_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@longestChoice",true,23,true,"@longestChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@longestChoice'");}
out
}
fn e306_c(&mut self,state:State)->Step {
let mut out=self.b306_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b306_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e307_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e307_c(&mut self,state:State)->Step {
let mut out=self.b307_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,307,41);
} else {self.display_failures(state.consumed.max(state.matched),&["'@predictiveChoice'"]);}
out
}
fn b307_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@predictiveChoice",true,26,true,"@predictiveChoice");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@predictiveChoice'");}
out
}
fn e308_c(&mut self,state:State)->Step {
let mut out=self.b308_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b308_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e309_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e310_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e311_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e312_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e313_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e314_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e309_c(&mut self,state:State)->Step {
let mut out=self.b309_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,309,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'@precedence'"]);}
out
}
fn b309_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@precedence",true,25,true,"@precedence");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@precedence'");}
out
}
fn e310_c(&mut self,state:State)->Step {
let mut out=self.b310_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,310,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b310_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e311_c(&mut self,state:State)->Step {
let mut out=self.b311_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,311,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'level'"]);}
out
}
fn b311_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"level",true,54,true,"level");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'level'");}
out
}
fn e312_c(&mut self,state:State)->Step {
let mut out=self.b312_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,312,42);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b312_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e313_c(&mut self,state:State)->Step {
let mut out=self.b313_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,313,42);
out.events=self.event(Event::Capture {site:50,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["UNSIGNED_INTEGERParser", "__CaptureSite"]);}
out
}
fn b313_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_5::<false>(out.state,"UNSIGNED_INTEGER","UNSIGNED_INTEGERParser");
out
}
fn e314_c(&mut self,state:State)->Step {
let mut out=self.b314_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,314,42);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b314_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e315_c(&mut self,state:State)->Step {
let mut out=self.b315_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b315_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e316_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e317_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e318_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e319_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e316_c(&mut self,state:State)->Step {
let mut out=self.b316_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,316,43);
} else {self.display_failures(state.consumed.max(state.matched),&["'@doc'"]);}
out
}
fn b316_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@doc",true,17,true,"@doc");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@doc'");}
out
}
fn e317_c(&mut self,state:State)->Step {
let mut out=self.b317_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,317,43);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b317_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e318_c(&mut self,state:State)->Step {
let mut out=self.b318_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,318,43);
out.events=self.event(Event::Capture {site:51,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b318_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e319_c(&mut self,state:State)->Step {
let mut out=self.b319_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,319,43);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b319_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e320_c(&mut self,state:State)->Step {
let mut out=self.b320_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b320_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e321_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e322_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e323_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e330_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e321_c(&mut self,state:State)->Step {
let mut out=self.b321_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,321,44);
} else {self.display_failures(state.consumed.max(state.matched),&["'@recovery'"]);}
out
}
fn b321_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@recovery",true,27,true,"@recovery");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@recovery'");}
out
}
fn e322_c(&mut self,state:State)->Step {
let mut out=self.b322_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,322,44);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b322_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e323_c(&mut self,state:State)->Step {
let mut out=self.b323_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b323_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.e324_c(out.state);
out
}
fn e324_c(&mut self,state:State)->Step {
let mut out=self.b324_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b324_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,115))) {self.guard_e325_c(out.state.begin())} else {self.e325_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e329_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e325_c(&mut self,state:State)->Step {
let mut out=self.b325_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'='", "'sync'"]);}
out
}
fn b325_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e326_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e327_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e328_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e326_c(&mut self,state:State)->Step {
let mut out=self.b326_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,326,44);
} else {self.display_failures(state.consumed.max(state.matched),&["'sync'"]);}
out
}
fn b326_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"sync",true,60,true,"sync");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'sync'");}
out
}
fn e327_c(&mut self,state:State)->Step {
let mut out=self.b327_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,327,44);
} else {self.display_failures(state.consumed.max(state.matched),&["'='"]);}
out
}
fn b327_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"=",true,10,false,"=");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'='");}
out
}
fn e328_c(&mut self,state:State)->Step {
let mut out=self.b328_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,328,44);
out.events=self.event(Event::Capture {site:52,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b328_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e329_c(&mut self,state:State)->Step {
let mut out=self.b329_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,329,44);
out.events=self.event(Event::Capture {site:53,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b329_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e330_c(&mut self,state:State)->Step {
let mut out=self.b330_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,330,44);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b330_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e331_c(&mut self,state:State)->Step {
let mut out=self.b331_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b331_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e332_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e332_c(&mut self,state:State)->Step {
let mut out=self.b332_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,332,45);
} else {self.display_failures(state.consumed.max(state.matched),&["'@skip'"]);}
out
}
fn b332_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@skip",true,31,true,"@skip");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@skip'");}
out
}
fn e333_c(&mut self,state:State)->Step {
let mut out=self.b333_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b333_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e334_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e335_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e336_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e337_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e341_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e334_c(&mut self,state:State)->Step {
let mut out=self.b334_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,334,46);
} else {self.display_failures(state.consumed.max(state.matched),&["'@commonField'"]);}
out
}
fn b334_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@commonField",true,15,true,"@commonField");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@commonField'");}
out
}
fn e335_c(&mut self,state:State)->Step {
let mut out=self.b335_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,335,46);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b335_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e336_c(&mut self,state:State)->Step {
let mut out=self.b336_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,336,46);
out.events=self.event(Event::Capture {site:54,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b336_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e337_c(&mut self,state:State)->Step {
let mut out=self.b337_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b337_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e338_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e338_c(&mut self,state:State)->Step {
let mut out=self.b338_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b338_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e339_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e340_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e339_c(&mut self,state:State)->Step {
let mut out=self.b339_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,339,46);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b339_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e340_c(&mut self,state:State)->Step {
let mut out=self.b340_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,340,46);
out.events=self.event(Event::Capture {site:55,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b340_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e341_c(&mut self,state:State)->Step {
let mut out=self.b341_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,341,46);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b341_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e342_c(&mut self,state:State)->Step {
let mut out=self.b342_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b342_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e343_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e343_c(&mut self,state:State)->Step {
let mut out=self.b343_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,343,47);
} else {self.display_failures(state.consumed.max(state.matched),&["'@enum'"]);}
out
}
fn b343_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@enum",true,18,true,"@enum");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@enum'");}
out
}
fn e344_c(&mut self,state:State)->Step {
let mut out=self.b344_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b344_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e345_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e346_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e345_c(&mut self,state:State)->Step {
let mut out=self.b345_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,345,48);
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b345_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,12,false,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e346_c(&mut self,state:State)->Step {
let mut out=self.b346_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,346,48);
out.events=self.event(Event::Capture {site:56,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b346_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e347_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:347,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b347_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b347_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e348_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e349_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e348_c(&mut self,state:State)->Step {
let mut out=self.b348_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:57,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b348_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r50_c(out.state);
out
}
fn e349_c(&mut self,state:State)->Step {
let mut out=self.b349_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b349_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e350_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e350_c(&mut self,state:State)->Step {
let mut out=self.b350_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b350_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e351_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e352_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e351_c(&mut self,state:State)->Step {
let mut out=self.b351_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,351,49);
} else {self.display_failures(state.consumed.max(state.matched),&["'|'"]);}
out
}
fn b351_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"|",true,64,false,"|");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'|'");}
out
}
fn e352_c(&mut self,state:State)->Step {
let mut out=self.b352_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:58,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b352_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r50_c(out.state);
out
}
fn e353_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:353,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b353_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b353_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e354_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e354_c(&mut self,state:State)->Step {
let mut out=self.b354_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat", "__CaptureSite"]);}
out
}
fn b354_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e355_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:59,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e355_c(&mut self,state:State)->Step {
let mut out=self.b355_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b355_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r51_c(out.state);
out
}
fn e356_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:356,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b356_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b356_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e357_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e359_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e360_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e362_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e364_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e366_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e357_c(&mut self,state:State)->Step {
let mut out=self.b357_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b357_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e358_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e358_c(&mut self,state:State)->Step {
let mut out=self.b358_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:60,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'('", "')'", "'@typeof'"]);}
out
}
fn b358_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r63_c(out.state);
out
}
fn e359_c(&mut self,state:State)->Step {
let mut out=self.b359_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:61,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b359_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r56_c(out.state);
out
}
fn e360_c(&mut self,state:State)->Step {
let mut out=self.b360_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'*'", "'+'", "'?'"]);}
out
}
fn b360_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e361_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e361_c(&mut self,state:State)->Step {
let mut out=self.b361_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b361_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r52_c(out.state);
out
}
fn e362_c(&mut self,state:State)->Step {
let mut out=self.b362_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'{'", "'}'"]);}
out
}
fn b362_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e363_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e363_c(&mut self,state:State)->Step {
let mut out=self.b363_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b363_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r54_c(out.state);
out
}
fn e364_c(&mut self,state:State)->Step {
let mut out=self.b364_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
out
}
fn b364_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e365_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e365_c(&mut self,state:State)->Step {
let mut out=self.b365_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b365_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r53_c(out.state);
out
}
fn e366_c(&mut self,state:State)->Step {
let mut out=self.b366_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b366_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e367_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e367_c(&mut self,state:State)->Step {
let mut out=self.b367_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b367_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e368_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e369_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e368_c(&mut self,state:State)->Step {
let mut out=self.b368_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,368,51);
} else {self.display_failures(state.consumed.max(state.matched),&["'@'"]);}
out
}
fn b368_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@",true,12,false,"@");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@'");}
out
}
fn e369_c(&mut self,state:State)->Step {
let mut out=self.b369_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,369,51);
out.events=self.event(Event::Capture {site:62,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b369_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e370_c(&mut self,state:State)->Step {
let mut out=self.b370_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b370_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
if !out.state.invert {let predicted=match self.input.cp_at(out.state.begin().position::<false>()).map(|(c,_)|c) {
Some('+')=>0usize,
Some('?')=>1usize,
Some('*')=>2usize,
_=>usize::MAX,};let candidate=match predicted {
0=>Some(self.e371_c(out.state.begin())),
1=>Some(self.e372_c(out.state.begin())),
2=>Some(self.e373_c(out.state.begin())),
_=>None,};
if !DIAG && self.options.predict && predicted==usize::MAX {self.restore(mark);out.ok=false;out.diag=diagnostic;break 'choice out;}
if let Some(mut child)=candidate {if child.ok {
if predicted>0 {let d=self.diag_fail(out.state.begin().position::<false>(),"+");diagnostic=self.diag_join(diagnostic,d);}
if predicted>1 {let d=self.diag_fail(out.state.begin().position::<false>(),"?");diagnostic=self.diag_join(diagnostic,d);}
if predicted>2 {let d=self.diag_fail(out.state.begin().position::<false>(),"*");diagnostic=self.diag_join(diagnostic,d);}
child.state=out.state.commit(child.state);child.diag=self.diag_join(diagnostic,child.diag);break 'choice child;}}self.restore(mark); }
self.restore(mark);let mut child=self.e371_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e372_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e373_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e371_c(&mut self,state:State)->Step {
let mut out=self.b371_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,371,52);
} else {self.display_failures(state.consumed.max(state.matched),&["'+'"]);}
out
}
fn b371_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"+",true,4,false,"+");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'+'");}
out
}
fn e372_c(&mut self,state:State)->Step {
let mut out=self.b372_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,372,52);
} else {self.display_failures(state.consumed.max(state.matched),&["'?'"]);}
out
}
fn b372_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"?",true,11,false,"?");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'?'");}
out
}
fn e373_c(&mut self,state:State)->Step {
let mut out=self.b373_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,373,52);
} else {self.display_failures(state.consumed.max(state.matched),&["'*'"]);}
out
}
fn b373_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"*",true,3,false,"*");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'*'");}
out
}
fn e374_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:374,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b374_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b374_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e375_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e376_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e375_c(&mut self,state:State)->Step {
let mut out=self.b375_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,375,53);
} else {self.display_failures(state.consumed.max(state.matched),&["'%'"]);}
out
}
fn b375_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"%",true,0,false,"%");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'%'");}
out
}
fn e376_c(&mut self,state:State)->Step {
let mut out=self.b376_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b376_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r56_c(out.state);
out
}
fn e377_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:377,state,matched_mode:false,version:0};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b377_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b377_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e378_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e379_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e380_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e385_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e378_c(&mut self,state:State)->Step {
let mut out=self.b378_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,378,54);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b378_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,63,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e379_c(&mut self,state:State)->Step {
let mut out=self.b379_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b379_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r55_c(out.state);
out
}
fn e380_c(&mut self,state:State)->Step {
let mut out=self.b380_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b380_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e381_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e381_c(&mut self,state:State)->Step {
let mut out=self.b381_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b381_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e382_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e383_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e382_c(&mut self,state:State)->Step {
let mut out=self.b382_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,382,54);
} else {self.display_failures(state.consumed.max(state.matched),&["','"]);}
out
}
fn b382_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,",",true,5,false,",");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"','");}
out
}
fn e383_c(&mut self,state:State)->Step {
let mut out=self.b383_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Optional"]);}
out
}
fn b383_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e384_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
if count>=1 {break;}
}
let _=count;
if count>1 {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e384_c(&mut self,state:State)->Step {
let mut out=self.b384_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b384_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r55_c(out.state);
out
}
fn e385_c(&mut self,state:State)->Step {
let mut out=self.b385_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,385,54);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b385_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,65,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e386_c(&mut self,state:State)->Step {
let mut out=self.b386_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b386_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e387_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e388_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e387_c(&mut self,state:State)->Step {
let mut out=self.b387_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,387,55);
} else {self.display_failures(state.consumed.max(state.matched),&["DIGITParser"]);}
out
}
fn b387_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"DIGIT","DIGITParser");
out
}
fn e388_c(&mut self,state:State)->Step {
let mut out=self.b388_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["Repeat"]);}
out
}
fn b388_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e389_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e389_c(&mut self,state:State)->Step {
let mut out=self.b389_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,389,55);
} else {self.display_failures(state.consumed.max(state.matched),&["DIGITParser"]);}
out
}
fn b389_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_0::<false>(out.state,"DIGIT","DIGITParser");
out
}
fn e390_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:390,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b390_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b390_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out='choice: {let mark=self.mark();let mut diagnostic=Diag::NONE;let mut best:Option<(Step,[usize;2])>=None;
let ct=self.input.cp_at(self.skip_0::<false>(out.state.begin()).position::<false>()).map(|(c,_)|c);
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,40))) {self.guard_e391_c(out.state.begin())} else {self.e391_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,91))) {self.guard_e392_c(out.state.begin())} else {self.e392_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,123))) {self.guard_e393_c(out.state.begin())} else {self.e393_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=if self.options.predict && !out.state.invert && !(ct.is_some_and(|c| matches!(c as u32,69))) {self.guard_e394_c(out.state.begin())} else {self.e394_c(out.state.begin())};diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e395_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);let mut child=self.e396_c(out.state.begin());diagnostic=self.diag_join(diagnostic,child.diag);
if child.ok {child.state=out.state.commit(child.state);child.diag=diagnostic;break 'choice child;}
self.restore(mark);break 'choice if let Some((mut child,effects))=best {self.replay_effects(effects);child.diag=diagnostic;child} else {out.ok=false;out.diag=diagnostic;out};
};
out
}
fn e391_c(&mut self,state:State)->Step {
let mut out=self.b391_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b391_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r57_c(out.state);
out
}
fn e392_c(&mut self,state:State)->Step {
let mut out=self.b392_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b392_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r58_c(out.state);
out
}
fn e393_c(&mut self,state:State)->Step {
let mut out=self.b393_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b393_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r59_c(out.state);
out
}
fn e394_c(&mut self,state:State)->Step {
let mut out=self.b394_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b394_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r60_c(out.state);
out
}
fn e395_c(&mut self,state:State)->Step {
let mut out=self.b395_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b395_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r61_c(out.state);
out
}
fn e396_c(&mut self,state:State)->Step {
let mut out=self.b396_c(state);
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:false,text:true});
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b396_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r62_c(out.state);
out
}
fn e397_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:397,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b397_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b397_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e398_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e399_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e400_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e398_c(&mut self,state:State)->Step {
let mut out=self.b398_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,398,57);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b398_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e399_c(&mut self,state:State)->Step {
let mut out=self.b399_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:63,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b399_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r65_c(out.state);
out
}
fn e400_c(&mut self,state:State)->Step {
let mut out=self.b400_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,400,57);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b400_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e401_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:401,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b401_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b401_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e402_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e403_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e404_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e402_c(&mut self,state:State)->Step {
let mut out=self.b402_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,402,58);
} else {self.display_failures(state.consumed.max(state.matched),&["'['"]);}
out
}
fn b402_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"[",true,48,false,"[");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'['");}
out
}
fn e403_c(&mut self,state:State)->Step {
let mut out=self.b403_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:64,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b403_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r65_c(out.state);
out
}
fn e404_c(&mut self,state:State)->Step {
let mut out=self.b404_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,404,58);
} else {self.display_failures(state.consumed.max(state.matched),&["']'"]);}
out
}
fn b404_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"]",true,49,false,"]");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"']'");}
out
}
fn e405_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:405,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b405_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b405_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e406_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e407_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e408_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e406_c(&mut self,state:State)->Step {
let mut out=self.b406_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,406,59);
} else {self.display_failures(state.consumed.max(state.matched),&["'{'"]);}
out
}
fn b406_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"{",true,63,false,"{");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'{'");}
out
}
fn e407_c(&mut self,state:State)->Step {
let mut out=self.b407_c(state);
if out.ok {
out.events=self.event(Event::Capture {site:65,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["__CaptureSite"]);}
out
}
fn b407_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r65_c(out.state);
out
}
fn e408_c(&mut self,state:State)->Step {
let mut out=self.b408_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,408,59);
} else {self.display_failures(state.consumed.max(state.matched),&["'}'"]);}
out
}
fn b408_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"}",true,65,false,"}");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'}'");}
out
}
fn e409_c(&mut self,state:State)->Step {
let mut out=self.b409_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b409_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e410_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e411_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e412_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e413_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e410_c(&mut self,state:State)->Step {
let mut out=self.b410_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,410,60);
} else {self.display_failures(state.consumed.max(state.matched),&["'ERROR'"]);}
out
}
fn b410_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"ERROR",true,41,true,"ERROR");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'ERROR'");}
out
}
fn e411_c(&mut self,state:State)->Step {
let mut out=self.b411_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,411,60);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b411_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e412_c(&mut self,state:State)->Step {
let mut out=self.b412_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,412,60);
out.events=self.event(Event::Capture {site:66,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b412_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e413_c(&mut self,state:State)->Step {
let mut out=self.b413_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,413,60);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b413_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e414_c(&mut self,state:State)->Step {
let mut out=self.b414_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b414_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e415_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e415_c(&mut self,state:State)->Step {
let mut out=self.b415_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,415,61);
out.events=self.event(Event::Capture {site:67,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["STRINGParser", "__CaptureSite"]);}
out
}
fn b415_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_3::<false>(out.state,"STRING","STRINGParser");
out
}
fn e416_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:416,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b416_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b416_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e417_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e421_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e417_c(&mut self,state:State)->Step {
let mut out=self.b417_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'", "__CaptureSite"]);}
out
}
fn b417_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e418_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
child.events=self.event(Event::Capture {site:68,span:[before.consumed,child.state.consumed],child:child.events,token_extent:false});
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e418_c(&mut self,state:State)->Step {
let mut out=self.b418_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b418_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e419_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e420_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e419_c(&mut self,state:State)->Step {
let mut out=self.b419_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,419,62);
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser"]);}
out
}
fn b419_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e420_c(&mut self,state:State)->Step {
let mut out=self.b420_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,420,62);
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b420_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,6,false,".");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'.'");}
out
}
fn e421_c(&mut self,state:State)->Step {
let mut out=self.b421_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,421,62);
out.events=self.event(Event::Capture {site:69,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b421_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e422_c(&mut self,state:State)->Step {
let mut out=self.b422_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b422_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e423_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e424_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e425_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e426_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e423_c(&mut self,state:State)->Step {
let mut out=self.b423_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,423,63);
} else {self.display_failures(state.consumed.max(state.matched),&["'@typeof'"]);}
out
}
fn b423_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"@typeof",true,32,true,"@typeof");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'@typeof'");}
out
}
fn e424_c(&mut self,state:State)->Step {
let mut out=self.b424_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,424,63);
} else {self.display_failures(state.consumed.max(state.matched),&["'('"]);}
out
}
fn b424_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,"(",true,1,false,"(");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'('");}
out
}
fn e425_c(&mut self,state:State)->Step {
let mut out=self.b425_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,425,63);
out.events=self.event(Event::Capture {site:70,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b425_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e426_c(&mut self,state:State)->Step {
let mut out=self.b426_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,426,63);
} else {self.display_failures(state.consumed.max(state.matched),&["')'"]);}
out
}
fn b426_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,")",true,2,false,")");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"')'");}
out
}
fn e427_c(&mut self,state:State)->Step {
let mut out=self.b427_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b427_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e428_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e429_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e428_c(&mut self,state:State)->Step {
let mut out=self.b428_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,428,64);
out.events=self.event(Event::Capture {site:71,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b428_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e429_c(&mut self,state:State)->Step {
let mut out=self.b429_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b429_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out.state=state.begin();
let mut count=0usize;loop {let mark=self.mark();let before=out.state;
let mut child=self.e430_c(out.state);out.diag=self.diag_join(out.diag,child.diag);out.state=child.state;if !child.ok {self.restore(mark);break;}
out.events=self.join(out.events,child.events);count+=1;
if before.position::<false>()==out.state.position::<false>() {break;}
}
let _=count;
if false {out.ok=false;out.state=state;let d=self.diag_fail(state.position::<false>(),"expression");out.diag=self.diag_join(out.diag,d);}
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e430_c(&mut self,state:State)->Step {
let mut out=self.b430_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b430_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e431_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e432_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e431_c(&mut self,state:State)->Step {
let mut out=self.b431_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,431,64);
} else {self.display_failures(state.consumed.max(state.matched),&["'.'"]);}
out
}
fn b431_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.literal::<false>(out.state,".",true,6,false,".");
if !out.ok {self.display_fail(state.consumed.max(state.matched),"'.'");}
out
}
fn e432_c(&mut self,state:State)->Step {
let mut out=self.b432_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.relabel_token(out.events,432,64);
out.events=self.event(Event::Capture {site:72,span:[state.consumed,out.state.consumed],child:out.events,token_extent:false});
} else {self.display_failures(state.consumed.max(state.matched),&["IDENTIFIERParser", "__CaptureSite"]);}
out
}
fn b432_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.token_2::<false>(out.state,"IDENTIFIER","IDENTIFIERParser");
out
}
fn e433_c(&mut self,state:State)->Step {
let mark=self.mark();
let key=Key {expression:433,state,matched_mode:false,version:self.scope.state_version()};if let Some(hit)=self.lookup(key) {return hit;}
self.display_enter(state.consumed.max(state.matched));let mut out=self.b433_c(state);self.display_reach(out.state.consumed.max(out.state.matched));
if out.ok {
out.events=self.event(Event::Values{span:[state.consumed,out.state.consumed],child:out.events,wrap:true,text:false});
} else {self.restore(mark);self.display_failures(state.consumed.max(state.matched),&[]);}
if (out.ok && true) || (!out.ok && true) {self.store(key,out,mark);}self.display_leave();
out
}
fn b433_c(&mut self,mut state:State)->Step {
state.reset=false;
let mut out=Step::yes(state);
out.state=state.begin();
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
let child=self.e434_c(out.state);out=self.combine(out,child);if !out.ok {out.state=state;return out;}
let trivia=self.trivia_0::<false>(out.state);out=self.combine(out,trivia);
out.state=if out.ok {state.commit(out.state)} else {state};
out
}
fn e434_c(&mut self,state:State)->Step {
let mut out=self.b434_c(state);
if out.ok {
} else {self.display_failures(state.consumed.max(state.matched),&[]);}
out
}
fn b434_c(&mut self,mut state:State)->Step {
let mut out=Step::yes(state);
out=self.r49_c(out.state);
out
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
fn guard_e45_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e45_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"=");
self.display_failures(t1.consumed.max(t1.matched),&["'='", "'='"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'='"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e48_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e48_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"::=");
self.display_failures(t1.consumed.max(t1.matched),&["'::='", "'::='"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'::='"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e73_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e73_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e86_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e86_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"(");
self.display_failures(t1.consumed.max(t1.matched),&["'('", "'('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e90_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e90_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"[");
self.display_failures(t1.consumed.max(t1.matched),&["'['", "'['"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'['", "']'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e94_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e94_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e98_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e98_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let mut d2=Diag::NONE;
let d3=self.diag_fail(t1.begin().position::<false>(),"LOOKAHEAD");
d2=self.diag_join(d2,d3);
let d4=self.diag_fail(t1.begin().position::<false>(),"NEGATIVE_LOOKAHEAD");
d2=self.diag_join(d2,d4);
self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);
if t1.begin().consumed.max(t1.begin().matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "'LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e106_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e106_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CAPTURE");
self.display_failures(t1.consumed.max(t1.matched),&["'CAPTURE'", "'CAPTURE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "','", "'CAPTURE'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e113_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e113_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"SAME_AS");
self.display_failures(t1.consumed.max(t1.matched),&["'SAME_AS'", "'SAME_AS'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "'SAME_AS'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e118_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e118_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CHAR_RANGE");
self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'CHAR_RANGE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "','", "'CHAR_RANGE'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e125_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e125_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATION");
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'NEGATION'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'", "'NEGATION'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e132_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e132_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"UNTIL");
let d3=self.diag_rule(16,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'UNTIL'", "'UNTIL'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'UNTIL'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e133_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e133_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATION");
let d3=self.diag_rule(17,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'NEGATION'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'NEGATION'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e134_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e134_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"LOOKAHEAD");
let d3=self.diag_rule(18,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'LOOKAHEAD'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'LOOKAHEAD'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e135_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e135_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"NEGATIVE_LOOKAHEAD");
let d3=self.diag_rule(19,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'NEGATIVE_LOOKAHEAD'", "'NEGATIVE_LOOKAHEAD'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'NEGATIVE_LOOKAHEAD'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e136_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e136_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CHAR_RANGE");
let d3=self.diag_rule(20,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'CHAR_RANGE'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CHAR_RANGE'", "'('", "','", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e137_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e137_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"CI");
let d3=self.diag_rule(21,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'CI'", "'CI'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'CI'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e138_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e138_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"REGEX");
let d3=self.diag_rule(22,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'REGEX'", "'REGEX'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'REGEX'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e139_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e139_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ADAPTER");
let d3=self.diag_rule(23,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ADAPTER'", "'ADAPTER'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ADAPTER'", "'('", "','", "'version'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e140_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e140_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ANY");
let d3=self.diag_rule(24,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ANY'", "'ANY'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ANY'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e141_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e141_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"EOF");
let d3=self.diag_rule(25,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'EOF'", "'EOF'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'EOF'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e142_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e142_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"EMPTY");
let d3=self.diag_rule(26,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'EMPTY'", "'EMPTY'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'EMPTY'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e204_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e204_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@root");
let d3=self.diag_rule(29,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@root'", "'@root'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@root'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e205_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e205_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@mapping");
let d3=self.diag_rule(30,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@mapping'", "'@mapping'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@mapping'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e206_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e206_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@eval");
let d3=self.diag_rule(31,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@eval'", "'@eval'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@eval'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e207_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e207_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@whitespace");
let d3=self.diag_rule(32,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@whitespace'", "'@whitespace'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@whitespace'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e208_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e208_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@interleave");
let d3=self.diag_rule(33,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@interleave'", "'@interleave'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@interleave'", "'('", "'profile'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e209_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e209_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@backref");
let d3=self.diag_rule(34,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@backref'", "'@backref'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@backref'", "'('", "'name'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e210_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e210_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@scopeTree");
let d3=self.diag_rule(35,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@scopeTree'", "'@scopeTree'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@scopeTree'", "'('", "'mode'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e211_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e211_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@declares");
let d3=self.diag_rule(36,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@declares'", "'@declares'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@declares'", "'('", "'symbol'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e212_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e212_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@catalog");
let d3=self.diag_rule(37,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@catalog'", "'@catalog'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@catalog'", "'('", "'context'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e213_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e213_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@leftAssoc");
let d3=self.diag_rule(38,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@leftAssoc'", "'@leftAssoc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@leftAssoc'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e214_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e214_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@rightAssoc");
let d3=self.diag_rule(39,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@rightAssoc'", "'@rightAssoc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@rightAssoc'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e215_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e215_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@longestChoice");
let d3=self.diag_rule(40,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@longestChoice'", "'@longestChoice'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@longestChoice'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e216_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e216_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@predictiveChoice");
let d3=self.diag_rule(41,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@predictiveChoice'", "'@predictiveChoice'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@predictiveChoice'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e217_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e217_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@precedence");
let d3=self.diag_rule(42,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@precedence'", "'@precedence'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@precedence'", "'('", "'level'", "'='", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e218_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e218_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@doc");
let d3=self.diag_rule(43,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@doc'", "'@doc'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@doc'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e219_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e219_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@recovery");
let d3=self.diag_rule(44,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@recovery'", "'@recovery'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@recovery'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e220_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e220_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@skip");
let d3=self.diag_rule(45,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@skip'", "'@skip'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@skip'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e221_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e221_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@commonField");
let d3=self.diag_rule(46,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@commonField'", "'@commonField'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@commonField'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e222_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e222_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@enum");
let d3=self.diag_rule(47,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@enum'", "'@enum'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@enum'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e223_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e223_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"@");
let d3=self.diag_rule(48,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'@'", "'@'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'@'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e325_c(&mut self,state:State)->Step {
if !self.can_replay(0) {return self.e325_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"sync");
self.display_failures(t1.consumed.max(t1.matched),&["'sync'", "'sync'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'='", "'sync'"]);}
Step {ok:false,state,events:EventId(0),diag:d2}
}
fn guard_e391_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e391_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"(");
let d3=self.diag_rule(57,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'('", "'('"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e392_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e392_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"[");
let d3=self.diag_rule(58,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'['", "'['"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'['", "']'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e393_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e393_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"{");
let d3=self.diag_rule(59,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'{'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'{'", "'}'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn guard_e394_c(&mut self,state:State)->Step {
if !self.can_replay(1) {return self.e394_c(state);}
if !DIAG {return Step {ok:false,state,events:EventId(0),diag:Diag::NONE};}
self.display_reach(state.consumed.max(state.matched));
let t1=self.trivia_0::<false>(state.entered(true,true)).state;
let d2=self.diag_fail(t1.position::<false>(),"ERROR");
let d3=self.diag_rule(60,d2);
self.display_failures(t1.consumed.max(t1.matched),&["'ERROR'", "'ERROR'"]);
if state.consumed.max(state.matched)==t1.consumed.max(t1.matched) {self.display_failures(t1.consumed.max(t1.matched),&["'ERROR'", "'('", "')'"]);}
Step {ok:false,state,events:EventId(0),diag:d3}
}
fn token_0<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:48,max:57,children:&[]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_1<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_2<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::REPEAT,text:"",min:0,max:-1,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:48,max:57,children:&[]}]}]}]}]}]}]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_3<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"'",min:0,max:0,children:&[]},lexical_program::LexicalExpression {op:lexical_program::Op::REPEAT,text:"",min:0,max:-1,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"\\",min:0,max:0,children:&[]},lexical_program::LexicalExpression {op:lexical_program::Op::ANY,text:"",min:0,max:0,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::EXCEPT,text:"'\\",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"'",min:0,max:0,children:&[]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_4<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::REPEAT,text:"",min:0,max:-1,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:97,max:122,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:65,max:90,children:&[]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::LITERAL,text:"_",min:0,max:0,children:&[]}]}]}]}]},lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:48,max:57,children:&[]}]}]}]}]}]}]}]}]}]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
};if !out.ok {self.display_fail(state.consumed.max(state.matched),display);}out}
fn token_5<const MATCH:bool>(&mut self,state:State,label:&'static str,display:&'static str)->Step {let _=label;let out={
let start=state.position::<MATCH>();let end=lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::REPEAT,text:"",min:1,max:-1,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SCOPE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::CHOICE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::SEQUENCE,text:"",min:0,max:0,children:&[lexical_program::LexicalExpression {op:lexical_program::Op::RANGE,text:"",min:48,max:57,children:&[]}]}]}]}]}]}]}]}.match_at(self.text,start);let end=if state.invert {if end.is_none() {Some(start)} else {None}} else {end};match end {Some(end)=>{let mut next=state;if end>start {next.advance::<MATCH>(end-start);}let mut step=Step::yes(next);step.events=self.event(Event::Token {text_span:None,content_span:None,expr:usize::MAX,rule:usize::MAX,span:[start,end]});step},None=>self.fail(state,start,label)}
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
